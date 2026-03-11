// SPDX-License-Identifier: AGPL-3.0-only
//! Node rendering for 2D graph visualization.

use crate::color_utils::hsv_to_rgb;
use egui::{Color32, Pos2, Stroke};
use petal_tongue_core::PrimalHealthStatus;
use petal_tongue_core::graph_engine::Node;

/// Draw a single node
pub fn draw_node(
    painter: &egui::Painter,
    node: &Node,
    screen_pos: Pos2,
    is_selected: bool,
    zoom: f32,
) {
    let radius = 20.0 * zoom;

    // Use trust level for color if available, otherwise fall back to health
    let trust_level = node
        .info
        .properties
        .get("trust_level")
        .and_then(|v| match v {
            petal_tongue_core::PropertyValue::Number(n) => {
                if *n >= 0.0 && *n <= 255.0 {
                    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let value = *n as u8;
                    Some(value)
                } else {
                    None
                }
            }
            _ => None,
        });

    let (fill_color, stroke_color) = if trust_level.is_some() {
        trust_level_to_colors(trust_level)
    } else {
        health_to_colors(node.info.health)
    };

    // Draw selection highlight
    if is_selected {
        painter.circle(
            screen_pos,
            radius + 5.0,
            Color32::TRANSPARENT,
            Stroke::new(3.0, Color32::YELLOW),
        );
    }

    // Draw family ID indicator (colored ring if present)
    if let Some(petal_tongue_core::PropertyValue::String(family_id)) =
        node.info.properties.get("family_id")
    {
        let family_color = family_id_to_color(family_id);
        painter.circle_stroke(screen_pos, radius + 3.0, Stroke::new(2.5, family_color));
    }

    // Draw node circle
    painter.circle(
        screen_pos,
        radius,
        fill_color,
        Stroke::new(2.0, stroke_color),
    );

    // Draw node label (if zoomed in enough)
    if zoom > 0.5 {
        let text = &node.info.name;
        painter.text(
            Pos2::new(screen_pos.x, screen_pos.y + radius + 10.0),
            egui::Align2::CENTER_TOP,
            text,
            egui::FontId::proportional(12.0),
            Color32::WHITE,
        );
    }

    // Draw trust level badge (if available and zoomed in)
    if zoom > 0.7 {
        if let Some(petal_tongue_core::PropertyValue::Number(trust_val)) =
            node.info.properties.get("trust_level")
        {
            if *trust_val >= 0.0 && *trust_val <= 255.0 {
                #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let trust_level = *trust_val as u8;
                let badge_text = match trust_level {
                    0 => "⚫",
                    1 => "🟡",
                    2 => "🟠",
                    3 => "🟢",
                    _ => "❓",
                };
                painter.text(
                    Pos2::new(screen_pos.x + radius, screen_pos.y - radius),
                    egui::Align2::LEFT_BOTTOM,
                    badge_text,
                    egui::FontId::proportional(14.0),
                    Color32::WHITE,
                );
            }
        }
    }

    // Draw capability badges (if zoomed in enough)
    if zoom > 0.9 && !node.info.capabilities.is_empty() {
        draw_capability_badges(painter, screen_pos, radius, zoom, &node.info.capabilities);
    }
}

