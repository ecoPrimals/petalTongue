// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Animation primitive tests — easing, construction, state machine, lerp, sequences.
//!
//! Player lifecycle, scene graph interaction, and serialization tests
//! live in `animation_playback_tests.rs`.

use petal_tongue_scene::animation::AnimationTarget;
use petal_tongue_scene::animation::{Animation, AnimationState, Easing, Sequence};

const EPS: f64 = 1e-10;

// ── Easing function boundary tests ──────────────────────────────────────────

#[test]
fn easing_linear_at_0_0_5_1_0() {
    assert!((Easing::Linear.apply(0.0) - 0.0).abs() < EPS);
    assert!((Easing::Linear.apply(0.5) - 0.5).abs() < EPS);
    assert!((Easing::Linear.apply(1.0) - 1.0).abs() < EPS);
}

#[test]
fn easing_ease_in_at_0_and_1_0() {
    assert!((Easing::EaseIn.apply(0.0) - 0.0).abs() < EPS);
    assert!((Easing::EaseIn.apply(1.0) - 1.0).abs() < EPS);
}

#[test]
fn easing_ease_out_at_0_and_1_0() {
    assert!((Easing::EaseOut.apply(0.0) - 0.0).abs() < EPS);
    assert!((Easing::EaseOut.apply(1.0) - 1.0).abs() < EPS);
}

#[test]
fn easing_ease_in_out_symmetry() {
    assert!((Easing::EaseInOut.apply(0.5) - 0.5).abs() < EPS);
}

#[test]
fn easing_ease_in_out_at_0_and_1() {
    assert!((Easing::EaseInOut.apply(0.0) - 0.0).abs() < EPS);
    assert!((Easing::EaseInOut.apply(1.0) - 1.0).abs() < EPS);
}

#[test]
fn easing_spring_at_0_0_5_1_0() {
    assert!((Easing::Spring.apply(0.0) - 0.0).abs() < EPS);
    assert!((Easing::Spring.apply(1.0) - 1.0).abs() < 0.1);
    let mid = Easing::Spring.apply(0.5);
    assert!(mid > 0.0 && mid < 1.0);
}

#[test]
fn easing_bounce_at_0_0_5_1_0() {
    assert!((Easing::Bounce.apply(0.0) - 0.0).abs() < EPS);
    assert!((Easing::Bounce.apply(1.0) - 1.0).abs() < EPS);
    let mid = Easing::Bounce.apply(0.5);
    assert!(mid > 0.0 && mid <= 1.0);
}

#[test]
fn easing_clamps_negative_t() {
    assert!((Easing::Linear.apply(-1.0) - 0.0).abs() < EPS);
    assert!((Easing::EaseIn.apply(-0.5) - 0.0).abs() < EPS);
}

#[test]
fn easing_clamps_t_above_one() {
    assert!((Easing::Linear.apply(2.0) - 1.0).abs() < EPS);
    assert!((Easing::Bounce.apply(1.5) - 1.0).abs() < EPS);
}

// ── Easing branch coverage ─────────────────────────────────────────────────

#[test]
fn easing_ease_in_out_branch_below_half() {
    let v = Easing::EaseInOut.apply(0.25);
    assert!(v < 0.5, "t=0.25 should use first branch (4*t^3)");
    assert!((v - 0.0625).abs() < EPS, "4*0.25^3 = 0.0625");
}

#[test]
fn easing_ease_in_out_branch_above_half() {
    let v = Easing::EaseInOut.apply(0.75);
    assert!(v > 0.5, "t=0.75 should use second branch");
    assert!((v - 0.9375).abs() < EPS, "1 - (-2*0.75+2)^3/2 = 0.9375");
}

#[test]
fn easing_bounce_first_branch() {
    let t = 0.03;
    let v = Easing::Bounce.apply(t);
    assert!(
        (7.5625 * t).mul_add(-t, v).abs() < EPS,
        "first branch: 7.5625*t^2"
    );
}

#[test]
fn easing_bounce_second_branch() {
    let t = 0.5;
    let v = Easing::Bounce.apply(t);
    assert!(
        v > 0.75 && v <= 1.0,
        "second branch should produce value in (0.75, 1]"
    );
}

#[test]
fn easing_bounce_third_branch() {
    let t = 0.8;
    let v = Easing::Bounce.apply(t);
    assert!(
        v > 0.9 && v <= 1.0,
        "third branch should produce value near 1"
    );
}

#[test]
fn easing_bounce_fourth_branch() {
    let t = 0.99;
    let v = Easing::Bounce.apply(t);
    assert!(
        (v - 1.0).abs() < 0.1,
        "fourth branch at t~1 should be near 1"
    );
}

