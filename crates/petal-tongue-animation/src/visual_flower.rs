// SPDX-License-Identifier: AGPL-3.0-only
//! # Visual Flower Rendering
//!
//! High-quality visual flower animation for egui-based awakening experience.
//!
//! This module provides smooth, beautiful flower opening animations using
//! egui's painting API with bezier curves and gradients.

use crate::flower::{FlowerAnimation, FlowerState};

/// Visual Flower Renderer for egui
///
/// Renders beautiful flower opening animation using vector graphics.
pub struct VisualFlowerRenderer {
    /// Animation state
    animation: FlowerAnimation,

    /// Current time for animation
    current_time: f32,

    /// Base color (hue in HSV) — reserved for future color customization
    #[allow(dead_code, reason = "reserved for GPU/SVG renderer color theming")]
    base_hue: f32,
}

impl VisualFlowerRenderer {
    /// Create new visual flower renderer
    #[must_use]
    pub const fn new() -> Self {
        Self {
            animation: FlowerAnimation::new(30), // 30 FPS
            current_time: 0.0,
            base_hue: 330.0, // Pink/magenta for flower
        }
    }

    /// Update animation
    pub fn update(&mut self, delta_time: f32) {
        self.current_time += delta_time;
    }

    /// Reset animation
    pub const fn reset(&mut self) {
        self.animation.reset();
        self.current_time = 0.0;
    }

    /// Get current state
    #[must_use]
    pub fn current_state(&self) -> FlowerState {
        let progress = (self.current_time / 3.0).clamp(0.0, 1.0);
        Self::calculate_state(progress)
    }

    /// Calculate state from progress (0.0 to 1.0)
    fn calculate_state(progress: f32) -> FlowerState {
        if progress < 0.1 {
            FlowerState::Closed
        } else if progress < 0.9 {
            #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let percent = ((progress - 0.1) / 0.8 * 100.0) as u8;
            FlowerState::Opening(percent)
        } else {
            FlowerState::Open
        }
    }

    /// Get opening percentage (0.0 to 1.0) — reserved for future use in custom renderers
    #[allow(dead_code, reason = "reserved for GPU/SVG renderer opening animation")]
    fn opening_percent(&self) -> f32 {
        match self.current_state() {
            FlowerState::Closed => 0.0,
            FlowerState::Opening(p) => f32::from(p) / 100.0,
            FlowerState::Open | FlowerState::Glowing | FlowerState::Reaching => 1.0,
        }
    }
}

impl Default for VisualFlowerRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "egui")]
mod egui_rendering {
    use egui::{Color32, Pos2, Stroke, Vec2};
    use std::f32::consts::PI;

    use super::VisualFlowerRenderer;

    impl VisualFlowerRenderer {
        /// Render flower to egui
        pub fn render(&self, ui: &mut egui::Ui, center: Pos2, size: f32) {
            let painter = ui.painter();
            let progress = self.opening_percent();

            // Render stem
            self.render_stem(painter, center, size);

            // Render leaves
            self.render_leaves(painter, center, size, progress);

            // Render flower petals
            self.render_petals(painter, center, size, progress);

            // Render center/pistil
            self.render_center(painter, center, size, progress);

            // Render glow effect for open state
            if progress > 0.9 {
                self.render_glow(painter, center, size);
            }
        }

        /// Render stem
        // Graphics helper - self not needed but kept for consistency with other render methods
        #[expect(clippy::unused_self)]
        fn render_stem(&self, painter: &egui::Painter, center: Pos2, size: f32) {
            let stem_start = center + Vec2::new(0.0, size * 0.3);
            let stem_end = center + Vec2::new(0.0, size * 0.8);

            // Draw stem with gradient (darker at bottom, lighter at top)
            let stem_color = Color32::from_rgb(60, 120, 60);
            painter.line_segment([stem_start, stem_end], Stroke::new(size * 0.04, stem_color));
        }

        /// Render leaves
        fn render_leaves(&self, painter: &egui::Painter, center: Pos2, size: f32, progress: f32) {
            let leaf_y = size.mul_add(0.5, center.y);
            let leaf_size = size * 0.15;

            // Left leaf
            let left_center = Pos2::new(size.mul_add(-0.1, center.x), leaf_y);
            self.render_leaf(painter, left_center, leaf_size, -0.3, progress);

            // Right leaf
            let right_center = Pos2::new(size.mul_add(0.1, center.x), leaf_y);
            self.render_leaf(painter, right_center, leaf_size, 0.3, progress);
        }

