// SPDX-License-Identifier: AGPL-3.0-or-later
//! Graph Manager Panel
//!
//! UI for saving, loading, and executing graphs via Neural API.

use crate::accessibility::ColorPalette;

#[must_use]
fn is_graph_selected(selected_id: &Option<String>, graph_id: &str) -> bool {
    selected_id.as_deref() == Some(graph_id)
}

#[must_use]
fn save_description_opt(description: &str) -> Option<String> {
    if description.is_empty() {
        None
    } else {
        Some(description.to_string())
    }
}

#[must_use]
fn format_graph_stats(node_count: usize, edge_count: usize) -> (String, String) {
    (
        format!("📊 {node_count} nodes"),
        format!("🔗 {edge_count} edges"),
    )
}

#[must_use]
fn format_modified_at(modified_at: &str) -> String {
    format!("Modified: {modified_at}")
}

#[must_use]
fn format_execution_status(status: &str) -> String {
    format!("Execution: {status}")
}

#[must_use]
fn format_error_display(error: &str) -> String {
    format!("❌ Error: {error}")
}
use egui::{Color32, RichText, Ui};
use petal_tongue_core::graph_builder::VisualGraph;
use petal_tongue_discovery::{GraphMetadata, NeuralApiProvider, NeuralGraphClient};
use std::sync::Arc;
use std::time::Instant;

/// Graph manager panel state
pub struct GraphManagerPanel {
    /// Available graphs from Neural API
    available_graphs: Vec<GraphMetadata>,

    /// Last time graphs were refreshed
    last_refresh: Option<Instant>,

    /// Selected graph for loading
    selected_graph_id: Option<String>,

    /// Execution status message
    execution_status: Option<String>,

    /// Save dialog state
    save_name: String,
    save_description: String,
    show_save_dialog: bool,

    /// Error message
    error_message: Option<String>,
}

impl GraphManagerPanel {
    /// Create a new graph manager panel
    #[must_use]
    pub const fn new() -> Self {
        Self {
            available_graphs: Vec::new(),
            last_refresh: None,
            selected_graph_id: None,
            execution_status: None,
            save_name: String::new(),
            save_description: String::new(),
            show_save_dialog: false,
            error_message: None,
        }
    }

