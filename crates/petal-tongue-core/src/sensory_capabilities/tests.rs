// SPDX-License-Identifier: AGPL-3.0-or-later
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

#[test]
fn test_has_minimal_output_false() {
    let caps = SensoryCapabilities::default();
    assert!(!caps.has_minimal_output());
    assert!(!caps.has_minimal_input());
}

#[test]
fn test_has_visual_audio_haptic() {
    let caps = SensoryCapabilities {
        visual_outputs: vec![VisualOutputCapability::TwoD {
            resolution: (800, 600),
            refresh_rate: 60,
            color_depth: 8,
            size_mm: None,
        }],
        audio_outputs: vec![AudioOutputCapability::Mono {
            sample_rate: 44100,
            bit_depth: 16,
        }],
        haptic_outputs: vec![HapticOutputCapability::SimpleVibration {
            intensity_levels: 10,
        }],
        ..Default::default()
    };
    assert!(caps.has_visual_output());
    assert!(caps.has_audio_output());
    assert!(caps.has_haptic_output());
}

#[test]
fn test_capability_error_display() {
    let err = CapabilityError::NoOutput;
    assert!(err.to_string().contains("output"));
    let err = CapabilityError::NoInput;
    assert!(err.to_string().contains("input"));
    let err = CapabilityError::DetectionFailed("failed".to_string());
    assert!(err.to_string().contains("failed"));
    let err = CapabilityError::UnsupportedPlatform("win".to_string());
    assert!(err.to_string().contains("win"));
}

#[test]
fn test_sensory_capabilities_default() {
    let caps = SensoryCapabilities::default();
    assert!(caps.visual_outputs.is_empty());
    assert!(caps.audio_outputs.is_empty());
    assert!(caps.pointer_inputs.is_empty());
    assert!(caps.keyboard_inputs.is_empty());
}

#[test]
fn test_minimal_output_audio_only() {
    let caps = SensoryCapabilities {
        audio_outputs: vec![AudioOutputCapability::Mono {
            sample_rate: 44100,
            bit_depth: 16,
        }],
        ..Default::default()
    };
    assert!(caps.has_minimal_output());
    assert!(!caps.has_minimal_input());
}

#[test]
fn test_minimal_input_audio_only() {
    let caps = SensoryCapabilities {
        audio_inputs: vec![AudioInputCapability {
            sample_rate: 48000,
            channels: 1,
            has_noise_cancellation: false,
            supports_wake_word: false,
        }],
        ..Default::default()
    };
    assert!(!caps.has_minimal_output());
    assert!(caps.has_minimal_input());
}

#[test]
fn test_visual_2d_diagonal_mm_none() {
    let visual = VisualOutputCapability::TwoD {
        resolution: (800, 600),
        refresh_rate: 60,
        color_depth: 8,
        size_mm: None,
    };
    assert!(visual.diagonal_mm().is_none());
    assert!(!visual.is_small_screen());
}

#[test]
fn test_visual_2d_diagonal_mm_some() {
    let visual = VisualOutputCapability::TwoD {
        resolution: (1920, 1080),
        refresh_rate: 60,
        color_depth: 8,
        size_mm: Some((400, 225)),
    };
    let diag = visual.diagonal_mm().unwrap();
    assert!(diag > 400.0);
    assert!(!visual.is_small_screen());
}

#[test]
fn test_visual_3d_pixel_count() {
    let visual = VisualOutputCapability::ThreeD {
        resolution_per_eye: (960, 1080),
        field_of_view: (110.0, 90.0),
        refresh_rate: 90,
        has_depth_tracking: true,
        has_hand_tracking: false,
    };
    assert_eq!(visual.pixel_count(), 960 * 1080 * 2);
    assert!(!visual.is_high_resolution());
}

#[test]
fn test_visual_3d_high_resolution() {
    let visual = VisualOutputCapability::ThreeD {
        resolution_per_eye: (1920, 1080),
        field_of_view: (110.0, 90.0),
        refresh_rate: 90,
        has_depth_tracking: true,
        has_hand_tracking: true,
    };
    assert!(visual.is_high_resolution());
}

#[test]
fn test_audio_mono_high_quality() {
    let audio = AudioOutputCapability::Mono {
        sample_rate: 48000,
        bit_depth: 16,
    };
    assert!(audio.is_high_quality());
}

