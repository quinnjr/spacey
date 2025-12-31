//! AI Agent - Main orchestrator using ReAct pattern
//!
//! Coordinates the model, memory, tools, and planner to execute
//! complex browser automation tasks.

use crate::ai::memory::{AgentMemory, PageContext, DEFAULT_SYSTEM_PROMPT};
use crate::ai::model::{ModelConfig, ModelError, Phi3Model};
use crate::ai::planner::{AgentPlanner, Plan, PlanStep};
use crate::ai::tools::{BrowserTool, ToolRegistry, ToolResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::RwLock;

/// Configuration for the AI agent
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Model configuration
    pub model_config: ModelConfig,
    /// Maximum iterations per step
    pub max_iterations_per_step: usize,
    /// Maximum total iterations
    pub max_total_iterations: usize,
    /// Whether to automatically download the model
    pub auto_download_model: bool,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            model_config: ModelConfig::default(),
            max_iterations_per_step: 5,
            max_total_iterations: 50,
            auto_download_model: true,
        }
    }
}

/// Result of a task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Whether the task completed successfully
    pub success: bool,
    /// Summary of what was accomplished
    pub summary: String,
    /// List of actions taken
    pub actions: Vec<ActionRecord>,
    /// Error message if failed
    pub error: Option<String>,
}

impl TaskResult {
    /// Create a successful result
    pub fn success(summary: impl Into<String>, actions: Vec<ActionRecord>) -> Self {
        Self {
            success: true,
            summary: summary.into(),
            actions,
            error: None,
        }
    }

    /// Create a failed result
    pub fn failure(error: impl Into<String>, actions: Vec<ActionRecord>) -> Self {
        Self {
            success: false,
            summary: String::new(),
            actions,
            error: Some(error.into()),
        }
    }
}

/// Record of an action taken
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRecord {
    /// Timestamp of the action
    pub timestamp: u64,
    /// The thought process
    pub thought: String,
    /// The action taken
    pub action: Option<BrowserTool>,
    /// Result of the action
    pub result: ToolResult,
}

/// The AI Agent - coordinates all components
pub struct AiAgent {
    /// The language model
    model: Option<Phi3Model>,
    /// Conversation memory
    memory: AgentMemory,
    /// Available tools
    tools: ToolRegistry,
    /// Task planner
    planner: AgentPlanner,
    /// Configuration
    config: AgentConfig,
    /// Whether the model is loaded
    model_loaded: bool,
    /// Current execution state
    state: AgentState,
}

/// Current state of the agent
#[derive(Debug, Clone, Default)]
pub struct AgentState {
    /// Currently executing plan
    pub current_plan: Option<Plan>,
    /// Total iterations executed
    pub total_iterations: usize,
    /// Whether agent is currently running
    pub is_running: bool,
    /// Last error encountered
    pub last_error: Option<String>,
}

/// Response from the model
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModelResponse {
    /// Agent's reasoning
    thought: String,
    /// Action to take (null if task complete)
    action: Option<ActionSpec>,
    /// Final result if task is complete
    result: Option<String>,
}

/// Action specification from the model
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ActionSpec {
    /// Tool name
    tool: String,
    /// Tool parameters
    params: serde_json::Value,
}

impl AiAgent {
    /// Create a new AI agent (model loaded lazily)
    pub fn new(config: AgentConfig) -> Self {
        Self {
            model: None,
            memory: AgentMemory::new(DEFAULT_SYSTEM_PROMPT),
            tools: ToolRegistry::new(),
            planner: AgentPlanner::new(),
            config,
            model_loaded: false,
            state: AgentState::default(),
        }
    }

    /// Create an agent with a pre-loaded model
    pub fn with_model(config: AgentConfig, model: Phi3Model) -> Self {
        Self {
            model: Some(model),
            memory: AgentMemory::new(DEFAULT_SYSTEM_PROMPT),
            tools: ToolRegistry::new(),
            planner: AgentPlanner::new(),
            config,
            model_loaded: true,
            state: AgentState::default(),
        }
    }

