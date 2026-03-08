// SPDX-License-Identifier: AGPL-3.0-only
//! Sensory Capability System - Runtime I/O Discovery
//!
//! This module implements TRUE PRIMAL capability discovery by detecting
//! available human sensory I/O channels at runtime, rather than assuming
//! device types at compile time.
//!
//! # Philosophy
//!
//! - **Zero Hardcoding**: No device type assumptions (Desktop, Phone, etc.)
//! - **Runtime Discovery**: Detect what capabilities are actually available
//! - **Self-Knowledge Only**: Know only what can be detected
//! - **Live Evolution**: Adapt when capabilities change (VR headset plugged in)
//! - **Graceful Degradation**: Work with any capability subset
//! - **Future-Proof**: New capabilities (neural, smell) just add new types
//!
//! # Human Sensory Framework
//!
//! Humans have sensory I/O. Systems should discover available channels:
//!
//! - **Visual** (2D screens, 3D VR/AR)
//! - **Audio** (mono, stereo, spatial)
//! - **Haptic** (vibration, force feedback)
//! - **Pointer** (2D mouse, 3D VR controller)
//! - **Keyboard** (physical, virtual, chorded)
//! - **Touch** (single, multitouch, pressure)
//! - **Audio Input** (microphone, voice)
//! - **Gesture** (hand tracking, body, eyes)
//!
//! Future: Taste, Smell, Neural (BCI)

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// Output Capabilities (System → Human)
// ============================================================================

/// Visual output capability to human eyes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VisualOutputCapability {
    /// 2D flat display (monitor, phone, tablet, TV)
    TwoD {
        /// Resolution in pixels (width, height)
        resolution: (u32, u32),
        /// Refresh rate in Hz
        refresh_rate: u32,
        /// Color depth in bits per channel (typically 8, 10, or 12)
        color_depth: u8,
        /// Physical size in millimeters (for DPI calculation)
        size_mm: Option<(u32, u32)>,
    },
    /// 3D stereoscopic display (VR, AR, holographic)
    ThreeD {
        /// Resolution per eye in pixels (width, height)
        resolution_per_eye: (u32, u32),
        /// Field of view in degrees (horizontal, vertical)
        field_of_view: (f32, f32),
        /// Refresh rate in Hz
        refresh_rate: u32,
        /// Head/depth tracking available
        has_depth_tracking: bool,
        /// Hand tracking available
        has_hand_tracking: bool,
    },
}

impl VisualOutputCapability {
    /// Calculate total pixel count for capability comparison
    #[must_use]
    pub fn pixel_count(&self) -> u32 {
        match self {
            Self::TwoD { resolution, .. } => resolution.0 * resolution.1,
            Self::ThreeD {
                resolution_per_eye, ..
            } => resolution_per_eye.0 * resolution_per_eye.1 * 2,
        }
    }

    /// Calculate diagonal size in millimeters (if known)
    #[must_use]
    pub fn diagonal_mm(&self) -> Option<f32> {
        match self {
            Self::TwoD {
                size_mm: Some((w, h)),
                ..
            } => Some(((*w as f32).powi(2) + (*h as f32).powi(2)).sqrt()),
            _ => None,
        }
    }

    /// Determine if this is a small screen (< 6 inches diagonal)
    #[must_use]
    pub fn is_small_screen(&self) -> bool {
        self.diagonal_mm().is_some_and(|d| d < 150.0)
    }

    /// Determine if this is high resolution (>= 1080p)
    #[must_use]
    pub fn is_high_resolution(&self) -> bool {
        match self {
            Self::TwoD { resolution, .. } => resolution.0 >= 1920 && resolution.1 >= 1080,
            Self::ThreeD {
                resolution_per_eye, ..
            } => resolution_per_eye.0 >= 1920 && resolution_per_eye.1 >= 1080,
        }
    }
}

