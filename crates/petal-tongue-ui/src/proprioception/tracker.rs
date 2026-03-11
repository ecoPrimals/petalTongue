// SPDX-License-Identifier: AGPL-3.0-only
//! SAME DAVE Proprioception System - main tracking logic

use crate::input_verification::{InputModality, InputVerificationSystem};
use crate::output_verification::{OutputModality, OutputVerificationSystem};
use crate::proprioception::types::ProprioceptiveState;
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// A diagnostic event for debugging and post-mortem analysis
#[derive(Debug, Clone)]
struct DiagnosticEvent {
    timestamp: Instant,
    event_type: String,
    message: String,
}

/// SAME DAVE Proprioception System
///
/// Implements the neuroanatomy model (SAME DAVE mnemonic):
/// - Sensory Afferent: Input pathways (keyboard, mouse, audio in)
/// - Motor Efferent: Output pathways (display, audio out, haptic)
/// - Dorsal Afferent: Sensory signals coming in
/// - Ventral Efferent: Motor commands going out
///
/// Both motor (efferent) and sensory (afferent) pathways are required
/// for proprioception function - complete self-awareness!
pub struct ProprioceptionSystem {
    /// Output verification (Motor/Efferent pathways)
    output_system: OutputVerificationSystem,

    /// Input verification (Sensory/Afferent pathways)
    input_system: InputVerificationSystem,

    /// Last proprioceptive assessment
    last_state: ProprioceptiveState,

    /// Last update time
    last_update: Instant,

    // === v1.2.0: Frame Tracking & Hang Detection ===
    /// Total frames rendered
    frame_count: u64,

    /// Last frame render time
    last_frame_time: Instant,

    /// Frame times for FPS calculation (ring buffer)
    frame_times: Vec<Instant>,

    /// Maximum frame time samples to keep
    max_frame_samples: usize,

    /// Hang threshold (seconds without a frame = hanging)
    hang_threshold: Duration,

    /// Diagnostic event log (ring buffer)
    diagnostic_events: Vec<DiagnosticEvent>,

    /// Maximum diagnostic events to keep
    max_diagnostic_events: usize,
}

impl ProprioceptionSystem {
    /// Create a new proprioception system
    ///
    /// Initializes the neuroanatomy model (SAME DAVE):
    /// - Sensory Afferent (input pathways)
    /// - Motor Efferent (output pathways)
    pub fn new() -> Self {
        info!("🧠 Initializing SAME DAVE proprioception system (neuroanatomy model)...");

        let now = Instant::now();

        Self {
            output_system: OutputVerificationSystem::new(),
            input_system: InputVerificationSystem::new(),
            last_state: ProprioceptiveState {
                motor_functional: false,
                sensory_functional: false,
                loop_complete: false,
                health: 0.0,
                confidence: 0.0,
                last_loop_confirmation: None,
                status: "System initializing...".to_string(),
                frame_rate: 0.0,
                time_since_last_frame: Duration::from_secs(0),
                is_hanging: false,
                hang_reason: None,
                total_frames: 0,
            },
            last_update: now,
            frame_count: 0,
            last_frame_time: now,
            frame_times: Vec::new(),
            max_frame_samples: 60, // Track last 60 frames for FPS
            hang_threshold: Duration::from_secs(5), // 5 seconds without frame = hang
            diagnostic_events: Vec::new(),
            max_diagnostic_events: 100, // Keep last 100 events
        }
    }

    /// Register an output modality
    pub fn register_output(&mut self, modality: OutputModality) {
        self.output_system.register_output(modality);
    }

    /// Register an input modality
    pub fn register_input(&mut self, modality: InputModality) {
        self.input_system.register_input(modality);
    }

    /// Record output activity
    pub const fn output_sent(&mut self, _modality: &OutputModality) {
        // This would be called when we send output (render frame, play audio, etc.)
        // For now, we track via user interaction confirming they received it
    }

    // === v1.2.0: Frame Tracking & Hang Detection ===

    /// Record that a frame was rendered (critical for hang detection)
    pub fn record_frame(&mut self) {
        let now = Instant::now();

        self.frame_count += 1;
        self.last_frame_time = now;

        // Add to frame times for FPS calculation
        self.frame_times.push(now);

        // Keep only the last N frames
        if self.frame_times.len() > self.max_frame_samples {
            self.frame_times.remove(0);
        }

        // If we just recovered from a hang, log it
        if self.last_state.is_hanging {
            let hang_duration = self.last_state.time_since_last_frame;
            warn!(
                "🔄 Recovered from hang! Duration: {:.1}s",
                hang_duration.as_secs_f32()
            );
            self.log_diagnostic_event(
                "hang_recovery",
                &format!("Recovered after {:.1}s", hang_duration.as_secs_f32()),
            );
        }
    }

