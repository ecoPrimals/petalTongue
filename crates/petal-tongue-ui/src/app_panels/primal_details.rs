// SPDX-License-Identifier: AGPL-3.0-only
//! Primal details panel - selected node property display.

use crate::accessibility::ColorPalette;
use petal_tongue_adapters::AdapterRegistry;
use petal_tongue_core::{DataBinding, GraphEngine, PrimalHealthStatus};
use petal_tongue_graph::{NodeDetail, Visual2DRenderer, draw_node_detail};
use std::sync::{Arc, RwLock};

/// Render the primal details panel for a selected node
pub fn render_primal_details_panel(
    ui: &mut egui::Ui,
    selected_id: &str,
    palette: &ColorPalette,
    graph: &Arc<RwLock<GraphEngine>>,
    adapter_registry: &AdapterRegistry,
    visual_renderer: &mut Visual2DRenderer,
) {
    ui.heading("🔍 Primal Details");
    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);

    let Ok(graph) = graph.read() else {
        tracing::error!("graph lock poisoned");
        ui.label(egui::RichText::new("Failed to access graph").color(egui::Color32::RED));
        return;
    };
    let primal_node = graph
        .nodes()
        .iter()
        .find(|n| n.info.id.as_str() == selected_id);

    if let Some(node) = primal_node {
        let info = &node.info;

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(&info.name).size(20.0).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("✖").clicked() {
                    visual_renderer.set_selected_node(None);
                }
            });
        });

        ui.add_space(8.0);

        ui.label(
            egui::RichText::new(format!("ID: {}", info.id))
                .size(12.0)
                .color(egui::Color32::GRAY),
        );
        ui.add_space(4.0);

        ui.label(egui::RichText::new(format!("Type: {}", info.primal_type)).size(14.0));
        ui.add_space(4.0);

        ui.label(
            egui::RichText::new(format!("📍 {}", info.endpoint))
                .size(12.0)
                .color(palette.text_dim),
        );
        ui.add_space(12.0);

        let properties = if info.properties.is_empty() {
            use petal_tongue_core::{Properties, PropertyValue};
            let mut props = Properties::new();

            #[expect(deprecated)]
            if let Some(trust_level) = info.trust_level {
                props.insert(
                    "trust_level".to_string(),
                    PropertyValue::Number(f64::from(trust_level)),
                );
            }

            #[expect(deprecated)]
            if let Some(family_id) = &info.family_id {
                props.insert(
                    "family_id".to_string(),
                    PropertyValue::String(family_id.clone()),
                );
            }

            let cap_array: Vec<PropertyValue> = info
                .capabilities
                .iter()
                .map(|c| PropertyValue::String(c.clone()))
                .collect();
            props.insert("capabilities".to_string(), PropertyValue::Array(cap_array));

            props
        } else {
            info.properties.clone()
        };

        if properties.contains_key("trust_level") {
            ui.separator();
            ui.add_space(8.0);
            ui.label(egui::RichText::new("🔒 Trust Level").size(16.0).strong());
            ui.add_space(6.0);

            egui::Frame::none()
                .fill(egui::Color32::from_rgb(40, 40, 45))
                .inner_margin(12.0)
                .show(ui, |ui| {
                    if let Some(trust_value) = properties.get("trust_level") {
                        adapter_registry.render_property("trust_level", trust_value, ui);
                    } else {
                        ui.label(
                            egui::RichText::new("Trust level not available")
                                .color(egui::Color32::GRAY),
                        );
                    }
                });

            ui.add_space(12.0);
        }

        if properties.contains_key("family_id") {
            ui.separator();
            ui.add_space(8.0);
            ui.label(egui::RichText::new("👨‍👩‍👧‍👦 Family Lineage").size(16.0).strong());
            ui.add_space(6.0);

            egui::Frame::none()
                .fill(egui::Color32::from_rgb(30, 40, 60))
                .inner_margin(12.0)
                .show(ui, |ui| {
                    if let Some(family_value) = properties.get("family_id") {
                        adapter_registry.render_property("family_id", family_value, ui);
                    } else {
                        ui.label(
                            egui::RichText::new("Family ID not available")
                                .color(egui::Color32::GRAY),
                        );
                    }
                });

            ui.add_space(12.0);
        }

        ui.separator();
        ui.add_space(8.0);
        ui.label(egui::RichText::new("🩺 Health Status").size(16.0).strong());
        ui.add_space(6.0);

        let (health_icon, health_color) = match info.health {
            PrimalHealthStatus::Healthy => ("✅", egui::Color32::from_rgb(0, 200, 0)),
            PrimalHealthStatus::Warning => ("⚠️", egui::Color32::from_rgb(255, 200, 0)),
            PrimalHealthStatus::Critical => ("❌", egui::Color32::from_rgb(255, 50, 50)),
            PrimalHealthStatus::Unknown => ("❓", egui::Color32::GRAY),
        };

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(health_icon).size(24.0));
            ui.label(
                egui::RichText::new(format!("{:?}", info.health))
                    .size(16.0)
                    .color(health_color),
            );
        });

        ui.add_space(12.0);

        ui.separator();
        ui.add_space(8.0);

        if info.capabilities.is_empty() {
            ui.label(egui::RichText::new("⚙️ Capabilities").size(16.0).strong());
            ui.add_space(6.0);
            ui.label(egui::RichText::new("No capabilities listed").color(egui::Color32::GRAY));
        } else {
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    if let Some(caps_value) = properties.get("capabilities") {
                        adapter_registry.render_property("capabilities", caps_value, ui);
                    } else {
                        ui.label(
                            egui::RichText::new("Capabilities not available")
                                .color(egui::Color32::GRAY),
                        );
                    }
                });
        }

        ui.add_space(12.0);

        // Data binding rendering (scenario data_bindings)
        let dc_json: Option<String> = properties
            .get("data_bindings_json")
            .or_else(|| properties.get("data_channels_json"))
            .and_then(|v| v.as_string().map(String::from))
            .or_else(|| {
                // DynamicScenarioProvider stores as PropertyValue tree — re-serialize
                properties
                    .get("data_bindings")
                    .or_else(|| properties.get("data_channels"))
                    .and_then(|v| serde_json::to_string(v).ok())
            });

        if let Some(ref json) = dc_json
            && let Ok(bindings) = serde_json::from_str::<Vec<DataBinding>>(json)
            && !bindings.is_empty()
        {
            ui.separator();
            ui.add_space(8.0);
            let health_u8 = properties
                .get("health")
                .and_then(petal_tongue_core::PropertyValue::as_number)
                .map_or(100, |n| {
                    #[expect(
                        clippy::cast_possible_truncation,
                        clippy::cast_sign_loss,
                        reason = "health 0..100"
                    )]
                    let v = n as u8;
                    v
                });
            let detail = NodeDetail {
                name: info.name.clone(),
                health: health_u8,
                status: info.primal_type.clone(),
                capabilities: info.capabilities.clone(),
                data_bindings: bindings,
            };
            draw_node_detail(ui, &detail);
            ui.add_space(12.0);
        }

        ui.separator();
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new(format!(
                "⏱️ Last seen: {} seconds ago",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
                    .saturating_sub(info.last_seen)
            ))
            .size(12.0)
            .color(egui::Color32::GRAY),
        );

        ui.add_space(16.0);

        ui.separator();
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button("🔍 Query Primal").clicked() {
                tracing::info!("Query primal: {}", info.id);
            }
            if ui.button("📊 View Logs").clicked() {
                tracing::info!("View logs for: {}", info.id);
            }
        });
    } else {
        ui.label(egui::RichText::new("Node not found").color(egui::Color32::RED));
    }
}
