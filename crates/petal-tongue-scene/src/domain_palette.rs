// SPDX-License-Identifier: AGPL-3.0-only
//! Domain-specific color palettes for visualization theming.
//!
//! Each domain (health, physics, ecology, agriculture, measurement, neural)
//! has a curated palette that satisfies Tufte principles: data-ink maximizing,
//! colorblind-safe, and semantically meaningful within the domain.
//!
//! Springs pass `domain: "health"` in their IPC requests and petalTongue
//! resolves the appropriate palette at render time.

use crate::primitive::Color;

/// A domain-specific color palette.
#[derive(Debug, Clone)]
pub struct DomainPalette {
    /// Domain identifier (matches IPC `domain` field).
    pub domain: &'static str,
    /// Primary data color (used for main data elements).
    pub primary: Color,
    /// Secondary data color (used for comparison/overlay).
    pub secondary: Color,
    /// Accent color (used for highlights, selections).
    pub accent: Color,
    /// Warning color (status: warning).
    pub warning: Color,
    /// Critical color (status: critical/error).
    pub critical: Color,
    /// Normal/healthy color (status: normal/good).
    pub normal: Color,
    /// Background hint (for chart areas, not full-screen).
    pub chart_bg: Color,
    /// Categorical palette (for facets, groups, series).
    pub categorical: &'static [Color],
}

const HEALTH_CATEGORICAL: [Color; 6] = [
    Color::rgb(0.106, 0.620, 0.467), // teal
    Color::rgb(0.851, 0.373, 0.008), // orange
    Color::rgb(0.459, 0.439, 0.702), // purple
    Color::rgb(0.906, 0.161, 0.541), // pink
    Color::rgb(0.400, 0.651, 0.118), // green
    Color::rgb(0.902, 0.671, 0.008), // yellow
];

const PHYSICS_CATEGORICAL: [Color; 6] = [
    Color::rgb(0.553, 0.227, 0.780), // violet
    Color::rgb(1.000, 0.498, 0.055), // orange
    Color::rgb(0.173, 0.627, 0.827), // cyan
    Color::rgb(0.839, 0.153, 0.157), // red
    Color::rgb(0.580, 0.404, 0.741), // lavender
    Color::rgb(0.549, 0.337, 0.294), // brown
];

const ECOLOGY_CATEGORICAL: [Color; 6] = [
    Color::rgb(0.102, 0.588, 0.314), // forest green
    Color::rgb(0.651, 0.463, 0.114), // amber
    Color::rgb(0.400, 0.271, 0.133), // brown
    Color::rgb(0.173, 0.435, 0.224), // dark green
    Color::rgb(0.741, 0.718, 0.420), // sage
    Color::rgb(0.337, 0.706, 0.537), // mint
];

const AGRICULTURE_CATEGORICAL: [Color; 6] = [
    Color::rgb(0.122, 0.467, 0.706), // blue
    Color::rgb(0.173, 0.627, 0.173), // green
    Color::rgb(0.682, 0.780, 0.910), // light blue
    Color::rgb(0.596, 0.875, 0.541), // light green
    Color::rgb(0.122, 0.376, 0.612), // navy
    Color::rgb(0.227, 0.714, 0.290), // lime
];

const MEASUREMENT_CATEGORICAL: [Color; 6] = [
    Color::rgb(0.400, 0.400, 0.400), // gray
    Color::rgb(0.122, 0.467, 0.706), // blue
    Color::rgb(0.600, 0.600, 0.600), // light gray
    Color::rgb(0.200, 0.627, 0.173), // green
    Color::rgb(0.300, 0.300, 0.300), // dark gray
    Color::rgb(0.737, 0.741, 0.133), // olive
];

const NEURAL_CATEGORICAL: [Color; 6] = [
    Color::rgb(0.122, 0.467, 0.906), // electric blue
    Color::rgb(0.800, 0.200, 0.800), // magenta
    Color::rgb(0.180, 0.800, 0.800), // cyan
    Color::rgb(0.996, 0.380, 0.000), // electric orange
    Color::rgb(0.580, 0.404, 0.900), // violet
    Color::rgb(0.000, 0.800, 0.400), // neon green
];

