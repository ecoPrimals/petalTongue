// SPDX-License-Identifier: AGPL-3.0-or-later
// Accessibility Settings and Color Schemes
//
// Universal accessibility for ALL users - color-blind, blind, deaf, motor disabilities
// Customizable color schemes, fonts, audio settings

use egui::Color32;
use serde::{Deserialize, Serialize};

/// Accessibility settings for the UI
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccessibilitySettings {
    /// Color scheme
    pub color_scheme: ColorScheme,
    /// Font size multiplier
    pub font_size: FontSize,
    /// Audio sonification enabled
    pub audio_enabled: bool,
    /// Audio volume (0.0 to 1.0)
    pub audio_volume: f32,
    /// Audio narration enabled (text-to-speech)
    pub narration_enabled: bool,
    /// Keyboard-only mode (optimized for keyboard nav)
    pub keyboard_only: bool,
    /// Screen reader mode (additional announcements)
    pub screen_reader_mode: bool,
    /// High contrast mode
    pub high_contrast: bool,
    /// Reduced motion (for vestibular disorders)
    pub reduced_motion: bool,
}

impl Default for AccessibilitySettings {
    fn default() -> Self {
        Self {
            color_scheme: ColorScheme::Default,
            font_size: FontSize::Medium,
            audio_enabled: true,
            audio_volume: 0.8,
            narration_enabled: false,
            keyboard_only: false,
            screen_reader_mode: false,
            high_contrast: false,
            reduced_motion: false,
        }
    }
}

/// Color scheme options
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorScheme {
    /// Default color scheme
    Default,
    /// High contrast (WCAG AAA compliant)
    HighContrast,
    /// Deuteranopia-friendly (red-green color blind, most common)
    Deuteranopia,
    /// Protanopia-friendly (red-blind)
    Protanopia,
    /// Tritanopia-friendly (blue-yellow color blind)
    Tritanopia,
    /// Dark mode
    Dark,
    /// Light mode
    Light,
}

impl ColorScheme {
    /// Get the name of the color scheme
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::HighContrast => "High Contrast",
            Self::Deuteranopia => "Deuteranopia (Red-Green)",
            Self::Protanopia => "Protanopia (Red-Blind)",
            Self::Tritanopia => "Tritanopia (Blue-Yellow)",
            Self::Dark => "Dark Mode",
            Self::Light => "Light Mode",
        }
    }

    /// Get all available color schemes
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Default,
            Self::HighContrast,
            Self::Deuteranopia,
            Self::Protanopia,
            Self::Tritanopia,
            Self::Dark,
            Self::Light,
        ]
    }
}

/// Font size options
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FontSize {
    /// Small (0.85x)
    Small,
    /// Medium (1.0x) - default
    Medium,
    /// Large (1.3x)
    Large,
    /// Extra Large (1.6x)
    ExtraLarge,
}

impl FontSize {
    /// Get the multiplier for this font size
    #[must_use]
    pub const fn multiplier(self) -> f32 {
        match self {
            Self::Small => 0.85,
            Self::Medium => 1.0,
            Self::Large => 1.3,
            Self::ExtraLarge => 1.6,
        }
    }

    /// Get the name of this font size
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Small => "Small",
            Self::Medium => "Medium",
            Self::Large => "Large",
            Self::ExtraLarge => "Extra Large",
        }
    }

    /// Increase font size (clamped at `ExtraLarge`).
    #[must_use]
    pub const fn increase(self) -> Self {
        match self {
            Self::Small => Self::Medium,
            Self::Medium => Self::Large,
            Self::Large | Self::ExtraLarge => Self::ExtraLarge,
        }
    }

    /// Decrease font size (clamped at Small).
    #[must_use]
    pub const fn decrease(self) -> Self {
        match self {
            Self::Small => Self::Small,
            Self::Medium => Self::Small,
            Self::Large => Self::Medium,
            Self::ExtraLarge => Self::Large,
        }
    }
}

