//! Adaptive rendering system for multi-device support
//!
//! This module enables petalTongue to adapt its UI based on device capabilities:
//! - Desktop: Full UI (1400x900, mouse + keyboard)
//! - Phone: Touch-optimized UI (320x568 to 428x926, touch)
//! - Watch: Glance UI (184x224 to 368x448, touch + crown)
//! - CLI: Text-based UI (80x24 terminal)
//!
//! # Philosophy
//!
//! **The UI should adapt to the device, not vice versa.**
//!
//! Don't force users into a single interaction model. Discover what the device
//! can do and render accordingly.

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
    /// Detect device type from screen size and input methods
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
    pub fn recommended_complexity(&self) -> UIComplexity {
        match self {
            Self::Desktop | Self::TV => UIComplexity::Full,
            Self::Tablet => UIComplexity::Simplified,
            Self::Phone => UIComplexity::Minimal,
            Self::Watch | Self::CLI => UIComplexity::Essential,
            Self::Unknown => UIComplexity::Minimal,
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
        /// Screen resolution (width, height)
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

    /// Screen size (if visual)
    pub screen_size: Option<(f32, f32)>,

    /// Input methods available
    pub input_methods: Vec<InputMethod>,

    /// Performance tier
    pub performance_tier: PerformanceTier,

    /// Recommended UI complexity
    pub ui_complexity: UIComplexity,
}

impl RenderingCapabilities {
    /// Detect capabilities from environment
    pub fn detect() -> Self {
        let screen_size = Self::detect_screen_size();
        let has_mouse = Self::has_mouse();
        let has_touch = Self::has_touch();

        let device_type = DeviceType::detect(screen_size, has_mouse, has_touch);
        let ui_complexity = device_type.recommended_complexity();

        let mut modalities = Vec::new();
        let mut input_methods = Vec::new();

        // Visual modality
        if let Some((w, h)) = screen_size {
            modalities.push(RenderingModality::Visual2D {
                resolution: (w, h),
                color: true, // Assume color for now
            });
        } else {
            // No screen = CLI mode
            modalities.push(RenderingModality::CLI {
                color: std::env::var("TERM").map(|t| t.contains("color")).unwrap_or(false),
                size: (80, 24), // Default terminal size
            });
        }

        // Input methods
        if has_mouse {
            input_methods.push(InputMethod::Mouse);
        }
        if has_touch {
            input_methods.push(InputMethod::Touch);
        }
        if Self::has_keyboard() {
            input_methods.push(InputMethod::Keyboard);
        }

        // Performance tier (basic heuristic)
        let performance_tier = match device_type {
            DeviceType::Desktop | DeviceType::TV => PerformanceTier::High,
            DeviceType::Tablet | DeviceType::Phone => PerformanceTier::Medium,
            DeviceType::Watch | DeviceType::CLI => PerformanceTier::Low,
            DeviceType::Unknown => PerformanceTier::Medium,
        };

        Self {
            device_type,
            modalities,
            screen_size,
            input_methods,
            performance_tier,
            ui_complexity,
        }
    }

    /// Detect screen size from environment
    fn detect_screen_size() -> Option<(f32, f32)> {
        // Try to get from DISPLAY environment variable
        if let Ok(display) = std::env::var("DISPLAY") {
            if !display.is_empty() {
                // Assume desktop resolution for now
                // TODO: Query actual screen size via winit or X11
                return Some((1400.0, 900.0));
            }
        }

        // Try Wayland
        if let Ok(wayland) = std::env::var("WAYLAND_DISPLAY") {
            if !wayland.is_empty() {
                return Some((1400.0, 900.0));
            }
        }

        // No display detected
        None
    }

    /// Check if mouse is available
    fn has_mouse() -> bool {
        // Heuristic: If we have a display server, assume mouse
        std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok()
    }

    /// Check if touch is available
    fn has_touch() -> bool {
        // TODO: Proper touch detection
        // For now, assume no touch unless it's a mobile device
        false
    }

    /// Check if keyboard is available
    fn has_keyboard() -> bool {
        // Assume keyboard is always available if we have input
        true
    }

    /// Check if this device supports a specific modality
    pub fn supports_modality(&self, check: impl Fn(&RenderingModality) -> bool) -> bool {
        self.modalities.iter().any(check)
    }

    /// Check if this device supports visual rendering
    pub fn has_visual(&self) -> bool {
        self.supports_modality(|m| matches!(m, RenderingModality::Visual2D { .. }))
    }

    /// Check if this device supports audio
    pub fn has_audio(&self) -> bool {
        self.supports_modality(|m| matches!(m, RenderingModality::Audio { .. }))
    }

    /// Check if this device supports haptics
    pub fn has_haptic(&self) -> bool {
        self.supports_modality(|m| matches!(m, RenderingModality::Haptic { .. }))
    }

    /// Get visual resolution, if available
    pub fn visual_resolution(&self) -> Option<(f32, f32)> {
        for modality in &self.modalities {
            if let RenderingModality::Visual2D { resolution, .. } = modality {
                return Some(*resolution);
            }
        }
        None
    }
}

/// Trait for adaptive rendering
pub trait AdaptiveRenderer {
    /// Check if this renderer can handle the given capabilities
    fn supports(&self, caps: &RenderingCapabilities) -> bool;

    /// Get renderer priority (higher = preferred)
    fn priority(&self) -> i32 {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_type_detection() {
        // Desktop
        let desktop = DeviceType::detect(Some((1920.0, 1080.0)), true, false);
        assert_eq!(desktop, DeviceType::Desktop);

        // Phone
        let phone = DeviceType::detect(Some((428.0, 926.0)), false, true);
        assert_eq!(phone, DeviceType::Phone);

        // Watch
        let watch = DeviceType::detect(Some((368.0, 448.0)), false, true);
        assert_eq!(watch, DeviceType::Watch);

        // CLI
        let cli = DeviceType::detect(None, false, false);
        assert_eq!(cli, DeviceType::CLI);
    }

    #[test]
    fn test_ui_complexity() {
        assert_eq!(
            DeviceType::Desktop.recommended_complexity(),
            UIComplexity::Full
        );
        assert_eq!(
            DeviceType::Phone.recommended_complexity(),
            UIComplexity::Minimal
        );
        assert_eq!(
            DeviceType::Watch.recommended_complexity(),
            UIComplexity::Essential
        );
    }

    #[test]
    fn test_rendering_capabilities_detection() {
        let caps = RenderingCapabilities::detect();
        
        // Should have at least one modality
        assert!(!caps.modalities.is_empty());

        // UI complexity should be set
        assert!(matches!(
            caps.ui_complexity,
            UIComplexity::Full
                | UIComplexity::Simplified
                | UIComplexity::Minimal
                | UIComplexity::Essential
        ));
    }
}

