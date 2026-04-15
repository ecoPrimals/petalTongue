// SPDX-License-Identifier: AGPL-3.0-or-later

use super::super::*;

#[test]
fn run_update_multiple_frames() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    let ctx = egui::Context::default();
    for _ in 0..3 {
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            super::super::update::run_update(&mut app, ctx);
        });
    }
    assert!(app.frame_count() >= 3);
}

#[test]
fn run_update_with_keyboard_shortcut_handling() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        super::super::update::run_update(&mut app, ctx);
    });
    let _ = app.introspect();
}

#[test]
fn process_sensory_feedback_increments_frame() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    let ctx = egui::Context::default();
    let initial = app.frame_count();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        super::super::sensory::process_sensory_feedback(&mut app, ctx);
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
        super::super::sensory::process_sensory_feedback(&mut app, ctx);
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
        super::super::sensory::process_sensory_feedback(&mut app, ctx);
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
        super::super::sensory::process_sensory_feedback(&mut app, ctx);
    });
    assert!(app.frame_count() >= 1);
}

#[test]
fn run_update_awakening_path_executes() {
    let mut app = PetalTongueApp::new_headless().expect("headless app");
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        super::super::update::run_update(&mut app, ctx);
    });
    assert!(app.frame_count() >= 1);
    let intro = app.introspect();
    assert!(
        intro.is_panel_visible(petal_tongue_core::PanelKind::Awakening),
        "headless starts with awakening overlay visible"
    );
}
