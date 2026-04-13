// SPDX-License-Identifier: AGPL-3.0-or-later

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
