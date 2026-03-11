// SPDX-License-Identifier: AGPL-3.0-only
//! Rendering awareness - petalTongue's self-knowledge of its display state
//!
//! This is the "central nervous system" - motor (output) + sensory (input) awareness.

mod types;

use crate::frame_introspection::{ContentAwareness, FrameIntrospection, PanelKind};
use crate::sensor::SensorEvent;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub use types::{
    CommandId, InteractivityState, MotorCommand, PanelId, RenderingMetrics, SelfAssessment,
    ValidationHealth, VisibilityState,
};

/// Central awareness of rendering state (motor + sensory + content)
pub struct RenderingAwareness {
    /// Motor state (output)
    motor: MotorState,

    /// Sensory state (input)
    sensory: SensoryState,

    /// Validation pipeline
    validation: ValidationPipeline,

    /// Metrics
    metrics: RenderingMetrics,

    /// Content awareness -- what each frame actually contains
    content: ContentAwareness,
}

impl RenderingAwareness {
    /// Create new rendering awareness system
    #[must_use]
    pub fn new() -> Self {
        Self {
            motor: MotorState::new(),
            sensory: SensoryState::new(),
            validation: ValidationPipeline::new(),
            metrics: RenderingMetrics::default(),
            content: ContentAwareness::new(),
        }
    }

    /// Record a motor command (frame sent)
    pub fn motor_command(&mut self, command: MotorCommand) -> CommandId {
        let id = self.motor.execute(command);
        self.validation.track_frame(id);
        self.metrics.commands_sent += 1;
        id
    }

    /// Record sensory feedback (confirmation received)
    pub fn sensory_feedback(&mut self, event: &SensorEvent) {
        self.sensory.record_event(event);

        match event {
            SensorEvent::FrameAcknowledged { frame_id, .. } => {
                self.validation.confirm_frame(*frame_id);
                self.metrics.frames_confirmed += 1;
            }
            SensorEvent::Heartbeat { latency, .. } => {
                self.sensory.update_substrate_health(*latency);
            }
            _ if event.is_user_interaction() => {
                self.metrics.user_interactions += 1;
            }
            _ => {}
        }
    }

    /// Assess complete state (motor + sensory)
    #[must_use]
    pub fn assess_self(&self) -> SelfAssessment {
        let motor_working = self.motor.is_functional();
        let sensory_working = self.sensory.is_functional();
        let validation_health = self.validation.health();

        SelfAssessment {
            // Motor function
            can_render: motor_working,
            frames_sent: self.metrics.commands_sent,

            // Sensory function
            can_sense: sensory_working,
            frames_confirmed: self.metrics.frames_confirmed,

            // Bidirectional
            is_complete_loop: motor_working && sensory_working && validation_health.healthy,

            // Validation
            confirmation_rate: validation_health.confirmation_rate,

            // User engagement
            user_visibility: self.user_visibility(),
            user_interactivity: self.user_interactivity(),

            // Health
            substrate_responsive: self.sensory.substrate_health.responsive,
        }
    }

    /// Determine if user can see our output
    fn user_visibility(&self) -> VisibilityState {
        let confirmed_rate = self.validation.health().confirmation_rate;

        if confirmed_rate > 90.0 {
            VisibilityState::Confirmed
        } else if confirmed_rate > 50.0 {
            VisibilityState::Probable
        } else if confirmed_rate > 0.0 {
            VisibilityState::Uncertain
        } else {
            VisibilityState::Unknown
        }
    }

    /// Determine if user can interact
    fn user_interactivity(&self) -> InteractivityState {
        match self.sensory.last_user_interaction {
            Some(time) if time.elapsed() < Duration::from_secs(5) => InteractivityState::Active,
            Some(time) if time.elapsed() < Duration::from_secs(30) => InteractivityState::Recent,
            Some(_) => InteractivityState::Idle,
            None => InteractivityState::Unconfirmed,
        }
    }

    /// Get metrics
    #[must_use]
    pub const fn metrics(&self) -> &RenderingMetrics {
        &self.metrics
    }

    /// Get time since last user interaction (for display verification)
    #[must_use]
    pub fn time_since_last_interaction(&self) -> Duration {
        self.sensory
            .last_user_interaction
            .map_or_else(|| Duration::from_secs(9999), |t| t.elapsed())
    }

    /// Record what a frame actually contains (content-level awareness).
    ///
    /// Called by the UI layer at the end of each `update()` cycle so the
    /// primal knows *what* it is showing, not just *that* it rendered.
    pub fn record_frame_content(&mut self, introspection: FrameIntrospection) {
        self.content.record(introspection);
    }

