// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn test_visual_flower_creation() {
    let renderer = VisualFlowerRenderer::new();
    assert!((renderer.current_time - 0.0).abs() < f32::EPSILON);
    assert_eq!(renderer.current_state(), FlowerState::Closed);
}

#[test]
fn test_visual_flower_default() {
    let renderer = VisualFlowerRenderer::default();
    assert_eq!(renderer.current_state(), FlowerState::Closed);
}

#[test]
fn test_visual_flower_update() {
    let mut renderer = VisualFlowerRenderer::new();
    renderer.update(1.5);
    assert!((renderer.current_time - 1.5).abs() < f32::EPSILON);
    assert!(matches!(renderer.current_state(), FlowerState::Opening(_)));
}

#[test]
fn test_visual_flower_update_zero_delta() {
    let mut renderer = VisualFlowerRenderer::new();
    renderer.update(0.0);
    assert_eq!(renderer.current_state(), FlowerState::Closed);
}

#[test]
fn test_visual_flower_reset() {
    let mut renderer = VisualFlowerRenderer::new();
    renderer.update(2.0);
    renderer.reset();
    assert!((renderer.current_time - 0.0).abs() < f32::EPSILON);
    assert_eq!(renderer.current_state(), FlowerState::Closed);
}

#[test]
fn test_opening_percent() {
    let mut renderer = VisualFlowerRenderer::new();

    // Closed
    assert!((renderer.opening_percent() - 0.0).abs() < f32::EPSILON);

    // Opening
    renderer.update(1.5);
    let percent = renderer.opening_percent();
    assert!(percent > 0.0 && percent < 1.0);

    // Open
    renderer.update(3.0);
    assert!((renderer.opening_percent() - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_state_progression() {
    let mut renderer = VisualFlowerRenderer::new();

    // Stage 1: Closed
    assert_eq!(renderer.current_state(), FlowerState::Closed);

    // Stage 2: Opening
    renderer.update(1.5);
    assert!(matches!(renderer.current_state(), FlowerState::Opening(_)));

    // Stage 3: Open
    renderer.update(5.0);
    assert_eq!(renderer.current_state(), FlowerState::Open);
}

#[test]
fn test_state_boundary_just_below_closed() {
    let mut renderer = VisualFlowerRenderer::new();
    renderer.update(0.29); // progress = 0.29/3 < 0.1
    assert_eq!(renderer.current_state(), FlowerState::Closed);
}

#[test]
fn test_state_boundary_just_above_closed() {
    let mut renderer = VisualFlowerRenderer::new();
    renderer.update(0.31); // progress ~= 0.103 > 0.1
    assert!(matches!(renderer.current_state(), FlowerState::Opening(_)));
}

#[test]
fn test_state_boundary_just_below_open() {
    let mut renderer = VisualFlowerRenderer::new();
    renderer.update(2.69); // progress ~= 0.897 < 0.9
    assert!(matches!(renderer.current_state(), FlowerState::Opening(_)));
}

#[test]
fn test_state_boundary_just_above_open() {
    let mut renderer = VisualFlowerRenderer::new();
    renderer.update(2.71); // progress ~= 0.903 > 0.9
    assert_eq!(renderer.current_state(), FlowerState::Open);
}

#[test]
fn test_opening_percent_mid_range() {
    let mut renderer = VisualFlowerRenderer::new();
    renderer.update(1.5); // progress = 0.5, opening percent ~= 50%
    let p = renderer.opening_percent();
    assert!(p > 0.4 && p < 0.6);
}

#[test]
fn test_opening_percent_glowing_reaching_return_one() {
    // VisualFlowerRenderer's calculate_state only returns Closed/Opening/Open,
    // but opening_percent handles Glowing/Reaching as 1.0. We test via
    // current_state which can't reach those - verify Open gives 1.0
    let mut renderer = VisualFlowerRenderer::new();
    renderer.update(4.0); // progress > 0.9 -> Open
    assert_eq!(renderer.current_state(), FlowerState::Open);
    assert!((renderer.opening_percent() - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_frame_generation_progression() {
    let mut renderer = VisualFlowerRenderer::new();
    let mut prev_time = 0.0;
    for _ in 0..5 {
        renderer.update(0.5);
        prev_time += 0.5;
        assert!((renderer.current_time - prev_time).abs() < f32::EPSILON);
    }
}

#[test]
fn test_reset_clears_animation() {
    let mut renderer = VisualFlowerRenderer::new();
    renderer.update(2.5);
    assert!((renderer.current_time - 2.5).abs() < f32::EPSILON);
    renderer.reset();
    assert!((renderer.current_time - 0.0).abs() < f32::EPSILON);
    assert_eq!(renderer.current_state(), FlowerState::Closed);
}

#[test]
fn test_progress_clamping_above_one() {
    let mut renderer = VisualFlowerRenderer::new();
    renderer.update(10.0);
    assert_eq!(renderer.current_state(), FlowerState::Open);
    assert!((renderer.opening_percent() - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_opening_interpolation_exact_boundaries() {
    let mut renderer = VisualFlowerRenderer::new();
    renderer.update(0.0);
    assert_eq!(renderer.current_state(), FlowerState::Closed);
    renderer.update(0.09);
    assert_eq!(renderer.current_state(), FlowerState::Closed);
    renderer.update(2.7);
    assert_eq!(renderer.current_state(), FlowerState::Open);
    renderer.update(3.0);
    assert_eq!(renderer.current_state(), FlowerState::Open);
}

#[test]
fn test_opening_percent_linear_midpoint() {
    let mut renderer = VisualFlowerRenderer::new();
    renderer.update(1.5);
    let p = renderer.opening_percent();
    let expected = (1.5 / 3.0 - 0.1) / 0.8;
    assert!(
        (p - expected).abs() < 0.02,
        "opening percent ~{p} should be near {expected}",
    );
}

#[test]
fn test_state_transition_closed_to_opening() {
    let mut renderer = VisualFlowerRenderer::new();
    renderer.update(0.05);
    assert_eq!(renderer.current_state(), FlowerState::Closed);
    renderer.update(0.35);
    assert!(matches!(renderer.current_state(), FlowerState::Opening(_)));
}

#[test]
fn test_state_transition_opening_to_open() {
    let mut renderer = VisualFlowerRenderer::new();
    renderer.update(1.0);
    assert!(matches!(renderer.current_state(), FlowerState::Opening(_)));
    renderer.update(2.8);
    assert_eq!(renderer.current_state(), FlowerState::Open);
}

#[test]
fn test_opening_percent_increments_with_time() {
    let mut renderer = VisualFlowerRenderer::new();
    let p0 = renderer.opening_percent();
    renderer.update(0.5);
    let p1 = renderer.opening_percent();
    renderer.update(0.5);
    let p2 = renderer.opening_percent();
    assert!(p0 < p1);
    assert!(p1 < p2);
}

#[test]
fn test_update_accumulates_delta() {
    let mut renderer = VisualFlowerRenderer::new();
    renderer.update(0.1);
    renderer.update(0.2);
    renderer.update(0.3);
    assert!((renderer.current_time - 0.6).abs() < f32::EPSILON);
}

#[test]
fn test_reset_after_open_returns_to_closed() {
    let mut renderer = VisualFlowerRenderer::new();
    renderer.update(5.0);
    assert_eq!(renderer.current_state(), FlowerState::Open);
    renderer.reset();
    assert_eq!(renderer.current_state(), FlowerState::Closed);
    assert!((renderer.opening_percent() - 0.0).abs() < f32::EPSILON);
}

#[test]
fn test_opening_state_percent_range() {
    let mut renderer = VisualFlowerRenderer::new();
    renderer.update(0.5);
    if let FlowerState::Opening(p) = renderer.current_state() {
        assert!(p > 0 && p < 100);
    } else {
        panic!("expected Opening");
    }
}

#[test]
fn test_update_negative_delta() {
    let mut renderer = VisualFlowerRenderer::new();
    renderer.update(1.0);
    renderer.update(-1.5);
    assert!((renderer.current_time - (-0.5)).abs() < f32::EPSILON);
    assert_eq!(renderer.current_state(), FlowerState::Closed);
}

#[test]
fn test_update_very_large_time() {
    let mut renderer = VisualFlowerRenderer::new();
    renderer.update(1_000_000.0);
    assert_eq!(renderer.current_state(), FlowerState::Open);
    assert!((renderer.opening_percent() - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_exact_progress_boundary_0_1() {
    let mut renderer = VisualFlowerRenderer::new();
    renderer.update(0.29);
    assert_eq!(renderer.current_state(), FlowerState::Closed);
    renderer.update(0.02);
    assert!(matches!(renderer.current_state(), FlowerState::Opening(_)));
}

#[test]
fn test_exact_progress_boundary_0_9() {
    let mut renderer = VisualFlowerRenderer::new();
    renderer.update(2.69);
    assert!(matches!(renderer.current_state(), FlowerState::Opening(_)));
    renderer.update(0.02);
    assert_eq!(renderer.current_state(), FlowerState::Open);
}

#[test]
fn test_visual_properties_at_closed() {
    let renderer = VisualFlowerRenderer::new();
    assert!((renderer.opening_percent() - 0.0).abs() < f32::EPSILON);
    assert_eq!(renderer.current_state(), FlowerState::Closed);
}

#[test]
fn test_visual_properties_at_mid_opening() {
    let mut renderer = VisualFlowerRenderer::new();
    renderer.update(1.2);
    let state = renderer.current_state();
    assert!(matches!(state, FlowerState::Opening(_)));
    let p = renderer.opening_percent();
    assert!(p > 0.2 && p < 0.6);
}

#[test]
fn test_visual_properties_at_open() {
    let mut renderer = VisualFlowerRenderer::new();
    renderer.update(3.5);
    assert!((renderer.opening_percent() - 1.0).abs() < f32::EPSILON);
    assert_eq!(renderer.current_state(), FlowerState::Open);
}

#[test]
fn test_tick_sequence_at_various_timestamps() {
    let mut renderer = VisualFlowerRenderer::new();
    let deltas = [0.15, 0.35, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 1.0];
    for &dt in &deltas {
        renderer.update(dt);
        let state = renderer.current_state();
        assert!(
            matches!(state, FlowerState::Closed)
                || matches!(state, FlowerState::Opening(_))
                || matches!(state, FlowerState::Open)
        );
    }
    assert_eq!(renderer.current_state(), FlowerState::Open);
}

#[test]
fn test_exact_progress_0_1_boundary() {
    let mut renderer = VisualFlowerRenderer::new();
    renderer.update(0.3);
    assert!(matches!(renderer.current_state(), FlowerState::Opening(_)));
}

#[test]
fn test_exact_progress_0_9_boundary() {
    let mut renderer = VisualFlowerRenderer::new();
    renderer.update(2.7);
    assert_eq!(renderer.current_state(), FlowerState::Open);
}

#[test]
fn test_opening_percent_at_boundary_closed() {
    let mut renderer = VisualFlowerRenderer::new();
    renderer.update(0.29);
    assert!((renderer.opening_percent() - 0.0).abs() < f32::EPSILON);
}

#[test]
fn test_opening_percent_at_boundary_open() {
    let mut renderer = VisualFlowerRenderer::new();
    renderer.update(2.71);
    assert!((renderer.opening_percent() - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_multiple_flowers_independent_state() {
    let mut r1 = VisualFlowerRenderer::new();
    let mut r2 = VisualFlowerRenderer::new();
    r1.update(0.5);
    r2.update(3.0);
    assert!(matches!(r1.current_state(), FlowerState::Opening(_)));
    assert_eq!(r2.current_state(), FlowerState::Open);
}

#[test]
fn test_base_hue_constant() {
    let renderer = VisualFlowerRenderer::new();
    assert!((renderer.base_hue - 330.0).abs() < f32::EPSILON);
}

#[test]
fn test_opening_percent_at_exact_0_1_boundary() {
    let mut renderer = VisualFlowerRenderer::new();
    renderer.update(0.3);
    let state = renderer.current_state();
    assert!(matches!(state, FlowerState::Opening(_)));
    let p = renderer.opening_percent();
    assert!((0.0..=1.0).contains(&p));
}

#[test]
fn test_opening_percent_at_exact_0_9_boundary() {
    let mut renderer = VisualFlowerRenderer::new();
    renderer.update(2.7);
    assert_eq!(renderer.current_state(), FlowerState::Open);
    assert!((renderer.opening_percent() - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_opening_percent_monotonic_over_time() {
    let mut renderer = VisualFlowerRenderer::new();
    let mut prev_pct = renderer.opening_percent();
    for _ in 0..10 {
        renderer.update(0.3);
        let pct = renderer.opening_percent();
        assert!(pct >= prev_pct - 0.01);
        prev_pct = pct;
    }
}

#[cfg(feature = "egui")]
#[test]
fn test_egui_render_headless_closed() {
    use egui::Pos2;
    let renderer = VisualFlowerRenderer::new();
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            renderer.render(ui, Pos2::new(100.0, 100.0), 50.0);
        });
    });
}

#[cfg(feature = "egui")]
#[test]
fn test_egui_render_headless_opening() {
    use egui::Pos2;
    let mut renderer = VisualFlowerRenderer::new();
    renderer.update(1.5);
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            renderer.render(ui, Pos2::new(100.0, 100.0), 50.0);
        });
    });
}

#[cfg(feature = "egui")]
#[test]
fn test_egui_render_headless_open_with_glow() {
    use egui::Pos2;
    let mut renderer = VisualFlowerRenderer::new();
    renderer.update(3.5);
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            renderer.render(ui, Pos2::new(100.0, 100.0), 50.0);
        });
    });
}
