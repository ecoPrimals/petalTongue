// SPDX-License-Identifier: AGPL-3.0-or-later
//! Types for adaptive rendering (device class, modalities, capabilities).

use serde::{Deserialize, Serialize};
use std::fmt;

/// Device type (auto-detected or explicit)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceType {
    /// Desktop computer (laptop, workstation, etc.)
    Desktop,
    /// Phone (iOS, Android)
    Phone,
    /// Watch or fitness tracker
    Watch,
    /// Tablet (iPad, Android tablet)
    Tablet,
    /// TV or large display
    TV,
    /// Command-line interface (SSH, terminal)
    CLI,
    /// Unknown/custom device
    Unknown,
}

impl DeviceType {
    /// Detect device type from display size and input methods
    #[must_use]
    pub fn detect(screen_size: Option<(f32, f32)>, has_mouse: bool, has_touch: bool) -> Self {
        match screen_size {
            Some((w, h)) => {
                let min_dim = w.min(h);
                let max_dim = w.max(h);

                // Watch: Very small screen
                if max_dim < 500.0 {
                    Self::Watch
                }
                // Phone: Small screen, usually portrait
                else if max_dim < 1000.0 && has_touch && !has_mouse {
                    Self::Phone
                }
                // Tablet: Medium screen, touch
                else if max_dim < 1400.0 && has_touch {
                    Self::Tablet
                }
                // TV: Very large screen
                else if min_dim > 1080.0 {
                    Self::TV
                }
                // Desktop: Large screen with mouse
                else if has_mouse {
                    Self::Desktop
                } else {
                    Self::Unknown
                }
            }
            None => Self::CLI, // No screen = CLI
        }
    }

    /// Get recommended UI complexity for this device type
    #[must_use]
    pub const fn recommended_complexity(&self) -> UIComplexity {
        match self {
            Self::Desktop | Self::TV => UIComplexity::Full,
            Self::Tablet => UIComplexity::Simplified,
            Self::Phone | Self::Unknown => UIComplexity::Minimal,
            Self::Watch | Self::CLI => UIComplexity::Essential,
        }
    }
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Desktop => write!(f, "Desktop"),
            Self::Phone => write!(f, "Phone"),
            Self::Watch => write!(f, "Watch"),
            Self::Tablet => write!(f, "Tablet"),
            Self::TV => write!(f, "TV"),
            Self::CLI => write!(f, "CLI"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// UI complexity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UIComplexity {
    /// Full UI (all features, multiple panels)
    Full,
    /// Simplified UI (key features, fewer panels)
    Simplified,
    /// Minimal UI (core features only, single view)
    Minimal,
    /// Essential UI (critical info only, glance-able)
    Essential,
}

/// Input method supported by device
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputMethod {
    /// Mouse/trackpad
    Mouse,
    /// Touch screen
    Touch,
    /// Keyboard
    Keyboard,
    /// Voice commands
    Voice,
    /// Digital crown (watch)
    Crown,
    /// Haptic feedback
    Haptic,
    /// Gamepad/controller
    Gamepad,
}

/// Performance tier of device
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PerformanceTier {
    /// Low-end device (watch, old phone)
    Low,
    /// Mid-range device (recent phone, tablet)
    Medium,
    /// High-end device (desktop, gaming laptop)
    High,
}

/// Rendering modality (visual, audio, haptic, etc.)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RenderingModality {
    /// 2D visual rendering
    Visual2D {
        /// Display resolution (width, height)
        resolution: (f32, f32),
        /// Color depth (true for full color, false for monochrome)
        color: bool,
    },
    /// 3D visual rendering
    Visual3D {
        /// Has GPU acceleration
        gpu: bool,
    },
    /// Audio rendering
    Audio {
        /// Sample rate (Hz)
        sample_rate: u32,
        /// Number of channels (1=mono, 2=stereo, etc.)
        channels: u8,
    },
    /// Haptic feedback
    Haptic {
        /// Precision level
        precision: HapticPrecision,
    },
    /// Command-line interface
    CLI {
        /// Supports color (ANSI escape codes)
        color: bool,
        /// Terminal size (columns, rows)
        size: (u16, u16),
    },
}

/// Haptic precision level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HapticPrecision {
    /// Basic vibration (on/off)
    Basic,
    /// Variable intensity
    Variable,
    /// High-precision taptics (Apple Watch, iPhone)
    Precise,
}

/// Complete rendering capabilities of a device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderingCapabilities {
    /// Device type
    pub device_type: DeviceType,

    /// Available modalities
    pub modalities: Vec<RenderingModality>,

    /// Display size (if visual)
    pub screen_size: Option<(f32, f32)>,

    /// Input methods available
    pub input_methods: Vec<InputMethod>,

    /// Performance tier
    pub performance_tier: PerformanceTier,

    /// Recommended UI complexity
    pub ui_complexity: UIComplexity,
}
