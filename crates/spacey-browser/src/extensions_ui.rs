//! Extension Management UI
//!
//! Provides a panel for browsing, installing, and managing extensions.

use crate::extensions::{
    Extension, AddonSummary, SearchResults,
};

/// State for the extensions UI panel
#[derive(Default)]
pub struct ExtensionsUiState {
    /// Whether the panel is visible
    pub panel_visible: bool,
    /// Current tab
    pub current_tab: ExtensionsTab,
    /// Search query
    pub search_query: String,
    /// Search results from AMO
    pub search_results: Option<SearchResults>,
    /// Featured extensions
    pub featured: Option<Vec<AddonSummary>>,
    /// Recommended blockers
    pub recommended_blockers: Option<Vec<AddonSummary>>,
    /// Currently installing extension
    pub installing: Option<String>,
    /// Last error message
    pub last_error: Option<String>,
    /// Success message
    pub success_message: Option<String>,
    /// Whether we're loading
    pub loading: bool,
    /// Pending action
    pub pending_action: Option<ExtensionsAction>,
}

/// Tab in the extensions panel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExtensionsTab {
    #[default]
    Installed,
    Browse,
    Blockers,
}

impl ExtensionsUiState {
    pub fn new() -> Self {
        Self {
            panel_visible: false,
            ..Default::default()
        }
    }

    pub fn toggle_panel(&mut self) {
        self.panel_visible = !self.panel_visible;
    }

    pub fn show_error(&mut self, message: impl Into<String>) {
        self.last_error = Some(message.into());
        self.success_message = None;
    }

    pub fn show_success(&mut self, message: impl Into<String>) {
        self.success_message = Some(message.into());
        self.last_error = None;
    }

    pub fn clear_messages(&mut self) {
        self.last_error = None;
        self.success_message = None;
    }

    pub fn take_action(&mut self) -> Option<ExtensionsAction> {
        self.pending_action.take()
    }
}

/// Actions triggered by the extensions UI
#[derive(Debug, Clone)]
pub enum ExtensionsAction {
    /// Search AMO for extensions
    Search(String),
    /// Install an extension from AMO
    InstallFromAmo(String),
    /// Install from local XPI file
    InstallXpi(std::path::PathBuf),
    /// Uninstall an extension
    Uninstall(String),
    /// Enable an extension
    Enable(String),
    /// Disable an extension
    Disable(String),
    /// Load featured extensions
    LoadFeatured,
    /// Load recommended blockers
    LoadBlockers,
    /// Reload an extension
    Reload(String),
}

/// Render the extensions panel
pub fn render_extensions_panel(
    ctx: &egui::Context,
    state: &mut ExtensionsUiState,
    installed: &[Extension],
) -> Option<ExtensionsAction> {
    if !state.panel_visible {
        return None;
    }

    egui::Window::new("🧩 Extensions")
        .default_width(600.0)
        .default_height(500.0)
        .collapsible(true)
        .resizable(true)
        .show(ctx, |ui| {
            // Tab bar
            ui.horizontal(|ui| {
                if ui.selectable_label(state.current_tab == ExtensionsTab::Installed, "📦 Installed").clicked() {
                    state.current_tab = ExtensionsTab::Installed;
                }
                if ui.selectable_label(state.current_tab == ExtensionsTab::Browse, "🔍 Browse AMO").clicked() {
                    state.current_tab = ExtensionsTab::Browse;
                    if state.featured.is_none() && !state.loading {
                        state.pending_action = Some(ExtensionsAction::LoadFeatured);
                    }
                }
                if ui.selectable_label(state.current_tab == ExtensionsTab::Blockers, "🛡️ Content Blockers").clicked() {
                    state.current_tab = ExtensionsTab::Blockers;
                    if state.recommended_blockers.is_none() && !state.loading {
                        state.pending_action = Some(ExtensionsAction::LoadBlockers);
                    }
                }
            });

            ui.separator();

            // Messages
            if let Some(ref error) = state.last_error {
                ui.colored_label(egui::Color32::RED, format!("❌ {}", error));
            }
            if let Some(ref success) = state.success_message {
                ui.colored_label(egui::Color32::GREEN, format!("✅ {}", success));
            }

            // Loading indicator
            if state.loading {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Loading...");
                });
            }

            ui.separator();

            // Tab content
            match state.current_tab {
                ExtensionsTab::Installed => {
                    render_installed_tab(ui, state, installed);
                }
                ExtensionsTab::Browse => {
                    render_browse_tab(ui, state);
                }
                ExtensionsTab::Blockers => {
                    render_blockers_tab(ui, state);
                }
            }
        });

    state.take_action()
}

