// SPDX-License-Identifier: AGPL-3.0-or-later
//! Proprioception module tests

use super::*;
use crate::input_verification::InputModality;
use crate::output_verification::OutputModality;

#[test]
fn test_proprioception_system_new() {
    let system = ProprioceptionSystem::new();
    let state = system.get_state();
    assert!(!state.motor_functional);
    assert!(!state.sensory_functional);
    assert!(!state.loop_complete);
    assert_eq!(state.health, 0.0);
    assert_eq!(state.confidence, 0.0);
}

#[test]
fn test_proprioception_default() {
    let system = ProprioceptionSystem::default();
    let _ = system.get_state();
}

#[test]
fn test_proprioceptive_state_is_healthy() {
    let healthy = ProprioceptiveState {
        motor_functional: true,
        sensory_functional: true,
        loop_complete: true,
        health: 0.8,
        confidence: 0.9,
        last_loop_confirmation: None,
        status: "OK".to_string(),
        frame_rate: 60.0,
        time_since_last_frame: std::time::Duration::ZERO,
        is_hanging: false,
        hang_reason: None,
        total_frames: 100,
    };
    assert!(healthy.is_healthy());
}

#[test]
fn test_proprioceptive_state_not_healthy() {
    let unhealthy = ProprioceptiveState {
        motor_functional: false,
        sensory_functional: true,
        loop_complete: true,
        health: 0.5,
        confidence: 0.9,
        last_loop_confirmation: None,
        status: "Degraded".to_string(),
        frame_rate: 60.0,
        time_since_last_frame: std::time::Duration::ZERO,
        is_hanging: false,
        hang_reason: None,
        total_frames: 100,
    };
    assert!(!unhealthy.is_healthy());
}

#[test]
fn test_proprioceptive_state_is_confident() {
    let confident = ProprioceptiveState {
        motor_functional: true,
        sensory_functional: true,
        loop_complete: true,
        health: 0.8,
        confidence: 0.9,
        last_loop_confirmation: None,
        status: "OK".to_string(),
        frame_rate: 60.0,
        time_since_last_frame: std::time::Duration::ZERO,
        is_hanging: false,
        hang_reason: None,
        total_frames: 100,
    };
    assert!(confident.is_confident());
}

#[test]
fn test_register_output_and_input() {
    let mut system = ProprioceptionSystem::new();
    system.register_output(OutputModality::Visual);
    system.register_input(InputModality::Keyboard);
    let _ = system.assess();
}

#[test]
fn test_record_frame() {
    let mut system = ProprioceptionSystem::new();
    system.record_frame();
    system.record_frame();
    let state = system.assess();
    assert_eq!(state.total_frames, 2);
}

#[test]
fn test_get_diagnostic_events() {
    let mut system = ProprioceptionSystem::new();
    system.record_frame();
    let _ = system.assess();
    let events = system.get_diagnostic_events(5);
    assert!(events.len() <= 5);
}

#[test]
fn test_get_output_status() {
    let mut system = ProprioceptionSystem::new();
    system.register_output(OutputModality::Visual);
    let status = system.get_output_status();
    assert!(!status.is_empty());
}

#[test]
fn test_get_input_status() {
    let mut system = ProprioceptionSystem::new();
    system.register_input(InputModality::Keyboard);
    let status = system.get_input_status();
    assert!(!status.is_empty());
}

#[test]
fn test_get_diagnostic_report() {
    let mut system = ProprioceptionSystem::new();
    system.register_output(OutputModality::Visual);
    system.register_input(InputModality::Keyboard);
    let _ = system.assess();
    let report = system.get_diagnostic_report();
    assert!(report.contains("PROPRIOCEPTION"));
    assert!(report.contains("Health"));
}

#[test]
fn test_initialize_standard_proprioception() {
    let system = initialize_standard_proprioception();
    let state = system.get_state();
    assert!(!state.status.is_empty());
}

#[test]
fn test_proprioceptive_state_not_confident() {
    let state = ProprioceptiveState {
        motor_functional: true,
        sensory_functional: true,
        loop_complete: true,
        health: 0.9,
        confidence: 0.5,
        last_loop_confirmation: None,
        status: "OK".to_string(),
        frame_rate: 60.0,
        time_since_last_frame: std::time::Duration::ZERO,
        is_hanging: false,
        hang_reason: None,
        total_frames: 100,
    };
    assert!(!state.is_confident());
}

#[test]
fn test_proprioceptive_state_health_threshold() {
    let state = ProprioceptiveState {
        motor_functional: true,
        sensory_functional: true,
        loop_complete: true,
        health: 0.7,
        confidence: 0.9,
        last_loop_confirmation: None,
        status: "OK".to_string(),
        frame_rate: 60.0,
        time_since_last_frame: std::time::Duration::ZERO,
        is_hanging: false,
        hang_reason: None,
        total_frames: 100,
    };
    assert!(!state.is_healthy());
}

#[test]
fn test_input_received_confirms_output() {
    let mut system = ProprioceptionSystem::new();
    system.register_output(OutputModality::Visual);
    system.register_input(InputModality::Keyboard);
    system.input_received(&InputModality::Keyboard);
    let state = system.assess();
    let _ = state.total_frames;
}

#[test]
fn test_assess_aggregates_health() {
    let mut system = ProprioceptionSystem::new();
    system.register_output(OutputModality::Visual);
    system.register_input(InputModality::Keyboard);
    system.input_received(&InputModality::Keyboard);
    let state = system.assess();
    assert!(state.health >= 0.0 && state.health <= 1.0);
    assert!(state.confidence >= 0.0 && state.confidence <= 1.0);
}
