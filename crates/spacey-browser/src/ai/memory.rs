//! Agent Memory - Conversation history and context management
//!
//! Manages the conversation history, page context, and provides
//! context compression when the history grows too long.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Maximum number of messages to keep before compression
const MAX_HISTORY_MESSAGES: usize = 50;

/// Maximum tokens before triggering compression
const MAX_CONTEXT_TOKENS: usize = 3000;

/// Role in a conversation message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// System prompt
    System,
    /// User message
    User,
    /// Assistant response
    Assistant,
    /// Tool/function result
    Tool,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::System => write!(f, "system"),
            Role::User => write!(f, "user"),
            Role::Assistant => write!(f, "assistant"),
            Role::Tool => write!(f, "tool"),
        }
    }
}

/// A single message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Role of the message sender
    pub role: Role,
    /// Content of the message
    pub content: String,
    /// Optional tool call ID (for tool results)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// Estimated token count
    #[serde(skip)]
    pub token_count: usize,
}

impl Message {
    /// Create a new message
    pub fn new(role: Role, content: impl Into<String>) -> Self {
        let content = content.into();
        let token_count = Self::estimate_tokens(&content);
        Self {
            role,
            content,
            tool_call_id: None,
            token_count,
        }
    }

    /// Create a tool result message
    pub fn tool_result(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        let content = content.into();
        let token_count = Self::estimate_tokens(&content);
        Self {
            role: Role::Tool,
            content,
            tool_call_id: Some(tool_call_id.into()),
            token_count,
        }
    }

    /// Estimate token count (rough approximation: ~4 chars per token)
    fn estimate_tokens(text: &str) -> usize {
        (text.len() + 3) / 4
    }
}

/// Context about the current page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageContext {
    /// Current URL
    pub url: String,
    /// Page title
    pub title: String,
    /// Visible text content (truncated)
    pub text_content: String,
    /// List of interactive elements
    pub interactive_elements: Vec<InteractiveElement>,
}

/// An interactive element on the page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveElement {
    /// Element type (button, link, input, etc.)
    pub element_type: String,
    /// CSS selector to target this element
    pub selector: String,
    /// Text content or label
    pub text: String,
    /// Additional attributes
    pub attributes: std::collections::HashMap<String, String>,
}

/// Manages conversation history and context
pub struct AgentMemory {
    /// Conversation history
    history: VecDeque<Message>,
    /// Current page context
    page_context: Option<PageContext>,
    /// Maximum tokens allowed in context
    max_tokens: usize,
    /// Current total token count
    current_tokens: usize,
    /// System prompt (always included)
    system_prompt: String,
}

impl AgentMemory {
    /// Create a new agent memory with the given system prompt
    pub fn new(system_prompt: impl Into<String>) -> Self {
        let system_prompt = system_prompt.into();
        let system_tokens = Message::estimate_tokens(&system_prompt);
        Self {
            history: VecDeque::new(),
            page_context: None,
            max_tokens: MAX_CONTEXT_TOKENS,
            current_tokens: system_tokens,
            system_prompt,
        }
    }

    /// Add a user message
    pub fn add_user_message(&mut self, content: impl Into<String>) {
        let msg = Message::new(Role::User, content);
        self.add_message(msg);
    }

    /// Add an assistant message
    pub fn add_assistant_message(&mut self, content: impl Into<String>) {
        let msg = Message::new(Role::Assistant, content);
        self.add_message(msg);
    }

    /// Add a tool result
    pub fn add_observation(&mut self, tool_call_id: impl Into<String>, result: impl Into<String>) {
        let msg = Message::tool_result(tool_call_id, result);
        self.add_message(msg);
    }

    /// Add a message and handle context overflow
    fn add_message(&mut self, msg: Message) {
        self.current_tokens += msg.token_count;
        self.history.push_back(msg);

        // Check if we need to compress
        if self.current_tokens > self.max_tokens || self.history.len() > MAX_HISTORY_MESSAGES {
            self.compress();
        }
    }

    /// Set the current page context
    pub fn set_page_context(&mut self, page: PageContext) {
        // Update token count
        if let Some(old_context) = &self.page_context {
            let old_tokens = Self::estimate_page_tokens(old_context);
            self.current_tokens = self.current_tokens.saturating_sub(old_tokens);
        }
        
        let new_tokens = Self::estimate_page_tokens(&page);
        self.current_tokens += new_tokens;
        self.page_context = Some(page);
    }

    /// Estimate tokens for page context
    fn estimate_page_tokens(page: &PageContext) -> usize {
        let base = (page.url.len() + page.title.len() + page.text_content.len()) / 4;
        let elements: usize = page.interactive_elements.iter()
            .map(|e| (e.text.len() + e.selector.len()) / 4)
            .sum();
        base + elements + 50 // Overhead for formatting
    }

    /// Clear the page context
    pub fn clear_page_context(&mut self) {
        if let Some(old_context) = &self.page_context {
            let old_tokens = Self::estimate_page_tokens(old_context);
            self.current_tokens = self.current_tokens.saturating_sub(old_tokens);
        }
        self.page_context = None;
    }

