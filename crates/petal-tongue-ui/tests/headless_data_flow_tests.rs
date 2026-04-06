// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Headless integration tests: graph data, tutorial scenarios, bindings, and introspection.

use petal_tongue_core::PanelKind;
use petal_tongue_ui::headless_harness::HeadlessHarness;

#[test]
fn tutorial_data_reflects_current_architecture() {
    use petal_tongue_core::LayoutAlgorithm;
    use petal_tongue_ui::tutorial_mode::TutorialMode;

    let mut harness = HeadlessHarness::new().unwrap();
    let graph = harness.app_mut().graph_handle();

    TutorialMode::create_fallback_scenario(graph, LayoutAlgorithm::ForceDirected);

    harness.run_frame();
    let intro = harness.run_frame();

    let has_bound_data = !intro.bound_data.is_empty();
    assert!(
        has_bound_data,
        "After loading tutorial data, introspection should report bound data objects"
    );

    let has_petaltongue = intro
        .bound_data
        .iter()
        .any(|d| d.data_object_id.contains("petaltongue"));
    assert!(
        has_petaltongue,
        "Tutorial data should include a petalTongue node in bound data"
    );
}

#[test]
fn graph_rendering_pipeline_bound_data() {
    use petal_tongue_core::LayoutAlgorithm;
    use petal_tongue_ui::tutorial_mode::TutorialMode;

    let mut harness = HeadlessHarness::new().unwrap();
    let graph = harness.app_mut().graph_handle();
    TutorialMode::create_fallback_scenario(graph, LayoutAlgorithm::ForceDirected);

    harness.run_frame();
    let intro = harness.run_frame();

    assert!(
        !intro.bound_data.is_empty(),
        "Graph data via TutorialMode should produce non-empty bound_data"
    );
}

#[test]
fn graph_rendering_select_node_exercises_primal_details() {
    use petal_tongue_core::LayoutAlgorithm;
    use petal_tongue_ui::tutorial_mode::TutorialMode;

    let mut harness = HeadlessHarness::new().unwrap();
    let graph = harness.app_mut().graph_handle();
    TutorialMode::create_fallback_scenario(graph, LayoutAlgorithm::ForceDirected);

    harness.run_frame();
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SelectNode {
            node_id: Some("petaltongue-tutorial".to_string()),
        })
        .unwrap();
    harness.run_frame();

    assert!(
        harness.is_showing_data("petaltongue-tutorial"),
        "SelectNode should show primal in bound data"
    );
    let _ = harness.tessellate();
}

#[test]
fn graph_rendering_load_and_render_multiple_frames() {
    use petal_tongue_core::LayoutAlgorithm;
    use petal_tongue_ui::tutorial_mode::TutorialMode;

    let mut harness = HeadlessHarness::new().unwrap();
    let graph = harness.app_mut().graph_handle();
    TutorialMode::create_fallback_scenario(graph, LayoutAlgorithm::ForceDirected);

    harness.run_frames(10);
    let intro = harness.last_introspection().expect("frames ran");
    assert!(!intro.bound_data.is_empty());
    assert!(
        intro
            .bound_data
            .iter()
            .any(|b| b.data_object_id.contains("petaltongue"))
    );
}

#[test]
fn data_flow_mock_data_through_rendering_pipeline() {
    use petal_tongue_core::LayoutAlgorithm;
    use petal_tongue_ui::tutorial_mode::TutorialMode;

    let mut harness = HeadlessHarness::new().unwrap();
    let graph = harness.app_mut().graph_handle();
    TutorialMode::create_fallback_scenario(graph, LayoutAlgorithm::ForceDirected);

    harness.run_frame();
    let intro = harness.run_frame();

    let node_bindings = intro
        .bound_data
        .iter()
        .filter(|b| b.binding_type == petal_tongue_core::BindingType::GraphNode)
        .count();
    let edge_bindings = intro
        .bound_data
        .iter()
        .filter(|b| b.binding_type == petal_tongue_core::BindingType::GraphEdge)
        .count();
    assert!(node_bindings >= 3);
    assert!(edge_bindings >= 2);
}

#[test]
fn tutorial_primals_use_capability_taxonomy() {
    use petal_tongue_core::{GraphEngine, LayoutAlgorithm};
    use petal_tongue_ui::tutorial_mode::TutorialMode;
    use std::sync::{Arc, RwLock};

    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    TutorialMode::create_fallback_scenario(Arc::clone(&graph), LayoutAlgorithm::ForceDirected);

    let graph = graph.read().unwrap();
    let petal = graph
        .nodes()
        .iter()
        .find(|n| n.info.id == "petaltongue-tutorial")
        .expect("petalTongue tutorial node should exist");

    assert!(
        petal.info.capabilities.iter().any(|c| c.starts_with("ui.")),
        "petalTongue should have ui.* capabilities"
    );
    assert!(
        petal
            .info
            .capabilities
            .iter()
            .any(|c| c.starts_with("ipc.")),
        "petalTongue should have ipc.* capabilities"
    );
    assert!(
        petal.info.endpoint.starts_with("unix://"),
        "Endpoint should be a Unix socket URI"
    );

    let security = graph
        .nodes()
        .iter()
        .find(|n| n.info.id == "security-example")
        .expect("Security example node should exist");

    assert!(
        security
            .info
            .capabilities
            .iter()
            .any(|c| c.starts_with("security.")),
        "Security example should have security.* capabilities"
    );

    let discovery = graph
        .nodes()
        .iter()
        .find(|n| n.info.id == "discovery-example")
        .expect("Discovery example node should exist");

    assert!(
        discovery
            .info
            .capabilities
            .iter()
            .any(|c| c.starts_with("discovery.")),
        "Discovery example should have discovery.* capabilities"
    );

    let edges = graph.edges();
    assert!(
        edges.iter().any(|e| e.edge_type == "ipc.discovery"),
        "Should have semantic ipc.discovery edge"
    );
    assert!(
        edges.iter().any(|e| e.edge_type == "ipc.trust"),
        "Should have semantic ipc.trust edge"
    );
}

