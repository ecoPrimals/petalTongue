// SPDX-License-Identifier: AGPL-3.0-or-later
//! egui renderer for `DataBinding::GameScene`.
//!
//! Renders tilemaps, sprites, and game entities using `egui::Painter` with
//! camera transform (zoom + pan). Supports z-order depth sorting and
//! health-bar overlays on entities.

use egui::{Align2, Color32, FontId, Pos2, Rect, Rounding, Sense, Stroke, Ui, Vec2, pos2, vec2};
use petal_tongue_scene::sprite::{GameEntity, GameScene, Sprite, Tilemap};

/// Draw a 2D game scene parsed from a `DataBinding::GameScene` JSON value.
///
/// Detects the scene format:
/// - **Tilemap/Sprite scene**: Has `tilemap`, `sprites`, or `entities` fields
/// - **Narrative scene** (RPGPT/esotericWebb): Has `description`, `npcs`, `node` fields
///
/// Gracefully degrades to a label if the scene JSON cannot be deserialized.
pub fn draw_game_scene(ui: &mut Ui, label: &str, scene_json: &serde_json::Value) {
    if is_narrative_scene(scene_json) {
        draw_narrative_scene(ui, label, scene_json);
    } else {
        draw_tilemap_scene(ui, label, scene_json);
    }
}

fn is_narrative_scene(json: &serde_json::Value) -> bool {
    let Some(obj) = json.as_object() else {
        return false;
    };
    obj.contains_key("description") || obj.contains_key("node") || obj.contains_key("npcs")
}

fn draw_tilemap_scene(ui: &mut Ui, label: &str, scene_json: &serde_json::Value) {
    let scene: GameScene = match serde_json::from_value(scene_json.clone()) {
        Ok(s) => s,
        Err(e) => {
            ui.label(format!("🎮 {label} (invalid scene: {e})"));
            return;
        }
    };

    ui.group(|ui| {
        ui.label(
            egui::RichText::new(format!("🎮 {label}"))
                .strong()
                .size(14.0),
        );

        let desired = vec2(
            ui.available_width().max(200.0),
            ui.available_width().clamp(200.0, 400.0),
        );
        let (response, painter) = ui.allocate_painter(desired, Sense::hover());
        let canvas = response.rect;

        let cam = CameraTransform::new(&scene, canvas);

        if let Some(ref tilemap) = scene.tilemap {
            paint_tilemap(&painter, tilemap, &cam);
        }

        let mut drawables: Vec<Drawable> = Vec::new();
        for sprite in &scene.sprites {
            if sprite.visible {
                drawables.push(Drawable::Sprite(sprite));
            }
        }
        for entity in &scene.entities {
            drawables.push(Drawable::Entity(entity));
        }
        drawables.sort_by_key(Drawable::z_order);

        for drawable in &drawables {
            match drawable {
                Drawable::Sprite(s) => paint_sprite(&painter, s, &cam),
                Drawable::Entity(e) => paint_entity(&painter, e, &cam),
            }
        }
    });
}

enum Drawable<'a> {
    Sprite(&'a Sprite),
    Entity(&'a GameEntity),
}

impl Drawable<'_> {
    const fn z_order(&self) -> i32 {
        match self {
            Drawable::Sprite(s) => s.z_order,
            Drawable::Entity(_) => i32::MAX,
        }
    }
}

struct CameraTransform {
    canvas: Rect,
    offset: Vec2,
    scale: f32,
}

impl CameraTransform {
    fn new(scene: &GameScene, canvas: Rect) -> Self {
        let zoom = scene.camera_zoom.max(0.01) as f32;
        let scale = canvas.width().min(canvas.height()) / 32.0 * zoom;
        let center = pos2(scene.camera_center[0] as f32, scene.camera_center[1] as f32);
        let offset = canvas.center().to_vec2() - (center.to_vec2() * scale);
        Self {
            canvas,
            offset,
            scale,
        }
    }

    fn world_to_screen(&self, wx: f64, wy: f64) -> Pos2 {
        pos2(
            wx as f32 * self.scale + self.offset.x,
            wy as f32 * self.scale + self.offset.y,
        )
    }

    fn world_size(&self, w: f64, h: f64) -> Vec2 {
        vec2(w as f32 * self.scale, h as f32 * self.scale)
    }
}

fn rgba_to_color32(rgba: [u8; 4]) -> Color32 {
    Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3])
}

