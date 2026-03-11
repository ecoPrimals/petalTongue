// SPDX-License-Identifier: AGPL-3.0-only
//! Game-engine-style tick loop with fixed timestep.
//!
//! Implements the 8-step interaction-rendering cycle:
//!
//! 1. **POLL** — Collect sensor events from all input adapters
//! 2. **TRANSLATE** — Convert raw events to semantic `InteractionIntent`
//! 3. **RESOLVE** — Run inverse pipelines to map intents to data targets
//! 4. **APPLY** — Update interaction state (selection, focus, navigation)
//! 5. **RECOMPILE** — Re-evaluate grammar/scene if data or view changed
//! 6. **RENDER** — Compile scene to output modalities (egui, audio, etc.)
//! 7. **BROADCAST** — Send state changes to other perspectives / IPC
//! 8. **CONFIRM** — Close the SAME DAVE loop (motor → sensor confirmation)
//!
//! The loop uses a fixed-timestep accumulator for physics simulation
//! while allowing variable-rate rendering.

use std::time::{Duration, Instant};

use crate::animation::AnimationPlayer;
use crate::physics::PhysicsWorld;
use crate::scene_graph::SceneGraph;

/// Configuration for the game loop.
#[derive(Debug, Clone)]
pub struct TickConfig {
    /// Fixed timestep for physics/animation (default 16.67ms = 60Hz).
    pub fixed_dt: Duration,
    /// Maximum accumulated time before dropping frames (prevents spiral of death).
    pub max_accumulator: Duration,
    /// Whether physics simulation is enabled.
    pub physics_enabled: bool,
    /// Whether animation is enabled.
    pub animation_enabled: bool,
}

impl Default for TickConfig {
    fn default() -> Self {
        Self {
            fixed_dt: Duration::from_micros(16_667),
            max_accumulator: Duration::from_millis(250),
            physics_enabled: true,
            animation_enabled: true,
        }
    }
}

/// Tracks timing for the fixed-timestep game loop.
#[derive(Debug)]
pub struct TickClock {
    config: TickConfig,
    last_tick: Instant,
    accumulator: Duration,
    frame_count: u64,
    physics_steps_this_frame: u32,
}

impl TickClock {
    pub fn new(config: TickConfig) -> Self {
        Self {
            config,
            last_tick: Instant::now(),
            accumulator: Duration::ZERO,
            frame_count: 0,
            physics_steps_this_frame: 0,
        }
    }

    /// Call at the start of each frame. Returns the real elapsed time.
    pub fn begin_frame(&mut self) -> Duration {
        let now = Instant::now();
        let elapsed = now - self.last_tick;
        self.last_tick = now;
        self.accumulate(elapsed);
        elapsed
    }

    /// Call at the start of each frame with an externally provided delta time.
    ///
    /// Use this when the host (e.g. egui) provides a stable delta time that
    /// should be authoritative instead of wall-clock measurement.
    pub fn begin_frame_with_dt(&mut self, dt_secs: f32) {
        self.accumulate(Duration::from_secs_f32(dt_secs));
    }

    fn accumulate(&mut self, elapsed: Duration) {
        self.accumulator += elapsed;
        if self.accumulator > self.config.max_accumulator {
            self.accumulator = self.config.max_accumulator;
        }
        self.frame_count += 1;
        self.physics_steps_this_frame = 0;
    }

    /// Whether there's enough accumulated time for another fixed-step tick.
    pub fn should_tick(&self) -> bool {
        self.accumulator >= self.config.fixed_dt
    }

    /// Consume one fixed-step tick worth of time. Returns the fixed dt in seconds.
    pub fn consume_tick(&mut self) -> f64 {
        self.accumulator -= self.config.fixed_dt;
        self.physics_steps_this_frame += 1;
        self.config.fixed_dt.as_secs_f64()
    }

