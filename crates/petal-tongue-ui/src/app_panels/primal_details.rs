// SPDX-License-Identifier: AGPL-3.0-or-later
//! Primal details panel - selected node property display.

use crate::accessibility::ColorPalette;
use petal_tongue_adapters::AdapterRegistry;
use petal_tongue_core::{
    DataBinding, GraphEngine, PrimalHealthStatus, PrimalInfo, Properties, PropertyValue,
};
use petal_tongue_graph::{NodeDetail, Visual2DRenderer, draw_node_detail};
use std::sync::{Arc, RwLock};

#[must_use]
fn build_properties_from_info(info: &PrimalInfo) -> Properties {
    let mut props = Properties::new();
    if let Some(trust_level) = info.trust_level() {
        props.insert(
            "trust_level".to_string(),
            PropertyValue::Number(f64::from(trust_level)),
        );
    }
    if let Some(family_id) = info.family_id() {
        props.insert(
            "family_id".to_string(),
            PropertyValue::String(family_id.to_string()),
        );
    }
    let cap_array: Vec<PropertyValue> = info
        .capabilities
        .iter()
        .map(|c| PropertyValue::String(c.clone()))
        .collect();
    props.insert("capabilities".to_string(), PropertyValue::Array(cap_array));
    props
}

/// Pure: health status icon (no egui)
#[must_use]
pub const fn health_status_icon(health: PrimalHealthStatus) -> &'static str {
    match health {
        PrimalHealthStatus::Healthy => "✅",
        PrimalHealthStatus::Warning => "⚠️",
        PrimalHealthStatus::Critical => "❌",
        PrimalHealthStatus::Unknown => "❓",
    }
}

/// Pure: health status RGB color [r, g, b] (no egui)
#[must_use]
pub const fn health_status_rgb(health: PrimalHealthStatus) -> [u8; 3] {
    match health {
        PrimalHealthStatus::Healthy => [0, 200, 0],
        PrimalHealthStatus::Warning => [255, 200, 0],
        PrimalHealthStatus::Critical => [255, 50, 50],
        PrimalHealthStatus::Unknown => [128, 128, 128],
    }
}

#[must_use]
fn format_last_seen_seconds(seconds_ago: u64) -> String {
    format!("{seconds_ago} seconds ago")
}

#[must_use]
fn extract_health_u8_from_properties(properties: &Properties) -> u8 {
    properties
        .get("health")
        .and_then(PropertyValue::as_number)
        .map_or(100, |n| {
            #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let v = n as u8;
            v
        })
}

// ============================================================================
// PrimalDetailsSummary - pure data preparation (testable, no egui)
// ============================================================================

/// Pre-computed summary for rendering primal details panel
pub struct PrimalDetailsSummary {
    pub name: String,
    pub id: String,
    pub primal_type: String,
    pub endpoint: String,
    pub health_icon: &'static str,
    pub health_color: [u8; 3],
    pub health_status_text: String,
    pub last_seen_text: String,
    pub properties: Properties,
    pub capabilities: Vec<String>,
    pub node_detail: Option<NodeDetail>,
}

impl PrimalDetailsSummary {
    /// Build summary from `PrimalInfo` (all data preparation logic)
    #[must_use]
    pub fn from_primal_info(info: &PrimalInfo, now_secs: u64) -> Self {
        let properties = if info.properties.is_empty() {
            build_properties_from_info(info)
        } else {
            info.properties.clone()
        };

        let health_icon = health_status_icon(info.health);
        let health_color = health_status_rgb(info.health);
        let health_status_text = format!("{:?}", info.health);

        let seconds_ago = now_secs.saturating_sub(info.last_seen);
        let last_seen_text = format_last_seen_seconds(seconds_ago);

        let node_detail = extract_node_detail_for_bindings(info, &properties);

        Self {
            name: info.name.clone(),
            id: info.id.to_string(),
            primal_type: info.primal_type.clone(),
            endpoint: info.endpoint.clone(),
            health_icon,
            health_color,
            health_status_text,
            last_seen_text,
            properties,
            capabilities: info.capabilities.clone(),
            node_detail,
        }
    }
}

