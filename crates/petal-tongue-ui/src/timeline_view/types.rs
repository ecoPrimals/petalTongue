// SPDX-License-Identifier: AGPL-3.0-only
//! Timeline View - Types and event status
//!
//! Core data types for the timeline event sequence visualization.

use chrono::{DateTime, Utc};
use egui::Color32;

/// Event in the timeline
#[derive(Clone, Debug)]
pub struct TimelineEvent {
    /// Unique event ID
    pub id: String,
    /// Source primal ID
    pub from: String,
    /// Target primal ID
    pub to: String,
    /// Event type (capability name, message type, etc.)
    pub event_type: String,
    /// Timestamp when event occurred
    pub timestamp: DateTime<Utc>,
    /// Duration of the event (if applicable)
    pub duration_ms: Option<f64>,
    /// Status (success, failure, etc.)
    pub status: EventStatus,
    /// Optional payload summary
    pub payload_summary: Option<String>,
}

/// User interaction intent produced by the timeline view render method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimelineIntent {
    ZoomIn,
    ZoomOut,
    ToggleDetails,
    SelectEvent(String),
    DeselectEvent,
    Clear,
    ExportCsv,
}

/// Pre-computed display data for a selected event detail panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventDetailDisplay {
    pub status_icon: &'static str,
    pub status_label: String,
    pub from: String,
    pub to: String,
    pub event_type: String,
    pub time_str: String,
    pub duration_str: Option<String>,
    pub payload: Option<String>,
}

/// Status of a timeline event
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventStatus {
    /// Event completed successfully
    Success,
    /// Event failed
    Failure,
    /// Event is still in progress
    InProgress,
    /// Event timed out
    Timeout,
}

impl EventStatus {
    /// Get color for this status
    #[must_use]
    pub const fn color(&self) -> Color32 {
        match self {
            Self::Success => Color32::from_rgb(100, 255, 100),
            Self::Failure => Color32::from_rgb(255, 100, 100),
            Self::InProgress => Color32::from_rgb(255, 200, 100),
            Self::Timeout => Color32::from_rgb(200, 100, 255),
        }
    }

    /// Get icon for this status
    #[must_use]
    pub const fn icon(&self) -> &'static str {
        match self {
            Self::Success => "✅",
            Self::Failure => "❌",
            Self::InProgress => "⏳",
            Self::Timeout => "⏱️",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_status_color_all_variants() {
        assert_eq!(
            EventStatus::Success.color(),
            Color32::from_rgb(100, 255, 100)
        );
        assert_eq!(
            EventStatus::Failure.color(),
            Color32::from_rgb(255, 100, 100)
        );
        assert_eq!(
            EventStatus::InProgress.color(),
            Color32::from_rgb(255, 200, 100)
        );
        assert_eq!(
            EventStatus::Timeout.color(),
            Color32::from_rgb(200, 100, 255)
        );
    }

    #[test]
    fn event_status_icon_all_variants() {
        assert_eq!(EventStatus::Success.icon(), "✅");
        assert_eq!(EventStatus::Failure.icon(), "❌");
        assert_eq!(EventStatus::InProgress.icon(), "⏳");
        assert_eq!(EventStatus::Timeout.icon(), "⏱️");
    }
}