/// Render the installed extensions tab
fn render_installed_tab(
    ui: &mut egui::Ui,
    state: &mut ExtensionsUiState,
    installed: &[Extension],
) {
    if installed.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.heading("No extensions installed");
            ui.label("Browse the Firefox Add-ons marketplace to find extensions");
            if ui.button("Browse Extensions").clicked() {
                state.current_tab = ExtensionsTab::Browse;
            }
        });
        return;
    }

    ui.label(format!("{} extensions installed", installed.len()));
    ui.add_space(10.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        for ext in installed {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    // Extension info
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            let status = if ext.enabled { "🟢" } else { "⚪" };
                            ui.label(status);
                            ui.heading(&ext.manifest.name);
                            ui.label(format!("v{}", ext.manifest.version));
                        });
                        
                        if !ext.manifest.description.is_empty() {
                            ui.label(&ext.manifest.description);
                        }
                        
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(&ext.id).small().weak());
                            if ext.temporary {
                                ui.label(egui::RichText::new("(temporary)").small().italics());
                            }
                        });
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Actions
                        if ui.button("🗑").on_hover_text("Uninstall").clicked() {
                            state.pending_action = Some(ExtensionsAction::Uninstall(ext.id.clone()));
                        }
                        
                        if ext.enabled {
                            if ui.button("⏸").on_hover_text("Disable").clicked() {
                                state.pending_action = Some(ExtensionsAction::Disable(ext.id.clone()));
                            }
                        } else if ui.button("▶").on_hover_text("Enable").clicked() {
                            state.pending_action = Some(ExtensionsAction::Enable(ext.id.clone()));
                        }
                        
                        if ui.button("🔄").on_hover_text("Reload").clicked() {
                            state.pending_action = Some(ExtensionsAction::Reload(ext.id.clone()));
                        }
                    });
                });

                // Show permissions for webRequest blocking
                if ext.manifest.needs_blocking_webrequest() {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("🛡️ Full content blocking support").small().color(egui::Color32::GREEN));
                    });
                }
            });
            ui.add_space(5.0);
        }
    });
}

/// Render the browse AMO tab
fn render_browse_tab(
    ui: &mut egui::Ui,
    state: &mut ExtensionsUiState,
) {
    // Search bar
    ui.horizontal(|ui| {
        ui.label("Search:");
        let response = ui.text_edit_singleline(&mut state.search_query);
        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if !state.search_query.is_empty() {
                state.pending_action = Some(ExtensionsAction::Search(state.search_query.clone()));
            }
        }
        if ui.button("🔍").clicked() && !state.search_query.is_empty() {
            state.pending_action = Some(ExtensionsAction::Search(state.search_query.clone()));
        }
    });

    ui.separator();

    // Clone data to avoid borrow issues
    let search_results = state.search_results.clone();
    let featured = state.featured.clone();
    let installing = state.installing.clone();

    // Results
    egui::ScrollArea::vertical().show(ui, |ui| {
        if let Some(results) = search_results {
            ui.label(format!("{} results found", results.count));
            ui.add_space(5.0);
            
            for addon in &results.results {
                if let Some(action) = render_addon_card(ui, addon, &installing) {
                    state.pending_action = Some(action);
                }
            }
        } else if let Some(featured) = featured {
            ui.heading("Featured Extensions");
            ui.add_space(5.0);
            
            for addon in &featured {
                if let Some(action) = render_addon_card(ui, addon, &installing) {
                    state.pending_action = Some(action);
                }
            }
        } else {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label("Search for extensions or browse featured add-ons");
            });
        }
    });
}

/// Render the content blockers tab
fn render_blockers_tab(
    ui: &mut egui::Ui,
    state: &mut ExtensionsUiState,
) {
    ui.heading("🛡️ Recommended Content Blockers");
    ui.add_space(5.0);
    
    ui.label("Unlike Chrome's Manifest V3, Spacey Browser provides FULL blocking support.");
    ui.label("These extensions work at full power with our complete webRequest API.");
    ui.add_space(10.0);

    ui.colored_label(
        egui::Color32::GREEN,
        "✓ Full webRequest.onBeforeRequest with blocking",
    );
    ui.colored_label(
        egui::Color32::GREEN,
        "✓ Unlimited filter rules (no 30K static rule limit)",
    );
    ui.colored_label(
        egui::Color32::GREEN,
        "✓ Dynamic rule updates without extension reload",
    );
    ui.colored_label(
        egui::Color32::GREEN,
        "✓ Access to request/response bodies",
    );
    
    ui.add_space(10.0);
    ui.separator();

    // Clone to avoid borrow issues
    let blockers = state.recommended_blockers.clone();
    let installing = state.installing.clone();

    egui::ScrollArea::vertical().show(ui, |ui| {
        if let Some(blockers) = blockers {
            for addon in &blockers {
                if let Some(action) = render_addon_card(ui, addon, &installing) {
                    state.pending_action = Some(action);
                }
            }
        } else {
            ui.vertical_centered(|ui| {
                ui.add_space(30.0);
                ui.spinner();
                ui.label("Loading recommended blockers...");
            });
        }
    });
}

/// Render an addon card
fn render_addon_card(
    ui: &mut egui::Ui,
    addon: &AddonSummary,
    installing: &Option<String>,
) -> Option<ExtensionsAction> {
    let mut action = None;

    ui.group(|ui| {
        ui.horizontal(|ui| {
            // Addon info
            ui.vertical(|ui| {
                ui.heading(addon.name.get());
                
                if let Some(ref summary) = addon.summary {
                    ui.label(summary.get());
                }
                
                ui.horizontal(|ui| {
                    if let Some(ref ratings) = addon.ratings {
                        let stars = "★".repeat((ratings.average.round() as usize).min(5));
                        let empty = "☆".repeat(5 - (ratings.average.round() as usize).min(5));
                        ui.label(format!("{}{} ({} reviews)", stars, empty, ratings.count));
                    }
                    
                    if let Some(users) = addon.average_daily_users {
                        ui.label(format!("👥 {} users", format_number(users)));
                    }
                });
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let is_installing = installing.as_ref() == Some(&addon.slug);
                
                if is_installing {
                    ui.spinner();
                } else if ui.button("➕ Install").clicked() {
                    action = Some(ExtensionsAction::InstallFromAmo(addon.slug.clone()));
                }
            });
        });
    });
    ui.add_space(5.0);

    action
}

/// Format a large number with K/M suffixes
fn format_number(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}
