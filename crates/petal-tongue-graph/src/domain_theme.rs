// SPDX-License-Identifier: AGPL-3.0-only
//! Domain-aware theme system for multi-spring visualization.
//!
//! Each spring domain gets colors that are appropriate to its field,
//! following Tufte's principle of smallest effective difference while
//! ensuring accessibility (WCAG AA contrast ratios on dark backgrounds).

use egui::Color32;

/// A complete color palette for a domain.
///
/// All palettes are designed for dark backgrounds (e.g. `#1a1a2e`) with
/// sufficient contrast for WCAG AA compliance.
#[derive(Debug, Clone, Copy)]
pub struct DomainPalette {
    /// Primary accent color (used for main data elements).
    pub primary: Color32,
    /// Secondary accent (used for comparison/secondary data).
    pub secondary: Color32,
    /// Color for "good" or "normal" state.
    pub positive: Color32,
    /// Color for "warning" or "attention" state.
    pub caution: Color32,
    /// Color for "critical" or "error" state.
    pub negative: Color32,
    /// Informational accent.
    pub info: Color32,
    /// Card/panel background.
    pub bg_card: Color32,
    /// Dimmed text.
    pub text_dim: Color32,
    /// Bright text.
    pub text_bright: Color32,
}

/// Shared background and text colors for all palettes.
const BG_CARD: Color32 = Color32::from_rgb(40, 40, 55);
const TEXT_DIM: Color32 = Color32::from_rgb(180, 180, 180);
const TEXT_BRIGHT: Color32 = Color32::from_rgb(240, 240, 240);

/// Health/clinical domain: green (healthy), amber (warning), red (critical).
/// Used by healthSpring data.
pub static HEALTH: DomainPalette = DomainPalette {
    primary: Color32::from_rgb(100, 181, 246),
    secondary: Color32::from_rgb(155, 89, 182),
    positive: Color32::from_rgb(76, 175, 80),
    caution: Color32::from_rgb(255, 183, 77),
    negative: Color32::from_rgb(244, 67, 54),
    info: Color32::from_rgb(52, 152, 219),
    bg_card: BG_CARD,
    text_dim: TEXT_DIM,
    text_bright: TEXT_BRIGHT,
};

/// Physics/plasma/energy domain: purple, orange, cyan.
/// Used by hotSpring data.
pub static PHYSICS: DomainPalette = DomainPalette {
    primary: Color32::from_rgb(156, 39, 176),
    secondary: Color32::from_rgb(255, 152, 0),
    positive: Color32::from_rgb(0, 188, 212),
    caution: Color32::from_rgb(255, 152, 0),
    negative: Color32::from_rgb(244, 67, 54),
    info: Color32::from_rgb(0, 188, 212),
    bg_card: BG_CARD,
    text_dim: TEXT_DIM,
    text_bright: TEXT_BRIGHT,
};

/// Ecology/earth/life domain: greens, browns.
/// Used by wetSpring data.
pub static ECOLOGY: DomainPalette = DomainPalette {
    primary: Color32::from_rgb(139, 195, 74),
    secondary: Color32::from_rgb(121, 85, 72),
    positive: Color32::from_rgb(76, 175, 80),
    caution: Color32::from_rgb(255, 193, 7),
    negative: Color32::from_rgb(255, 87, 34),
    info: Color32::from_rgb(66, 165, 245),
    bg_card: BG_CARD,
    text_dim: TEXT_DIM,
    text_bright: TEXT_BRIGHT,
};

/// Atmospheric/sky/water domain: blues, teals.
/// Used by airSpring data.
pub static ATMOSPHERIC: DomainPalette = DomainPalette {
    primary: Color32::from_rgb(3, 169, 244),
    secondary: Color32::from_rgb(0, 150, 136),
    positive: Color32::from_rgb(76, 175, 80),
    caution: Color32::from_rgb(255, 152, 0),
    negative: Color32::from_rgb(255, 152, 0),
    info: Color32::from_rgb(3, 169, 244),
    bg_card: BG_CARD,
    text_dim: TEXT_DIM,
    text_bright: TEXT_BRIGHT,
};

/// Measurement/precision domain: neutral grays, precision blue.
/// Used by groundSpring data.
pub static MEASUREMENT: DomainPalette = DomainPalette {
    primary: Color32::from_rgb(158, 158, 158),
    secondary: Color32::from_rgb(100, 181, 246),
    positive: Color32::from_rgb(76, 175, 80),
    caution: Color32::from_rgb(255, 183, 77),
    negative: Color32::from_rgb(244, 67, 54),
    info: Color32::from_rgb(100, 181, 246),
    bg_card: BG_CARD,
    text_dim: TEXT_DIM,
    text_bright: TEXT_BRIGHT,
};

/// Neural/ML domain: electric blue, magenta.
/// Used by neuralSpring data.
pub static NEURAL: DomainPalette = DomainPalette {
    primary: Color32::from_rgb(0, 176, 255),
    secondary: Color32::from_rgb(233, 30, 99),
    positive: Color32::from_rgb(0, 230, 118),
    caution: Color32::from_rgb(255, 193, 7),
    negative: Color32::from_rgb(255, 82, 82),
    info: Color32::from_rgb(0, 176, 255),
    bg_card: BG_CARD,
    text_dim: TEXT_DIM,
    text_bright: TEXT_BRIGHT,
};

