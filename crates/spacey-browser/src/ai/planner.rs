//! Agent Planner - Task decomposition and planning
//!
//! Breaks down complex user tasks into executable steps.

use crate::ai::memory::{AgentMemory, PageContext};
use crate::ai::tools::BrowserTool;
use serde::{Deserialize, Serialize};

/// A plan consisting of multiple steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    /// Original task description
    pub task: String,
    /// Steps to execute
    pub steps: Vec<PlanStep>,
    /// Current step index
    pub current_step: usize,
    /// Whether the plan is complete
    pub completed: bool,
}

impl Plan {
    /// Create a new plan
    pub fn new(task: impl Into<String>, steps: Vec<PlanStep>) -> Self {
        Self {
            task: task.into(),
            steps,
            current_step: 0,
            completed: false,
        }
    }

    /// Get the current step
    pub fn current(&self) -> Option<&PlanStep> {
        self.steps.get(self.current_step)
    }

    /// Advance to the next step
    pub fn advance(&mut self) -> bool {
        if self.current_step + 1 < self.steps.len() {
            self.current_step += 1;
            true
        } else {
            self.completed = true;
            false
        }
    }

    /// Check if there are more steps
    pub fn has_more_steps(&self) -> bool {
        self.current_step + 1 < self.steps.len()
    }

    /// Mark the plan as complete
    pub fn mark_complete(&mut self) {
        self.completed = true;
    }

    /// Get progress as a percentage
    pub fn progress(&self) -> f32 {
        if self.steps.is_empty() {
            1.0
        } else if self.completed {
            1.0
        } else {
            self.current_step as f32 / self.steps.len() as f32
        }
    }
}

/// A single step in a plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    /// Description of what this step does
    pub description: String,
    /// The action to take (optional - may be determined at runtime)
    pub action: Option<BrowserTool>,
    /// Expected outcome
    pub expected_outcome: String,
    /// Whether this step has been completed
    pub completed: bool,
    /// Result of execution (if completed)
    pub result: Option<String>,
}

impl PlanStep {
    /// Create a new plan step
    pub fn new(description: impl Into<String>, expected_outcome: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            action: None,
            expected_outcome: expected_outcome.into(),
            completed: false,
            result: None,
        }
    }

    /// Create a step with a predefined action
    pub fn with_action(
        description: impl Into<String>,
        action: BrowserTool,
        expected_outcome: impl Into<String>,
    ) -> Self {
        Self {
            description: description.into(),
            action: Some(action),
            expected_outcome: expected_outcome.into(),
            completed: false,
            result: None,
        }
    }

    /// Mark the step as complete
    pub fn complete(&mut self, result: impl Into<String>) {
        self.completed = true;
        self.result = Some(result.into());
    }
}

/// Planner for decomposing tasks into steps
pub struct AgentPlanner {
    /// Templates for common task patterns
    templates: Vec<TaskTemplate>,
}

/// A template for a common task pattern
struct TaskTemplate {
    /// Keywords that trigger this template
    keywords: Vec<&'static str>,
    /// Generator function for the plan
    generator: fn(&str, Option<&PageContext>) -> Vec<PlanStep>,
}

impl AgentPlanner {
    /// Create a new planner with default templates
    pub fn new() -> Self {
        Self {
            templates: Self::default_templates(),
        }
    }

    /// Create a plan for a given task
    pub fn create_plan(&self, task: &str, memory: &AgentMemory) -> Plan {
        let task_lower = task.to_lowercase();

        // Try to match a template
        for template in &self.templates {
            if template.keywords.iter().any(|kw| task_lower.contains(kw)) {
                let steps = (template.generator)(task, memory.page_context());
                if !steps.is_empty() {
                    return Plan::new(task, steps);
                }
            }
        }

        // Default: create a generic exploration plan
        Plan::new(task, self.create_generic_plan(task, memory.page_context()))
    }