    /// The most recent frame introspection (what we are currently showing).
    #[must_use]
    pub const fn current_content(&self) -> Option<&FrameIntrospection> {
        self.content.current()
    }

    /// Panels currently visible.
    #[must_use]
    pub fn visible_panels(&self) -> Vec<PanelId> {
        self.content.visible_panels()
    }

    /// Whether a panel of the given kind is currently visible.
    #[must_use]
    pub fn is_panel_visible(&self, kind: PanelKind) -> bool {
        self.content
            .current()
            .is_some_and(|f| f.is_panel_visible(kind))
    }

    /// Whether specific data is currently shown in any panel.
    #[must_use]
    pub fn is_showing_data(&self, data_id: &str) -> bool {
        self.content.is_showing_data(data_id)
    }

    /// Access the content awareness subsystem directly.
    #[must_use]
    pub const fn content(&self) -> &ContentAwareness {
        &self.content
    }
}

impl Default for RenderingAwareness {
    fn default() -> Self {
        Self::new()
    }
}

/// Motor state (output capability)
struct MotorState {
    last_command: Option<Instant>,
    commands_executed: u64,
}

impl MotorState {
    const fn new() -> Self {
        Self {
            last_command: None,
            commands_executed: 0,
        }
    }

    fn execute(&mut self, _command: MotorCommand) -> CommandId {
        self.last_command = Some(Instant::now());
        self.commands_executed += 1;
        self.commands_executed
    }

    const fn is_functional(&self) -> bool {
        self.commands_executed > 0
    }
}

/// Sensory state (input capability)
struct SensoryState {
    last_event: Option<Instant>,
    last_user_interaction: Option<Instant>,
    events_received: u64,
    substrate_health: SubstrateHealth,
}

impl SensoryState {
    fn new() -> Self {
        Self {
            last_event: None,
            last_user_interaction: None,
            events_received: 0,
            substrate_health: SubstrateHealth::default(),
        }
    }

    const fn record_event(&mut self, event: &SensorEvent) {
        self.last_event = Some(event.timestamp());
        self.events_received += 1;

        if event.is_user_interaction() {
            self.last_user_interaction = Some(event.timestamp());
        }
    }

    fn update_substrate_health(&mut self, latency: Duration) {
        self.substrate_health.last_heartbeat = Some(Instant::now());
        self.substrate_health.latency = Some(latency);
        self.substrate_health.responsive = latency < Duration::from_millis(100);
    }

    const fn is_functional(&self) -> bool {
        self.events_received > 0
    }
}

/// Substrate health monitoring
#[derive(Debug, Clone, Default)]
struct SubstrateHealth {
    responsive: bool,
    last_heartbeat: Option<Instant>,
    latency: Option<Duration>,
}

/// Validation pipeline - tracks frame confirmation
struct ValidationPipeline {
    sent_frames: VecDeque<(CommandId, Instant)>,
    confirmed_frames: VecDeque<(CommandId, Instant)>,
    max_unconfirmed: usize,
    timeout: Duration,
}

impl ValidationPipeline {
    const fn new() -> Self {
        Self {
            sent_frames: VecDeque::new(),
            confirmed_frames: VecDeque::new(),
            max_unconfirmed: 10,
            timeout: Duration::from_secs(5),
        }
    }

    fn track_frame(&mut self, frame_id: CommandId) {
        self.sent_frames.push_back((frame_id, Instant::now()));

        // Cleanup old frames
        self.cleanup_old_frames();
    }

    fn confirm_frame(&mut self, frame_id: CommandId) {
        self.confirmed_frames.push_back((frame_id, Instant::now()));

        // Remove from sent queue
        self.sent_frames.retain(|(id, _)| *id != frame_id);

        // Cleanup old confirmations
        while self.confirmed_frames.len() > 100 {
            self.confirmed_frames.pop_front();
        }
    }

    fn cleanup_old_frames(&mut self) {
        let now = Instant::now();
        self.sent_frames
            .retain(|(_, timestamp)| now.duration_since(*timestamp) < self.timeout);
    }

    fn health(&self) -> ValidationHealth {
        let unconfirmed = self.sent_frames.len();
        let total_sent = unconfirmed + self.confirmed_frames.len();

        #[expect(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
        let confirmation_rate = if total_sent > 0 {
            (self.confirmed_frames.len() as f64 / total_sent as f64) as f32 * 100.0
        } else {
            0.0
        };

        ValidationHealth {
            healthy: unconfirmed < self.max_unconfirmed,
            confirmation_rate,
            unconfirmed_count: unconfirmed,
        }
    }
}

#[cfg(test)]
mod tests;
