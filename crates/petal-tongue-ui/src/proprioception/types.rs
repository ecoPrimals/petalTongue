// SPDX-License-Identifier: AGPL-3.0-or-later
//! Proprioception data types and config structs

use std::time::{Duration, Instant};

/// Complete proprioceptive state - full sensory-motor awareness
///
/// Based on neuroanatomy model (SAME DAVE):
/// - Motor = Efferent pathways (output)
/// - Sensory = Afferent pathways (input)
/// - Both required for proprioception function
#[derive(Debug, Clone)]
pub struct ProprioceptiveState {
    /// Motor function: Can we send output? (Efferent pathways)
    pub motor_functional: bool,

    /// Sensory function: Can we receive input? (Afferent pathways)
    pub sensory_functional: bool,

    /// Is the bidirectional loop complete? (Motor + Sensory working)
    pub loop_complete: bool,

    /// Overall system health (0.0-1.0)
    pub health: f32,

    /// Confidence in our self-knowledge (0.0-1.0)
    pub confidence: f32,

    /// Last time loop was confirmed working
    pub last_loop_confirmation: Option<Instant>,

    /// Human-readable status
    pub status: String,

    // === v1.2.0: Performance & Hang Detection ===
    /// Current frame rate (frames per second)
    pub frame_rate: f32,

    /// Time since last frame rendered (potential hang indicator)
    pub time_since_last_frame: Duration,

    /// Is the rendering loop hanging?
    pub is_hanging: bool,

    /// Hang reason (if applicable)
    pub hang_reason: Option<String>,

    /// Total frames rendered since start
    pub total_frames: u64,
}

impl ProprioceptiveState {
    /// Check if we're healthy
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        self.motor_functional && self.sensory_functional && self.loop_complete && self.health > 0.7
    }

    /// Check if we're confident in our state
    #[must_use]
    pub fn is_confident(&self) -> bool {
        self.confidence > 0.7
    }
}
