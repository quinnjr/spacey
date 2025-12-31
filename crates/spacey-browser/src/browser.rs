//! Browser core - manages the window, rendering, and JavaScript execution

use std::sync::Arc;
use winit::window::Window;
use spacey_servo::SpaceyServo;

use crate::renderer::Renderer;
use crate::page::Page;

pub struct Browser {
    window: Arc<Window>,
    renderer: Renderer,
    js_engine: SpaceyServo,
    current_page: Option<Page>,
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
        <li>🚧 CSS support (coming soon)</li>
        <li>🚧 Full DOM API (in progress)</li>
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
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
        self.window.request_redraw();
    }

    pub fn render(&mut self) {
        if let Some(page) = &self.current_page {
            self.renderer.render(page);
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
        self.window.request_redraw();
    }

    pub fn execute_js(&mut self, code: &str) -> Result<String, String> {
        self.js_engine.eval(code)
    }

    pub fn window(&self) -> &Window {
        &self.window
    }
}
