// SPDX-License-Identifier: AGPL-3.0-only
//! UI complexity determination from sensory capabilities.

use std::fmt;

use super::SensoryCapabilities;
use super::types::{AudioOutputCapability, PointerInputCapability, VisualOutputCapability};

/// UI complexity level derived from discovered capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum UIComplexity {
    /// Very limited I/O (e.g., audio-only for accessibility)
    Minimal,
    /// Basic I/O (e.g., small screen + touch, or keyboard-only)
    Simple,
    /// Full I/O (e.g., desktop with mouse + keyboard)
    Standard,
    /// Enhanced I/O (e.g., large 4K display + precision input)
    Rich,
    /// Immersive I/O (e.g., VR with spatial audio and haptics)
    Immersive,
}

impl fmt::Display for UIComplexity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Minimal => write!(f, "Minimal"),
            Self::Simple => write!(f, "Simple"),
            Self::Standard => write!(f, "Standard"),
            Self::Rich => write!(f, "Rich"),
            Self::Immersive => write!(f, "Immersive"),
        }
    }
}

/// Determine appropriate UI complexity from discovered capabilities
#[must_use]
pub fn determine_ui_complexity(caps: &SensoryCapabilities) -> UIComplexity {
    let has_3d_visual = caps
        .visual_outputs
        .iter()
        .any(|v| matches!(v, VisualOutputCapability::ThreeD { .. }));

    let has_spatial_audio = caps
        .audio_outputs
        .iter()
        .any(|a| matches!(a, AudioOutputCapability::Spatial { .. }));

    let has_haptics = !caps.haptic_outputs.is_empty();

    if has_3d_visual && has_spatial_audio && has_haptics {
        return UIComplexity::Immersive;
    }

    let has_high_res = caps
        .visual_outputs
        .iter()
        .any(VisualOutputCapability::is_high_resolution);

    let has_precision_pointer = caps
        .pointer_inputs
        .iter()
        .any(PointerInputCapability::is_precision);

    let has_keyboard = !caps.keyboard_inputs.is_empty();

    if has_high_res && has_precision_pointer && has_keyboard {
        return UIComplexity::Rich;
    }

    if has_high_res || (has_keyboard && has_precision_pointer) {
        return UIComplexity::Standard;
    }

    let has_touch = !caps.touch_inputs.is_empty();

    let has_small_screen = caps
        .visual_outputs
        .iter()
        .any(VisualOutputCapability::is_small_screen);

    if has_small_screen || (has_touch && !has_keyboard) {
        return UIComplexity::Simple;
    }

    if caps.visual_outputs.is_empty() && !caps.audio_outputs.is_empty() {
        return UIComplexity::Minimal;
    }

    UIComplexity::Standard
}
