// SPDX-License-Identifier: AGPL-3.0-only
//! Rendering awareness - petalTongue's self-knowledge of its display state
//!
//! This is the "central nervous system" - motor (output) + sensory (input) awareness.

use crate::sensor::SensorEvent;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Central awareness of rendering state (motor + sensory)
pub struct RenderingAwareness {
    /// Motor state (output)
    motor: MotorState,

    /// Sensory state (input)
    sensory: SensoryState,

    /// Validation pipeline
    validation: ValidationPipeline,

    /// Metrics
    metrics: RenderingMetrics,
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
    pub fn metrics(&self) -> &RenderingMetrics {
        &self.metrics
    }

    /// Get time since last user interaction (for display verification)
    #[must_use]
    pub fn time_since_last_interaction(&self) -> Duration {
        self.sensory
            .last_user_interaction
            .map_or_else(|| Duration::from_secs(9999), |t| t.elapsed())
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
    fn new() -> Self {
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

    fn is_functional(&self) -> bool {
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

    fn record_event(&mut self, event: &SensorEvent) {
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

    fn is_functional(&self) -> bool {
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
    fn new() -> Self {
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

        let confirmation_rate = if total_sent > 0 {
            (self.confirmed_frames.len() as f32 / total_sent as f32) * 100.0
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

/// Validation health metrics
#[derive(Debug, Clone)]
pub struct ValidationHealth {
    /// Is validation pipeline healthy?
    pub healthy: bool,
    /// Percentage of frames confirmed
    pub confirmation_rate: f32,
    /// Number of unconfirmed frames
    pub unconfirmed_count: usize,
}

/// Motor command types
#[derive(Debug, Clone)]
pub enum MotorCommand {
    /// Render a specific frame
    RenderFrame {
        /// Frame identifier
        frame_id: u64,
    },
    /// Update display without new frame
    UpdateDisplay,
    /// Clear the display
    ClearDisplay,
}

/// Command ID (unique identifier for tracking)
pub type CommandId = u64;

/// Complete self-assessment of the central nervous system
#[derive(Debug, Clone)]
pub struct SelfAssessment {
    // Motor
    /// Whether the system can render output (motor capability)
    pub can_render: bool,
    /// Total number of frames sent (motor activity)
    pub frames_sent: u64,

    // Sensory
    /// Whether the system can receive input (sensory capability)
    pub can_sense: bool,
    /// Number of frames with confirmed sensory feedback
    pub frames_confirmed: u64,

    // Bidirectional
    /// Whether the bidirectional loop is complete
    pub is_complete_loop: bool,
    /// Rate of sensory confirmation (`frames_confirmed` / `frames_sent`)
    pub confirmation_rate: f32,

    // User state
    /// Current visibility state from user perspective
    pub user_visibility: VisibilityState,
    /// Current interactivity state from user perspective
    pub user_interactivity: InteractivityState,

    // Health
    /// Whether the display substrate is responsive
    pub substrate_responsive: bool,
}

impl SelfAssessment {
    /// Check if everything is working
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        self.can_render && self.can_sense && self.is_complete_loop && self.substrate_responsive
    }

    /// Get overall health percentage
    #[must_use]
    pub fn health_percentage(&self) -> f32 {
        let mut score = 0.0;

        if self.can_render {
            score += 20.0;
        }
        if self.can_sense {
            score += 20.0;
        }
        if self.is_complete_loop {
            score += 30.0;
        }
        if self.substrate_responsive {
            score += 10.0;
        }

        score += (self.confirmation_rate * 0.2).min(20.0);

        score
    }
}

/// User visibility state - confidence that user can see output
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisibilityState {
    /// High confidence user can see output (>90% confirmation)
    Confirmed,
    /// Likely user can see output (>50% confirmation)
    Probable,
    /// Uncertain if user can see output (>0% confirmation)
    Uncertain,
    /// No confirmation of visibility
    Unknown,
}

/// User interactivity state - recency of user interaction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractivityState {
    /// User actively interacting (interacted <5s ago)
    Active,
    /// Recent user interaction (interacted <30s ago)
    Recent,
    /// User idle (interacted >30s ago)
    Idle,
    /// No user interaction confirmed
    Unconfirmed,
}

/// Rendering metrics - quantitative feedback on rendering effectiveness
#[derive(Debug, Clone, Default)]
pub struct RenderingMetrics {
    /// Total rendering commands sent to output
    pub commands_sent: u64,
    /// Number of frames confirmed as visible
    pub frames_confirmed: u64,
    /// Number of user interactions detected
    pub user_interactions: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sensor::MouseButton;

    #[test]
    fn test_motor_command() {
        let mut awareness = RenderingAwareness::new();

        let cmd_id = awareness.motor_command(MotorCommand::RenderFrame { frame_id: 1 });
        assert_eq!(cmd_id, 1);
        assert_eq!(awareness.metrics.commands_sent, 1);
    }

    #[test]
    fn test_sensory_feedback() {
        let mut awareness = RenderingAwareness::new();

        // Send a frame
        let frame_id = awareness.motor_command(MotorCommand::RenderFrame { frame_id: 1 });

        // Confirm it
        let event = SensorEvent::FrameAcknowledged {
            frame_id,
            timestamp: Instant::now(),
        };
        awareness.sensory_feedback(&event);

        assert_eq!(awareness.metrics.frames_confirmed, 1);
    }

    #[test]
    fn test_user_interaction_tracking() {
        let mut awareness = RenderingAwareness::new();

        let click = SensorEvent::Click {
            x: 100.0,
            y: 200.0,
            button: MouseButton::Left,
            timestamp: Instant::now(),
        };

        awareness.sensory_feedback(&click);

        assert_eq!(awareness.metrics.user_interactions, 1);
        let assessment = awareness.assess_self();
        assert_eq!(assessment.user_interactivity, InteractivityState::Active);
    }

    #[test]
    fn test_bidirectional_loop() {
        let mut awareness = RenderingAwareness::new();

        // Motor: send frame
        let frame_id = awareness.motor_command(MotorCommand::RenderFrame { frame_id: 1 });

        // Sensory: confirm frame
        awareness.sensory_feedback(&SensorEvent::FrameAcknowledged {
            frame_id,
            timestamp: Instant::now(),
        });

        // Sensory: heartbeat
        awareness.sensory_feedback(&SensorEvent::Heartbeat {
            latency: Duration::from_millis(10),
            timestamp: Instant::now(),
        });

        let assessment = awareness.assess_self();
        assert!(assessment.can_render);
        assert!(assessment.can_sense);
        assert!(assessment.is_complete_loop);
    }

    #[test]
    fn test_health_percentage() {
        let mut awareness = RenderingAwareness::new();

        // Send and confirm multiple frames
        for i in 0..10 {
            let frame_id = awareness.motor_command(MotorCommand::RenderFrame { frame_id: i });
            awareness.sensory_feedback(&SensorEvent::FrameAcknowledged {
                frame_id,
                timestamp: Instant::now(),
            });
        }

        awareness.sensory_feedback(&SensorEvent::Heartbeat {
            latency: Duration::from_millis(5),
            timestamp: Instant::now(),
        });

        let assessment = awareness.assess_self();
        let health = assessment.health_percentage();

        // Should be near 100% (all checks pass)
        assert!(health > 95.0, "Health was {health}");
    }
}
