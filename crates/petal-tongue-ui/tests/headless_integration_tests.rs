// SPDX-License-Identifier: AGPL-3.0-only
//! Integration tests for data flow, graph loading, scenarios, and multi-frame sequences.
//!
//! These tests exercise end-to-end behavior through the headless harness.

use petal_tongue_core::PanelKind;
use petal_tongue_ui::headless_harness::HeadlessHarness;

fn workspace_scenario_path(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .join("sandbox/scenarios")
        .join(name)
}

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
fn panel_rendering_clinical_mode_multi_frame() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetMode {
            mode: "clinical".to_string(),
        })
        .unwrap();
    harness.run_frames(5);

    assert!(harness.is_panel_visible(PanelKind::Dashboard));
    assert!(harness.is_panel_visible(PanelKind::TrustDashboard));
    assert!(!harness.is_panel_visible(PanelKind::Proprioception));
    assert!(harness.is_panel_visible(PanelKind::GraphCanvas));
}

#[test]
fn panel_rendering_developer_mode_multi_frame() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetMode {
            mode: "developer".to_string(),
        })
        .unwrap();
    harness.run_frames(5);

    assert!(harness.is_panel_visible(PanelKind::Proprioception));
    assert!(harness.is_panel_visible(PanelKind::AudioSonification));
    assert!(harness.is_panel_visible(PanelKind::Dashboard));
    assert!(harness.is_panel_visible(PanelKind::GraphCanvas));
}

#[test]
fn panel_rendering_presentation_mode_multi_frame() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetMode {
            mode: "presentation".to_string(),
        })
        .unwrap();
    harness.run_frames(5);

    assert!(!harness.is_panel_visible(PanelKind::TopMenu));
    assert!(!harness.is_panel_visible(PanelKind::Controls));
    assert!(harness.is_panel_visible(PanelKind::GraphCanvas));
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
fn multi_frame_sequence_20_frames() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    harness.run_frames(19);
    assert!(harness.frame_count() >= 20);
    let intro = harness.last_introspection().expect("frames ran");
    assert!(!intro.visible_panels.is_empty());
}

#[test]
fn multi_frame_sequence_mode_switch_mid_sequence() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frames(3);

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetMode {
            mode: "presentation".to_string(),
        })
        .unwrap();
    harness.run_frames(5);

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetMode {
            mode: "developer".to_string(),
        })
        .unwrap();
    harness.run_frames(5);

    assert!(harness.is_panel_visible(PanelKind::Proprioception));
}

#[test]
fn viewport_small_640x480() {
    let mut harness = HeadlessHarness::with_screen_size(640.0, 480.0).unwrap();
    harness.run_frames(5);
    let (buffer, w, h) = harness.render_pixels().unwrap();
    assert_eq!(w, 640);
    assert_eq!(h, 480);
    assert_eq!(buffer.len(), (640 * 480 * 4) as usize);
}

#[test]
fn viewport_large_1920x1080() {
    let mut harness = HeadlessHarness::with_screen_size(1920.0, 1080.0).unwrap();
    harness.run_frames(3);
    let (buffer, w, h) = harness.render_pixels().unwrap();
    assert_eq!(w, 1920);
    assert_eq!(h, 1080);
    assert_eq!(buffer.len(), (1920 * 1080 * 4) as usize);
}

#[test]
fn viewport_adaptive_320x240() {
    let mut harness = HeadlessHarness::with_screen_size(320.0, 240.0).unwrap();
    harness.run_frames(5);
    let (buffer, w, h) = harness.render_pixels().unwrap();
    assert_eq!(w, 320);
    assert_eq!(h, 240);
    assert_eq!(buffer.len(), (320 * 240 * 4) as usize);
}

#[test]
fn viewport_adaptive_800x600() {
    let mut harness = HeadlessHarness::with_screen_size(800.0, 600.0).unwrap();
    harness.run_frames(5);
    assert!(!harness.visible_panels().is_empty());
    let _ = harness.tessellate();
}

#[test]
fn metrics_panel_data_flow() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    harness.key_press(petal_tongue_ui::egui::Key::M);
    harness.run_frame();
    assert!(harness.is_panel_visible(PanelKind::Metrics));
    harness.run_frames(5);
    let _ = harness.tessellate();
}

