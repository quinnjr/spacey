//! Page - represents a loaded web page with HTML content and JavaScript execution

use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{RcDom, NodeData, Handle};
use spacey_servo::SpaceyServo;

pub struct Page {
    title: String,
    content: String,
    dom: RcDom,
    js_engine: SpaceyServo,
}

impl Page {
    pub fn from_html(html: &str, js_engine: &SpaceyServo) -> Self {
        // Parse HTML
        let dom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut html.as_bytes())
            .unwrap();

        let title = Self::extract_title(&dom);
        let content = Self::extract_text_content(&dom.document);

        // Execute any inline scripts
        Self::execute_scripts(&dom, js_engine);

        Self {
            title,
            content,
            dom,
            js_engine: js_engine.clone(),
        }
    }
    
    /// Inject and execute a script in this page's context
    pub fn inject_script(&self, script: &str) -> Result<(), String> {
        log::debug!("Injecting script: {} bytes", script.len());
        self.js_engine.eval(script).map(|_| ())
    }

    fn extract_title(dom: &RcDom) -> String {
        Self::find_title_node(&dom.document)
            .unwrap_or_else(|| "Untitled".to_string())
    }

    fn find_title_node(node: &Handle) -> Option<String> {
        match &node.data {
            NodeData::Element { name, .. } if name.local.as_ref() == "title" => {
                // Get text content of title
                for child in node.children.borrow().iter() {
                    if let NodeData::Text { contents } = &child.data {
                        return Some(contents.borrow().to_string());
                    }
                }
                None
            }
            _ => {
                // Recursively search children
                for child in node.children.borrow().iter() {
                    if let Some(title) = Self::find_title_node(child) {
                        return Some(title);
                    }
                }
                None
            }
        }
    }

    fn extract_text_content(node: &Handle) -> String {
        let mut text = String::new();
        
        match &node.data {
            NodeData::Text { contents } => {
                text.push_str(&contents.borrow());
            }
            NodeData::Element { name, .. } => {
                let tag = name.local.as_ref();
                
                // Add newlines for block elements
                if matches!(tag, "p" | "div" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "li") {
                    text.push('\n');
                }
                
                for child in node.children.borrow().iter() {
                    text.push_str(&Self::extract_text_content(child));
                }
                
                if matches!(tag, "p" | "div" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6") {
                    text.push('\n');
                }
            }
            _ => {
                for child in node.children.borrow().iter() {
                    text.push_str(&Self::extract_text_content(child));
                }
            }
        }
        
        text
    }

    fn execute_scripts(dom: &RcDom, js_engine: &SpaceyServo) {
        Self::find_and_execute_scripts(&dom.document, js_engine);
    }

    fn find_and_execute_scripts(node: &Handle, js_engine: &SpaceyServo) {
        if let NodeData::Element { name, .. } = &node.data {
            if name.local.as_ref() == "script" {
                // Get script content
                for child in node.children.borrow().iter() {
                    if let NodeData::Text { contents } = &child.data {
                        let script = contents.borrow().to_string();
                        log::info!("Executing inline script: {} bytes", script.len());
                        
                        match js_engine.eval(&script) {
                            Ok(result) => log::debug!("Script result: {}", result),
                            Err(e) => log::error!("Script error: {}", e),
                        }
                    }
                }
            }
        }

        // Recursively search children
        for child in node.children.borrow().iter() {
            Self::find_and_execute_scripts(child, js_engine);
        }
    }

    pub fn render_ui(&self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading(&self.title);
                ui.separator();
                
                // Render content
                self.render_node(&self.dom.document, ui);
            });
        });
    }

    fn render_node(&self, node: &Handle, ui: &mut egui::Ui) {
        match &node.data {
            NodeData::Element { name, .. } => {
                let tag = name.local.as_ref();
                
                match tag {
                    "h1" => {
                        for child in node.children.borrow().iter() {
                            if let NodeData::Text { contents } = &child.data {
                                ui.heading(contents.borrow().to_string());
                            }
                        }
                    }
                    "h2" => {
                        for child in node.children.borrow().iter() {
                            if let NodeData::Text { contents } = &child.data {
                                ui.label(
                                    egui::RichText::new(contents.borrow().to_string())
                                        .heading()
                                        .size(20.0),
                                );
                            }
                        }
                    }
                    "h3" => {
                        for child in node.children.borrow().iter() {
                            if let NodeData::Text { contents } = &child.data {
                                ui.label(
                                    egui::RichText::new(contents.borrow().to_string())
                                        .heading()
                                        .size(18.0),
                                );
                            }
                        }
                    }
                    "p" => {
                        let mut text = String::new();
                        for child in node.children.borrow().iter() {
                            if let NodeData::Text { contents } = &child.data {
                                text.push_str(&contents.borrow());
                            }
                        }
                        if !text.trim().is_empty() {
                            ui.label(text);
                            ui.add_space(5.0);
                        }
                    }
                    "ul" | "ol" => {
                        for child in node.children.borrow().iter() {
                            self.render_node(child, ui);
                        }
                    }
                    "li" => {
                        let mut text = String::new();
                        for child in node.children.borrow().iter() {
                            if let NodeData::Text { contents } = &child.data {
                                text.push_str(&contents.borrow());
                            }
                        }
                        if !text.trim().is_empty() {
                            ui.label(format!("• {}", text.trim()));
                        }
                    }
                    "script" | "style" => {
                        // Skip script and style tags in rendering
                    }
                    _ => {
                        // Recursively render children
                        for child in node.children.borrow().iter() {
                            self.render_node(child, ui);
                        }
                    }
                }
            }
            NodeData::Text { contents } => {
                let text = contents.borrow().to_string();
                if !text.trim().is_empty() {
                    ui.label(text.trim());
                }
            }
            _ => {
                // Recursively render children
                for child in node.children.borrow().iter() {
                    self.render_node(child, ui);
                }
            }
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}
