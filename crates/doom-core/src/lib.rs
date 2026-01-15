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
//!
//! # Phase 1.1: WAD Parsing & Map Display
//!
//! We start by loading a real Doom WAD file and displaying the map geometry.
//! This validates our asset loading and rendering capabilities.

use std::collections::HashSet;
use std::path::Path;
use thiserror::Error;

pub mod wad_loader;
pub mod map_renderer;

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

/// Doom instance - Phase 1.1: Real map rendering!
///
/// # Evolution Note
/// We've evolved from test patterns to real Doom maps!
/// This demonstrates test-driven evolution in action.
pub struct DoomInstance {
    width: usize,
    height: usize,
    state: DoomState,
    keys_pressed: HashSet<DoomKey>,
    mouse_x: i32,
    mouse_y: i32,
    frame_count: u64,
    
    // Phase 1.1: Real Doom data!
    wad_data: Option<wad_loader::WadData>,
    current_map: Option<String>,
    map_renderer: Option<map_renderer::MapRenderer>,
}

impl DoomInstance {
    /// Create a new Doom instance
    pub fn new(width: usize, height: usize) -> Result<Self> {
        tracing::info!("Creating Doom instance: {}x{}", width, height);
        
        Ok(Self {
            width,
            height,
            state: DoomState::Uninitialized,
            keys_pressed: HashSet::new(),
            mouse_x: 0,
            mouse_y: 0,
            frame_count: 0,
            wad_data: None,
            current_map: None,
            map_renderer: None,
        })
    }
    
    /// Initialize the Doom engine with a WAD file
    ///
    /// # Phase 1.1: Real WAD loading!
    pub fn init(&mut self) -> Result<()> {
        self.init_with_wad(None::<&Path>)
    }
    
    /// Initialize with a specific WAD file path
    ///
    /// If no path is provided, looks for common locations:
    /// - ./doom1.wad (shareware)
    /// - ./freedoom1.wad (free alternative)
    /// - /usr/share/games/doom/doom1.wad (Linux)
    pub fn init_with_wad<P: AsRef<Path>>(&mut self, wad_path: Option<P>) -> Result<()> {
        tracing::info!("Initializing Doom engine");
        self.state = DoomState::Loading;
        
        // Try to find a WAD file
        let wad_path = if let Some(path) = wad_path {
            path.as_ref().to_path_buf()
        } else {
            self.find_wad_file()?
        };
        
        tracing::info!("Loading WAD: {}", wad_path.display());
        
        // Load the WAD file
        match wad_loader::WadData::load(&wad_path) {
            Ok(wad_data) => {
                tracing::info!("WAD loaded successfully with {} maps", wad_data.maps.len());
                
                // Start with the first map
                if let Some(first_map) = wad_data.first_map() {
                    self.current_map = Some(first_map.name.clone());
                    tracing::info!("Starting map: {}", first_map.name);
                }
                
                // Create map renderer
                self.map_renderer = Some(map_renderer::MapRenderer::new(self.width, self.height));
                
                self.wad_data = Some(wad_data);
                self.state = DoomState::Menu;
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to load WAD: {}", e);
                Err(DoomError::InvalidWad(e.to_string()))
            }
        }
    }
    
    /// Try to find a WAD file in common locations
    fn find_wad_file(&self) -> Result<std::path::PathBuf> {
        let candidates = vec![
            "./doom1.wad",
            "./freedoom1.wad",
            "./DOOM1.WAD",
            "./FREEDOOM1.WAD",
            "/usr/share/games/doom/doom1.wad",
            "/usr/share/games/doom/freedoom1.wad",
            "/usr/local/share/games/doom/doom1.wad",
        ];
        
        for candidate in candidates {
            let path = std::path::Path::new(candidate);
            if path.exists() {
                tracing::info!("Found WAD file: {}", path.display());
                return Ok(path.to_path_buf());
            }
        }
        
        Err(DoomError::WadNotFound(
            "No WAD file found. Please provide doom1.wad or freedoom1.wad".to_string()
        ))
    }
    
    /// Run one game tick
    ///
    /// # Phase 1.1: Real map rendering!
    pub fn tick(&mut self) -> Result<()> {
        if self.state != DoomState::Playing && self.state != DoomState::Menu {
            return Ok(());
        }
        
        self.frame_count += 1;
        
        // Phase 1.1: Render the current map!
        if let (Some(wad_data), Some(map_name), Some(renderer)) = 
            (&self.wad_data, &self.current_map, &mut self.map_renderer) 
        {
            if let Some(map) = wad_data.get_map(map_name) {
                renderer.render(map);
            }
        }
        
        Ok(())
    }
    
    /// Get the current framebuffer (RGBA format)
    ///
    /// # Phase 1.1: Real map rendering!
    pub fn framebuffer(&self) -> &[u8] {
        if let Some(renderer) = &self.map_renderer {
            renderer.framebuffer()
        } else {
            // Fallback: empty buffer
            &[]
        }
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
    
    /// Get the current map name
    pub fn current_map(&self) -> Option<&str> {
        self.current_map.as_deref()
    }
    
    /// Load a specific map
    pub fn load_map(&mut self, map_name: &str) -> Result<()> {
        if let Some(wad_data) = &self.wad_data {
            if wad_data.get_map(map_name).is_some() {
                tracing::info!("Loading map: {}", map_name);
                self.current_map = Some(map_name.to_string());
                Ok(())
            } else {
                Err(DoomError::EngineError(format!("Map {} not found", map_name)))
            }
        } else {
            Err(DoomError::EngineError("No WAD loaded".to_string()))
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

