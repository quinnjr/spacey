//! Type Tool - Type text into input elements

use super::ToolResult;

/// Type tool for entering text into form elements
pub struct TypeTool;

impl TypeTool {
    /// Execute a type action on an input element
    pub fn execute(selector: &str, text: &str, page_content: &str) -> ToolResult {
        log::debug!("Typing '{}' into element: {}", text, selector);

        // Validate inputs
        if selector.is_empty() {
            return ToolResult::error("Selector cannot be empty");
        }

        if text.is_empty() {
            return ToolResult::error("Text to type cannot be empty");
        }

        // Check if the element exists and is an input type
        if Self::is_input_element(selector, page_content) {
            ToolResult::success(format!(
                "Typed '{}' into element matching '{}'",
                text, selector
            ))
            .with_step_complete()
        } else {
            ToolResult::error(format!(
                "No input element found matching selector '{}'",
                selector
            ))
        }
    }

    /// Check if an element matching the selector is an input element
    fn is_input_element(selector: &str, page_content: &str) -> bool {
        // Check for common input elements
        let input_tags = ["<input", "<textarea", "<select", "contenteditable"];
        
        // For ID selectors, check if there's an input with that ID
        if selector.starts_with('#') {
            let id = &selector[1..];
            for tag in &input_tags {
                if page_content.contains(&format!("{}", tag))
                    && page_content.contains(&format!("id=\"{}\"", id))
                {
                    return true;
                }
            }
            // Check contenteditable
            if page_content.contains(&format!("id=\"{}\"", id))
                && page_content.contains("contenteditable")
            {
                return true;
            }
        }

        // For class selectors
        if selector.starts_with('.') {
            let class = &selector[1..];
            for tag in &input_tags {
                if page_content.contains(&format!("{}", tag))
                    && page_content.contains(&format!("class=\"{}", class))
                {
                    return true;
                }
            }
        }

        // For tag selectors
        let tag = selector.split_whitespace().next().unwrap_or(selector);
        if ["input", "textarea", "select"].contains(&tag) {
            return page_content.contains(&format!("<{}", tag));
        }

        // Default: assume it might be an input
        page_content.contains("<input") || page_content.contains("<textarea")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_into_input() {
        let html = r#"<input type="text" id="username" />"#;
        let result = TypeTool::execute("#username", "testuser", html);
        assert!(result.success);
    }

    #[test]
    fn test_type_into_textarea() {
        let html = r#"<textarea id="message"></textarea>"#;
        let result = TypeTool::execute("#message", "Hello world", html);
        assert!(result.success);
    }

    #[test]
    fn test_empty_selector() {
        let result = TypeTool::execute("", "text", "");
        assert!(!result.success);
    }

    #[test]
    fn test_empty_text() {
        let result = TypeTool::execute("#input", "", "<input id='input'/>");
        assert!(!result.success);
    }
}
