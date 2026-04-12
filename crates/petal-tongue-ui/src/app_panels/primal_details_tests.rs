// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use petal_tongue_core::{PrimalId, Properties, PropertyValue};

#[must_use]
const fn health_status_display(health: PrimalHealthStatus) -> (&'static str, egui::Color32) {
    let rgb = health_status_rgb(health);
    (
        health_status_icon(health),
        egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]),
    )
}

fn test_primal_info() -> PrimalInfo {
    let endpoint = petal_tongue_core::constants::default_headless_url();
    let mut info = PrimalInfo::new(
        PrimalId::from("test-1"),
        "Test Node",
        "compute",
        endpoint,
        vec!["cap1".to_string(), "cap2".to_string()],
        PrimalHealthStatus::Healthy,
        1_000_000,
    );
    info.set_trust_level(2);
    info.set_family_id("family-x");
    info
}

#[test]
fn build_properties_from_info_empty_props() {
    let info = test_primal_info();
    let props = build_properties_from_info(&info);
    assert_eq!(props.get("trust_level"), Some(&PropertyValue::Number(2.0)));
    assert_eq!(
        props.get("family_id"),
        Some(&PropertyValue::String("family-x".to_string()))
    );
    if let Some(PropertyValue::Array(caps)) = props.get("capabilities") {
        assert_eq!(caps.len(), 2);
    } else {
        panic!("expected capabilities array");
    }
}

#[test]
fn health_status_display_all() {
    let (icon, _) = health_status_display(PrimalHealthStatus::Healthy);
    assert_eq!(icon, "✅");
    let (icon, _) = health_status_display(PrimalHealthStatus::Warning);
    assert_eq!(icon, "⚠️");
    let (icon, _) = health_status_display(PrimalHealthStatus::Critical);
    assert_eq!(icon, "❌");
    let (icon, _) = health_status_display(PrimalHealthStatus::Unknown);
    assert_eq!(icon, "❓");
}

#[test]
fn health_status_icon_all() {
    assert_eq!(health_status_icon(PrimalHealthStatus::Healthy), "✅");
    assert_eq!(health_status_icon(PrimalHealthStatus::Warning), "⚠️");
    assert_eq!(health_status_icon(PrimalHealthStatus::Critical), "❌");
    assert_eq!(health_status_icon(PrimalHealthStatus::Unknown), "❓");
}

#[test]
fn health_status_rgb_all() {
    assert_eq!(health_status_rgb(PrimalHealthStatus::Healthy), [0, 200, 0]);
    assert_eq!(
        health_status_rgb(PrimalHealthStatus::Warning),
        [255, 200, 0]
    );
    assert_eq!(
        health_status_rgb(PrimalHealthStatus::Critical),
        [255, 50, 50]
    );
    assert_eq!(
        health_status_rgb(PrimalHealthStatus::Unknown),
        [128, 128, 128]
    );
}

#[test]
fn primal_details_summary_from_primal_info() {
    let info = test_primal_info();
    let now = 2_000_000u64;
    let summary = PrimalDetailsSummary::from_primal_info(&info, now);

    assert_eq!(summary.name, "Test Node");
    assert_eq!(summary.id, "test-1");
    assert_eq!(summary.primal_type, "compute");
    assert_eq!(summary.health_icon, "✅");
    assert_eq!(summary.health_color, [0, 200, 0]);
    assert_eq!(summary.health_status_text, "Healthy");
    assert_eq!(summary.capabilities.len(), 2);
    assert!(summary.properties.contains_key("trust_level"));
    assert!(summary.properties.contains_key("family_id"));
    assert!(summary.node_detail.is_none());
}

#[test]
fn primal_details_summary_last_seen() {
    let info = test_primal_info();
    let now = 1_000_042u64;
    let summary = PrimalDetailsSummary::from_primal_info(&info, now);
    assert_eq!(summary.last_seen_text, "42 seconds ago");
}

#[test]
fn format_last_seen_seconds_display() {
    assert_eq!(format_last_seen_seconds(0), "0 seconds ago");
    assert_eq!(format_last_seen_seconds(42), "42 seconds ago");
}

#[test]
fn extract_health_u8_default() {
    let props = Properties::new();
    assert_eq!(extract_health_u8_from_properties(&props), 100);
}

