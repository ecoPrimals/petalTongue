// SPDX-License-Identifier: AGPL-3.0-only
//! Timeline event types for awakening coordination.

use crate::awakening::AwakeningStage;

/// Timeline Event
///
/// Represents a synchronized event across all modalities.
#[derive(Debug, Clone)]
pub struct TimelineEvent {
    /// Time offset from start (seconds)
    pub time: f32,

    /// Stage this event belongs to
    pub stage: AwakeningStage,

    /// Event type
    pub event_type: TimelineEventType,
}

/// Timeline Event Type
#[derive(Debug, Clone)]
pub enum TimelineEventType {
    /// Stage transition
    StageTransition {
        /// New stage
        stage: AwakeningStage,
    },

    /// Visual frame update
    VisualFrame {
        /// Frame index
        frame: usize,
    },

    /// Audio layer start
    AudioStart {
        /// Layer name
        layer: String,
    },

    /// Audio layer stop
    AudioStop {
        /// Layer name
        layer: String,
    },

    /// Text message
    TextMessage {
        /// Message content
        message: String,
    },

    /// Discovery event (primal found)
    Discovery {
        /// Primal name
        primal: String,
        /// Index for audio chime
        index: u32,
    },
}
