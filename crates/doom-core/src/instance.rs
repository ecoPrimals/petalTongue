// SPDX-License-Identifier: AGPL-3.0-or-later
//! Doom instance - main game loop and state management.

use crate::error::{DoomError, Result};
use crate::key::DoomKey;
use crate::state::{DoomState, GameStats, ViewMode};
use crate::{map_renderer, raycast_renderer, wad_loader};
use petal_tongue_scene::scene_graph::SceneGraph;
use std::collections::HashSet;
use std::path::Path;

/// Doom instance - Phase 1.2: First-person view!
pub struct DoomInstance {
    width: usize,
    height: usize,
    state: DoomState,
    keys_pressed: HashSet<DoomKey>,
    mouse_x: i32,
    mouse_y: i32,
    mouse_dx: f32,
    frame_count: u64,
    wad_data: Option<wad_loader::WadData>,
    current_map: Option<String>,
    map_renderer: Option<map_renderer::MapRenderer>,
    raycast_renderer: Option<raycast_renderer::RaycastRenderer>,
    first_person_mode: bool,
}

impl DoomInstance {
    /// Create a new Doom instance with the given framebuffer dimensions.
    ///
    /// # Errors
    ///
    /// Returns `DoomError::InitializationFailed` if dimensions are zero.
    pub fn new(width: usize, height: usize) -> Result<Self> {
        tracing::info!("Creating Doom instance: {width}x{height}");

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
            first_person_mode: true,
        })
    }

    /// Initialize the Doom engine, searching for WAD files in common locations.
    ///
    /// # Errors
    ///
    /// Returns `DoomError::WadNotFound` if no WAD file can be located,
    /// or `DoomError::InvalidWad` if the file cannot be parsed.
    pub fn init(&mut self) -> Result<()> {
        self.init_with_wad(None::<&Path>)
    }

    /// Initialize with a specific WAD file path.
    ///
    /// If no path is provided, searches common locations.
    ///
    /// # Errors
    ///
    /// Returns `DoomError::WadNotFound` or `DoomError::InvalidWad` on failure.
    pub fn init_with_wad<P: AsRef<Path>>(&mut self, wad_path: Option<P>) -> Result<()> {
        tracing::info!("Initializing Doom engine");
        self.state = DoomState::Loading;

        let wad_path = if let Some(path) = wad_path {
            path.as_ref().to_path_buf()
        } else {
            Self::find_wad_file()?
        };

        tracing::info!("Loading WAD: {}", wad_path.display());

        match wad_loader::WadData::load(&wad_path) {
            Ok(wad_data) => {
                tracing::info!("WAD loaded successfully with {} maps", wad_data.maps.len());

                if let Some(first_map) = wad_data.first_map() {
                    self.current_map = Some(first_map.name.clone());
                    tracing::info!("Starting map: {}", first_map.name);
                }

                self.map_renderer = Some(map_renderer::MapRenderer::new(self.width, self.height));

                let mut raycast = raycast_renderer::RaycastRenderer::new(self.width, self.height);

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
                tracing::error!("Failed to load WAD: {e}");
                Err(DoomError::InvalidWad(e.to_string()))
            }
        }
    }

    /// Try to find a WAD file in common locations.
    ///
    /// Uses XDG data dirs when available, falls back to well-known paths.
    fn find_wad_file() -> Result<std::path::PathBuf> {
        let mut candidates: Vec<std::path::PathBuf> = vec![
            "./doom1.wad".into(),
            "./freedoom1.wad".into(),
            "./DOOM1.WAD".into(),
            "./FREEDOOM1.WAD".into(),
        ];

        if let Ok(data_home) = std::env::var("XDG_DATA_HOME") {
            candidates.push(std::path::PathBuf::from(&data_home).join("games/doom/doom1.wad"));
            candidates.push(std::path::PathBuf::from(&data_home).join("games/doom/freedoom1.wad"));
        }

        #[cfg(target_family = "unix")]
        {
            candidates.push("/usr/share/games/doom/doom1.wad".into());
            candidates.push("/usr/share/games/doom/freedoom1.wad".into());
            candidates.push("/usr/local/share/games/doom/doom1.wad".into());
        }

        for candidate in candidates {
            if candidate.exists() {
                tracing::info!("Found WAD file: {}", candidate.display());
                return Ok(candidate);
            }
        }

        Err(DoomError::WadNotFound(
            "No WAD file found. Please provide doom1.wad or freedoom1.wad".to_string(),
        ))
    }

    /// Run one game tick, updating player state and rendering.
    ///
    /// # Errors
    ///
    /// Currently infallible but returns `Result` for future evolution.
    pub fn tick(&mut self) -> Result<()> {
        if self.state != DoomState::Playing && self.state != DoomState::Menu {
            return Ok(());
        }

        self.frame_count += 1;
        self.update_player();

        if let (Some(wad_data), Some(map_name)) = (&self.wad_data, &self.current_map)
            && let Some(map) = wad_data.get_map(map_name)
        {
            if self.first_person_mode {
                if let Some(renderer) = &mut self.raycast_renderer {
                    renderer.render(map);
                }
            } else if let Some(renderer) = &mut self.map_renderer {
                renderer.render(map);
            }
        }

        Ok(())
    }

    fn update_player(&mut self) {
        if let Some(renderer) = &mut self.raycast_renderer {
            let move_speed = 6.0;
            let turn_speed = 0.03;

            renderer.rotate(self.mouse_dx * turn_speed);
            self.mouse_dx = 0.0;

            if self.keys_pressed.contains(&DoomKey::Left) {
                renderer.rotate(-turn_speed);
            }
            if self.keys_pressed.contains(&DoomKey::Right) {
                renderer.rotate(turn_speed);
            }
            if self.keys_pressed.contains(&DoomKey::Up) {
                renderer.move_forward(move_speed);
            }
            if self.keys_pressed.contains(&DoomKey::Down) {
                renderer.move_forward(-move_speed);
            }
            if self.keys_pressed.contains(&DoomKey::StrafeLeft) {
                renderer.move_strafe(-move_speed);
            }
            if self.keys_pressed.contains(&DoomKey::StrafeRight) {
                renderer.move_strafe(move_speed);
            }
        }
    }

    /// Render the current frame as a scene graph.
    ///
    /// Every pixel region in the output maps to a `Primitive::Rect` with a
    /// `data_id` so the full frame is traceable.
    #[must_use]
    pub fn render_scene(&self) -> SceneGraph {
        if let (Some(wad_data), Some(map_name)) = (&self.wad_data, &self.current_map)
            && let Some(map) = wad_data.get_map(map_name)
            && self.first_person_mode
            && let Some(renderer) = &self.raycast_renderer
        {
            return renderer.render_to_scene(map);
        }
        SceneGraph::new()
    }

    /// Get the current framebuffer (RGBA format).
    #[must_use]
    pub fn framebuffer(&self) -> &[u8] {
        if self.first_person_mode {
            if let Some(renderer) = &self.raycast_renderer {
                return renderer.framebuffer();
            }
        } else if let Some(renderer) = &self.map_renderer {
            return renderer.framebuffer();
        }
        &[]
    }

    /// Get framebuffer dimensions.
    #[must_use]
    pub const fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// Get current game state.
    #[must_use]
    pub const fn state(&self) -> DoomState {
        self.state
    }

    /// Check if a key is currently pressed.
    #[must_use]
    pub fn is_key_pressed(&self, key: DoomKey) -> bool {
        self.keys_pressed.contains(&key)
    }

    /// Press a key.
    pub fn key_down(&mut self, key: DoomKey) {
        self.keys_pressed.insert(key);
        tracing::trace!("Key down: {key:?}");
    }

    /// Release a key.
    pub fn key_up(&mut self, key: DoomKey) {
        self.keys_pressed.remove(&key);
        tracing::trace!("Key up: {key:?}");
    }

    /// Update mouse position (delta used for turning).
    pub const fn mouse_move(&mut self, x: i32, y: i32) {
        let old_x = self.mouse_x;
        self.mouse_x = x;
        self.mouse_y = y;
        #[expect(
            clippy::cast_precision_loss,
            reason = "mouse deltas are small integers"
        )]
        {
            self.mouse_dx = (x - old_x) as f32;
        }
    }

    /// Toggle between first-person and top-down view.
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

    /// Check if in first-person mode.
    #[must_use]
    pub const fn is_first_person(&self) -> bool {
        self.first_person_mode
    }

    /// Start a new game.
    ///
    /// # Errors
    ///
    /// Currently infallible but returns `Result` for future evolution.
    pub fn new_game(&mut self) -> Result<()> {
        tracing::info!("Starting new game");
        self.state = DoomState::Playing;
        Ok(())
    }

    /// Pause the game.
    pub fn pause(&mut self) {
        if self.state == DoomState::Playing {
            self.state = DoomState::Paused;
            tracing::info!("Game paused");
        }
    }

    /// Resume the game.
    pub fn resume_game(&mut self) {
        if self.state == DoomState::Paused {
            self.state = DoomState::Playing;
            tracing::info!("Game resumed");
        }
    }

    /// Get the current map name.
    #[must_use]
    pub fn current_map(&self) -> Option<&str> {
        self.current_map.as_deref()
    }

    /// Load a specific map by name (e.g. `"E1M1"`).
    ///
    /// # Errors
    ///
    /// Returns `DoomError::EngineError` if the map is not found or no WAD is loaded.
    pub fn load_map(&mut self, map_name: &str) -> Result<()> {
        if let Some(wad_data) = &self.wad_data {
            if wad_data.get_map(map_name).is_some() {
                tracing::info!("Loading map: {map_name}");
                self.current_map = Some(map_name.to_string());
                Ok(())
            } else {
                Err(DoomError::EngineError(format!("Map {map_name} not found")))
            }
        } else {
            Err(DoomError::EngineError("No WAD loaded".to_string()))
        }
    }

    /// Get game statistics for display.
    #[must_use]
    pub fn stats(&self) -> GameStats {
        let (player_x, player_y, player_angle) =
            self.raycast_renderer
                .as_ref()
                .map_or((None, None, None), |renderer| {
                    (
                        Some(renderer.player_x),
                        Some(renderer.player_y),
                        Some(renderer.player_angle),
                    )
                });

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