/// Audio output capability to human ears
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AudioOutputCapability {
    /// Mono audio (single channel)
    Mono {
        /// Sample rate in Hz (typically 44100 or 48000)
        sample_rate: u32,
        /// Bit depth (typically 16 or 24)
        bit_depth: u8,
    },
    /// Stereo audio (left/right channels)
    Stereo {
        /// Sample rate in Hz
        sample_rate: u32,
        /// Bit depth
        bit_depth: u8,
    },
    /// Spatial audio (surround sound, binaural, Atmos)
    Spatial {
        /// Number of channels (e.g., 6 for 5.1, 8 for 7.1)
        channels: u8,
        /// Sample rate in Hz
        sample_rate: u32,
        /// Head tracking for dynamic spatial audio
        has_head_tracking: bool,
    },
}

impl AudioOutputCapability {
    /// Determine if this is high quality audio (>= 48kHz, >= 16-bit)
    #[must_use]
    pub fn is_high_quality(&self) -> bool {
        match self {
            Self::Mono {
                sample_rate,
                bit_depth,
            }
            | Self::Stereo {
                sample_rate,
                bit_depth,
            } => *sample_rate >= 48000 && *bit_depth >= 16,
            Self::Spatial { sample_rate, .. } => *sample_rate >= 48000,
        }
    }
}

/// Haptic/tactile output capability to human touch
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HapticOutputCapability {
    /// Simple vibration (phone, game controller)
    SimpleVibration {
        /// Number of distinct intensity levels (1-255)
        intensity_levels: u8,
    },
    /// Force feedback (VR controllers, advanced game controllers)
    ForceFeedback {
        /// Number of force axes (typically 1-6)
        axes: u8,
        /// Precision of force control (8-bit, 16-bit, etc.)
        precision: u8,
    },
    /// Advanced haptics (texture simulation, temperature)
    Advanced {
        /// Temperature feedback available
        has_temperature: bool,
        /// Texture simulation available
        has_texture: bool,
        /// Number of independent actuators
        actuators: u8,
    },
}

/// Taste output capability (future: medical sensors, chemical delivery)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TasteOutputCapability {
    /// Basic taste types supported (sweet, sour, salty, bitter, umami)
    pub basic_tastes: Vec<String>,
}

/// Smell output capability (future: environmental alerts, aromatherapy)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmellOutputCapability {
    /// Number of independent scent channels
    pub scent_channels: u8,
}

/// Neural output capability (future: BCI, direct cortex stimulation)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NeuralOutputCapability {
    /// Signal types supported (visual cortex, audio cortex, etc.)
    pub signal_types: Vec<String>,
    /// Bandwidth in bits per second
    pub bandwidth: u32,
}

// ============================================================================
// Input Capabilities (Human → System)
// ============================================================================

/// Pointer input capability from human (spatial selection)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PointerInputCapability {
    /// 2D pointer (mouse, touchpad, trackpoint)
    TwoD {
        /// Precision in pixels per movement unit (higher = more precise)
        precision: f32,
        /// Scroll wheel available
        has_wheel: bool,
        /// Pressure sensitivity available
        has_pressure: bool,
        /// Number of buttons (typically 2-5)
        button_count: u8,
    },
    /// 3D pointer (VR controllers, leap motion, spatial mouse)
    ThreeD {
        /// Degrees of freedom (3DOF = position, 6DOF = position + rotation)
        degrees_of_freedom: u8,
        /// Precision in millimeters
        precision: f32,
        /// Haptic feedback available
        has_haptics: bool,
        /// Number of buttons
        button_count: u8,
    },
}

impl PointerInputCapability {
    /// Determine if this is a precision pointer (mouse-level)
    #[must_use]
    pub fn is_precision(&self) -> bool {
        match self {
            Self::TwoD { precision, .. } => *precision >= 1.0,
            Self::ThreeD { precision, .. } => *precision <= 5.0, // 5mm precision
        }
    }
}

