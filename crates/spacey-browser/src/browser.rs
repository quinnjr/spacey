//! Browser core - manages the window, rendering, and JavaScript execution

use std::sync::Arc;
use std::path::PathBuf;
use winit::window::Window;
use spacey_servo::SpaceyServo;

use crate::ai::{AiAgent, AgentConfig, BrowserTool, PageContext, ToolResult};
use crate::ai::tools::{ClickTool, TypeTool, NavigateTool, ExtractTool, ScrollTool, WaitTool};
use crate::ai_ui::{AiUiState, AiPanelAction, ChatRole};
use crate::extensions::{ExtensionManager, ExtensionError, RequestDetails, ResourceType, RequestAction};
use crate::extensions_ui::{ExtensionsUiState, ExtensionsAction};
use crate::renderer::Renderer;
use crate::page::Page;

/// The main browser struct
pub struct Browser {
    window: Arc<Window>,
    renderer: Renderer,
    js_engine: SpaceyServo,
    current_page: Option<Page>,
    current_url: String,
    
    // AI components
    ai_agent: Option<AiAgent>,
    ai_ui_state: AiUiState,
    ai_task_running: bool,
    
    // Extension system
    extension_manager: ExtensionManager,
    extensions_ui_state: ExtensionsUiState,
}

impl Browser {
    pub fn new(window: Arc<Window>) -> Self {
        let renderer = Renderer::new(Arc::clone(&window));
        let js_engine = SpaceyServo::new();
        
        // Setup extension system data directory
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("spacey-browser");
        
        let extension_manager = ExtensionManager::new(data_dir);
        
        // Load a default page
        let mut browser = Self {
            window: Arc::clone(&window),
            renderer,
            js_engine,
            current_page: None,
            current_url: "about:welcome".to_string(),
            ai_agent: None,
            ai_ui_state: AiUiState::new(),
            ai_task_running: false,
            extension_manager,
            extensions_ui_state: ExtensionsUiState::new(),
        };
        
        // Initialize installed extensions
        if let Err(e) = browser.extension_manager.init() {
            log::warn!("Failed to load extensions: {:?}", e);
        }
        
        // Load the welcome page
        browser.navigate_to_welcome();
        
        browser
    }

    fn navigate_to_welcome(&mut self) {
        let ext_count = self.extension_manager.list().len();
        let html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Welcome to Spacey Browser</title>
</head>
<body>
    <h1>🚀 Welcome to Spacey Browser!</h1>
    <p>This is a minimal web browser powered by the Spacey JavaScript engine.</p>
    
    <h2>Features:</h2>
    <ul>
        <li>✅ Custom JavaScript engine (Spacey)</li>
        <li>✅ Basic HTML rendering</li>
        <li>✅ GPU-accelerated graphics (wgpu)</li>
        <li>✅ AI-powered browsing assistant (Phi-3)</li>
        <li>✅ Firefox-compatible extensions ({} installed)</li>
        <li>✅ Full webRequest API (ad blockers work!)</li>
        <li>🚧 CSS support (coming soon)</li>
        <li>🚧 Full DOM API (in progress)</li>
    </ul>
    
    <h2>Extensions:</h2>
    <p>Install extensions from the Firefox Add-ons Marketplace (AMO)!</p>
    <p>Unlike Chrome, we support FULL webRequest blocking for ad blockers like uBlock Origin.</p>
    
    <h2>AI Assistant:</h2>
    <p>Use the AI panel on the right to automate browsing tasks!</p>
    <p>Try commands like:</p>
    <ul>
        <li>"Search for Rust tutorials"</li>
        <li>"Navigate to github.com"</li>
        <li>"Extract all headings from this page"</li>
    </ul>
    
    <h2>Try some JavaScript:</h2>
    <p>Open the developer console to execute JavaScript with the Spacey engine.</p>
    
    <script>
        console.log("Hello from Spacey!");
        var x = 42;
        var y = 10;
        console.log("x + y =", x + y);
    </script>
</body>
</html>
        "#, ext_count);
        
        self.current_page = Some(Page::from_html(&html, &self.js_engine));
        self.current_url = "about:welcome".to_string();
        self.update_ai_page_context();
    }

