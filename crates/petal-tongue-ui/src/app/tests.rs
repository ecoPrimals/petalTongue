// SPDX-License-Identifier: AGPL-3.0-or-later
//! Unit tests for petalTongue application.

use super::*;
use std::sync::{Arc, RwLock};

#[test]
fn mode_presets_produce_commands() {
    let cmds = crate::mode_presets::commands_for_mode("clinical");
    assert!(!cmds.is_empty());
    let cmds = crate::mode_presets::commands_for_mode("developer");
    assert!(!cmds.is_empty());
}

#[test]
fn headless_app_introspect() {
    let app = PetalTongueApp::new_headless().expect("headless app");
    let introspection = app.introspect();
    let _ = introspection.frame_id;
    assert!(!introspection.visible_panels.is_empty());
    assert!(
        introspection
            .active_modalities
            .contains(&petal_tongue_core::interaction::perspective::OutputModality::Gui)
    );
}

#[test]
fn headless_app_motor_sender() {
    let app = PetalTongueApp::new_headless().expect("headless app");
    let tx = app.motor_sender();
    let _ = tx;
}

#[test]
fn headless_app_graph_handle() {
    let app = PetalTongueApp::new_headless().expect("headless app");
    let handle = app.graph_handle();
    let _guard = handle.read().unwrap();
}

#[test]
fn headless_app_getters() {
    let app = PetalTongueApp::new_headless().expect("headless app");
    assert_eq!(app.frame_count(), 0);
    assert!(!app.is_help_visible());
    assert!(!app.is_continuous_mode());
    assert_eq!(app.active_session_count(), 0);
}

#[test]
fn mode_presets_unknown_returns_empty() {
    let cmds = crate::mode_presets::commands_for_mode("nonexistent_mode_xyz");
    assert!(cmds.is_empty());
}

#[test]
fn introspect_panel_snapshots() {
    let app = PetalTongueApp::new_headless().expect("headless app");
    let introspection = app.introspect();
    let panel_ids: Vec<_> = introspection
        .visible_panels
        .iter()
        .map(|p| format!("{:?}", p.id))
        .collect();
    assert!(
        panel_ids.iter().any(|s| s.contains("TopMenu")),
        "should have TopMenu in panels"
    );
    assert!(
        panel_ids.iter().any(|s| s.contains("graph_canvas")),
        "should have graph_canvas"
    );
}

#[test]
fn introspect_bound_data_from_default_graph() {
    let app = PetalTongueApp::new_headless().expect("headless app");
    let introspection = app.introspect();
    for bd in &introspection.bound_data {
        assert!(
            !bd.data_object_id.is_empty(),
            "bound data objects must have an id"
        );
    }
}

#[test]
fn introspect_possible_interactions() {
    let app = PetalTongueApp::new_headless().expect("headless app");
    let introspection = app.introspect();
    assert!(
        !introspection.possible_interactions.is_empty(),
        "should have at least Navigate"
    );
    assert!(
        introspection
            .possible_interactions
            .iter()
            .any(|i| matches!(i.intent, petal_tongue_core::InteractionKind::Navigate)),
        "should have Navigate"
    );
}

#[test]
fn set_visualization_state_and_active_session_count() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    assert_eq!(app.active_session_count(), 0);
    let state = Arc::new(RwLock::new(petal_tongue_ipc::VisualizationState::new()));
    app.set_visualization_state(state);
    assert_eq!(app.active_session_count(), 0);
}

#[test]
fn set_sensor_stream_and_interaction_subscribers() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    let sensor_reg = Arc::new(RwLock::new(petal_tongue_ipc::SensorStreamRegistry::new()));
    let interaction_reg = Arc::new(RwLock::new(
        petal_tongue_ipc::InteractionSubscriberRegistry::new(),
    ));
    app.set_sensor_stream(sensor_reg);
    app.set_interaction_subscribers(interaction_reg);
}

#[test]
fn update_headless_single_frame() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        app.update_headless(ctx);
    });
}

#[test]
fn tick_clock_access() {
    let app = PetalTongueApp::new_headless().expect("headless app");
    let clock = app.tick_clock();
    let _ = clock;
}

#[test]
#[allow(unused_mut)]
fn headless_app_refresh_graph_data() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    app.refresh_graph_data();
    let _introspection = app.introspect();
}

#[test]
fn mode_presets_developer() {
    let cmds = crate::mode_presets::commands_for_mode("developer");
    assert!(!cmds.is_empty());
}