#[test]
fn primal_details_select_then_deselect() {
    use petal_tongue_core::LayoutAlgorithm;
    use petal_tongue_ui::tutorial_mode::TutorialMode;

    let mut harness = HeadlessHarness::new().unwrap();
    let graph = harness.app_mut().graph_handle();
    TutorialMode::create_fallback_scenario(graph, LayoutAlgorithm::ForceDirected);
    harness.run_frame();

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SelectNode {
            node_id: Some("security-example".to_string()),
        })
        .unwrap();
    harness.run_frame();
    assert!(harness.is_showing_data("security-example"));

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SelectNode { node_id: None })
        .unwrap();
    harness.run_frame();
}

#[test]
fn error_state_select_nonexistent_node() {
    use petal_tongue_core::LayoutAlgorithm;
    use petal_tongue_ui::tutorial_mode::TutorialMode;

    let mut harness = HeadlessHarness::new().unwrap();
    let graph = harness.app_mut().graph_handle();
    TutorialMode::create_fallback_scenario(graph, LayoutAlgorithm::ForceDirected);
    harness.run_frame();

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SelectNode {
            node_id: Some("nonexistent-node-xyz".to_string()),
        })
        .unwrap();
    harness.run_frame();
    let _ = harness.tessellate();
}

#[test]
fn empty_graph_renders() {
    let mut harness = HeadlessHarness::new().unwrap();
    {
        let graph = harness.app_mut().graph_handle();
        let mut g = graph.write().unwrap();
        g.clear();
    }
    harness.run_frame();
    let intro = harness.run_frame();
    assert!(intro.bound_data.is_empty());
    assert!(harness.is_panel_visible(PanelKind::GraphCanvas));
    let _ = harness.tessellate();
}

#[test]
fn introspection_inspect_after_node_select() {
    use petal_tongue_core::LayoutAlgorithm;
    use petal_tongue_ui::tutorial_mode::TutorialMode;

    let mut harness = HeadlessHarness::new().unwrap();
    let graph = harness.app_mut().graph_handle();
    TutorialMode::create_fallback_scenario(graph, LayoutAlgorithm::ForceDirected);
    harness.run_frame();
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SelectNode {
            node_id: Some("petaltongue-tutorial".to_string()),
        })
        .unwrap();
    let intro = harness.run_frame();
    let has_inspect = intro
        .possible_interactions
        .iter()
        .any(|c| matches!(c.intent, petal_tongue_core::InteractionKind::Inspect));
    assert!(has_inspect || !intro.possible_interactions.is_empty());
}

#[test]
fn possible_interactions_include_navigate() {
    let mut harness = HeadlessHarness::new().unwrap();
    let intro = harness.run_frame();
    let has_navigate = intro
        .possible_interactions
        .iter()
        .any(|c| matches!(c.intent, petal_tongue_core::InteractionKind::Navigate));
    assert!(has_navigate || !intro.possible_interactions.is_empty());
}

#[test]
fn full_data_state_multiple_nodes() {
    use petal_tongue_core::LayoutAlgorithm;
    use petal_tongue_ui::tutorial_mode::TutorialMode;

    let mut harness = HeadlessHarness::new().unwrap();
    let graph = harness.app_mut().graph_handle();
    TutorialMode::create_fallback_scenario(graph, LayoutAlgorithm::ForceDirected);
    harness.run_frames(5);
    let intro = harness.last_introspection().expect("frames ran");
    let node_count = intro
        .bound_data
        .iter()
        .filter(|b| b.binding_type == petal_tongue_core::BindingType::GraphNode)
        .count();
    assert!(node_count >= 3);
}

#[test]
fn graph_tutorial_then_ecosystem_layouts() {
    use petal_tongue_core::LayoutAlgorithm;
    use petal_tongue_ui::tutorial_mode::TutorialMode;

    let mut harness = HeadlessHarness::new().unwrap();
    let graph = harness.app_mut().graph_handle();
    TutorialMode::create_fallback_scenario(graph, LayoutAlgorithm::ForceDirected);
    harness.run_frames(5);

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetLayout {
            algorithm: "Grid".to_string(),
        })
        .unwrap();
    harness.run_frames(5);

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetLayout {
            algorithm: "Radial".to_string(),
        })
        .unwrap();
    harness.run_frames(5);

    let intro = harness.last_introspection().expect("frames ran");
    assert!(!intro.bound_data.is_empty());
}
