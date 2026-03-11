// SPDX-License-Identifier: AGPL-3.0-only
//! Generalized animation system for scene properties.
//!
//! Animations interpolate between scene states using easing functions.
//! This generalizes the existing `AnimationEngine` (flow particles, pulses)
//! to animate any scene property.

use serde::{Deserialize, Serialize};

/// Easing function for animation interpolation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    #[default]
    EaseInOut,
    Spring,
    Bounce,
}

impl Easing {
    /// Evaluate the easing function at time t (0.0 to 1.0).
    #[must_use]
    pub fn apply(self, t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,
            Self::EaseIn => t * t * t,
            Self::EaseOut => 1.0 - (1.0 - t).powi(3),
            Self::EaseInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0f64).mul_add(t, 2.0).powi(3) / 2.0
                }
            }
            Self::Spring => {
                let freq = 4.5 * std::f64::consts::PI;
                (-6.0 * t).exp().mul_add(-(freq * t).cos(), 1.0)
            }
            Self::Bounce => {
                if t < 1.0 / 2.75 {
                    7.5625 * t * t
                } else if t < 2.0 / 2.75 {
                    let t2 = t - 1.5 / 2.75;
                    (7.5625 * t2).mul_add(t2, 0.75)
                } else if t < 2.5 / 2.75 {
                    let t2 = t - 2.25 / 2.75;
                    (7.5625 * t2).mul_add(t2, 0.9375)
                } else {
                    let t2 = t - 2.625 / 2.75;
                    (7.5625 * t2).mul_add(t2, 0.984_375)
                }
            }
        }
    }
}

/// The target property being animated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnimationTarget {
    /// Animate opacity (0.0 to 1.0).
    Opacity { node_id: String, from: f32, to: f32 },
    /// Animate translation.
    Translate {
        node_id: String,
        from: [f64; 2],
        to: [f64; 2],
    },
    /// Animate uniform scale.
    Scale { node_id: String, from: f64, to: f64 },
    /// Animate rotation (radians).
    Rotation { node_id: String, from: f64, to: f64 },
    /// Progressive stroke drawing (0.0 = invisible, 1.0 = fully drawn).
    StrokeDraw { node_id: String },
}

/// A single animation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Animation {
    pub target: AnimationTarget,
    pub duration_secs: f64,
    pub easing: Easing,
    pub delay_secs: f64,
}

impl Animation {
    /// Create a fade-in animation for a node.
    pub fn fade_in(node_id: impl Into<String>, duration: f64) -> Self {
        Self {
            target: AnimationTarget::Opacity {
                node_id: node_id.into(),
                from: 0.0,
                to: 1.0,
            },
            duration_secs: duration,
            easing: Easing::EaseInOut,
            delay_secs: 0.0,
        }
    }

    /// Create a fade-out animation.
    pub fn fade_out(node_id: impl Into<String>, duration: f64) -> Self {
        Self {
            target: AnimationTarget::Opacity {
                node_id: node_id.into(),
                from: 1.0,
                to: 0.0,
            },
            duration_secs: duration,
            easing: Easing::EaseInOut,
            delay_secs: 0.0,
        }
    }

    /// Create a move animation.
    pub fn move_to(
        node_id: impl Into<String>,
        from: [f64; 2],
        to: [f64; 2],
        duration: f64,
    ) -> Self {
        Self {
            target: AnimationTarget::Translate {
                node_id: node_id.into(),
                from,
                to,
            },
            duration_secs: duration,
            easing: Easing::EaseInOut,
            delay_secs: 0.0,
        }
    }

    /// Create a stroke-draw (Manim "Create") animation.
    pub fn create(node_id: impl Into<String>, duration: f64) -> Self {
        Self {
            target: AnimationTarget::StrokeDraw {
                node_id: node_id.into(),
            },
            duration_secs: duration,
            easing: Easing::EaseInOut,
            delay_secs: 0.0,
        }
    }

    /// Builder: set easing.
    #[must_use]
    pub const fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Builder: set delay.
    #[must_use]
    pub const fn with_delay(mut self, delay: f64) -> Self {
        self.delay_secs = delay;
        self
    }

    /// Total duration including delay.
    #[must_use]
    pub fn total_duration(&self) -> f64 {
        self.delay_secs + self.duration_secs
    }
}

