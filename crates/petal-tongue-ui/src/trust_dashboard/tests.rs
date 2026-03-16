// SPDX-License-Identifier: AGPL-3.0-only
//! Trust dashboard unit tests.

use super::TrustDashboard;
use super::compute::{
    average_trust_display, prepare_trust_display, trust_level_number_to_label, trust_level_style,
    trust_level_to_display_row,
};
use super::types::TrustSummary;
use petal_tongue_core::{PrimalHealthStatus, PrimalInfo, Properties, PropertyValue};
use std::collections::HashMap;

fn create_test_primal(id: &str, trust: Option<u8>, family: Option<&str>) -> PrimalInfo {
    let mut props = Properties::new();
    if let Some(t) = trust {
        props.insert("trust_level".to_string(), PropertyValue::Number(t as f64));
    }
    if let Some(f) = family {
        props.insert(
            "family_id".to_string(),
            PropertyValue::String(f.to_string()),
        );
    }

    PrimalInfo {
        id: id.to_string().into(),
        name: format!("Test Primal {}", id),
        primal_type: "Test".to_string(),
        endpoint: "http://test".to_string(),
        capabilities: vec![],
        health: PrimalHealthStatus::Healthy,
        last_seen: 0,
        endpoints: None,
        metadata: None,
        properties: props,
        #[expect(deprecated)]
        trust_level: trust,
        #[expect(deprecated)]
        family_id: family.map(String::from),
    }
}

// === Pure function tests ===

#[test]
fn trust_level_to_display_row_full() {
    let row = trust_level_to_display_row("Full (3)", 5, 10);
    assert_eq!(row.emoji, "🟢");
    assert_eq!(row.color, [76, 175, 80, 255]);
    assert!((row.percentage - 50.0).abs() < f32::EPSILON);
    assert_eq!(row.count, 5);
}

#[test]
fn trust_level_to_display_row_elevated() {
    let row = trust_level_to_display_row("Elevated (2)", 3, 12);
    assert_eq!(row.emoji, "🟠");
    assert_eq!(row.color, [255, 152, 0, 255]);
    assert!((row.percentage - 25.0).abs() < f32::EPSILON);
}

#[test]
fn trust_level_to_display_row_limited() {
    let row = trust_level_to_display_row("Limited (1)", 2, 8);
    assert_eq!(row.emoji, "🟡");
    assert_eq!(row.color, [255, 235, 59, 255]);
    assert!((row.percentage - 25.0).abs() < f32::EPSILON);
}

#[test]
fn trust_level_to_display_row_none() {
    let row = trust_level_to_display_row("None (0)", 1, 4);
    assert_eq!(row.emoji, "⚫");
    assert_eq!(row.color, [158, 158, 158, 255]);
}

#[test]
fn trust_level_to_display_row_unknown_label() {
    let row = trust_level_to_display_row("Unknown (99)", 2, 10);
    assert_eq!(row.emoji, "⚫");
    assert_eq!(row.color, [158, 158, 158, 255]);
    assert_eq!(row.label, "Unknown (99)");
}

#[test]
fn trust_level_to_display_row_zero_total() {
    let row = trust_level_to_display_row("Full (3)", 0, 0);
    assert!((row.percentage).abs() < f32::EPSILON);
}

#[test]
fn trust_level_to_display_row_large_numbers() {
    let row = trust_level_to_display_row("Full (3)", 1_000_000, 2_000_000);
    assert_eq!(row.count, 1_000_000);
    assert!((row.percentage - 50.0).abs() < f32::EPSILON);
}

#[test]
fn average_trust_display_full() {
    let d = average_trust_display(3.0);
    assert_eq!(d.emoji, "🟢");
    assert_eq!(d.label, "Full");
    assert_eq!(d.sound_name, "success");
    assert_eq!(d.color, [76, 175, 80, 255]);
}

#[test]
fn average_trust_display_elevated() {
    let d = average_trust_display(2.0);
    assert_eq!(d.emoji, "🟠");
    assert_eq!(d.label, "Elevated");
    assert_eq!(d.sound_name, "notification");
}

#[test]
fn average_trust_display_limited() {
    let d = average_trust_display(1.0);
    assert_eq!(d.emoji, "🟡");
    assert_eq!(d.label, "Limited");
    assert_eq!(d.sound_name, "warning");
}

#[test]
fn average_trust_display_none() {
    let d = average_trust_display(0.0);
    assert_eq!(d.emoji, "⚫");
    assert_eq!(d.label, "None");
    assert_eq!(d.sound_name, "error");
}