    /// Enable AI assistant
    pub fn enable_ai(&mut self, config: AgentConfig) -> Result<(), String> {
        self.ai_ui_state.set_loading_message("Initializing AI agent...");
        self.ai_agent = Some(AiAgent::new(config));
        self.ai_ui_state.set_loading_message("AI agent created (model not yet loaded)");
        Ok(())
    }

    /// Load the AI model
    pub fn load_ai_model(&mut self) -> Result<(), String> {
        if let Some(agent) = &mut self.ai_agent {
            self.ai_ui_state.set_loading_message("Downloading and loading Phi-3 model...");
            match agent.load_model() {
                Ok(_) => {
                    self.ai_ui_state.set_model_loaded(true);
                    self.ai_ui_state.add_message(
                        ChatRole::System,
                        "AI model loaded successfully! I'm ready to help you browse.",
                    );
                    Ok(())
                }
                Err(e) => {
                    let msg = format!("Failed to load model: {}", e);
                    self.ai_ui_state.set_loading_message(&msg);
                    Err(msg)
                }
            }
        } else {
            // Initialize agent if not already done
            self.enable_ai(AgentConfig::default())?;
            self.load_ai_model()
        }
    }

    /// Execute a browser tool (standalone function to avoid borrow issues)
    fn execute_tool_impl(
        tool: BrowserTool,
        page_content: &str,
        js_engine: &SpaceyServo,
    ) -> (ToolResult, Option<String>) {
        let mut nav_url = None;
        
        let result = match tool {
            BrowserTool::Click { selector } => {
                ClickTool::execute(&selector, page_content)
            }
            BrowserTool::Type { selector, text } => {
                TypeTool::execute(&selector, &text, page_content)
            }
            BrowserTool::Navigate { ref url } => {
                let result = NavigateTool::execute(url);
                if result.success {
                    nav_url = Some(url.clone());
                }
                result
            }
            BrowserTool::Extract { selector, format } => {
                ExtractTool::execute(&selector, &format, page_content)
            }
            BrowserTool::Scroll { direction, amount } => {
                ScrollTool::execute(direction, amount)
            }
            BrowserTool::ExecuteJs { code } => {
                match js_engine.eval(&code) {
                    Ok(result) => ToolResult::success_with_data(
                        "JavaScript executed",
                        serde_json::json!({ "result": result }),
                    ),
                    Err(e) => ToolResult::error(format!("JS Error: {}", e)),
                }
            }
            BrowserTool::Wait { selector, timeout_ms } => {
                WaitTool::execute(&selector, timeout_ms)
            }
        };
        
        (result, nav_url)
    }

    /// Update AI agent with current page context
    fn update_ai_page_context(&mut self) {
        if let (Some(agent), Some(page)) = (&mut self.ai_agent, &self.current_page) {
            let context = PageContext {
                url: self.current_url.clone(),
                title: page.title().to_string(),
                text_content: page.content().chars().take(5000).collect(),
                interactive_elements: vec![], // TODO: extract from DOM
            };
            agent.set_page_context(context);
        }
    }
    
    /// Inject content scripts for the current URL
    fn inject_content_scripts(&mut self) {
        let scripts = self.extension_manager.get_content_scripts_for_url(&self.current_url);
        
        if let Some(page) = &self.current_page {
            for (ext, content_script) in scripts {
                log::info!(
                    "Injecting content script from {} for {}",
                    ext.manifest.name,
                    self.current_url
                );
                
                // Read and inject each JS file
                for js_file in &content_script.js {
                    if let Ok(script) = self.extension_manager
                        .loader()
                        .read()
                        .read_extension_file(&ext.id, js_file)
                    {
                        if let Ok(script_str) = String::from_utf8(script) {
                            let wrapped = self.extension_manager
                                .runtime()
                                .wrap_content_script(&ext.id, &script_str, "ISOLATED");
                            
                            if let Err(e) = page.inject_script(&wrapped) {
                                log::warn!("Failed to inject content script: {}", e);
                            }
                        }
                    }
                }
            }
        }
    }
    
