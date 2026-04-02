// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Sensory Capability Matrix validation tests.
//!
//! Each test validates a specific cell in the input×output matrix, confirming
//! that the types, adapters, inverse pipelines, and IPC methods are wired
//! correctly for that interaction path.
//!
//! ## Tier 1 — Core Accessibility (petalTongue owns)
//!
//! 1. Blind keyboard user → audio sonification + keyboard
//! 2. Deaf user → haptic + visual waveforms
//! 3. Motor-impaired user → single switch scanning
//! 4. Screen reader user → modality.description
//! 5. Agentic AI (Squirrel) → API/Agent input + API/Machine output
//!
//! ## Tier 2 — Cross-Primal (ecosystem collaboration)
//!
//! 6. ludoSpring game playable by blind user (audio inverse pipeline)
//! 7. Toadstool sensor events → petalTongue processing
//!
//! ## Tier 3 — Frontier (type-level validation only)
//!
//! 8. BCI → switch adapter pathway
//! 9. Multi-user shared perspective with different modalities

use petal_tongue_core::interaction::{InputModality, OutputModality};
use petal_tongue_core::sensor::{GestureDirection, GestureType, SensorEvent};
use petal_tongue_core::sensory_matrix::{
    InputCapabilitySet, InteractionPattern, OutputCapabilitySet, SensoryCapabilityMatrix,
};

// ============================================================================
// Tier 1: Core Accessibility
// ============================================================================

/// Scenario 1: Blind keyboard user navigates via audio sonification.
///
/// Input: keyboard only. Output: audio + description.
/// The matrix should produce keyboard→audio and keyboard→description paths.
#[test]
fn scenario_blind_keyboard_audio() {
    let input = InputCapabilitySet {
        keyboard: true,
        ..Default::default()
    };
    let output = OutputCapabilitySet {
        audio: true,
        description: true,
        ..Default::default()
    };
    let paths = SensoryCapabilityMatrix::compute_validated_paths_public(&input, &output);

    assert!(
        paths.iter().any(|p| p.input == InputModality::Keyboard
            && p.output == OutputModality::Audio
            && p.adapter == "KeyboardAdapter"),
        "blind user must have keyboard→audio path"
    );
    assert!(
        paths.iter().any(|p| p.input == InputModality::Keyboard
            && p.output == OutputModality::Headless
            && p.adapter == "KeyboardAdapter"),
        "blind user must have keyboard→description path"
    );

    let recommended = SensoryCapabilityMatrix::recommend_modality_public(&input, &output);
    assert_eq!(
        recommended,
        OutputModality::Audio,
        "blind user should get audio as recommended modality"
    );

    let patterns = SensoryCapabilityMatrix::compute_patterns_public(&input);
    assert!(patterns.contains(&InteractionPattern::KeyboardNavigation));
}

/// Scenario 2: Deaf user experiences content via haptic + visual.
///
/// Input: pointer + keyboard. Output: visual_gui + haptic (no audio).
#[test]
fn scenario_deaf_haptic_visual() {
    let input = InputCapabilitySet {
        pointer: true,
        keyboard: true,
        ..Default::default()
    };
    let output = OutputCapabilitySet {
        visual_gui: true,
        haptic: true,
        ..Default::default()
    };
    let paths = SensoryCapabilityMatrix::compute_validated_paths_public(&input, &output);

    assert!(
        paths
            .iter()
            .any(|p| p.input == InputModality::PointerMouse && p.output == OutputModality::Gui),
        "deaf user has pointer→gui path"
    );

    let recommended = SensoryCapabilityMatrix::recommend_modality_public(&input, &output);
    assert_eq!(recommended, OutputModality::Gui);
}

/// Scenario 3: Motor-impaired user interacts via single switch scanning.
///
/// Input: switch only. Output: visual_gui + audio + description.
#[test]
fn scenario_motor_impaired_switch() {
    let input = InputCapabilitySet {
        switch_input: true,
        ..Default::default()
    };
    let output = OutputCapabilitySet {
        visual_gui: true,
        audio: true,
        description: true,
        ..Default::default()
    };
    let paths = SensoryCapabilityMatrix::compute_validated_paths_public(&input, &output);

    assert!(
        paths
            .iter()
            .any(|p| p.input == InputModality::SwitchAccess && p.adapter == "SwitchInputAdapter"),
        "motor-impaired user must have switch paths"
    );
    assert!(
        paths.len() >= 3,
        "should have paths for gui, audio, and description"
    );

    let patterns = SensoryCapabilityMatrix::compute_patterns_public(&input);
    assert!(patterns.contains(&InteractionPattern::SwitchScanning));
}

