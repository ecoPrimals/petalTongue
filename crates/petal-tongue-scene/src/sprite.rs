// SPDX-License-Identifier: AGPL-3.0-only
//! 2D sprite and tilemap primitives for game visualization.
//!
//! Provides the scene-graph primitives needed for ludoSpring and other
//! primals that push 2D game state to petalTongue for rendering.
//!
//! # Architecture
//!
//! These types are pure data — no rendering logic. Rendering is handled by
//! the `visual_2d` pipeline in `petal-tongue-graph` or the egui/ratatui
//! backends in `petal-tongue-ui`/`petal-tongue-tui`.

use serde::{Deserialize, Serialize};

/// A 2D sprite: a positioned, scaled, optionally rotated image region.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sprite {
    /// Unique entity identifier.
    pub id: String,
    /// Position in world coordinates.
    pub position: [f64; 2],
    /// Size in world units (width, height).
    pub size: [f64; 2],
    /// Rotation in radians (counter-clockwise from +x).
    #[serde(default)]
    pub rotation: f64,
    /// RGBA tint color (0-255 per channel). White = no tint.
    #[serde(default = "default_white")]
    pub tint: [u8; 4],
    /// Sprite sheet region (u, v, w, h) in normalized 0.0-1.0 coords.
    /// `None` means the entire texture.
    #[serde(default)]
    pub uv_rect: Option<[f64; 4]>,
    /// Texture/atlas identifier (capability-discovered, not a file path).
    #[serde(default)]
    pub texture_id: Option<String>,
    /// Z-order for depth sorting (higher = in front).
    #[serde(default)]
    pub z_order: i32,
    /// Whether this sprite is visible.
    #[serde(default = "default_true")]
    pub visible: bool,
    /// Optional label for UI display.
    #[serde(default)]
    pub label: Option<String>,
}

/// A tile in a tilemap grid.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct Tile {
    /// Tile type index (maps to a palette or sprite sheet region).
    pub tile_type: u16,
    /// RGBA color override (0 = use palette default).
    #[serde(default)]
    pub color: [u8; 4],
    /// Whether this tile blocks movement (for game logic display).
    #[serde(default)]
    pub solid: bool,
}

/// A 2D grid of tiles for map/level visualization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tilemap {
    /// Grid dimensions (columns, rows).
    pub dimensions: [u32; 2],
    /// Tile size in world units (width, height).
    pub tile_size: [f64; 2],
    /// Origin position in world coordinates (top-left corner).
    #[serde(default)]
    pub origin: [f64; 2],
    /// Row-major tile data (length = columns * rows).
    pub tiles: Vec<Tile>,
    /// Named color palette: `tile_type` → RGBA.
    #[serde(default)]
    pub palette: Vec<[u8; 4]>,
}

impl Tilemap {
    /// Get a tile at grid coordinates. Returns `None` if out of bounds.
    #[must_use]
    pub fn get(&self, col: u32, row: u32) -> Option<&Tile> {
        if col < self.dimensions[0] && row < self.dimensions[1] {
            let idx = (row * self.dimensions[0] + col) as usize;
            self.tiles.get(idx)
        } else {
            None
        }
    }

    /// World-space bounding box: (`min_x`, `min_y`, `max_x`, `max_y`).
    #[must_use]
    pub fn bounds(&self) -> [f64; 4] {
        let w = f64::from(self.dimensions[0]) * self.tile_size[0];
        let h = f64::from(self.dimensions[1]) * self.tile_size[1];
        [
            self.origin[0],
            self.origin[1],
            self.origin[0] + w,
            self.origin[1] + h,
        ]
    }

    /// Resolve tile color: tile's own color if non-zero, otherwise palette lookup.
    #[must_use]
    pub fn tile_color(&self, tile: &Tile) -> [u8; 4] {
        if tile.color == [0, 0, 0, 0] {
            self.palette
                .get(tile.tile_type as usize)
                .copied()
                .unwrap_or([128, 128, 128, 255])
        } else {
            tile.color
        }
    }
}

/// A game entity with position, velocity, and visual state.
///
/// Used for characters, projectiles, items, and other dynamic objects
/// that springs push for visualization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEntity {
    /// Unique entity identifier.
    pub id: String,
    /// Entity type tag (e.g. "player", "enemy", "projectile", "item").
    pub entity_type: String,
    /// World position.
    pub position: [f64; 2],
    /// Velocity (units/sec) for interpolation/trail rendering.
    #[serde(default)]
    pub velocity: [f64; 2],
    /// Health / hit points (0.0-1.0 normalized for bar rendering).
    #[serde(default)]
    pub health: Option<f64>,
    /// RGBA color.
    #[serde(default = "default_white")]
    pub color: [u8; 4],
    /// Size in world units.
    #[serde(default = "default_entity_size")]
    pub size: [f64; 2],
    /// Display label.
    #[serde(default)]
    pub label: Option<String>,
}

/// A complete 2D game scene pushed by a spring for visualization.
///
/// This is the top-level structure that `visualization.render` can accept
/// alongside `DataBinding` variants. Springs push this as part of their
/// game state for petalTongue to render.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameScene {
    /// Optional tilemap background.
    #[serde(default)]
    pub tilemap: Option<Tilemap>,
    /// Sprites (static or animated).
    #[serde(default)]
    pub sprites: Vec<Sprite>,
    /// Dynamic game entities.
    #[serde(default)]
    pub entities: Vec<GameEntity>,
    /// Camera center in world coordinates.
    #[serde(default)]
    pub camera_center: [f64; 2],
    /// Camera zoom level (1.0 = default).
    #[serde(default = "default_one")]
    pub camera_zoom: f64,
}

