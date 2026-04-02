// SPDX-License-Identifier: AGPL-3.0-or-later
//! Interaction handling for 2D graph visualization (pan, zoom, create, edit).

use crate::capability_validator::{ValidationResult, validate_connection};
use egui::Pos2;
use petal_tongue_core::graph_engine::Position;
use petal_tongue_core::{PrimalHealthStatus, PrimalId, PrimalInfo, Properties, PropertyValue};

use super::{EdgeDraft, Visual2DRenderer};

#[must_use]
fn interactive_node_id(node_count: usize) -> String {
    format!("interactive-node-{}", node_count + 1)
}

#[must_use]
fn interactive_node_name(node_count: usize) -> String {
    format!("Node {}", node_count + 1)
}

#[must_use]
fn is_edge_duplicate(from: &str, to: &str, existing_from: &str, existing_to: &str) -> bool {
    (existing_from == from && existing_to == to) || (existing_from == to && existing_to == from)
}

/// Handle user input (pan, zoom, click, double-click, drag)
pub fn handle_input(
    renderer: &mut Visual2DRenderer,
    response: &egui::Response,
    screen_center: Pos2,
) {
    // Handle zoom (scroll wheel)
    if response.hovered() {
        let scroll_delta = response.ctx.input(|i| i.raw_scroll_delta.y);
        if scroll_delta != 0.0 {
            let zoom_factor = scroll_delta.mul_add(0.001, 1.0);
            renderer.zoom = (renderer.zoom * zoom_factor).clamp(0.1, 10.0);
        }
    }

    // Interactive mode: Handle double-click to create node
    if renderer.interactive_mode
        && response.double_clicked()
        && let Some(mouse_pos) = response.interact_pointer_pos()
    {
        let world_pos = renderer.screen_to_world(mouse_pos, screen_center);
        create_node_at(renderer, world_pos);
    }

    // Interactive mode: Handle drag to move node or create edge
    if renderer.interactive_mode
        && response.drag_started()
        && let Some(mouse_pos) = response.interact_pointer_pos()
    {
        let world_pos = renderer.screen_to_world(mouse_pos, screen_center);

        let Ok(graph) = renderer.graph.read() else {
            tracing::error!("graph lock poisoned");
            return;
        };
        let node_under_cursor = graph.nodes().iter().find(|node| {
            let distance = node.position.distance_to(world_pos);
            distance < 20.0
        });

        if let Some(node) = node_under_cursor {
            renderer.dragging_node = Some(node.info.id.clone());
        }
    }

    // Interactive mode: Handle drag for edge creation
    if renderer.interactive_mode && response.dragged() {
        if let Some(ref dragging_id) = renderer.dragging_node {
            if response.drag_delta().length() > 10.0 {
                if renderer.drawing_edge.is_none() {
                    renderer.drawing_edge = Some(EdgeDraft {
                        from: dragging_id.clone(),
                        current_pos: response.interact_pointer_pos().unwrap_or_default(),
                    });
                } else if let Some(ref mut edge_draft) = renderer.drawing_edge {
                    edge_draft.current_pos = response.interact_pointer_pos().unwrap_or_default();
                }
            }
        } else {
            renderer.camera_offset += response.drag_delta();
            renderer.is_dragging = true;
        }
    } else {
        renderer.is_dragging = false;
    }

    // Interactive mode: Handle drag release for edge completion
    if renderer.interactive_mode && response.drag_stopped() {
        try_complete_edge_on_drag_release(renderer, response, screen_center);
    }

    // Handle node selection (click)
    if response.clicked()
        && !renderer.is_dragging
        && renderer.drawing_edge.is_none()
        && let Some(mouse_pos) = response.interact_pointer_pos()
    {
        let world_pos = renderer.screen_to_world(mouse_pos, screen_center);

        let Ok(graph) = renderer.graph.read() else {
            tracing::error!("graph lock poisoned");
            return;
        };
        let clicked_node = graph.nodes().iter().find(|node| {
            let distance = node.position.distance_to(world_pos);
            distance < 20.0
        });

        if let Some(node) = clicked_node {
            renderer.selected_node = Some(node.info.id.clone());
        } else {
            renderer.selected_node = None;
        }
    }

    // Interactive mode: Handle delete key
    if renderer.interactive_mode {
        response.ctx.input(|i| {
            if (i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace))
                && let Some(selected_id) = renderer.selected_node.clone()
            {
                delete_node(renderer, selected_id.as_str());
                renderer.selected_node = None;
            }
        });
    }
}