        /// Render single leaf
        // Graphics helper - self not needed but kept for consistency with other render methods
        #[expect(clippy::unused_self)]
        fn render_leaf(
            &self,
            painter: &egui::Painter,
            center: Pos2,
            size: f32,
            angle: f32,
            progress: f32,
        ) {
            let leaf_growth = (progress * 1.5).clamp(0.0, 1.0);
            let actual_size = size * leaf_growth;

            // Create leaf shape (circle for now, egui doesn't have ellipse_filled)
            let leaf_color = Color32::from_rgb(70, 140, 70);
            let leaf_pos = center + Vec2::new(angle * actual_size * 0.6, 0.0);

            painter.circle_filled(leaf_pos, actual_size * 0.4, leaf_color);
        }

        /// Render flower petals
        fn render_petals(&self, painter: &egui::Painter, center: Pos2, size: f32, progress: f32) {
            let num_petals = 8;
            let petal_length = size * 0.25 * progress;
            let petal_width = size * 0.12 * progress;

            // Calculate angle offset based on opening progress
            let spread = progress * 0.8; // Petals spread as they open

            // Graphics calculations - precision loss acceptable for visual rendering
            #[expect(clippy::cast_precision_loss)]
            for i in 0..num_petals {
                let base_angle = (i as f32 / num_petals as f32) * 2.0 * PI;
                let angle =
                    (spread * (i as f32 - num_petals as f32 / 2.0)).mul_add(0.1, base_angle);

                self.render_petal(painter, center, petal_length, petal_width, angle, progress);
            }
        }

        /// Render single petal
        fn render_petal(
            &self,
            painter: &egui::Painter,
            center: Pos2,
            length: f32,
            width: f32,
            angle: f32,
            progress: f32,
        ) {
            let tip = center + Vec2::new(angle.cos() * length, angle.sin() * length);

            // Create petal gradient (lighter at tip, darker at base)
            let hue = self.base_hue;
            let saturation = 0.7;
            let value_base = 0.8 * progress.mul_add(0.5, 0.5);
            let value_tip = 1.0 * progress.mul_add(0.3, 0.7);

            let color_base = Self::hsv_to_color32(hue, saturation, value_base);
            let color_tip = Self::hsv_to_color32(hue, saturation, value_tip);

            // Draw petal as circle (egui doesn't have ellipse_filled)
            let petal_center =
                center + Vec2::new(angle.cos() * length * 0.6, angle.sin() * length * 0.6);

            painter.circle_filled(petal_center, width * 0.6, color_base);

            // Add highlight at tip
            let highlight_center =
                tip + Vec2::new(-angle.cos() * length * 0.15, -angle.sin() * length * 0.15);
            painter.circle_filled(highlight_center, width * 0.3, color_tip);
        }

        /// Render flower center (pistil/stamen)
        // Graphics helper - self not needed but kept for consistency with other render methods
        #[expect(clippy::unused_self)]
        fn render_center(&self, painter: &egui::Painter, center: Pos2, size: f32, progress: f32) {
            let center_size = size * 0.08 * progress;

            // Draw yellow/golden center
            let center_color = Color32::from_rgb(255, 220, 100);
            painter.circle_filled(center, center_size, center_color);

            // Add texture dots
            if progress > 0.5 {
                let dot_count = 12;
                let dot_radius = center_size * 0.6;
                // Graphics calculations - precision loss acceptable for visual rendering
                #[expect(clippy::cast_precision_loss)]
                for i in 0..dot_count {
                    let angle = (i as f32 / dot_count as f32) * 2.0 * PI;
                    let dot_pos =
                        center + Vec2::new(angle.cos() * dot_radius, angle.sin() * dot_radius);
                    painter.circle_filled(dot_pos, size * 0.01, Color32::from_rgb(200, 150, 50));
                }
            }
        }