/// Health/clinical domain (healthSpring).
pub const HEALTH: DomainPalette = DomainPalette {
    domain: "health",
    primary: Color::rgb(0.106, 0.620, 0.467),
    secondary: Color::rgb(0.459, 0.439, 0.702),
    accent: Color::rgb(0.851, 0.373, 0.008),
    warning: Color::rgb(0.902, 0.671, 0.008),
    critical: Color::rgb(0.839, 0.153, 0.157),
    normal: Color::rgb(0.173, 0.627, 0.173),
    chart_bg: Color::rgba(0.98, 0.98, 0.98, 1.0),
    categorical: &HEALTH_CATEGORICAL,
};

/// Physics/plasma domain (hotSpring).
pub const PHYSICS: DomainPalette = DomainPalette {
    domain: "physics",
    primary: Color::rgb(0.553, 0.227, 0.780),
    secondary: Color::rgb(1.000, 0.498, 0.055),
    accent: Color::rgb(0.173, 0.627, 0.827),
    warning: Color::rgb(0.902, 0.671, 0.008),
    critical: Color::rgb(0.839, 0.153, 0.157),
    normal: Color::rgb(0.173, 0.627, 0.173),
    chart_bg: Color::rgba(0.02, 0.02, 0.06, 1.0),
    categorical: &PHYSICS_CATEGORICAL,
};

/// Ecology/metagenomics domain (wetSpring).
pub const ECOLOGY: DomainPalette = DomainPalette {
    domain: "ecology",
    primary: Color::rgb(0.102, 0.588, 0.314),
    secondary: Color::rgb(0.651, 0.463, 0.114),
    accent: Color::rgb(0.337, 0.706, 0.537),
    warning: Color::rgb(0.902, 0.671, 0.008),
    critical: Color::rgb(0.839, 0.153, 0.157),
    normal: Color::rgb(0.102, 0.588, 0.314),
    chart_bg: Color::rgba(0.97, 0.98, 0.96, 1.0),
    categorical: &ECOLOGY_CATEGORICAL,
};

/// Agriculture/atmospheric domain (airSpring).
pub const AGRICULTURE: DomainPalette = DomainPalette {
    domain: "agriculture",
    primary: Color::rgb(0.122, 0.467, 0.706),
    secondary: Color::rgb(0.173, 0.627, 0.173),
    accent: Color::rgb(0.682, 0.780, 0.910),
    warning: Color::rgb(0.902, 0.671, 0.008),
    critical: Color::rgb(0.839, 0.153, 0.157),
    normal: Color::rgb(0.173, 0.627, 0.173),
    chart_bg: Color::rgba(0.96, 0.98, 1.0, 1.0),
    categorical: &AGRICULTURE_CATEGORICAL,
};

/// Measurement/calibration domain (groundSpring).
pub const MEASUREMENT: DomainPalette = DomainPalette {
    domain: "measurement",
    primary: Color::rgb(0.400, 0.400, 0.400),
    secondary: Color::rgb(0.122, 0.467, 0.706),
    accent: Color::rgb(0.600, 0.600, 0.600),
    warning: Color::rgb(0.902, 0.671, 0.008),
    critical: Color::rgb(0.839, 0.153, 0.157),
    normal: Color::rgb(0.173, 0.627, 0.173),
    chart_bg: Color::rgba(0.97, 0.97, 0.97, 1.0),
    categorical: &MEASUREMENT_CATEGORICAL,
};

/// ML/neural domain (neuralSpring).
pub const NEURAL: DomainPalette = DomainPalette {
    domain: "neural",
    primary: Color::rgb(0.122, 0.467, 0.906),
    secondary: Color::rgb(0.800, 0.200, 0.800),
    accent: Color::rgb(0.180, 0.800, 0.800),
    warning: Color::rgb(0.902, 0.671, 0.008),
    critical: Color::rgb(0.839, 0.153, 0.157),
    normal: Color::rgb(0.173, 0.627, 0.173),
    chart_bg: Color::rgba(0.02, 0.02, 0.04, 1.0),
    categorical: &NEURAL_CATEGORICAL,
};

/// Game domain palette (ludoSpring).
pub const GAME: DomainPalette = DomainPalette {
    domain: "game",
    primary: Color::rgb(0.863, 0.627, 0.314), // warm gold (220, 160, 80)
    secondary: Color::rgb(0.400, 0.600, 0.800), // steel blue
    accent: Color::rgb(0.900, 0.400, 0.200),  // ember
    warning: Color::rgb(0.902, 0.671, 0.008),
    critical: Color::rgb(0.839, 0.153, 0.157),
    normal: Color::rgb(0.173, 0.627, 0.173),
    chart_bg: Color::rgba(0.06, 0.05, 0.08, 1.0),
    categorical: &GAME_CATEGORICAL,
};

