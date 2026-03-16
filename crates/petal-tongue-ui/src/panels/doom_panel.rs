// SPDX-License-Identifier: AGPL-3.0-or-later
//! Doom Panel - Embeds Doom within petalTongue
//!
//! This is our first test of the panel system. As we implement this,
//! we'll discover gaps in petalTongue's architecture and evolve to fill them.

use super::doom_helpers::{
    compute_fps, compute_keys_diff, egui_to_doom_key_static, prepare_doom_display,
};
use doom_core::{DoomInstance, DoomKey, Result};
use egui::{ColorImage, Key, TextureHandle, Ui};
use std::collections::HashSet;
use std::time::Instant;

/// Panel that embeds Doom
pub struct DoomPanel {
    /// Doom game instance
    doom: Option<DoomInstance>,

    /// egui texture for rendering
    texture: Option<TextureHandle>,

    /// Last update time (for frame timing)
    last_update: Instant,

    /// Show debug overlay?
    show_debug: bool,

    /// Frame counter
    frame_count: u64,

    /// FPS tracker
    fps: f32,
    last_fps_update: Instant,
    frames_since_fps_update: u32,

    /// 🖥️ Track previously pressed keys (for state change detection)
    prev_keys_down: HashSet<Key>,
}

impl DoomPanel {
    /// Create a new Doom panel
    #[must_use]
    pub fn new() -> Self {
        Self {
            doom: None,
            texture: None,
            last_update: Instant::now(),
            show_debug: true,
            frame_count: 0,
            fps: 0.0,
            last_fps_update: Instant::now(),
            frames_since_fps_update: 0,
            prev_keys_down: HashSet::new(),
        }
    }

    /// Initialize Doom (lazy initialization)
    fn ensure_initialized(&mut self, width: usize, height: usize) -> Result<()> {
        if self.doom.is_none() {
            tracing::info!("Initializing Doom panel: {}x{}", width, height);
            let mut doom = DoomInstance::new(width, height)?;
            doom.init()?;
            self.doom = Some(doom);
        }
        Ok(())
    }

    /// Update game logic
    fn update(&mut self) {
        if let Some(doom) = &mut self.doom {
            let _elapsed = self.last_update.elapsed();

            // 🎮 FIX: Tick every frame for smooth movement!
            // Original Doom ran at 35 Hz, but modern games tick at render rate
            // This fixes the "tick, pause, tick" stuttering
            if let Err(e) = doom.tick() {
                tracing::error!("Doom tick error: {}", e);
            }
            self.last_update = Instant::now();
            self.frame_count += 1;
        }

        // Update FPS counter
        self.frames_since_fps_update += 1;
        let fps_elapsed = self.last_fps_update.elapsed();
        if fps_elapsed.as_secs_f64() >= 1.0 {
            self.fps = compute_fps(self.frames_since_fps_update, fps_elapsed.as_secs_f64());
            self.frames_since_fps_update = 0;
            self.last_fps_update = Instant::now();
        }
    }

    /// Render to egui
    pub fn render(&mut self, ui: &mut Ui) {
        // 🖥️ CRITICAL REMOTE DESKTOP FIX: Tell egui we want ALL input
        ui.ctx().set_cursor_icon(egui::CursorIcon::Default);

        // Initialize on first render
        if let Err(e) = self.ensure_initialized(640, 480) {
            ui.colored_label(
                egui::Color32::RED,
                format!("Doom initialization failed: {e}"),
            );
            return;
        }

        // Update game state
        self.update();

        // Get framebuffer from Doom
        if let Some(doom) = &self.doom {
            let (width, height) = doom.dimensions();
            let framebuffer = doom.framebuffer();

            // Convert to egui ColorImage
            let color_image = ColorImage::from_rgba_unmultiplied([width, height], framebuffer);

            // Update or create texture
            if let Some(texture) = &mut self.texture {
                texture.set(color_image, Default::default());
            } else {
                self.texture = Some(ui.ctx().load_texture(
                    "doom_frame",
                    color_image,
                    Default::default(),
                ));
            }

            // Display texture
            if let Some(texture) = &self.texture {
                // 🖥️ Make the image interactive (critical for input capture!)
                let response = ui.add(
                    egui::Image::new(egui::load::SizedTexture::new(
                        texture.id(),
                        egui::vec2(width as f32, height as f32),
                    ))
                    .sense(egui::Sense::click_and_drag()), // Make it sense ALL input
                );

                // 🎮 Request focus AGGRESSIVELY
                response.request_focus();
                ui.memory_mut(|mem| mem.request_focus(response.id));

                // 🖥️ ALWAYS handle input (not conditional!)
                // Take doom out temporarily to avoid borrow checker issues
                if let Some(mut doom) = self.doom.take() {
                    self.handle_input(ui, &mut doom);
                    self.doom = Some(doom);
                }
            }

            // Debug overlay
            if self.show_debug {
                self.render_debug_overlay(ui);
            }
        }
    }

