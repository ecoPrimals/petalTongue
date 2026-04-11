// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sensory Capability System - Runtime I/O Discovery
//!
//! This module implements TRUE PRIMAL capability discovery by detecting
//! available human sensory I/O channels at runtime.

mod complexity;
mod display;
mod types;

pub use complexity::{UIComplexity, determine_ui_complexity};
pub use types::*;

use serde::{Deserialize, Serialize};

/// Complete set of sensory capabilities available to the system
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SensoryCapabilities {
    /// Visual output capabilities (displays)
    pub visual_outputs: Vec<VisualOutputCapability>,

    /// Audio output capabilities (speakers, headphones)
    pub audio_outputs: Vec<AudioOutputCapability>,

    /// Haptic output capabilities (vibration, force feedback)
    pub haptic_outputs: Vec<HapticOutputCapability>,

    /// Taste output capabilities (future: chemical delivery)
    pub taste_outputs: Vec<TasteOutputCapability>,

    /// Smell output capabilities (future: scent generators)
    pub smell_outputs: Vec<SmellOutputCapability>,

    /// Neural output capabilities (future: BCI, cortex stimulation)
    pub neural_outputs: Vec<NeuralOutputCapability>,

    /// Pointer input capabilities (mouse, VR controllers)
    pub pointer_inputs: Vec<PointerInputCapability>,

    /// Keyboard input capabilities (physical, virtual)
    pub keyboard_inputs: Vec<KeyboardInputCapability>,

    /// Touch input capabilities (touchscreen, touchpad)
    pub touch_inputs: Vec<TouchInputCapability>,

    /// Audio input capabilities (microphone, voice)
    pub audio_inputs: Vec<AudioInputCapability>,

    /// Gesture input capabilities (hand tracking, body, eyes)
    pub gesture_inputs: Vec<GestureInputCapability>,

    /// Neural input capabilities (future: BCI, thought input)
    pub neural_inputs: Vec<NeuralInputCapability>,
}

impl SensoryCapabilities {
    /// Determine appropriate UI complexity from discovered capabilities
    #[must_use]
    pub fn determine_ui_complexity(&self) -> UIComplexity {
        complexity::determine_ui_complexity(self)
    }

    /// Check if system has minimal viable output for UI
    #[must_use]
    pub const fn has_minimal_output(&self) -> bool {
        !self.visual_outputs.is_empty() || !self.audio_outputs.is_empty()
    }

    /// Check if system has minimal viable input for UI
    #[must_use]
    pub const fn has_minimal_input(&self) -> bool {
        !self.pointer_inputs.is_empty()
            || !self.keyboard_inputs.is_empty()
            || !self.touch_inputs.is_empty()
            || !self.audio_inputs.is_empty()
            || !self.gesture_inputs.is_empty()
    }

    /// Check if system has visual output capability
    #[must_use]
    pub const fn has_visual_output(&self) -> bool {
        !self.visual_outputs.is_empty()
    }

    /// Check if system has audio output capability
    #[must_use]
    pub const fn has_audio_output(&self) -> bool {
        !self.audio_outputs.is_empty()
    }

    /// Check if system has haptic output capability
    #[must_use]
    pub const fn has_haptic_output(&self) -> bool {
        !self.haptic_outputs.is_empty()
    }

    /// Get a human-readable description of capabilities
    #[must_use]
    pub fn describe(&self) -> String {
        display::describe(self)
    }
}

/// Errors that can occur during capability discovery
#[derive(Debug, Clone, thiserror::Error)]
pub enum CapabilityError {
    /// No visual or audio output detected
    #[error("No output capabilities detected (need visual or audio)")]
    NoOutput,

    /// No input capabilities detected
    #[error("No input capabilities detected")]
    NoInput,

    /// Platform detection failed
    #[error("Platform detection failed: {0}")]
    DetectionFailed(String),

    /// Unsupported platform
    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),
}

#[cfg(test)]
mod tests;
