// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::interaction::adapter::InputModality;
use crate::interaction::perspective::OutputModality;
use crate::sensory_capabilities::SensoryCapabilities;

use super::capability_sets::{InputCapabilitySet, OutputCapabilitySet};
use super::matrix::{
    InteractionPattern, SensoryCapabilityMatrix, compute_patterns, compute_validated_paths,
    recommend_modality,
};

#[test]
fn default_desktop_matrix() {
    let caps = SensoryCapabilities::discover().unwrap_or_default();
    let matrix = SensoryCapabilityMatrix::from_sensory_capabilities(&caps);
    assert!(matrix.input.has_any() || matrix.output.has_any());
    assert!(matrix.output.description);
    assert!(matrix.output.svg);
    assert!(matrix.output.api_machine);
}

#[test]
fn agent_matrix_is_api_only() {
    let matrix = SensoryCapabilityMatrix::for_agent();
    assert!(matrix.input.api_agent);
    assert!(!matrix.input.pointer);
    assert!(!matrix.input.keyboard);
    assert_eq!(matrix.recommended_modality, OutputModality::Json);
    assert!(
        matrix
            .interaction_patterns
            .contains(&InteractionPattern::AgentApi)
    );
    assert!(!matrix.validated_paths.is_empty());
}

#[test]
fn input_capability_set_count() {
    let set = InputCapabilitySet {
        pointer: true,
        keyboard: true,
        ..Default::default()
    };
    assert_eq!(set.count(), 2);
    assert!(set.has_any());
}

#[test]
fn output_capability_set_count() {
    let set = OutputCapabilitySet {
        audio: true,
        description: true,
        api_machine: true,
        ..Default::default()
    };
    assert_eq!(set.count(), 3);
    assert!(set.has_any());
}

#[test]
fn empty_sets_report_zero() {
    assert_eq!(InputCapabilitySet::default().count(), 0);
    assert!(!InputCapabilitySet::default().has_any());
    assert_eq!(OutputCapabilitySet::default().count(), 0);
    assert!(!OutputCapabilitySet::default().has_any());
}

#[test]
fn active_input_modalities() {
    let set = InputCapabilitySet {
        pointer: true,
        switch_input: true,
        api_agent: true,
        ..Default::default()
    };
    let mods = set.active_modalities();
    assert_eq!(mods.len(), 3);
    assert!(mods.contains(&InputModality::PointerMouse));
    assert!(mods.contains(&InputModality::SwitchAccess));
}

#[test]
fn active_output_modalities() {
    let set = OutputCapabilitySet {
        audio: true,
        braille: true,
        ..Default::default()
    };
    let mods = set.active_modalities();
    assert!(mods.contains(&OutputModality::Audio));
    assert!(mods.contains(&OutputModality::Braille));
}

#[test]
fn validated_paths_for_pointer_gui() {
    let input = InputCapabilitySet {
        pointer: true,
        ..Default::default()
    };
    let output = OutputCapabilitySet {
        visual_gui: true,
        ..Default::default()
    };
    let paths = compute_validated_paths(&input, &output);
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].adapter, "PointerAdapter");
    assert!((paths[0].confidence - 1.0).abs() < f64::EPSILON);
}

#[test]
fn validated_paths_for_switch_input() {
    let input = InputCapabilitySet {
        switch_input: true,
        ..Default::default()
    };
    let output = OutputCapabilitySet {
        visual_gui: true,
        audio: true,
        braille: true,
        description: true,
        ..Default::default()
    };
    let paths = compute_validated_paths(&input, &output);
    assert_eq!(paths.len(), 4);
    assert!(paths.iter().all(|p| p.adapter == "SwitchInputAdapter"));
}

#[test]
fn recommend_modality_prefers_gui() {
    let input = InputCapabilitySet {
        pointer: true,
        keyboard: true,
        ..Default::default()
    };
    let output = OutputCapabilitySet {
        visual_gui: true,
        terminal: true,
        audio: true,
        ..Default::default()
    };
    assert_eq!(recommend_modality(&input, &output), OutputModality::Gui);
}

#[test]
fn recommend_modality_falls_back_to_tui() {
    let input = InputCapabilitySet {
        keyboard: true,
        ..Default::default()
    };
    let output = OutputCapabilitySet {
        terminal: true,
        ..Default::default()
    };
    assert_eq!(recommend_modality(&input, &output), OutputModality::Tui);
}

#[test]
fn recommend_modality_blind_user_gets_audio() {
    let input = InputCapabilitySet {
        keyboard: true,
        ..Default::default()
    };
    let output = OutputCapabilitySet {
        audio: true,
        ..Default::default()
    };
    assert_eq!(recommend_modality(&input, &output), OutputModality::Audio);
}

#[test]
fn interaction_patterns_multi_input() {
    let input = InputCapabilitySet {
        pointer: true,
        keyboard: true,
        voice: true,
        switch_input: true,
        ..Default::default()
    };
    let patterns = compute_patterns(&input);
    assert_eq!(patterns.len(), 4);
    assert!(patterns.contains(&InteractionPattern::PointAndClick));
    assert!(patterns.contains(&InteractionPattern::VoiceAndAudio));
    assert!(patterns.contains(&InteractionPattern::SwitchScanning));
}
