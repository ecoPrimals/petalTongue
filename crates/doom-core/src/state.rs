// SPDX-License-Identifier: AGPL-3.0-or-later
//! Doom game state and view mode types.

/// Doom game state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoomState {
    Uninitialized,
    Loading,
    Menu,
    Playing,
    Paused,
    Error,
}

/// View mode for rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    TopDown,
    FirstPerson,
}

/// Game statistics for display in UI.
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
