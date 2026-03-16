// SPDX-License-Identifier: AGPL-3.0-only
//! Pure display logic helpers for human entropy capture UI.
//!
//! Extracted from rendering.rs for testability and separation of concerns.

use super::types::CaptureWindowState;

/// Display state for the capture window (derived from `CaptureWindowState` + quality/elapsed).
#[derive(Debug, Clone, PartialEq)]
pub struct CaptureDisplayState {
    /// Label for the current state (e.g. "Idle", "Recording", "Stopped", "Processing")
    pub state_label: &'static str,
    /// Label for the primary action (e.g. "Start Capture", "Stop Capture")
    pub action_label: &'static str,
    /// Whether the Start button should be enabled
    pub can_start: bool,
    /// Whether the Stop button should be enabled
    pub can_stop: bool,
    /// Whether the Discard button should be enabled
    pub can_discard: bool,
    /// Progress/quality as percentage (0.0–100.0), or None when not applicable
    pub progress_percent: Option<f64>,
    /// Quality color RGB, or None when not applicable
    pub quality_color: Option<[u8; 3]>,
}

/// Map quality (0.0–1.0) to RGB color: green (good), yellow (medium), red (poor).
#[must_use]
pub fn quality_color_rgb(quality: f32) -> [u8; 3] {
    if quality > 0.7 {
        [0, 255, 0] // green
    } else if quality > 0.4 {
        [255, 255, 0] // yellow
    } else {
        [255, 0, 0] // red
    }
}

/// Format recording duration in seconds as human-readable string.
#[must_use]
pub fn format_recording_duration(elapsed_secs: f64) -> String {
    format!("{elapsed_secs:.1}s")
}

#[must_use]
pub fn quality_to_percent_display(quality: f64) -> String {
    format!("{:.1}%", quality * 100.0)
}

#[must_use]
pub fn stopped_state_message(quality: Option<f64>) -> String {
    quality.map_or_else(
        || "Capture complete!".to_string(),
        |q| format!("Capture complete! Quality: {:.1}%", q * 100.0),
    )
}

#[must_use]
pub const fn modality_selector_enabled(modality_available: bool, can_start: bool) -> bool {
    modality_available && can_start
}

