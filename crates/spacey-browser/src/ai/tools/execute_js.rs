//! Execute JavaScript Tool - Run JavaScript in page context

use super::ToolResult;

/// Execute JavaScript tool
pub struct ExecuteJsTool;

impl ExecuteJsTool {
    /// Execute JavaScript code
    pub fn execute(code: &str, js_engine: Option<&dyn JsExecutor>) -> ToolResult {
        log::debug!("Executing JavaScript: {}...", &code[..code.len().min(50)]);

        if code.is_empty() {
            return ToolResult::error("JavaScript code cannot be empty");
        }

        // Validate the code doesn't contain dangerous patterns
        if let Err(e) = Self::validate_code(code) {
            return ToolResult::error(e);
        }

        // Execute the code if an engine is provided
        if let Some(engine) = js_engine {
            match engine.eval(code) {
                Ok(result) => ToolResult::success_with_data(
                    "JavaScript executed successfully",
                    serde_json::json!({
                        "result": result,
                        "code": code
                    }),
                )
                .with_step_complete(),
                Err(e) => ToolResult::error(format!("JavaScript error: {}", e)),
            }
        } else {
            // No engine provided, just validate and return the code
            ToolResult::success_with_data(
                "JavaScript code prepared for execution",
                serde_json::json!({
                    "code": code
                }),
            )
            .with_step_complete()
        }
    }

    /// Validate that the code doesn't contain dangerous patterns
    fn validate_code(code: &str) -> Result<(), String> {
        // List of potentially dangerous patterns
        let dangerous_patterns = [
            "eval(",
            "Function(",
            "document.cookie",
            "localStorage",
            "sessionStorage",
            "XMLHttpRequest",
            "fetch(",
            "import(",
            "require(",
            "__proto__",
            "constructor[",
        ];

        for pattern in &dangerous_patterns {
            if code.contains(pattern) {
                return Err(format!(
                    "Code contains potentially dangerous pattern: '{}'",
                    pattern
                ));
            }
        }

        // Check for excessive length
        if code.len() > 10000 {
            return Err("Code exceeds maximum length of 10000 characters".to_string());
        }

        Ok(())
    }

    /// Execute a safe, predefined JavaScript snippet
    pub fn execute_safe(snippet: SafeJsSnippet) -> ToolResult {
        let code = match snippet {
            SafeJsSnippet::GetTitle => "document.title",
            SafeJsSnippet::GetUrl => "window.location.href",
            SafeJsSnippet::GetSelectedText => "window.getSelection()?.toString() || ''",
            SafeJsSnippet::GetScrollPosition => {
                "JSON.stringify({ x: window.scrollX, y: window.scrollY })"
            }
            SafeJsSnippet::GetViewportSize => {
                "JSON.stringify({ width: window.innerWidth, height: window.innerHeight })"
            }
            SafeJsSnippet::GetElementCount(selector) => {
                return ToolResult::success_with_data(
                    "Element count query prepared",
                    serde_json::json!({
                        "code": format!("document.querySelectorAll('{}').length", selector.replace('\'', "\\'"))
                    }),
                )
                .with_step_complete();
            }
        };

        ToolResult::success_with_data(
            "Safe JavaScript snippet prepared",
            serde_json::json!({ "code": code }),
        )
        .with_step_complete()
    }
}

/// Trait for JavaScript execution
pub trait JsExecutor {
    fn eval(&self, code: &str) -> Result<String, String>;
}

/// Predefined safe JavaScript snippets
pub enum SafeJsSnippet {
    GetTitle,
    GetUrl,
    GetSelectedText,
    GetScrollPosition,
    GetViewportSize,
    GetElementCount(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_simple() {
        let result = ExecuteJsTool::execute("1 + 1", None);
        assert!(result.success);
    }

    #[test]
    fn test_empty_code() {
        let result = ExecuteJsTool::execute("", None);
        assert!(!result.success);
    }

    #[test]
    fn test_dangerous_code() {
        let result = ExecuteJsTool::execute("eval('alert(1)')", None);
        assert!(!result.success);
        assert!(result.message.contains("dangerous"));
    }

    #[test]
    fn test_safe_snippets() {
        let result = ExecuteJsTool::execute_safe(SafeJsSnippet::GetTitle);
        assert!(result.success);

        let result = ExecuteJsTool::execute_safe(SafeJsSnippet::GetElementCount("#button".to_string()));
        assert!(result.success);
    }
}
