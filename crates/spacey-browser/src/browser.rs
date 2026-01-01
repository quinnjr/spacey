//! Browser core - manages the window, rendering, and JavaScript execution

use std::sync::Arc;
use std::path::PathBuf;
use winit::window::Window;
use spacey_servo::SpaceyServo;

use crate::ai::{AiAgent, AgentConfig, BrowserTool, PageContext, ToolResult};
use crate::ai::tools::{ClickTool, TypeTool, NavigateTool, ExtractTool, ScrollTool, WaitTool, ScreenshotTool, ScreenshotRegion, ScreenshotFormat, ScreenshotResult};
use crate::ai_ui::{AiUiState, AiPanelAction, ChatRole};
use crate::extensions::{ExtensionManager, ExtensionError, RequestDetails, ResourceType, RequestAction};
use crate::extensions_ui::{ExtensionsUiState, ExtensionsAction};
use crate::shield::{SpaceyShield, ShieldLevel, BlockReason};
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

    // Built-in privacy protection
    shield: SpaceyShield,
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

        // Initialize built-in privacy protection
        let shield = SpaceyShield::new();
        log::info!(
            "🛡️ Spacey Shield initialized with {} blocked domains",
            shield.blocked_domain_count()
        );

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
            shield,
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
        let shield_domains = self.shield.blocked_domain_count();
        let shield_level = match self.shield.level() {
            ShieldLevel::Off => "Off",
            ShieldLevel::Standard => "Standard",
            ShieldLevel::Strict => "Strict",
        };

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

    <h2>🛡️ Spacey Shield:</h2>
    <p><strong>Built-in privacy protection is ACTIVE!</strong></p>
    <ul>
        <li>Protection Level: <strong>{}</strong></li>
        <li>Blocking {} known ad/tracker domains</li>
        <li>Fingerprint protection enabled</li>
        <li>HTTPS upgrade enabled</li>
        <li>Tracking parameter stripping enabled</li>
    </ul>
    <p>Spacey Shield complements extensions like uBlock Origin - they work together!</p>
    <p>Shield handles domain blocking + fingerprinting, uBlock handles cosmetic rules + advanced filters.</p>

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

    <h2>🐛 Found a Bug?</h2>
    <p>Help us improve Spacey Browser! <a href="about:bugreport" style="color: #00d4ff;">Report a bug</a></p>

    <script>
        console.log("Hello from Spacey!");
        var x = 42;
        var y = 10;
        console.log("x + y =", x + y);
    </script>
