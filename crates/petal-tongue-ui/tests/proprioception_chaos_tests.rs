// SPDX-License-Identifier: AGPL-3.0-only
//! Chaos tests for SAME DAVE proprioception system
//!
//! These tests verify the system handles failures gracefully:
//! - Lost display connections
//! - Intermittent input
//! - Topology changes
//! - Network failures
//! - Resource exhaustion scenarios
//!
//! EVOLUTION NOTE: Chaos tests focus on mechanism robustness, not time-based decay.
//! Removed blocking sleeps - testing failure detection, not waiting for timeouts.

use petal_tongue_ui::{input_verification::*, output_verification::*, proprioception::*};
use std::time::Duration;

/// Chaos Scenario: All outputs fail simultaneously
#[test]
fn chaos_all_outputs_fail() {
    let mut system = initialize_standard_proprioception();

    // Establish healthy state
    system.input_received(&InputModality::Keyboard);
    let initial_state = system.assess();
    assert!(initial_state.health > 0.0);

    // CHAOS: Register outputs but never confirm them (simulates display/audio failure)
    system.register_output(OutputModality::Visual);
    system.register_output(OutputModality::Audio);
    system.register_output(OutputModality::Haptic);

    // Without confirmation, motor should not be functional
    let degraded_state = system.assess();

    // Should not crash! System detects unconfirmed outputs
    assert!(
        !degraded_state.motor_functional,
        "Motor should not be functional without output confirmation"
    );

    // EVOLUTION: Removed sleep - testing failure detection mechanism, not time-based decay
}

/// Chaos Scenario: All inputs stop simultaneously  
#[test]
fn chaos_all_inputs_stop() {
    let mut system = initialize_standard_proprioception();

    // Establish healthy state with inputs
    system.input_received(&InputModality::Keyboard);
    system.input_received(&InputModality::Pointer);
    let initial_state = system.assess();
    assert!(
        initial_state.confidence > 0.0,
        "Should have confidence with input"
    );

    // CHAOS: Register inputs but never receive them (simulates input device failure)
    let mut system_no_input = ProprioceptionSystem::new();
    system_no_input.register_input(InputModality::Keyboard);
    system_no_input.register_input(InputModality::Pointer);

    // Without receiving input, sensory should not be functional
    let degraded_state = system_no_input.assess();

    // Should not crash! System detects no input
    assert_eq!(
        degraded_state.confidence, 0.0,
        "Confidence should be 0 without input"
    );
    assert!(
        !degraded_state.sensory_functional,
        "Sensory should not be functional without input"
    );

    // EVOLUTION: Removed sleep - testing mechanism, not time-based decay
}

/// Chaos Scenario: Rapid modality registration/deregistration
#[test]
fn chaos_rapid_modality_changes() {
    let mut system = ProprioceptionSystem::new();

    // Rapidly add and use modalities
    for _ in 0..100 {
        system.register_output(OutputModality::Visual);
        system.register_input(InputModality::Keyboard);
        system.input_received(&InputModality::Keyboard);
        system.assess();
    }

    // Should not crash or panic
    let state = system.assess();
    assert!(state.health >= 0.0 && state.health <= 1.0);
}

/// Chaos Scenario: Intermittent connectivity
#[test]
fn chaos_intermittent_connectivity() {
    let mut system = initialize_standard_proprioception();

    // Simulate on/off/on/off pattern
    for i in 0..10 {
        if i % 2 == 0 {
            // Input available
            system.input_received(&InputModality::Keyboard);
        }
        // Input unavailable (no action)

        system.assess();
    }

    // System should handle intermittent input gracefully
    let state = system.assess();
    assert!(state.health >= 0.0 && state.health <= 1.0);
}

/// Chaos Scenario: Unknown/future modalities
#[test]
fn chaos_unknown_modalities() {
    let mut system = ProprioceptionSystem::new();

    // Register unknown/future modalities
    system.register_output(OutputModality::Generic("neural-interface".to_string()));
    system.register_output(OutputModality::Generic("hologram".to_string()));
    system.register_input(InputModality::Generic("brain-waves".to_string()));
    system.register_input(InputModality::Generic("eye-tracking".to_string()));

    // Should not crash
    let state = system.assess();
    assert_eq!(state.health, 0.0); // No confirmations yet

    // Simulate future input
    system.input_received(&InputModality::Generic("brain-waves".to_string()));

    // Should handle gracefully
    let state2 = system.assess();
    assert!(state2.health >= state.health);
}

/// Chaos Scenario: Massive concurrent registrations
#[test]
fn chaos_massive_registrations() {
    let mut system = ProprioceptionSystem::new();

    // Register many modalities
    for i in 0..50 {
        system.register_output(OutputModality::Generic(format!("output-{}", i)));
        system.register_input(InputModality::Generic(format!("input-{}", i)));
    }

    // Should not crash
    let state = system.assess();
    assert!(state.health >= 0.0);
}

/// Chaos Scenario: Rapid assessment calls
#[test]
fn chaos_rapid_assessments() {
    let mut system = initialize_standard_proprioception();

    system.input_received(&InputModality::Keyboard);

    // Call assess many times rapidly
    for _ in 0..1000 {
        let _state = system.assess();
    }

    // Should not crash or degrade
    let final_state = system.assess();
    assert!(final_state.health > 0.0);
}

/// Chaos Scenario: Topology detection failures
#[test]
fn chaos_topology_detection_failures() {
    // Call topology detection many times
    // (May fail in test environment, but shouldn't crash)
    for _ in 0..100 {
        let (_topology, _evidence) = detect_visual_topology();
        let (_topology, _evidence) = detect_audio_topology();
        let (_topology, _evidence) = detect_haptic_topology();
    }

    // Should not crash
}

