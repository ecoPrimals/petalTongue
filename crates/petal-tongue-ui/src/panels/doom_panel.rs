//! Doom Panel - Embeds Doom within petalTongue
//!
//! This is our first test of the panel system. As we implement this,
//! we'll discover gaps in petalTongue's architecture and evolve to fill them.

use doom_core::{DoomInstance, DoomKey, DoomState, Result};
use egui::{ColorImage, TextureHandle, Ui, Key, Vec2};
use std::time::Instant;
use std::collections::HashSet;

/// Panel that embeds Doom
pub struct DoomPanel {
    /// Doom game instance
    doom: Option<DoomInstance>,
    
    /// egui texture for rendering
    texture: Option<TextureHandle>,
    
    /// Last update time (for frame timing)
    last_update: Instant,
    
    /// Target frame time (35 FPS = ~28.57ms per frame)
    target_frame_time_ms: f32,
    
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
    pub fn new() -> Self {
        Self {
            doom: None,
            texture: None,
            last_update: Instant::now(),
            target_frame_time_ms: 1000.0 / 35.0, // 35 FPS Doom tick rate
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
            let elapsed = self.last_update.elapsed();
            
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
        if fps_elapsed.as_secs_f32() >= 1.0 {
            self.fps = self.frames_since_fps_update as f32 / fps_elapsed.as_secs_f32();
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
            ui.colored_label(egui::Color32::RED, format!("Doom initialization failed: {}", e));
            return;
        }
        
        // Update game state
        self.update();
        
        // Get framebuffer from Doom
        if let Some(doom) = &self.doom {
            let (width, height) = doom.dimensions();
            let framebuffer = doom.framebuffer();
            
            // Convert to egui ColorImage
            let color_image = ColorImage::from_rgba_unmultiplied(
                [width, height],
                framebuffer,
            );
            
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
                        egui::vec2(width as f32, height as f32)
                    ))
                    .sense(egui::Sense::click_and_drag()) // Make it sense ALL input
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
                Key::W, Key::ArrowUp,
                Key::S, Key::ArrowDown,
                Key::A, Key::ArrowLeft,
                Key::D, Key::ArrowRight,
                Key::Q, Key::E,
                Key::Space, Key::Enter, Key::Escape,
                Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5,
                Key::Tab,
            ];
            
            // Build current keys_down set
            let current_keys: HashSet<Key> = keys_to_check.iter()
                .filter(|k| i.keys_down.contains(k))
                .copied()
                .collect();
            
            // Send key_down for newly pressed keys
            for key in &current_keys {
                if !self.prev_keys_down.contains(key) {
                    if let Some(doom_key) = Self::egui_to_doom_key_static(*key) {
                        doom.key_down(doom_key);
                        tracing::debug!("🎮 Key DOWN: {:?}", key);
                    }
                }
            }
            
            // Send key_up for newly released keys
            for key in &self.prev_keys_down {
                if !current_keys.contains(key) {
                    if let Some(doom_key) = Self::egui_to_doom_key_static(*key) {
                        doom.key_up(doom_key);
                        tracing::debug!("🎮 Key UP: {:?}", key);
                    }
                }
            }
            
            // Update previous state
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
    
    /// Map egui keys to Doom keys (static helper)
    fn egui_to_doom_key_static(key: Key) -> Option<DoomKey> {
        Some(match key {
            Key::W | Key::ArrowUp => DoomKey::Up,
            Key::S | Key::ArrowDown => DoomKey::Down,
            Key::A | Key::ArrowLeft => DoomKey::Left,
            Key::D | Key::ArrowRight => DoomKey::Right,
            Key::Q => DoomKey::StrafeLeft,
            Key::E => DoomKey::StrafeRight,
            Key::Space => DoomKey::Use,
            Key::Enter => DoomKey::Enter,
            Key::Escape => DoomKey::Escape,
            Key::Num1 => DoomKey::Weapon1,
            Key::Num2 => DoomKey::Weapon2,
            Key::Num3 => DoomKey::Weapon3,
            Key::Num4 => DoomKey::Weapon4,
            Key::Num5 => DoomKey::Weapon5,
            Key::Tab => DoomKey::Map,
            _ => return None,
        })
    }
    
    /// Render debug overlay
    fn render_debug_overlay(&self, ui: &mut Ui) {
        if let Some(doom) = &self.doom {
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(format!("FPS: {:.1}", self.fps));
                ui.label(format!("Frame: {}", self.frame_count));
                ui.label(format!("State: {:?}", doom.state()));
            });
            
            ui.horizontal(|ui| {
                ui.label("Controls: WASD/Arrows=Move, Space=Use, Click=Fire");
            });
        }
    }
    
    /// Toggle debug overlay
    pub fn toggle_debug(&mut self) {
        self.show_debug = !self.show_debug;
    }
}

impl Default for DoomPanel {
    fn default() -> Self {
        Self::new()
    }
}