/// Color palette for a specific scheme
#[derive(Clone, Debug)]
pub struct ColorPalette {
    /// Healthy/good state
    pub healthy: Color32,
    /// Warning state
    pub warning: Color32,
    /// Error/critical state
    pub error: Color32,
    /// Background
    pub background: Color32,
    /// Background alternate (for contrast)
    pub background_alt: Color32,
    /// Primary text
    pub text: Color32,
    /// Secondary text (dim)
    pub text_dim: Color32,
    /// Accent color
    pub accent: Color32,
    /// Border color
    pub border: Color32,
}

impl ColorPalette {
    /// Get the color palette for a given scheme
    #[must_use]
    pub const fn from_scheme(scheme: ColorScheme) -> Self {
        match scheme {
            ColorScheme::Default => Self::default(),
            ColorScheme::HighContrast => Self::high_contrast(),
            ColorScheme::Deuteranopia => Self::deuteranopia(),
            ColorScheme::Protanopia => Self::protanopia(),
            ColorScheme::Tritanopia => Self::tritanopia(),
            ColorScheme::Dark => Self::dark(),
            ColorScheme::Light => Self::light(),
        }
    }

    /// Default color palette
    const fn default() -> Self {
        Self {
            healthy: Color32::from_rgb(50, 200, 100),  // Green
            warning: Color32::from_rgb(255, 180, 50),  // Orange
            error: Color32::from_rgb(220, 50, 50),     // Red
            background: Color32::from_rgb(25, 25, 30), // Dark gray
            background_alt: Color32::from_rgb(35, 35, 40),
            text: Color32::from_rgb(240, 240, 245), // Light gray
            text_dim: Color32::from_rgb(160, 160, 170),
            accent: Color32::from_rgb(100, 150, 255), // Blue
            border: Color32::from_rgb(60, 60, 70),
        }
    }

    /// High contrast palette (WCAG AAA)
    const fn high_contrast() -> Self {
        Self {
            healthy: Color32::from_rgb(0, 255, 100), // Bright green
            warning: Color32::from_rgb(255, 200, 0), // Bright yellow
            error: Color32::from_rgb(255, 0, 50),    // Bright red
            background: Color32::from_rgb(0, 0, 0),  // Pure black
            background_alt: Color32::from_rgb(20, 20, 20),
            text: Color32::from_rgb(255, 255, 255), // Pure white
            text_dim: Color32::from_rgb(200, 200, 200),
            accent: Color32::from_rgb(100, 200, 255), // Bright cyan
            border: Color32::from_rgb(100, 100, 100),
        }
    }

    /// Deuteranopia-friendly (red-green color blind - most common)
    /// Uses blue/yellow instead of red/green
    const fn deuteranopia() -> Self {
        Self {
            healthy: Color32::from_rgb(50, 150, 255), // Blue (good)
            warning: Color32::from_rgb(255, 200, 50), // Yellow (warning)
            error: Color32::from_rgb(255, 100, 200),  // Magenta (error)
            background: Color32::from_rgb(25, 25, 30),
            background_alt: Color32::from_rgb(35, 35, 40),
            text: Color32::from_rgb(240, 240, 245),
            text_dim: Color32::from_rgb(160, 160, 170),
            accent: Color32::from_rgb(100, 200, 255),
            border: Color32::from_rgb(60, 60, 70),
        }
    }

    /// Protanopia-friendly (red-blind)
    const fn protanopia() -> Self {
        Self {
            healthy: Color32::from_rgb(50, 180, 255),  // Cyan
            warning: Color32::from_rgb(255, 220, 100), // Yellow
            error: Color32::from_rgb(200, 100, 255),   // Purple
            background: Color32::from_rgb(25, 25, 30),
            background_alt: Color32::from_rgb(35, 35, 40),
            text: Color32::from_rgb(240, 240, 245),
            text_dim: Color32::from_rgb(160, 160, 170),
            accent: Color32::from_rgb(80, 200, 255),
            border: Color32::from_rgb(60, 60, 70),
        }
    }

