// SPDX-License-Identifier: AGPL-3.0-or-later

use super::super::compute::trust_level_to_display_row;

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