// ── Easing monotonicity ────────────────────────────────────────────────────

#[test]
fn animation_easing_monotonic_linear() {
    let mut prev = -1.0;
    for i in 0..=10 {
        let t = f64::from(i) / 10.0;
        let v = Easing::Linear.apply(t);
        assert!(v >= prev, "Linear at t={t} should be monotonic");
        prev = v;
    }
}

#[test]
fn animation_easing_monotonic_ease_in_out() {
    let mut prev = -1.0;
    for i in 0..=10 {
        let t = f64::from(i) / 10.0;
        let v = Easing::EaseInOut.apply(t);
        assert!(v >= prev, "EaseInOut at t={t} should be monotonic");
        prev = v;
    }
}

// ── Easing midpoints ──────────────────────────────────────────────────────

#[test]
fn animation_ease_in_midpoint() {
    let v = Easing::EaseIn.apply(0.5);
    assert!((v - 0.125).abs() < EPS, "EaseIn: t^3 at 0.5 = 0.125");
}

#[test]
fn animation_ease_out_midpoint() {
    let v = Easing::EaseOut.apply(0.5);
    assert!((v - 0.875).abs() < EPS, "EaseOut: 1-(1-t)^3 at 0.5 = 0.875");
}

#[test]
fn animation_easing_default() {
    assert_eq!(Easing::default(), Easing::EaseInOut);
}

// ── Animation construction ─────────────────────────────────────────────────

#[test]
fn animation_fade_in_total_duration() {
    let anim = Animation::fade_in("node1", 2.0);
    assert!((anim.total_duration() - 2.0).abs() < EPS);
}

#[test]
fn animation_with_delay_total_duration() {
    let anim = Animation::fade_in("node1", 2.0).with_delay(1.0);
    assert!((anim.total_duration() - 3.0).abs() < EPS);
}

#[test]
fn animation_fade_out_and_create() {
    let fade_out = Animation::fade_out("n", 1.0);
    assert_eq!(
        fade_out.target,
        AnimationTarget::Opacity {
            node_id: "n".to_string(),
            from: 1.0,
            to: 0.0,
        }
    );

    let create = Animation::create("n", 2.0);
    assert!(matches!(create.target, AnimationTarget::StrokeDraw { .. }));
    assert!((create.total_duration() - 2.0).abs() < EPS);
}

#[test]
fn animation_with_easing_and_delay() {
    let anim = Animation::fade_in("n", 1.0)
        .with_easing(Easing::EaseIn)
        .with_delay(0.5);
    assert!((anim.total_duration() - 1.5).abs() < EPS);
}

#[test]
fn animation_move_to_construction() {
    let anim = Animation::move_to("n", [0.0, 0.0], [10.0, 20.0], 1.0);
    assert!(matches!(anim.target, AnimationTarget::Translate { .. }));
    assert!((anim.duration_secs - 1.0).abs() < EPS);
}

#[test]
fn animation_scale_target_direct_construction() {
    let anim = Animation {
        target: AnimationTarget::Scale {
            node_id: "n".to_string(),
            from: 1.0,
            to: 2.0,
        },
        duration_secs: 1.0,
        easing: Easing::Linear,
        delay_secs: 0.0,
    };
    assert!((anim.total_duration() - 1.0).abs() < EPS);
}

// ── AnimationState ─────────────────────────────────────────────────────────

#[test]
fn animation_state_progress_before_during_after() {
    let anim = Animation::fade_in("n", 1.0);
    let mut state = AnimationState::new(anim);
    assert!((state.progress() - 0.0).abs() < EPS);
    state.advance(0.5);
    assert!(state.progress() > 0.0 && state.progress() < 1.0);
    state.advance(1.0);
    assert!((state.progress() - 1.0).abs() < EPS);
}

#[test]
fn animation_state_progress_with_delay() {
    let anim = Animation::fade_in("n", 1.0).with_delay(0.5);
    let mut state = AnimationState::new(anim);
    state.advance(0.25);
    assert!((state.progress() - 0.0).abs() < EPS);
    state.advance(0.5);
    assert!(state.progress() > 0.0 && state.progress() < 1.0);
}

#[test]
fn animation_state_is_done() {
    let anim = Animation::fade_in("n", 1.0);
    let mut state = AnimationState::new(anim);
    assert!(!state.is_done());
    state.advance(0.5);
    assert!(!state.is_done());
    state.advance(1.0);
    assert!(state.is_done());
}