/// Extract `NodeDetail` for data bindings section if present
fn extract_node_detail_for_bindings(
    info: &PrimalInfo,
    properties: &Properties,
) -> Option<NodeDetail> {
    let dc_json: Option<String> = properties
        .get("data_bindings_json")
        .or_else(|| properties.get("data_channels_json"))
        .and_then(|v| v.as_string().map(String::from))
        .or_else(|| {
            properties
                .get("data_bindings")
                .or_else(|| properties.get("data_channels"))
                .and_then(|v| serde_json::to_string(v).ok())
        });

    dc_json.and_then(|json| {
        serde_json::from_str::<Vec<DataBinding>>(&json)
            .ok()
            .and_then(|bindings| {
                if bindings.is_empty() {
                    None
                } else {
                    Some(NodeDetail {
                        name: info.name.clone(),
                        health: extract_health_u8_from_properties(properties),
                        status: info.primal_type.clone(),
                        capabilities: info.capabilities.clone(),
                        data_bindings: bindings,
                    })
                }
            })
    })
}

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
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let summary = PrimalDetailsSummary::from_primal_info(&node.info, now_secs);
        render_primal_details_summary(ui, &summary, palette, adapter_registry, visual_renderer);
    } else {
        ui.label(egui::RichText::new("Node not found").color(egui::Color32::RED));
    }
}

/// Render a pre-computed primal details summary (pure rendering, no data lookup)
fn render_primal_details_summary(
    ui: &mut egui::Ui,
    summary: &PrimalDetailsSummary,
    palette: &ColorPalette,
    adapter_registry: &AdapterRegistry,
    visual_renderer: &mut Visual2DRenderer,
) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(&summary.name).size(20.0).strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("✖").clicked() {
                visual_renderer.set_selected_node(None);
            }
        });
    });

    ui.add_space(8.0);

    ui.label(
        egui::RichText::new(format!("ID: {}", summary.id))
            .size(12.0)
            .color(egui::Color32::GRAY),
    );
    ui.add_space(4.0);

    ui.label(egui::RichText::new(format!("Type: {}", summary.primal_type)).size(14.0));
    ui.add_space(4.0);

    ui.label(
        egui::RichText::new(format!("📍 {}", summary.endpoint))
            .size(12.0)
            .color(palette.text_dim),
    );
    ui.add_space(12.0);

    let properties = &summary.properties;

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
                        egui::RichText::new("Trust level not available").color(egui::Color32::GRAY),
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
                        egui::RichText::new("Family ID not available").color(egui::Color32::GRAY),
                    );
                }
            });

        ui.add_space(12.0);
    }

    ui.separator();
    ui.add_space(8.0);
    ui.label(egui::RichText::new("🩺 Health Status").size(16.0).strong());
    ui.add_space(6.0);

    let health_color = egui::Color32::from_rgb(
        summary.health_color[0],
        summary.health_color[1],
        summary.health_color[2],
    );

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(summary.health_icon).size(24.0));
        ui.label(
            egui::RichText::new(&summary.health_status_text)
                .size(16.0)
                .color(health_color),
        );
    });

    ui.add_space(12.0);

    ui.separator();
    ui.add_space(8.0);

    if summary.capabilities.is_empty() {
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

    if let Some(ref detail) = summary.node_detail {
        ui.separator();
        ui.add_space(8.0);
        draw_node_detail(ui, detail);
        ui.add_space(12.0);
    }

    ui.separator();
    ui.add_space(8.0);
    ui.label(
        egui::RichText::new(format!("⏱️ Last seen: {}", summary.last_seen_text))
            .size(12.0)
            .color(egui::Color32::GRAY),
    );

    ui.add_space(16.0);
}

#[cfg(test)]
#[path = "primal_details_tests.rs"]
mod tests;