/// Compute capture window display state from state, quality, and elapsed time.
#[must_use]
pub fn capture_state_display(
    state: &CaptureWindowState,
    current_quality: Option<f64>,
    _elapsed_secs: Option<f64>,
) -> CaptureDisplayState {
    match state {
        CaptureWindowState::Idle => CaptureDisplayState {
            state_label: "Idle",
            action_label: "Start Capture",
            can_start: true,
            can_stop: false,
            can_discard: false,
            progress_percent: None,
            quality_color: None,
        },
        CaptureWindowState::Recording => CaptureDisplayState {
            state_label: "Recording",
            action_label: "Stop Capture",
            can_start: false,
            can_stop: true,
            can_discard: false,
            progress_percent: current_quality.map(|q| q * 100.0),
            quality_color: current_quality.map(|q| quality_color_rgb(q as f32)),
        },
        CaptureWindowState::Stopped => CaptureDisplayState {
            state_label: "Stopped",
            action_label: "Send to Entropy Source",
            can_start: false,
            can_stop: false,
            can_discard: true,
            progress_percent: current_quality.map(|q| q * 100.0),
            quality_color: current_quality.map(|q| quality_color_rgb(q as f32)),
        },
        CaptureWindowState::Processing => CaptureDisplayState {
            state_label: "Processing",
            action_label: "Processing...",
            can_start: false,
            can_stop: false,
            can_discard: false,
            progress_percent: None,
            quality_color: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::CaptureWindowState;
    use super::*;

    #[test]
    fn quality_color_rgb_good() {
        let rgb = quality_color_rgb(0.8f32);
        assert_eq!(rgb, [0, 255, 0]);
    }

    #[test]
    fn quality_color_rgb_medium() {
        let rgb = quality_color_rgb(0.5f32);
        assert_eq!(rgb, [255, 255, 0]);
    }

    #[test]
    fn quality_color_rgb_poor() {
        let rgb = quality_color_rgb(0.3f32);
        assert_eq!(rgb, [255, 0, 0]);
    }

    #[test]
    fn quality_color_rgb_boundary_07() {
        let rgb = quality_color_rgb(0.71f32);
        assert_eq!(rgb, [0, 255, 0]);
    }

    #[test]
    fn quality_color_rgb_boundary_04() {
        let rgb = quality_color_rgb(0.41f32);
        assert_eq!(rgb, [255, 255, 0]);
    }

    #[test]
    fn test_format_recording_duration() {
        assert_eq!(format_recording_duration(0.0), "0.0s");
        assert_eq!(format_recording_duration(42.5), "42.5s");
        assert_eq!(format_recording_duration(123.456), "123.5s");
    }

    // Capture state display: Idle -> Recording -> Stopped -> Processing

    #[test]
    fn capture_state_display_idle() {
        let s = capture_state_display(&CaptureWindowState::Idle, None, None);
        assert_eq!(s.state_label, "Idle");
        assert_eq!(s.action_label, "Start Capture");
        assert!(s.can_start);
        assert!(!s.can_stop);
        assert!(!s.can_discard);
        assert_eq!(s.progress_percent, None);
        assert_eq!(s.quality_color, None);
    }

    #[test]
    fn capture_state_display_recording_no_quality() {
        let s = capture_state_display(&CaptureWindowState::Recording, None, Some(10.0));
        assert_eq!(s.state_label, "Recording");
        assert_eq!(s.action_label, "Stop Capture");
        assert!(!s.can_start);
        assert!(s.can_stop);
        assert!(!s.can_discard);
        assert_eq!(s.progress_percent, None);
        assert_eq!(s.quality_color, None);
    }

    #[test]
    fn capture_state_display_recording_with_quality() {
        let s = capture_state_display(&CaptureWindowState::Recording, Some(0.85), Some(5.0));
        assert_eq!(s.state_label, "Recording");
        assert!(s.can_stop);
        assert_eq!(s.progress_percent, Some(85.0));
        assert_eq!(s.quality_color, Some([0, 255, 0]));
    }

    #[test]
    fn capture_state_display_stopped() {
        let s = capture_state_display(&CaptureWindowState::Stopped, Some(0.6), None);
        assert_eq!(s.state_label, "Stopped");
        assert_eq!(s.action_label, "Send to Entropy Source");
        assert!(!s.can_start);
        assert!(!s.can_stop);
        assert!(s.can_discard);
        assert_eq!(s.progress_percent, Some(60.0));
        assert_eq!(s.quality_color, Some([255, 255, 0]));
    }

    #[test]
    fn capture_state_display_processing() {
        let s = capture_state_display(&CaptureWindowState::Processing, None, None);
        assert_eq!(s.state_label, "Processing");
        assert_eq!(s.action_label, "Processing...");
        assert!(!s.can_start);
        assert!(!s.can_stop);
        assert!(!s.can_discard);
        assert_eq!(s.progress_percent, None);
        assert_eq!(s.quality_color, None);
    }

    #[test]
    fn capture_state_transitions_idle_to_recording() {
        let idle = capture_state_display(&CaptureWindowState::Idle, None, None);
        let recording = capture_state_display(&CaptureWindowState::Recording, None, Some(1.0));
        assert!(idle.can_start);
        assert!(!recording.can_start);
        assert!(recording.can_stop);
    }

    #[test]
    fn capture_state_transitions_recording_to_stopped() {
        let recording =
            capture_state_display(&CaptureWindowState::Recording, Some(0.8), Some(30.0));
        let stopped = capture_state_display(&CaptureWindowState::Stopped, Some(0.8), None);
        assert!(!recording.can_discard);
        assert!(stopped.can_discard);
        assert!(!stopped.can_stop);
    }

    #[test]
    fn capture_state_transitions_stopped_to_processing() {
        let stopped = capture_state_display(&CaptureWindowState::Stopped, Some(0.9), None);
        let processing = capture_state_display(&CaptureWindowState::Processing, None, None);
        assert!(stopped.can_discard);
        assert!(!processing.can_discard);
        assert!(!processing.can_start);
    }

    #[test]
    fn quality_to_percent_display_values() {
        assert_eq!(quality_to_percent_display(0.0), "0.0%");
        assert_eq!(quality_to_percent_display(0.5), "50.0%");
        assert_eq!(quality_to_percent_display(1.0), "100.0%");
    }

    #[test]
    fn stopped_state_message_with_quality() {
        assert!(stopped_state_message(Some(0.85)).contains("85.0%"));
    }

    #[test]
    fn stopped_state_message_without_quality() {
        assert_eq!(stopped_state_message(None), "Capture complete!");
    }

    #[test]
    fn modality_selector_enabled_both() {
        assert!(modality_selector_enabled(true, true));
    }

    #[test]
    fn modality_selector_enabled_unavailable() {
        assert!(!modality_selector_enabled(false, true));
    }

    #[test]
    fn modality_selector_enabled_cannot_start() {
        assert!(!modality_selector_enabled(true, false));
    }
}