#[test]
fn extract_health_u8_from_property() {
    let mut props = Properties::new();
    props.insert("health".to_string(), PropertyValue::Number(75.0));
    assert_eq!(extract_health_u8_from_properties(&props), 75);
}

#[test]
fn extract_health_u8_clamps_at_boundaries() {
    let mut props = Properties::new();
    props.insert("health".to_string(), PropertyValue::Number(0.0));
    assert_eq!(extract_health_u8_from_properties(&props), 0);

    props.insert("health".to_string(), PropertyValue::Number(255.0));
    assert_eq!(extract_health_u8_from_properties(&props), 255);
}

#[test]
fn extract_health_u8_ignores_non_number() {
    let mut props = Properties::new();
    props.insert(
        "health".to_string(),
        PropertyValue::String("bad".to_string()),
    );
    assert_eq!(extract_health_u8_from_properties(&props), 100);
}

#[test]
fn build_properties_from_info_without_trust_family() {
    let endpoint = petal_tongue_core::constants::default_headless_url();
    let info = PrimalInfo::new(
        PrimalId::from("minimal-1"),
        "Minimal Node",
        "compute",
        endpoint,
        vec![],
        PrimalHealthStatus::Unknown,
        0,
    );
    let props = build_properties_from_info(&info);
    assert!(!props.contains_key("trust_level"));
    assert!(!props.contains_key("family_id"));
    if let Some(PropertyValue::Array(caps)) = props.get("capabilities") {
        assert!(caps.is_empty());
    } else {
        panic!("expected capabilities array");
    }
}

#[test]
fn primal_details_summary_uses_existing_properties() {
    let mut info = test_primal_info();
    let mut props = Properties::new();
    props.insert(
        "custom".to_string(),
        PropertyValue::String("value".to_string()),
    );
    info.properties = props;
    let summary = PrimalDetailsSummary::from_primal_info(&info, 2_000_000);
    assert_eq!(
        summary.properties.get("custom"),
        Some(&PropertyValue::String("value".to_string()))
    );
}

#[test]
fn extract_node_detail_with_data_bindings_json() {
    let mut props = Properties::new();
    props.insert(
        "data_bindings_json".to_string(),
        PropertyValue::String(
            r#"[{"channel_type":"gauge","id":"ch1","label":"ch1","value":0,"min":0,"max":100,"unit":"","normal_range":[0,100],"warning_range":[10,90]}]"#
                .to_string(),
        ),
    );
    props.insert("health".to_string(), PropertyValue::Number(80.0));
    let info = PrimalInfo::new(
        PrimalId::from("bind-test"),
        "Bind Test",
        "sensor",
        petal_tongue_core::constants::default_headless_url(),
        vec!["sensor".to_string()],
        PrimalHealthStatus::Healthy,
        0,
    );
    let detail = extract_node_detail_for_bindings(&info, &props);
    assert!(detail.is_some());
    let d = detail.unwrap();
    assert_eq!(d.name, "Bind Test");
    assert_eq!(d.health, 80);
    assert_eq!(d.data_bindings.len(), 1);
}

#[test]
fn extract_node_detail_empty_bindings_returns_none() {
    let mut props = Properties::new();
    props.insert(
        "data_bindings_json".to_string(),
        PropertyValue::String("[]".to_string()),
    );
    let info = PrimalInfo::new(
        PrimalId::from("empty"),
        "Empty",
        "compute",
        petal_tongue_core::constants::default_headless_url(),
        vec![],
        PrimalHealthStatus::Healthy,
        0,
    );
    let detail = extract_node_detail_for_bindings(&info, &props);
    assert!(detail.is_none());
}

#[test]
fn build_properties_from_info_empty_capabilities() {
    let endpoint = petal_tongue_core::constants::default_headless_url();
    let info = PrimalInfo::new(
        PrimalId::from("empty-caps"),
        "Empty Caps",
        "sensor",
        endpoint,
        vec![],
        PrimalHealthStatus::Healthy,
        1_000,
    );
    let props = build_properties_from_info(&info);
    if let Some(PropertyValue::Array(caps)) = props.get("capabilities") {
        assert_eq!(caps.len(), 0);
    } else {
        panic!("expected capabilities array");
    }
}