    /// Calculate current frame rate (FPS)
    fn calculate_fps(&self) -> f32 {
        if self.frame_times.len() < 2 {
            return 0.0;
        }

        let (first, last) = match (self.frame_times.first(), self.frame_times.last()) {
            (Some(f), Some(l)) => (f, l),
            _ => return 0.0,
        };
        let duration = last.duration_since(*first);

        if duration.as_secs_f32() > 0.0 {
            (self.frame_times.len() - 1) as f32 / duration.as_secs_f32()
        } else {
            0.0
        }
    }

    /// Check if the rendering loop is hanging
    fn check_hang(&self) -> (bool, Option<String>) {
        let time_since_frame = self.last_frame_time.elapsed();

        if time_since_frame > self.hang_threshold {
            let reason = format!(
                "No frames rendered for {:.1}s (threshold: {:.1}s)",
                time_since_frame.as_secs_f32(),
                self.hang_threshold.as_secs_f32()
            );
            (true, Some(reason))
        } else {
            (false, None)
        }
    }

    /// Log a diagnostic event
    fn log_diagnostic_event(&mut self, event_type: &str, message: &str) {
        let event = DiagnosticEvent {
            timestamp: Instant::now(),
            event_type: event_type.to_string(),
            message: message.to_string(),
        };

        self.diagnostic_events.push(event);

        // Keep only the last N events
        if self.diagnostic_events.len() > self.max_diagnostic_events {
            self.diagnostic_events.remove(0);
        }
    }

    /// Get recent diagnostic events (for debugging)
    #[must_use]
    pub fn get_diagnostic_events(&self, count: usize) -> Vec<(Duration, String, String)> {
        let now = Instant::now();
        self.diagnostic_events
            .iter()
            .rev()
            .take(count)
            .map(|e| {
                let age = now.duration_since(e.timestamp);
                (age, e.event_type.clone(), e.message.clone())
            })
            .collect()
    }

    /// Record input activity
    pub fn input_received(&mut self, modality: &InputModality) {
        self.input_system.record_input(modality);

        // KEY INSIGHT: Input from user also confirms they can SEE/HEAR output!
        // This is the bidirectional feedback loop!
        match modality {
            InputModality::Keyboard | InputModality::Pointer => {
                // User interacting via keyboard/mouse confirms they can see visual output
                self.output_system
                    .confirm_via_interaction(&OutputModality::Visual);
            }
            InputModality::Audio => {
                // User speaking confirms they can hear audio output (if we prompted them)
                self.output_system
                    .confirm_via_interaction(&OutputModality::Audio);
            }
            InputModality::Haptic => {
                // User touch confirms they can feel haptic output
                self.output_system
                    .confirm_via_interaction(&OutputModality::Haptic);
            }
            _ => {}
        }
    }

