// SPDX-License-Identifier: AGPL-3.0-or-later
//! Unit tests for the spring data adapter.

use crate::data_channel::DataBinding;

use super::{SpringDataAdapter, SpringPayloadFormat};

#[test]
fn detect_bindings_format() {
    let json = serde_json::json!({
        "bindings": [
            {"channel_type": "gauge", "id": "g1", "label": "Test", "value": 50.0,
             "min": 0.0, "max": 100.0, "unit": "u", "normal_range": [20.0, 80.0],
             "warning_range": [0.0, 100.0]}
        ]
    });
    assert_eq!(
        SpringDataAdapter::detect_format(&json),
        SpringPayloadFormat::Bindings
    );
}

#[test]
fn detect_game_channel_format() {
    let json = serde_json::json!({
        "channel": "engagement_curve",
        "data": {"id": "ec1", "label": "Engagement", "timestamps": [0.0, 1.0], "values": [0.5, 0.8], "unit": "score"}
    });
    assert_eq!(
        SpringDataAdapter::detect_format(&json),
        SpringPayloadFormat::GameChannel
    );
}

#[test]
fn detect_eco_timeseries_format() {
    let json = serde_json::json!({
        "schema": "ecoPrimals/time-series/v1",
        "series": []
    });
    assert_eq!(
        SpringDataAdapter::detect_format(&json),
        SpringPayloadFormat::EcoTimeSeries
    );
}

#[test]
fn detect_raw_array_format() {
    let json = serde_json::json!([
        {"channel_type": "gauge", "id": "g1", "label": "Test", "value": 50.0,
         "min": 0.0, "max": 100.0, "unit": "u", "normal_range": [20.0, 80.0],
         "warning_range": [0.0, 100.0]}
    ]);
    assert_eq!(
        SpringDataAdapter::detect_format(&json),
        SpringPayloadFormat::Raw
    );
}

#[test]
fn adapt_bindings_envelope() {
    let json = serde_json::json!({
        "bindings": [
            {"channel_type": "timeseries", "id": "ts1", "label": "Metric",
             "x_label": "t", "y_label": "v", "unit": "u",
             "x_values": [0.0, 1.0], "y_values": [10.0, 20.0]}
        ]
    });
    let result = SpringDataAdapter::adapt(json).unwrap();
    assert_eq!(result.len(), 1);
    assert!(matches!(&result[0], DataBinding::TimeSeries { id, .. } if id == "ts1"));
}

#[test]
fn adapt_game_channel_engagement_curve() {
    let json = serde_json::json!({
        "channel": "engagement_curve",
        "data": {
            "id": "ec1", "label": "Player Engagement",
            "timestamps": [0.0, 1.0, 2.0], "values": [0.5, 0.8, 0.6],
            "unit": "score"
        }
    });
    let result = SpringDataAdapter::adapt(json).unwrap();
    assert_eq!(result.len(), 1);
    match &result[0] {
        DataBinding::TimeSeries {
            id,
            x_values,
            y_values,
            ..
        } => {
            assert_eq!(id, "ec1");
            assert_eq!(x_values.len(), 3);
            assert_eq!(y_values.len(), 3);
        }
        _ => panic!("expected TimeSeries"),
    }
}

#[test]
fn adapt_game_channel_flow_timeline() {
    let json = serde_json::json!({
        "channel": "flow_timeline",
        "data": {
            "id": "ft1", "label": "Flow States",
            "categories": ["Zone 1", "Zone 2", "Zone 3"],
            "values": [0.3, 0.5, 0.2],
            "unit": "ratio"
        }
    });
    let result = SpringDataAdapter::adapt(json).unwrap();
    assert_eq!(result.len(), 1);
    match &result[0] {
        DataBinding::Bar { id, categories, .. } => {
            assert_eq!(id, "ft1");
            assert_eq!(categories.len(), 3);
        }
        _ => panic!("expected Bar"),
    }
}

#[test]
fn adapt_game_channel_interaction_cost_map() {
    let json = serde_json::json!({
        "channel": "interaction_cost_map",
        "data": {
            "id": "icm1", "label": "Cost Map",
            "x_labels": ["A", "B"], "y_labels": ["X", "Y"],
            "values": [1.0, 2.0, 3.0, 4.0],
            "unit": "ms"
        }
    });
    let result = SpringDataAdapter::adapt(json).unwrap();
    assert_eq!(result.len(), 1);
    assert!(matches!(&result[0], DataBinding::Heatmap { .. }));
}

#[test]
fn adapt_game_channel_generation_preview() {
    let json = serde_json::json!({
        "channel": "generation_preview",
        "data": {
            "id": "gp1", "label": "Preview",
            "x": [1.0, 2.0], "y": [3.0, 4.0],
            "unit": "px"
        }
    });
    let result = SpringDataAdapter::adapt(json).unwrap();
    assert_eq!(result.len(), 1);
    assert!(matches!(&result[0], DataBinding::Scatter { .. }));
}

#[test]
fn adapt_eco_timeseries_rejects_wrong_schema() {
    let json = serde_json::json!({
        "schema": "other/schema",
        "series": []
    });
    assert!(SpringDataAdapter::adapt(json).is_err());
}

