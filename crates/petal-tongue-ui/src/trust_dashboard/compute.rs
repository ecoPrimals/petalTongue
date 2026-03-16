// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pure functions for trust display computation (fully testable, no egui context).

use super::types::{AverageTrustDisplay, TrustDisplayState, TrustLevelRow, TrustSummary};

/// Map a trust level label + count to a display row (emoji, color, percentage).
#[must_use]
pub fn trust_level_to_display_row(label: &str, count: usize, total: usize) -> TrustLevelRow {
    let percentage = if total > 0 {
        (count as f32 / total as f32) * 100.0
    } else {
        0.0
    };
    let (emoji, color) = trust_level_style(label);
    TrustLevelRow {
        label: label.to_string(),
        count,
        percentage,
        emoji,
        color,
    }
}

/// Determine emoji and color for a trust level label string.
#[must_use]
pub fn trust_level_style(label: &str) -> (&'static str, [u8; 4]) {
    if label.contains("Full") || label.contains("(3)") {
        ("🟢", [76, 175, 80, 255])
    } else if label.contains("Elevated") || label.contains("(2)") {
        ("🟠", [255, 152, 0, 255])
    } else if label.contains("Limited") || label.contains("(1)") {
        ("🟡", [255, 235, 59, 255])
    } else {
        ("⚫", [158, 158, 158, 255])
    }
}

/// Map an average trust value to its display representation.
#[must_use]
pub const fn average_trust_display(avg: f64) -> AverageTrustDisplay {
    let (emoji, color, label, sound_name) = match avg.round() as i32 {
        0 => ("⚫", [158, 158, 158, 255], "None", "error"),
        1 => ("🟡", [255, 235, 59, 255], "Limited", "warning"),
        2 => ("🟠", [255, 152, 0, 255], "Elevated", "notification"),
        3 => ("🟢", [76, 175, 80, 255], "Full", "success"),
        _ => ("❓", [158, 158, 158, 255], "Unknown", "notification"),
    };
    AverageTrustDisplay {
        value: avg,
        emoji,
        color,
        label,
        sound_name,
    }
}

/// Build the complete display state from a `TrustSummary`.
#[must_use]
pub fn prepare_trust_display(summary: &TrustSummary, elapsed_secs: u64) -> TrustDisplayState {
    let mut rows: Vec<TrustLevelRow> = summary
        .trust_distribution
        .iter()
        .map(|(label, &count)| trust_level_to_display_row(label, count, summary.total_primals))
        .collect();
    rows.sort_by(|a, b| b.count.cmp(&a.count));

    TrustDisplayState {
        rows,
        total_primals: summary.total_primals,
        average: summary.average_trust.map(average_trust_display),
        family_count: summary.family_count,
        unique_families: summary.unique_families,
        last_update_label: format!("Updated {elapsed_secs} seconds ago"),
    }
}

/// Map trust level number to display label (used by `update_from_primals` and tests).
#[must_use]
pub fn trust_level_number_to_label(n: i32) -> String {
    match n {
        0 => "None (0)".to_string(),
        1 => "Limited (1)".to_string(),
        2 => "Elevated (2)".to_string(),
        3 => "Full (3)".to_string(),
        _ => format!("Unknown ({n})"),
    }
}
