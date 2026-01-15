//! Doom Core - Doom integration for petalTongue
//!
//! This crate provides the infrastructure for running Doom within petalTongue.
//! It demonstrates petalTongue's platform capabilities through test-driven evolution.
//!
//! # Architecture
//!
//! The Doom integration is designed to expose gaps in petalTongue's architecture:
//! - Panel lifecycle management
//! - Input focus and routing
//! - Performance budgets
//! - Resource coordination
//! - Asset loading
//! - Audio mixing
//!
//! As we implement Doom, we discover and solve these gaps, evolving petalTongue
//! into a robust platform for ANY embedded application.

use std::collections::HashSet;
use thiserror::Error;

/// Doom-specific errors
#[derive(Debug, Error)]
pub enum DoomError {
    #[error("Doom engine initialization failed: {0}")]
    InitializationFailed(String),
    
    #[error("WAD file not found: {0}")]
    WadNotFound(String),
    
    #[error("Invalid WAD file: {0}")]
    InvalidWad(String),
    
    #[error("Doom engine error: {0}")]
    EngineError(String),
}

pub type Result<T> = std::result::Result<T, DoomError>;

/// Doom key codes (mapped from egui)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DoomKey {
    // Movement
    Up,
    Down,
    Left,
    Right,
    StrafeLeft,
    StrafeRight,
    
    // Actions
    Fire,
    Use,
    Run,
    
    // Weapons
    Weapon1,
    Weapon2,
    Weapon3,
    Weapon4,
    Weapon5,
    Weapon6,
    Weapon7,
    
    // Menu
    Enter,
    Escape,
    
    // Other
    Map,
}

impl DoomKey {
    /// Convert to Doom's internal keycode
    pub fn to_doom_code(self) -> i32 {
        match self {
            // Arrow keys
            DoomKey::Up => 0xAE,
            DoomKey::Down => 0xAF,
            DoomKey::Left => 0xAC,
            DoomKey::Right => 0xAD,
            
            // Actions
            DoomKey::Fire => 0x9D,      // Ctrl
            DoomKey::Use => b' ' as i32, // Space
            DoomKey::Run => 0x9E,        // Shift
            
            // Weapons
            DoomKey::Weapon1 => b'1' as i32,
            DoomKey::Weapon2 => b'2' as i32,
            DoomKey::Weapon3 => b'3' as i32,
            DoomKey::Weapon4 => b'4' as i32,
            DoomKey::Weapon5 => b'5' as i32,
            DoomKey::Weapon6 => b'6' as i32,
            DoomKey::Weapon7 => b'7' as i32,
            
            // Strafe
            DoomKey::StrafeLeft => b',' as i32,
            DoomKey::StrafeRight => b'.' as i32,
            
            // Menu
            DoomKey::Enter => 13,
            DoomKey::Escape => 27,
            
            // Other
            DoomKey::Map => b'\t' as i32, // Tab
        }
    }
}

/// Doom game state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoomState {
    Uninitialized,
    Loading,
    Menu,
    Playing,
    Paused,
    Error,
}

/// Mock Doom instance (for now - will be replaced with real implementation)
///
/// # Evolution Note
/// This is intentionally minimal. As we implement, we'll discover what we actually need.
/// The spec will evolve based on real requirements.
pub struct DoomInstance {
    width: usize,
    height: usize,
    state: DoomState,
    framebuffer: Vec<u8>,
    keys_pressed: HashSet<DoomKey>,
    mouse_x: i32,
    mouse_y: i32,
    frame_count: u64,
}

impl DoomInstance {
    /// Create a new Doom instance
    pub fn new(width: usize, height: usize) -> Result<Self> {
        tracing::info!("Creating Doom instance: {}x{}", width, height);
        
        Ok(Self {
            width,
            height,
            state: DoomState::Uninitialized,
            framebuffer: vec![0; width * height * 4], // RGBA
            keys_pressed: HashSet::new(),
            mouse_x: 0,
            mouse_y: 0,
            frame_count: 0,
        })
    }
    