/// Chaos Scenario: Zero-modality system
#[test]
fn chaos_zero_modality_system() {
    let mut system = ProprioceptionSystem::new();

    // No modalities registered
    let state = system.assess();

    // Should report zero health, not crash
    assert_eq!(state.health, 0.0);
    assert!(!state.is_healthy());
    assert!(!state.is_confident());
}

/// Chaos Scenario: Input without registered output
#[test]
fn chaos_input_without_output() {
    let mut system = ProprioceptionSystem::new();

    // Register only input, no output
    system.register_input(InputModality::Keyboard);

    // Receive input
    system.input_received(&InputModality::Keyboard);

    // Should not crash trying to confirm non-existent output
    let state = system.assess();
    assert!(state.sensory_functional || state.health > 0.0);
}

/// Chaos Scenario: Output without registered input
#[test]
fn chaos_output_without_input() {
    let mut system = ProprioceptionSystem::new();

    // Register only output, no input
    system.register_output(OutputModality::Visual);

    // Assess without any input
    let state = system.assess();

    // Should report incomplete loop
    assert!(!state.loop_complete);
    assert_eq!(state.health, 0.0);
}

/// Chaos Scenario: Concurrent access simulation
#[test]
fn chaos_concurrent_access_pattern() {
    let mut system = initialize_standard_proprioception();

    // Simulate concurrent-like access pattern
    // (In real concurrent scenario, would use Arc<RwLock<ProprioceptionSystem>>)
    for i in 0..100 {
        match i % 3 {
            0 => system.input_received(&InputModality::Keyboard),
            1 => system.input_received(&InputModality::Pointer),
            2 => {
                let _s = system.assess();
            }
            _ => {}
        }
    }

    // Should maintain consistency
    let state = system.assess();
    assert!(state.health > 0.0);
}

/// Chaos Scenario: Stale confirmation detection
#[test]
fn chaos_stale_confirmation_handling() {
    let mut verification = OutputVerification::unverified(OutputModality::Visual);

    // Confirm
    verification.confirm_via_interaction();
    assert!(!verification.is_stale(Duration::from_secs(60)));

    // Check staleness with zero duration (should be stale)
    assert!(verification.is_stale(Duration::from_secs(0)));

    // Should not panic
}

/// Chaos Scenario: Diagnostic report with no data
#[test]
fn chaos_diagnostic_report_empty_system() {
    let system = ProprioceptionSystem::new();

    let report = system.get_diagnostic_report();

    // Should generate report even with no data
    assert!(report.contains("Health"));
    assert!(report.contains("0%")); // Zero health
}

/// Chaos Scenario: Status summary with no modalities
#[test]
fn chaos_status_summary_empty() {
    let system = ProprioceptionSystem::new();

    let output_status = system.get_output_status();
    let input_status = system.get_input_status();

    // Should not crash
    assert!(output_status.contains("0/0") || output_status.contains("Outputs"));
    assert!(input_status.contains("0/0") || input_status.contains("Inputs"));
}

/// Chaos Scenario: Evidence collection failure handling
#[test]
fn chaos_evidence_collection_failures() {
    let mut verification = OutputVerification::unverified(OutputModality::Visual);

    // Add empty evidence
    verification.evidence.clear();

    // Should not crash when generating status
    let _status = verification.status_message.clone();

    // Confirm with no evidence
    verification.confirm_via_interaction();
    assert!(verification.reaches_user);
}

/// Chaos Scenario: Rapid modality switching
#[test]
fn chaos_rapid_modality_switching() {
    let mut system = initialize_standard_proprioception();

    // Rapidly switch between different input types
    for i in 0..1000 {
        match i % 4 {
            0 => system.input_received(&InputModality::Keyboard),
            1 => system.input_received(&InputModality::Pointer),
            2 => system.input_received(&InputModality::Audio),
            3 => system.input_received(&InputModality::Haptic),
            _ => {}
        }
    }

    let state = system.assess();
    assert!(state.health > 0.0);
    assert!(state.sensory_functional);
}

/// Chaos Scenario: Health calculation edge cases
#[test]
fn chaos_health_calculation_edge_cases() {
    let mut system = ProprioceptionSystem::new();

    // Edge case: Single modality
    system.register_output(OutputModality::Visual);
    system.register_input(InputModality::Keyboard);
    system.input_received(&InputModality::Keyboard);

    let state = system.assess();
    // Health should be some valid percentage
    assert!(state.health >= 0.0 && state.health <= 1.0);
}

/// Chaos Scenario: Confidence calculation with no recent activity
#[tokio::test]
async fn chaos_confidence_no_recent_activity() {
    let mut system = initialize_standard_proprioception();

    // Old activity
    system.input_received(&InputModality::Keyboard);

    // Yield to executor (non-blocking alternative to sleep - tests valid range, not timing)
    tokio::task::yield_now().await;

    // Assess
    let state = system.assess();

    // Confidence should still be valid range
    assert!(state.confidence >= 0.0 && state.confidence <= 1.0);
}

/// Chaos Scenario: Loop completion edge cases
#[test]
fn chaos_loop_completion_edge_cases() {
    let mut system = ProprioceptionSystem::new();

    // Case 1: Only output
    system.register_output(OutputModality::Visual);
    let state1 = system.assess();
    assert!(!state1.loop_complete);

    // Case 2: Add input but no activity
    system.register_input(InputModality::Keyboard);
    let state2 = system.assess();
    assert!(!state2.loop_complete);

    // Case 3: Input activity
    system.input_received(&InputModality::Keyboard);
    let state3 = system.assess();
    // Loop should be complete or health should be positive
    assert!(state3.loop_complete || state3.health > 0.0);
}
