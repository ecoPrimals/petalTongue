// SPDX-License-Identifier: AGPL-3.0-or-later
//! Doom-core integration tests.

use crate::error::DoomError;
use crate::instance::DoomInstance;
use crate::key::DoomKey;
use crate::state::{DoomState, GameStats, ViewMode};

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
    assert!(doom.is_key_pressed(DoomKey::Up));

    doom.key_up(DoomKey::Up);
    assert!(!doom.is_key_pressed(DoomKey::Up));
}

#[test]
fn test_framebuffer_size() {
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

#[test]
fn test_doom_key_to_code() {
    assert_eq!(DoomKey::Up.to_doom_code(), 0xAE);
    assert_eq!(DoomKey::Fire.to_doom_code(), 0x9D);
    assert_eq!(DoomKey::Enter.to_doom_code(), 13);
    assert_eq!(DoomKey::Escape.to_doom_code(), 27);
}

#[test]
fn test_doom_state_transitions() {
    let mut doom = DoomInstance::new(320, 240).unwrap();
    assert_eq!(doom.state(), DoomState::Uninitialized);

    doom.new_game().unwrap();
    assert_eq!(doom.state(), DoomState::Playing);

    doom.pause();
    assert_eq!(doom.state(), DoomState::Paused);

    doom.resume_game();
    assert_eq!(doom.state(), DoomState::Playing);
}

#[test]
fn test_view_mode_toggle() {
    let mut doom = DoomInstance::new(320, 240).unwrap();
    assert!(doom.is_first_person());
    doom.toggle_view_mode();
    assert!(!doom.is_first_person());
    doom.toggle_view_mode();
    assert!(doom.is_first_person());
}

#[test]
fn test_mouse_move() {
    let mut doom = DoomInstance::new(320, 240).unwrap();
    doom.mouse_move(100, 50);
    doom.mouse_move(150, 60);
    doom.tick().unwrap();
}

#[test]
fn test_game_stats() {
    let doom = DoomInstance::new(320, 240).unwrap();
    let stats = doom.stats();
    assert_eq!(stats.state, DoomState::Uninitialized);
    assert_eq!(stats.dimensions, (320, 240));
    assert_eq!(stats.frame_count, 0);
    assert!(stats.player_x.is_none());
}

#[test]
fn test_load_map_no_wad() {
    let mut doom = DoomInstance::new(320, 240).unwrap();
    let result = doom.load_map("E1M1");
    assert!(result.is_err());
}

#[test]
fn test_view_mode_enum() {
    assert_eq!(
        std::mem::discriminant(&ViewMode::FirstPerson),
        std::mem::discriminant(&ViewMode::FirstPerson)
    );
    assert_ne!(
        std::mem::discriminant(&ViewMode::FirstPerson),
        std::mem::discriminant(&ViewMode::TopDown)
    );
}

#[test]
fn test_game_stats_clone() {
    let stats = GameStats {
        state: DoomState::Playing,
        frame_count: 100,
        dimensions: (320, 240),
        current_map: Some("E1M1".to_string()),
        view_mode: ViewMode::FirstPerson,
        player_x: Some(100.0),
        player_y: Some(50.0),
        player_angle: Some(1.57),
    };
    let cloned = stats.clone();
    assert_eq!(cloned.state, stats.state);
    assert_eq!(cloned.frame_count, stats.frame_count);
    assert_eq!(cloned.dimensions, stats.dimensions);
    assert_eq!(cloned.current_map, stats.current_map);
    assert_eq!(cloned.view_mode, stats.view_mode);
    assert_eq!(cloned.player_x, stats.player_x);
}

#[test]
fn test_view_mode_variants() {
    assert_eq!(ViewMode::TopDown, ViewMode::TopDown);
    assert_eq!(ViewMode::FirstPerson, ViewMode::FirstPerson);
    assert_ne!(ViewMode::TopDown, ViewMode::FirstPerson);
}

#[test]
fn test_game_stats_without_raycast() {
    let doom = DoomInstance::new(320, 240).unwrap();
    let stats = doom.stats();
    assert!(stats.player_x.is_none());
    assert!(stats.player_y.is_none());
    assert!(stats.player_angle.is_none());
}

#[test]
fn test_doom_key_hash_set() {
    use std::collections::HashSet;
    let mut keys = HashSet::new();
    keys.insert(DoomKey::Up);
    keys.insert(DoomKey::Fire);
    assert!(keys.contains(&DoomKey::Up));
    assert!(!keys.contains(&DoomKey::Down));
}

#[test]
fn test_doom_key_all_codes() {
    assert_eq!(DoomKey::Down.to_doom_code(), 0xAF);
    assert_eq!(DoomKey::Left.to_doom_code(), 0xAC);
    assert_eq!(DoomKey::Right.to_doom_code(), 0xAD);
    assert_eq!(DoomKey::StrafeLeft.to_doom_code(), i32::from(b','));
    assert_eq!(DoomKey::StrafeRight.to_doom_code(), i32::from(b'.'));
    assert_eq!(DoomKey::Use.to_doom_code(), i32::from(b' '));
    assert_eq!(DoomKey::Run.to_doom_code(), 0x9E);
    assert_eq!(DoomKey::Weapon1.to_doom_code(), i32::from(b'1'));
    assert_eq!(DoomKey::Weapon7.to_doom_code(), i32::from(b'7'));
    assert_eq!(DoomKey::Map.to_doom_code(), i32::from(b'\t'));
}

#[test]
fn test_doom_error_display() {
    let e = DoomError::WadNotFound("test".to_string());
    assert!(e.to_string().contains("test"));
    let e2 = DoomError::InvalidWad("bad".to_string());
    assert!(e2.to_string().contains("bad"));
    let e3 = DoomError::EngineError("err".to_string());
    assert!(e3.to_string().contains("err"));
}

#[test]
fn test_doom_state_variants() {
    assert!(matches!(DoomState::Uninitialized, DoomState::Uninitialized));
    assert!(matches!(DoomState::Loading, DoomState::Loading));
    assert!(matches!(DoomState::Menu, DoomState::Menu));
    assert!(matches!(DoomState::Playing, DoomState::Playing));
    assert!(matches!(DoomState::Paused, DoomState::Paused));
    assert!(matches!(DoomState::Error, DoomState::Error));
}

#[test]
fn test_doom_error_initialization_failed() {
    let e = DoomError::InitializationFailed("msg".to_string());
    assert!(e.to_string().contains("msg"));
}

#[test]
fn test_render_scene_empty_when_uninitialized() {
    let doom = DoomInstance::new(320, 240).unwrap();
    let scene = doom.render_scene();
    assert_eq!(scene.node_count(), 1);
}

#[test]
fn test_pause_when_not_playing_no_effect() {
    let mut doom = DoomInstance::new(320, 240).unwrap();
    doom.pause();
    assert_eq!(doom.state(), DoomState::Uninitialized);
}

#[test]
fn test_resume_when_not_paused_no_effect() {
    let mut doom = DoomInstance::new(320, 240).unwrap();
    doom.resume_game();
    assert_eq!(doom.state(), DoomState::Uninitialized);
}

#[test]
fn test_tick_when_uninitialized_returns_ok() {
    let mut doom = DoomInstance::new(320, 240).unwrap();
    assert!(doom.tick().is_ok());
}

#[test]
fn test_load_map_not_found() {
    let wad_bytes = create_minimal_wad_bytes();
    let path = std::env::temp_dir().join("petaltongue_doom_loadmap_test2.wad");
    std::fs::write(&path, &wad_bytes).unwrap();
    let mut doom = DoomInstance::new(320, 240).unwrap();
    doom.init_with_wad(Some(&path)).unwrap();
    std::fs::remove_file(&path).ok();
    let result = doom.load_map("NONEXISTENT");
    assert!(result.is_err());
}

#[test]
fn test_view_mode_top_down_after_toggle() {
    let mut doom = DoomInstance::new(320, 240).unwrap();
    doom.toggle_view_mode();
    assert!(!doom.is_first_person());
    let stats = doom.stats();
    assert_eq!(stats.view_mode, ViewMode::TopDown);
}

#[test]
fn test_game_stats_view_mode() {
    let doom = DoomInstance::new(320, 240).unwrap();
    let stats = doom.stats();
    assert_eq!(stats.view_mode, ViewMode::FirstPerson);
}

#[test]
fn test_load_map_with_wad() {
    let wad_bytes = create_minimal_wad_bytes();
    let path = std::env::temp_dir().join("petaltongue_doom_loadmap_test.wad");
    std::fs::write(&path, &wad_bytes).unwrap();
    let mut doom = DoomInstance::new(320, 240).unwrap();
    doom.init_with_wad(Some(&path)).unwrap();
    std::fs::remove_file(&path).ok();
    assert!(doom.load_map("E1M1").is_ok());
    assert_eq!(doom.current_map(), Some("E1M1"));
}

#[test]
fn test_tick_when_playing() {
    let wad_bytes = create_minimal_wad_bytes();
    let path = std::env::temp_dir().join("petaltongue_doom_tick_test.wad");
    std::fs::write(&path, &wad_bytes).unwrap();
    let mut doom = DoomInstance::new(64, 64).unwrap();
    doom.init_with_wad(Some(&path)).unwrap();
    std::fs::remove_file(&path).ok();
    doom.new_game().unwrap();
    assert!(doom.tick().is_ok());
    assert!(doom.tick().is_ok());
}

#[test]
fn test_tick_when_menu() {
    let wad_bytes = create_minimal_wad_bytes();
    let path = std::env::temp_dir().join("petaltongue_doom_tick_menu_test.wad");
    std::fs::write(&path, &wad_bytes).unwrap();
    let mut doom = DoomInstance::new(64, 64).unwrap();
    doom.init_with_wad(Some(&path)).unwrap();
    std::fs::remove_file(&path).ok();
    assert_eq!(doom.state(), DoomState::Menu);
    assert!(doom.tick().is_ok());
}

#[test]
fn test_render_scene_with_wad() {
    let wad_bytes = create_minimal_wad_bytes();
    let path = std::env::temp_dir().join("petaltongue_doom_render_test.wad");
    std::fs::write(&path, &wad_bytes).unwrap();
    let mut doom = DoomInstance::new(64, 64).unwrap();
    doom.init_with_wad(Some(&path)).unwrap();
    std::fs::remove_file(&path).ok();
    doom.new_game().unwrap();
    let scene = doom.render_scene();
    assert!(scene.node_count() > 1);
}

#[test]
fn test_framebuffer_map_view() {
    let wad_bytes = create_minimal_wad_bytes();
    let path = std::env::temp_dir().join("petaltongue_doom_fb_test.wad");
    std::fs::write(&path, &wad_bytes).unwrap();
    let mut doom = DoomInstance::new(64, 64).unwrap();
    doom.init_with_wad(Some(&path)).unwrap();
    std::fs::remove_file(&path).ok();
    doom.toggle_view_mode();
    assert!(!doom.is_first_person());
    let fb = doom.framebuffer();
    assert_eq!(fb.len(), 64 * 64 * 4);
}

#[test]
fn test_stats_with_raycast() {
    let wad_bytes = create_minimal_wad_bytes();
    let path = std::env::temp_dir().join("petaltongue_doom_stats_test.wad");
    std::fs::write(&path, &wad_bytes).unwrap();
    let mut doom = DoomInstance::new(64, 64).unwrap();
    doom.init_with_wad(Some(&path)).unwrap();
    std::fs::remove_file(&path).ok();
    let stats = doom.stats();
    assert!(stats.player_x.is_some());
    assert!(stats.player_y.is_some());
    assert!(stats.player_angle.is_some());
}

#[test]
fn test_init_with_nonexistent_wad_path() {
    let mut doom = DoomInstance::new(320, 240).unwrap();
    let path = std::path::Path::new("/nonexistent/doom1.wad");
    let result = doom.init_with_wad(Some(path));
    assert!(result.is_err());
}

#[expect(
    clippy::cast_possible_wrap,
    reason = "WAD test offsets are small and fit in i32"
)]
fn dir_entry(off: u32, size: i32, name: &str) -> [u8; 16] {
    let mut bytes = [0u8; 16];
    bytes[0..4].copy_from_slice(&(off as i32).to_le_bytes());
    bytes[4..8].copy_from_slice(&size.to_le_bytes());
    let name_bytes = name.as_bytes();
    bytes[8..8 + name_bytes.len().min(8)].copy_from_slice(name_bytes);
    bytes
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "WAD test data uses small known sizes"
)]
fn create_minimal_wad_bytes() -> Vec<u8> {
    let mut wad = Vec::new();
    let data_start = 12u32;
    let vertex_data = [0i16, 0i16, 100i16, 100i16];
    let vertex_bytes: Vec<u8> = vertex_data.iter().flat_map(|v| v.to_le_bytes()).collect();
    let vertex_size = vertex_bytes.len() as i32;
    let linedef_data: [u8; 14] = [0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let linedef_size = 14i32;
    let mut sector_data = [0u8; 26];
    sector_data[0..2].copy_from_slice(&0i16.to_le_bytes());
    sector_data[2..4].copy_from_slice(&128i16.to_le_bytes());
    sector_data[4..12].copy_from_slice(b"FLOOR4_6");
    sector_data[12..20].copy_from_slice(b"CEIL3_5 ");
    sector_data[20..22].copy_from_slice(&160u16.to_le_bytes());
    let sector_size = 26i32;
    let thing_data: [u8; 10] = [50, 0, 50, 0, 0, 0, 1, 0, 0, 0];
    let thing_size = 10i32;
    let vertex_offset = data_start;
    let linedef_offset = data_start + vertex_size as u32;
    let sector_offset = linedef_offset + linedef_size as u32;
    let thing_offset = sector_offset + sector_size as u32;
    let dir_offset = thing_offset + thing_size as u32;
    wad.extend_from_slice(b"IWAD");
    wad.extend_from_slice(&5i32.to_le_bytes());
    wad.extend_from_slice(&dir_offset.to_le_bytes());
    wad.extend_from_slice(&vertex_bytes);
    wad.extend_from_slice(&linedef_data);
    wad.extend_from_slice(&sector_data);
    wad.extend_from_slice(&thing_data);
    wad.extend_from_slice(&dir_entry(vertex_offset, 0, "E1M1"));
    wad.extend_from_slice(&dir_entry(vertex_offset, vertex_size, "VERTEXES"));
    wad.extend_from_slice(&dir_entry(linedef_offset, linedef_size, "LINEDEFS"));
    wad.extend_from_slice(&dir_entry(sector_offset, sector_size, "SECTORS"));
    wad.extend_from_slice(&dir_entry(thing_offset, thing_size, "THINGS"));
    wad
}