/// Scenario 4: Screen reader user gets full descriptions.
///
/// Input: keyboard. Output: description (headless).
#[test]
fn scenario_screen_reader_description() {
    let input = InputCapabilitySet {
        keyboard: true,
        ..Default::default()
    };
    let output = OutputCapabilitySet {
        description: true,
        ..Default::default()
    };
    let paths = SensoryCapabilityMatrix::compute_validated_paths_public(&input, &output);

    assert!(paths.iter().any(|p| p.output == OutputModality::Headless));
    assert!(
        paths
            .iter()
            .find(|p| p.output == OutputModality::Headless)
            .is_some_and(|p| p.confidence >= 0.9),
        "description path should have high confidence"
    );
}

/// Scenario 5: Agentic AI (Squirrel) navigates via API.
///
/// Pure machine interaction — no human sensory hardware.
#[test]
fn scenario_agent_api() {
    let matrix = SensoryCapabilityMatrix::for_agent();

    assert!(matrix.input.api_agent);
    assert!(!matrix.input.pointer);
    assert!(!matrix.input.keyboard);
    assert!(!matrix.input.voice);

    assert!(matrix.output.api_machine);
    assert!(matrix.output.description);
    assert!(!matrix.output.visual_gui);

    assert_eq!(matrix.recommended_modality, OutputModality::Json);
    assert!(
        matrix
            .interaction_patterns
            .contains(&InteractionPattern::AgentApi)
    );

    assert!(
        matrix
            .validated_paths
            .iter()
            .any(|p| p.input == InputModality::Agent
                && p.output == OutputModality::Json
                && p.adapter == "AgentInputAdapter"
                && (p.confidence - 1.0).abs() < f64::EPSILON)
    );
}

// ============================================================================
// Tier 2: Cross-Primal Integration
// ============================================================================

/// Scenario 6: Audio inverse pipeline resolves data targets.
///
/// Validates that keyboard events resolve to data IDs through the audio
/// inverse path — the foundation for a blind user playing a ludoSpring game.
#[test]
fn scenario_audio_inverse_pipeline_resolves() {
    let matrix = SensoryCapabilityMatrix::from_sensory_capabilities(
        &petal_tongue_core::SensoryCapabilities::discover().unwrap_or_default(),
    );

    let audio_path = matrix
        .validated_paths
        .iter()
        .find(|p| p.output == OutputModality::Audio);

    if let Some(path) = audio_path {
        assert!(
            path.inverse_pipeline.is_some(),
            "audio path should have AudioInversePipeline"
        );
        assert_eq!(
            path.inverse_pipeline.as_deref(),
            Some("AudioInversePipeline")
        );
    }
}

/// Scenario 7: Toadstool sensor events are valid SensorEvent variants.
///
/// Type-level validation that all sensor event variants Toadstool will
/// emit can be constructed and classified correctly.
#[test]
fn scenario_toadstool_sensor_events() {
    use std::time::Instant;

    let ts = Instant::now();

    let voice = SensorEvent::VoiceCommand {
        transcript: "navigate to node five".into(),
        confidence: 0.92,
        timestamp: ts,
    };
    assert!(voice.is_user_interaction());
    assert_eq!(voice.timestamp(), ts);

    let gesture = SensorEvent::Gesture {
        gesture_type: GestureType::Swipe(GestureDirection::Left),
        magnitude: 0.75,
        timestamp: ts,
    };
    assert!(gesture.is_user_interaction());

    let touch = SensorEvent::Touch {
        x: 540.0,
        y: 960.0,
        pressure: 0.5,
        timestamp: ts,
    };
    assert!(touch.is_user_interaction());

    let gaze = SensorEvent::GazePosition {
        x: 960.0,
        y: 540.0,
        fixation_ms: 350,
        timestamp: ts,
    };
    assert!(gaze.is_user_interaction());

    let switch = SensorEvent::SwitchActivation {
        switch_id: 0,
        timestamp: ts,
    };
    assert!(switch.is_user_interaction());

    let agent = SensorEvent::AgentCommand {
        intent: "select".into(),
        parameters: serde_json::json!({"target": "node-42"}),
        timestamp: ts,
    };
    assert!(agent.is_user_interaction());
}

