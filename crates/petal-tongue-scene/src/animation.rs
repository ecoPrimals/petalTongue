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
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            Self::Spring => {
                let freq = 4.5 * std::f64::consts::PI;
                1.0 - ((-6.0 * t).exp() * (freq * t).cos())
            }
            Self::Bounce => {
                if t < 1.0 / 2.75 {
                    7.5625 * t * t
                } else if t < 2.0 / 2.75 {
                    let t2 = t - 1.5 / 2.75;
                    7.5625 * t2 * t2 + 0.75
                } else if t < 2.5 / 2.75 {
                    let t2 = t - 2.25 / 2.75;
                    7.5625 * t2 * t2 + 0.9375
                } else {
                    let t2 = t - 2.625 / 2.75;
                    7.5625 * t2 * t2 + 0.984_375
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
    pub fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Builder: set delay.
    #[must_use]
    pub fn with_delay(mut self, delay: f64) -> Self {
        self.delay_secs = delay;
        self
    }

    /// Total duration including delay.
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
    Group(Vec<Sequence>),
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
            Self::Group(seqs) => seqs.iter().map(Sequence::total_duration).sum(),
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
    pub fn new(animation: Animation) -> Self {
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
    pub fn is_done(&self) -> bool {
        self.elapsed >= self.animation.total_duration()
    }

    /// Current progress ratio (0.0 to 1.0), accounting for delay and easing.
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
    pub fn lerp_f64(&self, from: f64, to: f64) -> f64 {
        let t = self.progress();
        from + (to - from) * t
    }

    /// Interpolate a f32 value.
    pub fn lerp_f32(&self, from: f32, to: f32) -> f64 {
        let t = self.progress();
        f64::from(from) + f64::from(to - from) * t
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
}
