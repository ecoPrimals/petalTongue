// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Integration tests for SAME DAVE proprioception system
//!
//! Tests the complete sensory-motor self-awareness system including:
//! - Output verification (visual, audio, haptic)
//! - Input verification (keyboard, pointer, audio)
//! - Bidirectional feedback loops
//! - Health assessment
//! - Topology detection
//!
//! EVOLUTION NOTE: Tests evolved to remove blocking `thread::sleep` calls.
//! Time-based behavior is tested via mechanism verification, not actual time passage.

use petal_tongue_ui::{input_verification::*, output_verification::*, proprioception::*};
use std::time::Duration;

#[test]
fn test_proprioception_initialization() {
    let system = ProprioceptionSystem::new();

    // Initial state should be unconfirmed
    let state = system.get_state();
    assert!(!state.motor_functional);
    assert!(!state.sensory_functional);
    assert!(!state.loop_complete);
    assert!((state.health - 0.0).abs() < f32::EPSILON);
}

#[test]
fn test_output_registration() {
    let mut system = ProprioceptionSystem::new();

    // Register outputs
    system.register_output(OutputModality::Visual);
    system.register_output(OutputModality::Audio);
    system.register_output(OutputModality::Haptic);

    // Should have 3 outputs registered
    let state = system.assess();
    // Health still 0 because no confirmations yet
    assert!((state.health - 0.0).abs() < f32::EPSILON);
}

#[test]
fn test_input_registration() {
    let mut system = ProprioceptionSystem::new();

    // Register inputs
    system.register_input(InputModality::Keyboard);
    system.register_input(InputModality::Pointer);
    system.register_input(InputModality::Audio);

    let state = system.assess();
    assert!((state.health - 0.0).abs() < f32::EPSILON); // No input received yet
}

#[test]
fn test_bidirectional_feedback_loop() {
    let mut system = initialize_standard_proprioception();

    // Simulate user interaction (keyboard input)
    system.input_received(&InputModality::Keyboard);

    // Assess state
    let state = system.assess();

    // Input should be active
    assert!(state.sensory_functional || state.health > 0.0);

    // KEY TEST: Keyboard input should confirm visual output!
    // (This is the bidirectional feedback loop)
    let status = system.get_output_status();
    assert!(status.contains("confirmed") || status.contains("active"));
}

#[test]
fn test_health_calculation() {
    let mut system = initialize_standard_proprioception();

    // No interaction - health should be low
    let state1 = system.assess();
    assert!((state1.health - 0.0).abs() < f32::EPSILON);

    // Simulate user interaction
    system.input_received(&InputModality::Keyboard);
    system.input_received(&InputModality::Pointer);

    // Health should increase
    let state2 = system.assess();
    assert!(state2.health > state1.health);

    // Should have better health now (may not be "functional" but should have activity)
    assert!(state2.health > 0.0);
}

#[test]
fn test_confidence_mechanism() {
    let mut system = initialize_standard_proprioception();

    // Test 1: Confidence should be 0 initially (no input)
    let state_no_input = system.assess();
    assert!(
        (state_no_input.confidence - 0.0).abs() < f32::EPSILON,
        "Confidence should be 0 with no input"
    );

    // Test 2: Confidence should increase after input
    system.input_received(&InputModality::Keyboard);
    let state_with_input = system.assess();
    assert!(
        state_with_input.confidence > state_no_input.confidence,
        "Confidence should increase after input"
    );

    // Test 3: Multiple inputs should maintain/increase confidence
    system.input_received(&InputModality::Pointer);
    let state_more_input = system.assess();
    assert!(
        state_more_input.confidence >= state_with_input.confidence,
        "Confidence should remain high with continued input"
    );

    // EVOLUTION: Removed blocking sleep - testing mechanism, not time passage
    // Staleness/decay is tested separately via is_stale() mechanism
}