    /// Handle keyboard and mouse input with state change detection
    fn handle_input(&mut self, ui: &Ui, doom: &mut DoomInstance) {
        // 🖥️ REMOTE DESKTOP FIX: Poll key state and only send on CHANGE
        // This prevents stuttering from repeated key_down/key_up calls

        ui.input(|i| {
            let keys_to_check = [
                Key::W,
                Key::ArrowUp,
                Key::S,
                Key::ArrowDown,
                Key::A,
                Key::ArrowLeft,
                Key::D,
                Key::ArrowRight,
                Key::Q,
                Key::E,
                Key::Space,
                Key::Enter,
                Key::Escape,
                Key::Num1,
                Key::Num2,
                Key::Num3,
                Key::Num4,
                Key::Num5,
                Key::Tab,
            ];

            // Build current keys_down set
            let current_keys: HashSet<Key> = keys_to_check
                .iter()
                .filter(|k| i.keys_down.contains(k))
                .copied()
                .collect();

            let (newly_pressed, newly_released) =
                compute_keys_diff(&self.prev_keys_down, &current_keys);

            for key in &newly_pressed {
                if let Some(doom_key) = egui_to_doom_key_static(*key) {
                    doom.key_down(doom_key);
                    tracing::debug!("🎮 Key DOWN: {:?}", key);
                }
            }
            for key in &newly_released {
                if let Some(doom_key) = egui_to_doom_key_static(*key) {
                    doom.key_up(doom_key);
                    tracing::debug!("🎮 Key UP: {:?}", key);
                }
            }

            self.prev_keys_down = current_keys;

            // 🖥️ REMOVED: Event processing
            // We use state polling exclusively now (works for both local AND remote)
            // Event processing was causing DOUBLE key_down/key_up calls!
        });

        // Mouse input
        if ui.input(|i| i.pointer.button_pressed(egui::PointerButton::Primary)) {
            doom.key_down(DoomKey::Fire);
        }
        if ui.input(|i| i.pointer.button_released(egui::PointerButton::Primary)) {
            doom.key_up(DoomKey::Fire);
        }
    }

    /// Render debug overlay
    fn render_debug_overlay(&self, ui: &mut Ui) {
        let display_state = prepare_doom_display(self.doom.as_ref(), self.fps, self.show_debug);
        if display_state.initialized && display_state.show_debug {
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(format!("FPS: {:.1}", display_state.fps));
                ui.label(format!("Frame: {}", self.frame_count));
                ui.label(format!("State: {}", display_state.state_text));
            });

            ui.horizontal(|ui| {
                ui.label("Controls: WASD/Arrows=Move, Space=Use, Click=Fire");
            });
        }
    }

    /// Toggle debug overlay
    pub const fn toggle_debug(&mut self) {
        self.show_debug = !self.show_debug;
    }
}

impl Default for DoomPanel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doom_panel_new() {
        let panel = DoomPanel::new();
        assert!(panel.doom.is_none());
        assert!(panel.texture.is_none());
        assert!(panel.show_debug);
        assert_eq!(panel.frame_count, 0);
        assert_eq!(panel.fps, 0.0);
        assert!(panel.prev_keys_down.is_empty());
    }

    #[test]
    fn test_doom_panel_default() {
        let panel = DoomPanel::default();
        assert!(panel.doom.is_none());
        assert!(panel.show_debug);
    }

    #[test]
    fn test_doom_panel_toggle_debug() {
        let mut panel = DoomPanel::new();
        assert!(panel.show_debug);
        panel.toggle_debug();
        assert!(!panel.show_debug);
        panel.toggle_debug();
        assert!(panel.show_debug);
    }
}
