// SPDX-License-Identifier: AGPL-3.0-only
//! Animation rendering for flow particles and node pulses.

use egui::{Color32, Pos2, Stroke};
use petal_tongue_animation::AnimationEngine;
use petal_tongue_core::GraphEngine;
use petal_tongue_core::graph_engine::Position;

use super::renderer::Visual2DRenderer;

#[must_use]
fn lerp_position(source: Position, target: Position, t: f32) -> Position {
    let x = source.x + (target.x - source.x) * t;
    let y = source.y + (target.y - source.y) * t;
    Position::new_2d(x, y)
}

/// Renders flow particles along edges and node pulse effects.
pub fn render_animation(
    renderer: &Visual2DRenderer,
    painter: &egui::Painter,
    animation_engine: &AnimationEngine,
    graph: &GraphEngine,
    screen_center: Pos2,
) {
    for edge_anim in &animation_engine.edge_animations {
        for particle in &edge_anim.particles {
            if let (Some(source_node), Some(target_node)) = (
                graph.get_node(&edge_anim.source),
                graph.get_node(&edge_anim.target),
            ) {
                let source_pos = source_node.position;
                let target_pos = target_node.position;
                let world_pos = lerp_position(source_pos, target_pos, particle.progress);
                let screen_pos = renderer.world_to_screen(world_pos, screen_center);

                painter.circle_filled(
                    screen_pos,
                    4.0 * renderer.zoom,
                    Color32::from_rgb(100, 200, 255),
                );
            }
        }
    }

    for pulse in &animation_engine.node_pulses {
        if let Some(node) = graph.get_node(&pulse.node_id) {
            let screen_pos = renderer.world_to_screen(node.position, screen_center);

            let pulse_radius = 25.0 * renderer.zoom * pulse.radius_multiplier();
            #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let alpha = (255.0 * pulse.alpha()) as u8;

            painter.circle_stroke(
                screen_pos,
                pulse_radius,
                Stroke::new(2.0, Color32::from_rgba_premultiplied(100, 200, 255, alpha)),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::graph_engine::Position;

    #[test]
    fn lerp_position_start() {
        let a = Position::new_2d(0.0, 0.0);
        let b = Position::new_2d(100.0, 50.0);
        let r = lerp_position(a, b, 0.0);
        assert!((r.x - 0.0).abs() < 0.001);
        assert!((r.y - 0.0).abs() < 0.001);
    }

    #[test]
    fn lerp_position_end() {
        let a = Position::new_2d(0.0, 0.0);
        let b = Position::new_2d(100.0, 50.0);
        let r = lerp_position(a, b, 1.0);
        assert!((r.x - 100.0).abs() < 0.001);
        assert!((r.y - 50.0).abs() < 0.001);
    }

    #[test]
    fn lerp_position_mid() {
        let a = Position::new_2d(0.0, 0.0);
        let b = Position::new_2d(100.0, 50.0);
        let r = lerp_position(a, b, 0.5);
        assert!((r.x - 50.0).abs() < 0.001);
        assert!((r.y - 25.0).abs() < 0.001);
    }
}
