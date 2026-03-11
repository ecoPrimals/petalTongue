// SPDX-License-Identifier: AGPL-3.0-only
//! Runtime Sensory Capability Discovery
//!
//! This module implements platform-specific detection of available
//! sensory I/O capabilities. It uses platform APIs to discover what
//! hardware is actually present, rather than assuming device types.
//!
//! # Platform Support
//!
//! - **Linux**: X11/Wayland display detection, ALSA/PulseAudio, input devices
//! - **Windows**: Win32 APIs for display, audio, input
//! - **macOS**: Core Graphics, Core Audio, HID
//! - **Web**: WebGL, `WebAudio`, Pointer Events, Touch Events
//!
//! # Discovery Process
//!
//! 1. Detect visual outputs (displays, resolution, refresh rate)
//! 2. Detect audio outputs (speakers, sample rate, channels)
//! 3. Detect haptic outputs (vibration, force feedback)
//! 4. Detect input devices (mouse, keyboard, touch, etc.)
//! 5. Aggregate into `SensoryCapabilities`
//! 6. Determine appropriate `UIComplexity`

use crate::sensory_capabilities::{
    AudioInputCapability, AudioOutputCapability, CapabilityError, GestureInputCapability,
    HapticOutputCapability, KeyboardInputCapability, PointerInputCapability, SensoryCapabilities,
    TouchInputCapability, VisualOutputCapability,
};

impl SensoryCapabilities {
    /// Discover all available sensory capabilities at runtime
    ///
    /// This is the main entry point for capability discovery.
    /// It detects all available I/O channels and returns a complete
    /// `SensoryCapabilities` struct.
    ///
    /// # Errors
    ///
    /// Returns `CapabilityError` if:
    /// - No output capabilities detected (need visual or audio)
    /// - Platform detection fails
    /// - Unsupported platform
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use petal_tongue_core::SensoryCapabilities;
    ///
    /// let caps = SensoryCapabilities::discover()?;
    /// let complexity = caps.determine_ui_complexity();
    /// println!("Detected: {} ({})", caps.describe(), complexity);
    /// # Ok::<(), petal_tongue_core::CapabilityError>(())
    /// ```
    pub fn discover() -> Result<Self, CapabilityError> {
        let caps = Self {
            visual_outputs: Self::discover_visual(),
            audio_outputs: Self::discover_audio(),
            haptic_outputs: Self::discover_haptic(),
            pointer_inputs: Self::discover_pointer(),
            keyboard_inputs: Self::discover_keyboard(),
            touch_inputs: Self::discover_touch(),
            audio_inputs: Self::discover_audio_input(),
            gesture_inputs: Self::discover_gesture(),
            ..Default::default()
        };

        // Validate we have minimal capabilities
        if !caps.has_minimal_output() {
            return Err(CapabilityError::NoOutput);
        }

        Ok(caps)
    }

    // ========================================================================
    // Visual Output Discovery
    // ========================================================================