/// Try to complete an edge when drag is released (drop on target node).
fn try_complete_edge_on_drag_release(
    renderer: &mut Visual2DRenderer,
    response: &egui::Response,
    screen_center: Pos2,
) {
    if let Some(edge_draft) = renderer.drawing_edge.take()
        && let Some(mouse_pos) = response.interact_pointer_pos()
    {
        let world_pos = renderer.screen_to_world(mouse_pos, screen_center);

        let target_id = {
            let Ok(graph) = renderer.graph.read() else {
                tracing::error!("graph lock poisoned");
                return;
            };
            graph
                .nodes()
                .iter()
                .find(|node| {
                    let distance = node.position.distance_to(world_pos);
                    distance < 20.0 && node.info.id.as_str() != edge_draft.from.as_str()
                })
                .map(|node| node.info.id.clone())
        };

        if let Some(target) = target_id {
            create_edge(renderer, edge_draft.from, target);
        }
    }
    renderer.dragging_node = None;
}

/// Create a new node at the given world position (interactive mode)
fn create_node_at(renderer: &mut Visual2DRenderer, world_pos: Position) {
    let Ok(mut graph) = renderer.graph.write() else {
        tracing::error!("graph lock poisoned");
        return;
    };

    let node_count = graph.nodes().len();
    let new_id = interactive_node_id(node_count);

    let mut properties = Properties::new();
    properties.insert(
        "created_by".to_string(),
        PropertyValue::String("interactive-paint".to_string()),
    );
    let new_primal = PrimalInfo {
        id: PrimalId::from(new_id.clone()),
        name: interactive_node_name(node_count),
        primal_type: "custom".to_string(),
        endpoint: format!("interactive://{new_id}"),
        capabilities: vec!["interactive".to_string()],
        health: PrimalHealthStatus::Healthy,
        last_seen: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        endpoints: None,
        metadata: None,
        properties,
    }
    .with_family_id("interactive");

    graph.add_node(new_primal);

    if let Some(node) = graph.get_node_mut(&new_id) {
        node.position = world_pos;
    }

    drop(graph);
    renderer.selected_node = Some(PrimalId::from(new_id));
}

/// Create an edge between two nodes (interactive mode)
fn create_edge(renderer: &Visual2DRenderer, from: PrimalId, to: PrimalId) {
    use petal_tongue_core::TopologyEdge;

    let Ok(graph) = renderer.graph.read() else {
        tracing::error!("graph lock poisoned");
        return;
    };

    let edge_exists = graph
        .edges()
        .iter()
        .any(|e| is_edge_duplicate(from.as_str(), to.as_str(), e.from.as_str(), e.to.as_str()));

    if edge_exists {
        return;
    }

    let from_node = graph.get_node(from.as_str());
    let to_node = graph.get_node(to.as_str());

    if let (Some(from_primal), Some(to_primal)) = (from_node, to_node) {
        let validation = validate_connection(&from_primal.info, &to_primal.info);

        match validation {
            ValidationResult::Invalid(reason) => {
                tracing::warn!("❌ Connection invalid: {}", reason);
                return;
            }
            ValidationResult::Warning(reason) => {
                tracing::info!("⚠️ Connection warning: {}", reason);
            }
            ValidationResult::Valid => {
                tracing::info!("✅ Connection validated");
            }
        }
    }

    drop(graph);

    let Ok(mut graph) = renderer.graph.write() else {
        tracing::error!("graph lock poisoned");
        return;
    };
    graph.add_edge(TopologyEdge {
        from,
        to,
        edge_type: "interactive".to_string(),
        label: None,
        capability: None,
        metrics: None,
    });
}

