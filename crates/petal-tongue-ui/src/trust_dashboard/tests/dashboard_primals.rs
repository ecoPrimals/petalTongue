// SPDX-License-Identifier: AGPL-3.0-or-later

use super::super::TrustDashboard;
use super::fixtures::create_test_primal;
use petal_tongue_core::{PrimalHealthStatus, PrimalInfo, Properties, PropertyValue};

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
fn test_update_from_primals_trust_unknown_property_type() {
    let mut dashboard = TrustDashboard::new();
    let mut props = Properties::new();
    props.insert("trust_level".to_string(), PropertyValue::Boolean(true));
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
    }];
    dashboard.update_from_primals(&primals);
    assert_eq!(dashboard.trust_summary().total_primals, 1);
    assert_eq!(
        dashboard.trust_summary().trust_distribution.get("Unknown"),
        Some(&1)
    );
}

#[test]
fn test_update_from_primals_deprecated_trust_level_only() {
    let mut dashboard = TrustDashboard::new();
    let primals = vec![
        PrimalInfo::new(
            "p1",
            "Test",
            "Test",
            "http://test",
            vec![],
            PrimalHealthStatus::Healthy,
            0,
        )
        .with_trust_level(2),
    ];
    dashboard.update_from_primals(&primals);
    assert_eq!(dashboard.trust_summary().total_primals, 1);
    assert_eq!(
        dashboard
            .trust_summary()
            .trust_distribution
            .get("Elevated (2)"),
        Some(&1)
    );
    assert!((dashboard.trust_summary().average_trust.unwrap() - 2.0).abs() < 0.01);
}

#[test]
fn test_update_from_primals_deprecated_family_id_only() {
    let mut dashboard = TrustDashboard::new();
    let primals = vec![
        PrimalInfo::new(
            "p1",
            "Test",
            "Test",
            "http://test",
            vec![],
            PrimalHealthStatus::Healthy,
            0,
        )
        .with_family_id("legacy_fam"),
    ];
    dashboard.update_from_primals(&primals);
    assert_eq!(dashboard.trust_summary().total_primals, 1);
    assert_eq!(dashboard.trust_summary().family_count, 1);
    assert_eq!(dashboard.trust_summary().unique_families, 1);
}