    /// Assess complete proprioceptive state
    pub fn assess(&mut self) -> ProprioceptiveState {
        let now = Instant::now();

        // Update subsystems
        self.output_system.update();
        self.input_system.update();

        // Check motor function
        let motor_functional = !self.output_system.has_unconfirmed_outputs();

        // Check sensory function
        let sensory_functional = !self.input_system.has_inactive_inputs();

        // Check bidirectional loop
        let output_verifications = self.output_system.get_all_verifications();
        let input_verifications = self.input_system.get_all_verifications();

        let outputs_confirmed = output_verifications
            .iter()
            .filter(|v| v.reaches_user)
            .count();
        let inputs_active = input_verifications
            .iter()
            .filter(|v| v.input_active)
            .count();

        let loop_complete = outputs_confirmed > 0 && inputs_active > 0;

        // Calculate health (0.0-1.0)
        let total_modalities = output_verifications.len() + input_verifications.len();
        let confirmed_modalities = outputs_confirmed + inputs_active;
        let health = if total_modalities > 0 {
            confirmed_modalities as f32 / total_modalities as f32
        } else {
            0.0
        };

        // Calculate confidence based on recency of confirmations
        let recent_threshold = Duration::from_secs(30);
        let recent_outputs = output_verifications
            .iter()
            .filter(|v| {
                v.last_confirmed
                    .is_some_and(|t| t.elapsed() < recent_threshold)
            })
            .count();
        let recent_inputs = input_verifications
            .iter()
            .filter(|v| v.last_input.is_some_and(|t| t.elapsed() < recent_threshold))
            .count();

        let confidence = if total_modalities > 0 {
            (recent_outputs + recent_inputs) as f32 / total_modalities as f32
        } else {
            0.0
        };

        // Last loop confirmation
        let last_loop_confirmation = if loop_complete {
            self.input_system.most_recent_interaction()
        } else {
            None
        };

        // === v1.2.0: Frame Tracking & Hang Detection ===
        let frame_rate = self.calculate_fps();
        let time_since_last_frame = self.last_frame_time.elapsed();
        let (is_hanging, hang_reason) = self.check_hang();

        // Generate status message
        let status = if is_hanging {
            format!(
                "HANGING: {} - {}",
                hang_reason.as_deref().unwrap_or("unknown"),
                if health >= 0.7 {
                    "otherwise healthy"
                } else {
                    "degraded"
                }
            )
        } else if health >= 0.9 {
            format!(
                "Proprioception excellent - {outputs_confirmed} outputs confirmed, {inputs_active} inputs active, {frame_rate:.1} FPS"
            )
        } else if health >= 0.7 {
            format!(
                "Proprioception good - {outputs_confirmed} outputs confirmed, {inputs_active} inputs active, {frame_rate:.1} FPS"
            )
        } else if health >= 0.5 {
            format!(
                "Proprioception degraded - {}/{} outputs unconfirmed, {}/{} inputs inactive, {:.1} FPS",
                output_verifications.len() - outputs_confirmed,
                output_verifications.len(),
                input_verifications.len() - inputs_active,
                input_verifications.len(),
                frame_rate
            )
        } else {
            format!(
                "Proprioception impaired - limited sensory-motor awareness, {frame_rate:.1} FPS"
            )
        };

        let state = ProprioceptiveState {
            motor_functional,
            sensory_functional,
            loop_complete,
            health,
            confidence,
            last_loop_confirmation,
            status,
            frame_rate,
            time_since_last_frame,
            is_hanging,
            hang_reason: hang_reason.clone(),
            total_frames: self.frame_count,
        };

        // === v1.2.0: Log hang detection (after borrowing is done) ===
        if is_hanging && !self.last_state.is_hanging {
            warn!(
                "⚠️  HANG DETECTED: {}",
                hang_reason.as_deref().unwrap_or("unknown")
            );
            self.log_diagnostic_event("hang_detected", hang_reason.as_deref().unwrap_or("unknown"));
        }

        // Log significant changes
        if state.health < 0.5 && self.last_state.health >= 0.5 {
            warn!("⚠️  Proprioceptive health degraded below 50%");
        }
        if !state.loop_complete && self.last_state.loop_complete {
            warn!("⚠️  Bidirectional loop lost!");
        }
        if state.loop_complete && !self.last_state.loop_complete {
            info!("✅ Bidirectional loop established!");
        }

        self.last_state = state.clone();
        self.last_update = now;

        state
    }

    /// Get current proprioceptive state (cached, fast)
    #[must_use]
    pub const fn get_state(&self) -> &ProprioceptiveState {
        &self.last_state
    }

    /// Get detailed status for all outputs
    #[must_use]
    pub fn get_output_status(&self) -> String {
        self.output_system.get_status_summary()
    }

    /// Get detailed status for all inputs
    #[must_use]
    pub fn get_input_status(&self) -> String {
        self.input_system.get_status_summary()
    }

    /// Get comprehensive diagnostic report
    #[must_use]
    pub fn get_diagnostic_report(&self) -> String {
        let mut report = String::new();

        report.push_str("🧠 PROPRIOCEPTION DIAGNOSTIC REPORT\n");
        report.push_str("═══════════════════════════════════\n\n");

        report.push_str(&format!("Health: {:.0}%\n", self.last_state.health * 100.0));
        report.push_str(&format!(
            "Confidence: {:.0}%\n",
            self.last_state.confidence * 100.0
        ));
        report.push_str(&format!(
            "Motor: {}\n",
            if self.last_state.motor_functional {
                "✅"
            } else {
                "❌"
            }
        ));
        report.push_str(&format!(
            "Sensory: {}\n",
            if self.last_state.sensory_functional {
                "✅"
            } else {
                "❌"
            }
        ));
        report.push_str(&format!(
            "Loop: {}\n",
            if self.last_state.loop_complete {
                "✅"
            } else {
                "❌"
            }
        ));
        report.push_str(&format!("\nStatus: {}\n", self.last_state.status));

        report.push_str(&format!("\n{}\n", self.output_system.get_status_summary()));
        report.push_str(&format!("{}\n", self.input_system.get_status_summary()));

        report
    }
}

impl Default for ProprioceptionSystem {
    fn default() -> Self {
        Self::new()
    }
}