    /// Create default task templates
    fn default_templates() -> Vec<TaskTemplate> {
        vec![
            // Navigation template
            TaskTemplate {
                keywords: vec!["go to", "navigate", "open", "visit"],
                generator: |task, _| {
                    // Extract URL from task
                    let url = Self::extract_url(task).unwrap_or("about:blank".to_string());
                    vec![
                        PlanStep::with_action(
                            format!("Navigate to {}", url),
                            BrowserTool::Navigate { url: url.clone() },
                            "Page should load successfully",
                        ),
                        PlanStep::new(
                            "Verify page loaded",
                            "Page title and content should be visible",
                        ),
                    ]
                },
            },
            // Search template
            TaskTemplate {
                keywords: vec!["search", "find", "look for", "query"],
                generator: |task, page_context| {
                    let search_term = Self::extract_search_term(task);
                    let mut steps = vec![];

                    // If not on a search page, navigate to one
                    let needs_navigation = page_context
                        .map(|ctx| !ctx.url.contains("google.com") && !ctx.url.contains("search"))
                        .unwrap_or(true);

                    if needs_navigation {
                        steps.push(PlanStep::with_action(
                            "Navigate to search engine",
                            BrowserTool::Navigate {
                                url: "https://www.google.com".to_string(),
                            },
                            "Google homepage should load",
                        ));
                    }

                    steps.push(PlanStep::with_action(
                        "Find search input",
                        BrowserTool::Wait {
                            selector: "input[name='q'], input[type='search'], #search".to_string(),
                            timeout_ms: 5000,
                        },
                        "Search input should be visible",
                    ));

                    steps.push(PlanStep::with_action(
                        format!("Type search query: {}", search_term),
                        BrowserTool::Type {
                            selector: "input[name='q'], input[type='search'], #search".to_string(),
                            text: search_term,
                        },
                        "Search term should be entered",
                    ));

                    steps.push(PlanStep::with_action(
                        "Submit search",
                        BrowserTool::Click {
                            selector: "button[type='submit'], input[type='submit']".to_string(),
                        },
                        "Search should be submitted",
                    ));

                    steps.push(PlanStep::new(
                        "Wait for results",
                        "Search results should appear",
                    ));

                    steps
                },
            },
            // Click template
            TaskTemplate {
                keywords: vec!["click", "press", "tap", "select"],
                generator: |task, _| {
                    let target = Self::extract_click_target(task);
                    vec![
                        PlanStep::with_action(
                            format!("Click on {}", target),
                            BrowserTool::Click {
                                selector: target.clone(),
                            },
                            "Element should be clicked",
                        ),
                        PlanStep::new("Observe result", "Action result should be visible"),
                    ]
                },
            },
            // Extract/read template
            TaskTemplate {
                keywords: vec!["extract", "get", "read", "copy", "what is", "what are"],
                generator: |task, _| {
                    let target = Self::extract_content_target(task);
                    vec![
                        PlanStep::with_action(
                            format!("Extract content from {}", target),
                            BrowserTool::Extract {
                                selector: target,
                                format: crate::ai::tools::ExtractFormat::Text,
                            },
                            "Content should be extracted",
                        ),
                    ]
                },
            },
            // Form filling template
            TaskTemplate {
                keywords: vec!["fill", "enter", "type", "input", "submit form"],
                generator: |task, _| {
                    vec![
                        PlanStep::new("Identify form fields", "Form inputs should be found"),
                        PlanStep::new("Fill in required fields", "Fields should be populated"),
                        PlanStep::new("Submit form", "Form should be submitted"),
                        PlanStep::new("Verify submission", "Confirmation should appear"),
                    ]
                },
            },
            // Scroll template
            TaskTemplate {
                keywords: vec!["scroll", "go down", "go up", "see more"],
                generator: |task, _| {
                    let direction = if task.contains("up") {
                        crate::ai::tools::Direction::Up
                    } else {
                        crate::ai::tools::Direction::Down
                    };

                    vec![PlanStep::with_action(
                        format!("Scroll {:?}", direction),
                        BrowserTool::Scroll {
                            direction,
                            amount: 500,
                        },
                        "Page should scroll",
                    )]
                },
            },
        ]
    }

    /// Create a generic plan for unknown tasks
    fn create_generic_plan(&self, task: &str, _page_context: Option<&PageContext>) -> Vec<PlanStep> {
        vec![
            PlanStep::new(
                "Analyze the current page",
                "Understand what's on the page",
            ),
            PlanStep::new(
                format!("Determine how to: {}", task),
                "Identify the right actions to take",
            ),
            PlanStep::new("Execute necessary actions", "Complete the task"),
            PlanStep::new("Verify the result", "Confirm task completion"),
        ]
    }

    /// Extract a URL from a task description
    fn extract_url(task: &str) -> Option<String> {
        // Look for URL patterns
        let words: Vec<&str> = task.split_whitespace().collect();

        for word in &words {
            // Check for full URLs
            if word.starts_with("http://") || word.starts_with("https://") {
                return Some(word.to_string());
            }

            // Check for domain-like patterns
            if word.contains('.') && !word.contains(' ') {
                let cleaned = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '.');
                if cleaned.split('.').count() >= 2 {
                    return Some(format!("https://{}", cleaned));
                }
            }
        }

        // Common site names
        let task_lower = task.to_lowercase();
        if task_lower.contains("google") {
            return Some("https://www.google.com".to_string());
        }
        if task_lower.contains("github") {
            return Some("https://github.com".to_string());
        }
        if task_lower.contains("youtube") {
            return Some("https://www.youtube.com".to_string());
        }