#[test]
fn mode_presets_presentation() {
    let cmds = crate::mode_presets::commands_for_mode("presentation");
    assert!(!cmds.is_empty());
}

#[test]
fn headless_app_introspect_with_visibility() {
    let app = PetalTongueApp::new_headless().expect("headless app");
    let i1 = app.introspect();
    let visible_count = i1.visible_panels.iter().filter(|p| p.visible).count();
    assert!(visible_count >= 1);
}

#[test]
fn run_update_multiple_frames() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    let ctx = egui::Context::default();
    for _ in 0..3 {
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            super::update::run_update(&mut app, ctx);
        });
    }
    assert!(app.frame_count() >= 3);
}

#[test]
fn run_update_with_keyboard_shortcut_handling() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        super::update::run_update(&mut app, ctx);
    });
    let _ = app.introspect();
}

#[test]
fn process_sensory_feedback_increments_frame() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    let ctx = egui::Context::default();
    let initial = app.frame_count();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        super::sensory::process_sensory_feedback(&mut app, ctx);
    });
    assert_eq!(app.frame_count(), initial + 1);
}

#[test]
fn process_sensory_feedback_with_pointer_click() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    let ctx = egui::Context::default();
    let mut raw = egui::RawInput::default();
    raw.events.push(egui::Event::PointerButton {
        pos: egui::Pos2::new(100.0, 100.0),
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::NONE,
    });
    raw.events.push(egui::Event::PointerButton {
        pos: egui::Pos2::new(100.0, 100.0),
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    });
    let _ = ctx.run(raw, |ctx| {
        super::sensory::process_sensory_feedback(&mut app, ctx);
    });
    assert!(app.frame_count() >= 1);
}

#[test]
fn process_sensory_feedback_with_pointer_hover() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    let ctx = egui::Context::default();
    let mut raw = egui::RawInput::default();
    raw.events
        .push(egui::Event::PointerMoved(egui::Pos2::new(50.0, 50.0)));
    let _ = ctx.run(raw, |ctx| {
        super::sensory::process_sensory_feedback(&mut app, ctx);
    });
    assert!(app.frame_count() >= 1);
}

#[test]
fn process_sensory_feedback_with_key_event() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    let ctx = egui::Context::default();
    let mut raw = egui::RawInput::default();
    raw.events.push(egui::Event::Key {
        key: egui::Key::Space,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    });
    let _ = ctx.run(raw, |ctx| {
        super::sensory::process_sensory_feedback(&mut app, ctx);
    });
    assert!(app.frame_count() >= 1);
}

#[test]
fn run_update_with_key_p_toggles_neural_proprioception() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    app.motor_sender()
        .send(petal_tongue_core::MotorCommand::SetAwakening { enabled: false })
        .ok();
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |_| {
        super::update::run_update(&mut app, &ctx);
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
        super::update::run_update(&mut app, ctx);
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
        super::update::run_update(&mut app, &ctx);
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
        super::update::run_update(&mut app, ctx);
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
        super::update::run_update(&mut app, ctx);
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
        super::update::run_update(&mut app, ctx);
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
        super::update::run_update(&mut app, ctx);
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
        super::update::run_update(&mut app, ctx);
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
        super::update::run_update(&mut app, ctx);
    });
    assert!(!app.keyboard_shortcuts.show_help);
    assert!(!app.accessibility_panel.show);
}

#[test]
fn run_update_awakening_path_executes() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        super::update::run_update(&mut app, ctx);
    });
    assert!(app.frame_count() >= 1);
    let intro = app.introspect();
    assert!(
        intro.is_panel_visible(petal_tongue_core::PanelKind::Awakening),
        "headless starts with awakening overlay visible"
    );
}

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
        super::update::run_update(&mut app, ctx);
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
        super::update::run_update(&mut app, ctx);
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
        super::update::run_update(&mut app, ctx);
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
        super::update::run_update(&mut app, ctx);
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
        super::update::run_update(&mut app, ctx);
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
        super::update::run_update(&mut app, ctx);
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
        super::update::run_update(&mut app, ctx);
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
        super::update::run_update(&mut app, ctx);
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
        super::update::run_update(&mut app, ctx);
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
        super::update::run_update(&mut app, ctx);
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
        super::update::run_update(&mut app, ctx);
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
        super::update::run_update(&mut app, ctx);
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
        super::update::run_update(&mut app, ctx);
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
        super::update::run_update(&mut app, ctx);
    });
    assert!(app.keyboard_shortcuts.show_help);
}