#[test]
fn adapt_eco_timeseries() {
    let json = serde_json::json!({
        "schema": "ecoPrimals/time-series/v1",
        "series": [
            {
                "id": "et0_daily",
                "label": "ET0 Daily",
                "unit": "mm/day",
                "timestamps": [0.0, 86_400.0, 172_800.0],
                "values": [3.2, 4.1, 3.8]
            },
            {
                "id": "rainfall",
                "label": "Rainfall",
                "unit": "mm",
                "timestamps": [0.0, 86_400.0],
                "values": [0.0, 12.5]
            }
        ]
    });
    let result = SpringDataAdapter::adapt(json).unwrap();
    assert_eq!(result.len(), 2);
    match &result[0] {
        DataBinding::TimeSeries {
            id,
            x_label,
            unit,
            x_values,
            ..
        } => {
            assert_eq!(id, "et0_daily");
            assert_eq!(x_label, "Time");
            assert_eq!(unit, "mm/day");
            assert_eq!(x_values.len(), 3);
        }
        _ => panic!("expected TimeSeries"),
    }
}

#[test]
fn adapt_raw_array() {
    let json = serde_json::json!([
        {"channel_type": "spectrum", "id": "s1", "label": "Spec",
         "frequencies": [1.0, 2.0], "amplitudes": [0.5, 0.3], "unit": "dB"}
    ]);
    let result = SpringDataAdapter::adapt(json).unwrap();
    assert_eq!(result.len(), 1);
    assert!(matches!(&result[0], DataBinding::Spectrum { .. }));
}

#[test]
fn adapt_invalid_json_returns_error() {
    let json = serde_json::json!("not a valid payload");
    let result = SpringDataAdapter::adapt(json);
    assert!(result.is_err());
}

#[test]
fn game_channel_difficulty_profile_maps_to_timeseries() {
    let json = serde_json::json!({
        "channel": "difficulty_profile",
        "data": {
            "id": "dp1", "label": "Difficulty",
            "x_values": [0.0, 1.0], "y_values": [1.0, 5.0],
            "unit": "level"
        }
    });
    let result = SpringDataAdapter::adapt(json).unwrap();
    assert!(matches!(&result[0], DataBinding::TimeSeries { .. }));
}

#[test]
fn game_channel_ui_analysis_maps_to_bar() {
    let json = serde_json::json!({
        "channel": "ui_analysis",
        "data": {
            "id": "ua1", "label": "UI Metrics",
            "metrics": ["Fitts", "Hick", "Steering"],
            "values": [0.9, 0.8, 0.7],
            "unit": "score"
        }
    });
    let result = SpringDataAdapter::adapt(json).unwrap();
    assert!(matches!(&result[0], DataBinding::Bar { .. }));
}

#[test]
fn game_channel_accessibility_report_maps_to_heatmap() {
    let json = serde_json::json!({
        "channel": "accessibility_report",
        "data": {
            "id": "ar1", "label": "Accessibility",
            "x_labels": ["Visual", "Motor"], "y_labels": ["Low", "High"],
            "values": [0.9, 0.7, 0.8, 0.6],
            "unit": "score"
        }
    });
    let result = SpringDataAdapter::adapt(json).unwrap();
    assert!(matches!(&result[0], DataBinding::Heatmap { .. }));
}

#[test]
fn detect_game_scene_format() {
    let json = serde_json::json!({
        "channel_type": "game_scene",
        "id": "dungeon_1",
        "label": "Dungeon Level 1",
        "scene": { "tilemap": null, "sprites": [], "entities": [] }
    });
    assert_eq!(
        SpringDataAdapter::detect_format(&json),
        SpringPayloadFormat::GameScene
    );
}

#[test]
fn adapt_game_scene() {
    let json = serde_json::json!({
        "channel_type": "game_scene",
        "id": "dungeon_1",
        "label": "Dungeon Level 1",
        "scene": {
            "tilemap": { "dimensions": [10, 10], "tile_size": [16.0, 16.0], "tiles": [] },
            "sprites": [],
            "entities": [{ "id": "p1", "entity_type": "player", "position": [5.0, 5.0] }],
            "camera_center": [5.0, 5.0],
            "camera_zoom": 1.0
        }
    });
    let result = SpringDataAdapter::adapt(json).unwrap();
    assert_eq!(result.len(), 1);
    match &result[0] {
        DataBinding::GameScene { id, label, scene } => {
            assert_eq!(id, "dungeon_1");
            assert_eq!(label, "Dungeon Level 1");
            assert!(scene.get("entities").is_some());
        }
        other => panic!("expected GameScene, got {other:?}"),
    }
}

#[test]
fn detect_soundscape_format() {
    let json = serde_json::json!({
        "channel_type": "soundscape",
        "id": "forest",
        "label": "Forest Ambience",
        "definition": { "name": "forest", "duration_secs": 30.0, "layers": [] }
    });
    assert_eq!(
        SpringDataAdapter::detect_format(&json),
        SpringPayloadFormat::SoundscapePush
    );
}

#[test]
fn adapt_soundscape() {
    let json = serde_json::json!({
        "channel_type": "soundscape",
        "id": "forest",
        "label": "Forest Ambience",
        "definition": {
            "name": "forest",
            "duration_secs": 30.0,
            "layers": [
                { "id": "wind", "waveform": "white_noise", "frequency": 0.0,
                  "amplitude": 0.1, "duration_secs": 30.0 }
            ]
        }
    });
    let result = SpringDataAdapter::adapt(json).unwrap();
    assert_eq!(result.len(), 1);
    match &result[0] {
        DataBinding::Soundscape {
            id,
            label,
            definition,
        } => {
            assert_eq!(id, "forest");
            assert_eq!(label, "Forest Ambience");
            assert!(definition.get("layers").is_some());
        }
        other => panic!("expected Soundscape, got {other:?}"),
    }
}
