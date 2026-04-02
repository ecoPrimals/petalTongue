// SPDX-License-Identifier: AGPL-3.0-or-later
//! Modality-agnostic descriptions for `DataBinding` variants.
//!
//! Produces structured text descriptions that screen readers, braille
//! displays, and audio narrators can consume. This is the accessibility
//! bridge: every `DataBinding` — including `GameScene` and `Soundscape` —
//! must be describable to someone who cannot see or hear.

use std::fmt::Write;

use petal_tongue_core::DataBinding;

use crate::modality::{AudioParam, HapticCommand, HapticPattern};

/// Produce a rich text description of any `DataBinding`.
///
/// Designed for screen readers, braille output, and audio TTS narration.
/// Returns structured prose that conveys the semantic content regardless
/// of visual or auditory rendering.
#[must_use]
pub fn describe_binding(binding: &DataBinding) -> String {
    match binding {
        DataBinding::TimeSeries {
            label,
            x_label,
            y_label,
            x_values,
            y_values,
            ..
        } => describe_timeseries(label, x_label, y_label, x_values, y_values),
        DataBinding::Distribution {
            label,
            values,
            mean,
            std,
            ..
        } => {
            format!(
                "Distribution '{label}': {n} values, mean {mean:.2}, standard deviation {std:.2}, range {lo:.2} to {hi:.2}.",
                n = values.len(),
                lo = values.iter().copied().fold(f64::INFINITY, f64::min),
                hi = values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            )
        }
        DataBinding::Bar {
            label,
            categories,
            values,
            unit,
            ..
        } => describe_bar(label, categories, values, unit),
        DataBinding::Gauge {
            label,
            value,
            min,
            max,
            unit,
            normal_range,
            warning_range,
            ..
        } => describe_gauge(label, *value, *min, *max, unit, normal_range, warning_range),
        DataBinding::Heatmap {
            label,
            x_labels,
            y_labels,
            values,
            unit,
            ..
        } => {
            format!(
                "Heatmap '{label}': {cols}x{rows} grid ({unit}). Values range {lo:.2} to {hi:.2}.",
                cols = x_labels.len(),
                rows = y_labels.len(),
                lo = values.iter().copied().fold(f64::INFINITY, f64::min),
                hi = values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            )
        }
        DataBinding::Scatter {
            label,
            x,
            x_label,
            y_label,
            ..
        } => {
            format!(
                "Scatter plot '{label}': {n} points. {x_label} vs {y_label}.",
                n = x.len()
            )
        }
        DataBinding::Scatter3D { label, x, unit, .. } => {
            format!("3D scatter '{label}': {n} points ({unit}).", n = x.len())
        }
        DataBinding::FieldMap {
            label,
            grid_x,
            grid_y,
            unit,
            ..
        } => {
            format!(
                "Field map '{label}': {cols}x{rows} grid ({unit}).",
                cols = grid_x.len(),
                rows = grid_y.len()
            )
        }
        DataBinding::Spectrum {
            label,
            frequencies,
            unit,
            ..
        } => {
            format!(
                "Spectrum '{label}': {n} frequency bands ({unit}).",
                n = frequencies.len()
            )
        }
        DataBinding::GameScene { label, scene, .. } => describe_game_scene(label, scene),
        DataBinding::Soundscape {
            label, definition, ..
        } => describe_soundscape(label, definition),
    }
}

fn describe_timeseries(
    label: &str,
    x_label: &str,
    y_label: &str,
    x_values: &[f64],
    y_values: &[f64],
) -> String {
    let n = x_values.len();
    let trend = if n >= 2 {
        let first = y_values.first().copied().unwrap_or(0.0);
        let last = y_values.last().copied().unwrap_or(0.0);
        if last > first {
            "rising"
        } else if last < first {
            "falling"
        } else {
            "flat"
        }
    } else {
        "single point"
    };
    format!("Time series '{label}': {n} points, {x_label} vs {y_label}, trend {trend}.")
}

fn describe_bar(label: &str, categories: &[String], values: &[f64], unit: &str) -> String {
    let mut desc = format!(
        "Bar chart '{label}': {n} categories ({unit}).",
        n = categories.len()
    );
    let max_show = 5;
    for (i, (cat, val)) in categories.iter().zip(values).enumerate() {
        if i >= max_show {
            let _ = write!(desc, " ...and {} more.", categories.len() - max_show);
            break;
        }
        let _ = write!(desc, " {cat}: {val:.1}.");
    }
    desc
}

