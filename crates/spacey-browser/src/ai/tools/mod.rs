//! Browser Tools - Actions the AI agent can perform
//!
//! This module defines all the tools available to the AI agent for
//! interacting with web pages.

mod click;
mod type_text;
mod navigate;
mod extract;
mod scroll;
mod execute_js;
mod wait;
mod screenshot;

pub use click::ClickTool;
pub use type_text::TypeTool;
pub use navigate::NavigateTool;
pub use extract::ExtractTool;
pub use scroll::ScrollTool;
pub use execute_js::ExecuteJsTool;
pub use wait::WaitTool;
pub use screenshot::{
    ScreenshotTool, ScreenshotRegion, ScreenshotFormat, 
    ScreenshotResult, ScreenshotCapture
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of a tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Whether the tool execution was successful
    pub success: bool,
    /// Output or error message
    pub message: String,
    /// Additional data returned by the tool
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    /// Whether this action completed the current step
    pub step_complete: bool,
}

impl ToolResult {
    /// Create a successful result
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: None,
            step_complete: false,
        }
    }

    /// Create a successful result with data
    pub fn success_with_data(message: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: Some(data),
            step_complete: false,
        }
    }

    /// Create an error result
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            data: None,
            step_complete: false,
        }
    }

    /// Mark this result as completing the current step
    pub fn with_step_complete(mut self) -> Self {
        self.step_complete = true;
        self
    }

    /// Check if the action completed successfully
    pub fn is_complete(&self) -> bool {
        self.success && self.step_complete
    }
}

/// Scroll direction
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// Format for extracted content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ExtractFormat {
    Text,
    Html,
    Markdown,
}

impl Default for ExtractFormat {
    fn default() -> Self {
        ExtractFormat::Text
    }
}

/// All available browser tools
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "tool", content = "params")]
#[serde(rename_all = "snake_case")]
pub enum BrowserTool {
    /// Click on an element
    Click { selector: String },
    /// Type text into an element
    Type { selector: String, text: String },
    /// Navigate to a URL
    Navigate { url: String },
    /// Extract content from elements
    Extract {
        selector: String,
        #[serde(default)]
        format: ExtractFormat,
    },
    /// Scroll the page
    Scroll {
        direction: Direction,
        #[serde(default = "default_scroll_amount")]
        amount: i32,
    },
    /// Execute JavaScript code
    ExecuteJs { code: String },
    /// Wait for an element to appear
    Wait {
        selector: String,
        #[serde(default = "default_timeout")]
        timeout_ms: u64,
    },
    /// Capture a screenshot of the page
    Screenshot {
        /// Region to capture: viewport, full_page, or element with selector
        #[serde(default)]
        region: ScreenshotRegion,
        /// Image format: png, jpeg, or webp
        #[serde(default)]
        format: ScreenshotFormat,
        /// Quality for jpeg/webp (1-100)
        #[serde(default = "default_quality")]
        quality: u8,
    },
}

fn default_quality() -> u8 {
    80
}

fn default_scroll_amount() -> i32 {
    300
}

fn default_timeout() -> u64 {
    5000
}

impl BrowserTool {
    /// Get the tool name
    pub fn name(&self) -> &'static str {
        match self {
            BrowserTool::Click { .. } => "click",
            BrowserTool::Type { .. } => "type",
            BrowserTool::Navigate { .. } => "navigate",
            BrowserTool::Extract { .. } => "extract",
            BrowserTool::Scroll { .. } => "scroll",
            BrowserTool::ExecuteJs { .. } => "execute_js",
            BrowserTool::Wait { .. } => "wait",
            BrowserTool::Screenshot { .. } => "screenshot",
        }
    }

    /// Parse a tool from JSON
    pub fn from_json(json: &serde_json::Value) -> Result<Self, String> {
        serde_json::from_value(json.clone())
            .map_err(|e| format!("Failed to parse tool: {}", e))
    }
    
    /// Create a viewport screenshot tool
    pub fn screenshot_viewport() -> Self {
        BrowserTool::Screenshot {
            region: ScreenshotRegion::Viewport,
            format: ScreenshotFormat::Png,
            quality: 80,
        }
    }
    
    /// Create a full page screenshot tool
    pub fn screenshot_full_page() -> Self {
        BrowserTool::Screenshot {
            region: ScreenshotRegion::FullPage,
            format: ScreenshotFormat::Png,
            quality: 80,
        }
    }
    
    /// Create an element screenshot tool
    pub fn screenshot_element(selector: impl Into<String>) -> Self {
        BrowserTool::Screenshot {
            region: ScreenshotRegion::Element { selector: selector.into() },
            format: ScreenshotFormat::Png,
            quality: 80,
        }
    }
}

/// Registry of available tools and their execution logic
pub struct ToolRegistry {
    tools: HashMap<String, ToolDescription>,
}

/// Description of a tool for the LLM
#[derive(Debug, Clone, Serialize)]
pub struct ToolDescription {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ParameterDescription>,
}

/// Description of a tool parameter
#[derive(Debug, Clone, Serialize)]
pub struct ParameterDescription {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub param_type: String,
}