#[test]
fn test_audio_mono_low_quality() {
    let audio = AudioOutputCapability::Mono {
        sample_rate: 44100,
        bit_depth: 8,
    };
    assert!(!audio.is_high_quality());
}

#[test]
fn test_audio_spatial_high_quality() {
    let audio = AudioOutputCapability::Spatial {
        channels: 8,
        sample_rate: 48000,
        has_head_tracking: true,
    };
    assert!(audio.is_high_quality());
}

#[test]
fn test_pointer_2d_precision() {
    let ptr = PointerInputCapability::TwoD {
        precision: 1.5,
        has_wheel: true,
        has_pressure: false,
        button_count: 3,
    };
    assert!(ptr.is_precision());
}

#[test]
fn test_pointer_2d_not_precision() {
    let ptr = PointerInputCapability::TwoD {
        precision: 0.5,
        has_wheel: false,
        has_pressure: false,
        button_count: 2,
    };
    assert!(!ptr.is_precision());
}

#[test]
fn test_pointer_3d_precision() {
    let ptr = PointerInputCapability::ThreeD {
        degrees_of_freedom: 6,
        precision: 2.0,
        has_haptics: true,
        button_count: 4,
    };
    assert!(ptr.is_precision());
}

#[test]
fn test_pointer_3d_not_precision() {
    let ptr = PointerInputCapability::ThreeD {
        degrees_of_freedom: 3,
        precision: 10.0,
        has_haptics: false,
        button_count: 2,
    };
    assert!(!ptr.is_precision());
}

#[test]
fn test_touch_multitouch() {
    let touch = TouchInputCapability {
        max_touch_points: 10,
        supports_pressure: true,
        supports_hover: false,
        screen_size_mm: None,
    };
    assert!(touch.is_multitouch());
}

#[test]
fn test_touch_single_touch() {
    let touch = TouchInputCapability {
        max_touch_points: 1,
        supports_pressure: false,
        supports_hover: false,
        screen_size_mm: None,
    };
    assert!(!touch.is_multitouch());
}

#[test]
fn test_visual_output_capability_serde() {
    let visual = VisualOutputCapability::TwoD {
        resolution: (1920, 1080),
        refresh_rate: 60,
        color_depth: 8,
        size_mm: Some((400, 225)),
    };
    let json = serde_json::to_string(&visual).unwrap();
    let restored: VisualOutputCapability = serde_json::from_str(&json).unwrap();
    assert_eq!(visual, restored);
}

#[test]
fn test_audio_output_capability_serde() {
    let audio = AudioOutputCapability::Stereo {
        sample_rate: 48000,
        bit_depth: 24,
    };
    let json = serde_json::to_string(&audio).unwrap();
    let restored: AudioOutputCapability = serde_json::from_str(&json).unwrap();
    assert_eq!(audio, restored);
}

#[test]
fn test_haptic_output_capability_serde() {
    let haptic = HapticOutputCapability::ForceFeedback {
        axes: 3,
        precision: 16,
    };
    let json = serde_json::to_string(&haptic).unwrap();
    let restored: HapticOutputCapability = serde_json::from_str(&json).unwrap();
    assert_eq!(haptic, restored);
}

#[test]
fn test_taste_output_capability_serde() {
    let taste = TasteOutputCapability {
        basic_tastes: vec!["sweet".to_string(), "sour".to_string()],
    };
    let json = serde_json::to_string(&taste).unwrap();
    let restored: TasteOutputCapability = serde_json::from_str(&json).unwrap();
    assert_eq!(taste, restored);
}

#[test]
fn test_smell_output_capability_serde() {
    let smell = SmellOutputCapability { scent_channels: 4 };
    let json = serde_json::to_string(&smell).unwrap();
    let restored: SmellOutputCapability = serde_json::from_str(&json).unwrap();
    assert_eq!(smell, restored);
}

#[test]
fn test_neural_output_capability_serde() {
    let neural = NeuralOutputCapability {
        signal_types: vec!["visual_cortex".to_string()],
        bandwidth: 1_000_000,
    };
    let json = serde_json::to_string(&neural).unwrap();
    let restored: NeuralOutputCapability = serde_json::from_str(&json).unwrap();
    assert_eq!(neural, restored);
}

