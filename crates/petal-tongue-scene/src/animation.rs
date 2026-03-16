// SPDX-License-Identifier: AGPL-3.0-or-later
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

// Tests extracted to tests/animation_tests.rs to keep this file under 1,000 lines.
// Run with: cargo test -p petal-tongue-scene
