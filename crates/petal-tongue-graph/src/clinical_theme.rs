// SPDX-License-Identifier: AGPL-3.0-only
//! Clinical color theme for healthSpring diagnostic UI.
//!
//! Absorbed from healthSpring petaltongue-health.

use egui::Color32;

/// Green color for healthy status indicators.
pub const HEALTHY: Color32 = Color32::from_rgb(46, 204, 113);
/// Amber color for warning status indicators.
pub const WARNING: Color32 = Color32::from_rgb(241, 196, 15);
/// Red color for critical status indicators.
pub const CRITICAL: Color32 = Color32::from_rgb(231, 76, 60);
/// Blue color for informational elements.
pub const INFO: Color32 = Color32::from_rgb(52, 152, 219);
/// Purple color for population/aggregate visualizations.
pub const POPULATION: Color32 = Color32::from_rgb(155, 89, 182);
/// Dark background for panel containers.
pub const BG_PANEL: Color32 = Color32::from_rgb(30, 30, 40);
/// Slightly lighter background for card surfaces.
pub const BG_CARD: Color32 = Color32::from_rgb(40, 42, 54);
/// Primary text color (high contrast).
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(248, 248, 242);
/// Dimmed text color for secondary content.
pub const TEXT_DIM: Color32 = Color32::from_rgb(128, 128, 160);

/// Map a health score (0..100) to a clinical color.
#[must_use]
pub const fn health_color(health: u8) -> Color32 {
    if health >= 90 {
        HEALTHY
    } else if health >= 50 {
        WARNING
    } else {
        CRITICAL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_color_healthy() {
        assert_eq!(health_color(90), HEALTHY);
        assert_eq!(health_color(95), HEALTHY);
        assert_eq!(health_color(100), HEALTHY);
    }

    #[test]
    fn test_health_color_warning() {
        assert_eq!(health_color(50), WARNING);
        assert_eq!(health_color(75), WARNING);
        assert_eq!(health_color(89), WARNING);
    }

    #[test]
    fn test_health_color_critical() {
        assert_eq!(health_color(0), CRITICAL);
        assert_eq!(health_color(25), CRITICAL);
        assert_eq!(health_color(49), CRITICAL);
    }

    #[test]
    fn test_theme_constants() {
        assert_ne!(HEALTHY, WARNING);
        assert_ne!(HEALTHY, CRITICAL);
        assert_ne!(WARNING, CRITICAL);
        assert_ne!(INFO, TEXT_PRIMARY);
    }

    #[test]
    fn test_health_color_boundary_89_90() {
        assert_eq!(health_color(89), WARNING);
        assert_eq!(health_color(90), HEALTHY);
    }

    #[test]
    fn test_health_color_boundary_49_50() {
        assert_eq!(health_color(49), CRITICAL);
        assert_eq!(health_color(50), WARNING);
    }

    #[test]
    fn test_health_color_all_constants_used() {
        let healthy = health_color(100);
        let warning = health_color(70);
        let critical = health_color(0);
        assert_eq!(healthy, HEALTHY);
        assert_eq!(warning, WARNING);
        assert_eq!(critical, CRITICAL);
    }

    #[test]
    fn test_theme_background_colors() {
        assert_ne!(BG_PANEL, BG_CARD);
        assert_ne!(TEXT_PRIMARY, TEXT_DIM);
    }
}
