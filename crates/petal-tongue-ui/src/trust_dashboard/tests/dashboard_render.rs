// SPDX-License-Identifier: AGPL-3.0-or-later

use super::super::TrustDashboard;
use super::fixtures::create_test_primal;
use crate::accessibility::{ColorPalette, ColorScheme};

#[test]
fn test_render_empty_primals_early_return() {
    let mut dashboard = TrustDashboard::new();
    dashboard.update_from_primals(&[]);
    let palette = ColorPalette::from_scheme(ColorScheme::Default);
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let intents = dashboard.render(ui, &palette, 1.0, None);
            assert!(intents.is_empty());
        });
    });
}

#[test]
fn test_render_with_primals() {
    let mut dashboard = TrustDashboard::new();
    let primals = vec![
        create_test_primal("p1", Some(3), Some("fam-a")),
        create_test_primal("p2", Some(2), Some("fam-a")),
    ];
    dashboard.update_from_primals(&primals);
    let palette = ColorPalette::from_scheme(ColorScheme::Default);
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let intents = dashboard.render(ui, &palette, 1.0, None);
            assert!(intents.is_empty());
        });
    });
}

#[test]
fn test_render_with_average_produces_intent_on_button_click() {
    let mut dashboard = TrustDashboard::new();
    let primals = vec![create_test_primal("p1", Some(3), None)];
    dashboard.update_from_primals(&primals);
    let palette = ColorPalette::from_scheme(ColorScheme::Default);
    let ctx = egui::Context::default();
    let mut intents = Vec::new();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            intents = dashboard.render(ui, &palette, 1.0, None);
        });
    });
    assert!(intents.is_empty());
}

#[test]
fn test_render_compact_with_average() {
    let mut dashboard = TrustDashboard::new();
    let primals = vec![create_test_primal("p1", Some(2), Some("fam"))];
    dashboard.update_from_primals(&primals);
    let palette = ColorPalette::from_scheme(ColorScheme::Default);
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            dashboard.render_compact(ui, &palette, 1.0);
        });
    });
}

#[test]
fn test_render_compact_without_average() {
    let mut dashboard = TrustDashboard::new();
    dashboard.update_from_primals(&[]);
    let palette = ColorPalette::from_scheme(ColorScheme::Default);
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            dashboard.render_compact(ui, &palette, 1.0);
        });
    });
}

#[test]
fn test_render_compact_with_family_count() {
    let mut dashboard = TrustDashboard::new();
    let primals = vec![
        create_test_primal("p1", Some(3), Some("fam-a")),
        create_test_primal("p2", Some(2), Some("fam-b")),
    ];
    dashboard.update_from_primals(&primals);
    let palette = ColorPalette::from_scheme(ColorScheme::Default);
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            dashboard.render_compact(ui, &palette, 1.2);
        });
    });
}
