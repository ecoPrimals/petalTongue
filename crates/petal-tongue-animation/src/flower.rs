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

        let progress = (self.frame_index as f32) / (self.total_frames as f32);
        let state = self.calculate_state(progress);
        let ascii = self.generate_ascii(state, progress);

        self.frame_index += 1;

        Some(FlowerFrame {
            ascii,
            duration: Duration::from_secs_f32(1.0 / self.fps as f32),
            state,
        })
    }

    /// Reset animation
    pub fn reset(&mut self) {
        self.frame_index = 0;
    }

    /// Calculate state from progress
    fn calculate_state(&self, progress: f32) -> FlowerState {
        if progress < 0.1 {
            FlowerState::Closed
        } else if progress < 0.9 {
            FlowerState::Opening(((progress - 0.1) / 0.8 * 100.0) as u8)
        } else {
            FlowerState::Open
        }
    }

    /// Generate ASCII art for current state
    fn generate_ascii(&self, state: FlowerState, progress: f32) -> String {
        match state {
            FlowerState::Closed => self.ascii_closed(),
            FlowerState::Opening(percent) => self.ascii_opening(percent),
            FlowerState::Open => self.ascii_open(),
            FlowerState::Glowing => self.ascii_glowing(),
            FlowerState::Reaching => self.ascii_reaching(),
        }
    }

    /// ASCII: Closed bud
    fn ascii_closed(&self) -> String {
        r"
    ___
   /   \
  |  •  |
   \___/
        "
        .to_string()
    }

    /// ASCII: Opening flower
    fn ascii_opening(&self, percent: u8) -> String {
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
    fn ascii_open(&self) -> String {
        r"
  🌸🌸🌸
 /  |  \
| ••••• |
 \_____|
        "
        .to_string()
    }

    /// ASCII: Glowing (self-knowledge)
    fn ascii_glowing(&self) -> String {
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
    fn ascii_reaching(&self) -> String {
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
pub fn generate_awakening_sequence(fps: u32) -> Vec<FlowerFrame> {
    let mut animation = FlowerAnimation::new(fps);
    let mut frames = Vec::new();

    // Stage 1: Awakening (0-3s) - Opening animation
    while let Some(frame) = animation.next_frame() {
        frames.push(frame);
    }

    // Stage 2: Self-Knowledge (3-6s) - Glowing
    for _ in 0..(fps * 3) {
        frames.push(FlowerFrame {
            ascii: animation.ascii_glowing(),
            duration: Duration::from_secs_f32(1.0 / fps as f32),
            state: FlowerState::Glowing,
        });
    }

    // Stage 3: Discovery (6-10s) - Reaching
    for _ in 0..(fps * 4) {
        frames.push(FlowerFrame {
            ascii: animation.ascii_reaching(),
            duration: Duration::from_secs_f32(1.0 / fps as f32),
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
        let animation = FlowerAnimation::new(30);

        // Test state calculation
        assert_eq!(animation.calculate_state(0.0), FlowerState::Closed);
        assert!(matches!(
            animation.calculate_state(0.5),
            FlowerState::Opening(_)
        ));
        assert_eq!(animation.calculate_state(1.0), FlowerState::Open);
    }

    #[test]
    fn test_ascii_generation() {
        let animation = FlowerAnimation::new(30);

        let closed = animation.ascii_closed();
        assert!(closed.contains("___"));

        let open = animation.ascii_open();
        assert!(open.contains("🌸"));

        let glowing = animation.ascii_glowing();
        assert!(glowing.contains("✨"));

        let reaching = animation.ascii_reaching();
        assert!(reaching.contains("~"));
    }
}
