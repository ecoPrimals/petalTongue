// SPDX-License-Identifier: AGPL-3.0-or-later

use super::super::compute::{average_trust_display, prepare_trust_display};
use super::super::types::TrustSummary;
use std::collections::HashMap;

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
