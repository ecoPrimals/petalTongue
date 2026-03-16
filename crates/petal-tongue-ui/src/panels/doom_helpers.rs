// SPDX-License-Identifier: AGPL-3.0-or-later
//! Doom Panel - Pure helper functions (headless-testable, no egui rendering)

use doom_core::{DoomInstance, DoomKey};
use egui::Key;
use std::collections::HashSet;
use std::hash::BuildHasher;

/// Map egui keys to Doom keys (static helper).
///
/// Modern FPS controls:
/// - WASD: W/S move, A/D strafe (modern)
/// - Arrows: Up/Down move, Left/Right turn (classic)
#[must_use]
pub const fn egui_to_doom_key_static(key: Key) -> Option<DoomKey> {
    Some(match key {
        // Arrow keys: Classic Doom controls (move + turn)
        Key::ArrowUp => DoomKey::Up,
        Key::ArrowDown => DoomKey::Down,
        Key::ArrowLeft => DoomKey::Left,
        Key::ArrowRight => DoomKey::Right,

        // WASD: Modern FPS controls (move + strafe)
        Key::W => DoomKey::Up,
        Key::S => DoomKey::Down,
        Key::A => DoomKey::StrafeLeft,  // Strafe, not turn!
        Key::D => DoomKey::StrafeRight, // Strafe, not turn!

        // Q/E: Alternative strafe (for those who prefer it)
        Key::Q => DoomKey::StrafeLeft,
        Key::E => DoomKey::StrafeRight,

        // Actions
        Key::Space => DoomKey::Use,
        Key::Enter => DoomKey::Enter,
        Key::Escape => DoomKey::Escape,

        // Weapons
        Key::Num1 => DoomKey::Weapon1,
        Key::Num2 => DoomKey::Weapon2,
        Key::Num3 => DoomKey::Weapon3,
        Key::Num4 => DoomKey::Weapon4,
        Key::Num5 => DoomKey::Weapon5,

        // Other
        Key::Tab => DoomKey::Map,

        _ => return None,
    })
}

/// Compute FPS from frame count and elapsed time.
#[must_use]
pub fn compute_fps(frames_since_update: u32, elapsed_secs: f64) -> f32 {
    if elapsed_secs <= 0.0 {
        return 0.0;
    }
    frames_since_update as f32 / elapsed_secs as f32
}

/// Compute key state diff: (newly_pressed, newly_released).
#[must_use]
pub fn compute_keys_diff<S: BuildHasher>(
    prev: &HashSet<Key, S>,
    current: &HashSet<Key, S>,
) -> (Vec<Key>, Vec<Key>) {
    let newly_pressed: Vec<Key> = current
        .iter()
        .filter(|k| !prev.contains(k))
        .copied()
        .collect();
    let newly_released: Vec<Key> = prev
        .iter()
        .filter(|k| !current.contains(k))
        .copied()
        .collect();
    (newly_pressed, newly_released)
}

/// Precomputed display state for the Doom panel (used by thin render methods).
#[derive(Debug, Clone)]
pub struct DoomDisplayState {
    /// Whether Doom is initialized and ready.
    pub initialized: bool,
    /// Current FPS.
    pub fps: f32,
    /// Whether to show the debug overlay.
    pub show_debug: bool,
    /// Formatted doom state string (e.g. "Playing", "Menu").
    pub state_text: String,
}

