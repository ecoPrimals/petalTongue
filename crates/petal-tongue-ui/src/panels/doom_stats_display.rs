// SPDX-License-Identifier: AGPL-3.0-only
//! Pure display logic for Doom Stats Panel
//!
//! Extracted formatting and display state preparation for testability.

use doom_core::{DoomState, GameStats, ViewMode};

/// Maps DoomState enum variants to display strings.
#[must_use]
pub const fn format_doom_state(state: &DoomState) -> &'static str {
    match state {
        DoomState::Uninitialized => "❌ Uninitialized",
        DoomState::Loading => "⏳ Loading",
        DoomState::Menu => "📋 Menu",
        DoomState::Playing => "▶️ Playing",
        DoomState::Paused => "⏸️ Paused",
        DoomState::Error => "❌ Error",
    }
}

/// Maps ViewMode enum variants to display strings.
#[must_use]
pub const fn format_view_mode(mode: &ViewMode) -> &'static str {
    match mode {
        ViewMode::FirstPerson => "First-Person (3D)",
        ViewMode::TopDown => "Top-Down (2D)",
    }
}

/// Formats player position and angle for display.
#[must_use]
pub fn format_player_position(x: f32, y: f32, angle: f32) -> String {
    format!(
        "  X: {x:.0}\n  Y: {y:.0}\n  Angle: {:.0}°",
        angle.to_degrees()
    )
}

/// Pre-computed display state for the Doom Stats panel.
#[derive(Debug, Clone)]
pub struct DoomStatsDisplayState {
    pub state_text: &'static str,
    pub map_name: String,
    pub view_mode: &'static str,
    pub position: Option<String>,
    pub fps: Option<f32>,
    pub frame_count: u64,
}

/// Prepares display state from raw game stats.
#[must_use]
pub fn prepare_doom_stats_display(stats: &GameStats) -> DoomStatsDisplayState {
    let state_text = format_doom_state(&stats.state);
    let map_name = stats
        .current_map
        .as_ref()
        .map_or_else(|| "None".to_string(), |m| format!("{m} (Hangar)"));
    let view_mode = format_view_mode(&stats.view_mode);
    let position = match (stats.player_x, stats.player_y, stats.player_angle) {
        (Some(x), Some(y), Some(angle)) => Some(format_player_position(x, y, angle)),
        _ => None,
    };

    DoomStatsDisplayState {
        state_text,
        map_name,
        view_mode,
        position,
        fps: None, // FPS not available in GameStats; could be derived over time
        frame_count: stats.frame_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_doom_state() {
        assert_eq!(
            format_doom_state(&DoomState::Uninitialized),
            "❌ Uninitialized"
        );
        assert_eq!(format_doom_state(&DoomState::Loading), "⏳ Loading");
        assert_eq!(format_doom_state(&DoomState::Menu), "📋 Menu");
        assert_eq!(format_doom_state(&DoomState::Playing), "▶️ Playing");
        assert_eq!(format_doom_state(&DoomState::Paused), "⏸️ Paused");
        assert_eq!(format_doom_state(&DoomState::Error), "❌ Error");
    }

    #[test]
    fn test_format_view_mode() {
        assert_eq!(
            format_view_mode(&ViewMode::FirstPerson),
            "First-Person (3D)"
        );
        assert_eq!(format_view_mode(&ViewMode::TopDown), "Top-Down (2D)");
    }

    #[test]
    fn test_format_player_position() {
        let s = format_player_position(100.0, 200.0, std::f32::consts::PI);
        assert!(s.contains("X: 100"));
        assert!(s.contains("Y: 200"));
        assert!(s.contains("Angle: 180"));
    }

    #[test]
    fn test_prepare_doom_stats_display_with_position() {
        let stats = GameStats {
            state: DoomState::Playing,
            frame_count: 42,
            dimensions: (640, 480),
            current_map: Some("E1M1".to_string()),
            view_mode: ViewMode::FirstPerson,
            player_x: Some(100.0),
            player_y: Some(200.0),
            player_angle: Some(0.0),
        };
        let display = prepare_doom_stats_display(&stats);
        assert_eq!(display.state_text, "▶️ Playing");
        assert_eq!(display.map_name, "E1M1 (Hangar)");
        assert_eq!(display.view_mode, "First-Person (3D)");
        assert!(display.position.is_some());
        assert_eq!(display.frame_count, 42);
        assert!(display.fps.is_none());
    }

    #[test]
    fn test_prepare_doom_stats_display_no_map_no_position() {
        let stats = GameStats {
            state: DoomState::Uninitialized,
            frame_count: 0,
            dimensions: (320, 240),
            current_map: None,
            view_mode: ViewMode::TopDown,
            player_x: None,
            player_y: None,
            player_angle: None,
        };
        let display = prepare_doom_stats_display(&stats);
        assert_eq!(display.state_text, "❌ Uninitialized");
        assert_eq!(display.map_name, "None");
        assert_eq!(display.view_mode, "Top-Down (2D)");
        assert!(display.position.is_none());
        assert_eq!(display.frame_count, 0);
    }
}
