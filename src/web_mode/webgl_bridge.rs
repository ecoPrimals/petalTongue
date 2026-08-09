// SPDX-License-Identifier: AGPL-3.0-or-later
//! WebGL compilation bridge — DoomFrame / SceneGraph → WebGlScene → scene stream.
//!
//! Provides the compilation path for the G19 WebGL pipeline:
//! 1. A `SceneGraph` or `DoomFrame` is compiled to `WebGlScene` (vertex/index buffers)
//! 2. The JSON-serialized scene is published through the broadcast channel to all
//!    connected `/ws/scene` WebSocket clients.
//!
//! This module bridges the scene engine (`petal-tongue-scene`) with the real-time
//! push delivery layer (`scene_stream`).

#![allow(dead_code)] // Public API consumed by esotericWebb/footPrint integrations

use petal_tongue_scene::modality::webgl::{
    DrawCall, Topology, WebGlCompiler, WebGlScene,
};
use petal_tongue_scene::modality::{ModalityCompiler, ModalityOutput};
use petal_tongue_scene::scene_graph::SceneGraph;
use tokio::sync::broadcast;

use super::scene_stream::{SceneFrame, publish_scene_frame};

/// Compile a `SceneGraph` to `WebGlScene` and publish it to the scene stream.
///
/// Returns `true` if the frame was successfully serialized and sent (at least one
/// receiver was available), `false` otherwise (no subscribers or serialization error).
pub fn compile_and_publish_scene(
    tx: &broadcast::Sender<SceneFrame>,
    session_id: &str,
    scene: &SceneGraph,
) -> bool {
    let compiler = WebGlCompiler::new();
    let output = compiler.compile(scene);

    let json = match output {
        ModalityOutput::GpuCommands(bytes) => {
            String::from_utf8_lossy(&bytes).into_owned()
        }
        _ => return false,
    };

    publish_scene_frame(tx, session_id, json);
    true
}

/// Compile a set of colored rectangles (DoomFrame-compatible) directly to `WebGlScene`.
///
/// This avoids coupling to `doom-core` by accepting raw rectangle data.
/// Each rect becomes a two-triangle quad in the vertex buffer with a corresponding draw call.
#[must_use]
pub fn compile_rects_to_webgl(
    rects: &[RectInput],
    viewport_width: f32,
    viewport_height: f32,
) -> WebGlScene {
    let mut vertices: Vec<f32> = Vec::with_capacity(rects.len() * 28);
    let mut indices: Vec<u32> = Vec::with_capacity(rects.len() * 6);
    let mut draw_calls: Vec<DrawCall> = Vec::with_capacity(rects.len());

    for rect in rects {
        #[expect(clippy::cast_possible_truncation, reason = "vertex index fits u32")]
        let base = (vertices.len() / 7) as u32;
        let base_index = indices.len();

        let x = rect.x as f32;
        let y = rect.y as f32;
        let w = rect.width as f32;
        let h = rect.height as f32;

        // Four corners: position (3×f32) + color (4×f32)
        vertices.extend_from_slice(&[x, y, 0.0, rect.r, rect.g, rect.b, rect.a]);
        vertices.extend_from_slice(&[x + w, y, 0.0, rect.r, rect.g, rect.b, rect.a]);
        vertices.extend_from_slice(&[x + w, y + h, 0.0, rect.r, rect.g, rect.b, rect.a]);
        vertices.extend_from_slice(&[x, y + h, 0.0, rect.r, rect.g, rect.b, rect.a]);

        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);

        #[expect(clippy::cast_possible_truncation, reason = "index count fits u32")]
        draw_calls.push(DrawCall {
            index_offset: base_index as u32,
            index_count: 6,
            topology: Topology::Triangles,
            data_id: Some(rect.data_id.clone()),
        });
    }

    WebGlScene {
        vertices,
        indices,
        draw_calls,
        view_projection: ortho_identity(viewport_width, viewport_height),
        viewport: [viewport_width, viewport_height],
    }
}

/// Compile rectangles to WebGL JSON and publish to the scene stream.
///
/// Combines [`compile_rects_to_webgl`] + serialization + broadcast publish.
pub fn compile_rects_and_publish(
    tx: &broadcast::Sender<SceneFrame>,
    session_id: &str,
    rects: &[RectInput],
    viewport_width: f32,
    viewport_height: f32,
) -> bool {
    let scene = compile_rects_to_webgl(rects, viewport_width, viewport_height);
    let Ok(json) = serde_json::to_string(&scene) else {
        return false;
    };
    publish_scene_frame(tx, session_id, json);
    true
}

/// Rectangle input for WebGL compilation (matches DoomFrame's `FrameRect` layout).
#[derive(Debug, Clone)]
pub struct RectInput {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
    pub data_id: String,
}

/// Orthographic projection matrix for 2D rendering (maps pixel coords to clip space).
fn ortho_identity(width: f32, height: f32) -> [f32; 16] {
    let r = width / 2.0;
    let t = height / 2.0;
    [
        1.0 / r, 0.0, 0.0, 0.0,
        0.0, 1.0 / t, 0.0, 0.0,
        0.0, 0.0, -1.0, 0.0,
        -1.0, -1.0, 0.0, 1.0,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_rects_produces_valid_webgl_scene() {
        let rects = vec![
            RectInput {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 50.0,
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
                data_id: "sky".to_owned(),
            },
            RectInput {
                x: 10.0,
                y: 50.0,
                width: 80.0,
                height: 30.0,
                r: 0.0,
                g: 0.5,
                b: 0.0,
                a: 1.0,
                data_id: "wall:1:0".to_owned(),
            },
        ];

        let scene = compile_rects_to_webgl(&rects, 320.0, 200.0);

        assert_eq!(scene.vertices.len(), 2 * 4 * 7); // 2 rects × 4 verts × 7 floats
        assert_eq!(scene.indices.len(), 2 * 6); // 2 rects × 6 indices (2 triangles)
        assert_eq!(scene.draw_calls.len(), 2);
        assert_eq!(scene.draw_calls[0].data_id.as_deref(), Some("sky"));
        assert_eq!(scene.draw_calls[1].data_id.as_deref(), Some("wall:1:0"));
        assert_eq!(scene.viewport, [320.0, 200.0]);
    }

    #[test]
    fn compile_rects_empty_input() {
        let scene = compile_rects_to_webgl(&[], 640.0, 480.0);
        assert!(scene.vertices.is_empty());
        assert!(scene.indices.is_empty());
        assert!(scene.draw_calls.is_empty());
    }

    #[test]
    fn ortho_identity_nonzero() {
        let m = ortho_identity(800.0, 600.0);
        let nonzero_count = m.iter().filter(|&&v| v != 0.0).count();
        assert!(nonzero_count >= 4, "ortho matrix should have non-zero elements");
    }

    #[test]
    fn webgl_scene_serializes_to_json() {
        let rects = vec![RectInput {
            x: 5.0,
            y: 10.0,
            width: 50.0,
            height: 25.0,
            r: 0.2,
            g: 0.4,
            b: 0.6,
            a: 0.8,
            data_id: "test".to_owned(),
        }];
        let scene = compile_rects_to_webgl(&rects, 320.0, 200.0);
        let json = serde_json::to_string(&scene);
        assert!(json.is_ok());
        let json = json.unwrap();
        assert!(json.contains("\"vertices\""));
        assert!(json.contains("\"indices\""));
        assert!(json.contains("\"draw_calls\""));
    }
}