impl Default for GameScene {
    fn default() -> Self {
        Self {
            tilemap: None,
            sprites: Vec::new(),
            entities: Vec::new(),
            camera_center: [0.0, 0.0],
            camera_zoom: 1.0,
        }
    }
}

const fn default_white() -> [u8; 4] {
    [255, 255, 255, 255]
}

const fn default_true() -> bool {
    true
}

const fn default_entity_size() -> [f64; 2] {
    [1.0, 1.0]
}

const fn default_one() -> f64 {
    1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sprite_serde_round_trip() {
        let sprite = Sprite {
            id: "player".to_string(),
            position: [10.0, 20.0],
            size: [32.0, 32.0],
            rotation: 0.0,
            tint: [255, 255, 255, 255],
            uv_rect: Some([0.0, 0.0, 0.25, 0.25]),
            texture_id: Some("characters".to_string()),
            z_order: 10,
            visible: true,
            label: Some("Player 1".to_string()),
        };
        let json = serde_json::to_string(&sprite).unwrap();
        let back: Sprite = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "player");
        assert_eq!(back.z_order, 10);
    }

    #[test]
    fn tilemap_get_in_bounds() {
        let map = Tilemap {
            dimensions: [3, 2],
            tile_size: [16.0, 16.0],
            origin: [0.0, 0.0],
            tiles: vec![Tile::default(); 6],
            palette: vec![[0, 128, 0, 255]],
        };
        assert!(map.get(0, 0).is_some());
        assert!(map.get(2, 1).is_some());
        assert!(map.get(3, 0).is_none());
        assert!(map.get(0, 2).is_none());
    }

    #[test]
    fn tilemap_bounds() {
        let map = Tilemap {
            dimensions: [10, 8],
            tile_size: [16.0, 16.0],
            origin: [5.0, 10.0],
            tiles: vec![Tile::default(); 80],
            palette: vec![],
        };
        let b = map.bounds();
        assert!((b[0] - 5.0).abs() < f64::EPSILON);
        assert!((b[1] - 10.0).abs() < f64::EPSILON);
        assert!((b[2] - 165.0).abs() < f64::EPSILON);
        assert!((b[3] - 138.0).abs() < f64::EPSILON);
    }

    #[test]
    fn tilemap_color_resolution() {
        let map = Tilemap {
            dimensions: [1, 1],
            tile_size: [1.0, 1.0],
            origin: [0.0, 0.0],
            tiles: vec![Tile {
                tile_type: 0,
                color: [0, 0, 0, 0],
                solid: false,
            }],
            palette: vec![[255, 0, 0, 255]],
        };
        assert_eq!(map.tile_color(&map.tiles[0]), [255, 0, 0, 255]);

        let override_tile = Tile {
            tile_type: 0,
            color: [0, 255, 0, 255],
            solid: false,
        };
        assert_eq!(map.tile_color(&override_tile), [0, 255, 0, 255]);
    }

    #[test]
    fn game_entity_defaults() {
        let json = r#"{"id": "e1", "entity_type": "enemy", "position": [5.0, 10.0]}"#;
        let entity: GameEntity = serde_json::from_str(json).unwrap();
        assert_eq!(entity.velocity, [0.0, 0.0]);
        assert!(entity.health.is_none());
        assert_eq!(entity.size, [1.0, 1.0]);
    }

    #[test]
    fn game_scene_empty() {
        let scene = GameScene::default();
        assert!(scene.tilemap.is_none());
        assert!(scene.sprites.is_empty());
        assert!(scene.entities.is_empty());
        assert!((scene.camera_zoom - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn game_scene_serde_round_trip() {
        let scene = GameScene {
            tilemap: Some(Tilemap {
                dimensions: [2, 2],
                tile_size: [16.0, 16.0],
                origin: [0.0, 0.0],
                tiles: vec![Tile::default(); 4],
                palette: vec![[0, 0, 0, 255], [255, 255, 255, 255]],
            }),
            sprites: vec![Sprite {
                id: "s1".to_string(),
                position: [0.0, 0.0],
                size: [16.0, 16.0],
                rotation: 0.0,
                tint: default_white(),
                uv_rect: None,
                texture_id: None,
                z_order: 0,
                visible: true,
                label: None,
            }],
            entities: vec![GameEntity {
                id: "player".to_string(),
                entity_type: "player".to_string(),
                position: [8.0, 8.0],
                velocity: [1.0, 0.0],
                health: Some(0.75),
                color: [0, 128, 255, 255],
                size: [1.0, 1.0],
                label: Some("Hero".to_string()),
            }],
            camera_center: [8.0, 8.0],
            camera_zoom: 2.0,
        };
        let json = serde_json::to_string(&scene).unwrap();
        let back: GameScene = serde_json::from_str(&json).unwrap();
        assert!(back.tilemap.is_some());
        assert_eq!(back.sprites.len(), 1);
        assert_eq!(back.entities.len(), 1);
        assert!((back.camera_zoom - 2.0).abs() < f64::EPSILON);
    }
}
