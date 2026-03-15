// SPDX-License-Identifier: AGPL-3.0-only
//! Tests for 2D visual renderer.

#[cfg(test)]
#[expect(clippy::float_cmp)]
mod visual_2d_tests {
    use super::super::nodes;
    use super::super::*;
    use egui::{Color32, Pos2, Vec2};
    use petal_tongue_animation::AnimationEngine;
    use petal_tongue_core::graph_engine::Position;
    use petal_tongue_core::{GraphEngine, LayoutAlgorithm, PrimalHealthStatus, TopologyEdge};
    use std::sync::{Arc, RwLock};

    fn create_test_graph() -> Arc<RwLock<GraphEngine>> {
        let mut graph = GraphEngine::new();

        let mut node1 = petal_tongue_core::test_fixtures::primals::test_primal("node1");
        node1.name = "Node 1".to_string();
        node1.health = PrimalHealthStatus::Healthy;
        graph.add_node(node1);

        let mut node2 = petal_tongue_core::test_fixtures::primals::test_primal("node2");
        node2.name = "Node 2".to_string();
        node2.health = PrimalHealthStatus::Warning;
        graph.add_node(node2);

        graph.add_edge(TopologyEdge {
            from: "node1".into(),
            to: "node2".into(),
            edge_type: "test".to_string(),
            label: None,
            capability: None,
            metrics: None,
        });

        graph.set_layout(LayoutAlgorithm::Circular);
        graph.layout(1);

        Arc::new(RwLock::new(graph))
    }

    #[test]
    fn test_renderer_creation() {
        let graph = create_test_graph();
        let renderer = Visual2DRenderer::new(graph);
        assert!((renderer.zoom - 1.0).abs() < f32::EPSILON);
        assert_eq!(renderer.camera_offset, Vec2::ZERO);
    }

    #[test]
    fn test_world_to_screen_conversion() {
        let graph = create_test_graph();
        let renderer = Visual2DRenderer::new(graph);

        let world_pos = Position::new_2d(100.0, 50.0);
        let screen_center = Pos2::new(400.0, 300.0);
        let screen_pos = renderer.world_to_screen(world_pos, screen_center);

        assert_eq!(screen_pos.x, 500.0);
        assert_eq!(screen_pos.y, 350.0);
    }

    #[test]
    fn test_screen_to_world_conversion() {
        let graph = create_test_graph();
        let renderer = Visual2DRenderer::new(graph);

        let screen_pos = Pos2::new(500.0, 350.0);
        let screen_center = Pos2::new(400.0, 300.0);
        let world_pos = renderer.screen_to_world(screen_pos, screen_center);

        assert_eq!(world_pos.x, 100.0);
        assert_eq!(world_pos.y, 50.0);
    }

    #[test]
    fn test_health_to_colors() {
        let graph = create_test_graph();
        let _renderer = Visual2DRenderer::new(graph);

        let (fill, _stroke) = nodes::health_to_colors(PrimalHealthStatus::Healthy);
        assert_eq!(fill, Color32::from_rgb(40, 180, 40));

        let (fill, _stroke) = nodes::health_to_colors(PrimalHealthStatus::Critical);
        assert_eq!(fill, Color32::from_rgb(200, 40, 40));
    }

    #[test]
    fn test_node_selection() {
        let graph = create_test_graph();
        let mut renderer = Visual2DRenderer::new(graph);

        assert!(renderer.selected_node().is_none());

        renderer.set_selected_node(Some("node1".to_string()));
        assert_eq!(renderer.selected_node(), Some("node1"));

        renderer.set_selected_node(None);
        assert!(renderer.selected_node().is_none());
    }

