// SPDX-License-Identifier: AGPL-3.0-only
//! # Flower Animation
//!
//! ASCII and high-quality flower opening animation for awakening experience.

use std::time::Duration;

/// Flower Animation State
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowerState {
    /// Closed bud
    Closed,

    /// Opening (0-100%)
    Opening(u8),

    /// Fully open
    Open,

    /// Glowing (self-knowledge stage)
    Glowing,

    /// Reaching (discovery stage)
    Reaching,
}

/// Flower Animation Frame
#[derive(Debug, Clone)]
pub struct FlowerFrame {
    /// ASCII art representation
    pub ascii: String,

    /// Frame duration
    pub duration: Duration,

    /// Current state
    pub state: FlowerState,
}

/// Flower Animation Generator
pub struct FlowerAnimation {
    /// Current frame index
    frame_index: usize,

    /// Total frames
    total_frames: usize,

    /// Frame rate (FPS)
    fps: u32,
}

impl FlowerAnimation {
    /// Create new flower animation
    #[must_use]
    pub fn new(fps: u32) -> Self {
        Self {
            frame_index: 0,
            total_frames: 90, // 3 seconds at 30 FPS
            fps,
        }
    }

    /// Get next frame
    pub fn next_frame(&mut self) -> Option<FlowerFrame> {
        if self.frame_index >= self.total_frames {
            return None;
        }

        #[expect(clippy::cast_precision_loss)]
        let progress = (self.frame_index as f32) / (self.total_frames as f32);
        let state = Self::calculate_state(progress);
        let ascii = Self::generate_ascii(state);

        self.frame_index += 1;

        #[expect(clippy::cast_precision_loss)]
        let duration = Duration::from_secs_f32(1.0 / self.fps as f32);

        Some(FlowerFrame {
            ascii,
            duration,
            state,
        })
    }

    /// Reset animation
    pub fn reset(&mut self) {
        self.frame_index = 0;
    }

    /// Calculate state from progress
    fn calculate_state(progress: f32) -> FlowerState {
        if progress < 0.1 {
            FlowerState::Closed
        } else if progress < 0.9 {
            #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let percent = ((progress - 0.1) / 0.8 * 100.0) as u8;
            FlowerState::Opening(percent)
        } else {
            FlowerState::Open
        }
    }

    /// Generate ASCII art for current state
    fn generate_ascii(state: FlowerState) -> String {
        match state {
            FlowerState::Closed => Self::ascii_closed(),
            FlowerState::Opening(percent) => Self::ascii_opening(percent),
            FlowerState::Open => Self::ascii_open(),
            FlowerState::Glowing => Self::ascii_glowing(),
            FlowerState::Reaching => Self::ascii_reaching(),
        }
    }

    /// ASCII: Closed bud
    fn ascii_closed() -> String {
        r"
    ___
   /   \
  |  •  |
   \___/
        "
        .to_string()
    }

    /// ASCII: Opening flower
    fn ascii_opening(percent: u8) -> String {
        if percent < 33 {
            r"
    ___
   /   \
  | ••• |
   \___/
            "
            .to_string()
        } else if percent < 66 {
            r"
    _🌸_
   /   \
  | ••• |
   \____/
            "
            .to_string()
        } else {
            r"
   _🌸🌸_
  /     \
 | ••••• |
  \_____/
            "
            .to_string()
        }
    }

    /// ASCII: Fully open
    fn ascii_open() -> String {
        r"
  🌸🌸🌸
 /  |  \
| ••••• |
 \_____|
        "
        .to_string()
    }

    /// ASCII: Glowing (self-knowledge)
    fn ascii_glowing() -> String {
        r"
  ✨🌸✨
 /  |  \
| ••••• |
 \_____|
  ✨ ✨
        "
        .to_string()
    }

    /// ASCII: Reaching (discovery)
    fn ascii_reaching() -> String {
        r"
  🌸🌸🌸
 /~~|~~\
| ••••• |
 \_____|
  ~   ~
        "
        .to_string()
    }
}

