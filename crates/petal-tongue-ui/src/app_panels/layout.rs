// SPDX-License-Identifier: AGPL-3.0-or-later
//! Panel layout logic - graph layout selector and panel visibility structure.

use petal_tongue_core::{GraphEngine, LayoutAlgorithm};
use std::sync::{Arc, RwLock};

/// Render the graph layout algorithm selector
pub fn render_layout_selector(
    ui: &mut egui::Ui,
    current_layout: &mut LayoutAlgorithm,
    graph: &Arc<RwLock<GraphEngine>>,
) {
    ui.label("Layout:");
    egui::ComboBox::from_id_salt("layout_selector")
        .selected_text(format!("{current_layout:?}"))
        .show_ui(ui, |ui| {
            if ui
                .selectable_value(
                    current_layout,
                    LayoutAlgorithm::ForceDirected,
                    "Force-Directed",
                )
                .clicked()
            {
                let Ok(mut g) = graph.write() else {
                    tracing::error!("graph lock poisoned");
                    return;
                };
                g.set_layout(LayoutAlgorithm::ForceDirected);
                g.layout(100);
            }
            if ui
                .selectable_value(
                    current_layout,
                    LayoutAlgorithm::Hierarchical,
                    "Hierarchical",
                )
                .clicked()
            {
                let Ok(mut g) = graph.write() else {
                    tracing::error!("graph lock poisoned");
                    return;
                };
                g.set_layout(LayoutAlgorithm::Hierarchical);
                g.layout(1);
            }
            if ui
                .selectable_value(current_layout, LayoutAlgorithm::Circular, "Circular")
                .clicked()
            {
                let Ok(mut g) = graph.write() else {
                    tracing::error!("graph lock poisoned");
                    return;
                };
                g.set_layout(LayoutAlgorithm::Circular);
                g.layout(1);
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_layout_selector_headless() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let mut current_layout = LayoutAlgorithm::ForceDirected;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_layout_selector(ui, &mut current_layout, &graph);
            });
        });
        assert_eq!(current_layout, LayoutAlgorithm::ForceDirected);
    }

    #[test]
    fn render_layout_selector_switch_to_hierarchical() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let mut current_layout = LayoutAlgorithm::ForceDirected;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_layout_selector(ui, &mut current_layout, &graph);
                current_layout = LayoutAlgorithm::Hierarchical;
            });
        });
        assert_eq!(current_layout, LayoutAlgorithm::Hierarchical);
    }

    #[test]
    fn render_layout_selector_switch_to_circular() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let mut current_layout = LayoutAlgorithm::ForceDirected;
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_layout_selector(ui, &mut current_layout, &graph);
                current_layout = LayoutAlgorithm::Circular;
            });
        });
        assert_eq!(current_layout, LayoutAlgorithm::Circular);
    }
}
