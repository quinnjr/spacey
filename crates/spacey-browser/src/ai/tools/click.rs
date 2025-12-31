//! Click Tool - Click on page elements

use super::ToolResult;

/// Click tool for interacting with elements
pub struct ClickTool;

impl ClickTool {
    /// Execute a click action on an element
    pub fn execute(selector: &str, page_content: &str) -> ToolResult {
        log::debug!("Clicking element: {}", selector);

        // Validate selector format
        if selector.is_empty() {
            return ToolResult::error("Selector cannot be empty");
        }

        // In a real implementation, this would:
        // 1. Find the element in the DOM
        // 2. Simulate a click event
        // 3. Handle any resulting navigation or DOM changes

        // For now, we simulate the click and check if the element exists
        if Self::element_exists(selector, page_content) {
            ToolResult::success(format!("Clicked element matching '{}'", selector))
                .with_step_complete()
        } else {
            ToolResult::error(format!("No element found matching selector '{}'", selector))
        }
    }

    /// Check if an element matching the selector exists
    fn element_exists(selector: &str, page_content: &str) -> bool {
        // Simple heuristic checks for common selector patterns
        if selector.starts_with('#') {
            // ID selector
            let id = &selector[1..];
            page_content.contains(&format!("id=\"{}\"", id))
                || page_content.contains(&format!("id='{}'", id))
        } else if selector.starts_with('.') {
            // Class selector
            let class = &selector[1..];
            page_content.contains(&format!("class=\"{}", class))
                || page_content.contains(&format!("class='{}", class))
                || page_content.contains(&format!(" {} ", class))
        } else if selector.starts_with('[') && selector.ends_with(']') {
            // Attribute selector
            let attr = &selector[1..selector.len() - 1];
            page_content.contains(attr)
        } else {
            // Tag selector or complex selector - assume it might exist
            let tag = selector.split_whitespace().next().unwrap_or(selector);
            page_content.contains(&format!("<{}", tag))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_click_with_id() {
        let html = r#"<button id="submit">Submit</button>"#;
        let result = ClickTool::execute("#submit", html);
        assert!(result.success);
    }

    #[test]
    fn test_click_with_class() {
        let html = r#"<button class="btn primary">Click me</button>"#;
        let result = ClickTool::execute(".btn", html);
        assert!(result.success);
    }

    #[test]
    fn test_click_not_found() {
        let html = r#"<button>Click me</button>"#;
        let result = ClickTool::execute("#nonexistent", html);
        assert!(!result.success);
    }

    #[test]
    fn test_empty_selector() {
        let result = ClickTool::execute("", "");
        assert!(!result.success);
        assert!(result.message.contains("empty"));
    }
}