/// Generate full awakening sequence
#[must_use]
pub fn generate_awakening_sequence(fps: u32) -> Vec<FlowerFrame> {
    let mut animation = FlowerAnimation::new(fps);
    let mut frames = Vec::new();

    // Stage 1: Awakening (0-3s) - Opening animation
    while let Some(frame) = animation.next_frame() {
        frames.push(frame);
    }

    // Stage 2: Self-Knowledge (3-6s) - Glowing
    #[expect(clippy::cast_precision_loss)]
    let frame_duration = Duration::from_secs_f32(1.0 / fps as f32);

    for _ in 0..(fps * 3) {
        frames.push(FlowerFrame {
            ascii: FlowerAnimation::ascii_glowing(),
            duration: frame_duration,
            state: FlowerState::Glowing,
        });
    }

    // Stage 3: Discovery (6-10s) - Reaching
    for _ in 0..(fps * 4) {
        frames.push(FlowerFrame {
            ascii: FlowerAnimation::ascii_reaching(),
            duration: frame_duration,
            state: FlowerState::Reaching,
        });
    }

    frames
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flower_animation_creation() {
        let animation = FlowerAnimation::new(30);
        assert_eq!(animation.frame_index, 0);
        assert_eq!(animation.fps, 30);
    }

    #[test]
    fn test_flower_animation_frames() {
        let mut animation = FlowerAnimation::new(30);

        let frame1 = animation.next_frame();
        assert!(frame1.is_some());

        let frame = frame1.unwrap();
        assert!(matches!(
            frame.state,
            FlowerState::Closed | FlowerState::Opening(_)
        ));
    }

    #[test]
    fn test_flower_animation_completion() {
        let mut animation = FlowerAnimation::new(30);

        // Exhaust all frames
        let mut count = 0;
        while animation.next_frame().is_some() {
            count += 1;
        }

        assert_eq!(count, 90); // 3 seconds at 30 FPS

        // Should return None after completion
        assert!(animation.next_frame().is_none());
    }

    #[test]
    fn test_flower_animation_reset() {
        let mut animation = FlowerAnimation::new(30);

        // Advance a few frames
        animation.next_frame();
        animation.next_frame();
        animation.next_frame();

        // Reset
        animation.reset();
        assert_eq!(animation.frame_index, 0);
    }

    #[test]
    fn test_awakening_sequence_generation() {
        let frames = generate_awakening_sequence(30);

        // Should have frames for all 3 stages
        // Stage 1: 3s * 30fps = 90 frames
        // Stage 2: 3s * 30fps = 90 frames
        // Stage 3: 4s * 30fps = 120 frames
        // Total: 300 frames
        assert_eq!(frames.len(), 300);
    }

    #[test]
    fn test_flower_states() {
        let _animation = FlowerAnimation::new(30);

        // Test state calculation
        assert_eq!(FlowerAnimation::calculate_state(0.0), FlowerState::Closed);
        assert!(matches!(
            FlowerAnimation::calculate_state(0.5),
            FlowerState::Opening(_)
        ));
        assert_eq!(FlowerAnimation::calculate_state(1.0), FlowerState::Open);
    }

    #[test]
    fn test_ascii_generation() {
        let closed = FlowerAnimation::ascii_closed();
        assert!(closed.contains("___"));

        let open = FlowerAnimation::ascii_open();
        assert!(open.contains("🌸"));

        let glowing = FlowerAnimation::ascii_glowing();
        assert!(glowing.contains("✨"));

        let reaching = FlowerAnimation::ascii_reaching();
        assert!(reaching.contains('~'));
    }

    #[test]
    fn test_ascii_opening_stages() {
        let early = FlowerAnimation::ascii_opening(10);
        let mid = FlowerAnimation::ascii_opening(50);
        let late = FlowerAnimation::ascii_opening(80);
        assert!(early.contains('•'));
        assert!(mid.contains('🌸') || mid.contains('_'));
        assert!(late.contains('🌸') || late.contains('•'));
    }

    #[test]
    fn test_flower_state_enum() {
        assert!(matches!(FlowerState::Closed, FlowerState::Closed));
        assert!(matches!(FlowerState::Opening(50), FlowerState::Opening(50)));
        assert!(matches!(FlowerState::Open, FlowerState::Open));
        assert!(matches!(FlowerState::Glowing, FlowerState::Glowing));
        assert!(matches!(FlowerState::Reaching, FlowerState::Reaching));
    }

    #[test]
    fn test_flower_frame_structure() {
        let mut animation = FlowerAnimation::new(30);
        let frame = animation.next_frame().expect("first frame");
        assert!(!frame.ascii.is_empty());
        assert_eq!(
            frame.duration,
            std::time::Duration::from_secs_f32(1.0 / 30.0)
        );
        assert!(matches!(
            frame.state,
            FlowerState::Closed | FlowerState::Opening(_)
        ));
    }

    #[test]
    fn test_calculate_state_boundaries() {
        assert_eq!(FlowerAnimation::calculate_state(0.0), FlowerState::Closed);
        assert_eq!(FlowerAnimation::calculate_state(0.09), FlowerState::Closed);
        assert_eq!(FlowerAnimation::calculate_state(1.0), FlowerState::Open);
        assert_eq!(FlowerAnimation::calculate_state(0.9), FlowerState::Open);
        if let FlowerState::Opening(p) = FlowerAnimation::calculate_state(0.5) {
            assert!(p > 0 && p < 100);
        } else {
            panic!("expected Opening");
        }
    }

    #[test]
    fn test_awakening_sequence_stages() {
        let frames = generate_awakening_sequence(30);
        let mut closed_count = 0;
        let mut opening_count = 0;
        let mut _open_count = 0;
        let mut glowing_count = 0;
        let mut reaching_count = 0;
        for f in &frames {
            match f.state {
                FlowerState::Closed => closed_count += 1,
                FlowerState::Opening(_) => opening_count += 1,
                FlowerState::Open => _open_count += 1,
                FlowerState::Glowing => glowing_count += 1,
                FlowerState::Reaching => reaching_count += 1,
            }
        }
        assert!(
            closed_count + opening_count > 0,
            "should have opening stage"
        );
        assert!(glowing_count > 0, "should have glowing stage");
        assert!(reaching_count > 0, "should have reaching stage");
    }

    #[test]
    fn test_flower_animation_different_fps() {
        let mut anim_30 = FlowerAnimation::new(30);
        let mut anim_60 = FlowerAnimation::new(60);
        let frame_30 = anim_30.next_frame().expect("frame");
        let frame_60 = anim_60.next_frame().expect("frame");
        assert_eq!(
            frame_30.duration,
            std::time::Duration::from_secs_f32(1.0 / 30.0)
        );
        assert_eq!(
            frame_60.duration,
            std::time::Duration::from_secs_f32(1.0 / 60.0)
        );
    }

    #[test]
    fn test_flower_frame_debug() {
        let frame = FlowerFrame {
            ascii: "test".to_string(),
            duration: std::time::Duration::from_secs(1),
            state: FlowerState::Closed,
        };
        let debug_str = format!("{frame:?}");
        assert!(debug_str.contains("FlowerFrame"));
    }
}