    /// Load the model (if not already loaded)
    pub fn load_model(&mut self) -> Result<(), ModelError> {
        if self.model_loaded {
            return Ok(());
        }

        log::info!("Loading AI model...");
        let model = Phi3Model::new(self.config.model_config.clone())?;
        self.model = Some(model);
        self.model_loaded = true;
        log::info!("AI model loaded successfully");

        Ok(())
    }

    /// Check if the model is loaded
    pub fn is_model_loaded(&self) -> bool {
        self.model_loaded
    }

    /// Execute a task
    pub async fn execute_task<F>(
        &mut self,
        task: &str,
        mut action_executor: F,
    ) -> TaskResult
    where
        F: FnMut(BrowserTool) -> ToolResult,
    {
        log::info!("Executing task: {}", task);

        // Ensure model is loaded
        if !self.model_loaded {
            if let Err(e) = self.load_model() {
                return TaskResult::failure(format!("Failed to load model: {}", e), vec![]);
            }
        }

        // Reset state
        self.state = AgentState {
            current_plan: None,
            total_iterations: 0,
            is_running: true,
            last_error: None,
        };

        let mut actions = Vec::new();

        // Create a plan for the task
        let plan = self.planner.create_plan(task, &self.memory);
        self.state.current_plan = Some(plan.clone());

        // Add the task to memory
        self.memory.add_user_message(task);

        // Execute the plan using ReAct loop
        for step in plan.steps.iter() {
            if self.state.total_iterations >= self.config.max_total_iterations {
                return TaskResult::failure(
                    "Maximum iterations exceeded",
                    actions,
                );
            }

            match self.execute_step(step, &mut action_executor, &mut actions).await {
                Ok(true) => {
                    // Step completed successfully, continue
                    if let Some(ref mut plan) = self.state.current_plan {
                        plan.advance();
                    }
                }
                Ok(false) => {
                    // Step needs more work, but we've hit iteration limit for this step
                    log::warn!("Step iteration limit reached: {}", step.description);
                }
                Err(e) => {
                    self.state.last_error = Some(e.clone());
                    return TaskResult::failure(e, actions);
                }
            }
        }

        // Mark as complete
        self.state.is_running = false;
        if let Some(ref mut plan) = self.state.current_plan {
            plan.mark_complete();
        }

        // Generate summary
        let summary = self.generate_summary(&actions);

        TaskResult::success(summary, actions)
    }

    /// Execute a single step using ReAct loop
    async fn execute_step<F>(
        &mut self,
        step: &PlanStep,
        action_executor: &mut F,
        actions: &mut Vec<ActionRecord>,
    ) -> Result<bool, String>
    where
        F: FnMut(BrowserTool) -> ToolResult,
    {
        let mut step_iterations = 0;

        loop {
            if step_iterations >= self.config.max_iterations_per_step {
                log::warn!("Max iterations reached for step: {}", step.description);
                return Ok(false);
            }

            step_iterations += 1;
            self.state.total_iterations += 1;

            // Get the model's response
            let response = self.think(&step.description)?;

            // Record the action
            let record = ActionRecord {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                thought: response.thought.clone(),
                action: None,
                result: ToolResult::success("Thinking..."),
            };

            // Check if task/step is complete
            if response.action.is_none() {
                if let Some(result) = response.result {
                    self.memory.add_assistant_message(&result);
                    let mut final_record = record;
                    final_record.result = ToolResult::success(&result).with_step_complete();
                    actions.push(final_record);
                    return Ok(true);
                }
            }

            // Execute the action
            if let Some(action_spec) = response.action {
                let tool = self.parse_tool(&action_spec)?;
                let result = action_executor(tool.clone());

                let mut action_record = record;
                action_record.action = Some(tool);
                action_record.result = result.clone();
                actions.push(action_record);

                // Add observation to memory
                self.memory.add_observation(
                    &format!("action_{}", step_iterations),
                    &format!("{}: {}", if result.success { "Success" } else { "Failed" }, result.message),
                );

                // Check if step is complete
                if result.is_complete() {
                    return Ok(true);
                }

                // If failed, the model will try to recover on next iteration
                if !result.success {
                    log::warn!("Action failed: {}", result.message);
                }
            }
        }
    }