fn paint_tilemap(painter: &egui::Painter, tilemap: &Tilemap, cam: &CameraTransform) {
    let [cols, rows] = tilemap.dimensions;
    for row in 0..rows {
        for col in 0..cols {
            let Some(tile) = tilemap.get(col, row) else {
                continue;
            };
            let color = rgba_to_color32(tilemap.tile_color(tile));
            let wx = tilemap.origin[0] + f64::from(col) * tilemap.tile_size[0];
            let wy = tilemap.origin[1] + f64::from(row) * tilemap.tile_size[1];
            let tl = cam.world_to_screen(wx, wy);
            let size = cam.world_size(tilemap.tile_size[0], tilemap.tile_size[1]);
            let rect = Rect::from_min_size(tl, size);
            if rect.intersects(cam.canvas) {
                painter.rect_filled(rect, Rounding::ZERO, color);
                if tile.solid {
                    painter.rect_stroke(rect, Rounding::ZERO, Stroke::new(1.0, Color32::DARK_GRAY));
                }
            }
        }
    }
}

fn paint_sprite(painter: &egui::Painter, sprite: &Sprite, cam: &CameraTransform) {
    let tl = cam.world_to_screen(
        sprite.position[0] - sprite.size[0] / 2.0,
        sprite.position[1] - sprite.size[1] / 2.0,
    );
    let size = cam.world_size(sprite.size[0], sprite.size[1]);
    let rect = Rect::from_min_size(tl, size);
    let color = rgba_to_color32(sprite.tint);

    painter.rect_filled(rect, Rounding::same(2.0), color);
    painter.rect_stroke(rect, Rounding::same(2.0), Stroke::new(1.0, Color32::GRAY));

    if let Some(ref lbl) = sprite.label {
        painter.text(
            rect.center_bottom() + vec2(0.0, 2.0),
            Align2::CENTER_TOP,
            lbl,
            FontId::proportional(10.0),
            Color32::WHITE,
        );
    }
}

fn paint_entity(painter: &egui::Painter, entity: &GameEntity, cam: &CameraTransform) {
    let center = cam.world_to_screen(entity.position[0], entity.position[1]);
    let half = cam.world_size(entity.size[0] / 2.0, entity.size[1] / 2.0);
    let rect = Rect::from_center_size(center, half * 2.0);
    let color = rgba_to_color32(entity.color);

    let icon = match entity.entity_type.as_str() {
        "player" => "◆",
        "enemy" => "▲",
        "projectile" => "•",
        "item" => "★",
        "npc" => "●",
        _ => "■",
    };

    painter.rect_filled(rect, Rounding::same(3.0), color.gamma_multiply(0.4));
    painter.text(
        center,
        Align2::CENTER_CENTER,
        icon,
        FontId::monospace(14.0),
        color,
    );

    if entity.velocity != [0.0, 0.0] {
        let trail_end = cam.world_to_screen(
            entity.position[0] - entity.velocity[0] * 0.1,
            entity.position[1] - entity.velocity[1] * 0.1,
        );
        painter.line_segment(
            [trail_end, center],
            Stroke::new(1.5, color.gamma_multiply(0.5)),
        );
    }

    if let Some(hp) = entity.health {
        let bar_w = rect.width().max(16.0);
        let bar_h = 3.0_f32;
        let bar_tl = rect.left_top() - vec2(0.0, bar_h + 2.0);
        let bg_rect = Rect::from_min_size(bar_tl, vec2(bar_w, bar_h));
        painter.rect_filled(bg_rect, Rounding::same(1.0), Color32::from_gray(40));

        let hp_clamped = hp.clamp(0.0, 1.0) as f32;
        let hp_color = if hp_clamped > 0.6 {
            Color32::from_rgb(80, 200, 80)
        } else if hp_clamped > 0.3 {
            Color32::from_rgb(220, 180, 40)
        } else {
            Color32::from_rgb(220, 60, 60)
        };
        let fill_rect = Rect::from_min_size(bar_tl, vec2(bar_w * hp_clamped, bar_h));
        painter.rect_filled(fill_rect, Rounding::same(1.0), hp_color);
    }

    if let Some(ref lbl) = entity.label {
        painter.text(
            rect.center_bottom() + vec2(0.0, 2.0),
            Align2::CENTER_TOP,
            lbl,
            FontId::proportional(10.0),
            Color32::WHITE,
        );
    }
}