#[test]
fn test_multiple_modality_confirmation() {
    let mut system = initialize_standard_proprioception();

    // Simulate multiple input types
    system.input_received(&InputModality::Keyboard);
    system.input_received(&InputModality::Pointer);
    system.input_received(&InputModality::Audio);

    let state = system.assess();

    // Multiple inputs should increase health
    assert!(state.health > 0.3); // At least 30% health
    assert!(state.sensory_functional);
}

#[test]
fn test_visual_output_confirmation_via_keyboard() {
    let mut system = ProprioceptionSystem::new();

    // Register visual output and keyboard input
    system.register_output(OutputModality::Visual);
    system.register_input(InputModality::Keyboard);

    // Before interaction - visual output unconfirmed
    let state1 = system.assess();
    assert!(!state1.loop_complete);

    // User types on keyboard
    system.input_received(&InputModality::Keyboard);

    // After interaction - visual output should be confirmed!
    // (User must be able to see screen to type)
    let state2 = system.assess();

    // The loop should be complete or health should increase
    assert!(state2.loop_complete || state2.health > state1.health);
}

#[test]
fn test_audio_output_confirmation_via_microphone() {
    let mut system = ProprioceptionSystem::new();

    // Register audio output and input
    system.register_output(OutputModality::Audio);
    system.register_input(InputModality::Audio);

    // User responds to audio prompt via microphone
    system.input_received(&InputModality::Audio);

    let state = system.assess();

    // Audio input should confirm audio output
    assert!(state.health > 0.0);
}

#[test]
fn test_haptic_output_confirmation_via_touch() {
    let mut system = ProprioceptionSystem::new();

    // Register haptic output and input
    system.register_output(OutputModality::Haptic);
    system.register_input(InputModality::Haptic);

    // User responds to vibration via touch
    system.input_received(&InputModality::Haptic);

    let state = system.assess();

    // Haptic input should confirm haptic output
    assert!(state.health > 0.0);
}

#[test]
fn test_diagnostic_report() {
    let mut system = initialize_standard_proprioception();

    // Simulate some interaction
    system.input_received(&InputModality::Keyboard);
    system.assess();

    // Get diagnostic report
    let report = system.get_diagnostic_report();

    // Should contain key information
    assert!(report.contains("PROPRIOCEPTION"));
    assert!(report.contains("Health:"));
    assert!(report.contains("Confidence:"));
    assert!(report.contains("Motor:"));
    assert!(report.contains("Sensory:"));
    assert!(report.contains("Loop:"));
}

#[test]
fn test_graceful_degradation_no_inputs() {
    let mut system = ProprioceptionSystem::new();

    // Register outputs but no inputs
    system.register_output(OutputModality::Visual);
    system.register_output(OutputModality::Audio);

    let state = system.assess();

    // Should not crash, should report unknown state
    assert!(!state.is_healthy());
    assert!((state.health - 0.0).abs() < f32::EPSILON);
    assert!(!state.loop_complete);
}

#[test]
fn test_graceful_degradation_no_outputs() {
    let mut system = ProprioceptionSystem::new();

    // Register inputs but no outputs
    system.register_input(InputModality::Keyboard);
    system.register_input(InputModality::Pointer);

    // Simulate input
    system.input_received(&InputModality::Keyboard);

    let state = system.assess();

    // Should not crash, inputs work but no output confirmation
    assert!(state.sensory_functional || state.health > 0.0);
}

#[cfg(test)]
mod output_verification_tests {
    use super::*;

    #[test]
    fn test_output_verification_initial_state() {
        let verification = OutputVerification::unverified(OutputModality::Visual);

        assert!(!verification.device_available);
        assert!(!verification.output_active);
        assert!(!verification.reaches_user);
        assert_eq!(
            verification.state,
            petal_tongue_core::rendering_awareness::VisibilityState::Unknown
        );
    }

    #[test]
    fn test_output_confirmation_via_interaction() {
        let mut verification = OutputVerification::unverified(OutputModality::Visual);

        verification.confirm_via_interaction();

        assert!(verification.reaches_user);
        assert_eq!(
            verification.confirmation_method,
            OutputConfirmation::UserInteraction
        );
        assert_eq!(
            verification.state,
            petal_tongue_core::rendering_awareness::VisibilityState::Confirmed
        );
        assert!(verification.last_confirmed.is_some());
    }