        None
    }

    /// Extract a search term from a task
    fn extract_search_term(task: &str) -> String {
        let task_lower = task.to_lowercase();

        // Remove common prefixes
        let prefixes = [
            "search for",
            "search",
            "find",
            "look for",
            "look up",
            "query",
        ];

        let mut result = task.to_string();
        for prefix in &prefixes {
            if task_lower.starts_with(prefix) {
                result = task[prefix.len()..].trim().to_string();
                break;
            }
        }

        // Remove quotes if present
        result = result.trim_matches('"').trim_matches('\'').to_string();

        result
    }

    /// Extract click target from a task
    fn extract_click_target(task: &str) -> String {
        let task_lower = task.to_lowercase();

        // Try to extract quoted strings first
        if let Some(start) = task.find('"') {
            if let Some(end) = task[start + 1..].find('"') {
                return task[start + 1..start + 1 + end].to_string();
            }
        }

        // Look for "the X button" pattern
        if task_lower.contains("button") {
            let words: Vec<&str> = task.split_whitespace().collect();
            for (i, word) in words.iter().enumerate() {
                if word.to_lowercase() == "button" && i > 0 {
                    let target = words[i - 1];
                    return format!("button:contains('{}'), [class*='{}'], #{}", target, target, target);
                }
            }
        }

        // Look for "the X link" pattern
        if task_lower.contains("link") {
            let words: Vec<&str> = task.split_whitespace().collect();
            for (i, word) in words.iter().enumerate() {
                if word.to_lowercase() == "link" && i > 0 {
                    let target = words[i - 1];
                    return format!("a:contains('{}'), a[href*='{}']", target, target);
                }
            }
        }

        // Default: try to find any describable element
        let prefixes = ["click on", "click", "press", "tap", "select"];
        for prefix in &prefixes {
            if task_lower.starts_with(prefix) {
                let remainder = task[prefix.len()..].trim();
                return format!("[class*='{}'], #{}, :contains('{}')", 
                    remainder.replace(' ', "-"),
                    remainder.replace(' ', "-"),
                    remainder
                );
            }
        }

        // Fallback
        "button, a, [role='button']".to_string()
    }

    /// Extract content target from a task
    fn extract_content_target(task: &str) -> String {
        let task_lower = task.to_lowercase();

        // Common content targets
        if task_lower.contains("heading") || task_lower.contains("title") {
            return "h1, h2, h3, [role='heading']".to_string();
        }
        if task_lower.contains("paragraph") || task_lower.contains("text") {
            return "p, article, .content, #content".to_string();
        }
        if task_lower.contains("link") {
            return "a[href]".to_string();
        }
        if task_lower.contains("image") {
            return "img[alt]".to_string();
        }
        if task_lower.contains("list") {
            return "ul, ol, li".to_string();
        }

        // Try to find a quoted selector
        if let Some(start) = task.find('"') {
            if let Some(end) = task[start + 1..].find('"') {
                return task[start + 1..start + 1 + end].to_string();
            }
        }

        // Default: main content area
        "main, article, .content, #content, body".to_string()
    }
}

impl Default for AgentPlanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_navigation_plan() {
        let planner = AgentPlanner::new();
        let memory = AgentMemory::default();
        let plan = planner.create_plan("go to google.com", &memory);

        assert!(!plan.steps.is_empty());
        assert!(plan.task.contains("google"));
    }

    #[test]
    fn test_create_search_plan() {
        let planner = AgentPlanner::new();
        let memory = AgentMemory::default();
        let plan = planner.create_plan("search for rust programming", &memory);

        assert!(!plan.steps.is_empty());
        assert!(plan.steps.len() >= 3); // At least: navigate, type, submit
    }

    #[test]
    fn test_plan_progress() {
        let mut plan = Plan::new(
            "test",
            vec![
                PlanStep::new("Step 1", "Result 1"),
                PlanStep::new("Step 2", "Result 2"),
            ],
        );

        assert_eq!(plan.progress(), 0.0);
        plan.advance();
        assert_eq!(plan.progress(), 0.5);
        plan.advance();
        assert_eq!(plan.progress(), 1.0);
    }

    #[test]
    fn test_extract_url() {
        assert_eq!(
            AgentPlanner::extract_url("go to https://example.com"),
            Some("https://example.com".to_string())
        );
        assert_eq!(
            AgentPlanner::extract_url("visit example.com"),
            Some("https://example.com".to_string())
        );
        assert_eq!(
            AgentPlanner::extract_url("go to google"),
            Some("https://www.google.com".to_string())
        );
    }

    #[test]
    fn test_extract_search_term() {
        assert_eq!(
            AgentPlanner::extract_search_term("search for rust tutorials"),
            "rust tutorials"
        );
        assert_eq!(
            AgentPlanner::extract_search_term("find \"exact phrase\""),
            "exact phrase"
        );
    }
}