#[test]
fn test_pointer_input_capability_serde() {
    let ptr = PointerInputCapability::TwoD {
        precision: 1.0,
        has_wheel: true,
        has_pressure: true,
        button_count: 5,
    };
    let json = serde_json::to_string(&ptr).unwrap();
    let restored: PointerInputCapability = serde_json::from_str(&json).unwrap();
    assert_eq!(ptr, restored);
}

#[test]
fn test_keyboard_input_capability_serde() {
    let kb = KeyboardInputCapability::Physical {
        layout: "QWERTY".to_string(),
        has_numpad: true,
        modifier_keys: 4,
    };
    let json = serde_json::to_string(&kb).unwrap();
    let restored: KeyboardInputCapability = serde_json::from_str(&json).unwrap();
    assert_eq!(kb, restored);
}

#[test]
fn test_touch_input_capability_serde() {
    let touch = TouchInputCapability {
        max_touch_points: 5,
        supports_pressure: true,
        supports_hover: true,
        screen_size_mm: Some((100, 200)),
    };
    let json = serde_json::to_string(&touch).unwrap();
    let restored: TouchInputCapability = serde_json::from_str(&json).unwrap();
    assert_eq!(touch, restored);
}

#[test]
fn test_audio_input_capability_serde() {
    let audio = AudioInputCapability {
        sample_rate: 48000,
        channels: 2,
        has_noise_cancellation: true,
        supports_wake_word: true,
    };
    let json = serde_json::to_string(&audio).unwrap();
    let restored: AudioInputCapability = serde_json::from_str(&json).unwrap();
    assert_eq!(audio, restored);
}

#[test]
fn test_gesture_input_capability_serde() {
    let gesture = GestureInputCapability::Hand {
        tracking_points: 21,
        precision: 2.0,
    };
    let json = serde_json::to_string(&gesture).unwrap();
    let restored: GestureInputCapability = serde_json::from_str(&json).unwrap();
    assert_eq!(gesture, restored);
}

#[test]
fn test_neural_input_capability_serde() {
    let neural = NeuralInputCapability {
        signal_types: vec!["EEG".to_string(), "EMG".to_string()],
        channels: 32,
        bandwidth: 500_000,
    };
    let json = serde_json::to_string(&neural).unwrap();
    let restored: NeuralInputCapability = serde_json::from_str(&json).unwrap();
    assert_eq!(neural, restored);
}

#[test]
fn test_keyboard_virtual_variant() {
    let kb = KeyboardInputCapability::Virtual {
        layout: "QWERTY".to_string(),
        supports_autocomplete: true,
        supports_swipe: true,
    };
    let json = serde_json::to_string(&kb).unwrap();
    let restored: KeyboardInputCapability = serde_json::from_str(&json).unwrap();
    assert_eq!(kb, restored);
}

#[test]
fn test_keyboard_chorded_variant() {
    let kb = KeyboardInputCapability::Chorded { key_count: 22 };
    let json = serde_json::to_string(&kb).unwrap();
    let restored: KeyboardInputCapability = serde_json::from_str(&json).unwrap();
    assert_eq!(kb, restored);
}

#[test]
fn test_gesture_full_body_variant() {
    let gesture = GestureInputCapability::FullBody {
        tracking_points: 33,
        has_facial_tracking: true,
    };
    let json = serde_json::to_string(&gesture).unwrap();
    let restored: GestureInputCapability = serde_json::from_str(&json).unwrap();
    assert_eq!(gesture, restored);
}

#[test]
fn test_gesture_eyes_variant() {
    let gesture = GestureInputCapability::Eyes {
        precision: 0.5,
        supports_blink_detection: true,
    };
    let json = serde_json::to_string(&gesture).unwrap();
    let restored: GestureInputCapability = serde_json::from_str(&json).unwrap();
    assert_eq!(gesture, restored);
}

#[test]
fn test_haptic_simple_vibration_variant() {
    let haptic = HapticOutputCapability::SimpleVibration {
        intensity_levels: 10,
    };
    let json = serde_json::to_string(&haptic).unwrap();
    let restored: HapticOutputCapability = serde_json::from_str(&json).unwrap();
    assert_eq!(haptic, restored);
}

#[test]
fn test_haptic_advanced_variant() {
    let haptic = HapticOutputCapability::Advanced {
        has_temperature: true,
        has_texture: true,
        actuators: 8,
    };
    let json = serde_json::to_string(&haptic).unwrap();
    let restored: HapticOutputCapability = serde_json::from_str(&json).unwrap();
    assert_eq!(haptic, restored);
}
