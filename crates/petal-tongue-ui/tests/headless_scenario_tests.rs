// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Headless integration tests: workspace scenario files and load-path error handling.

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
fn scene_bridge_load_complex_scenario() {
    let path = workspace_scenario_path("paint-simple.json");
    if !path.exists() {
        return;
    }
    let Ok(scenario) = petal_tongue_ui::scenario::Scenario::load(&path) else {
        return;
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
                .and_then(serde_json::Value::as_u64)
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
                    petal_tongue_core::PropertyValue::Number(f64::from(t)),
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