    /// Tritanopia-friendly (blue-yellow color blind)
    const fn tritanopia() -> Self {
        Self {
            healthy: Color32::from_rgb(50, 255, 150),  // Green
            warning: Color32::from_rgb(255, 150, 150), // Pink
            error: Color32::from_rgb(255, 50, 100),    // Red-pink
            background: Color32::from_rgb(25, 25, 30),
            background_alt: Color32::from_rgb(35, 35, 40),
            text: Color32::from_rgb(240, 240, 245),
            text_dim: Color32::from_rgb(160, 160, 170),
            accent: Color32::from_rgb(100, 255, 200),
            border: Color32::from_rgb(60, 60, 70),
        }
    }

    /// Dark mode palette
    const fn dark() -> Self {
        Self::default() // Already dark by default
    }

    /// Light mode palette
    const fn light() -> Self {
        Self {
            healthy: Color32::from_rgb(0, 150, 50),       // Dark green
            warning: Color32::from_rgb(200, 140, 0),      // Dark orange
            error: Color32::from_rgb(180, 0, 30),         // Dark red
            background: Color32::from_rgb(245, 245, 250), // Light gray
            background_alt: Color32::from_rgb(235, 235, 240),
            text: Color32::from_rgb(20, 20, 25), // Dark gray
            text_dim: Color32::from_rgb(100, 100, 110),
            accent: Color32::from_rgb(50, 100, 200), // Blue
            border: Color32::from_rgb(200, 200, 210),
        }
    }
}

/// Live data indicator state
#[derive(Clone, Debug)]
pub struct LiveIndicator {
    /// Is data currently live/streaming?
    pub is_live: bool,
    /// Last update timestamp (seconds since epoch)
    pub last_update_secs: f64,
    /// Data source name
    pub source: String,
    /// Update interval in seconds
    pub update_interval: f64,
}

impl LiveIndicator {
    /// Create a new live indicator
    #[must_use]
    pub const fn new(source: String, update_interval: f64) -> Self {
        Self {
            is_live: false,
            last_update_secs: 0.0,
            source,
            update_interval,
        }
    }

    /// Create indicator with fixed last-update time (for testing `age_string` branches)
    #[cfg(test)]
    #[must_use]
    pub const fn new_with_last_update(
        source: String,
        update_interval: f64,
        last_update_secs: f64,
    ) -> Self {
        Self {
            is_live: true,
            last_update_secs,
            source,
            update_interval,
        }
    }