fn describe_gauge(
    label: &str,
    value: f64,
    min: f64,
    max: f64,
    unit: &str,
    normal_range: &[f64; 2],
    warning_range: &[f64; 2],
) -> String {
    let status = if value >= normal_range[0] && value <= normal_range[1] {
        "normal"
    } else if value >= warning_range[0] && value <= warning_range[1] {
        "warning"
    } else {
        "critical"
    };
    format!("Gauge '{label}': {value:.1} {unit} (range {min:.0} to {max:.0}). Status: {status}.")
}

/// Rich description of a game scene for accessibility.
///
/// A blind user hears: "Game scene 'Dungeon Level 1': 4x3 tilemap,
/// 1 sprite (Treasure), 2 entities. Hero at (16,16), health 80%, moving right.
/// Goblin at (30,20), health 30%, stationary."
fn describe_game_scene(label: &str, scene_json: &serde_json::Value) -> String {
    let Some(obj) = scene_json.as_object() else {
        return format!("Game scene '{label}': empty.");
    };

    if obj.contains_key("description") || obj.contains_key("node") || obj.contains_key("npcs") {
        return describe_narrative_scene(label, scene_json);
    }

    let mut desc = format!("Game scene '{label}':");

    if let Some(dims) = obj
        .get("tilemap")
        .and_then(|v| v.as_object())
        .and_then(|tm| tm.get("dimensions"))
        .and_then(|v| v.as_array())
    {
        let cols = dims
            .first()
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let rows = dims.get(1).and_then(serde_json::Value::as_u64).unwrap_or(0);
        let _ = write!(desc, " {cols}x{rows} tilemap.");
    }

    if let Some(sprites) = obj.get("sprites").and_then(|v| v.as_array()) {
        let visible: Vec<_> = sprites
            .iter()
            .filter(|s| {
                s.get("visible")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true)
            })
            .collect();
        if !visible.is_empty() {
            let _ = write!(desc, " {} sprites:", visible.len());
            for s in visible.iter().take(5) {
                let sprite_label = s
                    .get("label")
                    .or_else(|| s.get("id"))
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("unnamed");
                let _ = write!(desc, " {sprite_label};");
            }
        }
    }

    if let Some(entities) = obj
        .get("entities")
        .and_then(serde_json::Value::as_array)
        .filter(|e| !e.is_empty())
    {
        let _ = write!(desc, " {} entities.", entities.len());
        for entity in entities.iter().take(8) {
            describe_entity(&mut desc, entity);
        }
    }

    desc
}

fn describe_entity(desc: &mut String, entity: &serde_json::Value) {
    let name = entity
        .get("label")
        .or_else(|| entity.get("id"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown");
    let etype = entity
        .get("entity_type")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("entity");
    let pos = entity.get("position").and_then(|v| {
        let arr = v.as_array()?;
        Some((arr.first()?.as_f64()?, arr.get(1)?.as_f64()?))
    });
    let health = entity.get("health").and_then(serde_json::Value::as_f64);
    let velocity = entity.get("velocity").and_then(|v| {
        let arr = v.as_array()?;
        Some((arr.first()?.as_f64()?, arr.get(1)?.as_f64()?))
    });

    let _ = write!(desc, " {name} ({etype})");
    if let Some((x, y)) = pos {
        let _ = write!(desc, " at ({x:.0},{y:.0})");
    }
    if let Some(hp) = health {
        let _ = write!(desc, ", health {:.0}%", hp * 100.0);
    }
    if let Some((vx, vy)) = velocity {
        if vx.abs() > 0.01 || vy.abs() > 0.01 {
            let dir = velocity_direction(vx, vy);
            let _ = write!(desc, ", moving {dir}");
        } else {
            let _ = write!(desc, ", stationary");
        }
    }
    desc.push('.');
}

fn velocity_direction(vx: f64, vy: f64) -> &'static str {
    const DIRS: [&str; 8] = [
        "left",
        "down-left",
        "down",
        "down-right",
        "right",
        "up-right",
        "up",
        "up-left",
    ];
    let angle = vy.atan2(vx);
    let raw = (angle + std::f64::consts::PI) / (std::f64::consts::PI / 4.0);
    #[expect(
        clippy::cast_sign_loss,
        reason = "rem_euclid(8.0) guarantees [0, 8) — always non-negative"
    )]
    let octant = raw.rem_euclid(8.0) as usize;
    DIRS[octant.min(7)]
}

