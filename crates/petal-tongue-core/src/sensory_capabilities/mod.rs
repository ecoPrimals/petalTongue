// SPDX-License-Identifier: AGPL-3.0-only
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
    pub fn has_minimal_output(&self) -> bool {
        !self.visual_outputs.is_empty() || !self.audio_outputs.is_empty()
    }

    /// Check if system has minimal viable input for UI
    #[must_use]
    pub fn has_minimal_input(&self) -> bool {
        !self.pointer_inputs.is_empty()
            || !self.keyboard_inputs.is_empty()
            || !self.touch_inputs.is_empty()
            || !self.audio_inputs.is_empty()
            || !self.gesture_inputs.is_empty()
    }

    /// Check if system has visual output capability
    #[must_use]
    pub fn has_visual_output(&self) -> bool {
        !self.visual_outputs.is_empty()
    }

    /// Check if system has audio output capability
    #[must_use]
    pub fn has_audio_output(&self) -> bool {
        !self.audio_outputs.is_empty()
    }

    /// Check if system has haptic output capability
    #[must_use]
    pub fn has_haptic_output(&self) -> bool {
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
mod tests {
    use super::*;

    #[test]
    fn test_visual_2d_high_res() {
        let visual = VisualOutputCapability::TwoD {
            resolution: (1920, 1080),
            refresh_rate: 60,
            color_depth: 8,
            size_mm: None,
        };

        assert!(visual.is_high_resolution());
        assert_eq!(visual.pixel_count(), 1920 * 1080);
    }

    #[test]
    fn test_visual_2d_small_screen() {
        let visual = VisualOutputCapability::TwoD {
            resolution: (454, 454),
            refresh_rate: 60,
            color_depth: 8,
            size_mm: Some((35, 35)),
        };

        assert!(visual.is_small_screen());
        assert!(!visual.is_high_resolution());
    }

    #[test]
    fn test_ui_complexity_desktop() {
        let caps = SensoryCapabilities {
            visual_outputs: vec![VisualOutputCapability::TwoD {
                resolution: (1920, 1080),
                refresh_rate: 60,
                color_depth: 8,
                size_mm: None,
            }],
            audio_outputs: vec![AudioOutputCapability::Stereo {
                sample_rate: 48000,
                bit_depth: 16,
            }],
            pointer_inputs: vec![PointerInputCapability::TwoD {
                precision: 1.5,
                has_wheel: true,
                has_pressure: false,
                button_count: 3,
            }],
            keyboard_inputs: vec![KeyboardInputCapability::Physical {
                layout: "QWERTY".to_string(),
                has_numpad: true,
                modifier_keys: 4,
            }],
            ..Default::default()
        };

        assert_eq!(caps.determine_ui_complexity(), UIComplexity::Rich);
        assert!(caps.has_minimal_output());
        assert!(caps.has_minimal_input());
    }

    #[test]
    fn test_ui_complexity_phone() {
        let caps = SensoryCapabilities {
            visual_outputs: vec![VisualOutputCapability::TwoD {
                resolution: (1080, 2400),
                refresh_rate: 90,
                color_depth: 8,
                size_mm: Some((70, 156)),
            }],
            audio_outputs: vec![AudioOutputCapability::Stereo {
                sample_rate: 48000,
                bit_depth: 16,
            }],
            touch_inputs: vec![TouchInputCapability {
                max_touch_points: 10,
                supports_pressure: true,
                supports_hover: false,
                screen_size_mm: Some((70, 156)),
            }],
            ..Default::default()
        };

        assert_eq!(caps.determine_ui_complexity(), UIComplexity::Simple);
    }

    #[test]
    fn test_ui_complexity_vr() {
        let caps = SensoryCapabilities {
            visual_outputs: vec![VisualOutputCapability::ThreeD {
                resolution_per_eye: (1832, 1920),
                field_of_view: (110.0, 90.0),
                refresh_rate: 90,
                has_depth_tracking: true,
                has_hand_tracking: true,
            }],
            audio_outputs: vec![AudioOutputCapability::Spatial {
                channels: 8,
                sample_rate: 48000,
                has_head_tracking: true,
            }],
            haptic_outputs: vec![HapticOutputCapability::ForceFeedback {
                axes: 3,
                precision: 16,
            }],
            pointer_inputs: vec![PointerInputCapability::ThreeD {
                degrees_of_freedom: 6,
                precision: 1.0,
                has_haptics: true,
                button_count: 4,
            }],
            gesture_inputs: vec![GestureInputCapability::Hand {
                tracking_points: 21,
                precision: 2.0,
            }],
            ..Default::default()
        };

        assert_eq!(caps.determine_ui_complexity(), UIComplexity::Immersive);
    }

    #[test]
    fn test_ui_complexity_audio_only() {
        let caps = SensoryCapabilities {
            audio_outputs: vec![AudioOutputCapability::Stereo {
                sample_rate: 48000,
                bit_depth: 16,
            }],
            audio_inputs: vec![AudioInputCapability {
                sample_rate: 48000,
                channels: 1,
                has_noise_cancellation: true,
                supports_wake_word: true,
            }],
            keyboard_inputs: vec![KeyboardInputCapability::Physical {
                layout: "QWERTY".to_string(),
                has_numpad: false,
                modifier_keys: 3,
            }],
            ..Default::default()
        };

        assert_eq!(caps.determine_ui_complexity(), UIComplexity::Minimal);
    }

    #[test]
    fn test_capability_description() {
        let caps = SensoryCapabilities {
            visual_outputs: vec![VisualOutputCapability::TwoD {
                resolution: (1920, 1080),
                refresh_rate: 60,
                color_depth: 8,
                size_mm: None,
            }],
            pointer_inputs: vec![PointerInputCapability::TwoD {
                precision: 1.5,
                has_wheel: true,
                has_pressure: false,
                button_count: 3,
            }],
            keyboard_inputs: vec![KeyboardInputCapability::Physical {
                layout: "QWERTY".to_string(),
                has_numpad: true,
                modifier_keys: 4,
            }],
            ..Default::default()
        };

        let desc = caps.describe();
        assert!(desc.contains("2D visual"));
        assert!(desc.contains("pointer"));
        assert!(desc.contains("keyboard"));
    }
}
