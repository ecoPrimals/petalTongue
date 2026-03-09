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
    pub fn color(&self) -> Color32 {
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