fn describe_narrative_scene(label: &str, scene_json: &serde_json::Value) -> String {
    let mut desc = format!("Narrative scene '{label}':");

    if let Some(node) = scene_json.get("node").and_then(serde_json::Value::as_str) {
        let _ = write!(desc, " Node: {node}.");
    }
    if let Some(turn) = scene_json.get("turn").and_then(serde_json::Value::as_u64) {
        let _ = write!(desc, " Turn {turn}.");
    }
    if scene_json
        .get("is_ending")
        .and_then(serde_json::Value::as_bool)
        == Some(true)
    {
        desc.push_str(" This is an ending.");
    }
    if let Some(description) = scene_json
        .get("description")
        .and_then(serde_json::Value::as_str)
    {
        let _ = write!(desc, " {description}");
    }
    if let Some(npcs) = scene_json
        .get("npcs")
        .and_then(serde_json::Value::as_array)
        .filter(|n| !n.is_empty())
    {
        let _ = write!(desc, " {} characters present:", npcs.len());
        for npc in npcs.iter().take(8) {
            let name = npc
                .get("name")
                .and_then(serde_json::Value::as_str)
                .or_else(|| npc.as_str())
                .unwrap_or("unknown");
            let status = npc
                .get("status")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("present");
            let _ = write!(desc, " {name} ({status})");
            if let Some(hp) = npc.get("health").and_then(serde_json::Value::as_f64) {
                let _ = write!(desc, " health {:.0}%", hp * 100.0);
            }
            desc.push(';');
        }
    }
    if let Some(choices) = scene_json
        .get("choices")
        .and_then(serde_json::Value::as_array)
        .filter(|c| !c.is_empty())
    {
        let _ = write!(desc, " {} choices:", choices.len());
        for (i, choice) in choices.iter().enumerate() {
            if let Some(text) = choice.as_str() {
                let _ = write!(desc, " {}. {text};", i + 1);
            }
        }
    }
    desc
}

/// Rich description of a soundscape for accessibility.
///
/// A deaf user reads: "Soundscape 'Forest Ambience': 30 seconds, 3 layers.
/// Wind: white noise 200Hz amplitude 30% panned left. Birdsong: sine 800Hz
/// amplitude 60% panned right, starts at 5s. Creek: triangle 400Hz amplitude
/// 40% centered."
fn describe_soundscape(label: &str, definition: &serde_json::Value) -> String {
    let Some(obj) = definition.as_object() else {
        return format!("Soundscape '{label}': empty.");
    };

    let name = obj
        .get("name")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(label);
    let duration = obj
        .get("duration_secs")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let layers = obj.get("layers").and_then(serde_json::Value::as_array);
    let layer_count = layers.map_or(0, Vec::len);

    let mut desc = format!("Soundscape '{name}': {duration:.1} seconds, {layer_count} layers.");

    if let Some(layers) = layers {
        for layer in layers.iter().take(10) {
            let layer_id = layer
                .get("id")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unnamed");
            let waveform = layer
                .get("waveform")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown");
            let freq = layer
                .get("frequency")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0);
            let amp = layer
                .get("amplitude")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0);
            let pan = layer
                .get("pan")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0);
            let offset = layer
                .get("offset_secs")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0);

            let pan_desc = if pan < -0.3 {
                "left"
            } else if pan > 0.3 {
                "right"
            } else {
                "center"
            };
            let _ = write!(
                desc,
                " {layer_id}: {waveform} {freq:.0}Hz amplitude {amp_pct:.0}% {pan_desc}",
                amp_pct = amp * 100.0,
            );
            if offset > 0.01 {
                let _ = write!(desc, ", starts at {offset:.1}s");
            }
            desc.push('.');
        }
    }
    desc
}