    #[test]
    fn test_camera_reset() {
        let graph = create_test_graph();
        let mut renderer = Visual2DRenderer::new(graph);

        renderer.camera_offset = Vec2::new(100.0, 50.0);
        renderer.zoom = 2.5;

        renderer.reset_camera();

        assert_eq!(renderer.camera_offset, Vec2::ZERO);
        assert!((renderer.zoom - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_animation_engine_integration() {
        let graph = create_test_graph();
        let mut renderer = Visual2DRenderer::new(graph);

        assert!(!renderer.is_animation_enabled());

        let animation = Arc::new(RwLock::new(AnimationEngine::new()));
        renderer.set_animation_engine(animation);

        renderer.set_animation_enabled(true);
        assert!(renderer.is_animation_enabled());

        renderer.set_animation_enabled(false);
        assert!(!renderer.is_animation_enabled());
    }

    #[test]
    fn test_zoom_levels() {
        let graph = create_test_graph();
        let mut renderer = Visual2DRenderer::new(graph);

        assert!((renderer.zoom - 1.0).abs() < f32::EPSILON);

        renderer.zoom = 0.5;
        assert!((renderer.zoom - 0.5).abs() < f32::EPSILON);

        renderer.zoom = 2.0;
        assert!((renderer.zoom - 2.0).abs() < f32::EPSILON);

        renderer.zoom = 3.0;
        assert!((renderer.zoom - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_camera_panning() {
        let graph = create_test_graph();
        let mut renderer = Visual2DRenderer::new(graph);

        assert_eq!(renderer.camera_offset, Vec2::ZERO);

        renderer.camera_offset = Vec2::new(100.0, 50.0);
        assert_eq!(renderer.camera_offset, Vec2::new(100.0, 50.0));

        renderer.camera_offset = Vec2::new(-50.0, -25.0);
        assert_eq!(renderer.camera_offset, Vec2::new(-50.0, -25.0));
    }

    #[test]
    fn test_world_to_screen_with_zoom() {
        let graph = create_test_graph();
        let mut renderer = Visual2DRenderer::new(graph);

        renderer.zoom = 2.0;
        let world_pos = Position::new_2d(100.0, 50.0);
        let screen_center = Pos2::new(400.0, 300.0);
        let screen_pos = renderer.world_to_screen(world_pos, screen_center);

        assert_eq!(screen_pos.x, 600.0);
        assert_eq!(screen_pos.y, 400.0);
    }

    #[test]
    fn test_world_to_screen_with_camera_offset() {
        let graph = create_test_graph();
        let mut renderer = Visual2DRenderer::new(graph);

        renderer.camera_offset = Vec2::new(50.0, 25.0);
        let world_pos = Position::new_2d(100.0, 50.0);
        let screen_center = Pos2::new(400.0, 300.0);
        let screen_pos = renderer.world_to_screen(world_pos, screen_center);

        assert_eq!(screen_pos.x, 550.0);
        assert_eq!(screen_pos.y, 375.0);
    }

    #[test]
    fn test_health_status_all_states() {
        let (fill_healthy, _stroke_healthy) = nodes::health_to_colors(PrimalHealthStatus::Healthy);
        assert_eq!(fill_healthy, Color32::from_rgb(40, 180, 40));

        let (fill_warning, _stroke_warning) = nodes::health_to_colors(PrimalHealthStatus::Warning);
        assert_eq!(fill_warning, Color32::from_rgb(200, 180, 40));

        let (fill_critical, _stroke_critical) =
            nodes::health_to_colors(PrimalHealthStatus::Critical);
        assert_eq!(fill_critical, Color32::from_rgb(200, 40, 40));

        let (fill_unknown, _stroke_unknown) = nodes::health_to_colors(PrimalHealthStatus::Unknown);
        assert_eq!(fill_unknown, Color32::from_rgb(120, 120, 120));
    }

    #[test]
    fn test_selected_node_persistence() {
        let graph = create_test_graph();
        let mut renderer = Visual2DRenderer::new(graph);

        renderer.set_selected_node(Some("node1".to_string()));
        assert_eq!(renderer.selected_node(), Some("node1"));

        renderer.set_selected_node(Some("node2".to_string()));
        assert_eq!(renderer.selected_node(), Some("node2"));

        renderer.set_selected_node(None);
        assert!(renderer.selected_node().is_none());
    }

    #[test]
    fn test_coordinate_conversion_roundtrip() {
        let graph = create_test_graph();
        let renderer = Visual2DRenderer::new(graph);

        let screen_center = Pos2::new(400.0, 300.0);
        let original_world = Position::new_2d(123.45, 67.89);

        let screen_pos = renderer.world_to_screen(original_world, screen_center);
        let converted_world = renderer.screen_to_world(screen_pos, screen_center);

        assert!((converted_world.x - original_world.x).abs() < 0.001);
        assert!((converted_world.y - original_world.y).abs() < 0.001);
    }

    #[test]
    fn test_renderer_with_empty_graph() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let renderer = Visual2DRenderer::new(graph.clone());

        assert!((renderer.zoom - 1.0).abs() < f32::EPSILON);
        assert_eq!(renderer.camera_offset, Vec2::ZERO);
        assert!(renderer.selected_node().is_none());

        let graph_read = graph.read().expect("lock poisoned");
        assert_eq!(graph_read.nodes().len(), 0);
        assert_eq!(graph_read.edges().len(), 0);
    }

    #[test]
    fn test_renderer_with_many_nodes() {
        let mut graph = GraphEngine::new();

        for i in 0..10 {
            let mut node =
                petal_tongue_core::test_fixtures::primals::test_primal(&format!("node{i}"));
            node.name = format!("Node {i}");
            graph.add_node(node);
        }

        for i in 0..9 {
            graph.add_edge(TopologyEdge {
                from: format!("node{i}").into(),
                to: format!("node{}", i + 1).into(),
                edge_type: "test".to_string(),
                label: None,
                capability: None,
                metrics: None,
            });
        }

        graph.set_layout(LayoutAlgorithm::ForceDirected);
        graph.layout(1);

        let graph_arc = Arc::new(RwLock::new(graph));
        let renderer = Visual2DRenderer::new(graph_arc.clone());

        let graph_read = graph_arc.read().expect("lock poisoned");
        assert_eq!(graph_read.nodes().len(), 10);
        assert_eq!(graph_read.edges().len(), 9);
        drop(graph_read);

        assert!((renderer.zoom - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_animation_engine_optional() {
        let graph = create_test_graph();
        let renderer = Visual2DRenderer::new(graph);

        assert!(!renderer.is_animation_enabled());
    }

    #[test]
    fn test_zoom_default_value() {
        let graph = create_test_graph();
        let renderer = Visual2DRenderer::new(graph);

        assert!((renderer.zoom - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_health_color_mapping() {
        let (healthy_fill, healthy_stroke) = nodes::health_to_colors(PrimalHealthStatus::Healthy);
        let (warning_fill, _warning_stroke) = nodes::health_to_colors(PrimalHealthStatus::Warning);
        let (critical_fill, _critical_stroke) =
            nodes::health_to_colors(PrimalHealthStatus::Critical);

        assert_ne!(healthy_fill, warning_fill);
        assert_ne!(healthy_fill, critical_fill);
        assert_ne!(warning_fill, critical_fill);

        assert_ne!(healthy_fill, Color32::TRANSPARENT);
        assert_ne!(healthy_stroke, Color32::TRANSPARENT);
    }

    #[test]
    fn test_renderer_initial_state() {
        let graph = create_test_graph();
        let renderer = Visual2DRenderer::new(graph);

        assert!((renderer.zoom - 1.0).abs() < f32::EPSILON);
        assert!(!renderer.is_animation_enabled());
    }

    #[test]
    fn test_animation_lifecycle() {
        let graph = create_test_graph();
        let mut renderer = Visual2DRenderer::new(graph);

        assert!(!renderer.is_animation_enabled());

        let engine = Arc::new(RwLock::new(AnimationEngine::default()));
        renderer.set_animation_engine(engine);
        assert!(!renderer.is_animation_enabled());

        renderer.set_animation_enabled(true);
        assert!(renderer.is_animation_enabled());

        renderer.set_animation_enabled(false);
        assert!(!renderer.is_animation_enabled());

        renderer.set_animation_enabled(true);
        assert!(renderer.is_animation_enabled());
    }

    #[test]
    fn test_multi_edge_rendering() {
        let mut graph = GraphEngine::new();

        for i in 1..=3 {
            let mut node =
                petal_tongue_core::test_fixtures::primals::test_primal(&format!("node{i}"));
            node.name = format!("Node {i}");
            graph.add_node(node);
        }

        graph.add_edge(TopologyEdge {
            from: "node1".into(),
            to: "node2".into(),
            edge_type: "connection".to_string(),
            label: Some("Edge 1-2".to_string()),
            capability: None,
            metrics: None,
        });
        graph.add_edge(TopologyEdge {
            from: "node2".into(),
            to: "node3".into(),
            edge_type: "connection".to_string(),
            label: Some("Edge 2-3".to_string()),
            capability: None,
            metrics: None,
        });
        graph.add_edge(TopologyEdge {
            from: "node1".into(),
            to: "node3".into(),
            edge_type: "connection".to_string(),
            label: Some("Edge 1-3".to_string()),
            capability: None,
            metrics: None,
        });

        let graph_arc = Arc::new(RwLock::new(graph));
        let _renderer = Visual2DRenderer::new(graph_arc.clone());

        let graph_read = graph_arc.read().expect("lock poisoned");
        assert_eq!(graph_read.edges().len(), 3);
    }

    #[test]
    fn test_renderer_with_different_health_states() {
        let mut graph = GraphEngine::new();

        graph.add_node(
            petal_tongue_core::test_fixtures::primals::test_primal_with_health(
                "healthy_node",
                PrimalHealthStatus::Healthy,
            ),
        );

        graph.add_node(
            petal_tongue_core::test_fixtures::primals::test_primal_with_health(
                "warning_node",
                PrimalHealthStatus::Warning,
            ),
        );

        graph.add_node(
            petal_tongue_core::test_fixtures::primals::test_primal_with_health(
                "critical_node",
                PrimalHealthStatus::Critical,
            ),
        );

        let graph_arc = Arc::new(RwLock::new(graph));
        let _renderer = Visual2DRenderer::new(graph_arc.clone());

        let graph_read = graph_arc.read().expect("lock poisoned");
        assert_eq!(graph_read.nodes().len(), 3);
    }

    #[test]
    fn test_set_zoom_clamps_min() {
        let graph = create_test_graph();
        let mut renderer = Visual2DRenderer::new(graph);
        renderer.set_zoom(0.05);
        assert!((renderer.zoom - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn test_set_zoom_clamps_max() {
        let graph = create_test_graph();
        let mut renderer = Visual2DRenderer::new(graph);
        renderer.set_zoom(15.0);
        assert!((renderer.zoom - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_set_zoom_within_range() {
        let graph = create_test_graph();
        let mut renderer = Visual2DRenderer::new(graph);
        renderer.set_zoom(2.5);
        assert!((renderer.zoom - 2.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_fit_to_view_empty_graph() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let mut renderer = Visual2DRenderer::new(graph.clone());
        renderer.zoom = 2.0;
        renderer.camera_offset = Vec2::new(100.0, 50.0);
        renderer.fit_to_view(&graph);
        assert!((renderer.zoom - 2.0).abs() < f32::EPSILON);
        assert_eq!(renderer.camera_offset, Vec2::new(100.0, 50.0));
    }

    #[test]
    fn test_fit_to_view_with_nodes() {
        let graph = create_test_graph();
        let mut renderer = Visual2DRenderer::new(graph.clone());
        renderer.fit_to_view(&graph);
        assert!(renderer.zoom >= 0.1 && renderer.zoom <= 5.0);
        assert!(renderer.camera_offset.x != 0.0 || renderer.camera_offset.y != 0.0);
    }

    #[test]
    fn test_navigate_to_node_existing() {
        let graph = create_test_graph();
        let mut renderer = Visual2DRenderer::new(graph.clone());
        renderer.zoom = 1.0;
        renderer.navigate_to_node("node1", &graph);
        let g = graph.read().expect("lock");
        let node = g.get_node("node1").expect("node1");
        let expected_offset = Vec2::new(-node.position.x, -node.position.y);
        drop(g);
        assert!((renderer.camera_offset.x - expected_offset.x).abs() < 1.0);
        assert!((renderer.camera_offset.y - expected_offset.y).abs() < 1.0);
    }

    #[test]
    fn test_navigate_to_node_missing() {
        let graph = create_test_graph();
        let mut renderer = Visual2DRenderer::new(graph.clone());
        renderer.camera_offset = Vec2::new(50.0, 25.0);
        renderer.navigate_to_node("nonexistent", &graph);
        assert_eq!(renderer.camera_offset, Vec2::new(50.0, 25.0));
    }

    #[test]
    fn test_select_node_by_id() {
        let graph = create_test_graph();
        let mut renderer = Visual2DRenderer::new(graph);
        renderer.select_node(Some("node1"));
        assert_eq!(renderer.selected_node(), Some("node1"));
        renderer.select_node(None);
        assert!(renderer.selected_node().is_none());
    }

    #[test]
    fn test_interactive_mode_toggle() {
        let graph = create_test_graph();
        let mut renderer = Visual2DRenderer::new(graph);
        assert!(!renderer.is_interactive());
        renderer.set_interactive_mode(true);
        assert!(renderer.is_interactive());
        renderer.set_interactive_mode(false);
        assert!(!renderer.is_interactive());
    }

    #[test]
    fn test_show_stats_toggle() {
        let graph = create_test_graph();
        let mut renderer = Visual2DRenderer::new(graph);
        assert!(renderer.show_stats());
        renderer.set_show_stats(false);
        assert!(!renderer.show_stats());
    }

    #[test]
    fn test_render_headless() {
        let graph = create_test_graph();
        let mut renderer = Visual2DRenderer::new(graph);
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                renderer.render(ui);
            });
        });
    }

    #[test]
    fn test_render_empty_graph_headless() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let mut renderer = Visual2DRenderer::new(graph);
        renderer.set_show_stats(false);
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                renderer.render(ui);
            });
        });
    }
}