#[test]
fn animation_state_zero_duration_immediate_complete() {
    let anim = Animation::fade_in("n", 0.0);
    let state = AnimationState::new(anim);
    assert!(state.is_done());
    assert!((state.progress() - 1.0).abs() < EPS);
}

#[test]
fn animation_state_negative_t_from_advance_beyond_duration() {
    let anim = Animation::fade_in("n", 1.0);
    let mut state = AnimationState::new(anim);
    state.advance(2.0);
    assert!(state.is_done());
    assert!((state.progress() - 1.0).abs() < EPS);
}

#[test]
fn animation_state_lerp_f32_complete() {
    let anim = Animation::fade_in("n", 1.0);
    let mut state = AnimationState::new(anim);
    state.advance(1.0);
    assert!((state.lerp_f32(0.0, 1.0) - 1.0).abs() < EPS);
}

#[test]
fn animation_state_clone() {
    let anim = Animation::fade_in("n", 1.0);
    let state = AnimationState::new(anim);
    let cloned = state.clone();
    assert_eq!(state.elapsed, cloned.elapsed);
    assert_eq!(
        state.animation.duration_secs,
        cloned.animation.duration_secs
    );
}

// ── Lerp tests ─────────────────────────────────────────────────────────────

#[test]
fn animation_lerp_f64() {
    let anim = Animation::fade_in("n", 1.0);
    let mut state = AnimationState::new(anim);
    assert!((state.lerp_f64(0.0, 100.0) - 0.0).abs() < EPS);
    state.advance(0.5);
    let mid = state.lerp_f64(0.0, 100.0);
    assert!(mid > 0.0 && mid < 100.0);
    state.advance(1.0);
    assert!((state.lerp_f64(0.0, 100.0) - 100.0).abs() < 1.0);
}

#[test]
fn animation_lerp_f32() {
    let anim = Animation::fade_in("n", 1.0);
    let mut state = AnimationState::new(anim);
    assert!((state.lerp_f32(0.0, 1.0) - 0.0).abs() < EPS);
    state.advance(0.5);
    let mid = state.lerp_f32(0.0, 1.0);
    assert!(mid > 0.0 && mid < 1.0);
}

#[test]
fn animation_lerp_with_negative_values() {
    let anim = Animation::fade_in("n", 1.0);
    let mut state = AnimationState::new(anim);
    assert!((state.lerp_f64(-1.0, 1.0) - (-1.0)).abs() < EPS);
    state.advance(0.5);
    let mid = state.lerp_f64(-1.0, 1.0);
    assert!(mid > -1.0 && mid < 1.0);
    state.advance(1.0);
    assert!((state.lerp_f64(-1.0, 1.0) - 1.0).abs() < EPS);
}

// ── Sequence duration tests ────────────────────────────────────────────────

#[test]
fn sequence_sequential_total_duration() {
    let a1 = Animation::fade_in("n1", 2.0);
    let a2 = Animation::fade_in("n2", 3.0);
    let seq = Sequence::Sequential(vec![a1, a2]);
    assert!((seq.total_duration() - 5.0).abs() < EPS);
}

#[test]
fn sequence_parallel_total_duration_max() {
    let a1 = Animation::fade_in("n1", 2.0);
    let a2 = Animation::fade_in("n2", 5.0);
    let a3 = Animation::fade_in("n3", 3.0);
    let seq = Sequence::Parallel(vec![a1, a2, a3]);
    assert!((seq.total_duration() - 5.0).abs() < EPS);
}

#[test]
fn sequence_group_total_duration() {
    let a1 = Animation::fade_in("n1", 2.0);
    let a2 = Animation::fade_in("n2", 3.0);
    let inner = Sequence::Sequential(vec![a1, a2]);
    let outer = Sequence::Group(vec![inner.clone(), inner]);
    assert!((outer.total_duration() - 10.0).abs() < EPS);
}

#[test]
fn sequence_empty_sequential_total_duration_zero() {
    let seq = Sequence::Sequential(vec![]);
    assert!(
        (seq.total_duration() - 0.0).abs() < EPS,
        "empty sequential should have 0 duration"
    );
}

#[test]
fn sequence_empty_parallel_total_duration_zero() {
    let seq = Sequence::Parallel(vec![]);
    assert!(
        (seq.total_duration() - 0.0).abs() < EPS,
        "empty parallel should have 0 duration"
    );
}

#[test]
fn sequence_empty_group_total_duration_zero() {
    let seq = Sequence::Group(vec![]);
    assert!(
        (seq.total_duration() - 0.0).abs() < EPS,
        "empty group should have 0 duration"
    );
}
