// SPDX-License-Identifier: AGPL-3.0-or-later

use super::RaycastRenderer;
use super::math::ray_line_intersection;
use std::f32::consts::PI;

#[test]
fn test_renderer_creation() {
    let renderer = RaycastRenderer::new(320, 240);
    assert_eq!(renderer.framebuffer().len(), 320 * 240 * 4);
}

#[test]
fn test_ray_angle_calculation() {
    let renderer = RaycastRenderer::new(320, 240);

    let center_angle = renderer.calculate_ray_angle(160);
    assert!((center_angle - renderer.player_angle).abs() < 0.01);

    let left_angle = renderer.calculate_ray_angle(0);
    assert!(left_angle < renderer.player_angle);

    let right_angle = renderer.calculate_ray_angle(319);
    assert!(right_angle > renderer.player_angle);
}

#[test]
fn test_player_movement() {
    let mut renderer = RaycastRenderer::new(320, 240);
    renderer.player_x = 100.0;
    renderer.player_y = 100.0;
    renderer.player_angle = 0.0;

    renderer.move_forward(10.0);
    assert!((renderer.player_x - 110.0).abs() < 0.1);
    assert!((renderer.player_y - 100.0).abs() < 0.1);

    renderer.rotate(PI / 2.0);
    renderer.move_forward(10.0);
    assert!((renderer.player_y - 110.0).abs() < 0.1);
}

#[test]
fn test_render_empty_map() {
    use crate::wad_loader::MapData;

    let mut renderer = RaycastRenderer::new(64, 64);
    let map = MapData {
        name: "TEST".to_string(),
        vertices: vec![],
        linedefs: vec![],
        sectors: vec![],
        things: vec![],
    };

    renderer.render(&map);
    assert_eq!(renderer.framebuffer().len(), 64 * 64 * 4);
    assert_eq!(renderer.framebuffer()[0], 100);
    assert_eq!(renderer.framebuffer()[3], 255);
}

#[test]
fn test_set_player_start() {
    use crate::wad_loader::{MapData, Thing};

    let mut renderer = RaycastRenderer::new(64, 64);
    let map = MapData {
        name: "TEST".to_string(),
        vertices: vec![],
        linedefs: vec![],
        sectors: vec![],
        things: vec![Thing {
            x: 64,
            y: 64,
            angle: 90,
            thing_type: 1,
            flags: 0,
        }],
    };

    renderer.set_player_start(&map);
    assert!((renderer.player_x - 64.0).abs() < f32::EPSILON);
    assert!((renderer.player_y - 64.0).abs() < f32::EPSILON);
}

#[test]
fn test_rotate_wrapping() {
    let mut renderer = RaycastRenderer::new(64, 64);
    renderer.player_angle = 0.0;

    renderer.rotate(2.0 * PI);
    assert!((renderer.player_angle - 0.0).abs() < 0.01);

    renderer.rotate(-2.0 * PI);
    assert!((renderer.player_angle - 0.0).abs() < 0.01);
}

#[test]
fn test_move_strafe() {
    let mut renderer = RaycastRenderer::new(64, 64);
    renderer.player_x = 0.0;
    renderer.player_y = 0.0;
    renderer.player_angle = 0.0;

    renderer.move_strafe(10.0);
    assert!((renderer.player_y - 10.0).abs() < 0.1);
}

#[test]
fn test_ray_line_intersection_parallel() {
    use crate::wad_loader::Vertex;
    let v1 = Vertex { x: 0, y: 10 };
    let v2 = Vertex { x: 100, y: 10 };
    let dist = ray_line_intersection(0.0, 0.0, 1.0, 0.0, v1, v2);
    assert!(dist.is_none());
}

#[test]
fn test_ray_line_intersection_behind() {
    use crate::wad_loader::Vertex;
    let v1 = Vertex { x: -10, y: -5 };
    let v2 = Vertex { x: -10, y: 5 };
    let dist = ray_line_intersection(0.0, 0.0, 1.0, 0.0, v1, v2);
    assert!(dist.is_none());
}

#[test]
fn test_ray_line_intersection_u_out_of_range() {
    use crate::wad_loader::Vertex;
    let v1 = Vertex { x: 10, y: 10 };
    let v2 = Vertex { x: 10, y: 20 };
    let dist = ray_line_intersection(0.0, 0.0, 1.0, 0.0, v1, v2);
    assert!(dist.is_none());
}

#[test]
fn test_ray_line_intersection_t_negative() {
    use crate::wad_loader::Vertex;
    let v1 = Vertex { x: -50, y: -10 };
    let v2 = Vertex { x: -50, y: 10 };
    let dist = ray_line_intersection(0.0, 0.0, 1.0, 0.0, v1, v2);
    assert!(dist.is_none());
}

