// SPDX-License-Identifier: AGPL-3.0-only
//! Capability descriptions and display formatting.

use super::SensoryCapabilities;
use super::types::{AudioOutputCapability, VisualOutputCapability};

/// Get a human-readable description of capabilities
#[must_use]
pub fn describe(caps: &SensoryCapabilities) -> String {
    let mut parts = Vec::new();

    if !caps.visual_outputs.is_empty() {
        let visual_desc = if caps
            .visual_outputs
            .iter()
            .any(|v| matches!(v, VisualOutputCapability::ThreeD { .. }))
        {
            "3D visual"
        } else {
            "2D visual"
        };
        parts.push(visual_desc);
    }

    if !caps.audio_outputs.is_empty() {
        let audio_desc = if caps
            .audio_outputs
            .iter()
            .any(|a| matches!(a, AudioOutputCapability::Spatial { .. }))
        {
            "spatial audio"
        } else {
            "audio"
        };
        parts.push(audio_desc);
    }

    if !caps.pointer_inputs.is_empty() {
        parts.push("pointer");
    }
    if !caps.keyboard_inputs.is_empty() {
        parts.push("keyboard");
    }
    if !caps.touch_inputs.is_empty() {
        parts.push("touch");
    }
    if !caps.gesture_inputs.is_empty() {
        parts.push("gesture");
    }
    if !caps.haptic_outputs.is_empty() {
        parts.push("haptics");
    }

    if parts.is_empty() {
        "no capabilities detected".to_string()
    } else {
        parts.join(" + ")
    }
}