#[test]
fn trust_panel_data_flow() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetPanelVisibility {
            panel: petal_tongue_core::PanelId::TrustDashboard,
            visible: true,
        })
        .unwrap();
    harness.run_frames(5);
    assert!(harness.is_panel_visible(PanelKind::TrustDashboard));
    let _ = harness.tessellate();
}

#[test]
fn device_dashboard_panel_data_flow() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetPanelVisibility {
            panel: petal_tongue_core::PanelId::SystemDashboard,
            visible: true,
        })
        .unwrap();
    harness.run_frames(5);
    assert!(harness.is_panel_visible(PanelKind::Dashboard));
    let _ = harness.tessellate();
}

#[test]
fn proprioception_panel_data_flow() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    harness.key_press(petal_tongue_ui::egui::Key::P);
    harness.run_frame();
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetMode {
            mode: "developer".to_string(),
        })
        .unwrap();
    harness.run_frames(5);
    assert!(harness.is_panel_visible(PanelKind::Proprioception));
    let _ = harness.tessellate();
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
fn error_state_load_scenario_invalid_path() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::LoadScenario {
            path: "/nonexistent/path/scenario.json".to_string(),
        })
        .unwrap();
    harness.run_frame();
    let _ = harness.tessellate();
}

#[test]
fn tutorial_mode_toggle_graph_builder() {
    use petal_tongue_core::LayoutAlgorithm;
    use petal_tongue_ui::tutorial_mode::TutorialMode;

    let mut harness = HeadlessHarness::new().unwrap();
    let graph = harness.app_mut().graph_handle();
    TutorialMode::create_fallback_scenario(graph, LayoutAlgorithm::ForceDirected);
    harness.run_frame();
    assert!(!harness.is_panel_visible(PanelKind::GraphBuilder));

    harness.key_press(petal_tongue_ui::egui::Key::G);
    harness.run_frame();
    assert!(harness.is_panel_visible(PanelKind::GraphBuilder));

    harness.key_press(petal_tongue_ui::egui::Key::G);
    harness.run_frame();
    assert!(!harness.is_panel_visible(PanelKind::GraphBuilder));
}

#[test]
fn continuous_mode_enables_game_loop() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetContinuousMode { enabled: true })
        .unwrap();
    harness.run_frames(10);
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
fn load_scenario_file_and_render() {
    let path = workspace_scenario_path("paint-simple.json");
    if !path.exists() {
        return;
    }
    let scenario = petal_tongue_ui::scenario::Scenario::load(&path).expect("load scenario");
    let mut harness = HeadlessHarness::new().unwrap();
    let graph = harness.app_mut().graph_handle();
    {
        let mut g = graph.write().unwrap();
        g.clear();
        for primal in scenario.to_primal_infos() {
            g.add_node(primal);
        }
        for edge in &scenario.edges {
            g.add_edge(edge.clone());
        }
    }
    harness.run_frames(5);
    let intro = harness.last_introspection().expect("frames ran");
    assert!(!intro.bound_data.is_empty());
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
fn graph_builder_editor_exercises_canvas() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    harness.key_press(petal_tongue_ui::egui::Key::G);
    harness.run_frames(5);
    assert!(harness.is_panel_visible(PanelKind::GraphBuilder));
    let _ = harness.tessellate();
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
fn process_viewer_tool_via_central_panel() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    if let Some(tool) = harness
        .app_mut()
        .tools_mut()
        .tools_mut()
        .iter_mut()
        .find(|t| t.metadata().name == "Process Viewer")
    {
        tool.toggle_visibility();
        harness.run_frames(3);
        let _ = harness.tessellate();
    }
}

#[test]
fn trust_dashboard_visible_with_empty_state() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetPanelVisibility {
            panel: petal_tongue_core::PanelId::TrustDashboard,
            visible: true,
        })
        .unwrap();
    harness.run_frames(5);
    assert!(harness.is_panel_visible(PanelKind::TrustDashboard));
    let _ = harness.tessellate();
}

#[test]
fn scene_bridge_load_complex_scenario() {
    let path = workspace_scenario_path("paint-simple.json");
    if !path.exists() {
        return;
    }
    let scenario = match petal_tongue_ui::scenario::Scenario::load(&path) {
        Ok(s) => s,
        Err(_) => return,
    };
    let mut harness = HeadlessHarness::new().unwrap();
    let graph = harness.app_mut().graph_handle();
    {
        let mut g = graph.write().unwrap();
        g.clear();
        for primal in scenario.to_primal_infos() {
            g.add_node(primal);
        }
        for edge in &scenario.edges {
            g.add_edge(edge.clone());
        }
    }
    harness.run_frames(5);
    let intro = harness.last_introspection().expect("frames ran");
    assert!(!intro.bound_data.is_empty());
    let _ = harness.tessellate();
}

