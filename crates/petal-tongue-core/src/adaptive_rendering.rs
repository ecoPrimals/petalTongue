// SPDX-License-Identifier: AGPL-3.0-only
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
    #[must_use]
    pub fn detect() -> Self {
        let screen_size = Self::detect_screen_size();
        let has_mouse = Self::has_mouse();
        let has_touch = Self::has_touch();

        let device_type = DeviceType::detect(Some(screen_size), has_mouse, has_touch);
        let ui_complexity = device_type.recommended_complexity();

        let mut modalities = Vec::new();
        let mut input_methods = Vec::new();

        // Visual modality
        let (w, h) = screen_size;
        modalities.push(RenderingModality::Visual2D {
            resolution: (w, h),
            color: true,
        });

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
            DeviceType::Tablet | DeviceType::Phone | DeviceType::Unknown => PerformanceTier::Medium,
            DeviceType::Watch | DeviceType::CLI => PerformanceTier::Low,
        };

        Self {
            device_type,
            modalities,
            screen_size: Some(screen_size),
            input_methods,
            performance_tier,
            ui_complexity,
        }
    }

    /// Detect screen size from environment
    ///
    /// Tries, in order:
    /// 1. Linux framebuffer: `/sys/class/graphics/fb0/virtual_size`
    /// 2. Terminal dimensions (when no graphical display)
    /// 3. DISPLAY/WAYLAND_DISPLAY presence → sensible desktop defaults
    /// 4. Fallback: (1280, 720) when detection fails
    fn detect_screen_size() -> (f32, f32) {
        use terminal_size::{Height, Width};

        // 1. Linux: Read framebuffer virtual size
        #[cfg(target_os = "linux")]
        if let Some((w, h)) = Self::read_fb0_virtual_size() {
            return (w, h);
        }

        // 2. Graphical environment detected → use defaults (actual size from winit at runtime)
        let has_graphical = std::env::var("DISPLAY")
            .map(|d| !d.is_empty())
            .unwrap_or(false)
            || std::env::var("WAYLAND_DISPLAY")
                .map(|w| !w.is_empty())
                .unwrap_or(false);

        if has_graphical {
            return (1400.0, 900.0);
        }

        // 3. CLI/terminal: Use terminal dimensions (convert cols/rows to pixel estimate)
        if let Some((Width(cols), Height(rows))) = terminal_size::terminal_size() {
            let w = f32::from(cols) * 8.0;
            let h = f32::from(rows) * 16.0;
            return (w, h);
        }

        // 4. Fallback when all detection fails
        (1280.0, 720.0)
    }

    /// Read Linux framebuffer virtual size from sysfs
    #[cfg(target_os = "linux")]
    fn read_fb0_virtual_size() -> Option<(f32, f32)> {
        let path = "/sys/class/graphics/fb0/virtual_size";
        let s = std::fs::read_to_string(path).ok()?;
        let s = s.trim();
        let mut parts = s.split(',');
        let w: f32 = parts.next()?.trim().parse().ok()?;
        let h: f32 = parts.next()?.trim().parse().ok()?;
        Some((w, h))
    }

    /// Check if mouse is available
    fn has_mouse() -> bool {
        // Heuristic: If we have a display server, assume mouse
        std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok()
    }

    /// Check if touch is available
    ///
    /// On Linux: checks for touch input devices and graphical session type.
    fn has_touch() -> bool {
        // 1. Linux: Check for touch devices in /dev/input/
        #[cfg(target_os = "linux")]
        if Self::has_touch_input_devices() {
            return true;
        }

        // 2. Wayland/touch-friendly session (often indicates touch-capable device)
        if std::env::var("WAYLAND_DISPLAY")
            .map(|w| !w.is_empty())
            .unwrap_or(false)
        {
            // Wayland is common on touch devices (tablets, convertibles)
            // Conservative: don't assume touch from Wayland alone
        }

        if std::env::var("XDG_SESSION_TYPE")
            .map(|s| s.to_lowercase() == "wayland")
            .unwrap_or(false)
        {
            // Wayland session - could be touch, but we already checked devices
        }

        false
    }

    /// Check for touch input devices on Linux
    #[cfg(target_os = "linux")]
    fn has_touch_input_devices() -> bool {
        let Ok(entries) = std::fs::read_dir("/sys/class/input") else {
            return false;
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("event") {
                let device_name = entry.path().join("device/name");
                if let Ok(name_content) = std::fs::read_to_string(device_name) {
                    let name_lower = name_content.to_lowercase();
                    if name_lower.contains("touch")
                        || name_lower.contains("touchscreen")
                        || name_lower.contains("wacom")
                        || name_lower.contains("digitizer")
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check if keyboard is available
    const fn has_keyboard() -> bool {
        // Assume keyboard is always available if we have input
        true
    }

    /// Check if this device supports a specific modality
    pub fn supports_modality(&self, check: impl Fn(&RenderingModality) -> bool) -> bool {
        self.modalities.iter().any(check)
    }

    /// Check if this device supports visual rendering
    #[must_use]
    pub fn has_visual(&self) -> bool {
        self.supports_modality(|m| matches!(m, RenderingModality::Visual2D { .. }))
    }

    /// Check if this device supports audio
    #[must_use]
    pub fn has_audio(&self) -> bool {
        self.supports_modality(|m| matches!(m, RenderingModality::Audio { .. }))
    }

    /// Check if this device supports haptics
    #[must_use]
    pub fn has_haptic(&self) -> bool {
        self.supports_modality(|m| matches!(m, RenderingModality::Haptic { .. }))
    }

    /// Get visual resolution, if available
    #[must_use]
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

        assert!(!caps.modalities.is_empty());

        assert!(matches!(
            caps.ui_complexity,
            UIComplexity::Full
                | UIComplexity::Simplified
                | UIComplexity::Minimal
                | UIComplexity::Essential
        ));
    }

    #[test]
    fn test_device_type_tablet() {
        let tablet = DeviceType::detect(Some((1024.0, 768.0)), false, true);
        assert_eq!(tablet, DeviceType::Tablet);
    }

    #[test]
    fn test_device_type_tv() {
        let tv = DeviceType::detect(Some((1920.0, 1200.0)), false, false);
        assert_eq!(tv, DeviceType::TV);
    }

    #[test]
    fn test_device_type_unknown_no_mouse() {
        let unknown = DeviceType::detect(Some((1920.0, 1080.0)), false, false);
        assert_eq!(unknown, DeviceType::Unknown);
    }

    #[test]
    fn test_device_type_phone_touch_no_mouse() {
        let phone = DeviceType::detect(Some((375.0, 667.0)), false, true);
        assert_eq!(phone, DeviceType::Phone);
    }

    #[test]
    fn test_device_type_watch_small_screen() {
        let watch = DeviceType::detect(Some((184.0, 224.0)), false, true);
        assert_eq!(watch, DeviceType::Watch);
    }

    #[test]
    fn test_device_type_display() {
        assert_eq!(format!("{}", DeviceType::Desktop), "Desktop");
        assert_eq!(format!("{}", DeviceType::Phone), "Phone");
        assert_eq!(format!("{}", DeviceType::Watch), "Watch");
        assert_eq!(format!("{}", DeviceType::Tablet), "Tablet");
        assert_eq!(format!("{}", DeviceType::TV), "TV");
        assert_eq!(format!("{}", DeviceType::CLI), "CLI");
        assert_eq!(format!("{}", DeviceType::Unknown), "Unknown");
    }

    #[test]
    fn test_ui_complexity_tablet() {
        assert_eq!(
            DeviceType::Tablet.recommended_complexity(),
            UIComplexity::Simplified
        );
    }

    #[test]
    fn test_ui_complexity_tv() {
        assert_eq!(DeviceType::TV.recommended_complexity(), UIComplexity::Full);
    }

    #[test]
    fn test_ui_complexity_unknown() {
        assert_eq!(
            DeviceType::Unknown.recommended_complexity(),
            UIComplexity::Minimal
        );
    }

    #[test]
    fn test_rendering_capabilities_has_visual() {
        let caps = RenderingCapabilities::detect();
        assert!(caps.has_visual());
    }

    #[test]
    fn test_rendering_capabilities_visual_resolution() {
        let caps = RenderingCapabilities::detect();
        let res = caps.visual_resolution();
        assert!(res.is_some());
        let (w, h) = res.unwrap();
        assert!(w > 0.0 && h > 0.0);
    }

    #[test]
    fn test_rendering_capabilities_supports_modality() {
        let caps = RenderingCapabilities::detect();
        assert!(caps.supports_modality(|m| matches!(m, RenderingModality::Visual2D { .. })));
    }

    #[test]
    fn test_rendering_capabilities_no_audio_by_default() {
        let caps = RenderingCapabilities::detect();
        assert!(!caps.has_audio());
    }

    #[test]
    fn test_rendering_capabilities_no_haptic_by_default() {
        let caps = RenderingCapabilities::detect();
        assert!(!caps.has_haptic());
    }

    #[test]
    fn test_rendering_modality_visual2d() {
        let caps = RenderingCapabilities {
            device_type: DeviceType::Desktop,
            modalities: vec![RenderingModality::Visual2D {
                resolution: (800.0, 600.0),
                color: true,
            }],
            screen_size: Some((800.0, 600.0)),
            input_methods: vec![],
            performance_tier: PerformanceTier::High,
            ui_complexity: UIComplexity::Full,
        };
        assert!(caps.has_visual());
        assert_eq!(caps.visual_resolution(), Some((800.0, 600.0)));
    }

    #[test]
    fn test_adaptive_renderer_trait_default_priority() {
        struct TestRenderer;
        impl AdaptiveRenderer for TestRenderer {
            fn supports(&self, _caps: &RenderingCapabilities) -> bool {
                true
            }
        }
        let r = TestRenderer;
        let caps = RenderingCapabilities::detect();
        assert!(r.supports(&caps));
        assert_eq!(r.priority(), 0);
    }
}
