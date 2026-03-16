// SPDX-License-Identifier: AGPL-3.0-or-later
//! # Awakening Overlay for Egui
//!
//! Full-screen awakening experience with visual flower animation.

use crate::error::Result;
use petal_tongue_animation::VisualFlowerRenderer;
use petal_tongue_core::awakening::AwakeningStage;
use std::time::Instant;

/// Awakening Overlay State
pub struct AwakeningOverlay {
    /// Visual flower renderer
    flower_renderer: VisualFlowerRenderer,

    /// Current stage
    current_stage: AwakeningStage,

    /// Start time
    start_time: Instant,

    /// Is active
    active: bool,

    /// Should transition to tutorial
    transition_to_tutorial: bool,
}

impl AwakeningOverlay {
    /// Create new awakening overlay
    #[must_use]
    pub fn new() -> Self {
        Self {
            flower_renderer: VisualFlowerRenderer::new(),
            current_stage: AwakeningStage::Awakening,
            start_time: Instant::now(),
            active: false,
            transition_to_tutorial: false,
        }
    }

    /// Start the awakening experience
    pub fn start(&mut self) {
        self.active = true;
        self.start_time = Instant::now();
        self.flower_renderer.reset();
        self.current_stage = AwakeningStage::Awakening;
        tracing::info!("🌸 Starting visual awakening experience");
    }

    /// Check if active
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active
    }

    /// Skip the awakening overlay immediately (motor efferent command).
    pub const fn skip(&mut self) {
        self.active = false;
    }

    /// Check if should transition to tutorial
    #[must_use]
    pub const fn should_transition_to_tutorial(&self) -> bool {
        self.transition_to_tutorial
    }

    /// Update awakening state
    ///
    /// # Errors
    ///
    /// Currently always returns `Ok(())`; reserved for future extensibility.
    pub fn update(&mut self, delta_time: f32) -> Result<()> {
        if !self.active {
            return Ok(());
        }

        let elapsed = self.start_time.elapsed().as_secs_f32();

        // Update stage based on time
        self.current_stage = match elapsed {
            t if t < 3.0 => AwakeningStage::Awakening,
            t if t < 6.0 => AwakeningStage::SelfKnowledge,
            t if t < 10.0 => AwakeningStage::Discovery,
            t if t < 12.0 => AwakeningStage::Tutorial,
            _ => {
                // Check for tutorial mode
                self.transition_to_tutorial = std::env::var("SHOWCASE_MODE")
                    .ok()
                    .and_then(|v| v.parse::<bool>().ok())
                    .unwrap_or(false);

                self.active = false;
                AwakeningStage::Complete
            }
        };

        // Update flower animation
        self.flower_renderer.update(delta_time);

        Ok(())
    }

    /// Render awakening overlay
    pub fn render(&self, ctx: &egui::Context) {
        if !self.active {
            return;
        }

        // Full-screen overlay
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(20, 20, 25)))
            .show(ctx, |ui| {
                let rect = ui.max_rect();
                let center = rect.center();

                // Calculate size based on screen
                let size = rect.width().min(rect.height()) * 0.4;

                // Render visual flower
                self.flower_renderer.render(ui, center, size);

                // Render stage text
                self.render_stage_text(ui, rect);
            });
    }

    /// Render stage text
    fn render_stage_text(&self, ui: &egui::Ui, rect: egui::Rect) {
        let text = match self.current_stage {
            AwakeningStage::Awakening => "🌸 Awakening...",
            AwakeningStage::SelfKnowledge => "✨ I am petalTongue",
            AwakeningStage::Discovery => "🔍 Discovering the garden...",
            AwakeningStage::Tutorial => "🌿 Ready to explore...",
            AwakeningStage::Complete => "✅ Complete!",
        };

        // Position text below flower
        let text_pos = rect.center() + egui::Vec2::new(0.0, rect.height() * 0.25);

        ui.painter().text(
            text_pos,
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(32.0),
            egui::Color32::from_rgb(150, 255, 180),
        );

        // Show stage name below
        let stage_name = match self.current_stage {
            AwakeningStage::Awakening => "Stage 1: Awakening",
            AwakeningStage::SelfKnowledge => "Stage 2: Self-Knowledge",
            AwakeningStage::Discovery => "Stage 3: Discovery",
            AwakeningStage::Tutorial => "Stage 4: Tutorial Invitation",
            AwakeningStage::Complete => "Complete",
        };
        let stage_pos = text_pos + egui::Vec2::new(0.0, 40.0);

        ui.painter().text(
            stage_pos,
            egui::Align2::CENTER_CENTER,
            stage_name,
            egui::FontId::proportional(20.0),
            egui::Color32::from_rgb(100, 200, 150),
        );
    }
}

impl Default for AwakeningOverlay {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_awakening_overlay_creation() {
        let overlay = AwakeningOverlay::new();
        assert!(!overlay.is_active());
        assert_eq!(overlay.current_stage, AwakeningStage::Awakening);
    }

    #[test]
    fn test_awakening_overlay_start() {
        let mut overlay = AwakeningOverlay::new();
        overlay.start();
        assert!(overlay.is_active());
    }

    #[test]
    fn test_awakening_overlay_progression() {
        let mut overlay = AwakeningOverlay::new();
        overlay.start();

        // Stage 1: Awakening
        assert_eq!(overlay.current_stage, AwakeningStage::Awakening);

        // Simulate time passing
        overlay.start_time = Instant::now() - std::time::Duration::from_secs(4);
        overlay.update(0.016).unwrap();
        assert_eq!(overlay.current_stage, AwakeningStage::SelfKnowledge);

        // Stage 3
        overlay.start_time = Instant::now() - std::time::Duration::from_secs(7);
        overlay.update(0.016).unwrap();
        assert_eq!(overlay.current_stage, AwakeningStage::Discovery);
    }

    #[test]
    fn test_awakening_overlay_completion() {
        let mut overlay = AwakeningOverlay::new();
        overlay.start();

        // Simulate completion
        overlay.start_time = Instant::now() - std::time::Duration::from_secs(13);
        overlay.update(0.016).unwrap();

        assert!(!overlay.is_active());
        assert_eq!(overlay.current_stage, AwakeningStage::Complete);
    }
}