/// Keyboard input capability from human (text/command entry)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KeyboardInputCapability {
    /// Physical keyboard (desktop, laptop)
    Physical {
        /// Layout (QWERTY, DVORAK, AZERTY, etc.)
        layout: String,
        /// Numeric keypad available
        has_numpad: bool,
        /// Number of modifier keys (Ctrl, Alt, Shift, etc.)
        modifier_keys: u8,
    },
    /// Virtual/on-screen keyboard (phone, tablet)
    Virtual {
        /// Layout (QWERTY, etc.)
        layout: String,
        /// Autocomplete support
        supports_autocomplete: bool,
        /// Swipe typing support
        supports_swipe: bool,
    },
    /// Chorded keyboard (stenotype, one-handed)
    Chorded {
        /// Number of keys
        key_count: u8,
    },
}

/// Touch input capability from human (direct manipulation)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TouchInputCapability {
    /// Maximum simultaneous touch points (1 = single, 10 = multitouch)
    pub max_touch_points: u8,
    /// Pressure sensitivity available
    pub supports_pressure: bool,
    /// Hover detection (pen proximity)
    pub supports_hover: bool,
    /// Screen size in millimeters (for gesture recognition)
    pub screen_size_mm: Option<(u32, u32)>,
}

impl TouchInputCapability {
    /// Determine if this is multitouch capable
    #[must_use]
    pub fn is_multitouch(&self) -> bool {
        self.max_touch_points > 1
    }
}

/// Audio input capability from human (voice, sound)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioInputCapability {
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of channels (1 = mono, 2 = stereo)
    pub channels: u8,
    /// Noise cancellation available
    pub has_noise_cancellation: bool,
    /// Wake word detection support
    pub supports_wake_word: bool,
}

/// Gesture input capability from human (body movement)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GestureInputCapability {
    /// Hand tracking (VR, leap motion, camera)
    Hand {
        /// Number of tracked points per hand (e.g., 21 for full finger tracking)
        tracking_points: u8,
        /// Tracking precision in millimeters
        precision: f32,
    },
    /// Full body tracking (Kinect, VR trackers)
    FullBody {
        /// Number of skeleton tracking points
        tracking_points: u8,
        /// Facial tracking available
        has_facial_tracking: bool,
    },
    /// Eye tracking (gaze detection)
    Eyes {
        /// Gaze tracking precision in degrees
        precision: f32,
        /// Blink detection support
        supports_blink_detection: bool,
    },
}

/// Neural input capability (future: BCI, EEG, EMG)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NeuralInputCapability {
    /// Signal types (EEG, EMG, fNIRS, etc.)
    pub signal_types: Vec<String>,
    /// Number of channels/electrodes
    pub channels: u8,
    /// Bandwidth in bits per second
    pub bandwidth: u32,
}

// ============================================================================
// Complete Sensory Capability Set
// ============================================================================

/// Complete set of sensory capabilities available to the system
///
/// This represents all detected human sensory I/O channels.
/// Discovery happens at runtime - no device type assumptions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SensoryCapabilities {
    // ========================================================================
    // Outputs (System → Human)
    // ========================================================================
    /// Visual output capabilities (displays)
    pub visual_outputs: Vec<VisualOutputCapability>,

    /// Audio output capabilities (speakers, headphones)
    pub audio_outputs: Vec<AudioOutputCapability>,

    /// Haptic output capabilities (vibration, force feedback)
    pub haptic_outputs: Vec<HapticOutputCapability>,

    // Future outputs
    /// Taste output capabilities (future: chemical delivery)
    pub taste_outputs: Vec<TasteOutputCapability>,

    /// Smell output capabilities (future: scent generators)
    pub smell_outputs: Vec<SmellOutputCapability>,

    /// Neural output capabilities (future: BCI, cortex stimulation)
    pub neural_outputs: Vec<NeuralOutputCapability>,

    // ========================================================================
    // Inputs (Human → System)
    // ========================================================================
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

    // Future inputs
    /// Neural input capabilities (future: BCI, thought input)
    pub neural_inputs: Vec<NeuralInputCapability>,
}

