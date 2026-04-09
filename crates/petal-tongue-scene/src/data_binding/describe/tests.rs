// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use crate::modality::HapticPattern;
use petal_tongue_types::DataBinding;
use serde_json::json;

#[test]
fn describe_game_scene_tilemap() {
    let scene = json!({
        "tilemap": {"dimensions": [16, 12]},
        "sprites": [{"id": "chest", "visible": true, "label": "Treasure"}],
        "entities": [{
            "id": "hero", "entity_type": "player",
            "position": [16.0, 16.0], "health": 0.8,
            "velocity": [2.0, 0.0], "label": "Hero"
        }],
        "camera_center": [20.0, 16.0]
    });
    let b = DataBinding::GameScene {
        id: "s".into(),
        label: "Dungeon".into(),
        scene,
    };
    let desc = describe_binding(&b);
    assert!(desc.contains("16x12 tilemap"));
    assert!(desc.contains("Treasure"));
    assert!(desc.contains("Hero (player)"));
    assert!(desc.contains("health 80%"));
    assert!(desc.contains("moving"));
}

#[test]
fn describe_narrative_scene_dialogue() {
    let scene = json!({
        "node": "tavern_1",
        "description": "A dark tavern.",
        "npcs": [{"name": "Innkeeper", "status": "friendly", "health": 1.0}],
        "choices": ["Talk", "Leave"],
        "turn": 3
    });
    let b = DataBinding::GameScene {
        id: "s".into(),
        label: "Tavern".into(),
        scene,
    };
    let desc = describe_binding(&b);
    assert!(desc.contains("Narrative scene"));
    assert!(desc.contains("tavern_1"));
    assert!(desc.contains("A dark tavern"));
    assert!(desc.contains("Innkeeper"));
    assert!(desc.contains("Talk"));
    assert!(desc.contains("Turn 3"));
}

#[test]
fn describe_soundscape_layers() {
    let def = json!({
        "name": "Forest",
        "duration_secs": 30.0,
        "layers": [
            {"id": "wind", "waveform": "white_noise", "frequency": 200.0, "amplitude": 0.3, "pan": -0.5},
            {"id": "bird", "waveform": "sine", "frequency": 800.0, "amplitude": 0.6, "pan": 0.7, "offset_secs": 5.0}
        ]
    });
    let b = DataBinding::Soundscape {
        id: "s".into(),
        label: "Forest".into(),
        definition: def,
    };
    let desc = describe_binding(&b);
    assert!(desc.contains("30.0 seconds"));
    assert!(desc.contains("2 layers"));
    assert!(desc.contains("wind: white_noise"));
    assert!(desc.contains("left"));
    assert!(desc.contains("bird: sine"));
    assert!(desc.contains("starts at 5.0s"));
}

#[test]
fn sonify_game_scene_entities() {
    let scene = json!({
        "entities": [
            {"entity_type": "player", "position": [10.0, 10.0], "health": 0.9},
            {"entity_type": "enemy", "position": [40.0, 10.0], "health": 0.5}
        ],
        "camera_center": [20.0, 10.0]
    });
    let params = sonify_game_scene(&scene);
    assert_eq!(params.len(), 2);
    assert!((params[0].frequency - 440.0).abs() < 1.0, "player = A4");
    assert!((params[1].frequency - 220.0).abs() < 1.0, "enemy = A3");
    assert!(params[0].pan < 0.0, "player left of camera");
    assert!(params[1].pan > 0.0, "enemy right of camera");
}

#[test]
fn hapticize_game_scene_entities() {
    let scene = json!({
        "entities": [
            {"entity_type": "player", "position": [16.0, 16.0], "health": 0.8},
            {"entity_type": "enemy", "position": [32.0, 32.0], "health": 0.2}
        ]
    });
    let cmds = hapticize_game_scene(&scene);
    assert_eq!(cmds.len(), 2);
    assert_eq!(cmds[0].pattern, HapticPattern::Sustained);
    assert_eq!(cmds[1].pattern, HapticPattern::Pulse);
    assert!(
        cmds[1].intensity > cmds[0].intensity,
        "low-health enemy = stronger haptic"
    );
}

#[test]
fn hapticize_soundscape_layers() {
    let def = json!({
        "layers": [
            {"frequency": 100.0, "amplitude": 0.5, "pan": -1.0},
            {"frequency": 800.0, "amplitude": 0.8, "pan": 1.0}
        ]
    });
    let cmds = hapticize_soundscape(&def);
    assert_eq!(cmds.len(), 2);
    assert_eq!(cmds[0].pattern, HapticPattern::Sustained);
    assert_eq!(cmds[1].pattern, HapticPattern::Ramp);
    assert!(cmds[0].position[0] < 0.1, "full left");
    assert!(cmds[1].position[0] > 0.9, "full right");
}

#[test]
fn describe_timeseries_trend() {
    let b = DataBinding::TimeSeries {
        id: "t".into(),
        label: "Glucose".into(),
        x_label: "Time".into(),
        y_label: "mg/dL".into(),
        unit: "mg/dL".into(),
        x_values: vec![0.0, 1.0, 2.0],
        y_values: vec![90.0, 95.0, 100.0],
    };
    let desc = describe_binding(&b);
    assert!(desc.contains("rising"));
}

#[test]
fn describe_gauge_status() {
    let b = DataBinding::Gauge {
        id: "g".into(),
        label: "HR".into(),
        value: 150.0,
        min: 40.0,
        max: 200.0,
        unit: "bpm".into(),
        normal_range: [60.0, 100.0],
        warning_range: [40.0, 120.0],
    };
    let desc = describe_binding(&b);
    assert!(desc.contains("critical"));
    assert!(desc.contains("150.0 bpm"));
}

#[test]
fn empty_scene_graceful() {
    let desc = describe_game_scene("empty", &json!(null));
    assert!(desc.contains("empty"));
}

#[test]
fn empty_soundscape_graceful() {
    let desc = describe_soundscape("empty", &json!(null));
    assert!(desc.contains("empty"));
}
