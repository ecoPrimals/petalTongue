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