#[test]
fn average_trust_display_unknown() {
    let d = average_trust_display(99.0);
    assert_eq!(d.emoji, "❓");
    assert_eq!(d.label, "Unknown");
    assert_eq!(d.sound_name, "notification");
}

#[test]
fn average_trust_display_rounds_2_7_to_elevated() {
    let d = average_trust_display(2.7);
    assert_eq!(d.label, "Full");
    assert_eq!(d.sound_name, "success");
}

#[test]
fn average_trust_display_negative() {
    let d = average_trust_display(-1.0);
    assert_eq!(d.emoji, "❓");
    assert_eq!(d.label, "Unknown");
}

#[test]
fn average_trust_display_above_three() {
    let d = average_trust_display(4.0);
    assert_eq!(d.emoji, "❓");
    assert_eq!(d.label, "Unknown");
}

#[test]
fn average_trust_display_rounds_1_5_to_elevated() {
    let d = average_trust_display(1.5);
    assert_eq!(d.label, "Elevated");
}

#[test]
fn prepare_trust_display_empty() {
    let summary = TrustSummary::default();
    let ds = prepare_trust_display(&summary, 5);
    assert_eq!(ds.total_primals, 0);
    assert!(ds.rows.is_empty());
    assert!(ds.average.is_none());
    assert_eq!(ds.last_update_label, "Updated 5 seconds ago");
}

#[test]
fn prepare_trust_display_with_trust_properties() {
    let mut dist = HashMap::new();
    dist.insert("Full (3)".to_string(), 2);
    dist.insert("Limited (1)".to_string(), 1);
    let summary = TrustSummary {
        trust_distribution: dist,
        total_primals: 3,
        family_count: 2,
        unique_families: 1,
        average_trust: Some(2.33),
    };
    let ds = prepare_trust_display(&summary, 10);
    assert_eq!(ds.total_primals, 3);
    assert_eq!(ds.rows.len(), 2);
    assert!(ds.average.is_some());
    assert_eq!(ds.average.as_ref().unwrap().label, "Elevated");
}

#[test]
fn prepare_trust_display_without_trust() {
    let summary = TrustSummary {
        trust_distribution: HashMap::new(),
        total_primals: 5,
        family_count: 3,
        unique_families: 2,
        average_trust: None,
    };
    let ds = prepare_trust_display(&summary, 0);
    assert_eq!(ds.total_primals, 5);
    assert!(ds.rows.is_empty());
    assert!(ds.average.is_none());
}

#[test]
fn prepare_trust_display_mixed() {
    let mut dist = HashMap::new();
    dist.insert("Full (3)".to_string(), 1);
    dist.insert("Elevated (2)".to_string(), 1);
    dist.insert("Limited (1)".to_string(), 1);
    dist.insert("None (0)".to_string(), 1);
    dist.insert("Unknown (5)".to_string(), 1);
    let summary = TrustSummary {
        trust_distribution: dist,
        total_primals: 5,
        family_count: 4,
        unique_families: 3,
        average_trust: Some(1.6),
    };
    let ds = prepare_trust_display(&summary, 42);
    assert_eq!(ds.total_primals, 5);
    assert_eq!(ds.rows.len(), 5);
    assert_eq!(ds.family_count, 4);
    assert_eq!(ds.unique_families, 3);
    assert_eq!(ds.last_update_label, "Updated 42 seconds ago");
}

#[test]
fn prepare_trust_display_sorts_by_count_desc() {
    let mut dist = HashMap::new();
    dist.insert("Full (3)".to_string(), 1);
    dist.insert("Limited (1)".to_string(), 5);
    dist.insert("None (0)".to_string(), 3);
    let summary = TrustSummary {
        trust_distribution: dist,
        total_primals: 9,
        family_count: 2,
        unique_families: 1,
        average_trust: Some(1.5),
    };
    let ds = prepare_trust_display(&summary, 0);
    assert_eq!(ds.rows.len(), 3);
    assert_eq!(ds.rows[0].count, 5);
    assert_eq!(ds.rows[1].count, 3);
    assert_eq!(ds.rows[2].count, 1);
    assert!(ds.average.is_some());
}

#[test]
fn trust_level_style_full_variants() {
    let (emoji, color) = trust_level_style("Full (3)");
    assert_eq!(emoji, "🟢");
    assert_eq!(color, [76, 175, 80, 255]);
    let (emoji2, color2) = trust_level_style("Something (3) else");
    assert_eq!(emoji2, "🟢");
    assert_eq!(color2, [76, 175, 80, 255]);
}

