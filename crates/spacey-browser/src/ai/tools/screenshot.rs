//! Screenshot Tool - Capture screenshots of the current page
//!
//! This tool allows the AI agent to capture visual representations
//! of the current page state for analysis and decision making.

use super::ToolResult;
use serde::{Deserialize, Serialize};

/// Screenshot capture region
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ScreenshotRegion {
    /// Capture the entire visible viewport
    Viewport,
    /// Capture the full page (scrollable content)
    FullPage,
    /// Capture a specific element by selector
    Element { selector: String },
    /// Capture a specific rectangular region
    Region {
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    },
}

impl Default for ScreenshotRegion {
    fn default() -> Self {
        ScreenshotRegion::Viewport
    }
}

/// Screenshot output format
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ScreenshotFormat {
    /// PNG format (lossless, larger file)
    Png,
    /// JPEG format (lossy, smaller file)
    Jpeg,
    /// WebP format (modern, efficient)
    WebP,
}

impl Default for ScreenshotFormat {
    fn default() -> Self {
        ScreenshotFormat::Png
    }
}

/// Screenshot tool for capturing page visuals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotTool {
    /// Region to capture
    #[serde(default)]
    pub region: ScreenshotRegion,
    
    /// Output format
    #[serde(default)]
    pub format: ScreenshotFormat,
    
    /// JPEG/WebP quality (1-100, default 80)
    #[serde(default = "default_quality")]
    pub quality: u8,
    
    /// Scale factor (for high-DPI captures)
    #[serde(default = "default_scale")]
    pub scale: f32,
    
    /// Whether to include UI overlays (scrollbars, etc.)
    #[serde(default)]
    pub include_overlays: bool,
}

fn default_quality() -> u8 {
    80
}

fn default_scale() -> f32 {
    1.0
}

impl Default for ScreenshotTool {
    fn default() -> Self {
        Self {
            region: ScreenshotRegion::default(),
            format: ScreenshotFormat::default(),
            quality: default_quality(),
            scale: default_scale(),
            include_overlays: false,
        }
    }
}

impl ScreenshotTool {
    /// Create a viewport screenshot tool
    pub fn viewport() -> Self {
        Self::default()
    }
    
    /// Create a full page screenshot tool
    pub fn full_page() -> Self {
        Self {
            region: ScreenshotRegion::FullPage,
            ..Default::default()
        }
    }
    
    /// Create an element screenshot tool
    pub fn element(selector: impl Into<String>) -> Self {
        Self {
            region: ScreenshotRegion::Element {
                selector: selector.into(),
            },
            ..Default::default()
        }
    }
    
    /// Create a region screenshot tool
    pub fn region(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            region: ScreenshotRegion::Region { x, y, width, height },
            ..Default::default()
        }
    }
    
    /// Set the output format
    pub fn with_format(mut self, format: ScreenshotFormat) -> Self {
        self.format = format;
        self
    }
    
    /// Set the quality (for JPEG/WebP)
    pub fn with_quality(mut self, quality: u8) -> Self {
        self.quality = quality.min(100);
        self
    }
    
    /// Set the scale factor
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale.max(0.1).min(4.0);
        self
    }
}

/// Result of a screenshot capture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotResult {
    /// Base64-encoded image data
    pub data: String,
    
    /// Image format
    pub format: ScreenshotFormat,
    
    /// Image width in pixels
    pub width: u32,
    
    /// Image height in pixels
    pub height: u32,
    
    /// File size in bytes
    pub size_bytes: usize,
    
    /// Timestamp when captured
    pub timestamp: u64,
}

impl ScreenshotResult {
    /// Get the data as a data URL
    pub fn to_data_url(&self) -> String {
        let mime = match self.format {
            ScreenshotFormat::Png => "image/png",
            ScreenshotFormat::Jpeg => "image/jpeg",
            ScreenshotFormat::WebP => "image/webp",
        };
        format!("data:{};base64,{}", mime, self.data)
    }
    
    /// Get size in KB
    pub fn size_kb(&self) -> f64 {
        self.size_bytes as f64 / 1024.0
    }
}

/// Screenshot capture interface
pub trait ScreenshotCapture {
    /// Capture a screenshot with the given tool configuration
    fn capture(&self, tool: &ScreenshotTool) -> ToolResult;
    
    /// Capture viewport screenshot with default settings
    fn capture_viewport(&self) -> ToolResult {
        self.capture(&ScreenshotTool::viewport())
    }
    
    /// Capture full page screenshot
    fn capture_full_page(&self) -> ToolResult {
        self.capture(&ScreenshotTool::full_page())
    }
    
    /// Capture element screenshot
    fn capture_element(&self, selector: &str) -> ToolResult {
        self.capture(&ScreenshotTool::element(selector))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screenshot_tool_default() {
        let tool = ScreenshotTool::default();
        assert_eq!(tool.region, ScreenshotRegion::Viewport);
        assert_eq!(tool.format, ScreenshotFormat::Png);
        assert_eq!(tool.quality, 80);
        assert_eq!(tool.scale, 1.0);
    }

    #[test]
    fn test_screenshot_tool_builders() {
        let tool = ScreenshotTool::full_page()
            .with_format(ScreenshotFormat::Jpeg)
            .with_quality(90);
        
        assert_eq!(tool.region, ScreenshotRegion::FullPage);
        assert_eq!(tool.format, ScreenshotFormat::Jpeg);
        assert_eq!(tool.quality, 90);
    }

    #[test]
    fn test_element_screenshot() {
        let tool = ScreenshotTool::element("#main-content");
        
        if let ScreenshotRegion::Element { selector } = &tool.region {
            assert_eq!(selector, "#main-content");
        } else {
            panic!("Expected Element region");
        }
    }

    #[test]
    fn test_screenshot_result_data_url() {
        let result = ScreenshotResult {
            data: "dGVzdA==".to_string(), // "test" in base64
            format: ScreenshotFormat::Png,
            width: 100,
            height: 100,
            size_bytes: 1024,
            timestamp: 0,
        };
        
        assert!(result.to_data_url().starts_with("data:image/png;base64,"));
    }
}
