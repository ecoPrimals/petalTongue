// SPDX-License-Identifier: AGPL-3.0-only
//! Sensory capability type definitions - I/O channel descriptors.

use serde::{Deserialize, Serialize};

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
    pub const fn pixel_count(&self) -> u32 {
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
            } =>
            {
                #[expect(clippy::cast_possible_truncation)]
                Some(f64::from(*w).hypot(f64::from(*h)) as f32)
            }
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
    pub const fn is_high_resolution(&self) -> bool {
        match self {
            Self::TwoD { resolution, .. } => resolution.0 >= 1920 && resolution.1 >= 1080,
            Self::ThreeD {
                resolution_per_eye, ..
            } => resolution_per_eye.0 >= 1920 && resolution_per_eye.1 >= 1080,
        }
    }
}

/// Audio output capability to human ears
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    pub const fn is_high_quality(&self) -> bool {
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TasteOutputCapability {
    /// Basic taste types supported (sweet, sour, salty, bitter, umami)
    pub basic_tastes: Vec<String>,
}

/// Smell output capability (future: environmental alerts, aromatherapy)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SmellOutputCapability {
    /// Number of independent scent channels
    pub scent_channels: u8,
}

/// Neural output capability (future: BCI, direct cortex stimulation)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NeuralOutputCapability {
    /// Signal types supported (visual cortex, audio cortex, etc.)
    pub signal_types: Vec<String>,
    /// Bandwidth in bits per second
    pub bandwidth: u32,
}

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
            Self::ThreeD { precision, .. } => *precision <= 5.0,
        }
    }
}

/// Keyboard input capability from human (text/command entry)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    pub const fn is_multitouch(&self) -> bool {
        self.max_touch_points > 1
    }
}

/// Audio input capability from human (voice, sound)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NeuralInputCapability {
    /// Signal types (EEG, EMG, fNIRS, etc.)
    pub signal_types: Vec<String>,
    /// Number of channels/electrodes
    pub channels: u8,
    /// Bandwidth in bits per second
    pub bandwidth: u32,
}
