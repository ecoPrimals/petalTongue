// SPDX-License-Identifier: AGPL-3.0-or-later
//! Graph Manager Panel
//!
//! UI for saving, loading, and executing graphs via Neural API.

use crate::accessibility::ColorPalette;

#[must_use]
fn is_graph_selected(selected_id: Option<&String>, graph_id: &str) -> bool {
    selected_id.is_some_and(|s| s.as_str() == graph_id)
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
                        self.save_name.clone_from(&graph.name);
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
                                let is_selected = is_graph_selected(
                                    self.selected_graph_id.as_ref(),
                                    graph_meta.id.as_str(),
                                );

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
mod tests;
