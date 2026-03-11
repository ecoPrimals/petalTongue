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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sensory_capabilities::{
        AudioOutputCapability, GestureInputCapability, HapticOutputCapability,
        KeyboardInputCapability, PointerInputCapability, SensoryCapabilities, TouchInputCapability,
        VisualOutputCapability,
    };

    #[test]
    fn test_describe_empty() {
        let caps = SensoryCapabilities::default();
        assert_eq!(describe(&caps), "no capabilities detected");
    }

    #[test]
    fn test_describe_3d_visual_spatial_audio() {
        let caps = SensoryCapabilities {
            visual_outputs: vec![VisualOutputCapability::ThreeD {
                resolution_per_eye: (1920, 1080),
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
            haptic_outputs: vec![HapticOutputCapability::SimpleVibration {
                intensity_levels: 10,
            }],
            gesture_inputs: vec![GestureInputCapability::Hand {
                tracking_points: 21,
                precision: 2.0,
            }],
            ..Default::default()
        };
        let desc = describe(&caps);
        assert!(desc.contains("3D visual"));
        assert!(desc.contains("spatial audio"));
        assert!(desc.contains("haptics"));
        assert!(desc.contains("gesture"));
    }

    #[test]
    fn test_describe_2d_visual_audio() {
        let caps = SensoryCapabilities {
            visual_outputs: vec![VisualOutputCapability::TwoD {
                resolution: (1920, 1080),
                refresh_rate: 60,
                color_depth: 8,
                size_mm: None,
            }],
            audio_outputs: vec![AudioOutputCapability::Mono {
                sample_rate: 44100,
                bit_depth: 16,
            }],
            ..Default::default()
        };
        let desc = describe(&caps);
        assert!(desc.contains("2D visual"));
        assert!(desc.contains("audio"));
    }

    #[test]
    fn test_describe_all_input_types() {
        let caps = SensoryCapabilities {
            pointer_inputs: vec![PointerInputCapability::TwoD {
                precision: 1.0,
                has_wheel: true,
                has_pressure: false,
                button_count: 3,
            }],
            keyboard_inputs: vec![KeyboardInputCapability::Physical {
                layout: "QWERTY".to_string(),
                has_numpad: true,
                modifier_keys: 4,
            }],
            touch_inputs: vec![TouchInputCapability {
                max_touch_points: 10,
                supports_pressure: true,
                supports_hover: false,
                screen_size_mm: None,
            }],
            gesture_inputs: vec![GestureInputCapability::Hand {
                tracking_points: 21,
                precision: 2.0,
            }],
            ..Default::default()
        };
        let desc = describe(&caps);
        assert!(desc.contains("pointer"));
        assert!(desc.contains("keyboard"));
        assert!(desc.contains("touch"));
        assert!(desc.contains("gesture"));
    }
}