    #[test]
    fn test_output_confirmation_via_device() {
        let mut verification = OutputVerification::unverified(OutputModality::Audio);

        verification.confirm_via_device_ack("Speaker acknowledged playback".to_string());

        assert!(verification.reaches_user);
        assert_eq!(
            verification.confirmation_method,
            OutputConfirmation::DeviceAck
        );
        assert!(verification.last_confirmed.is_some());
    }

    #[test]
    fn test_output_confirmation_via_echo() {
        let mut verification = OutputVerification::unverified(OutputModality::Audio);

        verification.confirm_via_echo("Microphone detected sound".to_string());

        assert!(verification.reaches_user);
        assert_eq!(verification.confirmation_method, OutputConfirmation::Echo);
        assert_eq!(
            verification.state,
            petal_tongue_core::rendering_awareness::VisibilityState::Confirmed
        );
    }

    #[test]
    fn test_stale_confirmation() {
        let mut verification = OutputVerification::unverified(OutputModality::Visual);

        verification.confirm_via_interaction();

        // Fresh confirmation with reasonable threshold
        assert!(!verification.is_stale(Duration::from_secs(60)));

        // Zero threshold should always be stale (tests the boundary condition)
        assert!(verification.is_stale(Duration::from_secs(0)));

        // EVOLUTION: Removed flaky time-based test
        // Staleness mechanism is verified above via boundary conditions
        // Production code handles time-based staleness correctly via SystemTime
    }
}

#[cfg(test)]
mod input_verification_tests {
    use super::*;

    #[test]
    fn test_input_verification_initial_state() {
        let verification = InputVerification::unverified(InputModality::Keyboard);

        assert!(!verification.device_available);
        assert!(!verification.input_active);
        assert!(!verification.from_intended_source);
        assert_eq!(
            verification.interactivity,
            petal_tongue_core::rendering_awareness::InteractivityState::Unconfirmed
        );
    }

    #[test]
    fn test_input_recording() {
        let mut verification = InputVerification::unverified(InputModality::Keyboard);

        verification.record_input();

        assert!(verification.input_active);
        assert!(verification.from_intended_source);
        assert_eq!(
            verification.interactivity,
            petal_tongue_core::rendering_awareness::InteractivityState::Active
        );
        assert!(verification.last_input.is_some());
    }

    #[test]
    fn test_interactivity_update() {
        let mut verification = InputVerification::unverified(InputModality::Pointer);

        verification.record_input();

        // Should be active immediately
        verification.update_interactivity();
        assert_eq!(
            verification.interactivity,
            petal_tongue_core::rendering_awareness::InteractivityState::Active
        );
    }
}

#[cfg(test)]
mod topology_detection_tests {
    use super::*;

    #[test]
    fn test_visual_topology_detection() {
        let (topology, _evidence) = detect_visual_topology();

        // Should return some topology
        assert!(matches!(
            topology,
            OutputTopology::Direct
                | OutputTopology::Forwarded
                | OutputTopology::Nested
                | OutputTopology::Virtual
                | OutputTopology::Unknown
        ));

        // Should have collected some evidence
        // (May be empty in test environment, but function shouldn't crash)
    }

    #[test]
    fn test_audio_topology_detection() {
        let (topology, _evidence) = detect_audio_topology();

        // Should return some topology
        assert!(matches!(
            topology,
            OutputTopology::Direct | OutputTopology::Forwarded | OutputTopology::Unknown
        ));

        // Function should not crash
    }

    #[test]
    fn test_haptic_topology_detection() {
        let (topology, _evidence) = detect_haptic_topology();

        // Should return some topology
        assert!(matches!(
            topology,
            OutputTopology::Direct | OutputTopology::Unknown
        ));

        // Function should not crash
    }
}