/// Draw capability badges around the node
fn draw_capability_badges(
    painter: &egui::Painter,
    center: Pos2,
    radius: f32,
    zoom: f32,
    capabilities: &[String],
) {
    let badge_radius = 8.0 * zoom;
    let orbit_radius = radius + 15.0;

    let displayed_caps = capabilities.iter().take(6);
    let num_caps = displayed_caps.clone().count();

    for (i, capability) in displayed_caps.enumerate() {
        #[expect(clippy::cast_precision_loss)]
        let angle = (i as f32) * std::f32::consts::TAU / (num_caps as f32);
        let badge_pos = Pos2::new(
            center.x + orbit_radius * angle.cos(),
            center.y + orbit_radius * angle.sin(),
        );

        let (icon, color) = capability_to_icon_and_color(capability);

        painter.circle(
            badge_pos,
            badge_radius,
            color.gamma_multiply(0.3),
            Stroke::new(1.5, color),
        );

        painter.text(
            badge_pos,
            egui::Align2::CENTER_CENTER,
            icon,
            egui::FontId::proportional(10.0),
            Color32::WHITE,
        );
    }

    if capabilities.len() > 6 {
        let more_count = capabilities.len() - 6;
        let angle = std::f32::consts::TAU * 0.75;
        let badge_pos = Pos2::new(
            center.x + orbit_radius * angle.cos(),
            center.y + orbit_radius * angle.sin(),
        );

        painter.circle(
            badge_pos,
            badge_radius,
            Color32::DARK_GRAY,
            Stroke::new(1.5, Color32::GRAY),
        );

        painter.text(
            badge_pos,
            egui::Align2::CENTER_CENTER,
            format!("+{}", more_count),
            egui::FontId::proportional(8.0),
            Color32::WHITE,
        );
    }
}

/// Map capability to icon and color
fn capability_to_icon_and_color(capability: &str) -> (&'static str, Color32) {
    let cap_lower = capability.to_lowercase();

    if cap_lower.contains("security") || cap_lower.contains("trust") || cap_lower.contains("auth") {
        return ("🔒", Color32::from_rgb(255, 100, 100));
    }

    if cap_lower.contains("storage") || cap_lower.contains("persist") || cap_lower.contains("data")
    {
        return ("💾", Color32::from_rgb(100, 150, 255));
    }

    if cap_lower.contains("compute")
        || cap_lower.contains("container")
        || cap_lower.contains("workload")
        || cap_lower.contains("execution")
    {
        return ("⚙️", Color32::from_rgb(150, 200, 100));
    }

    if cap_lower.contains("discovery")
        || cap_lower.contains("orchestr")
        || cap_lower.contains("federation")
    {
        return ("🔍", Color32::from_rgb(200, 150, 255));
    }

    if cap_lower.contains("identity")
        || cap_lower.contains("lineage")
        || cap_lower.contains("genetic")
    {
        return ("🆔", Color32::from_rgb(255, 200, 100));
    }

    if cap_lower.contains("encrypt") || cap_lower.contains("crypto") || cap_lower.contains("sign") {
        return ("🔐", Color32::from_rgb(255, 150, 200));
    }

    if cap_lower.contains("ai")
        || cap_lower.contains("inference")
        || cap_lower.contains("intent")
        || cap_lower.contains("planning")
    {
        return ("🧠", Color32::from_rgb(200, 100, 255));
    }

    if cap_lower.contains("network")
        || cap_lower.contains("tcp")
        || cap_lower.contains("http")
        || cap_lower.contains("grpc")
    {
        return ("🌐", Color32::from_rgb(100, 200, 255));
    }

    if cap_lower.contains("attribution")
        || cap_lower.contains("provenance")
        || cap_lower.contains("audit")
    {
        return ("📋", Color32::from_rgb(255, 200, 150));
    }

    if cap_lower.contains("visual") || cap_lower.contains("ui") || cap_lower.contains("display") {
        return ("👁️", Color32::from_rgb(150, 255, 200));
    }

    if cap_lower.contains("audio")
        || cap_lower.contains("sound")
        || cap_lower.contains("sonification")
    {
        return ("🔊", Color32::from_rgb(255, 150, 100));
    }

    ("•", Color32::GRAY)
}

/// Map health status to colors
pub const fn health_to_colors(health: PrimalHealthStatus) -> (Color32, Color32) {
    match health {
        PrimalHealthStatus::Healthy => (
            Color32::from_rgb(40, 180, 40),
            Color32::from_rgb(20, 120, 20),
        ),
        PrimalHealthStatus::Warning => (
            Color32::from_rgb(200, 180, 40),
            Color32::from_rgb(140, 120, 20),
        ),
        PrimalHealthStatus::Critical => (
            Color32::from_rgb(200, 40, 40),
            Color32::from_rgb(140, 20, 20),
        ),
        PrimalHealthStatus::Unknown => (
            Color32::from_rgb(120, 120, 120),
            Color32::from_rgb(80, 80, 80),
        ),
    }
}