// ---------------------------------------------------------------------------
// Narrative / RPGPT scene rendering (esotericWebb, ludoSpring dialogue trees)
// ---------------------------------------------------------------------------

/// RPGPT-style narrative scene with description, NPCs, choices.
#[derive(Debug, serde::Deserialize)]
struct NarrativeScene {
    #[serde(default)]
    node: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    npcs: Vec<serde_json::Value>,
    #[serde(default)]
    turn: u32,
    #[serde(default)]
    is_ending: bool,
    #[serde(default)]
    choices: Vec<String>,
    #[serde(default)]
    terrain: Vec<serde_json::Value>,
    #[serde(default)]
    entities: Vec<serde_json::Value>,
    #[serde(default)]
    scene_type: String,
}

#[expect(
    clippy::too_many_lines,
    reason = "narrative scene renderer: sequential UI sections for description, NPCs, choices, combat grid"
)]
fn draw_narrative_scene(ui: &mut Ui, label: &str, scene_json: &serde_json::Value) {
    let scene: NarrativeScene = match serde_json::from_value(scene_json.clone()) {
        Ok(s) => s,
        Err(e) => {
            ui.label(format!("📖 {label} (invalid narrative: {e})"));
            return;
        }
    };

    let scene_kind = detect_narrative_kind(&scene);

    ui.group(|ui| {
        let icon = match scene_kind {
            NarrativeKind::Dialogue => "💬",
            NarrativeKind::Combat => "⚔️",
            NarrativeKind::Exploration => "🗺️",
            NarrativeKind::Narration => "📖",
        };
        ui.label(
            egui::RichText::new(format!("{icon} {label}"))
                .strong()
                .size(14.0),
        );

        if !scene.node.is_empty() {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("Node: {}", scene.node))
                        .monospace()
                        .size(10.0)
                        .color(Color32::GRAY),
                );
                if scene.turn > 0 {
                    ui.label(
                        egui::RichText::new(format!("Turn {}", scene.turn))
                            .size(10.0)
                            .color(Color32::LIGHT_GRAY),
                    );
                }
                if scene.is_ending {
                    ui.label(
                        egui::RichText::new("🏁 ENDING")
                            .size(10.0)
                            .color(Color32::from_rgb(255, 200, 80)),
                    );
                }
            });
        }

        if !scene.description.is_empty() {
            ui.add_space(4.0);
            let desc_frame = egui::Frame::default()
                .inner_margin(egui::Margin::same(8.0))
                .rounding(Rounding::same(4.0))
                .fill(Color32::from_gray(25));
            desc_frame.show(ui, |ui: &mut Ui| {
                ui.label(
                    egui::RichText::new(&scene.description)
                        .size(12.0)
                        .color(Color32::from_rgb(220, 220, 200)),
                );
            });
        }

        if !scene.npcs.is_empty() {
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Characters")
                    .size(11.0)
                    .color(Color32::GRAY),
            );
            for npc in &scene.npcs {
                let name = npc
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown");
                let status = npc
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("present");
                let health = npc.get("health").and_then(serde_json::Value::as_f64);

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("● {name}"))
                            .size(11.0)
                            .color(Color32::from_rgb(180, 200, 255)),
                    );
                    let status_color = match status {
                        "hostile" => Color32::from_rgb(220, 80, 80),
                        "friendly" => Color32::from_rgb(80, 200, 80),
                        "dead" => Color32::from_gray(100),
                        _ => Color32::LIGHT_GRAY,
                    };
                    ui.label(
                        egui::RichText::new(format!("[{status}]"))
                            .size(10.0)
                            .color(status_color),
                    );
                    if let Some(hp) = health {
                        let bar_w = 40.0;
                        let bar_h = 6.0;
                        let (rect, _) = ui.allocate_exact_size(vec2(bar_w, bar_h), Sense::hover());
                        ui.painter()
                            .rect_filled(rect, Rounding::same(2.0), Color32::from_gray(40));
                        let hp_frac = hp.clamp(0.0, 1.0) as f32;
                        let hp_color = if hp_frac > 0.6 {
                            Color32::from_rgb(80, 200, 80)
                        } else if hp_frac > 0.3 {
                            Color32::from_rgb(220, 180, 40)
                        } else {
                            Color32::from_rgb(220, 60, 60)
                        };
                        let fill =
                            Rect::from_min_size(rect.left_top(), vec2(bar_w * hp_frac, bar_h));
                        ui.painter()
                            .rect_filled(fill, Rounding::same(2.0), hp_color);
                    }
                });
            }
        }

        if !scene.choices.is_empty() {
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Choices")
                    .size(11.0)
                    .color(Color32::GRAY),
            );
            for (i, choice) in scene.choices.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("  {}.", i + 1))
                            .monospace()
                            .size(11.0)
                            .color(Color32::from_rgb(120, 180, 255)),
                    );
                    ui.label(
                        egui::RichText::new(choice)
                            .size(11.0)
                            .color(Color32::from_rgb(200, 200, 220)),
                    );
                });
            }
        }

        if matches!(scene_kind, NarrativeKind::Combat) && !scene.entities.is_empty() {
            ui.add_space(4.0);
            paint_combat_grid(ui, &scene);
        }
    });
}

