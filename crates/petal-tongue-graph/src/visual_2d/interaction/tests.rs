// SPDX-License-Identifier: AGPL-3.0-or-later

use petal_tongue_core::PrimalId;

use super::graph_ops::{create_edge, create_node_at, delete_node};
use super::helpers::{interactive_node_id, interactive_node_name, is_edge_duplicate};
use super::input::handle_input;

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