impl ToolRegistry {
    /// Create a new tool registry with all available tools
    pub fn new() -> Self {
        let mut tools = HashMap::new();

        tools.insert(
            "click".to_string(),
            ToolDescription {
                name: "click".to_string(),
                description: "Click on an element matching the CSS selector".to_string(),
                parameters: vec![ParameterDescription {
                    name: "selector".to_string(),
                    description: "CSS selector for the element to click".to_string(),
                    required: true,
                    param_type: "string".to_string(),
                }],
            },
        );

        tools.insert(
            "type".to_string(),
            ToolDescription {
                name: "type".to_string(),
                description: "Type text into an input element".to_string(),
                parameters: vec![
                    ParameterDescription {
                        name: "selector".to_string(),
                        description: "CSS selector for the input element".to_string(),
                        required: true,
                        param_type: "string".to_string(),
                    },
                    ParameterDescription {
                        name: "text".to_string(),
                        description: "Text to type into the element".to_string(),
                        required: true,
                        param_type: "string".to_string(),
                    },
                ],
            },
        );

        tools.insert(
            "navigate".to_string(),
            ToolDescription {
                name: "navigate".to_string(),
                description: "Navigate to a URL".to_string(),
                parameters: vec![ParameterDescription {
                    name: "url".to_string(),
                    description: "The URL to navigate to".to_string(),
                    required: true,
                    param_type: "string".to_string(),
                }],
            },
        );

        tools.insert(
            "extract".to_string(),
            ToolDescription {
                name: "extract".to_string(),
                description: "Extract text content from elements matching a selector".to_string(),
                parameters: vec![
                    ParameterDescription {
                        name: "selector".to_string(),
                        description: "CSS selector for elements to extract from".to_string(),
                        required: true,
                        param_type: "string".to_string(),
                    },
                    ParameterDescription {
                        name: "format".to_string(),
                        description: "Output format: text, html, or markdown".to_string(),
                        required: false,
                        param_type: "string".to_string(),
                    },
                ],
            },
        );

        tools.insert(
            "scroll".to_string(),
            ToolDescription {
                name: "scroll".to_string(),
                description: "Scroll the page in a direction".to_string(),
                parameters: vec![
                    ParameterDescription {
                        name: "direction".to_string(),
                        description: "Direction to scroll: up, down, left, right".to_string(),
                        required: true,
                        param_type: "string".to_string(),
                    },
                    ParameterDescription {
                        name: "amount".to_string(),
                        description: "Pixels to scroll (default: 300)".to_string(),
                        required: false,
                        param_type: "integer".to_string(),
                    },
                ],
            },
        );

        tools.insert(
            "execute_js".to_string(),
            ToolDescription {
                name: "execute_js".to_string(),
                description: "Execute JavaScript code in the page context".to_string(),
                parameters: vec![ParameterDescription {
                    name: "code".to_string(),
                    description: "JavaScript code to execute".to_string(),
                    required: true,
                    param_type: "string".to_string(),
                }],
            },
        );

        tools.insert(
            "wait".to_string(),
            ToolDescription {
                name: "wait".to_string(),
                description: "Wait for an element to appear on the page".to_string(),
                parameters: vec![
                    ParameterDescription {
                        name: "selector".to_string(),
                        description: "CSS selector for the element to wait for".to_string(),
                        required: true,
                        param_type: "string".to_string(),
                    },
                    ParameterDescription {
                        name: "timeout_ms".to_string(),
                        description: "Maximum time to wait in milliseconds (default: 5000)".to_string(),
                        required: false,
                        param_type: "integer".to_string(),
                    },
                ],
            },
        );

        tools.insert(
            "screenshot".to_string(),
            ToolDescription {
                name: "screenshot".to_string(),
                description: "Capture a screenshot of the current page for visual analysis. Use this to see what's on the page, verify UI state, or capture visual information.".to_string(),
                parameters: vec![
                    ParameterDescription {
                        name: "region".to_string(),
                        description: "Region to capture: 'viewport' (visible area), 'full_page' (entire scrollable page), or { element: 'selector' } for a specific element".to_string(),
                        required: false,
                        param_type: "string|object".to_string(),
                    },
                    ParameterDescription {
                        name: "format".to_string(),
                        description: "Image format: 'png' (default, lossless), 'jpeg' (smaller), or 'webp' (modern)".to_string(),
                        required: false,
                        param_type: "string".to_string(),
                    },
                    ParameterDescription {
                        name: "quality".to_string(),
                        description: "Quality for jpeg/webp format (1-100, default: 80)".to_string(),
                        required: false,
                        param_type: "integer".to_string(),
                    },
                ],
            },
        );

        Self { tools }
    }

    /// Get tool descriptions as JSON for the LLM
    pub fn to_tool_descriptions(&self) -> String {
        serde_json::to_string_pretty(&self.tools.values().collect::<Vec<_>>())
            .unwrap_or_else(|_| "[]".to_string())
    }

    /// Get a specific tool description
    pub fn get_tool(&self, name: &str) -> Option<&ToolDescription> {
        self.tools.get(name)
    }

    /// List all tool names
    pub fn tool_names(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_result() {
        let result = ToolResult::success("Clicked element");
        assert!(result.success);
        assert!(!result.step_complete);

        let result = result.with_step_complete();
        assert!(result.is_complete());
    }

    #[test]
    fn test_browser_tool_parsing() {
        let json = serde_json::json!({
            "tool": "click",
            "params": { "selector": "#button" }
        });

        let tool = BrowserTool::from_json(&json);
        assert!(tool.is_ok());

        if let Ok(BrowserTool::Click { selector }) = tool {
            assert_eq!(selector, "#button");
        } else {
            panic!("Expected Click tool");
        }
    }

    #[test]
    fn test_tool_registry() {
        let registry = ToolRegistry::new();
        assert!(registry.get_tool("click").is_some());
        assert!(registry.get_tool("navigate").is_some());
        assert!(registry.get_tool("nonexistent").is_none());
    }
}
