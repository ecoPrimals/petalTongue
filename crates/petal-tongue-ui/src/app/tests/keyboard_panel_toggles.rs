// SPDX-License-Identifier: AGPL-3.0-or-later

use super::super::*;

#[test]
fn run_update_with_key_p_toggles_neural_proprioception() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    app.motor_sender()
        .send(petal_tongue_core::MotorCommand::SetAwakening { enabled: false })
        .ok();
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |_| {
        super::super::update::run_update(&mut app, &ctx);
    });
    let before = app
        .introspect()
        .visible_panels
        .iter()
        .any(|p| format!("{:?}", p.id).contains("Proprioception") && p.visible);
    let mut raw = egui::RawInput::default();
    raw.events.push(egui::Event::Key {
        key: egui::Key::P,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    });
    let _ = ctx.run(raw, |ctx| {
        super::super::update::run_update(&mut app, ctx);
    });
    let after = app
        .introspect()
        .visible_panels
        .iter()
        .any(|p| format!("{:?}", p.id).contains("Proprioception") && p.visible);
    assert_ne!(before, after, "P key should toggle neural proprioception");
}

#[test]
fn run_update_with_key_m_toggles_neural_metrics() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    app.motor_sender()
        .send(petal_tongue_core::MotorCommand::SetAwakening { enabled: false })
        .ok();
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |_| {
        super::super::update::run_update(&mut app, &ctx);
    });
    let before = app
        .introspect()
        .visible_panels
        .iter()
        .any(|p| format!("{:?}", p.id).contains("metrics") && p.visible);
    let mut raw = egui::RawInput::default();
    raw.events.push(egui::Event::Key {
        key: egui::Key::M,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    });
    let _ = ctx.run(raw, |ctx| {
        super::super::update::run_update(&mut app, ctx);
    });
    let after = app
        .introspect()
        .visible_panels
        .iter()
        .any(|p| format!("{:?}", p.id).contains("metrics") && p.visible);
    assert_ne!(before, after, "M key should toggle neural metrics");
}

#[test]
fn run_update_with_ctrl_a_toggles_accessibility() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    app.motor_sender()
        .send(petal_tongue_core::MotorCommand::SetAwakening { enabled: false })
        .ok();
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        super::super::update::run_update(&mut app, ctx);
    });
    let before = app
        .introspect()
        .visible_panels
        .iter()
        .any(|p| format!("{:?}", p.id).contains("accessibility") && p.visible);
    let mut raw = egui::RawInput {
        modifiers: egui::Modifiers {
            ctrl: true,
            ..Default::default()
        },
        ..Default::default()
    };
    raw.events.push(egui::Event::Key {
        key: egui::Key::A,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers {
            ctrl: true,
            ..Default::default()
        },
    });
    let _ = ctx.run(raw, |ctx| {
        super::super::update::run_update(&mut app, ctx);
    });
    let after = app
        .introspect()
        .visible_panels
        .iter()
        .any(|p| format!("{:?}", p.id).contains("accessibility") && p.visible);
    assert_ne!(before, after, "Ctrl+A should toggle accessibility");
}

#[test]
fn run_update_with_ctrl_d_toggles_dashboard() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    app.motor_sender()
        .send(petal_tongue_core::MotorCommand::SetAwakening { enabled: false })
        .ok();
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        super::super::update::run_update(&mut app, ctx);
    });
    let before = app
        .introspect()
        .visible_panels
        .iter()
        .any(|p| format!("{:?}", p.id).contains("SystemDashboard") && p.visible);
    let mut raw = egui::RawInput {
        modifiers: egui::Modifiers {
            ctrl: true,
            ..Default::default()
        },
        ..Default::default()
    };
    raw.events.push(egui::Event::Key {
        key: egui::Key::D,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers {
            ctrl: true,
            ..Default::default()
        },
    });
    let _ = ctx.run(raw, |ctx| {
        super::super::update::run_update(&mut app, ctx);
    });
    let after = app
        .introspect()
        .visible_panels
        .iter()
        .any(|p| format!("{:?}", p.id).contains("SystemDashboard") && p.visible);
    assert_ne!(before, after, "Ctrl+D should toggle dashboard");
}

#[test]
fn run_update_with_escape_closes_overlays() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    app.motor_sender()
        .send(petal_tongue_core::MotorCommand::SetAwakening { enabled: false })
        .ok();
    let ctx = egui::Context::default();
    app.keyboard_shortcuts.show_help = true;
    app.accessibility_panel.show = true;
    let mut raw = egui::RawInput::default();
    raw.events.push(egui::Event::Key {
        key: egui::Key::Escape,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    });
    let _ = ctx.run(raw, |ctx| {
        super::super::update::run_update(&mut app, ctx);
    });
    assert!(!app.keyboard_shortcuts.show_help);
    assert!(!app.accessibility_panel.show);
}
