// SPDX-License-Identifier: AGPL-3.0-only

use crate::primitive::Primitive;
use crate::scene_graph::SceneGraph;

use super::{AudioParam, ModalityCompiler, ModalityOutput};

/// Compiles scene graph to audio parameters.
/// Maps data-carrying primitives: x→pan, y→frequency, size→amplitude.
#[derive(Debug, Clone, Default)]
pub struct AudioCompiler;

impl AudioCompiler {
    /// Create a new audio compiler.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl ModalityCompiler for AudioCompiler {
    fn name(&self) -> &'static str {
        "AudioCompiler"
    }

    fn compile(&self, scene: &SceneGraph) -> ModalityOutput {
        let mut params = Vec::new();
        for (transform, prim) in scene.flatten() {
            if !prim.carries_data() {
                continue;
            }
            let (x, y, size) = match prim {
                Primitive::Point { x, y, radius, .. } => {
                    let (sx, sy) = transform.apply(*x, *y);
                    (sx, sy, *radius)
                }
                Primitive::Rect {
                    x,
                    y,
                    width,
                    height,
                    ..
                } => {
                    let (sx, sy) = transform.apply(*x, *y);
                    let s = (width * height).sqrt() / 100.0;
                    (sx, sy, s)
                }
                Primitive::Line { points, .. } => {
                    if points.is_empty() {
                        continue;
                    }
                    let (sx, sy) = transform.apply(points[0][0], points[0][1]);
                    (sx, sy, 1.0)
                }
                _ => continue,
            };
            // Normalize to typical ranges: x→pan [-1,1], y→freq [200,2000], size→amp [0,1]
            let pan = (x / 400.0 - 0.5) * 2.0;
            let pan = pan.clamp(-1.0, 1.0);
            let freq = 200.0 + (y / 600.0) * 1800.0;
            let freq = freq.clamp(200.0, 2000.0);
            let amplitude = (size / 10.0).clamp(0.0, 1.0);
            params.push(AudioParam {
                frequency: freq,
                amplitude,
                pan,
                duration_secs: 0.1,
            });
        }
        ModalityOutput::AudioParams(params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::{Color, Primitive};
    use crate::scene_graph::{SceneGraph, SceneNode};

    #[test]
    fn audio_compiler_produces_params_from_points() {
        let mut graph = SceneGraph::new();
        let prim = Primitive::Point {
            x: 100.0,
            y: 300.0,
            radius: 5.0,
            fill: None,
            stroke: None,
            data_id: Some("d1".to_string()),
        };
        graph.add_to_root(SceneNode::new("p").with_primitive(prim));
        let out = AudioCompiler::new().compile(&graph);
        let ModalityOutput::AudioParams(params) = &out else {
            panic!("expected AudioParams");
        };
        assert_eq!(params.len(), 1);
        assert!(params[0].frequency >= 200.0 && params[0].frequency <= 2000.0);
        assert!(params[0].amplitude >= 0.0 && params[0].amplitude <= 1.0);
    }

    #[test]
    fn audio_compiler_handles_rect() {
        let mut graph = SceneGraph::new();
        let prim = Primitive::Rect {
            x: 200.0,
            y: 300.0,
            width: 50.0,
            height: 50.0,
            fill: Some(Color::BLACK),
            stroke: None,
            corner_radius: 0.0,
            data_id: Some("r1".to_string()),
        };
        graph.add_to_root(SceneNode::new("r").with_primitive(prim));
        let out = AudioCompiler::new().compile(&graph);
        let ModalityOutput::AudioParams(params) = &out else {
            panic!("expected AudioParams");
        };
        assert_eq!(params.len(), 1);
        assert!(params[0].amplitude > 0.0);
    }

    #[test]
    fn audio_compiler_skips_non_data_primitives() {
        let mut graph = SceneGraph::new();
        let prim = Primitive::Point {
            x: 100.0,
            y: 200.0,
            radius: 5.0,
            fill: Some(Color::BLACK),
            stroke: None,
            data_id: None,
        };
        graph.add_to_root(SceneNode::new("p").with_primitive(prim));
        let out = AudioCompiler::new().compile(&graph);
        let ModalityOutput::AudioParams(params) = &out else {
            panic!("expected AudioParams");
        };
        assert_eq!(params.len(), 0);
    }
}