/// Convert a GameScene to audio sonification parameters.
///
/// Entities become tones: position → stereo pan, entity type → frequency
/// range, health → amplitude. This lets a blind user "hear" the battlefield.
#[must_use]
pub fn sonify_game_scene(scene_json: &serde_json::Value) -> Vec<AudioParam> {
    let Some(obj) = scene_json.as_object() else {
        return vec![];
    };

    let mut params = Vec::new();
    let camera_center = obj.get("camera_center").and_then(|v| {
        let arr = v.as_array()?;
        Some((arr.first()?.as_f64()?, arr.get(1)?.as_f64()?))
    });
    let (cx, _cy) = camera_center.unwrap_or((0.0, 0.0));

    if let Some(entities) = obj.get("entities").and_then(serde_json::Value::as_array) {
        for entity in entities {
            let etype = entity
                .get("entity_type")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown");
            let pos = entity.get("position").and_then(|v| {
                let arr = v.as_array()?;
                Some((arr.first()?.as_f64()?, arr.get(1)?.as_f64()?))
            });
            let health = entity
                .get("health")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(1.0);

            let (ex, _ey) = pos.unwrap_or((0.0, 0.0));
            let relative_x = ex - cx;
            let pan = (relative_x / 32.0).clamp(-1.0, 1.0);

            let base_freq = match etype {
                "player" => 440.0,
                "enemy" => 220.0,
                "npc" | "ally" => 330.0,
                "item" => 660.0,
                "projectile" => 880.0,
                _ => 300.0,
            };
            let amplitude = health.clamp(0.0, 1.0) * 0.7;

            params.push(AudioParam {
                frequency: base_freq,
                amplitude,
                pan,
                duration_secs: 0.15,
            });
        }
    }
    params
}

/// Convert a GameScene to haptic commands.
///
/// Entities become haptic pulses: position → spatial location on device,
/// health → intensity, entity type → pattern. Combat events get stronger
/// pulses so the user feels the action.
#[must_use]
pub fn hapticize_game_scene(scene_json: &serde_json::Value) -> Vec<HapticCommand> {
    let Some(obj) = scene_json.as_object() else {
        return vec![];
    };

    let mut commands = Vec::new();

    if let Some(entities) = obj.get("entities").and_then(serde_json::Value::as_array) {
        for entity in entities {
            let etype = entity
                .get("entity_type")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("unknown");
            let pos = entity.get("position").and_then(|v| {
                let arr = v.as_array()?;
                Some((arr.first()?.as_f64()?, arr.get(1)?.as_f64()?))
            });
            let health = entity
                .get("health")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(1.0);

            let (ex, ey) = pos.unwrap_or((0.0, 0.0));
            let norm_x = (ex / 64.0).clamp(0.0, 1.0);
            let norm_y = (ey / 64.0).clamp(0.0, 1.0);

            let (pattern, intensity) = match etype {
                "player" => (HapticPattern::Sustained, 0.3 + health * 0.4),
                "enemy" => (HapticPattern::Pulse, 0.5 + (1.0 - health) * 0.5),
                "projectile" => (HapticPattern::Ramp, 0.8),
                "item" => (HapticPattern::Texture, 0.2),
                _ => (HapticPattern::Pulse, 0.3),
            };

            commands.push(HapticCommand {
                intensity: intensity.clamp(0.0, 1.0),
                duration_secs: 0.15,
                position: [norm_x, norm_y],
                pattern,
            });
        }
    }
    commands
}

/// Convert a Soundscape definition to haptic commands.
///
/// Each sound layer becomes a haptic channel: frequency → pattern speed,
/// amplitude → intensity, pan → spatial position. A deaf user feels the
/// rhythm and spatial distribution of the soundscape.
#[must_use]
pub fn hapticize_soundscape(definition: &serde_json::Value) -> Vec<HapticCommand> {
    let Some(layers) = definition
        .get("layers")
        .and_then(serde_json::Value::as_array)
    else {
        return vec![];
    };

    let mut commands = Vec::new();
    for layer in layers {
        let amp = layer
            .get("amplitude")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        let pan = layer
            .get("pan")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        let freq = layer
            .get("frequency")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(440.0);

        let pattern = if freq < 200.0 {
            HapticPattern::Sustained
        } else if freq < 600.0 {
            HapticPattern::Pulse
        } else {
            HapticPattern::Ramp
        };
        let norm_x = f64::midpoint(pan, 1.0).clamp(0.0, 1.0);

        commands.push(HapticCommand {
            intensity: amp.clamp(0.0, 1.0),
            duration_secs: 0.2,
            position: [norm_x, 0.5],
            pattern,
        });
    }
    commands
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