const GAME_CATEGORICAL: [Color; 6] = [
    Color::rgb(0.863, 0.627, 0.314), // warm gold
    Color::rgb(0.400, 0.600, 0.800), // steel blue
    Color::rgb(0.900, 0.400, 0.200), // ember
    Color::rgb(0.200, 0.800, 0.400), // neon green
    Color::rgb(0.700, 0.300, 0.700), // magenta
    Color::rgb(0.180, 0.800, 0.800), // cyan
];

/// A three-stop diverging color scale for continuous value → color mapping.
///
/// Used by neuralSpring for Kokkos parity heatmaps: green at/below `mid`,
/// yellow near `mid`, red above `high`.
#[derive(Debug, Clone)]
pub struct DivergingScale {
    /// Value at which the low color is fully saturated.
    pub low: f64,
    /// Value at which the mid color is shown.
    pub mid: f64,
    /// Value at which the high color is fully saturated.
    pub high: f64,
    /// Color for values at or below `low`.
    pub low_color: Color,
    /// Color for values at `mid`.
    pub mid_color: Color,
    /// Color for values at or above `high`.
    pub high_color: Color,
}

impl DivergingScale {
    /// Interpolate a value to a color on this scale.
    #[must_use]
    pub fn interpolate(&self, value: f64) -> Color {
        if value <= self.low {
            return self.low_color;
        }
        if value >= self.high {
            return self.high_color;
        }
        if value <= self.mid {
            let range = self.mid - self.low;
            if range.abs() < f64::EPSILON {
                return self.low_color;
            }
            let t = ((value - self.low) / range) as f32;
            lerp_color(&self.low_color, &self.mid_color, t)
        } else {
            let range = self.high - self.mid;
            if range.abs() < f64::EPSILON {
                return self.high_color;
            }
            let t = ((value - self.mid) / range) as f32;
            lerp_color(&self.mid_color, &self.high_color, t)
        }
    }

    /// Default diverging scale: green ≤1.0×, yellow 1.0-2.0×, red ≥2.0×
    /// (neuralSpring Kokkos parity convention).
    #[must_use]
    pub const fn kokkos_parity() -> Self {
        Self {
            low: 0.0,
            mid: 1.0,
            high: 2.0,
            low_color: Color::rgb(0.173, 0.627, 0.173), // green
            mid_color: Color::rgb(0.902, 0.671, 0.008), // yellow
            high_color: Color::rgb(0.839, 0.153, 0.157), // red
        }
    }
}

fn lerp_color(a: &Color, b: &Color, t: f32) -> Color {
    Color::rgba(
        (b.r - a.r).mul_add(t, a.r),
        (b.g - a.g).mul_add(t, a.g),
        (b.b - a.b).mul_add(t, a.b),
        (b.a - a.a).mul_add(t, a.a),
    )
}

/// Resolve a domain string to its palette. Falls back to MEASUREMENT for unknown domains.
#[must_use]
pub fn palette_for_domain(domain: &str) -> &'static DomainPalette {
    match domain {
        "health" | "clinical" => &HEALTH,
        "physics" | "plasma" => &PHYSICS,
        "ecology" | "metagenomics" => &ECOLOGY,
        "agriculture" | "atmospheric" => &AGRICULTURE,
        "ml" | "neural" => &NEURAL,
        "game" | "ludology" => &GAME,
        // "measurement", "calibration", and any unknown domain fall back to MEASUREMENT
        _ => &MEASUREMENT,
    }
}

