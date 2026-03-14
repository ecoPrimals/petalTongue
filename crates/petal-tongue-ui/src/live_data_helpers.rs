// SPDX-License-Identifier: AGPL-3.0-only
//! Pure display logic helpers for live data indicators.
//!
//! Extracted from live_data.rs for testability and separation of concerns.

/// Display state for the live badge (LIVE / STALE / WAITING).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BadgeDisplayState {
    /// RGB color for the badge
    pub color_rgb: (u8, u8, u8),
    /// Static label to display (e.g. "LIVE", "STALE", "WAITING")
    pub label: &'static str,
    /// Tooltip text (e.g. "Just now", "Data is 5.2s old")
    pub tooltip: String,
}

/// Display state for connection status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionDisplayState {
    /// Symbol to display (e.g. "●", "○")
    pub symbol: &'static str,
    /// Status text (e.g. "Connected", "Disconnected")
    pub status_text: &'static str,
    /// RGB color for the status
    pub color_rgb: (u8, u8, u8),
}

/// Compute badge display state from age, staleness, and liveness.
#[must_use]
pub fn badge_display_state(age_secs: f64, is_stale: bool, is_live: bool) -> BadgeDisplayState {
    if !is_live {
        BadgeDisplayState {
            color_rgb: (128, 128, 128),
            label: "WAITING",
            tooltip: "Waiting for data".to_string(),
        }
    } else if is_stale {
        BadgeDisplayState {
            color_rgb: (200, 140, 0),
            label: "STALE",
            tooltip: format!("Data is {} (stale)", format_age_for_display(age_secs)),
        }
    } else if age_secs < 1.0 {
        BadgeDisplayState {
            color_rgb: (0, 200, 100),
            label: "● LIVE",
            tooltip: "Just now".to_string(),
        }
    } else {
        BadgeDisplayState {
            color_rgb: (0, 200, 100),
            label: "LIVE",
            tooltip: format_age_for_display(age_secs),
        }
    }
}

/// Compute connection status display state.
#[must_use]
pub const fn connection_status_display(
    connected: bool,
    disconnected_secs: Option<f64>,
) -> ConnectionDisplayState {
    if connected {
        ConnectionDisplayState {
            symbol: "●",
            status_text: "Connected",
            color_rgb: (0, 200, 100),
        }
    } else if let Some(_secs) = disconnected_secs {
        ConnectionDisplayState {
            symbol: "○",
            status_text: "Disconnected",
            color_rgb: (200, 50, 50),
        }
    } else {
        ConnectionDisplayState {
            symbol: "○",
            status_text: "Disconnected",
            color_rgb: (200, 50, 50),
        }
    }
}

/// Format age in seconds as human-readable string.
#[must_use]
pub fn format_age_for_display(age_secs: f64) -> String {
    if age_secs < 1.0 {
        "Just now".to_string()
    } else if age_secs < 60.0 {
        format!("{age_secs:.1}s ago")
    } else if age_secs < 3600.0 {
        format!("{:.1}m ago", age_secs / 60.0)
    } else {
        format!("{:.1}h ago", age_secs / 3600.0)
    }
}