#[test]
fn render_primal_details_panel_headless() {
    use crate::accessibility::{ColorPalette, ColorScheme};
    use petal_tongue_adapters::AdapterRegistry;
    use petal_tongue_graph::Visual2DRenderer;

    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    {
        let mut g = graph.write().unwrap();
        g.add_node(test_primal_info());
    }
    let mut visual_renderer = Visual2DRenderer::new(graph.clone());
    visual_renderer.set_selected_node(Some("test-1".to_string()));
    let palette = ColorPalette::from_scheme(ColorScheme::Default);
    let adapter_registry = AdapterRegistry::default();

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::SidePanel::right("details").show(ctx, |ui| {
            render_primal_details_panel(
                ui,
                "test-1",
                &palette,
                &graph,
                &adapter_registry,
                &mut visual_renderer,
            );
        });
    });
}

#[test]
fn render_primal_details_panel_node_not_found() {
    use crate::accessibility::{ColorPalette, ColorScheme};
    use petal_tongue_adapters::AdapterRegistry;
    use petal_tongue_graph::Visual2DRenderer;

    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    let mut visual_renderer = Visual2DRenderer::new(graph.clone());
    let palette = ColorPalette::from_scheme(ColorScheme::Default);
    let adapter_registry = AdapterRegistry::default();

    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::SidePanel::right("details").show(ctx, |ui| {
            render_primal_details_panel(
                ui,
                "nonexistent",
                &palette,
                &graph,
                &adapter_registry,
                &mut visual_renderer,
            );
        });
    });
}

#[test]
fn extract_node_detail_data_channels_json() {
    let mut props = Properties::new();
    props.insert(
        "data_channels_json".to_string(),
        PropertyValue::String(
            r#"[{"channel_type":"gauge","id":"ch1","label":"ch1","value":0,"min":0,"max":100,"unit":"","normal_range":[0,100],"warning_range":[10,90]}]"#
                .to_string(),
        ),
    );
    props.insert("health".to_string(), PropertyValue::Number(90.0));
    let info = PrimalInfo::new(
        PrimalId::from("ch-test"),
        "Ch Test",
        "sensor",
        petal_tongue_core::constants::default_headless_url(),
        vec![],
        PrimalHealthStatus::Healthy,
        0,
    );
    let detail = extract_node_detail_for_bindings(&info, &props);
    assert!(detail.is_some());
}

#[test]
fn extract_node_detail_data_bindings_fallback() {
    let mut props = Properties::new();
    props.insert(
        "data_bindings".to_string(),
        PropertyValue::Array(vec![PropertyValue::String("x".to_string())]),
    );
    let info = PrimalInfo::new(
        PrimalId::from("fallback"),
        "Fallback",
        "compute",
        petal_tongue_core::constants::default_headless_url(),
        vec![],
        PrimalHealthStatus::Healthy,
        0,
    );
    let detail = extract_node_detail_for_bindings(&info, &props);
    assert!(detail.is_none());
}

#[test]
fn primal_details_summary_capabilities_empty_display() {
    let mut info = test_primal_info();
    info.capabilities = vec![];
    let summary = PrimalDetailsSummary::from_primal_info(&info, 2_000_000);
    assert!(summary.capabilities.is_empty());
}

#[test]
fn primal_details_summary_with_node_detail() {
    let mut props = Properties::new();
    props.insert(
        "data_bindings_json".to_string(),
        PropertyValue::String(
            r#"[{"channel_type":"gauge","id":"ch1","label":"ch1","value":50,"min":0,"max":100,"unit":"","normal_range":[0,100],"warning_range":[10,90]}]"#
                .to_string(),
        ),
    );
    props.insert("health".to_string(), PropertyValue::Number(75.0));
    let mut info = PrimalInfo::new(
        PrimalId::from("detail-test"),
        "Detail Test",
        "sensor",
        petal_tongue_core::constants::default_headless_url(),
        vec!["sensor".to_string()],
        PrimalHealthStatus::Healthy,
        0,
    );
    info.properties = props;
    let summary = PrimalDetailsSummary::from_primal_info(&info, 2_000_000);
    assert!(summary.node_detail.is_some());
    let d = summary.node_detail.unwrap();
    assert_eq!(d.health, 75);
}