#[test]
fn trust_level_style_elevated_variants() {
    let (emoji, color) = trust_level_style("Elevated (2)");
    assert_eq!(emoji, "🟠");
    assert_eq!(color, [255, 152, 0, 255]);
    let (emoji2, _) = trust_level_style("(2)");
    assert_eq!(emoji2, "🟠");
}

#[test]
fn trust_level_style_limited_variants() {
    let (emoji, color) = trust_level_style("Limited (1)");
    assert_eq!(emoji, "🟡");
    assert_eq!(color, [255, 235, 59, 255]);
    let (emoji2, _) = trust_level_style("(1)");
    assert_eq!(emoji2, "🟡");
}

#[test]
fn trust_level_style_unknown() {
    let (emoji, color) = trust_level_style("Custom");
    assert_eq!(emoji, "⚫");
    assert_eq!(color, [158, 158, 158, 255]);
}

// === Integration tests (tool-level) ===

#[test]
fn test_trust_dashboard_creation() {
    let dashboard = TrustDashboard::new();
    assert!(!dashboard.visible);
    assert_eq!(dashboard.trust_summary().total_primals, 0);
}

#[test]
fn test_update_from_primals() {
    let mut dashboard = TrustDashboard::new();

    let primals = vec![
        create_test_primal("p1", Some(3), Some("family-a")),
        create_test_primal("p2", Some(2), Some("family-a")),
        create_test_primal("p3", Some(1), Some("family-b")),
        create_test_primal("p4", Some(0), None),
    ];

    dashboard.update_from_primals(&primals);

    assert_eq!(dashboard.trust_summary().total_primals, 4);
    assert_eq!(dashboard.trust_summary().family_count, 3);
    assert_eq!(dashboard.trust_summary().unique_families, 2);
    assert!(dashboard.trust_summary().average_trust.is_some());
    assert!((dashboard.trust_summary().average_trust.unwrap() - 1.5).abs() < 0.01);
}

#[test]
fn test_trust_distribution() {
    let mut dashboard = TrustDashboard::new();

    let primals = vec![
        create_test_primal("p1", Some(3), None),
        create_test_primal("p2", Some(3), None),
        create_test_primal("p3", Some(2), None),
        create_test_primal("p4", Some(1), None),
    ];

    dashboard.update_from_primals(&primals);

    assert_eq!(
        dashboard.trust_summary().trust_distribution.get("Full (3)"),
        Some(&2)
    );
    assert_eq!(
        dashboard
            .trust_summary()
            .trust_distribution
            .get("Elevated (2)"),
        Some(&1)
    );
    assert_eq!(
        dashboard
            .trust_summary()
            .trust_distribution
            .get("Limited (1)"),
        Some(&1)
    );
}

#[test]
fn test_empty_primals() {
    let mut dashboard = TrustDashboard::new();
    dashboard.update_from_primals(&[]);

    assert_eq!(dashboard.trust_summary().total_primals, 0);
    assert_eq!(dashboard.trust_summary().family_count, 0);
    assert_eq!(dashboard.trust_summary().unique_families, 0);
    assert!(dashboard.trust_summary().average_trust.is_none());
}

#[test]
fn test_trust_string_property() {
    let mut dashboard = TrustDashboard::new();
    let mut props = Properties::new();
    props.insert(
        "trust_level".to_string(),
        PropertyValue::String("Custom".to_string()),
    );
    let primals = vec![PrimalInfo {
        id: "p1".to_string().into(),
        name: "Test".to_string(),
        primal_type: "Test".to_string(),
        endpoint: "http://test".to_string(),
        capabilities: vec![],
        health: PrimalHealthStatus::Healthy,
        last_seen: 0,
        endpoints: None,
        metadata: None,
        properties: props,
        #[expect(deprecated)]
        trust_level: None,
        #[expect(deprecated)]
        family_id: None,
    }];
    dashboard.update_from_primals(&primals);
    assert_eq!(dashboard.trust_summary().total_primals, 1);
    assert_eq!(
        dashboard.trust_summary().trust_distribution.get("Custom"),
        Some(&1)
    );
}