    /// Think about what to do next
    fn think(&mut self, step_description: &str) -> Result<ModelResponse, String> {
        let model = self.model.as_mut().ok_or("Model not loaded")?;

        // Build the prompt with current context
        let mut prompt = self.memory.build_prompt();
        prompt.push_str(&format!(
            "\nCurrent step: {}\n\nRespond with JSON:\n",
            step_description
        ));

        // Generate response
        let response_text = model
            .generate(&prompt, 500)
            .map_err(|e| format!("Generation failed: {}", e))?;

        // Parse the JSON response
        self.parse_model_response(&response_text)
    }

    /// Parse the model's response
    fn parse_model_response(&self, response: &str) -> Result<ModelResponse, String> {
        // Try to extract JSON from the response
        let json_str = self.extract_json(response)?;

        serde_json::from_str(&json_str)
            .map_err(|e| format!("Failed to parse model response: {}", e))
    }

    /// Extract JSON from the model's response
    fn extract_json(&self, response: &str) -> Result<String, String> {
        // Look for JSON object in the response
        let start = response.find('{').ok_or("No JSON object found in response")?;
        
        let mut depth = 0;
        let mut end = start;
        
        for (i, ch) in response[start..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = start + i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }

        if depth != 0 {
            return Err("Malformed JSON in response".to_string());
        }

        Ok(response[start..end].to_string())
    }

    /// Parse a tool from an action specification
    fn parse_tool(&self, action: &ActionSpec) -> Result<BrowserTool, String> {
        let json = serde_json::json!({
            "tool": action.tool,
            "params": action.params
        });

        BrowserTool::from_json(&json)
    }

    /// Generate a summary of the executed actions
    fn generate_summary(&self, actions: &[ActionRecord]) -> String {
        if actions.is_empty() {
            return "No actions were taken.".to_string();
        }

        let successful = actions.iter().filter(|a| a.result.success).count();
        let total = actions.len();

        let action_summary: Vec<String> = actions
            .iter()
            .filter(|a| a.action.is_some())
            .map(|a| {
                let tool_name = a.action.as_ref().map(|t| t.name()).unwrap_or("unknown");
                format!("- {}: {}", tool_name, a.result.message)
            })
            .collect();

        format!(
            "Completed {}/{} actions successfully.\n\nActions taken:\n{}",
            successful,
            total,
            action_summary.join("\n")
        )
    }

    /// Update page context
    pub fn set_page_context(&mut self, context: PageContext) {
        self.memory.set_page_context(context);
    }

    /// Get the current state
    pub fn state(&self) -> &AgentState {
        &self.state
    }

    /// Clear memory and reset state
    pub fn reset(&mut self) {
        self.memory.clear();
        self.state = AgentState::default();
    }

    /// Get the tool registry
    pub fn tools(&self) -> &ToolRegistry {
        &self.tools
    }

    /// Get the planner
    pub fn planner(&self) -> &AgentPlanner {
        &self.planner
    }
}

impl Default for AiAgent {
    fn default() -> Self {
        Self::new(AgentConfig::default())
    }
}

/// Thread-safe wrapper for the agent
pub type SharedAgent = Arc<RwLock<AiAgent>>;

/// Create a shared agent
pub fn create_shared_agent(config: AgentConfig) -> SharedAgent {
    Arc::new(RwLock::new(AiAgent::new(config)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let agent = AiAgent::new(AgentConfig::default());
        assert!(!agent.is_model_loaded());
    }

    #[test]
    fn test_extract_json() {
        let agent = AiAgent::new(AgentConfig::default());
        
        let response = r#"Here's my thinking: {"thought": "test", "action": null}"#;
        let json = agent.extract_json(response);
        assert!(json.is_ok());
        assert!(json.unwrap().contains("thought"));
    }

    #[test]
    fn test_task_result() {
        let result = TaskResult::success("Done", vec![]);
        assert!(result.success);
        assert!(result.error.is_none());

        let result = TaskResult::failure("Error occurred", vec![]);
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_shared_agent() {
        let agent = create_shared_agent(AgentConfig::default());
        let guard = agent.read();
        assert!(!guard.is_model_loaded());
    }
}