/// Get the primary color for a categorical index within a domain palette.
#[must_use]
pub fn categorical_color(palette: &DomainPalette, index: usize) -> Color {
    palette.categorical[index % palette.categorical.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_for_domain_resolves_all_domains() {
        assert_eq!(palette_for_domain("health").domain, "health");
        assert_eq!(palette_for_domain("clinical").domain, "health");
        assert_eq!(palette_for_domain("physics").domain, "physics");
        assert_eq!(palette_for_domain("plasma").domain, "physics");
        assert_eq!(palette_for_domain("ecology").domain, "ecology");
        assert_eq!(palette_for_domain("metagenomics").domain, "ecology");
        assert_eq!(palette_for_domain("agriculture").domain, "agriculture");
        assert_eq!(palette_for_domain("atmospheric").domain, "agriculture");
        assert_eq!(palette_for_domain("measurement").domain, "measurement");
        assert_eq!(palette_for_domain("calibration").domain, "measurement");
        assert_eq!(palette_for_domain("ml").domain, "neural");
        assert_eq!(palette_for_domain("neural").domain, "neural");
    }

    #[test]
    fn unknown_domain_falls_back_to_measurement() {
        assert_eq!(palette_for_domain("unknown").domain, "measurement");
    }

    #[test]
    fn categorical_color_wraps_around() {
        let palette = palette_for_domain("health");
        let c0 = categorical_color(palette, 0);
        let c6 = categorical_color(palette, 6);
        assert!((c0.r - c6.r).abs() < f32::EPSILON);
        assert!((c0.g - c6.g).abs() < f32::EPSILON);
    }

    #[test]
    fn all_palettes_have_six_categorical_colors() {
        for domain in &[
            "health",
            "physics",
            "ecology",
            "agriculture",
            "measurement",
            "neural",
        ] {
            let p = palette_for_domain(domain);
            assert_eq!(
                p.categorical.len(),
                6,
                "Domain {domain} should have 6 categorical colors"
            );
        }
    }

    #[test]
    fn palette_primary_secondary_differ() {
        for domain in &["health", "physics", "ecology"] {
            let p = palette_for_domain(domain);
            assert!(
                (p.primary.r - p.secondary.r).abs() > 0.01
                    || (p.primary.g - p.secondary.g).abs() > 0.01
                    || (p.primary.b - p.secondary.b).abs() > 0.01,
                "Domain {domain} primary and secondary should differ"
            );
        }
    }

    #[test]
    fn categorical_color_index_zero() {
        let palette = palette_for_domain("health");
        let c = categorical_color(palette, 0);
        assert!((c.r - palette.categorical[0].r).abs() < f32::EPSILON);
    }

    #[test]
    fn categorical_color_index_large_wraps() {
        let palette = palette_for_domain("measurement");
        let c7 = categorical_color(palette, 7);
        let c1 = categorical_color(palette, 1);
        assert!((c7.r - c1.r).abs() < f32::EPSILON);
    }

    #[test]
    fn game_domain_resolves() {
        assert_eq!(palette_for_domain("game").domain, "game");
        assert_eq!(palette_for_domain("ludology").domain, "game");
    }

    #[test]
    fn game_palette_has_warm_gold_primary() {
        let p = palette_for_domain("game");
        assert!((p.primary.r - 0.863).abs() < 0.01);
        assert!((p.primary.g - 0.627).abs() < 0.01);
    }

    #[test]
    fn diverging_scale_kokkos_parity() {
        let scale = DivergingScale::kokkos_parity();
        let low = scale.interpolate(0.0);
        assert!((low.g - 0.627).abs() < 0.01, "should be green at 0.0");
        let mid = scale.interpolate(1.0);
        assert!((mid.r - 0.902).abs() < 0.01, "should be yellow at 1.0");
        let high = scale.interpolate(2.0);
        assert!((high.r - 0.839).abs() < 0.01, "should be red at 2.0");
    }

    #[test]
    fn diverging_scale_interpolates_between_stops() {
        let scale = DivergingScale::kokkos_parity();
        let mid_low = scale.interpolate(0.5);
        assert!(mid_low.g > 0.3, "between green and yellow");
        let mid_high = scale.interpolate(1.5);
        assert!(mid_high.r > 0.5, "between yellow and red");
    }

    #[test]
    fn diverging_scale_clamps_extremes() {
        let scale = DivergingScale::kokkos_parity();
        let below = scale.interpolate(-1.0);
        assert!((below.g - scale.low_color.g).abs() < f32::EPSILON);
        let above = scale.interpolate(10.0);
        assert!((above.r - scale.high_color.r).abs() < f32::EPSILON);
    }

    #[test]
    fn domain_palette_has_all_status_colors() {
        let p = palette_for_domain("health");
        assert!(p.warning.r > 0.0 || p.warning.g > 0.0 || p.warning.b > 0.0);
        assert!(p.critical.r > 0.0 || p.critical.g > 0.0 || p.critical.b > 0.0);
        assert!(p.normal.r > 0.0 || p.normal.g > 0.0 || p.normal.b > 0.0);
    }
}
