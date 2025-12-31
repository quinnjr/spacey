//! Navigate Tool - Navigate to URLs

use super::ToolResult;
use url::Url;

/// Navigate tool for browser navigation
pub struct NavigateTool;

impl NavigateTool {
    /// Execute a navigation action
    pub fn execute(url: &str) -> ToolResult {
        log::debug!("Navigating to: {}", url);

        // Validate the URL
        match Self::validate_url(url) {
            Ok(validated_url) => {
                ToolResult::success_with_data(
                    format!("Navigating to {}", validated_url),
                    serde_json::json!({ "url": validated_url.to_string() }),
                )
                .with_step_complete()
            }
            Err(e) => ToolResult::error(format!("Invalid URL '{}': {}", url, e)),
        }
    }

    /// Validate and normalize a URL
    fn validate_url(url: &str) -> Result<Url, String> {
        // Handle common URL patterns
        let url = url.trim();

        // If no scheme, try adding https://
        let url = if !url.contains("://") {
            format!("https://{}", url)
        } else {
            url.to_string()
        };

        // Parse the URL
        Url::parse(&url).map_err(|e| e.to_string())
    }

    /// Check if URL is safe to navigate to
    pub fn is_safe_url(url: &str) -> bool {
        if let Ok(parsed) = Url::parse(url) {
            // Allow http and https
            let scheme = parsed.scheme();
            if scheme != "http" && scheme != "https" {
                return false;
            }

            // Block localhost in production (optional)
            // let host = parsed.host_str().unwrap_or("");
            // if host == "localhost" || host == "127.0.0.1" {
            //     return false;
            // }

            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigate_valid_url() {
        let result = NavigateTool::execute("https://example.com");
        assert!(result.success);
    }

    #[test]
    fn test_navigate_without_scheme() {
        let result = NavigateTool::execute("example.com");
        assert!(result.success);
    }

    #[test]
    fn test_navigate_with_path() {
        let result = NavigateTool::execute("https://example.com/page?q=test");
        assert!(result.success);
    }

    #[test]
    fn test_is_safe_url() {
        assert!(NavigateTool::is_safe_url("https://example.com"));
        assert!(NavigateTool::is_safe_url("http://example.com"));
        assert!(!NavigateTool::is_safe_url("file:///etc/passwd"));
        assert!(!NavigateTool::is_safe_url("javascript:alert(1)"));
    }
}
