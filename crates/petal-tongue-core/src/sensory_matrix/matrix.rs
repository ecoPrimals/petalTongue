// SPDX-License-Identifier: AGPL-3.0-or-later
//! Negotiated capability matrix and path computation.

use serde::{Deserialize, Serialize};

use crate::interaction::adapter::InputModality;
use crate::interaction::perspective::OutputModality;

use super::capability_sets::{InputCapabilitySet, OutputCapabilitySet};

/// A validated input→output interaction path.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidatedPath {
    /// Which input modality this path uses.
    pub input: InputModality,
    /// Which output modality this path renders to.
    pub output: OutputModality,
    /// Name of the `InputAdapter` implementation.
    pub adapter: String,
    /// Name of the `InversePipeline` (if bidirectional).
    pub inverse_pipeline: Option<String>,
    /// Confidence that this path is fully functional (0.0–1.0).
    pub confidence: f64,
}

/// Suggested interaction pattern for a capability combination.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InteractionPattern {
    /// Standard point-and-click desktop interaction.
    PointAndClick,
    /// Keyboard-driven navigation with focus ring.
    KeyboardNavigation,
    /// Voice command with audio feedback.
    VoiceAndAudio,
    /// Switch scanning with auto-advance.
    SwitchScanning,
    /// Eye gaze with dwell-to-select.
    GazeDwell,
    /// Touch gestures (pinch, swipe, tap).
    TouchGesture,
    /// Agentic AI programmatic navigation.
    AgentApi,
    /// Braille display with routing key navigation.
    BrailleRouting,
    /// Haptic exploration with force feedback.
    HapticExploration,
}

/// The full negotiated capability matrix for a session.
///
/// Built from runtime `SensoryCapabilities` discovery, this tells consumer
/// primals exactly which interaction paths are available and recommended.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SensoryCapabilityMatrix {
    /// What the user/environment can provide as input.
    pub input: InputCapabilitySet,
    /// What petalTongue can render to this user.
    pub output: OutputCapabilitySet,
    /// All validated input→output interaction paths.
    pub validated_paths: Vec<ValidatedPath>,
    /// The recommended primary output modality.
    pub recommended_modality: OutputModality,
    /// Suggested interaction patterns for this capability set.
    pub interaction_patterns: Vec<InteractionPattern>,
}

impl SensoryCapabilityMatrix {
    /// Build a matrix from runtime-discovered `SensoryCapabilities`.
    #[must_use]
    pub fn from_sensory_capabilities(
        caps: &crate::sensory_capabilities::SensoryCapabilities,
    ) -> Self {
        let input = InputCapabilitySet {
            pointer: !caps.pointer_inputs.is_empty(),
            keyboard: !caps.keyboard_inputs.is_empty(),
            voice: !caps.audio_inputs.is_empty(),
            gesture: !caps.gesture_inputs.is_empty(),
            touch: !caps.touch_inputs.is_empty(),
            gaze: caps.gesture_inputs.iter().any(|g| {
                matches!(
                    g,
                    crate::sensory_capabilities::GestureInputCapability::Eyes { .. }
                )
            }),
            switch_input: false,
            api_agent: false,
        };

        let output = OutputCapabilitySet {
            visual_gui: !caps.visual_outputs.is_empty(),
            terminal: true, // always available as a fallback
            audio: !caps.audio_outputs.is_empty(),
            braille: false, // discovered at runtime from compute/accessibility provider
            haptic: !caps.haptic_outputs.is_empty(),
            description: true, // always available
            svg: true,         // pure-software renderer
            gpu: !caps.visual_outputs.is_empty(),
            api_machine: true, // JSON always available
        };

        let validated_paths = compute_validated_paths(&input, &output);
        let recommended_modality = recommend_modality(&input, &output);
        let interaction_patterns = compute_patterns(&input);

        Self {
            input,
            output,
            validated_paths,
            recommended_modality,
            interaction_patterns,
        }
    }

    /// Build a matrix for an agentic AI (Squirrel) session — no human sensory
    /// hardware, pure API input/output.
    #[must_use]
    pub fn for_agent() -> Self {
        let input = InputCapabilitySet {
            api_agent: true,
            ..InputCapabilitySet::default()
        };
        let output = OutputCapabilitySet {
            description: true,
            api_machine: true,
            ..OutputCapabilitySet::default()
        };
        let validated_paths = compute_validated_paths(&input, &output);
        Self {
            input,
            output,
            validated_paths,
            recommended_modality: OutputModality::Json,
            interaction_patterns: vec![InteractionPattern::AgentApi],
        }
    }

    /// Compute validated paths for given input/output sets (public for IPC negotiate).
    #[must_use]
    pub fn compute_validated_paths_public(
        input: &InputCapabilitySet,
        output: &OutputCapabilitySet,
    ) -> Vec<ValidatedPath> {
        compute_validated_paths(input, output)
    }

