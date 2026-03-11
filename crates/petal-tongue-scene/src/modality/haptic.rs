// SPDX-License-Identifier: AGPL-3.0-only

use serde::{Deserialize, Serialize};

use crate::primitive::Primitive;
use crate::scene_graph::SceneGraph;

use super::{ModalityCompiler, ModalityOutput};

/// A haptic feedback command for force-feedback or vibration devices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HapticCommand {
    /// Intensity 0.0 to 1.0.
    pub intensity: f64,
    /// Duration in seconds.
    pub duration_secs: f64,
    /// Spatial position hint (normalized 0.0–1.0 on each axis).
    pub position: [f64; 2],
    /// Pattern type.
    pub pattern: HapticPattern,
}

/// Haptic feedback patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HapticPattern {
    /// Single pulse (data point).
    Pulse,
    /// Sustained vibration (line/edge).
    Sustained,
    /// Rising ramp (increasing value).
    Ramp,
    /// Texture (area fill).
    Texture,
}

/// Compiles scene graph to haptic feedback commands.
///
/// Maps data-carrying primitives to tactile feedback: position → spatial
/// location on a haptic surface, size → intensity, type → pattern.
#[derive(Debug, Clone, Default)]
pub struct HapticCompiler;

impl HapticCompiler {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl ModalityCompiler for HapticCompiler {
    fn name(&self) -> &'static str {
        "HapticCompiler"
    }

    fn compile(&self, scene: &SceneGraph) -> ModalityOutput {
        let mut commands = Vec::new();

        for (transform, prim) in scene.flatten() {
            if !prim.carries_data() {
                continue;
            }

            let (x, y, intensity, pattern) = match prim {
                Primitive::Point { x, y, radius, .. } => {
                    let (tx, ty) = transform.apply(*x, *y);
                    let intensity = (*radius / 10.0).clamp(0.1, 1.0);
                    (tx, ty, intensity, HapticPattern::Pulse)
                }
                Primitive::Line { points, .. } => {
                    if points.is_empty() {
                        continue;
                    }
                    let (tx, ty) = transform.apply(points[0][0], points[0][1]);
                    (tx, ty, 0.5, HapticPattern::Sustained)
                }
                Primitive::Rect {
                    x,
                    y,
                    width,
                    height,
                    ..
                } => {
                    let (tx, ty) = transform.apply(*x, *y);
                    let area = (width * height).sqrt() / 100.0;
                    (tx, ty, area.clamp(0.1, 1.0), HapticPattern::Texture)
                }
                _ => continue,
            };

            commands.push(HapticCommand {
                intensity,
                duration_secs: 0.1,
                position: [(x / 800.0).clamp(0.0, 1.0), (y / 600.0).clamp(0.0, 1.0)],
                pattern,
            });
        }

        ModalityOutput::HapticCommands(commands)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::{Color, Primitive};
    use crate::scene_graph::{SceneGraph, SceneNode};

    #[test]
    fn haptic_compiler_produces_commands() {
        let mut graph = SceneGraph::new();
        let prim = Primitive::Point {
            x: 100.0,
            y: 200.0,
            radius: 5.0,
            fill: Some(Color::BLACK),
            stroke: None,
            data_id: Some("d1".to_string()),
        };
        graph.add_to_root(SceneNode::new("p").with_primitive(prim));
        let out = HapticCompiler::new().compile(&graph);
        let ModalityOutput::HapticCommands(cmds) = &out else {
            panic!("expected HapticCommands");
        };
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].pattern, HapticPattern::Pulse);
        assert!(cmds[0].intensity > 0.0);
    }

    #[test]
    fn haptic_compiler_skips_non_data() {
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
        let out = HapticCompiler::new().compile(&graph);
        let ModalityOutput::HapticCommands(cmds) = &out else {
            panic!("expected HapticCommands");
        };
        assert_eq!(cmds.len(), 0);
    }
}