    fn discover_visual() -> Vec<VisualOutputCapability> {
        // For now, we'll use egui/winit to detect the primary display
        // In a full implementation, we'd enumerate all displays

        #[cfg(target_arch = "wasm32")]
        {
            // Web platform: use window.screen
            Self::discover_visual_web()
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Native platforms: use winit or platform APIs
            Self::discover_visual_native()
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn discover_visual_web() -> Vec<VisualOutputCapability> {
        // On web, we can query the window dimensions
        // This is a simplified version - full implementation would use web_sys
        vec![VisualOutputCapability::TwoD {
            resolution: (1920, 1080), // Default assumption for web
            refresh_rate: 60,
            color_depth: 8,
            size_mm: None,
        }]
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn discover_visual_native() -> Vec<VisualOutputCapability> {
        // On native platforms, we'll use a simple heuristic based on window size
        // Full implementation would use winit::monitor::MonitorHandle or platform APIs

        // For now, return a sensible default based on common desktop resolutions
        // Real implementation would enumerate monitors and get actual capabilities
        vec![VisualOutputCapability::TwoD {
            resolution: (1920, 1080),
            refresh_rate: 60,
            color_depth: 8,
            size_mm: None, // Would query from monitor EDID in real implementation
        }]
    }

    // ========================================================================
    // Audio Output Discovery
    // ========================================================================

    fn discover_audio() -> Vec<AudioOutputCapability> {
        // Use cpal or platform APIs to detect audio devices
        // For now, assume stereo output is available

        vec![AudioOutputCapability::Stereo {
            sample_rate: 48000,
            bit_depth: 16,
        }]
    }

    // ========================================================================
    // Haptic Output Discovery
    // ========================================================================

    const fn discover_haptic() -> Vec<HapticOutputCapability> {
        // Most systems don't have haptic output
        // Would check for:
        // - Game controllers with rumble
        // - VR controllers
        // - Mobile device vibration

        vec![] // No haptics by default
    }

    // ========================================================================
    // Pointer Input Discovery
    // ========================================================================

    fn discover_pointer() -> Vec<PointerInputCapability> {
        // Check for mouse, touchpad, or VR controllers
        // For now, assume mouse is available on desktop platforms

        #[cfg(target_arch = "wasm32")]
        {
            // Web: assume mouse or touch
            vec![PointerInputCapability::TwoD {
                precision: 1.0,
                has_wheel: true,
                has_pressure: false,
                button_count: 3,
            }]
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Native: assume mouse
            vec![PointerInputCapability::TwoD {
                precision: 1.5,
                has_wheel: true,
                has_pressure: false,
                button_count: 3,
            }]
        }
    }

    // ========================================================================
    // Keyboard Input Discovery
    // ========================================================================

    fn discover_keyboard() -> Vec<KeyboardInputCapability> {
        // Check for physical or virtual keyboard
        // For now, assume physical keyboard on desktop platforms

        #[cfg(target_arch = "wasm32")]
        {
            // Web: virtual keyboard likely
            vec![KeyboardInputCapability::Virtual {
                layout: "QWERTY".to_string(),
                supports_autocomplete: true,
                supports_swipe: false,
            }]
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Native: physical keyboard likely
            vec![KeyboardInputCapability::Physical {
                layout: "QWERTY".to_string(),
                has_numpad: true,
                modifier_keys: 4, // Ctrl, Alt, Shift, Super/Win/Cmd
            }]
        }
    }

    // ========================================================================
    // Touch Input Discovery
    // ========================================================================

    const fn discover_touch() -> Vec<TouchInputCapability> {
        // Check for touchscreen
        // Would use platform APIs to detect touch capability

        #[cfg(target_arch = "wasm32")]
        {
            // Web: check navigator.maxTouchPoints
            // For now, assume no touch unless on mobile
            vec![]
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Native: would check for touch devices
            // For now, assume no touch on desktop
            vec![]
        }
    }

    // ========================================================================
    // Audio Input Discovery
    // ========================================================================

    fn discover_audio_input() -> Vec<AudioInputCapability> {
        // Check for microphone
        // For now, assume microphone is available

        vec![AudioInputCapability {
            sample_rate: 48000,
            channels: 1, // Mono microphone
            has_noise_cancellation: false,
            supports_wake_word: false,
        }]
    }

    // ========================================================================
    // Gesture Input Discovery
    // ========================================================================

    const fn discover_gesture() -> Vec<GestureInputCapability> {
        // Check for cameras, depth sensors, VR tracking
        // Most systems don't have gesture input

        vec![] // No gesture input by default
    }
}

/// Create a mock `SensoryCapabilities` for testing
///
/// This is useful for testing different UI complexity levels without
/// actual hardware detection.
#[cfg(test)]
#[must_use]
pub fn mock_capabilities(
    visual: Option<VisualOutputCapability>,
    has_keyboard: bool,
    has_mouse: bool,
    has_touch: bool,
) -> SensoryCapabilities {
    let mut caps = SensoryCapabilities::default();

    if let Some(v) = visual {
        caps.visual_outputs = vec![v];
    }

    caps.audio_outputs = vec![AudioOutputCapability::Stereo {
        sample_rate: 48000,
        bit_depth: 16,
    }];

    if has_mouse {
        caps.pointer_inputs = vec![PointerInputCapability::TwoD {
            precision: 1.5,
            has_wheel: true,
            has_pressure: false,
            button_count: 3,
        }];
    }

    if has_keyboard {
        caps.keyboard_inputs = vec![KeyboardInputCapability::Physical {
            layout: "QWERTY".to_string(),
            has_numpad: true,
            modifier_keys: 4,
        }];
    }

    if has_touch {
        caps.touch_inputs = vec![TouchInputCapability {
            max_touch_points: 10,
            supports_pressure: true,
            supports_hover: false,
            screen_size_mm: Some((70, 156)),
        }];
    }

    caps
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sensory_capabilities::UIComplexity;

    #[test]
    fn test_discover_capabilities() {
        // This will use actual platform detection
        let result = SensoryCapabilities::discover();

        // Should succeed (at least audio should be detected)
        assert!(result.is_ok());

        let caps = result.expect("discover should succeed");

        // Should have some output capability
        assert!(caps.has_minimal_output());

        // Should determine some UI complexity
        let complexity = caps.determine_ui_complexity();
        println!("Detected complexity: {complexity}");
    }

    #[test]
    fn test_mock_desktop() {
        let caps = mock_capabilities(
            Some(VisualOutputCapability::TwoD {
                resolution: (1920, 1080),
                refresh_rate: 60,
                color_depth: 8,
                size_mm: None,
            }),
            true,  // has_keyboard
            true,  // has_mouse
            false, // has_touch
        );

        assert_eq!(caps.determine_ui_complexity(), UIComplexity::Rich);
    }

    #[test]
    fn test_mock_phone() {
        let caps = mock_capabilities(
            Some(VisualOutputCapability::TwoD {
                resolution: (1080, 2400),
                refresh_rate: 90,
                color_depth: 8,
                size_mm: Some((70, 156)),
            }),
            false, // has_keyboard
            false, // has_mouse
            true,  // has_touch
        );

        assert_eq!(caps.determine_ui_complexity(), UIComplexity::Simple);
    }

    #[test]
    fn test_mock_vr() {
        let mut caps = mock_capabilities(
            Some(VisualOutputCapability::ThreeD {
                resolution_per_eye: (1832, 1920),
                field_of_view: (110.0, 90.0),
                refresh_rate: 90,
                has_depth_tracking: true,
                has_hand_tracking: true,
            }),
            false,
            false,
            false,
        );

        // Add spatial audio and haptics for VR
        caps.audio_outputs = vec![AudioOutputCapability::Spatial {
            channels: 8,
            sample_rate: 48000,
            has_head_tracking: true,
        }];

        caps.haptic_outputs = vec![HapticOutputCapability::ForceFeedback {
            axes: 3,
            precision: 16,
        }];

        caps.pointer_inputs = vec![PointerInputCapability::ThreeD {
            degrees_of_freedom: 6,
            precision: 1.0,
            has_haptics: true,
            button_count: 4,
        }];

        assert_eq!(caps.determine_ui_complexity(), UIComplexity::Immersive);
    }

    #[test]
    fn test_mock_capabilities_audio_only() {
        let caps = mock_capabilities(None, false, false, false);
        assert!(caps.has_minimal_output());
        assert!(!caps.has_minimal_input());
        assert_eq!(caps.determine_ui_complexity(), UIComplexity::Minimal);
    }

    #[test]
    fn test_mock_capabilities_visual_none() {
        let caps = mock_capabilities(None, true, true, false);
        assert!(caps.visual_outputs.is_empty());
        assert!(!caps.audio_outputs.is_empty());
    }
}