/// Prepare display state from Doom instance and panel parameters.
#[must_use]
pub fn prepare_doom_display(
    doom: Option<&DoomInstance>,
    fps: f32,
    show_debug: bool,
) -> DoomDisplayState {
    let (initialized, state_text) = doom.map_or_else(
        || (false, String::new()),
        |d| (true, format!("{:?}", d.state())),
    );
    DoomDisplayState {
        initialized,
        fps,
        show_debug,
        state_text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_egui_to_doom_key_arrows() {
        assert_eq!(egui_to_doom_key_static(Key::ArrowUp), Some(DoomKey::Up));
        assert_eq!(egui_to_doom_key_static(Key::ArrowDown), Some(DoomKey::Down));
        assert_eq!(egui_to_doom_key_static(Key::ArrowLeft), Some(DoomKey::Left));
        assert_eq!(
            egui_to_doom_key_static(Key::ArrowRight),
            Some(DoomKey::Right)
        );
    }

    #[test]
    fn test_egui_to_doom_key_wasd() {
        assert_eq!(egui_to_doom_key_static(Key::W), Some(DoomKey::Up));
        assert_eq!(egui_to_doom_key_static(Key::S), Some(DoomKey::Down));
        assert_eq!(egui_to_doom_key_static(Key::A), Some(DoomKey::StrafeLeft));
        assert_eq!(egui_to_doom_key_static(Key::D), Some(DoomKey::StrafeRight));
    }

    #[test]
    fn test_egui_to_doom_key_actions() {
        assert_eq!(egui_to_doom_key_static(Key::Space), Some(DoomKey::Use));
        assert_eq!(egui_to_doom_key_static(Key::Enter), Some(DoomKey::Enter));
        assert_eq!(egui_to_doom_key_static(Key::Escape), Some(DoomKey::Escape));
        assert_eq!(egui_to_doom_key_static(Key::Tab), Some(DoomKey::Map));
    }

    #[test]
    fn test_egui_to_doom_key_weapons() {
        assert_eq!(egui_to_doom_key_static(Key::Num1), Some(DoomKey::Weapon1));
        assert_eq!(egui_to_doom_key_static(Key::Num2), Some(DoomKey::Weapon2));
        assert_eq!(egui_to_doom_key_static(Key::Num3), Some(DoomKey::Weapon3));
        assert_eq!(egui_to_doom_key_static(Key::Num4), Some(DoomKey::Weapon4));
        assert_eq!(egui_to_doom_key_static(Key::Num5), Some(DoomKey::Weapon5));
    }

    #[test]
    fn test_egui_to_doom_key_qe() {
        assert_eq!(egui_to_doom_key_static(Key::Q), Some(DoomKey::StrafeLeft));
        assert_eq!(egui_to_doom_key_static(Key::E), Some(DoomKey::StrafeRight));
    }

    #[test]
    fn test_egui_to_doom_key_unmapped() {
        assert_eq!(egui_to_doom_key_static(Key::F), None);
        assert_eq!(egui_to_doom_key_static(Key::B), None);
        assert_eq!(egui_to_doom_key_static(Key::Num0), None);
    }

    #[test]
    fn test_compute_fps_zero_frames() {
        assert!((compute_fps(0, 1.0) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_compute_fps_normal() {
        let fps = compute_fps(60, 1.0);
        assert!((fps - 60.0).abs() < 0.001);
    }

    #[test]
    fn test_compute_fps_half_second() {
        let fps = compute_fps(30, 0.5);
        assert!((fps - 60.0).abs() < 0.001);
    }

    #[test]
    fn test_compute_fps_zero_elapsed() {
        assert!((compute_fps(60, 0.0) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_compute_fps_negative_elapsed() {
        assert!((compute_fps(60, -1.0) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_compute_fps_edge_small_elapsed() {
        let fps = compute_fps(1, 0.016); // ~60fps
        assert!(fps > 50.0 && fps < 70.0);
    }

    #[test]
    fn test_compute_keys_diff_empty_sets() {
        let prev: HashSet<Key> = HashSet::new();
        let current: HashSet<Key> = HashSet::new();
        let (pressed, released) = compute_keys_diff(&prev, &current);
        assert!(pressed.is_empty());
        assert!(released.is_empty());
    }

    #[test]
    fn test_compute_keys_diff_new_keys() {
        let prev: HashSet<Key> = HashSet::new();
        let mut current = HashSet::new();
        current.insert(Key::W);
        current.insert(Key::A);
        let (pressed, released) = compute_keys_diff(&prev, &current);
        assert_eq!(pressed.len(), 2);
        assert!(pressed.contains(&Key::W));
        assert!(pressed.contains(&Key::A));
        assert!(released.is_empty());
    }

    #[test]
    fn test_compute_keys_diff_released_keys() {
        let mut prev = HashSet::new();
        prev.insert(Key::W);
        prev.insert(Key::S);
        let current: HashSet<Key> = HashSet::new();
        let (pressed, released) = compute_keys_diff(&prev, &current);
        assert!(pressed.is_empty());
        assert_eq!(released.len(), 2);
        assert!(released.contains(&Key::W));
        assert!(released.contains(&Key::S));
    }

    #[test]
    fn test_compute_keys_diff_mixed() {
        let mut prev = HashSet::new();
        prev.insert(Key::W);
        prev.insert(Key::S);
        let mut current = HashSet::new();
        current.insert(Key::W);
        current.insert(Key::A);
        let (pressed, released) = compute_keys_diff(&prev, &current);
        assert_eq!(pressed.len(), 1);
        assert!(pressed.contains(&Key::A));
        assert_eq!(released.len(), 1);
        assert!(released.contains(&Key::S));
    }

    #[test]
    fn test_compute_keys_diff_unchanged() {
        let mut prev = HashSet::new();
        prev.insert(Key::W);
        prev.insert(Key::A);
        let mut current = HashSet::new();
        current.insert(Key::W);
        current.insert(Key::A);
        let (pressed, released) = compute_keys_diff(&prev, &current);
        assert!(pressed.is_empty());
        assert!(released.is_empty());
    }

    #[test]
    fn test_prepare_doom_display_none() {
        let ds = prepare_doom_display(None, 60.0, true);
        assert!(!ds.initialized);
        assert_eq!(ds.fps, 60.0);
        assert!(ds.show_debug);
        assert!(ds.state_text.is_empty());
    }

    #[test]
    fn test_prepare_doom_display_some() {
        let doom = DoomInstance::new(320, 240).unwrap();
        let ds = prepare_doom_display(Some(&doom), 30.0, false);
        assert!(ds.initialized);
        assert_eq!(ds.fps, 30.0);
        assert!(!ds.show_debug);
        assert!(ds.state_text.contains("Uninitialized"));
    }

    #[test]
    fn test_prepare_doom_display_show_debug_variants() {
        let doom = DoomInstance::new(320, 240).unwrap();
        let ds_on = prepare_doom_display(Some(&doom), 0.0, true);
        let ds_off = prepare_doom_display(Some(&doom), 0.0, false);
        assert!(ds_on.show_debug);
        assert!(!ds_off.show_debug);
    }
}