    /// Mark as updated (now)
    pub fn mark_updated(&mut self) {
        use std::time::{SystemTime, UNIX_EPOCH};
        self.is_live = true;
        self.last_update_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0.0, |d| d.as_secs_f64());
    }

    /// Get age of last update in seconds
    #[must_use]
    pub fn age_seconds(&self) -> f64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0.0, |d| d.as_secs_f64());
        now - self.last_update_secs
    }

    /// Is the data stale? (older than 2x update interval)
    #[must_use]
    pub fn is_stale(&self) -> bool {
        self.age_seconds() > self.update_interval * 2.0
    }

    /// Format age as human-readable string
    #[must_use]
    pub fn age_string(&self) -> String {
        let age = self.age_seconds();
        if age < 1.0 {
            "Just now".to_string()
        } else if age < 60.0 {
            format!("{age:.1}s ago")
        } else if age < 3600.0 {
            format!("{:.1}m ago", age / 60.0)
        } else {
            format!("{:.1}h ago", age / 3600.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_size_multipliers() {
        assert_eq!(FontSize::Small.multiplier(), 0.85);
        assert_eq!(FontSize::Medium.multiplier(), 1.0);
        assert_eq!(FontSize::Large.multiplier(), 1.3);
        assert_eq!(FontSize::ExtraLarge.multiplier(), 1.6);
    }

    #[test]
    fn test_color_schemes() {
        for scheme in ColorScheme::all() {
            let palette = ColorPalette::from_scheme(*scheme);
            // Ensure all colors are different (basic sanity check)
            assert_ne!(palette.healthy, palette.error);
            assert_ne!(palette.background, palette.text);
        }
    }

    #[test]
    fn test_live_indicator() {
        let mut indicator = LiveIndicator::new("test".to_string(), 1.0);
        assert!(!indicator.is_live);

        indicator.mark_updated();
        assert!(indicator.is_live);
        assert!(indicator.age_seconds() < 0.1);
        assert!(!indicator.is_stale());
    }

    #[test]
    fn test_live_indicator_age_string_branches() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_secs_f64();

        let mut ind = LiveIndicator::new("x".to_string(), 1.0);
        ind.mark_updated();
        let s = ind.age_string();
        assert!(s == "Just now" || s.contains("s ago"));

        let ind = LiveIndicator::new_with_last_update("x".to_string(), 1.0, now - 45.0);
        assert!(ind.age_string().contains("s ago"));

        let ind = LiveIndicator::new_with_last_update("x".to_string(), 1.0, now - 120.0);
        assert!(ind.age_string().contains("m ago"));

        let ind = LiveIndicator::new_with_last_update("x".to_string(), 1.0, now - 7200.0);
        assert!(ind.age_string().contains("h ago"));
    }

    #[test]
    fn test_live_indicator_is_stale() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_secs_f64();
        let ind = LiveIndicator::new_with_last_update("x".to_string(), 1.0, now - 5.0);
        assert!(ind.is_stale());
    }

    #[test]
    fn test_color_scheme_names() {
        assert_eq!(ColorScheme::Default.name(), "Default");
        assert_eq!(ColorScheme::HighContrast.name(), "High Contrast");
        assert_eq!(ColorScheme::Deuteranopia.name(), "Deuteranopia (Red-Green)");
        assert_eq!(ColorScheme::Protanopia.name(), "Protanopia (Red-Blind)");
        assert_eq!(ColorScheme::Tritanopia.name(), "Tritanopia (Blue-Yellow)");
        assert_eq!(ColorScheme::Dark.name(), "Dark Mode");
        assert_eq!(ColorScheme::Light.name(), "Light Mode");
    }

    #[test]
    fn test_font_size_names() {
        assert_eq!(FontSize::Small.name(), "Small");
        assert_eq!(FontSize::Medium.name(), "Medium");
        assert_eq!(FontSize::Large.name(), "Large");
        assert_eq!(FontSize::ExtraLarge.name(), "Extra Large");
    }

    #[test]
    fn test_accessibility_settings_default() {
        let s = AccessibilitySettings::default();
        assert_eq!(s.color_scheme, ColorScheme::Default);
        assert_eq!(s.font_size, FontSize::Medium);
        assert!(s.audio_enabled);
        assert!((s.audio_volume - 0.8).abs() < f32::EPSILON);
        assert!(!s.narration_enabled);
        assert!(!s.keyboard_only);
        assert!(!s.screen_reader_mode);
        assert!(!s.high_contrast);
        assert!(!s.reduced_motion);
    }

    #[test]
    fn test_font_size_increase() {
        assert_eq!(FontSize::Small.increase(), FontSize::Medium);
        assert_eq!(FontSize::Medium.increase(), FontSize::Large);
        assert_eq!(FontSize::Large.increase(), FontSize::ExtraLarge);
        assert_eq!(FontSize::ExtraLarge.increase(), FontSize::ExtraLarge);
    }

    #[test]
    fn test_font_size_decrease() {
        assert_eq!(FontSize::Small.decrease(), FontSize::Small);
        assert_eq!(FontSize::Medium.decrease(), FontSize::Small);
        assert_eq!(FontSize::Large.decrease(), FontSize::Medium);
        assert_eq!(FontSize::ExtraLarge.decrease(), FontSize::Large);
    }
}