/// Map trust level to colors
pub const fn trust_level_to_colors(trust_level: Option<u8>) -> (Color32, Color32) {
    match trust_level {
        None | Some(0) => (
            Color32::from_rgb(100, 100, 100),
            Color32::from_rgb(60, 60, 60),
        ),
        Some(1) => (
            Color32::from_rgb(200, 180, 40),
            Color32::from_rgb(140, 120, 20),
        ),
        Some(2) => (
            Color32::from_rgb(220, 140, 40),
            Color32::from_rgb(160, 100, 20),
        ),
        Some(3) => (
            Color32::from_rgb(40, 200, 80),
            Color32::from_rgb(20, 140, 60),
        ),
        _ => (
            Color32::from_rgb(120, 120, 120),
            Color32::from_rgb(80, 80, 80),
        ),
    }
}

/// Map family ID to a consistent color
fn family_id_to_color(family_id: &str) -> Color32 {
    let hash: u32 = family_id.bytes().map(u32::from).sum();
    let hue = (hash % 360) as f32;
    let (r, g, b) = hsv_to_rgb(hue, 0.7, 0.9);
    Color32::from_rgb(r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::PrimalHealthStatus;

    #[test]
    fn test_health_to_colors_healthy() {
        let (fill, stroke) = health_to_colors(PrimalHealthStatus::Healthy);
        assert_eq!(fill, Color32::from_rgb(40, 180, 40));
        assert_eq!(stroke, Color32::from_rgb(20, 120, 20));
    }

    #[test]
    fn test_health_to_colors_warning() {
        let (fill, stroke) = health_to_colors(PrimalHealthStatus::Warning);
        assert_eq!(fill, Color32::from_rgb(200, 180, 40));
        assert_eq!(stroke, Color32::from_rgb(140, 120, 20));
    }

    #[test]
    fn test_health_to_colors_critical() {
        let (fill, stroke) = health_to_colors(PrimalHealthStatus::Critical);
        assert_eq!(fill, Color32::from_rgb(200, 40, 40));
        assert_eq!(stroke, Color32::from_rgb(140, 20, 20));
    }

    #[test]
    fn test_health_to_colors_unknown() {
        let (fill, stroke) = health_to_colors(PrimalHealthStatus::Unknown);
        assert_eq!(fill, Color32::from_rgb(120, 120, 120));
        assert_eq!(stroke, Color32::from_rgb(80, 80, 80));
    }

    #[test]
    fn test_trust_level_to_colors_none() {
        let (fill, _stroke) = trust_level_to_colors(None);
        assert_eq!(fill, Color32::from_rgb(100, 100, 100));
    }

    #[test]
    fn test_trust_level_to_colors_zero() {
        let (fill, _stroke) = trust_level_to_colors(Some(0));
        assert_eq!(fill, Color32::from_rgb(100, 100, 100));
    }

    #[test]
    fn test_trust_level_to_colors_one() {
        let (fill, _) = trust_level_to_colors(Some(1));
        assert_eq!(fill, Color32::from_rgb(200, 180, 40));
    }

    #[test]
    fn test_trust_level_to_colors_two() {
        let (fill, _) = trust_level_to_colors(Some(2));
        assert_eq!(fill, Color32::from_rgb(220, 140, 40));
    }

    #[test]
    fn test_trust_level_to_colors_three() {
        let (fill, _) = trust_level_to_colors(Some(3));
        assert_eq!(fill, Color32::from_rgb(40, 200, 80));
    }

    #[test]
    fn test_trust_level_to_colors_high() {
        let (fill, _) = trust_level_to_colors(Some(255));
        assert_eq!(fill, Color32::from_rgb(120, 120, 120));
    }
}