enum NarrativeKind {
    Dialogue,
    Combat,
    Exploration,
    Narration,
}

fn detect_narrative_kind(scene: &NarrativeScene) -> NarrativeKind {
    match scene.scene_type.as_str() {
        "dialogue_tree" | "dialogue" => NarrativeKind::Dialogue,
        "combat_grid" | "combat" => NarrativeKind::Combat,
        "exploration_map" | "exploration" => NarrativeKind::Exploration,
        "narration" | "narration_stream" => NarrativeKind::Narration,
        _ => {
            if !scene.terrain.is_empty()
                || scene.entities.iter().any(|e| e.get("position").is_some())
            {
                NarrativeKind::Combat
            } else if !scene.choices.is_empty() || !scene.npcs.is_empty() {
                NarrativeKind::Dialogue
            } else {
                NarrativeKind::Narration
            }
        }
    }
}

fn paint_combat_grid(ui: &mut Ui, scene: &NarrativeScene) {
    ui.label(
        egui::RichText::new("⚔️ Combat Grid")
            .size(11.0)
            .color(Color32::GRAY),
    );
    let grid_size = vec2(
        ui.available_width().max(200.0),
        200.0_f32.min(ui.available_width()),
    );
    let (response, painter) = ui.allocate_painter(grid_size, Sense::hover());
    let rect = response.rect;

    painter.rect_filled(rect, Rounding::same(3.0), Color32::from_gray(20));

    let cell = 24.0_f32;
    let cols = (rect.width() / cell) as u32;
    let rows = (rect.height() / cell) as u32;
    for r in 0..rows {
        for c in 0..cols {
            let tl = rect.left_top() + vec2(c as f32 * cell, r as f32 * cell);
            let cell_rect = Rect::from_min_size(tl, vec2(cell, cell));
            painter.rect_stroke(
                cell_rect,
                Rounding::ZERO,
                Stroke::new(0.3, Color32::from_gray(40)),
            );
        }
    }

    for entity in &scene.entities {
        let pos = entity.get("position").and_then(|p| {
            let arr = p.as_array()?;
            Some([arr.first()?.as_f64()?, arr.get(1)?.as_f64()?])
        });
        let Some([ex, ey]) = pos else { continue };
        let entity_type = entity
            .get("entity_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let name = entity
            .get("label")
            .or_else(|| entity.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let screen_pos =
            rect.left_top() + vec2(ex as f32 * cell + cell / 2.0, ey as f32 * cell + cell / 2.0);
        if !rect.contains(screen_pos) {
            continue;
        }

        let (icon, color) = match entity_type {
            "player" => ("◆", Color32::from_rgb(80, 180, 255)),
            "enemy" => ("▲", Color32::from_rgb(220, 80, 80)),
            "npc" | "ally" => ("●", Color32::from_rgb(80, 200, 80)),
            _ => ("■", Color32::LIGHT_GRAY),
        };
        painter.text(
            screen_pos,
            Align2::CENTER_CENTER,
            icon,
            FontId::monospace(14.0),
            color,
        );
        if !name.is_empty() {
            painter.text(
                screen_pos + vec2(0.0, cell / 2.0 + 2.0),
                Align2::CENTER_TOP,
                name,
                FontId::proportional(8.0),
                color.gamma_multiply(0.8),
            );
        }
    }
}

#[cfg(test)]
#[path = "game_scene_renderer_tests.rs"]
mod tests;
