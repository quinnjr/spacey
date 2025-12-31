//! AI UI State - Manages the AI assistant panel state

use crate::ai::{ActionRecord, AgentState};

/// State for the AI assistant UI panel
#[derive(Default)]
pub struct AiUiState {
    /// Whether the AI panel is visible
    pub panel_visible: bool,
    /// Current task input text
    pub task_input: String,
    /// Whether the AI is currently running
    pub is_running: bool,
    /// Progress of the current task (0.0 to 1.0)
    pub progress: f32,
    /// Current step description
    pub current_step: String,
    /// History of actions taken
    pub action_history: Vec<ActionHistoryItem>,
    /// Last result or error message
    pub last_message: Option<(bool, String)>,
    /// Chat history for display
    pub chat_history: Vec<ChatMessage>,
    /// Whether the model is loaded
    pub model_loaded: bool,
    /// Model loading progress message
    pub loading_message: String,
}

/// A record of an action for display
#[derive(Clone)]
pub struct ActionHistoryItem {
    /// Tool name
    pub tool: String,
    /// Brief description
    pub description: String,
    /// Whether it succeeded
    pub success: bool,
    /// Timestamp
    pub timestamp: String,
}

/// A message in the chat history
#[derive(Clone)]
pub struct ChatMessage {
    /// Who sent the message
    pub role: ChatRole,
    /// Message content
    pub content: String,
}

/// Role in the chat
#[derive(Clone, PartialEq)]
pub enum ChatRole {
    User,
    Assistant,
    System,
}

impl AiUiState {
    /// Create a new AI UI state
    pub fn new() -> Self {
        Self {
            panel_visible: true,
            loading_message: "AI not loaded".to_string(),
            ..Default::default()
        }
    }

    /// Update from agent state
    pub fn update_from_agent(&mut self, state: &AgentState) {
        self.is_running = state.is_running;
        
        if let Some(plan) = &state.current_plan {
            self.progress = plan.progress();
            if let Some(step) = plan.current() {
                self.current_step = step.description.clone();
            }
        }

        if let Some(ref error) = state.last_error {
            self.last_message = Some((false, error.clone()));
        }
    }

    /// Add an action to history
    pub fn add_action(&mut self, record: &ActionRecord) {
        let tool = record.action.as_ref()
            .map(|t| t.name().to_string())
            .unwrap_or_else(|| "thinking".to_string());

        let item = ActionHistoryItem {
            tool,
            description: record.thought.chars().take(100).collect(),
            success: record.result.success,
            timestamp: chrono_lite(),
        };

        self.action_history.push(item);

        // Keep only last 20 actions
        if self.action_history.len() > 20 {
            self.action_history.remove(0);
        }
    }

    /// Add a chat message
    pub fn add_message(&mut self, role: ChatRole, content: impl Into<String>) {
        self.chat_history.push(ChatMessage {
            role,
            content: content.into(),
        });

        // Keep only last 50 messages
        if self.chat_history.len() > 50 {
            self.chat_history.remove(0);
        }
    }

    /// Clear the chat history
    pub fn clear_chat(&mut self) {
        self.chat_history.clear();
        self.action_history.clear();
        self.last_message = None;
    }

    /// Toggle panel visibility
    pub fn toggle_panel(&mut self) {
        self.panel_visible = !self.panel_visible;
    }

    /// Set model loaded status
    pub fn set_model_loaded(&mut self, loaded: bool) {
        self.model_loaded = loaded;
        if loaded {
            self.loading_message = "AI Ready".to_string();
        }
    }

    /// Set loading message
    pub fn set_loading_message(&mut self, message: impl Into<String>) {
        self.loading_message = message.into();
    }
}

/// Simple timestamp without external deps
fn chrono_lite() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    // Simple time format HH:MM:SS
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