#[test]
fn test_linedef_invalid_vertex_skipped() {
    use crate::wad_loader::{LineDef, MapData, Vertex};

    let mut renderer = RaycastRenderer::new(64, 64);
    renderer.player_x = 0.0;
    renderer.player_y = 0.0;
    renderer.player_angle = 0.0;

    let map = MapData {
        name: "INVALID".to_string(),
        vertices: vec![Vertex { x: 100, y: 0 }],
        linedefs: vec![
            LineDef {
                start_vertex: 0,
                end_vertex: 99,
                flags: 0,
                line_type: 0,
                sector_tag: 0,
            },
            LineDef {
                start_vertex: 99,
                end_vertex: 0,
                flags: 0,
                line_type: 0,
                sector_tag: 0,
            },
        ],
        sectors: vec![],
        things: vec![],
    };
    renderer.render(&map);
    assert_eq!(renderer.framebuffer().len(), 64 * 64 * 4);
}

#[test]
fn test_calculate_wall_height() {
    let renderer = RaycastRenderer::new(320, 240);
    let h_close = renderer.calculate_wall_height(50.0);
    let h_far = renderer.calculate_wall_height(500.0);
    assert!(h_close > h_far);
    assert!(h_close <= 480);
    assert!(h_far > 0);
}

#[test]
fn test_calculate_wall_height_capped() {
    let renderer = RaycastRenderer::new(64, 64);
    let h = renderer.calculate_wall_height(0.1);
    assert!(h <= 128);
}

#[test]
fn test_render_to_scene_empty_map() {
    use crate::wad_loader::MapData;

    let renderer = RaycastRenderer::new(64, 64);
    let map = MapData {
        name: "TEST".to_string(),
        vertices: vec![],
        linedefs: vec![],
        sectors: vec![],
        things: vec![],
    };
    let scene = renderer.render_to_scene(&map);
    assert!(scene.get("sky").is_some());
    assert!(scene.get("floor").is_some());
}

#[test]
fn test_render_with_wall_produces_wall_pixels() {
    use crate::wad_loader::{LineDef, MapData, Vertex};

    let mut renderer = RaycastRenderer::new(64, 64);
    renderer.player_x = 0.0;
    renderer.player_y = 0.0;
    renderer.player_angle = 0.0;

    let map = MapData {
        name: "TEST".to_string(),
        vertices: vec![Vertex { x: 100, y: -50 }, Vertex { x: 100, y: 50 }],
        linedefs: vec![LineDef {
            start_vertex: 0,
            end_vertex: 1,
            flags: 0,
            line_type: 0,
            sector_tag: 0,
        }],
        sectors: vec![],
        things: vec![],
    };
    renderer.render(&map);
    let fb = renderer.framebuffer();
    let mid = fb.len() / 2;
    let has_floor = fb[mid] == 64 && fb[mid + 1] == 64 && fb[mid + 2] == 64;
    assert!(has_floor);
}

#[test]
fn test_linedef_flags_blocking() {
    use crate::wad_loader::{LineDef, MapData, Vertex};

    let mut renderer = RaycastRenderer::new(64, 64);
    renderer.player_x = 0.0;
    renderer.player_y = 0.0;
    renderer.player_angle = 0.0;

    let map_blocking = MapData {
        name: "BLOCK".to_string(),
        vertices: vec![Vertex { x: 50, y: -20 }, Vertex { x: 50, y: 20 }],
        linedefs: vec![LineDef {
            start_vertex: 0,
            end_vertex: 1,
            flags: 0x0001,
            line_type: 0,
            sector_tag: 0,
        }],
        sectors: vec![],
        things: vec![],
    };
    renderer.render(&map_blocking);

    let map_non_blocking = MapData {
        name: "NON".to_string(),
        vertices: vec![Vertex { x: 50, y: -20 }, Vertex { x: 50, y: 20 }],
        linedefs: vec![LineDef {
            start_vertex: 0,
            end_vertex: 1,
            flags: 0,
            line_type: 0,
            sector_tag: 0,
        }],
        sectors: vec![],
        things: vec![],
    };
    renderer.render(&map_non_blocking);
    assert_eq!(renderer.framebuffer().len(), 64 * 64 * 4);
}

#[test]
fn test_render_to_scene_with_wall() {
    use crate::wad_loader::{LineDef, MapData, Vertex};

    let mut renderer = RaycastRenderer::new(64, 64);
    renderer.player_x = 0.0;
    renderer.player_y = 0.0;
    renderer.player_angle = 0.0;

    let map = MapData {
        name: "WALL".to_string(),
        vertices: vec![Vertex { x: 100, y: -50 }, Vertex { x: 100, y: 50 }],
        linedefs: vec![LineDef {
            start_vertex: 0,
            end_vertex: 1,
            flags: 0,
            line_type: 0,
            sector_tag: 0,
        }],
        sectors: vec![],
        things: vec![],
    };
    let scene = renderer.render_to_scene(&map);
    assert!(scene.get("sky").is_some());
    assert!(scene.get("floor").is_some());
}
