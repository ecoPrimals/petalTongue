// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Unit tests for scenario loading and parsing
//!
//! Tests the modular UI control system introduced in v2.2.0

#![allow(clippy::unnecessary_get_then_check, clippy::bool_assert_comparison)]

use petal_tongue_ui::scenario::{FeatureFlags, PanelVisibility, Scenario};
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR points to the crate dir; go up two levels for workspace root
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("failed to resolve workspace root")
        .to_path_buf()
}

fn scenario_path(name: &str) -> PathBuf {
    workspace_root().join("sandbox/scenarios").join(name)
}

#[test]
fn test_paint_simple_scenario_loads() {
    let scenario = Scenario::load(scenario_path("paint-simple.json"))
        .expect("Failed to load paint-simple.json");

    assert_eq!(scenario.name, "Paint Test - Ultra Simple");
    assert_eq!(scenario.version, "2.0.0");
    assert_eq!(scenario.mode, "paint-canvas");
    assert_eq!(scenario.ecosystem.primals.len(), 3);
}

#[test]
fn test_paint_simple_panel_visibility() {
    let scenario = Scenario::load(scenario_path("paint-simple.json"))
        .expect("Failed to load paint-simple.json");

    let panels = &scenario.ui_config.show_panels;

    // Paint mode should hide all panels except top menu
    assert_eq!(panels.left_sidebar, false, "Left sidebar should be hidden");
    assert_eq!(
        panels.right_sidebar, false,
        "Right sidebar should be hidden"
    );
    assert_eq!(panels.top_menu, true, "Top menu should be visible");
    assert_eq!(
        panels.system_dashboard, false,
        "System dashboard should be hidden"
    );
    assert_eq!(panels.audio_panel, false, "Audio panel should be hidden");
    assert_eq!(
        panels.trust_dashboard, false,
        "Trust dashboard should be hidden"
    );
    assert_eq!(
        panels.graph_stats, false,
        "Graph statistics should be hidden"
    );
}

#[test]
fn test_paint_simple_features() {
    let scenario = Scenario::load(scenario_path("paint-simple.json"))
        .expect("Failed to load paint-simple.json");

    let features = &scenario.ui_config.features;

    // Paint mode should disable all extra features
    assert_eq!(
        features.audio_sonification, false,
        "Audio should be disabled"
    );
    assert_eq!(
        features.auto_refresh, false,
        "Auto-refresh should be disabled"
    );
    assert_eq!(features.neural_api, false, "Neural API should be disabled");
}

#[test]
fn test_full_dashboard_scenario_loads() {
    let scenario = Scenario::load(scenario_path("full-dashboard.json"))
        .expect("Failed to load full-dashboard.json");

    assert_eq!(scenario.name, "Full Dashboard - All Subsystems");
    assert_eq!(scenario.mode, "full-dashboard");
    assert_eq!(scenario.ecosystem.primals.len(), 5);
}

#[test]
fn test_full_dashboard_panel_visibility() {
    let scenario = Scenario::load(scenario_path("full-dashboard.json"))
        .expect("Failed to load full-dashboard.json");

    let panels = &scenario.ui_config.show_panels;

    // Full dashboard should show all panels
    assert_eq!(panels.left_sidebar, true, "Left sidebar should be visible");
    assert_eq!(
        panels.right_sidebar, true,
        "Right sidebar should be visible"
    );
    assert_eq!(
        panels.system_dashboard, true,
        "System dashboard should be visible"
    );
    assert_eq!(panels.audio_panel, true, "Audio panel should be visible");
    assert_eq!(
        panels.trust_dashboard, true,
        "Trust dashboard should be visible"
    );
    assert_eq!(
        panels.graph_stats, true,
        "Graph statistics should be visible"
    );
}

#[test]
fn test_full_dashboard_features() {
    let scenario = Scenario::load(scenario_path("full-dashboard.json"))
        .expect("Failed to load full-dashboard.json");

    let features = &scenario.ui_config.features;

    // Full dashboard should enable features
    assert_eq!(features.audio_sonification, true, "Audio should be enabled");
    assert_eq!(
        features.auto_refresh, true,
        "Auto-refresh should be enabled"
    );
}

#[test]
fn test_default_panel_visibility() {
    // Test that default is backward compatible (all visible)
    let default_panels = PanelVisibility::default();

    assert_eq!(default_panels.left_sidebar, true);
    assert_eq!(default_panels.right_sidebar, true);
    assert_eq!(default_panels.system_dashboard, true);
    assert_eq!(default_panels.audio_panel, true);
    assert_eq!(default_panels.trust_dashboard, true);
    assert_eq!(default_panels.graph_stats, true);
}

#[test]
fn test_default_feature_flags() {
    // Test that defaults are backward compatible
    let default_features = FeatureFlags::default();

    assert_eq!(default_features.audio_sonification, true);
    assert_eq!(default_features.auto_refresh, true);
    assert_eq!(default_features.neural_api, false); // Neural API requires external service
    assert_eq!(default_features.tutorial_mode, false);
}

#[test]
fn test_scenario_to_primal_infos() {
    let scenario = Scenario::load(scenario_path("paint-simple.json"))
        .expect("Failed to load paint-simple.json");

    let primals = scenario.to_primal_infos();

    assert_eq!(primals.len(), 3, "Should have 3 primals");

    // Verify first primal
    let red_circle = primals
        .iter()
        .find(|p| p.id == "node-1")
        .expect("Red Circle not found");
    assert_eq!(red_circle.name, "Red Circle");
    assert_eq!(red_circle.primal_type, "test");

    // Verify positions are preserved
    assert!(primals[0].properties.contains_key("cpu_percent"));
    assert!(primals[0].properties.contains_key("memory_mb"));
}

#[test]
fn test_sensory_config_validation() {
    let scenario = Scenario::load(scenario_path("paint-simple.json"))
        .expect("Failed to load paint-simple.json");

    // Paint mode requires only visual output
    assert_eq!(
        scenario.sensory_config.required_capabilities.outputs,
        vec!["visual"]
    );
    assert_eq!(
        scenario.sensory_config.required_capabilities.inputs.len(),
        0
    );

    // Complexity hint should be simple
    assert_eq!(scenario.sensory_config.complexity_hint, "simple");
}

#[test]
fn test_primal_positions_from_scenario() {
    let scenario = Scenario::load(scenario_path("paint-simple.json"))
        .expect("Failed to load paint-simple.json");

    // Verify explicit positions are in the scenario
    let primals = &scenario.ecosystem.primals;

    assert!(
        (primals[0].position.x - 200.0).abs() < f32::EPSILON,
        "Red Circle x position"
    );
    assert!(
        (primals[0].position.y - 200.0).abs() < f32::EPSILON,
        "Red Circle y position"
    );

    assert!(
        (primals[1].position.x - 400.0).abs() < f32::EPSILON,
        "Green Square x position"
    );
    assert!(
        (primals[1].position.y - 200.0).abs() < f32::EPSILON,
        "Green Square y position"
    );

    assert!(
        (primals[2].position.x - 300.0).abs() < f32::EPSILON,
        "Blue Triangle x position"
    );
    assert!(
        (primals[2].position.y - 350.0).abs() < f32::EPSILON,
        "Blue Triangle y position"
    );
}