    /// Compress the history to fit within token limits
    fn compress(&mut self) {
        // Strategy: Keep the first message (often important context) and recent messages
        // Remove messages from the middle
        
        while self.current_tokens > self.max_tokens && self.history.len() > 4 {
            // Remove the second message (keep first, remove from near-beginning)
            if let Some(msg) = self.history.remove(1) {
                self.current_tokens = self.current_tokens.saturating_sub(msg.token_count);
            }
        }

        // If still over limit, truncate oldest messages
        while self.history.len() > MAX_HISTORY_MESSAGES / 2 {
            if let Some(msg) = self.history.pop_front() {
                self.current_tokens = self.current_tokens.saturating_sub(msg.token_count);
            }
        }

        log::debug!(
            "Memory compressed: {} messages, ~{} tokens",
            self.history.len(),
            self.current_tokens
        );
    }

    /// Build the full prompt including system, history, and page context
    pub fn build_prompt(&self) -> String {
        let mut prompt = String::new();

        // System prompt
        prompt.push_str(&format!("<|system|>\n{}<|end|>\n", self.system_prompt));

        // Page context if available
        if let Some(page) = &self.page_context {
            prompt.push_str(&format!(
                "<|system|>\nCurrent page: {} ({})\n\nPage content:\n{}\n\nInteractive elements:\n{}<|end|>\n",
                page.title,
                page.url,
                &page.text_content[..page.text_content.len().min(2000)],
                self.format_interactive_elements(&page.interactive_elements)
            ));
        }

        // Conversation history
        for msg in &self.history {
            let role_tag = match msg.role {
                Role::System => "system",
                Role::User => "user",
                Role::Assistant => "assistant",
                Role::Tool => "tool",
            };
            prompt.push_str(&format!("<|{}|>\n{}<|end|>\n", role_tag, msg.content));
        }

        // Add assistant prefix for generation
        prompt.push_str("<|assistant|>\n");

        prompt
    }

    /// Format interactive elements for the prompt
    fn format_interactive_elements(&self, elements: &[InteractiveElement]) -> String {
        elements
            .iter()
            .take(20) // Limit to prevent context overflow
            .enumerate()
            .map(|(i, e)| {
                format!(
                    "{}. [{}] {} - selector: \"{}\"",
                    i + 1,
                    e.element_type,
                    e.text,
                    e.selector
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Get the conversation history
    pub fn history(&self) -> &VecDeque<Message> {
        &self.history
    }

    /// Get the current page context
    pub fn page_context(&self) -> Option<&PageContext> {
        self.page_context.as_ref()
    }

    /// Clear all history (but keep system prompt)
    pub fn clear(&mut self) {
        self.history.clear();
        self.page_context = None;
        self.current_tokens = Message::estimate_tokens(&self.system_prompt);
    }

    /// Get approximate token count
    pub fn token_count(&self) -> usize {
        self.current_tokens
    }

    /// Estimate tokens for a given string
    fn estimate_tokens(text: &str) -> usize {
        (text.len() + 3) / 4
    }
}

impl Default for AgentMemory {
    fn default() -> Self {
        Self::new(DEFAULT_SYSTEM_PROMPT)
    }
}

/// Default system prompt for the browser agent
pub const DEFAULT_SYSTEM_PROMPT: &str = r#"You are an AI browser assistant that can interact with web pages using tools.

Available tools:
- click(selector): Click on an element matching the CSS selector
- type(selector, text): Type text into an input element
- navigate(url): Navigate to a URL
- extract(selector): Extract text content from elements
- scroll(direction, amount): Scroll the page (direction: up/down/left/right)
- execute_js(code): Execute JavaScript code
- wait(selector): Wait for an element to appear

Always respond in JSON format with your thought process and action:
{
    "thought": "Your reasoning about what to do next",
    "action": {
        "tool": "tool_name",
        "params": { "param1": "value1" }
    }
}

When a task is complete, respond with:
{
    "thought": "Task completed because...",
    "action": null,
    "result": "Summary of what was accomplished"
}

Be precise with CSS selectors. Use IDs when available, otherwise use classes or tag names.
If an action fails, try alternative approaches."#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_creation() {
        let memory = AgentMemory::new("Test prompt");
        assert!(memory.history().is_empty());
    }

    #[test]
    fn test_add_messages() {
        let mut memory = AgentMemory::new("Test");
        memory.add_user_message("Hello");
        memory.add_assistant_message("Hi there");
        
        assert_eq!(memory.history().len(), 2);
        assert_eq!(memory.history()[0].role, Role::User);
        assert_eq!(memory.history()[1].role, Role::Assistant);
    }

    #[test]
    fn test_build_prompt() {
        let mut memory = AgentMemory::new("You are helpful.");
        memory.add_user_message("What's 2+2?");
        
        let prompt = memory.build_prompt();
        assert!(prompt.contains("You are helpful."));
        assert!(prompt.contains("What's 2+2?"));
        assert!(prompt.contains("<|assistant|>"));
    }

    #[test]
    fn test_page_context() {
        let mut memory = AgentMemory::new("Test");
        
        let context = PageContext {
            url: "https://example.com".to_string(),
            title: "Example".to_string(),
            text_content: "Hello World".to_string(),
            interactive_elements: vec![],
        };
        
        memory.set_page_context(context);
        assert!(memory.page_context().is_some());
        
        let prompt = memory.build_prompt();
        assert!(prompt.contains("example.com"));
    }
}