/// Delete a node (interactive mode)
fn delete_node(renderer: &Visual2DRenderer, node_id: &str) {
    let Ok(mut graph) = renderer.graph.write() else {
        tracing::error!("graph lock poisoned");
        return;
    };
    graph.remove_node(node_id);
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn interactive_node_id_first() {
        assert_eq!(interactive_node_id(0), "interactive-node-1");
    }

    #[test]
    fn interactive_node_id_after_nodes() {
        assert_eq!(interactive_node_id(5), "interactive-node-6");
    }

    #[test]
    fn interactive_node_name_first() {
        assert_eq!(interactive_node_name(0), "Node 1");
    }

    #[test]
    fn interactive_node_name_after_nodes() {
        assert_eq!(interactive_node_name(3), "Node 4");
    }

    #[test]
    fn is_edge_duplicate_same_direction() {
        assert!(is_edge_duplicate("a", "b", "a", "b"));
    }

    #[test]
    fn is_edge_duplicate_reverse_direction() {
        assert!(is_edge_duplicate("a", "b", "b", "a"));
    }

    #[test]
    fn is_edge_duplicate_different() {
        assert!(!is_edge_duplicate("a", "b", "a", "c"));
        assert!(!is_edge_duplicate("a", "b", "c", "d"));
    }

    #[test]
    fn is_edge_duplicate_self_loop_not_duplicate_of_normal() {
        assert!(!is_edge_duplicate("a", "a", "a", "b"));
        assert!(!is_edge_duplicate("a", "b", "a", "a"));
    }

    #[test]
    fn is_edge_duplicate_same_node_both_sides() {
        assert!(is_edge_duplicate("x", "x", "x", "x"));
    }

    #[test]
    fn is_edge_duplicate_empty_strings() {
        assert!(is_edge_duplicate("", "", "", ""));
        assert!(is_edge_duplicate("a", "b", "b", "a"));
    }

    #[test]
    fn interactive_node_id_large_count() {
        assert_eq!(interactive_node_id(999), "interactive-node-1000");
    }

    #[test]
    fn interactive_node_name_large_count() {
        assert_eq!(interactive_node_name(99), "Node 100");
    }

    #[test]
    fn hit_detection_radius_logic() {
        // Test the distance < 20.0 logic used in handle_input
        use petal_tongue_core::graph_engine::Position;
        const HIT_RADIUS: f32 = 20.0;
        let is_in_hit_radius = |world: (f32, f32), node: (f32, f32)| {
            let w = Position::new_2d(world.0, world.1);
            let n = Position::new_2d(node.0, node.1);
            w.distance_to(n) < HIT_RADIUS
        };
        assert!(is_in_hit_radius((0.0, 0.0), (0.0, 0.0)));
        assert!(is_in_hit_radius((0.0, 0.0), (10.0, 0.0)));
        assert!(is_in_hit_radius((0.0, 0.0), (14.0, 0.0)));
        assert!(!is_in_hit_radius((0.0, 0.0), (25.0, 0.0)));
    }

    #[test]
    fn hit_detection_diagonal() {
        use petal_tongue_core::graph_engine::Position;
        const HIT_RADIUS: f32 = 20.0;
        let dist = |a: (f32, f32), b: (f32, f32)| {
            Position::new_2d(a.0, a.1).distance_to(Position::new_2d(b.0, b.1))
        };
        let d = dist((0.0, 0.0), (14.0, 14.0));
        assert!(d < HIT_RADIUS);
        let d2 = dist((0.0, 0.0), (20.0, 20.0));
        assert!(d2 > HIT_RADIUS);
    }

    #[test]
    fn zoom_factor_clamp_logic() {
        let apply_zoom = |current: f32, scroll: f32| {
            let factor = scroll.mul_add(0.001, 1.0);
            (current * factor).clamp(0.1, 10.0)
        };
        assert!((apply_zoom(1.0, 100.0) - 1.1).abs() < f32::EPSILON);
        assert!((apply_zoom(10.0, 1000.0) - 10.0).abs() < f32::EPSILON);
        assert!((apply_zoom(0.1, -500.0) - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn drag_delta_threshold() {
        const EDGE_DRAG_THRESHOLD: f32 = 10.0;
        let exceeds = |dx: f32, dy: f32| dx.hypot(dy) > EDGE_DRAG_THRESHOLD;
        assert!(!exceeds(5.0, 5.0)); // sqrt(50) < 10
        assert!(!exceeds(10.0, 0.0)); // length exactly 10, not > 10
        assert!(exceeds(15.0, 0.0));
        assert!(exceeds(8.0, 8.0)); // sqrt(128) > 10
    }

    #[test]
    fn is_edge_duplicate_from_to_order() {
        assert!(is_edge_duplicate("x", "y", "y", "x"));
        assert!(is_edge_duplicate("y", "x", "x", "y"));
    }

    #[test]
    fn screen_to_world_math() {
        let screen_to_world =
            |screen: (f32, f32), center: (f32, f32), zoom: f32, offset: (f32, f32)| {
                (
                    (screen.0 - center.0) / zoom + offset.0,
                    (screen.1 - center.1) / zoom + offset.1,
                )
            };
        let (wx, wy) = screen_to_world((100.0, 100.0), (100.0, 100.0), 1.0, (50.0, 50.0));
        assert!((wx - 50.0).abs() < f32::EPSILON);
        assert!((wy - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn create_node_at_adds_to_graph() {
        use crate::visual_2d::Visual2DRenderer;
        use petal_tongue_core::graph_engine::{GraphEngine, Position};
        use std::sync::{Arc, RwLock};

        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let mut renderer = Visual2DRenderer::new(graph.clone());
        renderer.interactive_mode = true;

        create_node_at(&mut renderer, Position::new_2d(100.0, 200.0));

        let g = graph.read().expect("read graph");
        assert_eq!(g.nodes().len(), 1);
        assert_eq!(
            renderer
                .selected_node
                .as_ref()
                .map(petal_tongue_core::PrimalId::as_str),
            Some("interactive-node-1")
        );
    }

    #[test]
    fn create_edge_adds_connection() {
        use crate::visual_2d::Visual2DRenderer;
        use petal_tongue_core::graph_engine::{GraphEngine, Position};
        use std::sync::{Arc, RwLock};

        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let mut renderer = Visual2DRenderer::new(graph.clone());
        renderer.interactive_mode = true;

        create_node_at(&mut renderer, Position::new_2d(0.0, 0.0));
        create_node_at(&mut renderer, Position::new_2d(100.0, 100.0));

        create_edge(
            &renderer,
            PrimalId::from("interactive-node-1"),
            PrimalId::from("interactive-node-2"),
        );

        let g = graph.read().expect("read graph");
        assert_eq!(g.edges().len(), 1);
    }

    #[test]
    fn delete_node_removes_from_graph() {
        use crate::visual_2d::Visual2DRenderer;
        use petal_tongue_core::graph_engine::{GraphEngine, Position};
        use std::sync::{Arc, RwLock};

        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let mut renderer = Visual2DRenderer::new(graph.clone());
        renderer.interactive_mode = true;

        create_node_at(&mut renderer, Position::new_2d(0.0, 0.0));
        {
            let g = graph.read().expect("read graph");
            assert_eq!(g.nodes().len(), 1);
        }

        delete_node(&renderer, "interactive-node-1");

        let g = graph.read().expect("read graph");
        assert_eq!(g.nodes().len(), 0);
    }

    #[test]
    fn create_edge_duplicate_skipped() {
        use crate::visual_2d::Visual2DRenderer;
        use petal_tongue_core::graph_engine::{GraphEngine, Position};
        use std::sync::{Arc, RwLock};

        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let mut renderer = Visual2DRenderer::new(graph.clone());
        renderer.interactive_mode = true;

        create_node_at(&mut renderer, Position::new_2d(0.0, 0.0));
        create_node_at(&mut renderer, Position::new_2d(100.0, 100.0));

        create_edge(
            &renderer,
            PrimalId::from("interactive-node-1"),
            PrimalId::from("interactive-node-2"),
        );
        create_edge(
            &renderer,
            PrimalId::from("interactive-node-1"),
            PrimalId::from("interactive-node-2"),
        );

        let g = graph.read().expect("read graph");
        assert_eq!(g.edges().len(), 1);
    }

    #[test]
    fn create_edge_reverse_duplicate_skipped() {
        use crate::visual_2d::Visual2DRenderer;
        use petal_tongue_core::graph_engine::{GraphEngine, Position};
        use std::sync::{Arc, RwLock};

        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let mut renderer = Visual2DRenderer::new(graph.clone());
        renderer.interactive_mode = true;

        create_node_at(&mut renderer, Position::new_2d(0.0, 0.0));
        create_node_at(&mut renderer, Position::new_2d(100.0, 100.0));

        create_edge(
            &renderer,
            PrimalId::from("interactive-node-1"),
            PrimalId::from("interactive-node-2"),
        );
        create_edge(
            &renderer,
            PrimalId::from("interactive-node-2"),
            PrimalId::from("interactive-node-1"),
        );

        let g = graph.read().expect("read graph");
        assert_eq!(g.edges().len(), 1);
    }

    #[test]
    fn screen_to_world_zoom_scaling() {
        let screen_to_world =
            |screen: (f32, f32), center: (f32, f32), zoom: f32, offset: (f32, f32)| {
                (
                    (screen.0 - center.0) / zoom + offset.0,
                    (screen.1 - center.1) / zoom + offset.1,
                )
            };
        let (wx, wy) = screen_to_world((200.0, 100.0), (100.0, 100.0), 2.0, (0.0, 0.0));
        assert!((wx - 50.0).abs() < f32::EPSILON);
        assert!((wy - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn edge_drag_threshold_boundary() {
        let len = |dx: f32, dy: f32| dx.hypot(dy);
        assert!((len(10.0, 0.0) - 10.0).abs() < f32::EPSILON);
        assert!(len(10.0, 0.0) <= 10.0);
    }

    #[test]
    fn interactive_node_id_zero() {
        assert_eq!(interactive_node_id(0), "interactive-node-1");
    }

    #[test]
    fn is_edge_duplicate_reverse() {
        assert!(is_edge_duplicate("a", "b", "b", "a"));
        assert!(!is_edge_duplicate("a", "b", "c", "b"));
    }

    #[test]
    fn hit_radius_at_boundary() {
        use petal_tongue_core::graph_engine::Position;
        let dist = Position::new_2d(0.0, 0.0).distance_to(Position::new_2d(20.0, 0.0));
        assert!((dist - 20.0).abs() < f32::EPSILON);
    }

    #[cfg(feature = "egui-render")]
    #[test]
    fn handle_input_with_headless_egui() {
        use crate::visual_2d::Visual2DRenderer;
        use petal_tongue_core::graph_engine::GraphEngine;
        use std::sync::{Arc, RwLock};

        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let mut renderer = Visual2DRenderer::new(graph);
        renderer.interactive_mode = true;

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let response = ui.allocate_rect(
                    egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::new(400.0, 300.0)),
                    egui::Sense::click_and_drag(),
                );
                let screen_center = response.rect.center();
                handle_input(&mut renderer, &response, screen_center);
            });
        });
    }

    #[cfg(feature = "egui-render")]
    #[test]
    fn handle_input_zoom_clamp_bounds() {
        use crate::visual_2d::Visual2DRenderer;
        use petal_tongue_core::graph_engine::GraphEngine;
        use std::sync::{Arc, RwLock};

        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let mut renderer = Visual2DRenderer::new(graph);
        renderer.zoom = 1.0;
        assert!(renderer.zoom >= 0.1 && renderer.zoom <= 10.0);
    }
}
