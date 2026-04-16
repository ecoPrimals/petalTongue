// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Headless integration tests: panel visibility, UI modes, and graph builder flows.

use petal_tongue_core::PanelKind;
use petal_tongue_ui::headless_harness::HeadlessHarness;
use petal_tongue_ui::tool_integration::ToolPanel;

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
fn graph_builder_editor_exercises_canvas() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    harness.key_press(petal_tongue_ui::egui::Key::G);
    harness.run_frames(5);
    assert!(harness.is_panel_visible(PanelKind::GraphBuilder));
    let _ = harness.tessellate();
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
fn graph_editor_many_nodes() {
    use petal_tongue_core::{PrimalHealthStatus, PrimalId};

    let mut harness = HeadlessHarness::new().unwrap();
    let graph = harness.app_mut().graph_handle();
    {
        let mut g = graph.write().unwrap();
        g.clear();
        for i in 0..15 {
            let id = format!("node-{i}");
            g.add_node(petal_tongue_core::PrimalInfo::new(
                PrimalId::from(id.clone()),
                format!("Node {i}"),
                "Test",
                format!("http://localhost:{}", 8000 + i),
                vec![],
                PrimalHealthStatus::Healthy,
                0,
            ));
        }
        for i in 0..14 {
            g.add_edge(petal_tongue_core::TopologyEdge {
                from: petal_tongue_core::PrimalId::from(format!("node-{i}")),
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
