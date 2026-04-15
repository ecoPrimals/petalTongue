// SPDX-License-Identifier: AGPL-3.0-or-later

use super::super::*;
use std::sync::{Arc, RwLock};

#[test]
fn run_update_with_ctrl_r_refreshes() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    app.motor_sender()
        .send(petal_tongue_core::MotorCommand::SetAwakening { enabled: false })
        .ok();
    let ctx = egui::Context::default();
    let mut raw = egui::RawInput::default();
    raw.events.push(egui::Event::Key {
        key: egui::Key::R,
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
    let _ = app.introspect();
}

#[test]
fn run_update_with_ctrl_plus_increases_font_size() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    app.motor_sender()
        .send(petal_tongue_core::MotorCommand::SetAwakening { enabled: false })
        .ok();
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        super::super::update::run_update(&mut app, ctx);
    });
    let before = app.accessibility_panel.settings.font_size;
    let mut raw = egui::RawInput {
        modifiers: egui::Modifiers {
            ctrl: true,
            ..Default::default()
        },
        ..Default::default()
    };
    raw.events.push(egui::Event::Key {
        key: egui::Key::Plus,
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
    let after = app.accessibility_panel.settings.font_size;
    assert_ne!(before, after, "Ctrl+Plus should increase font size");
}

#[test]
fn run_update_with_ctrl_minus_decreases_font_size() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    app.motor_sender()
        .send(petal_tongue_core::MotorCommand::SetAwakening { enabled: false })
        .ok();
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        super::super::update::run_update(&mut app, ctx);
    });
    let before = app.accessibility_panel.settings.font_size;
    let mut raw = egui::RawInput {
        modifiers: egui::Modifiers {
            ctrl: true,
            ..Default::default()
        },
        ..Default::default()
    };
    raw.events.push(egui::Event::Key {
        key: egui::Key::Minus,
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
    let after = app.accessibility_panel.settings.font_size;
    assert_ne!(before, after, "Ctrl+Minus should decrease font size");
}

#[test]
fn run_update_with_key_g_toggles_graph_builder() {
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
        .any(|p| format!("{:?}", p.id).contains("graph_builder") && p.visible);
    let mut raw = egui::RawInput::default();
    raw.events.push(egui::Event::Key {
        key: egui::Key::G,
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
        .any(|p| format!("{:?}", p.id).contains("graph_builder") && p.visible);
    assert_ne!(before, after, "G key should toggle graph builder");
}

#[test]
fn run_update_with_sensor_stream_broadcasts_events() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    app.motor_sender()
        .send(petal_tongue_core::MotorCommand::SetAwakening { enabled: false })
        .ok();
    let sensor_reg = Arc::new(RwLock::new(petal_tongue_ipc::SensorStreamRegistry::new()));
    app.set_sensor_stream(sensor_reg.clone());
    let ctx = egui::Context::default();
    let mut raw = egui::RawInput::default();
    raw.events
        .push(egui::Event::PointerMoved(egui::Pos2::new(50.0, 50.0)));
    let _ = ctx.run(raw, |ctx| {
        super::super::update::run_update(&mut app, ctx);
    });
    let _guard = sensor_reg.read().unwrap();
}

#[test]
fn run_update_with_interaction_subscribers_broadcasts_selection() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    app.motor_sender()
        .send(petal_tongue_core::MotorCommand::SetAwakening { enabled: false })
        .ok();
    let interaction_reg = Arc::new(RwLock::new(
        petal_tongue_ipc::InteractionSubscriberRegistry::new(),
    ));
    app.set_interaction_subscribers(interaction_reg.clone());
    app.motor_sender()
        .send(petal_tongue_core::MotorCommand::SelectNode {
            node_id: Some("test-node".to_string()),
        })
        .ok();
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        super::super::update::run_update(&mut app, ctx);
    });
    let _guard = interaction_reg.read().unwrap();
}

#[test]
fn run_update_with_ctrl_2_selects_color_scheme() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    app.motor_sender()
        .send(petal_tongue_core::MotorCommand::SetAwakening { enabled: false })
        .ok();
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        super::super::update::run_update(&mut app, ctx);
    });
    let before = app.accessibility_panel.settings.color_scheme;
    let mut raw = egui::RawInput {
        modifiers: egui::Modifiers {
            ctrl: true,
            ..Default::default()
        },
        ..Default::default()
    };
    raw.events.push(egui::Event::Key {
        key: egui::Key::Num2,
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
    let after = app.accessibility_panel.settings.color_scheme;
    assert_ne!(before, after, "Ctrl+2 should select different color scheme");
}

#[test]
fn run_update_with_show_animation_updates_engine() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    app.motor_sender()
        .send(petal_tongue_core::MotorCommand::SetAwakening { enabled: false })
        .ok();
    app.show_animation = true;
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        super::super::update::run_update(&mut app, ctx);
    });
    let _ = app.introspect();
}

#[test]
fn run_update_with_auto_refresh_triggers_refresh() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    app.motor_sender()
        .send(petal_tongue_core::MotorCommand::SetAwakening { enabled: false })
        .ok();
    app.auto_refresh = true;
    app.refresh_interval = 0.0;
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        super::super::update::run_update(&mut app, ctx);
    });
    let _ = app.introspect();
}

#[test]
fn run_update_with_help_visible_renders_help() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    app.motor_sender()
        .send(petal_tongue_core::MotorCommand::SetAwakening { enabled: false })
        .ok();
    app.keyboard_shortcuts.show_help = true;
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        super::super::update::run_update(&mut app, ctx);
    });
    assert!(app.keyboard_shortcuts.show_help);
}
