// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sensory Capability Matrix — the structured answer to "what can this user do?"
//!
//! When ludoSpring asks "can this user play my game?" or Squirrel asks "how do
//! I navigate for a blind user?", this module provides the negotiated answer.
//!
//! The matrix crosses **input capabilities** (what the user/agent can provide)
//! with **output capabilities** (what petalTongue can render). Each validated
//! cell represents a tested interaction path.

use serde::{Deserialize, Serialize};

use crate::interaction::adapter::InputModality;
use crate::interaction::perspective::OutputModality;

/// What a user/environment can provide as input.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "each bool is an independent capability flag"
)]
pub struct InputCapabilitySet {
    /// Pointer device (mouse, trackpad, stylus).
    pub pointer: bool,
    /// Physical or virtual keyboard.
    pub keyboard: bool,
    /// Voice/speech command input.
    pub voice: bool,
    /// Gesture/motion capture input.
    pub gesture: bool,
    /// Touchscreen input.
    pub touch: bool,
    /// Eye/gaze tracking.
    pub gaze: bool,
    /// Single-switch or sip-and-puff binary input.
    pub switch_input: bool,
    /// Agentic AI (Squirrel) or machine API input.
    pub api_agent: bool,
}

impl InputCapabilitySet {
    /// All known input modalities as `InputModality` values.
    #[must_use]
    pub fn active_modalities(&self) -> Vec<InputModality> {
        let mut out = Vec::new();
        if self.pointer {
            out.push(InputModality::PointerMouse);
        }
        if self.keyboard {
            out.push(InputModality::Keyboard);
        }
        if self.voice {
            out.push(InputModality::VoiceCommand);
        }
        if self.gesture {
            out.push(InputModality::MotionCapture);
        }
        if self.touch {
            out.push(InputModality::PointerTouch);
        }
        if self.gaze {
            out.push(InputModality::EyeGaze);
        }
        if self.switch_input {
            out.push(InputModality::SwitchAccess);
        }
        if self.api_agent {
            out.push(InputModality::Agent);
        }
        out
    }

    /// True when at least one input channel is available.
    #[must_use]
    pub const fn has_any(&self) -> bool {
        self.pointer
            || self.keyboard
            || self.voice
            || self.gesture
            || self.touch
            || self.gaze
            || self.switch_input
            || self.api_agent
    }

    /// Count of active input channels.
    #[must_use]
    pub const fn count(&self) -> u8 {
        self.pointer as u8
            + self.keyboard as u8
            + self.voice as u8
            + self.gesture as u8
            + self.touch as u8
            + self.gaze as u8
            + self.switch_input as u8
            + self.api_agent as u8
    }
}

/// What petalTongue can render to this user.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "each bool is an independent capability flag"
)]
pub struct OutputCapabilitySet {
    /// Graphical display (egui, web canvas).
    pub visual_gui: bool,
    /// Terminal UI (ratatui).
    pub terminal: bool,
    /// Audio sonification.
    pub audio: bool,
    /// Braille display.
    pub braille: bool,
    /// Haptic/force feedback.
    pub haptic: bool,
    /// Text description / narration (screen reader).
    pub description: bool,
    /// SVG static export.
    pub svg: bool,
    /// GPU-accelerated 3D rendering.
    pub gpu: bool,
    /// JSON/API machine-readable output.
    pub api_machine: bool,
}

impl OutputCapabilitySet {
    /// All active output modalities as `OutputModality` values.
    #[must_use]
    pub fn active_modalities(&self) -> Vec<OutputModality> {
        let mut out = Vec::new();
        if self.visual_gui {
            out.push(OutputModality::Gui);
        }
        if self.terminal {
            out.push(OutputModality::Tui);
        }
        if self.audio {
            out.push(OutputModality::Audio);
        }
        if self.braille {
            out.push(OutputModality::Braille);
        }
        if self.haptic {
            out.push(OutputModality::Haptic);
        }
        if self.description {
            out.push(OutputModality::Headless);
        }
        if self.svg {
            out.push(OutputModality::Svg);
        }
        if self.gpu {
            out.push(OutputModality::Gui);
        }
        if self.api_machine {
            out.push(OutputModality::Json);
        }
        out
    }

    /// True when at least one output channel is available.
    #[must_use]
    pub const fn has_any(&self) -> bool {
        self.visual_gui
            || self.terminal
            || self.audio
            || self.braille
            || self.haptic
            || self.description
            || self.svg
            || self.gpu
            || self.api_machine
    }

    /// Count of active output channels.
    #[must_use]
    pub const fn count(&self) -> u8 {
        self.visual_gui as u8
            + self.terminal as u8
            + self.audio as u8
            + self.braille as u8
            + self.haptic as u8
            + self.description as u8
            + self.svg as u8
            + self.gpu as u8
            + self.api_machine as u8
    }
}

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
            braille: false, // Toadstool provides this at runtime
            haptic: !caps.haptic_outputs.is_empty(),
            description: true, // always available
            svg: true,         // pure-software renderer
            gpu: !caps.visual_outputs.is_empty(),
            api_machine: true, // JSON always available
        };

        let validated_paths = Self::compute_validated_paths(&input, &output);
        let recommended_modality = Self::recommend_modality(&input, &output);
        let interaction_patterns = Self::compute_patterns(&input);

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
        let validated_paths = Self::compute_validated_paths(&input, &output);
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
        Self::compute_validated_paths(input, output)
    }

    /// Recommend modality for given input/output sets (public for IPC negotiate).
    #[must_use]
    pub const fn recommend_modality_public(
        input: &InputCapabilitySet,
        output: &OutputCapabilitySet,
    ) -> OutputModality {
        Self::recommend_modality(input, output)
    }

    /// Compute interaction patterns for given input (public for IPC negotiate).
    #[must_use]
    pub fn compute_patterns_public(input: &InputCapabilitySet) -> Vec<InteractionPattern> {
        Self::compute_patterns(input)
    }

    fn compute_validated_paths(
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

    const fn recommend_modality(
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

    fn compute_patterns(input: &InputCapabilitySet) -> Vec<InteractionPattern> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sensory_capabilities::SensoryCapabilities;

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
        let paths = SensoryCapabilityMatrix::compute_validated_paths(&input, &output);
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
        let paths = SensoryCapabilityMatrix::compute_validated_paths(&input, &output);
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
        assert_eq!(
            SensoryCapabilityMatrix::recommend_modality(&input, &output),
            OutputModality::Gui
        );
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
        assert_eq!(
            SensoryCapabilityMatrix::recommend_modality(&input, &output),
            OutputModality::Tui
        );
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
        assert_eq!(
            SensoryCapabilityMatrix::recommend_modality(&input, &output),
            OutputModality::Audio
        );
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
        let patterns = SensoryCapabilityMatrix::compute_patterns(&input);
        assert_eq!(patterns.len(), 4);
        assert!(patterns.contains(&InteractionPattern::PointAndClick));
        assert!(patterns.contains(&InteractionPattern::VoiceAndAudio));
        assert!(patterns.contains(&InteractionPattern::SwitchScanning));
    }
}
