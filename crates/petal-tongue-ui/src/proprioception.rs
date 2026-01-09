//! Proprioception - Complete Sensory-Motor Self-Awareness
//!
//! Like humans knowing their body position without light through feedback,
//! the primal knows its complete input/output state through bidirectional
//! verification loops.
//!
//! This is SAME DAVE for primals - Self-Awareness via Multi-modal Evidence
//! and Deterministic Assessment of Verification Efficacy.

use crate::input_verification::{InputVerificationSystem, InputModality};
use crate::output_verification::{OutputVerificationSystem, OutputModality};
use petal_tongue_core::rendering_awareness::{VisibilityState, InteractivityState};
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// Complete proprioceptive state - full sensory-motor awareness
#[derive(Debug, Clone)]
pub struct ProprioceptiveState {
    /// Can we send output?
    pub motor_functional: bool,
    
    /// Can we receive input?
    pub sensory_functional: bool,
    
    /// Is the bidirectional loop complete?
    pub loop_complete: bool,
    
    /// Overall system health (0.0-1.0)
    pub health: f32,
    
    /// Confidence in our self-knowledge (0.0-1.0)
    pub confidence: f32,
    
    /// Last time loop was confirmed working
    pub last_loop_confirmation: Option<Instant>,
    
    /// Human-readable status
    pub status: String,
}

impl ProprioceptiveState {
    /// Check if we're healthy
    pub fn is_healthy(&self) -> bool {
        self.motor_functional 
            && self.sensory_functional 
            && self.loop_complete
            && self.health > 0.7
    }
    
    /// Check if we're confident in our state
    pub fn is_confident(&self) -> bool {
        self.confidence > 0.7
    }
}

/// SAME DAVE - Self-Awareness via Multi-modal Evidence
/// and Deterministic Assessment of Verification Efficacy
///
/// This is the complete central nervous system for primals!
pub struct ProprioceptionSystem {
    /// Output verification (motor)
    output_system: OutputVerificationSystem,
    
    /// Input verification (sensory)  
    input_system: InputVerificationSystem,
    
    /// Last proprioceptive assessment
    last_state: ProprioceptiveState,
    
    /// Last update time
    last_update: Instant,
}

impl ProprioceptionSystem {
    /// Create a new proprioception system
    pub fn new() -> Self {
        info!("🧠 Initializing SAME DAVE proprioception system...");
        
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
            },
            last_update: Instant::now(),
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
    pub fn output_sent(&mut self, modality: &OutputModality) {
        // This would be called when we send output (render frame, play audio, etc.)
        // For now, we track via user interaction confirming they received it
    }
    
    /// Record input activity
    pub fn input_received(&mut self, modality: &InputModality) {
        self.input_system.record_input(modality);
        
        // KEY INSIGHT: Input from user also confirms they can SEE/HEAR output!
        // This is the bidirectional feedback loop!
        match modality {
            InputModality::Keyboard | InputModality::Pointer => {
                // User interacting via keyboard/mouse confirms they can see visual output
                self.output_system.confirm_via_interaction(&OutputModality::Visual);
            }
            InputModality::Audio => {
                // User speaking confirms they can hear audio output (if we prompted them)
                self.output_system.confirm_via_interaction(&OutputModality::Audio);
            }
            InputModality::Haptic => {
                // User touch confirms they can feel haptic output
                self.output_system.confirm_via_interaction(&OutputModality::Haptic);
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
        
        let outputs_confirmed = output_verifications.iter().filter(|v| v.reaches_user).count();
        let inputs_active = input_verifications.iter().filter(|v| v.input_active).count();
        
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
        let recent_outputs = output_verifications.iter()
            .filter(|v| v.last_confirmed.map(|t| t.elapsed() < recent_threshold).unwrap_or(false))
            .count();
        let recent_inputs = input_verifications.iter()
            .filter(|v| v.last_input.map(|t| t.elapsed() < recent_threshold).unwrap_or(false))
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
        
        // Generate status message
        let status = if health >= 0.9 {
            format!("Proprioception excellent - {} outputs confirmed, {} inputs active", 
                outputs_confirmed, inputs_active)
        } else if health >= 0.7 {
            format!("Proprioception good - {} outputs confirmed, {} inputs active",
                outputs_confirmed, inputs_active)
        } else if health >= 0.5 {
            format!("Proprioception degraded - {}/{} outputs unconfirmed, {}/{} inputs inactive",
                output_verifications.len() - outputs_confirmed, output_verifications.len(),
                input_verifications.len() - inputs_active, input_verifications.len())
        } else {
            format!("Proprioception impaired - limited sensory-motor awareness")
        };
        
        let state = ProprioceptiveState {
            motor_functional,
            sensory_functional,
            loop_complete,
            health,
            confidence,
            last_loop_confirmation,
            status,
        };
        
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
    pub fn get_state(&self) -> &ProprioceptiveState {
        &self.last_state
    }
    
    /// Get detailed status for all outputs
    pub fn get_output_status(&self) -> String {
        self.output_system.get_status_summary()
    }
    
    /// Get detailed status for all inputs
    pub fn get_input_status(&self) -> String {
        self.input_system.get_status_summary()
    }
    
    /// Get comprehensive diagnostic report
    pub fn get_diagnostic_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("🧠 PROPRIOCEPTION DIAGNOSTIC REPORT\n");
        report.push_str("═══════════════════════════════════\n\n");
        
        report.push_str(&format!("Health: {:.0}%\n", self.last_state.health * 100.0));
        report.push_str(&format!("Confidence: {:.0}%\n", self.last_state.confidence * 100.0));
        report.push_str(&format!("Motor: {}\n", if self.last_state.motor_functional { "✅" } else { "❌" }));
        report.push_str(&format!("Sensory: {}\n", if self.last_state.sensory_functional { "✅" } else { "❌" }));
        report.push_str(&format!("Loop: {}\n", if self.last_state.loop_complete { "✅" } else { "❌" }));
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

/// Initialize proprioception with common modalities
pub fn initialize_standard_proprioception() -> ProprioceptionSystem {
    let mut system = ProprioceptionSystem::new();
    
    // Register standard outputs
    system.register_output(OutputModality::Visual);
    system.register_output(OutputModality::Audio);
    system.register_output(OutputModality::Haptic);
    
    // Register standard inputs
    system.register_input(InputModality::Keyboard);
    system.register_input(InputModality::Pointer);
    system.register_input(InputModality::Audio);
    
    info!("✅ Standard proprioception initialized");
    
    system
}

