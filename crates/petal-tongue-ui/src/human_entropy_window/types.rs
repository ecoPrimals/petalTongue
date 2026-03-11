// SPDX-License-Identifier: AGPL-3.0-only
//! Human entropy window types

/// Capture modality selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EntropyModality {
    Audio,
    Visual,
    Narrative,
    Gesture,
    Video,
}

impl EntropyModality {
    pub(super) fn name(&self) -> &'static str {
        match self {
            Self::Audio => "🎵 Audio (Sing a Song)",
            Self::Visual => "🎨 Visual (Draw/Paint)",
            Self::Narrative => "📝 Narrative (Tell a Story)",
            Self::Gesture => "✋ Gesture (Motion)",
            Self::Video => "📹 Video (Camera)",
        }
    }

    pub(super) fn description(&self) -> &'static str {
        match self {
            Self::Audio => {
                "Capture entropy from your voice - sing a song, tell a story, or just speak naturally."
            }
            Self::Visual => {
                "Draw or paint something unique. Your strokes, timing, and patterns create entropy."
            }
            Self::Narrative => {
                "Type a story, poem, or thoughts. Your keystroke timing and style create entropy."
            }
            Self::Gesture => {
                "Use motion sensors or touch patterns to create entropy from your natural movements."
            }
            Self::Video => {
                "Use your camera to capture entropy from your movements and environment."
            }
        }
    }

    pub(super) fn is_available(&self) -> bool {
        match self {
            Self::Audio => crate::sensors::audio::has_audio_input(),

            Self::Narrative => true, // Always available

            Self::Visual => false, // Phase 3: requires camera sensor integration
            Self::Gesture => false, // Phase 5: requires gesture recognition pipeline
            Self::Video => false,  // Phase 6: requires video stream processing
        }
    }
}

/// Capture window state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CaptureWindowState {
    /// Idle, ready to start
    Idle,

    /// Actively capturing
    Recording,

    /// Stopped, ready to finalize or discard
    Stopped,

    /// Processing/streaming
    Processing,
}