    /// Render the graph manager panel
    pub fn render(
        &mut self,
        ui: &mut Ui,
        palette: &ColorPalette,
        provider: Option<&Arc<NeuralApiProvider>>,
        current_graph: Option<&VisualGraph>,
        runtime: &tokio::runtime::Runtime,
    ) -> Option<GraphManagerAction> {
        let mut action = None;

        ui.heading(
            RichText::new("📊 Graph Manager")
                .size(16.0)
                .color(palette.accent),
        );

        ui.add_space(8.0);

        if let Some(provider) = provider {
            // Refresh button
            if ui.button("🔄 Refresh Graph List").clicked() {
                self.refresh_graphs(provider, runtime);
            }

            ui.add_space(12.0);

            // Current graph section
            if let Some(graph) = current_graph {
                ui.group(|ui| {
                    ui.label(RichText::new("Current Graph:").strong());
                    ui.label(format!("Name: {}", graph.name));
                    ui.label(format!("Nodes: {}", graph.nodes.len()));
                    ui.label(format!("Edges: {}", graph.edges.len()));

                    ui.add_space(8.0);

                    if ui.button("💾 Save to Neural API").clicked() {
                        self.show_save_dialog = true;
                        self.save_name = graph.name.clone();
                        self.save_description = graph.description.clone().unwrap_or_default();
                    }

                    if ui.button("🚀 Execute Graph").clicked() {
                        action = Some(GraphManagerAction::Execute);
                    }
                });

                ui.add_space(12.0);
            }

            // Save dialog
            if self.show_save_dialog {
                egui::Window::new("💾 Save Graph")
                    .collapsible(false)
                    .resizable(false)
                    .show(ui.ctx(), |ui| {
                        ui.label("Graph Name:");
                        ui.text_edit_singleline(&mut self.save_name);

                        ui.add_space(8.0);

                        ui.label("Description:");
                        ui.text_edit_multiline(&mut self.save_description);

                        ui.add_space(12.0);

                        ui.horizontal(|ui| {
                            if ui.button("✅ Save").clicked() {
                                action = Some(GraphManagerAction::Save {
                                    name: self.save_name.clone(),
                                    description: save_description_opt(&self.save_description),
                                });
                                self.show_save_dialog = false;
                            }

                            if ui.button("❌ Cancel").clicked() {
                                self.show_save_dialog = false;
                            }
                        });
                    });
            }

            // Available graphs list
            ui.separator();
            ui.label(RichText::new("Available Graphs:").strong());

            if self.available_graphs.is_empty() {
                ui.label(
                    RichText::new("No graphs available. Save a graph or refresh the list.")
                        .color(palette.text_dim),
                );
            } else {
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for graph_meta in &self.available_graphs {
                            ui.group(|ui| {
                                let is_selected =
                                    is_graph_selected(&self.selected_graph_id, &graph_meta.id);

                                let _bg_color = if is_selected {
                                    palette.accent.linear_multiply(0.2)
                                } else {
                                    palette.background_alt
                                };

                                let response = ui.add(
                                    egui::Label::new(
                                        RichText::new(&graph_meta.name).size(14.0).strong(),
                                    )
                                    .sense(egui::Sense::click()),
                                );

                                if response.clicked() {
                                    self.selected_graph_id = Some(graph_meta.id.clone());
                                }

                                if let Some(desc) = &graph_meta.description {
                                    ui.label(
                                        RichText::new(desc).size(11.0).color(palette.text_dim),
                                    );
                                }

                                let (nodes_str, edges_str) = format_graph_stats(
                                    graph_meta.node_count,
                                    graph_meta.edge_count,
                                );
                                ui.horizontal(|ui| {
                                    ui.label(&nodes_str);
                                    ui.label(&edges_str);
                                });

                                ui.label(
                                    RichText::new(format_modified_at(&graph_meta.modified_at))
                                        .size(10.0)
                                        .color(palette.text_dim),
                                );

                                if is_selected {
                                    ui.add_space(8.0);
                                    ui.horizontal(|ui| {
                                        if ui.button("📂 Load").clicked() {
                                            action = Some(GraphManagerAction::Load(
                                                graph_meta.id.clone(),
                                            ));
                                        }

                                        if ui.button("🗑️ Delete").clicked() {
                                            action = Some(GraphManagerAction::Delete(
                                                graph_meta.id.clone(),
                                            ));
                                        }
                                    });
                                }
                            });

                            ui.add_space(4.0);
                        }
                    });
            }

            // Execution status
            if let Some(status) = &self.execution_status {
                ui.add_space(12.0);
                ui.separator();
                ui.label(
                    RichText::new(format_execution_status(status))
                        .color(Color32::from_rgb(40, 180, 40)),
                );
            }

            // Error message
            if let Some(error) = &self.error_message {
                ui.add_space(12.0);
                ui.colored_label(Color32::from_rgb(208, 2, 27), format_error_display(error));
            }
        } else {
            ui.label(RichText::new("⚠️ Neural API not available").color(palette.text_dim));
            ui.label("Start biomeOS nucleus serve to enable graph management.");
        }

        action
    }

    /// Refresh the list of available graphs
    fn refresh_graphs(&mut self, provider: &NeuralApiProvider, runtime: &tokio::runtime::Runtime) {
        self.error_message = None;

        let client = NeuralGraphClient::new(provider);
        match runtime.block_on(async { client.list_graphs().await }) {
            Ok(graphs) => {
                self.available_graphs = graphs;
                self.last_refresh = Some(Instant::now());
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to refresh: {e}"));
            }
        }
    }

    /// Set execution status message
    pub fn set_execution_status(&mut self, status: Option<String>) {
        self.execution_status = status;
    }

    /// Set error message
    pub fn set_error(&mut self, error: Option<String>) {
        self.error_message = error;
    }

    /// Add a graph to the list (after saving)
    pub fn add_graph(&mut self, metadata: GraphMetadata) {
        self.available_graphs.push(metadata);
    }

    /// Remove a graph from the list (after deleting)
    pub fn remove_graph(&mut self, graph_id: &str) {
        self.available_graphs.retain(|g| g.id != graph_id);
        if self.selected_graph_id.as_deref() == Some(graph_id) {
            self.selected_graph_id = None;
        }
    }
}

impl Default for GraphManagerPanel {
    fn default() -> Self {
        Self::new()
    }
}

/// Actions that the graph manager can request
#[derive(Debug, Clone)]
pub enum GraphManagerAction {
    /// Save current graph with given name and description
    Save {
        /// Name for the saved graph
        name: String,
        /// Optional description of the graph
        description: Option<String>,
    },