// ============================================================================
// UI Complexity Determination
// ============================================================================

/// UI complexity level derived from discovered capabilities
///
/// Instead of assuming device types, we determine appropriate UI
/// complexity based on what I/O channels are actually available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

impl SensoryCapabilities {
    /// Determine appropriate UI complexity from discovered capabilities
    ///
    /// This uses heuristics based on actual capabilities, not device types.
    ///
    /// # Examples
    ///
    /// - **Immersive**: 3D visual + spatial audio + haptics (VR)
    /// - **Rich**: High-res 2D + precision pointer + keyboard (desktop)
    /// - **Standard**: 2D display + basic input
    /// - **Simple**: Small screen or limited input (phone, watch)
    /// - **Minimal**: Very limited capabilities (audio-only, accessibility)
    #[must_use]
    pub fn determine_ui_complexity(&self) -> UIComplexity {
        // Check for 3D visual output (VR/AR)
        let has_3d_visual = self
            .visual_outputs
            .iter()
            .any(|v| matches!(v, VisualOutputCapability::ThreeD { .. }));

        // Check for spatial audio
        let has_spatial_audio = self
            .audio_outputs
            .iter()
            .any(|a| matches!(a, AudioOutputCapability::Spatial { .. }));

        // Check for haptics
        let has_haptics = !self.haptic_outputs.is_empty();

        // Immersive: VR/AR with spatial audio and haptics
        if has_3d_visual && has_spatial_audio && has_haptics {
            return UIComplexity::Immersive;
        }

        // Check for high-resolution 2D visual
        let has_high_res = self
            .visual_outputs
            .iter()
            .any(VisualOutputCapability::is_high_resolution);

        // Check for precision pointer (mouse-level)
        let has_precision_pointer = self
            .pointer_inputs
            .iter()
            .any(PointerInputCapability::is_precision);

        // Check for keyboard
        let has_keyboard = !self.keyboard_inputs.is_empty();

        // Rich: High-res display + precision input + keyboard
        if has_high_res && has_precision_pointer && has_keyboard {
            return UIComplexity::Rich;
        }

        // Standard: Decent display + keyboard + pointer
        if has_high_res || (has_keyboard && has_precision_pointer) {
            return UIComplexity::Standard;
        }

        // Check for touch input
        let has_touch = !self.touch_inputs.is_empty();

        // Check for small screen
        let has_small_screen = self
            .visual_outputs
            .iter()
            .any(VisualOutputCapability::is_small_screen);

        // Simple: Small screen or limited input
        if has_small_screen || (has_touch && !has_keyboard) {
            return UIComplexity::Simple;
        }

        // Minimal: Very limited capabilities (e.g., audio-only)
        if self.visual_outputs.is_empty() && !self.audio_outputs.is_empty() {
            return UIComplexity::Minimal;
        }

        // Default to Standard for unknown configurations
        UIComplexity::Standard
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
        let mut parts = Vec::new();

        // Describe visual outputs
        if !self.visual_outputs.is_empty() {
            let visual_desc = if self
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

        // Describe audio outputs
        if !self.audio_outputs.is_empty() {
            let audio_desc = if self
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

        // Describe inputs
        if !self.pointer_inputs.is_empty() {
            parts.push("pointer");
        }
        if !self.keyboard_inputs.is_empty() {
            parts.push("keyboard");
        }
        if !self.touch_inputs.is_empty() {
            parts.push("touch");
        }
        if !self.gesture_inputs.is_empty() {
            parts.push("gesture");
        }
        if !self.haptic_outputs.is_empty() {
            parts.push("haptics");
        }

        if parts.is_empty() {
            "no capabilities detected".to_string()
        } else {
            parts.join(" + ")
        }
    }
}

// ============================================================================
// Error Handling
// ============================================================================

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
            size_mm: Some((35, 35)), // ~2 inch watch
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
                size_mm: Some((70, 156)), // ~6.5 inch phone
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
