//! Integration tests for the AI Agent

use spacey_browser::ai::{
    AgentConfig, AiAgent, BrowserTool, ToolResult,
};
use spacey_browser::ai::tools::{
    ClickTool, TypeTool, NavigateTool, ExtractTool, ScrollTool,
    WaitTool, Direction, ExtractFormat,
};

#[test]
fn test_agent_creation() {
    let config = AgentConfig::default();
    let agent = AiAgent::new(config);
    
    assert!(!agent.is_model_loaded());
}

#[test]
fn test_click_tool() {
    let html = r#"<button id="submit" class="btn primary">Submit</button>"#;
    
    let result = ClickTool::execute("#submit", html);
    assert!(result.success);
    
    let result = ClickTool::execute(".btn", html);
    assert!(result.success);
    
    let result = ClickTool::execute("#nonexistent", html);
    assert!(!result.success);
}

#[test]
fn test_type_tool() {
    let html = r#"<input type="text" id="username" />"#;
    
    let result = TypeTool::execute("#username", "testuser", html);
    assert!(result.success);
    
    let result = TypeTool::execute("", "text", html);
    assert!(!result.success);
}

#[test]
fn test_navigate_tool() {
    let result = NavigateTool::execute("https://example.com");
    assert!(result.success);
    
    let result = NavigateTool::execute("example.com");
    assert!(result.success);
}

#[test]
fn test_extract_tool() {
    let html = r#"<h1 id="title">Hello World</h1><p class="content">This is content</p>"#;
    
    let result = ExtractTool::execute("#title", &ExtractFormat::Text, html);
    assert!(result.success);
    
    let result = ExtractTool::execute("h1", &ExtractFormat::Text, html);
    assert!(result.success);
}

#[test]
fn test_scroll_tool() {
    let result = ScrollTool::execute(Direction::Down, 300);
    assert!(result.success);
    
    let result = ScrollTool::execute(Direction::Up, 200);
    assert!(result.success);
    
    // Invalid amount
    let result = ScrollTool::execute(Direction::Down, 0);
    assert!(!result.success);
}

#[test]
fn test_wait_tool() {
    let result = WaitTool::execute("#loading", 5000);
    assert!(result.success);
    
    let result = WaitTool::execute("", 5000);
    assert!(!result.success);
    
    let result = WaitTool::execute("#element", 0);
    assert!(!result.success);
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
fn test_tool_result_chaining() {
    let result = ToolResult::success("Done")
        .with_step_complete();
    
    assert!(result.success);
    assert!(result.is_complete());
    
    let result = ToolResult::error("Failed");
    assert!(!result.success);
    assert!(!result.is_complete());
}
