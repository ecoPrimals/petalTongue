// SPDX-License-Identifier: AGPL-3.0-or-later
//! Helper functions for proprioception panel rendering.
//!
//! Formatting, color mapping, and shared SAME DAVE visualization components.

use egui::{Color32, ProgressBar, RichText, Ui};
use petal_tongue_core::ProprioceptionData;

#[must_use]
pub fn format_age_seconds(age_secs: i64) -> String {
    if age_secs < 60 {
        format!("{age_secs}s ago")
    } else {
        format!("{}m ago", age_secs / 60)
    }
}

#[must_use]
pub fn confidence_bar_color(confidence: f32) -> egui::Color32 {
    if confidence >= 80.0 {
        egui::Color32::from_rgb(34, 197, 94)
    } else if confidence >= 50.0 {
        egui::Color32::from_rgb(234, 179, 8)
    } else {
        egui::Color32::from_rgb(239, 68, 68)
    }
}

#[must_use]
pub const fn evaluative_status_text(is_healthy: bool, is_confident: bool) -> &'static str {
    if is_healthy && is_confident {
        "System is healthy and confident"
    } else if is_healthy {
        "System is healthy but low confidence"
    } else if is_confident {
        "System is confident but degraded"
    } else {
        "System requires attention"
    }
}

/// Shared rendering: health indicator with emoji + progress bar.
///
/// Used by both the main proprioception panel and the panel-registry version.
pub fn render_shared_health(ui: &mut Ui, health: &petal_tongue_core::proprioception::HealthData) {
    let emoji = health.status.emoji();
    let (r, g, b) = health.status.color_rgb();
    let color = Color32::from_rgb(r, g, b);

    ui.horizontal(|ui| {
        ui.label(RichText::new(emoji).size(24.0));
        ui.vertical(|ui| {
            ui.label(
                RichText::new(format!("Health: {:.1}%", health.percentage))
                    .size(18.0)
                    .color(color)
                    .strong(),
            );
            ui.label(RichText::new(format!("Status: {}", health.status)).color(color));
        });
    });

    ui.add(
        ProgressBar::new(health.percentage / 100.0)
            .show_percentage()
            .animate(true),
    );
}

/// Shared rendering: SAME DAVE data summary.
///
/// Used by both the main proprioception panel and the panel-registry version.
pub fn render_shared_same_dave(ui: &mut Ui, data: &ProprioceptionData) {
    ui.label(RichText::new(format!(
        "Confidence: {:.0}%",
        data.confidence
    )));
    ui.add(
        ProgressBar::new(data.confidence / 100.0)
            .show_percentage()
            .animate(true),
    );

    ui.separator();
    ui.label("SAME DAVE Assessment:");
    ui.add_space(2.0);

    ui.label("👁️ Sensory:");
    ui.label(format!(
        "  {} active sockets detected",
        data.sensory.active_sockets
    ));

    ui.add_space(2.0);
    ui.label("💭 Awareness:");
    ui.label(format!(
        "  Knows about {} primals",
        data.self_awareness.knows_about
    ));
    if data.self_awareness.can_coordinate {
        ui.label("  Can coordinate primals");
    }

    ui.add_space(2.0);
    ui.label("💪 Motor:");
    if data.motor.can_deploy {
        ui.label("  Can deploy primals");
    }
    if data.motor.can_execute_graphs {
        ui.label("  Can execute graphs");
    }
    if data.motor.can_coordinate_primals {
        ui.label("  Can coordinate primals");
    }

    ui.separator();

    ui.label("Core Systems:");
    let green = Color32::from_rgb(34, 197, 94);
    if data.self_awareness.has_security {
        ui.colored_label(green, "  Security (Entropy Source)");
    } else {
        ui.colored_label(Color32::GRAY, "  Security (Entropy Source) - not available");
    }
    if data.self_awareness.has_discovery {
        ui.colored_label(green, "  Discovery (Discovery Service)");
    } else {
        ui.colored_label(
            Color32::GRAY,
            "  Discovery (Discovery Service) - not available",
        );
    }
    if data.self_awareness.has_compute {
        ui.colored_label(green, "  Compute (Compute Backend)");
    } else {
        ui.colored_label(Color32::GRAY, "  Compute (Compute Backend) - not available");
    }

    ui.add_space(4.0);
    ui.label(format!("Family: {}", data.family_id));
}
