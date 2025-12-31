//! Browser core - manages the window, rendering, and JavaScript execution

use std::sync::Arc;
use winit::window::Window;
use spacey_servo::SpaceyServo;

use crate::ai::{AiAgent, AgentConfig, BrowserTool, PageContext, ToolResult};
use crate::ai::tools::{ClickTool, TypeTool, NavigateTool, ExtractTool, ScrollTool, WaitTool};
use crate::ai_ui::{AiUiState, AiPanelAction, ChatRole};
use crate::renderer::Renderer;
use crate::page::Page;

/// The main browser struct
pub struct Browser {
    window: Arc<Window>,
    renderer: Renderer,
    js_engine: SpaceyServo,
    current_page: Option<Page>,
    
    // AI components
    ai_agent: Option<AiAgent>,
    ai_ui_state: AiUiState,
    ai_task_running: bool,
}

impl Browser {
    pub fn new(window: Arc<Window>) -> Self {
        let renderer = Renderer::new(Arc::clone(&window));
        let js_engine = SpaceyServo::new();
        
        // Load a default page
        let mut browser = Self {
            window: Arc::clone(&window),
            renderer,
            js_engine,
            current_page: None,
            ai_agent: None,
            ai_ui_state: AiUiState::new(),
            ai_task_running: false,
        };
        
        // Load the welcome page
        browser.navigate_to_welcome();
        
        browser
    }

    fn navigate_to_welcome(&mut self) {
        let html = r#"
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
        <li>🚧 CSS support (coming soon)</li>
        <li>🚧 Full DOM API (in progress)</li>
    </ul>
    
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
        "#;
        
        self.current_page = Some(Page::from_html(html, &self.js_engine));
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
                url: "about:welcome".to_string(), // TODO: track actual URL
                title: page.title().to_string(),
                text_content: page.content().chars().take(5000).collect(),
                interactive_elements: vec![], // TODO: extract from DOM
            };
            agent.set_page_context(context);
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
        self.window.request_redraw();
    }

    pub fn render(&mut self) {
        // Handle AI panel actions
        let action = self.renderer.render_with_ai(
            self.current_page.as_ref(),
            &mut self.ai_ui_state,
        );

        // Process any AI panel actions
        if let Some(action) = action {
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
    }

    pub fn navigate(&mut self, url: &str) {
        log::info!("Navigating to: {}", url);
        
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
        
        self.current_page = Some(Page::from_html(&html, &self.js_engine));
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
}