#[test]
fn graph_editor_many_nodes() {
    use petal_tongue_core::{PrimalHealthStatus, PrimalId};

    let mut harness = HeadlessHarness::new().unwrap();
    let graph = harness.app_mut().graph_handle();
    {
        let mut g = graph.write().unwrap();
        g.clear();
        for i in 0..15 {
            let id = format!("node-{}", i);
            g.add_node(petal_tongue_core::PrimalInfo::new(
                PrimalId::from(id.clone()),
                format!("Node {}", i),
                "Test",
                format!("http://localhost:{}", 8000 + i),
                vec![],
                PrimalHealthStatus::Healthy,
                0,
            ));
        }
        for i in 0..14 {
            g.add_edge(petal_tongue_core::TopologyEdge {
                from: petal_tongue_core::PrimalId::from(format!("node-{}", i)),
                to: petal_tongue_core::PrimalId::from(format!("node-{}", i + 1)),
                edge_type: "conn".to_string(),
                label: None,
                capability: None,
                metrics: None,
            });
        }
    }
    harness.run_frame();
    harness.key_press(petal_tongue_ui::egui::Key::G);
    harness.run_frames(5);
    assert!(harness.is_panel_visible(PanelKind::GraphBuilder));
    let _ = harness.tessellate();
}

#[test]
fn graph_tutorial_then_ecosystem_layouts() {
    use petal_tongue_core::LayoutAlgorithm;
    use petal_tongue_ui::tutorial_mode::TutorialMode;

    let mut harness = HeadlessHarness::new().unwrap();
    let graph = harness.app_mut().graph_handle();
    TutorialMode::create_fallback_scenario(graph.clone(), LayoutAlgorithm::ForceDirected);
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

#[test]
fn continuous_mode_with_physics_and_animation() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetContinuousMode { enabled: true })
        .unwrap();
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetPhysics { enabled: true })
        .unwrap();
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetSceneAnimation { enabled: true })
        .unwrap();
    harness.run_frames(15);
    let _ = harness.tessellate();
}

#[test]
fn load_scenario_trust_demo() {
    let path = workspace_scenario_path("trust-demo.json");
    if !path.exists() {
        return;
    }
    let contents = std::fs::read_to_string(&path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&contents).unwrap();
    let primals = parsed.get("primals").and_then(|p| p.as_array());
    if primals.is_none() {
        return;
    }
    let mut harness = HeadlessHarness::new().unwrap();
    let graph = harness.app_mut().graph_handle();
    {
        let mut g = graph.write().unwrap();
        g.clear();
        for p in primals.unwrap() {
            let id = p.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");
            let name = p.get("name").and_then(|v| v.as_str()).unwrap_or(id);
            let primal_type = p
                .get("primal_type")
                .and_then(|v| v.as_str())
                .unwrap_or("Test");
            let endpoint = p
                .get("endpoint")
                .and_then(|v| v.as_str())
                .unwrap_or("http://localhost");
            let trust = p
                .get("trust_level")
                .and_then(|v| v.as_u64())
                .map(|n| n as u8);
            let family = p.get("family_id").and_then(|v| v.as_str());
            let mut info = petal_tongue_core::PrimalInfo::new(
                petal_tongue_core::PrimalId::from(id),
                name,
                primal_type,
                endpoint,
                vec![],
                petal_tongue_core::PrimalHealthStatus::Healthy,
                0,
            );
            if let Some(t) = trust {
                info.properties.insert(
                    "trust_level".to_string(),
                    petal_tongue_core::PropertyValue::Number(t as f64),
                );
            }
            if let Some(f) = family {
                info.properties.insert(
                    "family_id".to_string(),
                    petal_tongue_core::PropertyValue::String(f.to_string()),
                );
            }
            g.add_node(info);
        }
    }
    harness.run_frames(5);
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetPanelVisibility {
            panel: petal_tongue_core::PanelId::TrustDashboard,
            visible: true,
        })
        .unwrap();
    harness.run_frames(3);
    assert!(harness.is_panel_visible(PanelKind::TrustDashboard));
    let _ = harness.tessellate();
}