#[test]
fn test_trust_unknown_level() {
    let mut dashboard = TrustDashboard::new();
    let mut props = Properties::new();
    props.insert("trust_level".to_string(), PropertyValue::Number(99.0));
    let primals = vec![PrimalInfo {
        id: "p1".to_string().into(),
        name: "Test".to_string(),
        primal_type: "Test".to_string(),
        endpoint: "http://test".to_string(),
        capabilities: vec![],
        health: PrimalHealthStatus::Healthy,
        last_seen: 0,
        endpoints: None,
        metadata: None,
        properties: props,
        #[expect(deprecated)]
        trust_level: None,
        #[expect(deprecated)]
        family_id: None,
    }];
    dashboard.update_from_primals(&primals);
    assert!(
        dashboard
            .trust_summary()
            .trust_distribution
            .contains_key("Unknown (99)")
    );
}

#[test]
fn test_trust_dashboard_visible_toggle() {
    let mut dashboard = TrustDashboard::new();
    assert!(!dashboard.visible);
    dashboard.visible = true;
    assert!(dashboard.visible);
}

#[test]
fn test_trust_dashboard_default() {
    let dashboard = TrustDashboard::default();
    assert!(!dashboard.visible);
}

#[test]
fn test_trust_level_number_to_label() {
    assert_eq!(trust_level_number_to_label(0), "None (0)");
    assert_eq!(trust_level_number_to_label(1), "Limited (1)");
    assert_eq!(trust_level_number_to_label(2), "Elevated (2)");
    assert_eq!(trust_level_number_to_label(3), "Full (3)");
    assert_eq!(trust_level_number_to_label(4), "Unknown (4)");
    assert_eq!(trust_level_number_to_label(99), "Unknown (99)");
    assert_eq!(trust_level_number_to_label(-1), "Unknown (-1)");
}

#[test]
fn test_update_from_primals_properties_over_deprecated() {
    let mut dashboard = TrustDashboard::new();
    let mut props = Properties::new();
    props.insert("trust_level".to_string(), PropertyValue::Number(2.0));
    props.insert(
        "family_id".to_string(),
        PropertyValue::String("fam-x".to_string()),
    );
    let primals = vec![PrimalInfo {
        id: "p1".to_string().into(),
        name: "Test".to_string(),
        primal_type: "Test".to_string(),
        endpoint: "http://test".to_string(),
        capabilities: vec![],
        health: PrimalHealthStatus::Healthy,
        last_seen: 0,
        endpoints: None,
        metadata: None,
        properties: props,
        #[expect(deprecated)]
        trust_level: None,
        #[expect(deprecated)]
        family_id: None,
    }];
    dashboard.update_from_primals(&primals);
    assert_eq!(dashboard.trust_summary().total_primals, 1);
    assert_eq!(dashboard.trust_summary().unique_families, 1);
    assert_eq!(
        dashboard
            .trust_summary()
            .trust_distribution
            .get("Elevated (2)"),
        Some(&1)
    );
}

#[test]
fn test_display_state_rebuilt_on_update() {
    let mut dashboard = TrustDashboard::new();
    let primals = vec![
        create_test_primal("p1", Some(3), Some("fam")),
        create_test_primal("p2", Some(3), None),
    ];
    dashboard.update_from_primals(&primals);
    let ds = dashboard.display_state();
    assert_eq!(ds.total_primals, 2);
    assert_eq!(ds.rows.len(), 1);
    assert_eq!(ds.rows[0].count, 2);
    assert!(ds.average.is_some());
    assert_eq!(ds.average.as_ref().unwrap().label, "Full");
}

#[test]
fn test_display_state_accessor() {
    let dashboard = TrustDashboard::new();
    let ds = dashboard.display_state();
    assert_eq!(ds.total_primals, 0);
}

#[test]
fn trust_display_state_construction_and_field_access() {
    let mut dist = HashMap::new();
    dist.insert("Full (3)".to_string(), 2);
    let summary = TrustSummary {
        trust_distribution: dist,
        total_primals: 2,
        family_count: 1,
        unique_families: 1,
        average_trust: Some(3.0),
    };
    let ds = prepare_trust_display(&summary, 7);
    assert_eq!(ds.total_primals, 2);
    assert_eq!(ds.family_count, 1);
    assert_eq!(ds.unique_families, 1);
    assert_eq!(ds.last_update_label, "Updated 7 seconds ago");
    assert_eq!(ds.rows.len(), 1);
    assert_eq!(ds.rows[0].label, "Full (3)");
    assert_eq!(ds.rows[0].count, 2);
    assert_eq!(ds.rows[0].color, [76, 175, 80, 255]);
    let avg = ds.average.as_ref().unwrap();
    assert_eq!(avg.value, 3.0);
    assert_eq!(avg.label, "Full");
    assert_eq!(avg.color, [76, 175, 80, 255]);
}