</body>
</html>
        "#, ext_count, shield_level, shield_domains);

        self.current_page = Some(Page::from_html(&html, &self.js_engine));
        self.current_url = "about:welcome".to_string();
        self.update_ai_page_context();
    }

    /// Navigate to bug report page
    fn navigate_to_bugreport(&mut self) {
        let version = env!("CARGO_PKG_VERSION");
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;
        let shield_level = format!("{:?}", self.shield.level());
        let ext_count = self.extension_manager.list().len();

        let html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Bug Report - Spacey Browser</title>
    <style>
        * {{
            box-sizing: border-box;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        }}
        body {{
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
            color: #e4e4e4;
            min-height: 100vh;
        }}
        h1 {{
            color: #00d4ff;
            border-bottom: 2px solid #00d4ff;
            padding-bottom: 10px;
        }}
        h2 {{
            color: #7b68ee;
            margin-top: 30px;
        }}
        .form-group {{
            margin-bottom: 20px;
        }}
        label {{
            display: block;
            margin-bottom: 8px;
            color: #b8b8b8;
            font-weight: 500;
        }}
        input, select, textarea {{
            width: 100%;
            padding: 12px;
            border: 1px solid #3a3a5a;
            border-radius: 8px;
            background: #2a2a4a;
            color: #e4e4e4;
            font-size: 14px;
        }}
        input:focus, select:focus, textarea:focus {{
            outline: none;
            border-color: #00d4ff;
            box-shadow: 0 0 0 2px rgba(0, 212, 255, 0.2);
        }}
        textarea {{
            min-height: 120px;
            resize: vertical;
        }}
        .system-info {{
            background: #2a2a4a;
            padding: 15px;
            border-radius: 8px;
            margin-bottom: 20px;
            font-family: monospace;
            font-size: 13px;
        }}
        .system-info span {{
            color: #00d4ff;
        }}
        button {{
            background: linear-gradient(135deg, #00d4ff, #7b68ee);
            color: white;
            padding: 14px 32px;
            border: none;
            border-radius: 8px;
            font-size: 16px;
            font-weight: 600;
            cursor: pointer;
            transition: transform 0.2s, box-shadow 0.2s;
        }}
        button:hover {{
            transform: translateY(-2px);
            box-shadow: 0 4px 20px rgba(0, 212, 255, 0.4);
        }}
        .note {{
            background: rgba(123, 104, 238, 0.2);
            border-left: 4px solid #7b68ee;
            padding: 12px 16px;
            margin: 20px 0;
            border-radius: 0 8px 8px 0;
        }}
        .required {{
            color: #ff6b6b;
        }}
    </style>
</head>
<body>
    <h1>🐛 Report a Bug</h1>
    <p>Found an issue with Spacey Browser? Let us know and we'll fix it!</p>

    <div class="note">
        <strong>📧 Your report will be sent to:</strong> support@pegasusheavy.dev
    </div>

    <h2>System Information</h2>
    <div class="system-info">
        <div><span>Browser Version:</span> Spacey v{}</div>
        <div><span>Operating System:</span> {} ({})</div>
        <div><span>Shield Level:</span> {}</div>
        <div><span>Extensions Installed:</span> {}</div>
    </div>

    <form action="https://formsubmit.co/support@pegasusheavy.dev" method="POST">
        <!-- FormSubmit configuration -->
        <input type="hidden" name="_subject" value="[Spacey Bug Report] New Issue Reported">
        <input type="hidden" name="_captcha" value="false">
        <input type="hidden" name="_template" value="table">
        <input type="hidden" name="_next" value="about:bugreport-thanks">

        <!-- System info (hidden) -->
        <input type="hidden" name="Browser Version" value="Spacey v{}">
        <input type="hidden" name="Operating System" value="{} ({})">
        <input type="hidden" name="Shield Level" value="{}">
        <input type="hidden" name="Extensions Count" value="{}">

        <h2>Bug Details</h2>

        <div class="form-group">
            <label for="issue-type">Issue Type <span class="required">*</span></label>
            <select id="issue-type" name="Issue Type" required>
                <option value="">-- Select an issue type --</option>
                <option value="crash">💥 Crash / Freeze</option>
                <option value="rendering">🎨 Rendering Issue</option>
                <option value="javascript">⚡ JavaScript Error</option>
                <option value="extension">🧩 Extension Problem</option>
                <option value="shield">🛡️ Shield / Blocking Issue</option>
                <option value="ai">🤖 AI Assistant Issue</option>
                <option value="performance">🐢 Performance Problem</option>
                <option value="ui">🖼️ UI / UX Issue</option>
                <option value="other">📝 Other</option>
            </select>
        </div>

        <div class="form-group">
            <label for="summary">Summary <span class="required">*</span></label>
            <input type="text" id="summary" name="Summary" placeholder="Brief description of the issue" required>
        </div>

        <div class="form-group">
            <label for="url">URL where issue occurred (if applicable)</label>
            <input type="text" id="url" name="URL" placeholder="https://example.com">
        </div>

        <div class="form-group">
            <label for="steps">Steps to Reproduce <span class="required">*</span></label>
            <textarea id="steps" name="Steps to Reproduce" placeholder="1. Go to...&#10;2. Click on...&#10;3. Observe..." required></textarea>
        </div>

        <div class="form-group">
            <label for="expected">Expected Behavior</label>
            <textarea id="expected" name="Expected Behavior" placeholder="What should have happened?"></textarea>
        </div>

        <div class="form-group">
            <label for="actual">Actual Behavior <span class="required">*</span></label>
            <textarea id="actual" name="Actual Behavior" placeholder="What actually happened?" required></textarea>
        </div>

        <div class="form-group">
            <label for="additional">Additional Information</label>
            <textarea id="additional" name="Additional Info" placeholder="Any other details, error messages, screenshots URLs, etc."></textarea>
        </div>

        <div class="form-group">
            <label for="email">Your Email (optional, for follow-up)</label>
            <input type="email" id="email" name="Reporter Email" placeholder="you@example.com">
        </div>

        <button type="submit">📨 Submit Bug Report</button>
    </form>

    <div class="note" style="margin-top: 30px;">
        <strong>💡 Tip:</strong> For faster resolution, include as much detail as possible.
        Screenshots or screen recordings can be shared via links in the Additional Information field.
    </div>
</body>
</html>
        "#,
        version, os, arch, shield_level, ext_count,
        version, os, arch, shield_level, ext_count
        );

        self.current_page = Some(Page::from_html(&html, &self.js_engine));
        self.current_url = "about:bugreport".to_string();
        self.update_ai_page_context();
    }

    /// Navigate to bug report thank you page
    fn navigate_to_bugreport_thanks(&mut self) {
        let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Thank You - Spacey Browser</title>
    <style>
        * {
            box-sizing: border-box;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        }
        body {
            max-width: 600px;
            margin: 0 auto;
            padding: 40px 20px;
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
            color: #e4e4e4;
            min-height: 100vh;
            text-align: center;
        }
        .success-icon {
            font-size: 80px;
            margin-bottom: 20px;
        }
        h1 {
            color: #00d4ff;
            margin-bottom: 10px;
        }
        p {
            font-size: 18px;
            color: #b8b8b8;
            line-height: 1.6;
        }
        .card {
            background: #2a2a4a;
            padding: 30px;
            border-radius: 16px;
            margin-top: 30px;
        }
        a {
            color: #00d4ff;
            text-decoration: none;
        }
        a:hover {
            text-decoration: underline;
        }
        .back-link {
            display: inline-block;
            margin-top: 30px;
            background: linear-gradient(135deg, #00d4ff, #7b68ee);
            color: white;
            padding: 12px 24px;
            border-radius: 8px;
            font-weight: 600;
        }
        .back-link:hover {
            text-decoration: none;
            transform: translateY(-2px);
        }
    </style>
</head>
<body>
    <div class="success-icon">✅</div>
    <h1>Thank You!</h1>
    <p>Your bug report has been submitted successfully.</p>

    <div class="card">
        <h2 style="color: #7b68ee; margin-top: 0;">What happens next?</h2>
        <p>Our team will review your report and investigate the issue. If you provided your email, we'll follow up with any questions or updates.</p>
        <p style="margin-bottom: 0;"><strong>Typical response time:</strong> 24-48 hours</p>
    </div>

    <a href="about:welcome" class="back-link">← Back to Home</a>
</body>
</html>
        "#;

        self.current_page = Some(Page::from_html(html, &self.js_engine));
        self.current_url = "about:bugreport-thanks".to_string();
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
            BrowserTool::Screenshot { region, format, quality } => {
                Self::create_screenshot_result(&region, &format, quality)
            }
        };

        (result, nav_url)
    }
    
    /// Create a screenshot result (standalone method)
    fn create_screenshot_result(
        region: &crate::ai::tools::ScreenshotRegion,
        format: &crate::ai::tools::ScreenshotFormat,
        _quality: u8,
    ) -> ToolResult {
        use crate::ai::tools::{ScreenshotResult, ScreenshotRegion, ScreenshotFormat};
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Create a 1x1 transparent PNG as placeholder
        // TODO: Implement actual framebuffer capture using wgpu
        let placeholder_png = [
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
            0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
            0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
            0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41,
            0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
            0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00,
            0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
            0x42, 0x60, 0x82,
        ];
        
        let data = STANDARD.encode(&placeholder_png);
        
        let result = ScreenshotResult {
            data: data.clone(),
            format: format.clone(),
            width: 1,
            height: 1,
            size_bytes: placeholder_png.len(),
            timestamp,
        };
        
        let region_desc = match region {
            ScreenshotRegion::Viewport => "viewport".to_string(),
            ScreenshotRegion::FullPage => "full page".to_string(),
            ScreenshotRegion::Element { selector } => format!("element '{}'", selector),
            ScreenshotRegion::Region { x, y, width, height } => {
                format!("region {}x{} at ({},{})", width, height, x, y)
            }
        };
        
        ToolResult::success_with_data(
            format!("Captured {} screenshot", region_desc),
            serde_json::json!({
                "data_url": result.to_data_url(),
                "width": result.width,
                "height": result.height,
                "size_bytes": result.size_bytes,
                "format": format!("{:?}", result.format),
            }),
        )
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

    /// Create RequestDetails for a URL
    fn make_request_details(&self, url: &str, resource_type: ResourceType) -> RequestDetails {
        RequestDetails {
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
        }
    }

    /// Process a network request through extensions
    #[allow(dead_code)]
    fn process_request(&self, url: &str, resource_type: ResourceType) -> RequestAction {
        let details = self.make_request_details(url, resource_type);
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

        // Handle special about: pages
        match url {
            "about:welcome" | "about:home" => {
                self.navigate_to_welcome();
                return;
            }
            "about:bugreport" | "about:bug" | "about:report" => {
                self.navigate_to_bugreport();
                return;
            }
            "about:bugreport-thanks" => {
                self.navigate_to_bugreport_thanks();
                return;
            }
            _ => {}
        }

        // Check HTTPS upgrade first
        if let Some(upgraded_url) = self.shield.should_upgrade_https(url) {
            log::info!("🛡️ Shield upgrading to HTTPS: {}", upgraded_url);
            self.navigate(&upgraded_url);
            return;
        }

        // Check Spacey Shield (built-in protection)
        let details = self.make_request_details(url, ResourceType::MainFrame);
        if let Some(reason) = self.shield.should_block(&details) {
            log::info!("🛡️ Shield blocked: {} ({})", url, reason.description());
            let html = format!(r#"
<!DOCTYPE html>
<html>
<head><title>Blocked by Spacey Shield</title></head>
<body>
    <h1>🛡️ Blocked by Spacey Shield</h1>
    <p>This page was blocked by built-in privacy protection.</p>
    <p><strong>Reason:</strong> {}</p>
    <p><strong>URL:</strong> {}</p>
    <hr>
    <p>If you believe this is a mistake, you can:</p>
    <ul>
        <li>Add this site to Shield exceptions</li>
        <li>Temporarily disable Shield protection</li>
    </ul>
</body>
</html>"#, reason.description(), url);
            self.current_page = Some(Page::from_html(&html, &self.js_engine));
            self.current_url = "about:blocked".to_string();
            return;
        }

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
    <h1>🧩 Blocked by Extension</h1>
    <p>This page was blocked by a browser extension.</p>
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

    // ===== Spacey Shield Controls =====

    /// Get reference to Spacey Shield
    pub fn shield(&self) -> &SpaceyShield {
        &self.shield
    }

    /// Set Shield protection level
    pub fn set_shield_level(&self, level: ShieldLevel) {
        log::info!("🛡️ Shield level changed to: {:?}", level);
        self.shield.set_level(level);
    }

    /// Get Shield statistics
    pub fn shield_stats(&self) -> crate::shield::ShieldStats {
        self.shield.stats()
    }

    /// Add a site exception to Shield
    pub fn add_shield_exception(&self, domain: &str) {
        log::info!("🛡️ Shield exception added for: {}", domain);
        self.shield.add_exception(domain);
    }

    /// Remove a site exception from Shield
    pub fn remove_shield_exception(&self, domain: &str) {
        self.shield.remove_exception(domain);
    }
}