        /// Render glow effect
        // Graphics helper - self not needed but kept for consistency with other render methods
        #[expect(clippy::unused_self)]
        fn render_glow(&self, painter: &egui::Painter, center: Pos2, size: f32) {
            // Multiple layers of glow with decreasing alpha
            // Graphics calculations - casts are intentional for color values
            #[expect(
                clippy::cast_precision_loss,
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss
            )]
            for i in 0..5 {
                let layer_size = size * (i as f32).mul_add(0.08, 0.35);
                let alpha = 30 - i * 5;
                let glow_color = Color32::from_rgba_premultiplied(255, 200, 220, alpha as u8);
                painter.circle_filled(center, layer_size, glow_color);
            }
        }

        /// Convert HSV to Color32
        // Standard HSV to RGB conversion algorithm
        // Single-letter variable names are standard notation in color science
        // Float to u8 casts are intentional for color conversion (0.0-1.0 → 0-255)
        #[expect(
            clippy::many_single_char_names,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        fn hsv_to_color32(h: f32, s: f32, v: f32) -> Color32 {
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

            Color32::from_rgb(
                ((r + m) * 255.0) as u8,
                ((g + m) * 255.0) as u8,
                ((b + m) * 255.0) as u8,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visual_flower_creation() {
        let renderer = VisualFlowerRenderer::new();
        assert!((renderer.current_time - 0.0).abs() < f32::EPSILON);
        assert_eq!(renderer.current_state(), FlowerState::Closed);
    }

    #[test]
    fn test_visual_flower_default() {
        let renderer = VisualFlowerRenderer::default();
        assert_eq!(renderer.current_state(), FlowerState::Closed);
    }

    #[test]
    fn test_visual_flower_update() {
        let mut renderer = VisualFlowerRenderer::new();
        renderer.update(1.5);
        assert!((renderer.current_time - 1.5).abs() < f32::EPSILON);
        assert!(matches!(renderer.current_state(), FlowerState::Opening(_)));
    }

    #[test]
    fn test_visual_flower_update_zero_delta() {
        let mut renderer = VisualFlowerRenderer::new();
        renderer.update(0.0);
        assert_eq!(renderer.current_state(), FlowerState::Closed);
    }

    #[test]
    fn test_visual_flower_reset() {
        let mut renderer = VisualFlowerRenderer::new();
        renderer.update(2.0);
        renderer.reset();
        assert!((renderer.current_time - 0.0).abs() < f32::EPSILON);
        assert_eq!(renderer.current_state(), FlowerState::Closed);
    }

    #[test]
    fn test_opening_percent() {
        let mut renderer = VisualFlowerRenderer::new();

        // Closed
        assert!((renderer.opening_percent() - 0.0).abs() < f32::EPSILON);

        // Opening
        renderer.update(1.5);
        let percent = renderer.opening_percent();
        assert!(percent > 0.0 && percent < 1.0);

        // Open
        renderer.update(3.0);
        assert!((renderer.opening_percent() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_state_progression() {
        let mut renderer = VisualFlowerRenderer::new();

        // Stage 1: Closed
        assert_eq!(renderer.current_state(), FlowerState::Closed);

        // Stage 2: Opening
        renderer.update(1.5);
        assert!(matches!(renderer.current_state(), FlowerState::Opening(_)));

        // Stage 3: Open
        renderer.update(5.0);
        assert_eq!(renderer.current_state(), FlowerState::Open);
    }

    #[test]
    fn test_state_boundary_just_below_closed() {
        let mut renderer = VisualFlowerRenderer::new();
        renderer.update(0.29); // progress = 0.29/3 < 0.1
        assert_eq!(renderer.current_state(), FlowerState::Closed);
    }

    #[test]
    fn test_state_boundary_just_above_closed() {
        let mut renderer = VisualFlowerRenderer::new();
        renderer.update(0.31); // progress ~= 0.103 > 0.1
        assert!(matches!(renderer.current_state(), FlowerState::Opening(_)));
    }

    #[test]
    fn test_state_boundary_just_below_open() {
        let mut renderer = VisualFlowerRenderer::new();
        renderer.update(2.69); // progress ~= 0.897 < 0.9
        assert!(matches!(renderer.current_state(), FlowerState::Opening(_)));
    }

    #[test]
    fn test_state_boundary_just_above_open() {
        let mut renderer = VisualFlowerRenderer::new();
        renderer.update(2.71); // progress ~= 0.903 > 0.9
        assert_eq!(renderer.current_state(), FlowerState::Open);
    }

    #[test]
    fn test_opening_percent_mid_range() {
        let mut renderer = VisualFlowerRenderer::new();
        renderer.update(1.5); // progress = 0.5, opening percent ~= 50%
        let p = renderer.opening_percent();
        assert!(p > 0.4 && p < 0.6);
    }

    #[test]
    fn test_opening_percent_glowing_reaching_return_one() {
        // VisualFlowerRenderer's calculate_state only returns Closed/Opening/Open,
        // but opening_percent handles Glowing/Reaching as 1.0. We test via
        // current_state which can't reach those - verify Open gives 1.0
        let mut renderer = VisualFlowerRenderer::new();
        renderer.update(4.0); // progress > 0.9 -> Open
        assert_eq!(renderer.current_state(), FlowerState::Open);
        assert!((renderer.opening_percent() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_frame_generation_progression() {
        let mut renderer = VisualFlowerRenderer::new();
        let mut prev_time = 0.0;
        for _ in 0..5 {
            renderer.update(0.5);
            prev_time += 0.5;
            assert!((renderer.current_time - prev_time).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn test_reset_clears_animation() {
        let mut renderer = VisualFlowerRenderer::new();
        renderer.update(2.5);
        assert!((renderer.current_time - 2.5).abs() < f32::EPSILON);
        renderer.reset();
        assert!((renderer.current_time - 0.0).abs() < f32::EPSILON);
        assert_eq!(renderer.current_state(), FlowerState::Closed);
    }

    #[test]
    fn test_progress_clamping_above_one() {
        let mut renderer = VisualFlowerRenderer::new();
        renderer.update(10.0);
        assert_eq!(renderer.current_state(), FlowerState::Open);
        assert!((renderer.opening_percent() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_opening_interpolation_exact_boundaries() {
        let mut renderer = VisualFlowerRenderer::new();
        renderer.update(0.0);
        assert_eq!(renderer.current_state(), FlowerState::Closed);
        renderer.update(0.09);
        assert_eq!(renderer.current_state(), FlowerState::Closed);
        renderer.update(2.7);
        assert_eq!(renderer.current_state(), FlowerState::Open);
        renderer.update(3.0);
        assert_eq!(renderer.current_state(), FlowerState::Open);
    }

    #[test]
    fn test_opening_percent_linear_midpoint() {
        let mut renderer = VisualFlowerRenderer::new();
        renderer.update(1.5);
        let p = renderer.opening_percent();
        let expected = (1.5 / 3.0 - 0.1) / 0.8;
        assert!(
            (p - expected).abs() < 0.02,
            "opening percent ~{p} should be near {expected}",
        );
    }

    #[test]
    fn test_state_transition_closed_to_opening() {
        let mut renderer = VisualFlowerRenderer::new();
        renderer.update(0.05);
        assert_eq!(renderer.current_state(), FlowerState::Closed);
        renderer.update(0.35);
        assert!(matches!(renderer.current_state(), FlowerState::Opening(_)));
    }

    #[test]
    fn test_state_transition_opening_to_open() {
        let mut renderer = VisualFlowerRenderer::new();
        renderer.update(1.0);
        assert!(matches!(renderer.current_state(), FlowerState::Opening(_)));
        renderer.update(2.8);
        assert_eq!(renderer.current_state(), FlowerState::Open);
    }

    #[test]
    fn test_opening_percent_increments_with_time() {
        let mut renderer = VisualFlowerRenderer::new();
        let p0 = renderer.opening_percent();
        renderer.update(0.5);
        let p1 = renderer.opening_percent();
        renderer.update(0.5);
        let p2 = renderer.opening_percent();
        assert!(p0 < p1);
        assert!(p1 < p2);
    }

    #[test]
    fn test_update_accumulates_delta() {
        let mut renderer = VisualFlowerRenderer::new();
        renderer.update(0.1);
        renderer.update(0.2);
        renderer.update(0.3);
        assert!((renderer.current_time - 0.6).abs() < f32::EPSILON);
    }

    #[test]
    fn test_reset_after_open_returns_to_closed() {
        let mut renderer = VisualFlowerRenderer::new();
        renderer.update(5.0);
        assert_eq!(renderer.current_state(), FlowerState::Open);
        renderer.reset();
        assert_eq!(renderer.current_state(), FlowerState::Closed);
        assert!((renderer.opening_percent() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_opening_state_percent_range() {
        let mut renderer = VisualFlowerRenderer::new();
        renderer.update(0.5);
        if let FlowerState::Opening(p) = renderer.current_state() {
            assert!(p > 0 && p < 100);
        } else {
            panic!("expected Opening");
        }
    }
}
