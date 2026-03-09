// SPDX-License-Identifier: AGPL-3.0-only
//! Animation rendering for flow particles and node pulses.

use egui::{Color32, Pos2, Stroke};
use petal_tongue_animation::AnimationEngine;
use petal_tongue_core::GraphEngine;
use petal_tongue_core::graph_engine::Position;

use super::renderer::Visual2DRenderer;

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

                let x = source_pos.x + (target_pos.x - source_pos.x) * particle.progress;
                let y = source_pos.y + (target_pos.y - source_pos.y) * particle.progress;

                let world_pos = Position::new_2d(x, y);
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
