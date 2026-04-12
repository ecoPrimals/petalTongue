// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

fn run_in_egui(mut f: impl FnMut(&mut Ui)) {
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| f(ui));
    });
}

#[test]
fn draw_empty_scene() {
    run_in_egui(|ui| {
        draw_game_scene(ui, "Empty", &serde_json::json!({}));
    });
}

#[test]
fn draw_full_scene() {
    let scene = serde_json::json!({
        "tilemap": {
            "dimensions": [4, 3],
            "tile_size": [16.0, 16.0],
            "origin": [0.0, 0.0],
            "tiles": (0..12).map(|i| serde_json::json!({
                "tile_type": i % 3,
                "color": [0, 0, 0, 0],
                "solid": i % 4 == 0
            })).collect::<Vec<_>>(),
            "palette": [[0, 128, 0, 255], [64, 64, 64, 255], [0, 0, 200, 255]]
        },
        "sprites": [{
            "id": "chest",
            "position": [24.0, 8.0],
            "size": [8.0, 8.0],
            "tint": [255, 200, 50, 255],
            "z_order": 5,
            "visible": true,
            "label": "Treasure"
        }],
        "entities": [{
            "id": "hero",
            "entity_type": "player",
            "position": [16.0, 16.0],
            "velocity": [2.0, 0.0],
            "health": 0.8,
            "color": [0, 200, 255, 255],
            "size": [2.0, 2.0],
            "label": "Hero"
        }, {
            "id": "goblin",
            "entity_type": "enemy",
            "position": [30.0, 20.0],
            "health": 0.3,
            "color": [200, 50, 50, 255],
            "size": [1.5, 1.5],
            "label": "Goblin"
        }],
        "camera_center": [20.0, 16.0],
        "camera_zoom": 1.5
    });
    run_in_egui(|ui| {
        draw_game_scene(ui, "Dungeon Level 1", &scene);
    });
}

#[test]
fn draw_invalid_scene_degrades() {
    run_in_egui(|ui| {
        draw_game_scene(ui, "Bad", &serde_json::json!("not a scene"));
    });
}

#[test]
fn camera_transform_world_to_screen() {
    let scene = GameScene::default();
    let canvas = Rect::from_min_size(pos2(0.0, 0.0), vec2(320.0, 240.0));
    let cam = CameraTransform::new(&scene, canvas);
    let origin = cam.world_to_screen(0.0, 0.0);
    assert!(origin.x.is_finite());
    assert!(origin.y.is_finite());
}

#[test]
fn rgba_conversion() {
    let c = rgba_to_color32([255, 128, 0, 255]);
    assert_eq!(c.r(), 255);
    assert_eq!(c.g(), 128);
    assert_eq!(c.b(), 0);
    assert_eq!(c.a(), 255);
}

#[test]
fn detect_narrative_vs_tilemap() {
    assert!(is_narrative_scene(&serde_json::json!({
        "description": "A dark room", "npcs": [], "node": "n1"
    })));
    assert!(!is_narrative_scene(&serde_json::json!({
        "tilemap": null, "sprites": [], "entities": []
    })));
    assert!(!is_narrative_scene(&serde_json::json!({})));
}

#[test]
fn draw_dialogue_scene() {
    let scene = serde_json::json!({
        "node": "tavern_1",
        "description": "You enter the dimly lit tavern. A bard plays a melancholy tune.",
        "npcs": [
            {"name": "Innkeeper", "status": "friendly", "health": 1.0},
            {"name": "Mysterious Stranger", "status": "present"}
        ],
        "turn": 3,
        "is_ending": false,
        "choices": [
            "Talk to the innkeeper",
            "Approach the stranger",
            "Order a drink",
            "Leave the tavern"
        ]
    });
    run_in_egui(|ui| draw_game_scene(ui, "Tavern Scene", &scene));
}

#[test]
fn draw_combat_scene() {
    let scene = serde_json::json!({
        "scene_type": "combat_grid",
        "description": "The goblins attack!",
        "entities": [
            {"id": "hero", "entity_type": "player", "position": [3.0, 2.0], "label": "Hero"},
            {"id": "goblin1", "entity_type": "enemy", "position": [5.0, 4.0], "label": "Goblin"}
        ],
        "npcs": [],
        "turn": 1
    });
    run_in_egui(|ui| draw_game_scene(ui, "Combat", &scene));
}

#[test]
fn draw_narration_scene() {
    let scene = serde_json::json!({
        "node": "epilogue",
        "description": "And so the hero returned home, forever changed by the journey.",
        "is_ending": true,
        "turn": 42
    });
    run_in_egui(|ui| draw_game_scene(ui, "Epilogue", &scene));
}

#[test]
fn draw_esotericwebb_minimal_scene() {
    let scene = serde_json::json!({
        "node": "scene_7",
        "description": "The forest path splits.",
        "npcs": ["Old Hermit"],
        "turn": 5,
        "is_ending": false
    });
    run_in_egui(|ui| draw_game_scene(ui, "Webb Scene", &scene));
}