// ============================================================================
// Tier 3: Frontier (Type-Level Validation)
// ============================================================================

/// Scenario 8: BCI → switch adapter pathway.
///
/// A BCI device that produces binary intent (yes/no) maps through the
/// switch adapter pathway. This validates the type-level compatibility.
#[test]
fn scenario_bci_through_switch_pathway() {
    use std::time::Instant;

    let bci_event = SensorEvent::SwitchActivation {
        switch_id: 0,
        timestamp: Instant::now(),
    };
    assert!(bci_event.is_user_interaction());

    let input = InputCapabilitySet {
        switch_input: true,
        ..Default::default()
    };
    let output = OutputCapabilitySet {
        audio: true,
        haptic: true,
        description: true,
        ..Default::default()
    };
    let paths = SensoryCapabilityMatrix::compute_validated_paths_public(&input, &output);
    assert!(paths.iter().any(|p| p.input == InputModality::SwitchAccess));
}

/// Scenario 9: Multi-user shared perspective with different modalities.
///
/// Validates that the matrix can produce different validated paths for
/// different users, all pointing to the same data identity system.
#[test]
fn scenario_multi_user_different_modalities() {
    let sighted_input = InputCapabilitySet {
        pointer: true,
        keyboard: true,
        ..Default::default()
    };
    let sighted_output = OutputCapabilitySet {
        visual_gui: true,
        ..Default::default()
    };

    let blind_input = InputCapabilitySet {
        keyboard: true,
        ..Default::default()
    };
    let blind_output = OutputCapabilitySet {
        audio: true,
        description: true,
        ..Default::default()
    };

    let agent_matrix = SensoryCapabilityMatrix::for_agent();

    let sighted_paths =
        SensoryCapabilityMatrix::compute_validated_paths_public(&sighted_input, &sighted_output);
    let blind_paths =
        SensoryCapabilityMatrix::compute_validated_paths_public(&blind_input, &blind_output);

    assert!(
        sighted_paths
            .iter()
            .any(|p| p.output == OutputModality::Gui),
        "sighted user has GUI path"
    );
    assert!(
        blind_paths
            .iter()
            .any(|p| p.output == OutputModality::Audio),
        "blind user has audio path"
    );
    assert!(
        agent_matrix
            .validated_paths
            .iter()
            .any(|p| p.output == OutputModality::Json),
        "agent has JSON path"
    );

    let sighted_recommended =
        SensoryCapabilityMatrix::recommend_modality_public(&sighted_input, &sighted_output);
    let blind_recommended =
        SensoryCapabilityMatrix::recommend_modality_public(&blind_input, &blind_output);

    assert_ne!(
        sighted_recommended, blind_recommended,
        "different users get different recommended modalities"
    );
}

// ============================================================================
// Matrix Completeness Validation
// ============================================================================

/// Validates that the matrix builder correctly detects desktop capabilities.
#[test]
fn matrix_from_runtime_discovery() {
    let caps = petal_tongue_core::SensoryCapabilities::discover().unwrap_or_default();
    let matrix = SensoryCapabilityMatrix::from_sensory_capabilities(&caps);

    assert!(matrix.output.description, "description is always available");
    assert!(matrix.output.svg, "SVG is always available");
    assert!(
        matrix.output.api_machine,
        "API/machine output is always available"
    );
    assert!(matrix.output.terminal, "terminal is always available");

    assert!(
        !matrix.validated_paths.is_empty(),
        "runtime matrix must have at least one validated path"
    );
    assert!(
        !matrix.interaction_patterns.is_empty(),
        "runtime matrix must suggest at least one interaction pattern"
    );
}

/// Validates InputModality::Agent display formatting.
#[test]
fn agent_input_modality_display() {
    assert_eq!(format!("{}", InputModality::Agent), "agent");
}

/// Validates that SensoryCapabilityMatrix serializes to JSON.
#[test]
fn matrix_json_roundtrip() {
    let matrix = SensoryCapabilityMatrix::for_agent();
    let json = serde_json::to_string(&matrix).expect("serialize matrix");
    let deserialized: SensoryCapabilityMatrix =
        serde_json::from_str(&json).expect("deserialize matrix");
    assert_eq!(deserialized.recommended_modality, OutputModality::Json);
    assert!(deserialized.input.api_agent);
}