/// Format a metric value with its unit for display.
#[must_use]
pub fn format_metric_value(value: f64, unit: &str) -> String {
    if unit.is_empty() {
        format!("{value}")
    } else {
        format!("{value}{unit}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn badge_display_state_waiting() {
        let s = badge_display_state(0.0, false, false);
        assert_eq!(s.color_rgb, (128, 128, 128));
        assert_eq!(s.label, "WAITING");
        assert_eq!(s.tooltip, "Waiting for data");
    }

    #[test]
    fn badge_display_state_stale() {
        let s = badge_display_state(30.0, true, true);
        assert_eq!(s.color_rgb, (200, 140, 0));
        assert_eq!(s.label, "STALE");
        assert!(s.tooltip.contains("30.0s ago"));
        assert!(s.tooltip.contains("stale"));
    }

    #[test]
    fn badge_display_state_live_fresh() {
        let s = badge_display_state(0.5, false, true);
        assert_eq!(s.color_rgb, (0, 200, 100));
        assert_eq!(s.label, "● LIVE");
        assert_eq!(s.tooltip, "Just now");
    }

    #[test]
    fn badge_display_state_live_older() {
        let s = badge_display_state(5.0, false, true);
        assert_eq!(s.color_rgb, (0, 200, 100));
        assert_eq!(s.label, "LIVE");
        assert_eq!(s.tooltip, "5.0s ago");
    }

    #[test]
    fn badge_display_state_stale_overrides_fresh_age() {
        let s = badge_display_state(0.5, true, true);
        assert_eq!(s.label, "STALE");
    }

    #[test]
    fn connection_status_display_connected() {
        let s = connection_status_display(true, None);
        assert_eq!(s.symbol, "●");
        assert_eq!(s.status_text, "Connected");
        assert_eq!(s.color_rgb, (0, 200, 100));
    }

    #[test]
    fn connection_status_display_disconnected_no_secs() {
        let s = connection_status_display(false, None);
        assert_eq!(s.symbol, "○");
        assert_eq!(s.status_text, "Disconnected");
        assert_eq!(s.color_rgb, (200, 50, 50));
    }

    #[test]
    fn connection_status_display_disconnected_with_secs() {
        let s = connection_status_display(false, Some(45.5));
        assert_eq!(s.symbol, "○");
        assert_eq!(s.status_text, "Disconnected");
        assert_eq!(s.color_rgb, (200, 50, 50));
    }

    #[test]
    fn format_age_just_now() {
        assert_eq!(format_age_for_display(0.0), "Just now");
        assert_eq!(format_age_for_display(0.1), "Just now");
        assert_eq!(format_age_for_display(0.5), "Just now");
        assert_eq!(format_age_for_display(0.99), "Just now");
    }

    #[test]
    fn format_age_seconds() {
        assert_eq!(format_age_for_display(1.0), "1.0s ago");
        assert_eq!(format_age_for_display(30.0), "30.0s ago");
        assert_eq!(format_age_for_display(59.0), "59.0s ago");
        assert_eq!(format_age_for_display(59.9), "59.9s ago");
    }

    #[test]
    fn format_age_minutes() {
        assert_eq!(format_age_for_display(60.0), "1.0m ago");
        assert_eq!(format_age_for_display(90.0), "1.5m ago");
        assert_eq!(format_age_for_display(3599.0), "60.0m ago");
    }

    #[test]
    fn format_age_hours() {
        assert_eq!(format_age_for_display(3600.0), "1.0h ago");
        assert_eq!(format_age_for_display(7200.0), "2.0h ago");
        assert_eq!(format_age_for_display(86400.0), "24.0h ago");
    }

    #[test]
    fn format_age_boundary_seconds() {
        assert_eq!(format_age_for_display(0.1), "Just now");
        assert_eq!(format_age_for_display(1.0), "1.0s ago");
        assert_eq!(format_age_for_display(59.0), "59.0s ago");
    }

    #[test]
    fn format_age_boundary_minutes() {
        assert_eq!(format_age_for_display(60.0), "1.0m ago");
        assert_eq!(format_age_for_display(3599.0), "60.0m ago");
    }

    #[test]
    fn format_metric_value_with_unit() {
        assert_eq!(format_metric_value(45.2, "%"), "45.2%");
        assert_eq!(format_metric_value(72.5, "°F"), "72.5°F");
        assert_eq!(format_metric_value(99.5, "ms"), "99.5ms");
        assert_eq!(format_metric_value(42.0, "°C"), "42°C");
    }

    #[test]
    fn format_metric_value_empty_unit() {
        assert_eq!(format_metric_value(42.0, ""), "42");
        assert_eq!(format_metric_value(0.0, ""), "0");
    }

    #[test]
    fn format_metric_value_integer_display() {
        assert_eq!(format_metric_value(100.0, "ms"), "100ms");
    }
}