/// A sequence of animations (played one after another or in parallel).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Sequence {
    /// Play animations one after another.
    Sequential(Vec<Animation>),
    /// Play animations simultaneously.
    Parallel(Vec<Animation>),
    /// Nested sequences.
    Group(Vec<Self>),
}

impl Sequence {
    /// Total duration of this sequence.
    pub fn total_duration(&self) -> f64 {
        match self {
            Self::Sequential(anims) => anims.iter().map(Animation::total_duration).sum(),
            Self::Parallel(anims) => anims
                .iter()
                .map(Animation::total_duration)
                .fold(0.0_f64, f64::max),
            Self::Group(seqs) => seqs.iter().map(Self::total_duration).sum(),
        }
    }
}

/// Tracks the state of an active animation.
#[derive(Debug, Clone)]
pub struct AnimationState {
    pub animation: Animation,
    pub elapsed: f64,
}

impl AnimationState {
    /// Create a new animation state.
    #[must_use]
    pub const fn new(animation: Animation) -> Self {
        Self {
            animation,
            elapsed: 0.0,
        }
    }

    /// Advance time by dt seconds.
    pub fn advance(&mut self, dt: f64) {
        self.elapsed += dt;
    }

    /// Whether this animation has completed.
    #[must_use]
    pub fn is_done(&self) -> bool {
        self.elapsed >= self.animation.total_duration()
    }

    /// Current progress ratio (0.0 to 1.0), accounting for delay and easing.
    #[must_use]
    pub fn progress(&self) -> f64 {
        let active_time = (self.elapsed - self.animation.delay_secs).max(0.0);
        let raw = if self.animation.duration_secs > 0.0 {
            (active_time / self.animation.duration_secs).min(1.0)
        } else {
            1.0
        };
        self.animation.easing.apply(raw)
    }

    /// Interpolate a f64 value between `from` and `to` at current progress.
    #[must_use]
    pub fn lerp_f64(&self, from: f64, to: f64) -> f64 {
        let t = self.progress();
        (to - from).mul_add(t, from)
    }

    /// Interpolate a f32 value.
    #[must_use]
    pub fn lerp_f32(&self, from: f32, to: f32) -> f64 {
        let t = self.progress();
        f64::from(to - from).mul_add(t, f64::from(from))
    }
}

/// Manages active animations and applies them to a scene graph each frame.
///
/// The player holds a queue of animations. Each frame, `tick()` is called with
/// delta time, advancing all animations and applying interpolated values to
/// the scene graph nodes they target.
#[derive(Debug, Clone, Default)]
pub struct AnimationPlayer {
    active: Vec<AnimationState>,
}

impl AnimationPlayer {
    /// Create a new empty player.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Queue a single animation.
    pub fn play(&mut self, animation: Animation) {
        self.active.push(AnimationState::new(animation));
    }

    /// Queue all animations from a sequence (flattened).
    pub fn play_sequence(&mut self, sequence: Sequence) {
        match sequence {
            Sequence::Sequential(anims) | Sequence::Parallel(anims) => {
                for a in anims {
                    self.active.push(AnimationState::new(a));
                }
            }
            Sequence::Group(seqs) => {
                for s in seqs {
                    self.play_sequence(s);
                }
            }
        }
    }

    /// Advance all animations by `dt` seconds and apply to the scene graph.
    /// Returns the number of animations still active.
    pub fn tick(&mut self, dt: f64, scene: &mut crate::scene_graph::SceneGraph) -> usize {
        for state in &mut self.active {
            state.advance(dt);
            apply_animation_to_scene(state, scene);
        }
        self.active.retain(|s| !s.is_done());
        self.active.len()
    }

    /// Number of currently active animations.
    #[must_use]
    pub const fn active_count(&self) -> usize {
        self.active.len()
    }

    /// Whether any animations are currently playing.
    #[must_use]
    pub const fn is_playing(&self) -> bool {
        !self.active.is_empty()
    }
}