/// Render the AI panel
pub fn render_ai_panel(ctx: &egui::Context, state: &mut AiUiState) -> Option<AiPanelAction> {
    let mut action = None;

    if !state.panel_visible {
        // Show a small toggle button when hidden
        egui::Area::new(egui::Id::new("ai_toggle"))
            .fixed_pos(egui::pos2(ctx.screen_rect().right() - 50.0, 10.0))
            .show(ctx, |ui| {
                if ui.button("🤖").clicked() {
                    state.toggle_panel();
                }
            });
        return None;
    }

    egui::SidePanel::right("ai_panel")
        .default_width(350.0)
        .min_width(280.0)
        .show(ctx, |ui| {
            // Header
            ui.horizontal(|ui| {
                ui.heading("🤖 AI Assistant");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✕").clicked() {
                        state.toggle_panel();
                    }
                });
            });
            ui.separator();

            // Model status
            ui.horizontal(|ui| {
                if state.model_loaded {
                    ui.label(egui::RichText::new("● Ready").color(egui::Color32::GREEN));
                } else {
                    ui.label(egui::RichText::new("○ Not loaded").color(egui::Color32::GRAY));
                    if ui.button("Load Model").clicked() {
                        action = Some(AiPanelAction::LoadModel);
                    }
                }
            });
            ui.label(&state.loading_message);
            ui.separator();

            // Task input
            ui.label("What would you like me to do?");
            let _response = ui.add(
                egui::TextEdit::multiline(&mut state.task_input)
                    .desired_rows(2)
                    .desired_width(f32::INFINITY)
                    .hint_text("e.g., 'Search for Rust tutorials'")
            );

            ui.horizontal(|ui| {
                let can_execute = state.model_loaded && !state.is_running && !state.task_input.is_empty();
                
                if ui.add_enabled(can_execute, egui::Button::new("▶ Execute")).clicked() {
                    action = Some(AiPanelAction::ExecuteTask(state.task_input.clone()));
                    state.add_message(ChatRole::User, state.task_input.clone());
                    state.task_input.clear();
                }

                if state.is_running {
                    if ui.button("⏹ Stop").clicked() {
                        action = Some(AiPanelAction::StopTask);
                    }
                }

                if ui.button("🗑 Clear").clicked() {
                    state.clear_chat();
                }
            });

            // Progress
            if state.is_running {
                ui.separator();
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(&state.current_step);
                });
                ui.add(egui::ProgressBar::new(state.progress).show_percentage());
            }

            ui.separator();

            // Chat history
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    for msg in &state.chat_history {
                        let (icon, color) = match msg.role {
                            ChatRole::User => ("👤", egui::Color32::LIGHT_BLUE),
                            ChatRole::Assistant => ("🤖", egui::Color32::LIGHT_GREEN),
                            ChatRole::System => ("⚙", egui::Color32::LIGHT_GRAY),
                        };
                        
                        ui.horizontal_wrapped(|ui| {
                            ui.label(icon);
                            ui.label(egui::RichText::new(&msg.content).color(color));
                        });
                        ui.add_space(2.0);
                    }
                });

            // Action history
            ui.separator();
            ui.collapsing("Action History", |ui| {
                egui::ScrollArea::vertical()
                    .max_height(150.0)
                    .show(ui, |ui| {
                        for item in state.action_history.iter().rev() {
                            let status = if item.success { "✓" } else { "✗" };
                            let color = if item.success {
                                egui::Color32::GREEN
                            } else {
                                egui::Color32::RED
                            };

                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(status).color(color));
                                ui.label(&item.tool);
                                ui.label(egui::RichText::new(&item.timestamp).small().weak());
                            });
                        }
                    });
            });

            // Last message/error
            if let Some((success, ref msg)) = state.last_message {
                ui.separator();
                let color = if success {
                    egui::Color32::GREEN
                } else {
                    egui::Color32::RED
                };
                ui.label(egui::RichText::new(msg).color(color).small());
            }
        });

    action
}

/// Actions the AI panel can trigger
#[derive(Debug, Clone)]
pub enum AiPanelAction {
    /// Load the AI model
    LoadModel,
    /// Execute a task
    ExecuteTask(String),
    /// Stop the current task
    StopTask,
}
