//! Wait Tool - Wait for elements or conditions

use super::ToolResult;
use std::time::Duration;

/// Wait tool for waiting on page conditions
pub struct WaitTool;

impl WaitTool {
    /// Wait for an element to appear
    pub fn execute(selector: &str, timeout_ms: u64) -> ToolResult {
        log::debug!(
            "Waiting for element '{}' with timeout {}ms",
            selector,
            timeout_ms
        );

        if selector.is_empty() {
            return ToolResult::error("Selector cannot be empty");
        }

        if timeout_ms == 0 {
            return ToolResult::error("Timeout must be greater than 0");
        }

        if timeout_ms > 60000 {
            return ToolResult::error("Timeout too long (max 60000ms)");
        }

        // Generate JavaScript for waiting
        let js_code = Self::generate_wait_script(selector, timeout_ms);

        ToolResult::success_with_data(
            format!(
                "Waiting for element '{}' (timeout: {}ms)",
                selector, timeout_ms
            ),
            serde_json::json!({
                "selector": selector,
                "timeout_ms": timeout_ms,
                "js_code": js_code
            }),
        )
        .with_step_complete()
    }

    /// Wait for an element to be visible
    pub fn wait_for_visible(selector: &str, timeout_ms: u64) -> ToolResult {
        if selector.is_empty() {
            return ToolResult::error("Selector cannot be empty");
        }

        let js_code = format!(
            r#"
            new Promise((resolve, reject) => {{
                const timeout = {};
                const start = Date.now();
                const check = () => {{
                    const el = document.querySelector('{}');
                    if (el && el.offsetParent !== null) {{
                        resolve(true);
                    }} else if (Date.now() - start > timeout) {{
                        reject(new Error('Timeout waiting for visible element'));
                    }} else {{
                        requestAnimationFrame(check);
                    }}
                }};
                check();
            }})
            "#,
            timeout_ms,
            selector.replace('\'', "\\'")
        );

        ToolResult::success_with_data(
            format!("Waiting for '{}' to be visible", selector),
            serde_json::json!({
                "selector": selector,
                "condition": "visible",
                "timeout_ms": timeout_ms,
                "js_code": js_code
            }),
        )
        .with_step_complete()
    }

    /// Wait for an element to be hidden
    pub fn wait_for_hidden(selector: &str, timeout_ms: u64) -> ToolResult {
        if selector.is_empty() {
            return ToolResult::error("Selector cannot be empty");
        }

        let js_code = format!(
            r#"
            new Promise((resolve, reject) => {{
                const timeout = {};
                const start = Date.now();
                const check = () => {{
                    const el = document.querySelector('{}');
                    if (!el || el.offsetParent === null) {{
                        resolve(true);
                    }} else if (Date.now() - start > timeout) {{
                        reject(new Error('Timeout waiting for element to hide'));
                    }} else {{
                        requestAnimationFrame(check);
                    }}
                }};
                check();
            }})
            "#,
            timeout_ms,
            selector.replace('\'', "\\'")
        );

        ToolResult::success_with_data(
            format!("Waiting for '{}' to be hidden", selector),
            serde_json::json!({
                "selector": selector,
                "condition": "hidden",
                "timeout_ms": timeout_ms,
                "js_code": js_code
            }),
        )
        .with_step_complete()
    }

    /// Wait for a fixed duration
    pub fn wait_for_duration(duration_ms: u64) -> ToolResult {
        if duration_ms == 0 {
            return ToolResult::error("Duration must be greater than 0");
        }

        if duration_ms > 30000 {
            return ToolResult::error("Duration too long (max 30000ms)");
        }

        ToolResult::success_with_data(
            format!("Waiting for {}ms", duration_ms),
            serde_json::json!({
                "duration_ms": duration_ms,
                "js_code": format!("await new Promise(r => setTimeout(r, {}))", duration_ms)
            }),
        )
        .with_step_complete()
    }

    /// Wait for network idle
    pub fn wait_for_network_idle(timeout_ms: u64) -> ToolResult {
        let js_code = format!(
            r#"
            new Promise((resolve, reject) => {{
                const timeout = {};
                const start = Date.now();
                let lastActivity = Date.now();
                
                const observer = new PerformanceObserver((list) => {{
                    lastActivity = Date.now();
                }});
                observer.observe({{ entryTypes: ['resource'] }});
                
                const check = () => {{
                    if (Date.now() - lastActivity > 500) {{
                        observer.disconnect();
                        resolve(true);
                    }} else if (Date.now() - start > timeout) {{
                        observer.disconnect();
                        reject(new Error('Timeout waiting for network idle'));
                    }} else {{
                        setTimeout(check, 100);
                    }}
                }};
                check();
            }})
            "#,
            timeout_ms
        );

        ToolResult::success_with_data(
            "Waiting for network to be idle",
            serde_json::json!({
                "condition": "network_idle",
                "timeout_ms": timeout_ms,
                "js_code": js_code
            }),
        )
        .with_step_complete()
    }

    /// Generate JavaScript for waiting for an element
    fn generate_wait_script(selector: &str, timeout_ms: u64) -> String {
        format!(
            r#"
            new Promise((resolve, reject) => {{
                const timeout = {};
                const start = Date.now();
                const check = () => {{
                    const el = document.querySelector('{}');
                    if (el) {{
                        resolve(el);
                    }} else if (Date.now() - start > timeout) {{
                        reject(new Error('Timeout waiting for element'));
                    }} else {{
                        requestAnimationFrame(check);
                    }}
                }};
                check();
            }})
            "#,
            timeout_ms,
            selector.replace('\'', "\\'")
        )
    }

    /// Get duration from timeout in milliseconds
    pub fn get_duration(timeout_ms: u64) -> Duration {
        Duration::from_millis(timeout_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wait_for_element() {
        let result = WaitTool::execute("#button", 5000);
        assert!(result.success);
    }

    #[test]
    fn test_empty_selector() {
        let result = WaitTool::execute("", 5000);
        assert!(!result.success);
    }

    #[test]
    fn test_zero_timeout() {
        let result = WaitTool::execute("#button", 0);
        assert!(!result.success);
    }

    #[test]
    fn test_timeout_too_long() {
        let result = WaitTool::execute("#button", 100000);
        assert!(!result.success);
    }

    #[test]
    fn test_wait_for_visible() {
        let result = WaitTool::wait_for_visible("#modal", 3000);
        assert!(result.success);
    }

    #[test]
    fn test_wait_for_hidden() {
        let result = WaitTool::wait_for_hidden(".loading", 3000);
        assert!(result.success);
    }

    #[test]
    fn test_wait_for_duration() {
        let result = WaitTool::wait_for_duration(1000);
        assert!(result.success);

        let result = WaitTool::wait_for_duration(0);
        assert!(!result.success);
    }
}
