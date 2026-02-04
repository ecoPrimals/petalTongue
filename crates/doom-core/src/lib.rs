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

pub mod map_renderer;
pub mod raycast_renderer;
pub mod wad_loader;

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
            DoomKey::Fire => 0x9D,       // Ctrl
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

/// Doom instance - Phase 1.2: First-person view!
///
/// # Evolution Note
/// Phase 1.1: Added real WAD loading and 2D map rendering
/// Phase 1.2: Added first-person raycasting renderer!
pub struct DoomInstance {
    width: usize,
    height: usize,
    state: DoomState,
    keys_pressed: HashSet<DoomKey>,
    mouse_x: i32,
    mouse_y: i32,
    mouse_dx: f32, // Mouse movement for turning
    frame_count: u64,

    // Phase 1.1: Real Doom data!
    wad_data: Option<wad_loader::WadData>,
    current_map: Option<String>,
    map_renderer: Option<map_renderer::MapRenderer>,

    // Phase 1.2: First-person rendering!
    raycast_renderer: Option<raycast_renderer::RaycastRenderer>,
    first_person_mode: bool, // true = first-person, false = top-down
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
            mouse_dx: 0.0,
            frame_count: 0,
            wad_data: None,
            current_map: None,
            map_renderer: None,
            raycast_renderer: None,
            first_person_mode: true, // Start in first-person mode!
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

                // Create renderers
                self.map_renderer = Some(map_renderer::MapRenderer::new(self.width, self.height));

                // Phase 1.2: Create raycasting renderer!
                let mut raycast = raycast_renderer::RaycastRenderer::new(self.width, self.height);

                // Set player start position from map
                if let Some(first_map) = wad_data.first_map() {
                    raycast.set_player_start(first_map);
                    tracing::info!(
                        "Player start: ({}, {}) angle: {}",
                        raycast.player_x,
                        raycast.player_y,
                        raycast.player_angle
                    );
                }

                self.raycast_renderer = Some(raycast);
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
            "No WAD file found. Please provide doom1.wad or freedoom1.wad".to_string(),
        ))
    }

    /// Run one game tick
    ///
    /// # Phase 1.2: First-person + player movement!
    pub fn tick(&mut self) -> Result<()> {
        if self.state != DoomState::Playing && self.state != DoomState::Menu {
            return Ok(());
        }

        self.frame_count += 1;

        // Update player movement based on keys
        self.update_player();

        // Render current map
        if let (Some(wad_data), Some(map_name)) = (&self.wad_data, &self.current_map) {
            if let Some(map) = wad_data.get_map(map_name) {
                if self.first_person_mode {
                    // Phase 1.2: First-person raycasting!
                    if let Some(renderer) = &mut self.raycast_renderer {
                        renderer.render(map);
                    }
                } else {
                    // Phase 1.1: Top-down view
                    if let Some(renderer) = &mut self.map_renderer {
                        renderer.render(map);
                    }
                }
            }
        }

        Ok(())
    }

    /// Update player position/rotation based on input
    ///
    /// # Phase 1.2: Player movement!
    fn update_player(&mut self) {
        if let Some(renderer) = &mut self.raycast_renderer {
            // 🎮 Speed scaled for 60 Hz tick rate (was 35 Hz)
            // Original: 10.0 units/tick × 35 Hz = 350 units/sec
            // Now: 6.0 units/tick × 60 Hz = 360 units/sec (similar feel)
            let move_speed = 6.0; // Units per frame (scaled for 60 Hz)
            let turn_speed = 0.03; // Radians per frame (scaled for 60 Hz)

            // Rotation (mouse)
            renderer.rotate(self.mouse_dx * turn_speed);
            self.mouse_dx = 0.0; // Reset mouse delta

            // Rotation (arrow keys)
            if self.keys_pressed.contains(&DoomKey::Left) {
                renderer.rotate(-turn_speed);
            }
            if self.keys_pressed.contains(&DoomKey::Right) {
                renderer.rotate(turn_speed);
            }

            // Forward/backward
            if self.keys_pressed.contains(&DoomKey::Up) {
                renderer.move_forward(move_speed);
            }
            if self.keys_pressed.contains(&DoomKey::Down) {
                renderer.move_forward(-move_speed);
            }

            // Strafe left/right
            if self.keys_pressed.contains(&DoomKey::StrafeLeft) {
                renderer.move_strafe(-move_speed);
            }
            if self.keys_pressed.contains(&DoomKey::StrafeRight) {
                renderer.move_strafe(move_speed);
            }
        }
    }

    /// Get the current framebuffer (RGBA format)
    ///
    /// # Phase 1.2: First-person or top-down!
    pub fn framebuffer(&self) -> &[u8] {
        if self.first_person_mode {
            if let Some(renderer) = &self.raycast_renderer {
                return renderer.framebuffer();
            }
        } else if let Some(renderer) = &self.map_renderer {
            return renderer.framebuffer();
        }

        // Fallback: empty buffer
        &[]
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
        let old_x = self.mouse_x;
        self.mouse_x = x;
        self.mouse_y = y;

        // Calculate mouse delta for turning
        self.mouse_dx = (x - old_x) as f32;
    }

    /// Toggle between first-person and top-down view
    pub fn toggle_view_mode(&mut self) {
        self.first_person_mode = !self.first_person_mode;
        tracing::info!(
            "View mode: {}",
            if self.first_person_mode {
                "first-person"
            } else {
                "top-down"
            }
        );
    }

    /// Check if in first-person mode
    pub fn is_first_person(&self) -> bool {
        self.first_person_mode
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
                Err(DoomError::EngineError(format!(
                    "Map {} not found",
                    map_name
                )))
            }
        } else {
            Err(DoomError::EngineError("No WAD loaded".to_string()))
        }
    }

    /// Get game statistics for display
    pub fn stats(&self) -> GameStats {
        let (player_x, player_y, player_angle) = if let Some(renderer) = &self.raycast_renderer {
            (
                Some(renderer.player_x),
                Some(renderer.player_y),
                Some(renderer.player_angle),
            )
        } else {
            (None, None, None)
        };

        GameStats {
            state: self.state,
            frame_count: self.frame_count,
            dimensions: (self.width, self.height),
            current_map: self.current_map.clone(),
            view_mode: if self.first_person_mode {
                ViewMode::FirstPerson
            } else {
                ViewMode::TopDown
            },
            player_x,
            player_y,
            player_angle,
        }
    }
}

/// Game statistics for display in UI
#[derive(Debug, Clone)]
pub struct GameStats {
    pub state: DoomState,
    pub frame_count: u64,
    pub dimensions: (usize, usize),
    pub current_map: Option<String>,
    pub view_mode: ViewMode,
    pub player_x: Option<f32>,
    pub player_y: Option<f32>,
    pub player_angle: Option<f32>,
}

/// View mode for rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    TopDown,
    FirstPerson,
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
    #[ignore = "Requires WAD file (doom1.wad or freedoom1.wad) - run with --ignored"]
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
        // Framebuffer is empty until init() is called with a valid WAD
        // This tests the uninitialized state correctly returns empty buffer
        let doom = DoomInstance::new(320, 240).unwrap();
        assert_eq!(
            doom.framebuffer().len(),
            0,
            "Uninitialized instance should have empty framebuffer"
        );
    }

    #[test]
    #[ignore = "Requires WAD file (doom1.wad or freedoom1.wad) - run with --ignored"]
    fn test_framebuffer_size_with_wad() {
        let mut doom = DoomInstance::new(320, 240).unwrap();
        doom.init().unwrap();
        assert_eq!(doom.framebuffer().len(), 320 * 240 * 4);
    }
}