/// Apply an animation state's current progress to the targeted scene graph node.
fn apply_animation_to_scene(state: &AnimationState, scene: &mut crate::scene_graph::SceneGraph) {
    match &state.animation.target {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "opacity is in [0,1], f32 sufficient"
        )]
        AnimationTarget::Opacity { node_id, from, to } => {
            if let Some(node) = scene.get_mut(node_id) {
                node.opacity = state.lerp_f32(*from, *to) as f32;
            }
        }
        AnimationTarget::Translate { node_id, from, to } => {
            if let Some(node) = scene.get_mut(node_id) {
                let x = state.lerp_f64(from[0], to[0]);
                let y = state.lerp_f64(from[1], to[1]);
                node.transform = crate::transform::Transform2D::translate(x, y);
            }
        }
        AnimationTarget::Scale { node_id, from, to } => {
            if let Some(node) = scene.get_mut(node_id) {
                let s = state.lerp_f64(*from, *to);
                node.transform = crate::transform::Transform2D::scale(s, s);
            }
        }
        AnimationTarget::Rotation { node_id, from, to } => {
            if let Some(node) = scene.get_mut(node_id) {
                let r = state.lerp_f64(*from, *to);
                node.transform = crate::transform::Transform2D::rotate(r);
            }
        }
        #[expect(
            clippy::cast_possible_truncation,
            reason = "progress is in [0,1], f32 sufficient"
        )]
        AnimationTarget::StrokeDraw { node_id } => {
            if let Some(node) = scene.get_mut(node_id) {
                let progress = state.progress() as f32;
                node.opacity = progress;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-10;

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
    fn animation_state_progress_before_during_after() {
        let anim = Animation::fade_in("n", 1.0);
        let mut state = AnimationState::new(anim);
        // Before start (no delay)
        assert!((state.progress() - 0.0).abs() < EPS);
        // During
        state.advance(0.5);
        assert!(state.progress() > 0.0 && state.progress() < 1.0);
        // After
        state.advance(1.0);
        assert!((state.progress() - 1.0).abs() < EPS);
    }

    #[test]
    fn animation_state_progress_with_delay() {
        let anim = Animation::fade_in("n", 1.0).with_delay(0.5);
        let mut state = AnimationState::new(anim);
        // During delay
        state.advance(0.25);
        assert!((state.progress() - 0.0).abs() < EPS);
        // Mid-animation
        state.advance(0.5); // total 0.75, active 0.25
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
    fn animation_player_plays_and_completes() {
        use crate::scene_graph::{SceneGraph, SceneNode};
        let mut scene = SceneGraph::new();
        scene.add_to_root(SceneNode::new("n1").with_opacity(0.0));

        let mut player = AnimationPlayer::new();
        assert!(!player.is_playing());

        player.play(Animation::fade_in("n1", 1.0));
        assert!(player.is_playing());
        assert_eq!(player.active_count(), 1);

        // Half-way
        let remaining = player.tick(0.5, &mut scene);
        assert_eq!(remaining, 1);
        let opacity = scene.get("n1").expect("node n1 should exist").opacity;
        assert!(
            opacity > 0.0 && opacity < 1.0,
            "half-way opacity should be between 0 and 1"
        );

        // Complete
        let remaining = player.tick(1.0, &mut scene);
        assert_eq!(remaining, 0);
        assert!(!player.is_playing());
    }

    #[test]
    fn animation_player_tick_translates_node() {
        use crate::scene_graph::{SceneGraph, SceneNode};
        let mut scene = SceneGraph::new();
        scene.add_to_root(SceneNode::new("n1"));

        let mut player = AnimationPlayer::new();
        player.play(Animation::move_to("n1", [0.0, 0.0], [100.0, 200.0], 1.0));
        player.tick(1.0, &mut scene);
        let node = scene.get("n1").expect("node n1 should exist");
        let (x, y) = node.transform.apply(0.0, 0.0);
        assert!((x - 100.0).abs() < 1.0);
        assert!((y - 200.0).abs() < 1.0);
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
    fn easing_ease_in_out_at_0_and_1() {
        assert!((Easing::EaseInOut.apply(0.0) - 0.0).abs() < EPS);
        assert!((Easing::EaseInOut.apply(1.0) - 1.0).abs() < EPS);
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

    #[test]
    fn sequence_group_total_duration() {
        let a1 = Animation::fade_in("n1", 2.0);
        let a2 = Animation::fade_in("n2", 3.0);
        let inner = Sequence::Sequential(vec![a1, a2]);
        let outer = Sequence::Group(vec![inner.clone(), inner]);
        assert!((outer.total_duration() - 10.0).abs() < EPS);
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
    fn animation_player_sequence_group_in_tick_path() {
        use crate::scene_graph::{SceneGraph, SceneNode};
        let mut scene = SceneGraph::new();
        scene.add_to_root(SceneNode::new("n1").with_opacity(0.0));
        scene.add_to_root(SceneNode::new("n2").with_opacity(0.0));

        let a1 = Animation::fade_in("n1", 0.5);
        let a2 = Animation::fade_in("n2", 0.5);
        let inner = Sequence::Sequential(vec![a1, a2]);
        let group = Sequence::Group(vec![inner.clone(), inner]);

        let mut player = AnimationPlayer::new();
        player.play_sequence(group);
        assert_eq!(player.active_count(), 4);

        // Tick through to completion (4 * 0.5 = 2.0 total)
        let remaining = player.tick(2.0, &mut scene);
        assert_eq!(remaining, 0);
        assert!(!player.is_playing());
        assert!(
            (scene.get("n1").expect("n1 exists").opacity - 1.0).abs() < 0.01,
            "n1 should be fully opaque"
        );
        assert!(
            (scene.get("n2").expect("n2 exists").opacity - 1.0).abs() < 0.01,
            "n2 should be fully opaque"
        );
    }

    #[test]
    fn animation_lerp_with_negative_values() {
        let anim = Animation::fade_in("n", 1.0);
        let mut state = AnimationState::new(anim);

        // Lerp from -1 to 1
        assert!((state.lerp_f64(-1.0, 1.0) - (-1.0)).abs() < EPS);
        state.advance(0.5);
        let mid = state.lerp_f64(-1.0, 1.0);
        assert!(mid > -1.0 && mid < 1.0);
        state.advance(1.0);
        assert!((state.lerp_f64(-1.0, 1.0) - 1.0).abs() < EPS);
    }

    #[test]
    fn animation_scale_with_negative_from() {
        use crate::scene_graph::{SceneGraph, SceneNode};
        let mut scene = SceneGraph::new();
        scene.add_to_root(SceneNode::new("n1"));

        let anim = Animation {
            target: AnimationTarget::Scale {
                node_id: "n1".to_string(),
                from: -1.0,
                to: 1.0,
            },
            duration_secs: 1.0,
            easing: Easing::Linear,
            delay_secs: 0.0,
        };

        let mut player = AnimationPlayer::new();
        player.play(anim);
        player.tick(1.0, &mut scene);

        let node = scene.get("n1").expect("node n1 should exist");
        let (x, _) = node.transform.apply(1.0, 0.0);
        assert!(
            (x - 1.0).abs() < 0.01,
            "Scale should interpolate from -1 to 1"
        );
    }

    #[test]
    fn animation_rotation_interpolation() {
        use crate::scene_graph::{SceneGraph, SceneNode};
        let mut scene = SceneGraph::new();
        scene.add_to_root(SceneNode::new("n1"));

        let anim = Animation {
            target: AnimationTarget::Rotation {
                node_id: "n1".to_string(),
                from: 0.0,
                to: std::f64::consts::PI,
            },
            duration_secs: 1.0,
            easing: Easing::Linear,
            delay_secs: 0.0,
        };

        let mut player = AnimationPlayer::new();
        player.play(anim);
        player.tick(1.0, &mut scene);
        let node = scene.get("n1").expect("node n1 should exist");
        let (x, _y) = node.transform.apply(1.0, 0.0);
        assert!((x - (-1.0)).abs() < 0.1, "PI rotation should flip x");
    }

    #[test]
    fn animation_player_parallel_animations() {
        use crate::scene_graph::{SceneGraph, SceneNode};
        let mut scene = SceneGraph::new();
        scene.add_to_root(SceneNode::new("n1").with_opacity(0.0));
        scene.add_to_root(SceneNode::new("n2").with_opacity(0.0));

        let seq = Sequence::Parallel(vec![
            Animation::fade_in("n1", 1.0),
            Animation::fade_in("n2", 1.0),
        ]);
        let mut player = AnimationPlayer::new();
        player.play_sequence(seq);
        assert_eq!(player.active_count(), 2);
        player.tick(1.0, &mut scene);
        assert_eq!(player.active_count(), 0);
        assert!(
            (scene.get("n1").expect("n1 exists").opacity - 1.0).abs() < 0.01,
            "n1 should be fully opaque"
        );
        assert!(
            (scene.get("n2").expect("n2 exists").opacity - 1.0).abs() < 0.01,
            "n2 should be fully opaque"
        );
    }

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

    #[test]
    fn animation_stroke_draw_progress() {
        use crate::scene_graph::{SceneGraph, SceneNode};
        let mut scene = SceneGraph::new();
        scene.add_to_root(SceneNode::new("n1").with_opacity(0.0));

        let anim = Animation::create("n1", 1.0);
        let mut player = AnimationPlayer::new();
        player.play(anim);
        player.tick(0.5, &mut scene);
        let opacity = scene.get("n1").expect("n1 exists").opacity;
        assert!(
            opacity > 0.0 && opacity < 1.0,
            "mid-animation opacity should be between 0 and 1"
        );
        player.tick(1.0, &mut scene);
        assert!(
            (scene.get("n1").expect("n1 exists").opacity - 1.0).abs() < 0.01,
            "stroke draw should complete to full opacity"
        );
    }

    #[test]
    fn animation_target_missing_node_no_panic() {
        use crate::scene_graph::{SceneGraph, SceneNode};
        let mut scene = SceneGraph::new();
        scene.add_to_root(SceneNode::new("n1"));

        let anim = Animation::fade_in("nonexistent", 1.0);
        let mut player = AnimationPlayer::new();
        player.play(anim);
        let remaining = player.tick(1.0, &mut scene);
        assert_eq!(remaining, 0);
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
    fn animation_easing_default() {
        assert_eq!(Easing::default(), Easing::EaseInOut);
    }

    #[test]
    fn animation_player_empty_tick() {
        use crate::scene_graph::SceneGraph;
        let mut scene = SceneGraph::new();
        let mut player = AnimationPlayer::new();
        let remaining = player.tick(1.0, &mut scene);
        assert_eq!(remaining, 0);
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
        let t = 0.03; // < 1/2.75 ≈ 0.3636
        let v = Easing::Bounce.apply(t);
        assert!(
            (7.5625 * t).mul_add(-t, v).abs() < EPS,
            "first branch: 7.5625*t^2"
        );
    }

    #[test]
    fn easing_bounce_second_branch() {
        let t = 0.5; // between 1/2.75 and 2/2.75
        let v = Easing::Bounce.apply(t);
        assert!(
            v > 0.75 && v <= 1.0,
            "second branch should produce value in (0.75, 1]"
        );
    }

    #[test]
    fn easing_bounce_third_branch() {
        let t = 0.8; // between 2/2.75 and 2.5/2.75
        let v = Easing::Bounce.apply(t);
        assert!(
            v > 0.9 && v <= 1.0,
            "third branch should produce value near 1"
        );
    }

    #[test]
    fn easing_bounce_fourth_branch() {
        let t = 0.99; // > 2.5/2.75
        let v = Easing::Bounce.apply(t);
        assert!(
            (v - 1.0).abs() < 0.1,
            "fourth branch at t≈1 should be near 1"
        );
    }

    #[test]
    fn animation_serialization_roundtrip() {
        let anim = Animation::fade_in("node1", 2.5)
            .with_easing(Easing::Spring)
            .with_delay(0.5);
        let json = serde_json::to_string(&anim).expect("serialization should succeed");
        let decoded: Animation =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(anim.target, decoded.target);
        assert!((anim.duration_secs - decoded.duration_secs).abs() < EPS);
        assert_eq!(anim.easing, decoded.easing);
        assert!((anim.delay_secs - decoded.delay_secs).abs() < EPS);
    }

    #[test]
    fn animation_target_serialization_all_variants() {
        let targets = [
            AnimationTarget::Opacity {
                node_id: "n".to_string(),
                from: 0.0,
                to: 1.0,
            },
            AnimationTarget::Translate {
                node_id: "n".to_string(),
                from: [0.0, 0.0],
                to: [10.0, 20.0],
            },
            AnimationTarget::Scale {
                node_id: "n".to_string(),
                from: 1.0,
                to: 2.0,
            },
            AnimationTarget::Rotation {
                node_id: "n".to_string(),
                from: 0.0,
                to: std::f64::consts::PI,
            },
            AnimationTarget::StrokeDraw {
                node_id: "n".to_string(),
            },
        ];
        for target in targets {
            let json = serde_json::to_string(&target).expect("serialization should succeed");
            let decoded: AnimationTarget =
                serde_json::from_str(&json).expect("deserialization should succeed");
            assert_eq!(target, decoded, "roundtrip for {target:?}");
        }
    }

    #[test]
    fn easing_serialization_roundtrip() {
        for easing in [
            Easing::Linear,
            Easing::EaseIn,
            Easing::EaseOut,
            Easing::EaseInOut,
            Easing::Spring,
            Easing::Bounce,
        ] {
            let json = serde_json::to_string(&easing).expect("serialization should succeed");
            let decoded: Easing =
                serde_json::from_str(&json).expect("deserialization should succeed");
            assert_eq!(easing, decoded);
        }
    }

    #[test]
    fn animation_player_play_sequence_sequential() {
        use crate::scene_graph::{SceneGraph, SceneNode};
        let mut scene = SceneGraph::new();
        scene.add_to_root(SceneNode::new("n1"));
        scene.add_to_root(SceneNode::new("n2"));

        let seq = Sequence::Sequential(vec![
            Animation::fade_in("n1", 0.5),
            Animation::fade_in("n2", 0.5),
        ]);
        let mut player = AnimationPlayer::new();
        player.play_sequence(seq);
        assert_eq!(player.active_count(), 2);
    }

    #[test]
    fn animation_state_lerp_f32_complete() {
        let anim = Animation::fade_in("n", 1.0);
        let mut state = AnimationState::new(anim);
        state.advance(1.0);
        assert!((state.lerp_f32(0.0, 1.0) - 1.0).abs() < EPS);
    }

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
    fn sequence_serialization_roundtrip() {
        let seq = Sequence::Sequential(vec![
            Animation::fade_in("n1", 1.0),
            Animation::fade_out("n2", 2.0),
        ]);
        let json = serde_json::to_string(&seq).expect("serialization should succeed");
        let decoded: Sequence =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(seq, decoded);
    }

    #[test]
    fn sequence_parallel_serialization_roundtrip() {
        let seq = Sequence::Parallel(vec![Animation::create("n", 1.0)]);
        let json = serde_json::to_string(&seq).expect("serialization should succeed");
        let decoded: Sequence =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(seq, decoded);
    }

    #[test]
    fn sequence_group_serialization_roundtrip() {
        let inner = Sequence::Sequential(vec![Animation::fade_in("n", 0.5)]);
        let seq = Sequence::Group(vec![inner]);
        let json = serde_json::to_string(&seq).expect("serialization should succeed");
        let decoded: Sequence =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(seq, decoded);
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

    #[test]
    fn animation_player_default_equals_new() {
        let defaulted = AnimationPlayer::default();
        let created = AnimationPlayer::new();
        assert_eq!(defaulted.active_count(), created.active_count());
        assert!(!defaulted.is_playing());
    }

    #[test]
    fn animation_player_clone() {
        let mut player = AnimationPlayer::new();
        player.play(Animation::fade_in("n", 1.0));
        let cloned = player.clone();
        assert_eq!(cloned.active_count(), 1);
    }

    #[test]
    fn animation_target_debug_formatting() {
        let target = AnimationTarget::Opacity {
            node_id: "n".to_string(),
            from: 0.0,
            to: 1.0,
        };
        let s = format!("{target:?}");
        assert!(s.contains("Opacity"));
        assert!(s.contains('n'));
    }

    #[test]
    fn animation_target_partial_eq() {
        let a = AnimationTarget::StrokeDraw {
            node_id: "x".to_string(),
        };
        let b = AnimationTarget::StrokeDraw {
            node_id: "x".to_string(),
        };
        let c = AnimationTarget::StrokeDraw {
            node_id: "y".to_string(),
        };
        assert_eq!(a, b);
        assert_ne!(a, c);
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
}