    /// Interpolation factor for rendering between physics steps (0.0 to 1.0).
    pub fn alpha(&self) -> f64 {
        self.accumulator.as_secs_f64() / self.config.fixed_dt.as_secs_f64()
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    pub fn physics_steps_this_frame(&self) -> u32 {
        self.physics_steps_this_frame
    }

    pub fn fixed_dt_secs(&self) -> f64 {
        self.config.fixed_dt.as_secs_f64()
    }

    /// Access the tick config for reading.
    #[must_use]
    pub fn config(&self) -> &TickConfig {
        &self.config
    }

    /// Access the tick config for mutation (e.g. enabling/disabling physics).
    pub fn config_mut(&mut self) -> &mut TickConfig {
        &mut self.config
    }
}

/// Result of a single tick step (fixed-timestep physics/animation update).
#[derive(Debug, Default)]
pub struct TickResult {
    pub physics_stepped: bool,
    pub animation_stepped: bool,
    pub scene_dirty: bool,
}

/// Run the fixed-timestep update loop for one frame.
///
/// The caller must call `clock.begin_frame()` or `clock.begin_frame_with_dt()`
/// before calling this function to accumulate the frame's delta time.
pub fn tick_frame(
    clock: &mut TickClock,
    mut physics: Option<&mut PhysicsWorld>,
    mut animation: Option<&mut AnimationPlayer>,
    mut scene: Option<&mut SceneGraph>,
) -> TickResult {
    let mut result = TickResult::default();

    while clock.should_tick() {
        let dt = clock.consume_tick();

        if clock.config.physics_enabled
            && let Some(phys) = physics.as_deref_mut()
        {
            phys.step_euler();
            result.physics_stepped = true;
            result.scene_dirty = true;
        }

        if clock.config.animation_enabled
            && let Some(anim) = animation.as_deref_mut()
            && let Some(sc) = scene.as_deref_mut()
        {
            anim.tick(dt, sc);
            result.animation_stepped = true;
            result.scene_dirty = true;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_config_default() {
        let cfg = TickConfig::default();
        assert!(cfg.fixed_dt.as_micros() > 0);
        assert!(cfg.physics_enabled);
        assert!(cfg.animation_enabled);
    }

    #[test]
    fn tick_clock_begin_frame() {
        let mut clock = TickClock::new(TickConfig::default());
        std::thread::sleep(Duration::from_millis(20));
        let elapsed = clock.begin_frame();
        assert!(elapsed >= Duration::from_millis(15));
        assert_eq!(clock.frame_count(), 1);
    }

    #[test]
    fn tick_clock_consume_tick() {
        let cfg = TickConfig {
            fixed_dt: Duration::from_millis(10),
            ..TickConfig::default()
        };
        let mut clock = TickClock::new(cfg);
        std::thread::sleep(Duration::from_millis(25));
        clock.begin_frame();

        let mut ticks = 0;
        while clock.should_tick() {
            let dt = clock.consume_tick();
            assert!((dt - 0.01).abs() < 0.001);
            ticks += 1;
        }
        assert!(ticks >= 2, "expected at least 2 ticks, got {ticks}");
    }

    #[test]
    fn tick_clock_alpha() {
        let cfg = TickConfig {
            fixed_dt: Duration::from_millis(16),
            ..TickConfig::default()
        };
        let mut clock = TickClock::new(cfg);
        std::thread::sleep(Duration::from_millis(20));
        clock.begin_frame();
        while clock.should_tick() {
            clock.consume_tick();
        }
        let alpha = clock.alpha();
        assert!((0.0..=1.0).contains(&alpha), "alpha={alpha}");
    }

    #[test]
    fn tick_frame_no_physics_no_animation() {
        let cfg = TickConfig {
            fixed_dt: Duration::from_millis(10),
            physics_enabled: false,
            animation_enabled: false,
            ..TickConfig::default()
        };
        let mut clock = TickClock::new(cfg);
        std::thread::sleep(Duration::from_millis(20));
        clock.begin_frame();
        let result = tick_frame(&mut clock, None, None, None);
        assert!(!result.physics_stepped);
        assert!(!result.animation_stepped);
        assert!(!result.scene_dirty);
    }

    #[test]
    fn begin_frame_with_dt_accumulates() {
        let cfg = TickConfig {
            fixed_dt: Duration::from_millis(10),
            ..TickConfig::default()
        };
        let mut clock = TickClock::new(cfg);
        clock.begin_frame_with_dt(0.025);
        let mut ticks = 0;
        while clock.should_tick() {
            clock.consume_tick();
            ticks += 1;
        }
        assert!(
            ticks >= 2,
            "expected at least 2 ticks from 25ms dt, got {ticks}"
        );
    }

    #[test]
    fn tick_clock_max_accumulator_prevents_spiral() {
        let cfg = TickConfig {
            fixed_dt: Duration::from_millis(16),
            max_accumulator: Duration::from_millis(100),
            ..TickConfig::default()
        };
        let mut clock = TickClock::new(cfg);
        std::thread::sleep(Duration::from_millis(200));
        clock.begin_frame();

        let mut ticks = 0;
        while clock.should_tick() {
            clock.consume_tick();
            ticks += 1;
        }
        assert!(ticks <= 7, "max_accumulator should cap ticks, got {ticks}");
    }
}
