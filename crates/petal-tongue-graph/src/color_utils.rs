//! Color Utilities
//!
//! Pure functions for color space conversions and manipulation.
//! Extracted from visual_2d.rs for reusability.

/// Convert HSV to RGB color space
///
/// # Arguments
/// * `h` - Hue (0-360 degrees)
/// * `s` - Saturation (0.0-1.0)
/// * `v` - Value/Brightness (0.0-1.0)
///
/// # Returns
/// RGB tuple (r, g, b) where each component is 0-255
///
/// # Example
/// ```
/// use petal_tongue_graph::color_utils::hsv_to_rgb;
///
/// // Pure red
/// let (r, g, b) = hsv_to_rgb(0.0, 1.0, 1.0);
/// assert_eq!((r, g, b), (255, 0, 0));
///
/// // Pure green
/// let (r, g, b) = hsv_to_rgb(120.0, 1.0, 1.0);
/// assert_eq!((r, g, b), (0, 255, 0));
/// ```
#[allow(clippy::many_single_char_names)] // Standard HSV→RGB notation: h,s,v,r,g,b,c,x,m
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // Values clamped to [0,255]
#[must_use]
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let h_prime = h / 60.0;
    let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
    let m = v - c;

    let (r, g, b) = if h_prime < 1.0 {
        (c, x, 0.0)
    } else if h_prime < 2.0 {
        (x, c, 0.0)
    } else if h_prime < 3.0 {
        (0.0, c, x)
    } else if h_prime < 4.0 {
        (0.0, x, c)
    } else if h_prime < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

/// Convert RGB to HSV color space
///
/// # Arguments
/// * `r` - Red (0-255)
/// * `g` - Green (0-255)
/// * `b` - Blue (0-255)
///
/// # Returns
/// HSV tuple (h, s, v) where:
/// - h: Hue (0-360 degrees)
/// - s: Saturation (0.0-1.0)
/// - v: Value (0.0-1.0)
#[must_use]
pub fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = f32::from(r) / 255.0;
    let g = f32::from(g) / 255.0;
    let b = f32::from(b) / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    // Value
    let v = max;

    // Saturation
    let s = if max == 0.0 { 0.0 } else { delta / max };

    // Hue
    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    // Normalize hue to [0, 360)
    let h = if h < 0.0 { h + 360.0 } else { h };

    (h, s, v)
}

/// Interpolate between two colors in HSV space
///
/// This often produces more visually pleasing gradients than RGB interpolation.
///
/// # Arguments
/// * `color1` - First color as (r, g, b)
/// * `color2` - Second color as (r, g, b)
/// * `t` - Interpolation factor (0.0-1.0)
///
/// # Returns
/// Interpolated color as (r, g, b)
#[must_use]
pub fn lerp_hsv(color1: (u8, u8, u8), color2: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    let (h1, s1, v1) = rgb_to_hsv(color1.0, color1.1, color1.2);
    let (h2, s2, v2) = rgb_to_hsv(color2.0, color2.1, color2.2);

    let t = t.clamp(0.0, 1.0);

    // Interpolate hue (handle wrap-around)
    let h = if (h2 - h1).abs() > 180.0 {
        // Take shorter path around color wheel
        if h1 < h2 {
            h1 + ((h2 - 360.0 - h1) * t)
        } else {
            h1 + ((h2 + 360.0 - h1) * t)
        }
    } else {
        h1 + ((h2 - h1) * t)
    };

    let h = if h < 0.0 {
        h + 360.0
    } else if h >= 360.0 {
        h - 360.0
    } else {
        h
    };

    let s = s1 + ((s2 - s1) * t);
    let v = v1 + ((v2 - v1) * t);

    hsv_to_rgb(h, s, v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hsv_to_rgb_primary_colors() {
        // Red
        assert_eq!(hsv_to_rgb(0.0, 1.0, 1.0), (255, 0, 0));
        // Green
        assert_eq!(hsv_to_rgb(120.0, 1.0, 1.0), (0, 255, 0));
        // Blue
        assert_eq!(hsv_to_rgb(240.0, 1.0, 1.0), (0, 0, 255));
    }

    #[test]
    fn test_hsv_to_rgb_grayscale() {
        // White (no saturation, full value)
        assert_eq!(hsv_to_rgb(0.0, 0.0, 1.0), (255, 255, 255));
        // Black (no value)
        assert_eq!(hsv_to_rgb(0.0, 0.0, 0.0), (0, 0, 0));
        // Gray (no saturation, half value)
        let (r, g, b) = hsv_to_rgb(0.0, 0.0, 0.5);
        assert!((r as i16 - 127).abs() <= 1);
        assert!((g as i16 - 127).abs() <= 1);
        assert!((b as i16 - 127).abs() <= 1);
    }

    #[test]
    fn test_rgb_to_hsv_roundtrip() {
        let test_colors = [(255, 0, 0), (0, 255, 0), (0, 0, 255), (128, 128, 128)];

        for (r, g, b) in test_colors {
            let (h, s, v) = rgb_to_hsv(r, g, b);
            let (r2, g2, b2) = hsv_to_rgb(h, s, v);
            assert!(
                (r as i16 - r2 as i16).abs() <= 1,
                "Red mismatch: {} vs {}",
                r,
                r2
            );
            assert!(
                (g as i16 - g2 as i16).abs() <= 1,
                "Green mismatch: {} vs {}",
                g,
                g2
            );
            assert!(
                (b as i16 - b2 as i16).abs() <= 1,
                "Blue mismatch: {} vs {}",
                b,
                b2
            );
        }
    }

    #[test]
    fn test_lerp_hsv() {
        // Interpolate from red to blue
        let color1 = (255, 0, 0); // Red
        let color2 = (0, 0, 255); // Blue

        // At t=0, should be red
        let result = lerp_hsv(color1, color2, 0.0);
        assert_eq!(result, color1);

        // At t=1, should be blue
        let result = lerp_hsv(color1, color2, 1.0);
        assert_eq!(result, color2);

        // At t=0.5, should be somewhere in between
        let result = lerp_hsv(color1, color2, 0.5);
        assert!(result != color1 && result != color2);
    }
}