    /// Process a network request through extensions
    #[allow(dead_code)]
    fn process_request(&self, url: &str, resource_type: ResourceType) -> RequestAction {
        let details = RequestDetails {
            request_id: self.extension_manager.runtime().webrequest().next_request_id(),
            url: url.to_string(),
            method: "GET".to_string(),
            frame_id: 0,
            parent_frame_id: -1,
            tab_id: 0,
            resource_type,
            time_stamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as f64,
            originator_url: Some(self.current_url.clone()),
            document_url: Some(self.current_url.clone()),
            request_headers: None,
            response_headers: None,
            status_code: None,
            status_line: None,
            request_body: None,
            third_party: !url.contains(&self.current_url),
        };
        
        self.extension_manager.process_request(&details)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
        self.window.request_redraw();
    }

    pub fn render(&mut self) {
        // Get installed extensions for the UI
        let installed = self.extension_manager.list();
        
        // Handle AI panel actions
        let (ai_action, ext_action) = self.renderer.render_with_panels(
            self.current_page.as_ref(),
            &mut self.ai_ui_state,
            &mut self.extensions_ui_state,
            &installed,
        );

        // Process any AI panel actions
        if let Some(action) = ai_action {
            match action {
                AiPanelAction::LoadModel => {
                    if let Err(e) = self.load_ai_model() {
                        log::error!("Failed to load AI model: {}", e);
                    }
                }
                AiPanelAction::ExecuteTask(task) => {
                    // For now, we'll just note the task - actual async execution
                    // would require a runtime
                    log::info!("AI task requested: {}", task);
                    self.ai_ui_state.add_message(
                        ChatRole::System,
                        "Task execution requires async runtime - coming soon!",
                    );
                }
                AiPanelAction::StopTask => {
                    self.ai_task_running = false;
                    self.ai_ui_state.is_running = false;
                }
            }
        }
        
        // Process extension UI actions
        if let Some(action) = ext_action {
            self.handle_extension_action(action);
        }
    }
    
    /// Handle extension UI actions
    fn handle_extension_action(&mut self, action: ExtensionsAction) {
        match action {
            ExtensionsAction::Search(query) => {
                self.extensions_ui_state.loading = true;
                match self.extension_manager.search_amo(&query) {
                    Ok(results) => {
                        self.extensions_ui_state.search_results = Some(results);
                        self.extensions_ui_state.loading = false;
                    }
                    Err(e) => {
                        self.extensions_ui_state.show_error(format!("{}", e));
                        self.extensions_ui_state.loading = false;
                    }
                }
            }
            ExtensionsAction::InstallFromAmo(slug) => {
                self.extensions_ui_state.installing = Some(slug.clone());
                match self.extension_manager.install_from_amo(&slug) {
                    Ok(id) => {
                        self.extensions_ui_state.show_success(
                            format!("Installed extension: {}", id)
                        );
                        self.extensions_ui_state.installing = None;
                    }
                    Err(e) => {
                        self.extensions_ui_state.show_error(format!("{}", e));
                        self.extensions_ui_state.installing = None;
                    }
                }
            }
            ExtensionsAction::InstallXpi(path) => {
                match self.extension_manager.install_xpi(&path) {
                    Ok(id) => {
                        self.extensions_ui_state.show_success(
                            format!("Installed extension: {}", id)
                        );
                    }
                    Err(e) => {
                        self.extensions_ui_state.show_error(format!("{}", e));
                    }
                }
            }
            ExtensionsAction::Uninstall(id) => {
                match self.extension_manager.uninstall(&id) {
                    Ok(_) => {
                        self.extensions_ui_state.show_success(
                            format!("Uninstalled extension: {}", id)
                        );
                    }
                    Err(e) => {
                        self.extensions_ui_state.show_error(format!("{}", e));
                    }
                }
            }
            ExtensionsAction::Enable(id) => {
                match self.extension_manager.enable(&id) {
                    Ok(_) => {
                        self.extensions_ui_state.show_success(
                            format!("Enabled extension: {}", id)
                        );
                    }
                    Err(e) => {
                        self.extensions_ui_state.show_error(format!("{}", e));
                    }
                }
            }
            ExtensionsAction::Disable(id) => {
                match self.extension_manager.disable(&id) {
                    Ok(_) => {
                        self.extensions_ui_state.show_success(
                            format!("Disabled extension: {}", id)
                        );
                    }
                    Err(e) => {
                        self.extensions_ui_state.show_error(format!("{}", e));
                    }
                }
            }
            ExtensionsAction::LoadFeatured => {
                self.extensions_ui_state.loading = true;
                match self.extension_manager.get_featured() {
                    Ok(results) => {
                        self.extensions_ui_state.featured = Some(results.results);
                        self.extensions_ui_state.loading = false;
                    }
                    Err(e) => {
                        self.extensions_ui_state.show_error(format!("{}", e));
                        self.extensions_ui_state.loading = false;
                    }
                }
            }
            ExtensionsAction::LoadBlockers => {
                self.extensions_ui_state.loading = true;
                match self.extension_manager.get_recommended_blockers() {
                    Ok(blockers) => {
                        self.extensions_ui_state.recommended_blockers = Some(blockers);
                        self.extensions_ui_state.loading = false;
                    }
                    Err(e) => {
                        self.extensions_ui_state.show_error(format!("{}", e));
                        self.extensions_ui_state.loading = false;
                    }
                }
            }
            ExtensionsAction::Reload(id) => {
                // Reload by disabling and re-enabling
                let _ = self.extension_manager.disable(&id);
                match self.extension_manager.enable(&id) {
                    Ok(_) => {
                        self.extensions_ui_state.show_success(
                            format!("Reloaded extension: {}", id)
                        );
                    }
                    Err(e) => {
                        self.extensions_ui_state.show_error(format!("{}", e));
                    }
                }
            }
        }
    }

