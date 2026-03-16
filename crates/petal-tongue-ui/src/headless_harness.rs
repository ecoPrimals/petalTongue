// SPDX-License-Identifier: AGPL-3.0-only
//! Headless egui harness for UI introspection and testing.
//!
//! Runs `PetalTongueApp` without a display by feeding simulated
//! [`egui::RawInput`] into [`egui::Context::run`], exercising the exact
//! same code path as the real GUI.
//!
//! # Proprioception
//!
//! After each frame, the harness captures a [`FrameIntrospection`] so
//! tests (and production self-checks) can query what the primal is showing
//! without pixel inspection.

use crate::app::PetalTongueApp;
use crate::error::Result;
use bytes::Bytes;
use petal_tongue_core::{FrameIntrospection, MotorCommand, PanelKind};

/// Default virtual screen size for the headless harness (1280x720).
const DEFAULT_WIDTH: f32 = 1280.0;
const DEFAULT_HEIGHT: f32 = 720.0;

/// Headless egui harness for UI testing and introspection.
///
/// Creates an `egui::Context` with no platform backend, then drives
/// `PetalTongueApp::update_headless()` each frame exactly the way
/// `eframe` would.
///
/// By default the awakening overlay is skipped so tests can immediately
/// exercise the main UI panels.
pub struct HeadlessHarness {
    ctx: egui::Context,
    app: PetalTongueApp,
    time: f64,
    screen_width: f32,
    screen_height: f32,
    pending_events: Vec<egui::Event>,
    pending_modifiers: egui::Modifiers,
    frame_history: Vec<FrameIntrospection>,
}

impl HeadlessHarness {
    /// Create a new headless harness with default settings.
    ///
    /// The awakening overlay is automatically skipped.
    pub fn new() -> Result<Self> {
        let app = PetalTongueApp::new_headless()?;

        // Skip the awakening overlay so tests reach the main UI immediately
        app.motor_sender()
            .send(MotorCommand::SetAwakening { enabled: false })
            .ok();

        Ok(Self {
            ctx: egui::Context::default(),
            app,
            time: 0.0,
            screen_width: DEFAULT_WIDTH,
            screen_height: DEFAULT_HEIGHT,
            pending_events: Vec::new(),
            pending_modifiers: egui::Modifiers::NONE,
            frame_history: Vec::new(),
        })
    }

    /// Create a harness with a specific screen size.
    pub fn with_screen_size(width: f32, height: f32) -> Result<Self> {
        let mut harness = Self::new()?;
        harness.screen_width = width;
        harness.screen_height = height;
        Ok(harness)
    }

    /// Run a single frame and capture introspection.
    pub fn run_frame(&mut self) -> &FrameIntrospection {
        self.time += 1.0 / 60.0;

        let mut input = egui::RawInput {
            time: Some(self.time),
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(self.screen_width, self.screen_height),
            )),
            modifiers: self.pending_modifiers,
            ..Default::default()
        };
        self.pending_modifiers = egui::Modifiers::NONE;

        input.events.append(&mut self.pending_events);

        let _output = self.ctx.run(input, |ctx| {
            self.app.update_headless(ctx);
        });

        let introspection = self.app.introspect();
        self.frame_history.push(introspection);
        #[expect(clippy::expect_used, reason = "element was just pushed, last() is always Some")]
        self.frame_history.last().expect("just pushed")
    }

    /// Run multiple frames and return introspections.
    pub fn run_frames(&mut self, n: usize) -> Vec<&FrameIntrospection> {
        let start = self.frame_history.len();
        for _ in 0..n {
            self.run_frame();
        }
        self.frame_history[start..].iter().collect()
    }

    // === Input simulation ===

    /// Simulate a left-click at the given position (queued for next frame).
    pub fn click(&mut self, pos: egui::Pos2) {
        self.pending_events.push(egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        });
        self.pending_events.push(egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        });
    }

    /// Simulate a key press (queued for next frame).
    pub fn key_press(&mut self, key: egui::Key) {
        self.key_press_with_modifiers(key, egui::Modifiers::NONE);
    }

    /// Simulate a key press with modifiers (e.g. Ctrl+D for Ctrl).
    pub fn key_press_with_modifiers(&mut self, key: egui::Key, modifiers: egui::Modifiers) {
        self.pending_modifiers = modifiers;
        self.pending_events.push(egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        });
        self.pending_events.push(egui::Event::Key {
            key,
            physical_key: None,
            pressed: false,
            repeat: false,
            modifiers,
        });
    }

    /// Simulate typing text (queued for next frame).
    pub fn type_text(&mut self, text: &str) {
        self.pending_events.push(egui::Event::Text(text.into()));
    }

    /// Simulate a pointer move to the given position (queued for next frame).
    pub fn pointer_move(&mut self, pos: egui::Pos2) {
        self.pending_events.push(egui::Event::PointerMoved(pos));
    }

    // === Introspection queries ===

    /// Whether a panel of the given kind is currently visible.
    #[must_use]
    pub fn is_panel_visible(&self, kind: PanelKind) -> bool {
        self.last_introspection()
            .is_some_and(|f| f.is_panel_visible(kind))
    }

    /// All currently visible panel kinds.
    #[must_use]
    pub fn visible_panels(&self) -> Vec<PanelKind> {
        self.last_introspection()
            .map(FrameIntrospection::visible_panel_kinds)
            .unwrap_or_default()
    }

    /// Whether a specific data object is currently shown.
    #[must_use]
    pub fn is_showing_data(&self, data_id: &str) -> bool {
        self.last_introspection()
            .is_some_and(|f| f.is_showing_data(data_id))
    }

    /// The total number of frames executed so far.
    #[must_use]
    pub const fn frame_count(&self) -> u64 {
        self.app.frame_count()
    }

    /// The most recent frame introspection, if any.
    #[must_use]
    pub fn last_introspection(&self) -> Option<&FrameIntrospection> {
        self.frame_history.last()
    }

    /// All frame introspections captured so far.
    #[must_use]
    pub fn frame_history(&self) -> &[FrameIntrospection] {
        &self.frame_history
    }

    // === Access to internals ===

    /// Borrow the underlying app (for advanced queries).
    #[must_use]
    pub const fn app(&self) -> &PetalTongueApp {
        &self.app
    }

    /// Mutably borrow the underlying app (for motor commands, etc).
    pub const fn app_mut(&mut self) -> &mut PetalTongueApp {
        &mut self.app
    }

    /// Borrow the egui context.
    #[must_use]
    pub const fn ctx(&self) -> &egui::Context {
        &self.ctx
    }

    /// Tessellate the last frame's output (for visual snapshot tests).
    #[must_use]
    pub fn tessellate(&self) -> Vec<egui::ClippedPrimitive> {
        self.ctx.tessellate(Vec::new(), self.ctx.pixels_per_point())
    }

    /// Tessellate the last frame and render to an RGBA8 pixel buffer.
    ///
    /// Returns the pixel buffer and the screen dimensions used.
    pub fn render_pixels(&self) -> Result<(Bytes, u32, u32)> {
        let primitives = self.tessellate();
        let width = self.screen_width as u32;
        let height = self.screen_height as u32;
        let mut renderer = crate::display::renderer::EguiPixelRenderer::new(width, height);
        let buffer = renderer.render(&primitives)?;
        Ok((buffer, width, height))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headless_harness_render_pixels() {
        let mut harness = HeadlessHarness::new().unwrap();
        harness.run_frame();
        let (buffer, width, height) = harness.render_pixels().unwrap();
        assert_eq!(buffer.len(), (width * height * 4) as usize);
    }
}
