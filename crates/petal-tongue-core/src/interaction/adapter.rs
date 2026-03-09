// SPDX-License-Identifier: AGPL-3.0-only
//! Input adapter trait -- converts device events to semantic intents.
//!
//! Each input modality (mouse, keyboard, voice, Braille, gamepad) implements
//! [`InputAdapter`] to translate raw [`SensorEvent`](crate::sensor::SensorEvent)
//! values into modality-agnostic [`InteractionIntent`] values.

use crate::sensor::SensorEvent;
use serde::{Deserialize, Serialize};

use super::intent::InteractionIntent;
use super::perspective::PerspectiveId;
use super::result::InteractionResult;
use super::target::InteractionTarget;

/// Translates device events into semantic interaction intents.
///
/// Implementations are stateful -- they track focus, drag state, gesture
/// recognition, etc. The adapter does NOT resolve targets to data space;
/// that is the job of [`InversePipeline`](super::inverse::InversePipeline).
pub trait InputAdapter: Send + Sync {
    /// Human-readable name for logging.
    fn name(&self) -> &'static str;

    /// Which input modality this adapter handles.
    fn modality(&self) -> InputModality;

    /// What interaction capabilities this adapter supports.
    fn capabilities(&self) -> &[InteractionCapability];

    /// Translate a sensor event into a semantic intent.
    ///
    /// Returns `None` if this adapter doesn't handle the event (wrong
    /// modality, irrelevant event type, gesture not yet complete).
    fn translate(
        &self,
        event: &SensorEvent,
        context: &InteractionContext,
    ) -> Option<InteractionIntent>;

    /// What is currently under focus for this adapter (e.g. hovered node).
    fn active_target(&self, context: &InteractionContext) -> Option<InteractionTarget>;

    /// Receive feedback about a completed interaction (for haptic/audio
    /// confirmation, state update, etc).
    fn feedback(&mut self, result: &InteractionResult);
}

/// Input modality classification.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum InputModality {
    /// Mouse pointer.
    PointerMouse,
    /// Touchscreen.
    PointerTouch,
    /// Stylus / pen tablet.
    PointerStylus,
    /// Physical keyboard.
    Keyboard,
    /// Game controller.
    Gamepad,
    /// Voice / speech commands.
    VoiceCommand,
    /// Braille display with routing keys.
    BrailleDisplay,
    /// Single-switch scanning access.
    SwitchAccess,
    /// Eye gaze tracking.
    EyeGaze,
    /// Full-body motion capture (VR/AR).
    MotionCapture,
    /// Custom input modality.
    Custom(String),
}

impl std::fmt::Display for InputModality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PointerMouse => write!(f, "mouse"),
            Self::PointerTouch => write!(f, "touch"),
            Self::PointerStylus => write!(f, "stylus"),
            Self::Keyboard => write!(f, "keyboard"),
            Self::Gamepad => write!(f, "gamepad"),
            Self::VoiceCommand => write!(f, "voice"),
            Self::BrailleDisplay => write!(f, "braille"),
            Self::SwitchAccess => write!(f, "switch"),
            Self::EyeGaze => write!(f, "eyegaze"),
            Self::MotionCapture => write!(f, "motion"),
            Self::Custom(name) => write!(f, "custom:{name}"),
        }
    }
}

/// What kinds of interactions an adapter can produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum InteractionCapability {
    /// Can select individual objects.
    PointSelect,
    /// Can select ranges (brush, slider).
    RangeSelect,
    /// Can draw freeform selection areas (lasso).
    FreeformSelect,
    /// Can navigate in 2D space.
    Navigate2D,
    /// Can navigate in 3D space.
    Navigate3D,
    /// Can enter text.
    TextInput,
    /// Can provide continuous analog values (pressure, tilt).
    ContinuousValue,
    /// Can make discrete choices (button press, menu select).
    DiscreteChoice,
}

/// Shared context available to all adapters during translation.
///
/// Provides the adapter with awareness of the current state without
/// requiring it to maintain global references.
#[derive(Debug, Clone)]
pub struct InteractionContext {
    /// Active perspective ID.
    pub perspective_id: PerspectiveId,
    /// Current viewport center X.
    pub viewport_center_x: f64,
    /// Current viewport center Y.
    pub viewport_center_y: f64,
    /// Current zoom level.
    pub zoom: f64,
    /// Currently focused data object (from any adapter).
    pub current_focus: Option<InteractionTarget>,
    /// Currently selected data objects.
    pub current_selection: Vec<super::target::DataObjectId>,
    /// Canvas/screen dimensions in pixels.
    pub screen_width: f32,
    /// Canvas/screen height in pixels.
    pub screen_height: f32,
}

impl InteractionContext {
    /// Create a minimal context for testing.
    #[must_use]
    pub fn default_for_perspective(perspective_id: PerspectiveId) -> Self {
        Self {
            perspective_id,
            viewport_center_x: 0.0,
            viewport_center_y: 0.0,
            zoom: 1.0,
            current_focus: None,
            current_selection: Vec::new(),
            screen_width: 800.0,
            screen_height: 600.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_modality_display() {
        assert_eq!(format!("{}", InputModality::PointerMouse), "mouse");
        assert_eq!(format!("{}", InputModality::Keyboard), "keyboard");
        assert_eq!(
            format!("{}", InputModality::Custom("midi".into())),
            "custom:midi"
        );
    }

    #[test]
    fn interaction_context_defaults() {
        let ctx = InteractionContext::default_for_perspective(1);
        assert_eq!(ctx.perspective_id, 1);
        assert!((ctx.zoom - 1.0).abs() < f64::EPSILON);
        assert!(ctx.current_selection.is_empty());
    }
}