    /// Recommend modality for given input/output sets (public for IPC negotiate).
    #[must_use]
    pub const fn recommend_modality_public(
        input: &InputCapabilitySet,
        output: &OutputCapabilitySet,
    ) -> OutputModality {
        recommend_modality(input, output)
    }

    /// Compute interaction patterns for given input (public for IPC negotiate).
    #[must_use]
    pub fn compute_patterns_public(input: &InputCapabilitySet) -> Vec<InteractionPattern> {
        compute_patterns(input)
    }
}

/// Visible within [`crate::sensory_matrix`] for unit tests in `tests.rs`.
pub(in crate::sensory_matrix) fn compute_validated_paths(
    input: &InputCapabilitySet,
    output: &OutputCapabilitySet,
) -> Vec<ValidatedPath> {
    let mut paths = Vec::new();

    if input.pointer && output.visual_gui {
        paths.push(ValidatedPath {
            input: InputModality::PointerMouse,
            output: OutputModality::Gui,
            adapter: "PointerAdapter".into(),
            inverse_pipeline: Some("VisualInversePipeline".into()),
            confidence: 1.0,
        });
    }
    if input.keyboard && output.visual_gui {
        paths.push(ValidatedPath {
            input: InputModality::Keyboard,
            output: OutputModality::Gui,
            adapter: "KeyboardAdapter".into(),
            inverse_pipeline: Some("VisualInversePipeline".into()),
            confidence: 1.0,
        });
    }
    if input.keyboard && output.terminal {
        paths.push(ValidatedPath {
            input: InputModality::Keyboard,
            output: OutputModality::Tui,
            adapter: "KeyboardAdapter".into(),
            inverse_pipeline: None,
            confidence: 0.8,
        });
    }
    if input.keyboard && output.audio {
        paths.push(ValidatedPath {
            input: InputModality::Keyboard,
            output: OutputModality::Audio,
            adapter: "KeyboardAdapter".into(),
            inverse_pipeline: Some("AudioInversePipeline".into()),
            confidence: 0.6,
        });
    }
    if input.keyboard && output.braille {
        paths.push(ValidatedPath {
            input: InputModality::Keyboard,
            output: OutputModality::Braille,
            adapter: "KeyboardAdapter".into(),
            inverse_pipeline: None,
            confidence: 0.5,
        });
    }
    if input.keyboard && output.description {
        paths.push(ValidatedPath {
            input: InputModality::Keyboard,
            output: OutputModality::Headless,
            adapter: "KeyboardAdapter".into(),
            inverse_pipeline: None,
            confidence: 0.9,
        });
    }
    if input.switch_input {
        for &(output_mod, conf) in &[
            (OutputModality::Gui, 0.7),
            (OutputModality::Audio, 0.6),
            (OutputModality::Braille, 0.5),
            (OutputModality::Headless, 0.8),
        ] {
            paths.push(ValidatedPath {
                input: InputModality::SwitchAccess,
                output: output_mod,
                adapter: "SwitchInputAdapter".into(),
                inverse_pipeline: None,
                confidence: conf,
            });
        }
    }
    if input.api_agent {
        paths.push(ValidatedPath {
            input: InputModality::Agent,
            output: OutputModality::Json,
            adapter: "AgentInputAdapter".into(),
            inverse_pipeline: None,
            confidence: 1.0,
        });
        if output.description {
            paths.push(ValidatedPath {
                input: InputModality::Agent,
                output: OutputModality::Headless,
                adapter: "AgentInputAdapter".into(),
                inverse_pipeline: None,
                confidence: 1.0,
            });
        }
    }

    paths
}

pub(in crate::sensory_matrix) const fn recommend_modality(
    input: &InputCapabilitySet,
    output: &OutputCapabilitySet,
) -> OutputModality {
    if output.visual_gui && (input.pointer || input.touch) {
        return OutputModality::Gui;
    }
    if output.terminal && input.keyboard {
        return OutputModality::Tui;
    }
    if output.audio && (input.voice || input.keyboard) {
        return OutputModality::Audio;
    }
    if output.braille && input.keyboard {
        return OutputModality::Braille;
    }
    if output.api_machine && input.api_agent {
        return OutputModality::Json;
    }
    if output.description {
        return OutputModality::Headless;
    }
    OutputModality::Json
}

pub(in crate::sensory_matrix) fn compute_patterns(
    input: &InputCapabilitySet,
) -> Vec<InteractionPattern> {
    let mut patterns = Vec::new();
    if input.pointer {
        patterns.push(InteractionPattern::PointAndClick);
    }
    if input.keyboard {
        patterns.push(InteractionPattern::KeyboardNavigation);
    }
    if input.voice {
        patterns.push(InteractionPattern::VoiceAndAudio);
    }
    if input.switch_input {
        patterns.push(InteractionPattern::SwitchScanning);
    }
    if input.gaze {
        patterns.push(InteractionPattern::GazeDwell);
    }
    if input.touch {
        patterns.push(InteractionPattern::TouchGesture);
    }
    if input.api_agent {
        patterns.push(InteractionPattern::AgentApi);
    }
    patterns
}