    pub fn navigate(&mut self, url: &str) {
        log::info!("Navigating to: {}", url);
        
        // Check if any extension wants to block/redirect this navigation
        let action = self.process_request(url, ResourceType::MainFrame);
        
        match action {
            RequestAction::Cancel => {
                log::info!("Navigation blocked by extension: {}", url);
                let html = format!(r#"
<!DOCTYPE html>
<html>
<head><title>Blocked</title></head>
<body>
    <h1>🛡️ Request Blocked</h1>
    <p>This page was blocked by an extension.</p>
    <p>URL: {}</p>
</body>
</html>"#, url);
                self.current_page = Some(Page::from_html(&html, &self.js_engine));
                self.current_url = "about:blocked".to_string();
                return;
            }
            RequestAction::Redirect(new_url) => {
                log::info!("Extension redirecting {} to {}", url, new_url);
                self.navigate(&new_url);
                return;
            }
            _ => {}
        }
        
        // For now, just show a placeholder
        let html = format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <title>{}</title>
</head>
<body>
    <h1>Navigation</h1>
    <p>Requested URL: {}</p>
    <p>Full navigation support coming soon!</p>
</body>
</html>
            "#,
            url, url
        );
        
        self.current_url = url.to_string();
        self.current_page = Some(Page::from_html(&html, &self.js_engine));
        
        // Inject content scripts for this URL
        self.inject_content_scripts();
        
        self.update_ai_page_context();
        self.window.request_redraw();
    }

    pub fn execute_js(&mut self, code: &str) -> Result<String, String> {
        self.js_engine.eval(code)
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Check if AI is enabled
    pub fn is_ai_enabled(&self) -> bool {
        self.ai_agent.is_some()
    }

    /// Check if AI model is loaded
    pub fn is_ai_model_loaded(&self) -> bool {
        self.ai_agent
            .as_ref()
            .map(|a| a.is_model_loaded())
            .unwrap_or(false)
    }

    /// Toggle AI panel visibility
    pub fn toggle_ai_panel(&mut self) {
        self.ai_ui_state.toggle_panel();
    }
    
    /// Toggle extensions panel visibility
    pub fn toggle_extensions_panel(&mut self) {
        self.extensions_ui_state.toggle_panel();
    }
    
    /// Get the extension manager
    pub fn extension_manager(&self) -> &ExtensionManager {
        &self.extension_manager
    }
    
    /// Get the current URL
    pub fn current_url(&self) -> &str {
        &self.current_url
    }
}
