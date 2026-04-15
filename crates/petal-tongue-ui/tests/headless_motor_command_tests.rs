// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Motor command integration tests — layout, navigation, audio, and display commands.
//!
//! Panel visibility and mode-switching tests live in `headless_motor_panel_tests.rs`.

use petal_tongue_ui::headless_harness::HeadlessHarness;

#[test]
fn motor_command_fit_to_view() {
    use petal_tongue_core::LayoutAlgorithm;
    use petal_tongue_ui::tutorial_mode::TutorialMode;

    let mut harness = HeadlessHarness::new().unwrap();
    let graph = harness.app_mut().graph_handle();
    TutorialMode::create_fallback_scenario(graph, LayoutAlgorithm::ForceDirected);
    harness.run_frame();

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::FitToView)
        .unwrap();
    harness.run_frame();
    let _ = harness.tessellate();
}

#[test]
fn motor_command_set_zoom() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetZoom { level: 1.5 })
        .unwrap();
    harness.run_frame();

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetZoom { level: 0.5 })
        .unwrap();
    harness.run_frame();
    let _ = harness.tessellate();
}

#[test]
fn motor_command_navigate_to_node() {
    use petal_tongue_core::LayoutAlgorithm;
    use petal_tongue_ui::tutorial_mode::TutorialMode;

    let mut harness = HeadlessHarness::new().unwrap();
    let graph = harness.app_mut().graph_handle();
    TutorialMode::create_fallback_scenario(graph, LayoutAlgorithm::ForceDirected);
    harness.run_frame();

    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::Navigate {
            target_node: "discovery-example".to_string(),
        })
        .unwrap();
    harness.run_frame();
    let _ = harness.tessellate();
}

#[test]
fn motor_command_play_audio() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::PlayAudio {
            sound: "notification".to_string(),
        })
        .unwrap();
    harness.run_frame();
    let _ = harness.tessellate();
}

#[test]
fn motor_command_set_physics() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetPhysics { enabled: true })
        .unwrap();
    harness.run_frame();
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetPhysics { enabled: false })
        .unwrap();
    harness.run_frame();
}

#[test]
fn motor_command_set_scene_animation() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetSceneAnimation { enabled: true })
        .unwrap();
    harness.run_frame();
}

#[test]
fn zoom_extreme_values() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetZoom { level: 0.1 })
        .unwrap();
    harness.run_frame();
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetZoom { level: 3.0 })
        .unwrap();
    harness.run_frame();
    let _ = harness.tessellate();
}

#[test]
fn layout_ecosystem_algorithm() {
    use petal_tongue_core::LayoutAlgorithm;
    use petal_tongue_ui::tutorial_mode::TutorialMode;

    let mut harness = HeadlessHarness::new().unwrap();
    let graph = harness.app_mut().graph_handle();
    TutorialMode::create_fallback_scenario(graph, LayoutAlgorithm::ForceDirected);
    harness.run_frame();
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetLayout {
            algorithm: "Ecosystem".to_string(),
        })
        .unwrap();
    harness.run_frames(3);
    let intro = harness.last_introspection().expect("frames ran");
    assert!(!intro.bound_data.is_empty());
}

#[test]
fn layout_hierarchical_algorithm() {
    use petal_tongue_core::LayoutAlgorithm;
    use petal_tongue_ui::tutorial_mode::TutorialMode;

    let mut harness = HeadlessHarness::new().unwrap();
    let graph = harness.app_mut().graph_handle();
    TutorialMode::create_fallback_scenario(graph, LayoutAlgorithm::ForceDirected);
    harness.run_frame();
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetLayout {
            algorithm: "Hierarchical".to_string(),
        })
        .unwrap();
    harness.run_frames(3);
    let intro = harness.last_introspection().expect("frames ran");
    assert!(!intro.bound_data.is_empty());
}

#[test]
fn layout_circular_algorithm() {
    use petal_tongue_core::LayoutAlgorithm;
    use petal_tongue_ui::tutorial_mode::TutorialMode;

    let mut harness = HeadlessHarness::new().unwrap();
    let graph = harness.app_mut().graph_handle();
    TutorialMode::create_fallback_scenario(graph, LayoutAlgorithm::ForceDirected);
    harness.run_frame();
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetLayout {
            algorithm: "Circular".to_string(),
        })
        .unwrap();
    harness.run_frames(3);
    let _ = harness.tessellate();
}

#[test]
fn layout_random_algorithm() {
    use petal_tongue_core::LayoutAlgorithm;
    use petal_tongue_ui::tutorial_mode::TutorialMode;

    let mut harness = HeadlessHarness::new().unwrap();
    let graph = harness.app_mut().graph_handle();
    TutorialMode::create_fallback_scenario(graph, LayoutAlgorithm::ForceDirected);
    harness.run_frame();
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetLayout {
            algorithm: "Random".to_string(),
        })
        .unwrap();
    harness.run_frames(3);
    let _ = harness.tessellate();
}

#[test]
fn layout_force_directed_algorithm() {
    use petal_tongue_core::LayoutAlgorithm;
    use petal_tongue_ui::tutorial_mode::TutorialMode;

    let mut harness = HeadlessHarness::new().unwrap();
    let graph = harness.app_mut().graph_handle();
    TutorialMode::create_fallback_scenario(graph, LayoutAlgorithm::ForceDirected);
    harness.run_frame();
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetLayout {
            algorithm: "ForceDirected".to_string(),
        })
        .unwrap();
    harness.run_frames(3);
    let _ = harness.tessellate();
}

#[test]
fn layout_sequence_all_algorithms() {
    use petal_tongue_core::LayoutAlgorithm;
    use petal_tongue_ui::tutorial_mode::TutorialMode;

    let mut harness = HeadlessHarness::new().unwrap();
    let graph = harness.app_mut().graph_handle();
    TutorialMode::create_fallback_scenario(graph, LayoutAlgorithm::ForceDirected);
    harness.run_frame();

    for algo in [
        "ForceDirected",
        "Hierarchical",
        "Circular",
        "Random",
        "Grid",
        "Radial",
        "Ecosystem",
    ] {
        harness
            .app_mut()
            .motor_sender()
            .send(petal_tongue_core::MotorCommand::SetLayout {
                algorithm: algo.to_string(),
            })
            .unwrap();
        harness.run_frames(2);
    }
    let _ = harness.tessellate();
}

#[test]
fn motor_command_set_awakening_enabled() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::SetAwakening { enabled: true })
        .unwrap();
    harness.run_frames(3);
    let _ = harness.tessellate();
}

#[test]
fn motor_command_update_display() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::UpdateDisplay)
        .unwrap();
    harness.run_frame();
    let _ = harness.tessellate();
}

#[test]
fn motor_command_clear_display() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::ClearDisplay)
        .unwrap();
    harness.run_frame();
    let _ = harness.tessellate();
}

#[test]
fn motor_command_render_frame() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    harness
        .app_mut()
        .motor_sender()
        .send(petal_tongue_core::MotorCommand::RenderFrame { frame_id: 42 })
        .unwrap();
    harness.run_frame();
    assert!(harness.frame_count() >= 2);
}

#[test]
fn play_audio_multiple_sounds() {
    let mut harness = HeadlessHarness::new().unwrap();
    harness.run_frame();
    for sound in ["success", "warning", "notification", "error"] {
        harness
            .app_mut()
            .motor_sender()
            .send(petal_tongue_core::MotorCommand::PlayAudio {
                sound: sound.to_string(),
            })
            .unwrap();
        harness.run_frame();
    }
    let _ = harness.tessellate();
}