/// Default fallback palette (same as health).
pub static DEFAULT: DomainPalette = HEALTH;

/// Returns the color palette for a given spring domain.
///
/// Domain strings are matched case-insensitively. Unknown domains
/// receive the default (health) palette.
///
/// # Examples
///
/// ```
/// # use petal_tongue_graph::domain_theme::{palette_for_domain, HEALTH, PHYSICS};
/// let p = palette_for_domain("health");
/// assert!(std::ptr::eq(p, &HEALTH));
///
/// let p = palette_for_domain("plasma");
/// assert!(std::ptr::eq(p, &PHYSICS));
/// ```
#[must_use]
pub fn palette_for_domain(domain: &str) -> &'static DomainPalette {
    let lower = domain.to_lowercase();
    match lower.as_str() {
        "health" | "clinical" => &HEALTH,
        "physics" | "plasma" | "nuclear" => &PHYSICS,
        "ecology" | "metagenomics" | "chemistry" => &ECOLOGY,
        "agriculture" | "hydrology" | "atmospheric" => &ATMOSPHERIC,
        "measurement" | "uncertainty" | "calibration" => &MEASUREMENT,
        "ml" | "neural" | "surrogate" => &NEURAL,
        _ => &DEFAULT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_health() {
        let p = palette_for_domain("health");
        assert_eq!(p.primary, HEALTH.primary);
    }

    #[test]
    fn palette_clinical() {
        let p = palette_for_domain("clinical");
        assert_eq!(p.primary, HEALTH.primary);
    }

    #[test]
    fn palette_physics() {
        let p = palette_for_domain("physics");
        assert_eq!(p.primary, PHYSICS.primary);
    }

    #[test]
    fn palette_plasma() {
        let p = palette_for_domain("plasma");
        assert_eq!(p.primary, PHYSICS.primary);
    }

    #[test]
    fn palette_nuclear() {
        let p = palette_for_domain("nuclear");
        assert_eq!(p.primary, PHYSICS.primary);
    }

    #[test]
    fn palette_ecology() {
        let p = palette_for_domain("ecology");
        assert_eq!(p.primary, ECOLOGY.primary);
    }

    #[test]
    fn palette_metagenomics() {
        let p = palette_for_domain("metagenomics");
        assert_eq!(p.primary, ECOLOGY.primary);
    }

    #[test]
    fn palette_chemistry() {
        let p = palette_for_domain("chemistry");
        assert_eq!(p.primary, ECOLOGY.primary);
    }

    #[test]
    fn palette_atmospheric() {
        let p = palette_for_domain("atmospheric");
        assert_eq!(p.primary, ATMOSPHERIC.primary);
    }

    #[test]
    fn palette_measurement() {
        let p = palette_for_domain("measurement");
        assert_eq!(p.primary, MEASUREMENT.primary);
    }

    #[test]
    fn palette_neural() {
        let p = palette_for_domain("neural");
        assert_eq!(p.primary, NEURAL.primary);
    }

    #[test]
    fn palette_ml() {
        let p = palette_for_domain("ml");
        assert_eq!(p.primary, NEURAL.primary);
    }

    #[test]
    fn palette_unknown_returns_default() {
        let p = palette_for_domain("unknown_domain_xyz");
        assert_eq!(p.primary, DEFAULT.primary);
    }

    #[test]
    fn palette_finance_returns_default() {
        let p = palette_for_domain("finance");
        assert_eq!(p.primary, DEFAULT.primary);
    }

    #[test]
    fn palette_network_returns_default() {
        let p = palette_for_domain("network");
        assert_eq!(p.primary, DEFAULT.primary);
    }

    #[test]
    fn palette_case_insensitive() {
        assert_eq!(palette_for_domain("HEALTH").primary, HEALTH.primary);
        assert_eq!(palette_for_domain("Physics").primary, PHYSICS.primary);
        assert_eq!(palette_for_domain("ECOLOGY").primary, ECOLOGY.primary);
    }

    #[test]
    fn palette_hydrology_returns_atmospheric() {
        let p = palette_for_domain("hydrology");
        assert_eq!(p.primary, ATMOSPHERIC.primary);
    }

    #[test]
    fn palette_agriculture_returns_atmospheric() {
        let p = palette_for_domain("agriculture");
        assert_eq!(p.primary, ATMOSPHERIC.primary);
    }

    #[test]
    fn palette_surrogate_returns_neural() {
        let p = palette_for_domain("surrogate");
        assert_eq!(p.primary, NEURAL.primary);
    }

    #[test]
    fn palette_uncertainty_returns_measurement() {
        let p = palette_for_domain("uncertainty");
        assert_eq!(p.primary, MEASUREMENT.primary);
    }

    #[test]
    fn palette_calibration_returns_measurement() {
        let p = palette_for_domain("calibration");
        assert_eq!(p.primary, MEASUREMENT.primary);
    }

    #[test]
    fn domain_palette_has_all_fields() {
        let p = palette_for_domain("health");
        let _ = p.primary;
        let _ = p.secondary;
        let _ = p.positive;
        let _ = p.caution;
        let _ = p.negative;
        let _ = p.info;
        let _ = p.bg_card;
        let _ = p.text_dim;
        let _ = p.text_bright;
    }
}