    /// Initialize the Doom engine
    pub fn init(&mut self) -> Result<()> {
        tracing::info!("Initializing Doom engine");
        self.state = DoomState::Loading;
        
        // TODO: Load WAD file
        // TODO: Initialize Doom engine
        // For now: Just set state to menu
        
        // Mock initialization: Fill framebuffer with test pattern
        self.draw_test_pattern();
        
        self.state = DoomState::Menu;
        Ok(())
    }
    
    /// Run one game tick
    pub fn tick(&mut self) -> Result<()> {
        if self.state != DoomState::Playing && self.state != DoomState::Menu {
            return Ok(());
        }
        
        // TODO: Run actual Doom game logic
        // For now: Just update test pattern
        self.frame_count += 1;
        self.update_test_pattern();
        
        Ok(())
    }
    
    /// Get the current framebuffer (RGBA format)
    pub fn framebuffer(&self) -> &[u8] {
        &self.framebuffer
    }
    
    /// Get framebuffer dimensions
    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }
    
    /// Get current game state
    pub fn state(&self) -> DoomState {
        self.state
    }
    
    /// Press a key
    pub fn key_down(&mut self, key: DoomKey) {
        self.keys_pressed.insert(key);
        tracing::trace!("Key down: {:?}", key);
    }
    
    /// Release a key
    pub fn key_up(&mut self, key: DoomKey) {
        self.keys_pressed.remove(&key);
        tracing::trace!("Key up: {:?}", key);
    }
    
    /// Update mouse position
    pub fn mouse_move(&mut self, x: i32, y: i32) {
        self.mouse_x = x;
        self.mouse_y = y;
    }
    
    /// Start a new game
    pub fn new_game(&mut self) -> Result<()> {
        tracing::info!("Starting new game");
        self.state = DoomState::Playing;
        Ok(())
    }
    
    /// Pause the game
    pub fn pause(&mut self) {
        if self.state == DoomState::Playing {
            self.state = DoomState::Paused;
            tracing::info!("Game paused");
        }
    }
    
    /// Resume the game
    pub fn resume(&mut self) {
        if self.state == DoomState::Paused {
            self.state = DoomState::Playing;
            tracing::info!("Game resumed");
        }
    }
    
    // === Mock rendering for development ===
    
    fn draw_test_pattern(&mut self) {
        // Draw a simple gradient pattern to verify rendering works
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (y * self.width + x) * 4;
                self.framebuffer[idx] = (x * 255 / self.width) as u8;     // R
                self.framebuffer[idx + 1] = (y * 255 / self.height) as u8; // G
                self.framebuffer[idx + 2] = 128;                           // B
                self.framebuffer[idx + 3] = 255;                           // A
            }
        }
    }
    
    fn update_test_pattern(&mut self) {
        // Animate the test pattern
        let offset = (self.frame_count % 255) as u8;
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (y * self.width + x) * 4;
                self.framebuffer[idx + 2] = offset; // Animate blue channel
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_doom_instance_creation() {
        let doom = DoomInstance::new(640, 480).unwrap();
        assert_eq!(doom.dimensions(), (640, 480));
        assert_eq!(doom.state(), DoomState::Uninitialized);
    }
    
    #[test]
    fn test_doom_initialization() {
        let mut doom = DoomInstance::new(640, 480).unwrap();
        doom.init().unwrap();
        assert_eq!(doom.state(), DoomState::Menu);
    }
    
    #[test]
    fn test_key_input() {
        let mut doom = DoomInstance::new(640, 480).unwrap();
        doom.key_down(DoomKey::Up);
        assert!(doom.keys_pressed.contains(&DoomKey::Up));
        
        doom.key_up(DoomKey::Up);
        assert!(!doom.keys_pressed.contains(&DoomKey::Up));
    }
    
    #[test]
    fn test_framebuffer_size() {
        let doom = DoomInstance::new(320, 240).unwrap();
        assert_eq!(doom.framebuffer().len(), 320 * 240 * 4);
    }
}

