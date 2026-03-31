// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pure display logic (testable, no egui).

use crate::graph_editor::streaming::{Alternative, ErrorInfo, NodeStatus, Pattern, ResourceUsage};

/// Node status display data: (icon, `color_rgb`, text).
#[must_use]
pub fn node_status_display(status: &NodeStatus) -> (&'static str, [u8; 3], String) {
    match status {
        NodeStatus::Pending => ("⚪", [128, 128, 128], "Pending".to_string()),
        NodeStatus::Running { progress } => ("🔵", [0, 128, 255], format!("Running ({progress}%)")),
        NodeStatus::Completed => ("✅", [0, 255, 0], "Completed".to_string()),
        NodeStatus::Failed { .. } => ("❌", [255, 0, 0], "Failed".to_string()),
        NodeStatus::Paused => ("⏸️", [255, 255, 0], "Paused".to_string()),
    }
}

/// Progress value (0.0–1.0) to percent display string.
#[must_use]
pub fn progress_percent_text(progress: f32) -> String {
    format!("{:.0}%", progress * 100.0)
}

/// Resource usage display strings: (cpu, memory, `disk_io`, network).
#[must_use]
pub fn resource_usage_display(resources: &ResourceUsage) -> (String, String, String, String) {
    (
        format!("{:.1}%", resources.cpu_percent),
        format!("{} MB", resources.memory_mb),
        format!("{:.1} MB/s", resources.disk_io_mbps),
        format!("{:.1} MB/s", resources.network_mbps),
    )
}

/// Alternative display data: (description, `confidence_str`, `reason_str`).
#[must_use]
pub fn alternative_display(alt: &Alternative) -> (&str, String, String) {
    (
        &alt.description,
        format!("({:.0}%)", alt.confidence * 100.0),
        format!("→ {}", alt.reason_not_chosen),
    )
}

/// Pattern display data: (description, `relevance_str`).
#[must_use]
pub fn pattern_display(pattern: &Pattern) -> (&str, String) {
    (
        &pattern.description,
        format!(
            "({}, {:.0}% relevant)",
            pattern.source,
            pattern.relevance * 100.0
        ),
    )
}

/// Confidence value (0.0–1.0) to RGB color.
#[must_use]
pub fn confidence_color_rgb(confidence: f32) -> [u8; 3] {
    if confidence > 0.8 {
        [0, 255, 0] // Green
    } else if confidence > 0.5 {
        [255, 255, 0] // Yellow
    } else {
        [255, 165, 0] // Orange
    }
}

#[must_use]
pub fn error_header_text(error: &ErrorInfo) -> String {
    format!("❌ {}", error.error_type)
}

#[must_use]
pub fn error_recoverable_display(error: &ErrorInfo) -> (String, Option<String>) {
    (
        if error.recoverable {
            "⚠️ Recoverable error".to_string()
        } else {
            "❌ Non-recoverable error".to_string()
        },
        error
            .suggested_action
            .as_ref()
            .map(|a| format!("💡 Suggestion: {a}")),
    )
}

#[must_use]
pub const fn error_recoverable_color_rgb(recoverable: bool) -> [u8; 3] {
    if recoverable {
        [255, 165, 0]
    } else {
        [255, 0, 0]
    }
}

#[must_use]
pub fn confidence_percent_text(confidence: f32) -> String {
    format!("{:.0}%", confidence * 100.0)
}

#[must_use]
pub fn format_confidence_display(confidence: f32) -> String {
    format!("Confidence: {}", confidence_percent_text(confidence))
}

#[must_use]
pub fn format_data_source_item(source: &str) -> String {
    format!("  • {source}")
}

#[must_use]
pub fn format_rationale_item(index: usize) -> String {
    format!("  {}.", index + 1)
}
