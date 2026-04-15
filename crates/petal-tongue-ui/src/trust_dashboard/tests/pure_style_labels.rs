// SPDX-License-Identifier: AGPL-3.0-or-later

use super::super::compute::{trust_level_number_to_label, trust_level_style};

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
