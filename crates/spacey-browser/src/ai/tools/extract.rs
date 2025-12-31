//! Extract Tool - Extract content from page elements

use super::{ExtractFormat, ToolResult};

/// Extract tool for getting content from the page
pub struct ExtractTool;

impl ExtractTool {
    /// Execute an extraction action
    pub fn execute(selector: &str, format: &ExtractFormat, page_content: &str) -> ToolResult {
        log::debug!("Extracting content from: {} (format: {:?})", selector, format);

        if selector.is_empty() {
            return ToolResult::error("Selector cannot be empty");
        }

        // Extract content based on selector
        match Self::extract_content(selector, page_content, format) {
            Some(content) if !content.is_empty() => {
                ToolResult::success_with_data(
                    format!("Extracted content from '{}'", selector),
                    serde_json::json!({
                        "content": content,
                        "format": format,
                        "selector": selector
                    }),
                )
                .with_step_complete()
            }
            Some(_) => ToolResult::error(format!(
                "Element '{}' found but contains no content",
                selector
            )),
            None => ToolResult::error(format!("No element found matching '{}'", selector)),
        }
    }

    /// Extract content from the page based on selector
    fn extract_content(selector: &str, page_content: &str, format: &ExtractFormat) -> Option<String> {
        // This is a simplified extraction - in a real implementation,
        // we would use proper DOM parsing and CSS selector matching

        // For ID selectors, try to find the element
        if selector.starts_with('#') {
            let id = &selector[1..];
            return Self::extract_by_id(id, page_content, format);
        }

        // For class selectors
        if selector.starts_with('.') {
            let class = &selector[1..];
            return Self::extract_by_class(class, page_content, format);
        }

        // For tag selectors
        Self::extract_by_tag(selector, page_content, format)
    }

    /// Extract content from an element by ID
    fn extract_by_id(id: &str, page_content: &str, format: &ExtractFormat) -> Option<String> {
        // Find the element with the ID
        let id_pattern = format!("id=\"{}\"", id);
        if !page_content.contains(&id_pattern) {
            return None;
        }

        // Very simplified extraction - find content after the ID
        if let Some(start_idx) = page_content.find(&id_pattern) {
            // Find the closing tag
            if let Some(gt_idx) = page_content[start_idx..].find('>') {
                let content_start = start_idx + gt_idx + 1;
                if let Some(lt_idx) = page_content[content_start..].find('<') {
                    let content = &page_content[content_start..content_start + lt_idx];
                    return Some(Self::format_content(content.trim(), format));
                }
            }
        }

        None
    }

    /// Extract content from elements by class
    fn extract_by_class(class: &str, page_content: &str, format: &ExtractFormat) -> Option<String> {
        let class_pattern = format!("class=\"{}", class);
        if !page_content.contains(&class_pattern) {
            return None;
        }

        // Collect all matching content
        let mut results = Vec::new();
        let mut search_start = 0;

        while let Some(idx) = page_content[search_start..].find(&class_pattern) {
            let actual_idx = search_start + idx;
            
            // Find the closing > of the opening tag
            if let Some(gt_idx) = page_content[actual_idx..].find('>') {
                let content_start = actual_idx + gt_idx + 1;
                
                // Find the next < (start of closing or nested tag)
                if let Some(lt_idx) = page_content[content_start..].find('<') {
                    let content = &page_content[content_start..content_start + lt_idx];
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        results.push(trimmed.to_string());
                    }
                }
            }

            search_start = actual_idx + class_pattern.len();
        }

        if results.is_empty() {
            None
        } else {
            Some(Self::format_content(&results.join("\n"), format))
        }
    }

    /// Extract content from elements by tag name
    fn extract_by_tag(tag: &str, page_content: &str, format: &ExtractFormat) -> Option<String> {
        let open_tag = format!("<{}", tag);
        if !page_content.contains(&open_tag) {
            return None;
        }

        let mut results = Vec::new();
        let mut search_start = 0;

        while let Some(idx) = page_content[search_start..].find(&open_tag) {
            let actual_idx = search_start + idx;

            // Find the closing > of the opening tag
            if let Some(gt_idx) = page_content[actual_idx..].find('>') {
                let content_start = actual_idx + gt_idx + 1;

                // Find the closing tag
                let close_tag = format!("</{}>", tag);
                if let Some(close_idx) = page_content[content_start..].find(&close_tag) {
                    let content = &page_content[content_start..content_start + close_idx];
                    let trimmed = Self::strip_html_tags(content.trim());
                    if !trimmed.is_empty() {
                        results.push(trimmed);
                    }
                }
            }

            search_start = actual_idx + open_tag.len();
        }

        if results.is_empty() {
            None
        } else {
            Some(Self::format_content(&results.join("\n"), format))
        }
    }

    /// Strip HTML tags from content (simplified)
    fn strip_html_tags(content: &str) -> String {
        let mut result = String::new();
        let mut in_tag = false;

        for ch in content.chars() {
            match ch {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag => result.push(ch),
                _ => {}
            }
        }

        result
    }

    /// Format the extracted content based on the requested format
    fn format_content(content: &str, format: &ExtractFormat) -> String {
        match format {
            ExtractFormat::Text => Self::strip_html_tags(content),
            ExtractFormat::Html => content.to_string(),
            ExtractFormat::Markdown => Self::html_to_markdown(content),
        }
    }

    /// Convert HTML to markdown (simplified)
    fn html_to_markdown(html: &str) -> String {
        let mut result = html.to_string();

        // Convert headers
        for i in 1..=6 {
            let open = format!("<h{}>", i);
            let close = format!("</h{}>", i);
            let prefix = "#".repeat(i);
            result = result.replace(&open, &format!("{} ", prefix));
            result = result.replace(&close, "\n");
        }

        // Convert paragraphs
        result = result.replace("<p>", "");
        result = result.replace("</p>", "\n\n");

        // Convert bold
        result = result.replace("<strong>", "**");
        result = result.replace("</strong>", "**");
        result = result.replace("<b>", "**");
        result = result.replace("</b>", "**");

        // Convert italic
        result = result.replace("<em>", "*");
        result = result.replace("</em>", "*");
        result = result.replace("<i>", "*");
        result = result.replace("</i>", "*");

        // Convert links (simplified)
        result = result.replace("<a href=\"", "[");
        result = result.replace("\">", "](");
        result = result.replace("</a>", ")");

        // Strip remaining tags
        Self::strip_html_tags(&result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_by_id() {
        let html = r#"<div id="main">Hello World</div>"#;
        let result = ExtractTool::execute("#main", &ExtractFormat::Text, html);
        assert!(result.success);
    }

    #[test]
    fn test_extract_by_class() {
        let html = r#"<span class="highlight">Important</span>"#;
        let result = ExtractTool::execute(".highlight", &ExtractFormat::Text, html);
        assert!(result.success);
    }

    #[test]
    fn test_extract_by_tag() {
        let html = r#"<h1>Title</h1><p>Paragraph</p>"#;
        let result = ExtractTool::execute("h1", &ExtractFormat::Text, html);
        assert!(result.success);
    }

    #[test]
    fn test_empty_selector() {
        let result = ExtractTool::execute("", &ExtractFormat::Text, "<div></div>");
        assert!(!result.success);
    }
}
