//! Scroll Tool - Scroll the page

use super::{Direction, ToolResult};

/// Scroll tool for page scrolling
pub struct ScrollTool;

impl ScrollTool {
    /// Execute a scroll action
    pub fn execute(direction: Direction, amount: i32) -> ToolResult {
        log::debug!("Scrolling {:?} by {} pixels", direction, amount);

        if amount <= 0 {
            return ToolResult::error("Scroll amount must be positive");
        }

        if amount > 10000 {
            return ToolResult::error("Scroll amount too large (max 10000)");
        }

        // Generate the JavaScript for scrolling
        let (x, y) = match direction {
            Direction::Up => (0, -amount),
            Direction::Down => (0, amount),
            Direction::Left => (-amount, 0),
            Direction::Right => (amount, 0),
        };

        ToolResult::success_with_data(
            format!("Scrolled {:?} by {} pixels", direction, amount),
            serde_json::json!({
                "direction": format!("{:?}", direction).to_lowercase(),
                "amount": amount,
                "scroll_x": x,
                "scroll_y": y,
                "js_code": format!("window.scrollBy({}, {})", x, y)
            }),
        )
        .with_step_complete()
    }

    /// Scroll to a specific element
    pub fn scroll_to_element(selector: &str) -> ToolResult {
        log::debug!("Scrolling to element: {}", selector);

        if selector.is_empty() {
            return ToolResult::error("Selector cannot be empty");
        }

        ToolResult::success_with_data(
            format!("Scrolled to element '{}'", selector),
            serde_json::json!({
                "selector": selector,
                "js_code": format!(
                    "document.querySelector('{}')?.scrollIntoView({{ behavior: 'smooth', block: 'center' }})",
                    selector.replace('\'', "\\'")
                )
            }),
        )
        .with_step_complete()
    }

    /// Scroll to top of page
    pub fn scroll_to_top() -> ToolResult {
        ToolResult::success_with_data(
            "Scrolled to top of page",
            serde_json::json!({
                "js_code": "window.scrollTo({ top: 0, behavior: 'smooth' })"
            }),
        )
        .with_step_complete()
    }

    /// Scroll to bottom of page
    pub fn scroll_to_bottom() -> ToolResult {
        ToolResult::success_with_data(
            "Scrolled to bottom of page",
            serde_json::json!({
                "js_code": "window.scrollTo({ top: document.body.scrollHeight, behavior: 'smooth' })"
            }),
        )
        .with_step_complete()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_down() {
        let result = ScrollTool::execute(Direction::Down, 300);
        assert!(result.success);
    }

    #[test]
    fn test_scroll_up() {
        let result = ScrollTool::execute(Direction::Up, 200);
        assert!(result.success);
    }

    #[test]
    fn test_scroll_invalid_amount() {
        let result = ScrollTool::execute(Direction::Down, 0);
        assert!(!result.success);

        let result = ScrollTool::execute(Direction::Down, 20000);
        assert!(!result.success);
    }

    #[test]
    fn test_scroll_to_element() {
        let result = ScrollTool::scroll_to_element("#footer");
        assert!(result.success);
    }

    #[test]
    fn test_scroll_to_top() {
        let result = ScrollTool::scroll_to_top();
        assert!(result.success);
    }

    #[test]
    fn test_scroll_to_bottom() {
        let result = ScrollTool::scroll_to_bottom();
        assert!(result.success);
    }
}