    /// Load a graph by ID
    Load(String),

    /// Execute the current graph
    Execute,

    /// Delete a graph by ID
    Delete(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_creation() {
        let panel = GraphManagerPanel::new();
        assert!(panel.available_graphs.is_empty());
        assert!(panel.selected_graph_id.is_none());
        assert!(!panel.show_save_dialog);
    }

    #[test]
    fn test_add_remove_graph() {
        let mut panel = GraphManagerPanel::new();

        let metadata = GraphMetadata {
            id: "test-123".to_string(),
            name: "Test Graph".to_string(),
            description: Some("A test".to_string()),
            created_at: "2026-01-15T00:00:00Z".to_string(),
            modified_at: "2026-01-15T01:00:00Z".to_string(),
            node_count: 5,
            edge_count: 4,
        };

        panel.add_graph(metadata);
        assert_eq!(panel.available_graphs.len(), 1);

        panel.remove_graph("test-123");
        assert_eq!(panel.available_graphs.len(), 0);
    }

    #[test]
    fn test_set_error() {
        let mut panel = GraphManagerPanel::new();
        assert!(panel.error_message.is_none());

        panel.set_error(Some("Test error".to_string()));
        assert_eq!(panel.error_message, Some("Test error".to_string()));

        panel.set_error(None);
        assert!(panel.error_message.is_none());
    }

    #[test]
    fn test_set_execution_status() {
        let mut panel = GraphManagerPanel::new();
        assert!(panel.execution_status.is_none());

        panel.set_execution_status(Some("Running...".to_string()));
        assert_eq!(panel.execution_status, Some("Running...".to_string()));

        panel.set_execution_status(None);
        assert!(panel.execution_status.is_none());
    }

    #[test]
    fn test_remove_graph_clears_selected() {
        let mut panel = GraphManagerPanel::new();
        let metadata = GraphMetadata {
            id: "graph-a".to_string(),
            name: "Graph A".to_string(),
            description: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            modified_at: "2026-01-01T01:00:00Z".to_string(),
            node_count: 2,
            edge_count: 1,
        };
        panel.add_graph(metadata);
        panel.selected_graph_id = Some("graph-a".to_string());

        panel.remove_graph("graph-a");
        assert_eq!(panel.available_graphs.len(), 0);
        assert!(panel.selected_graph_id.is_none());
    }

    #[test]
    fn test_remove_graph_keeps_others() {
        let mut panel = GraphManagerPanel::new();
        panel.add_graph(GraphMetadata {
            id: "g1".to_string(),
            name: "G1".to_string(),
            description: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            modified_at: "2026-01-01T01:00:00Z".to_string(),
            node_count: 1,
            edge_count: 0,
        });
        panel.add_graph(GraphMetadata {
            id: "g2".to_string(),
            name: "G2".to_string(),
            description: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            modified_at: "2026-01-01T01:00:00Z".to_string(),
            node_count: 2,
            edge_count: 1,
        });

        panel.remove_graph("g1");
        assert_eq!(panel.available_graphs.len(), 1);
        assert_eq!(panel.available_graphs[0].id, "g2");
    }

    #[test]
    fn test_graph_manager_action_variants() {
        let save = GraphManagerAction::Save {
            name: "test".to_string(),
            description: Some("desc".to_string()),
        };
        match &save {
            GraphManagerAction::Save { name, description } => {
                assert_eq!(name, "test");
                assert_eq!(description.as_deref(), Some("desc"));
            }
            _ => panic!("expected Save"),
        }

        let load = GraphManagerAction::Load("id-123".to_string());
        match &load {
            GraphManagerAction::Load(id) => assert_eq!(id, "id-123"),
            _ => panic!("expected Load"),
        }

        let _ = GraphManagerAction::Execute;
        let delete = GraphManagerAction::Delete("del-id".to_string());
        match &delete {
            GraphManagerAction::Delete(id) => assert_eq!(id, "del-id"),
            _ => panic!("expected Delete"),
        }
    }

    #[test]
    fn test_panel_default() {
        let panel = GraphManagerPanel::default();
        assert!(panel.available_graphs.is_empty());
        assert!(panel.save_name.is_empty());
        assert!(panel.save_description.is_empty());
    }

    #[test]
    fn test_save_action_empty_description() {
        let save = GraphManagerAction::Save {
            name: "graph".to_string(),
            description: None,
        };
        match &save {
            GraphManagerAction::Save { name, description } => {
                assert_eq!(name, "graph");
                assert!(description.is_none());
            }
            _ => panic!("expected Save"),
        }
    }

    #[test]
    fn test_remove_graph_nonexistent_id() {
        let mut panel = GraphManagerPanel::new();
        panel.add_graph(GraphMetadata {
            id: "g1".to_string(),
            name: "G1".to_string(),
            description: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            modified_at: "2026-01-01T01:00:00Z".to_string(),
            node_count: 1,
            edge_count: 0,
        });
        panel.remove_graph("nonexistent");
        assert_eq!(panel.available_graphs.len(), 1);
    }

    #[test]
    fn test_add_multiple_graphs() {
        let mut panel = GraphManagerPanel::new();
        for i in 0..5 {
            panel.add_graph(GraphMetadata {
                id: format!("g{i}"),
                name: format!("Graph {i}"),
                description: None,
                created_at: "2026-01-01T00:00:00Z".to_string(),
                modified_at: "2026-01-01T01:00:00Z".to_string(),
                node_count: i,
                edge_count: i.saturating_sub(1),
            });
        }
        assert_eq!(panel.available_graphs.len(), 5);
    }

    #[test]
    fn test_graph_manager_action_clone() {
        let save = GraphManagerAction::Save {
            name: "x".to_string(),
            description: Some("y".to_string()),
        };
        let cloned = save.clone();
        match (&save, &cloned) {
            (
                GraphManagerAction::Save { name: a, .. },
                GraphManagerAction::Save { name: b, .. },
            ) => assert_eq!(a, b),
            _ => panic!(),
        }

        let load = GraphManagerAction::Load("id".to_string());
        let _ = load.clone();

        let _ = GraphManagerAction::Execute.clone();
        let del = GraphManagerAction::Delete("d".to_string());
        let _ = del.clone();
    }

    #[test]
    fn test_graph_manager_action_debug() {
        let s = format!("{:?}", GraphManagerAction::Execute);
        assert!(s.contains("Execute"));

        let s = format!("{:?}", GraphManagerAction::Load("x".to_string()));
        assert!(s.contains("Load"));
        assert!(s.contains("x"));
    }

    #[test]
    fn test_panel_initial_state() {
        let panel = GraphManagerPanel::new();
        assert!(panel.last_refresh.is_none());
        assert!(panel.execution_status.is_none());
        assert!(panel.error_message.is_none());
        assert!(!panel.show_save_dialog);
    }

    #[test]
    fn test_is_graph_selected() {
        assert!(is_graph_selected(&Some("a".to_string()), "a"));
        assert!(!is_graph_selected(&Some("a".to_string()), "b"));
        assert!(!is_graph_selected(&None, "a"));
    }

    #[test]
    fn test_save_description_opt() {
        assert_eq!(save_description_opt(""), None);
        assert_eq!(save_description_opt("desc"), Some("desc".to_string()));
    }

    #[test]
    fn test_format_graph_stats() {
        let (n, e) = format_graph_stats(5, 3);
        assert_eq!(n, "📊 5 nodes");
        assert_eq!(e, "🔗 3 edges");
    }

    #[test]
    fn test_format_modified_at() {
        assert_eq!(format_modified_at("2026-01-01"), "Modified: 2026-01-01");
    }

    #[test]
    fn test_format_execution_status() {
        assert!(format_execution_status("Running").contains("Execution:"));
    }

    #[test]
    fn test_format_error_display() {
        assert!(format_error_display("fail").contains("Error:"));
    }

    #[test]
    fn test_remove_graph_does_not_clear_other_selection() {
        let mut panel = GraphManagerPanel::new();
        panel.add_graph(GraphMetadata {
            id: "g1".to_string(),
            name: "G1".to_string(),
            description: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            modified_at: "2026-01-01T01:00:00Z".to_string(),
            node_count: 1,
            edge_count: 0,
        });
        panel.add_graph(GraphMetadata {
            id: "g2".to_string(),
            name: "G2".to_string(),
            description: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            modified_at: "2026-01-01T01:00:00Z".to_string(),
            node_count: 2,
            edge_count: 1,
        });
        panel.selected_graph_id = Some("g2".to_string());
        panel.remove_graph("g1");
        assert_eq!(panel.selected_graph_id.as_deref(), Some("g2"));
    }
}
