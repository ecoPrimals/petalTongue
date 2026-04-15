// SPDX-License-Identifier: AGPL-3.0-or-later
//! Input and output capability bitsets.

use serde::{Deserialize, Serialize};

use crate::interaction::adapter::InputModality;
use crate::interaction::perspective::OutputModality;

/// What a user/environment can provide as input.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "each bool is an independent capability flag"
)]
pub struct InputCapabilitySet {
    /// Pointer device (mouse, trackpad, stylus).
    pub pointer: bool,
    /// Physical or virtual keyboard.
    pub keyboard: bool,
    /// Voice/speech command input.
    pub voice: bool,
    /// Gesture/motion capture input.
    pub gesture: bool,
    /// Touchscreen input.
    pub touch: bool,
    /// Eye/gaze tracking.
    pub gaze: bool,
    /// Single-switch or sip-and-puff binary input.
    pub switch_input: bool,
    /// Agentic AI (Squirrel) or machine API input.
    pub api_agent: bool,
}

impl InputCapabilitySet {
    /// All known input modalities as `InputModality` values.
    #[must_use]
    pub fn active_modalities(&self) -> Vec<InputModality> {
        let mut out = Vec::new();
        if self.pointer {
            out.push(InputModality::PointerMouse);
        }
        if self.keyboard {
            out.push(InputModality::Keyboard);
        }
        if self.voice {
            out.push(InputModality::VoiceCommand);
        }
        if self.gesture {
            out.push(InputModality::MotionCapture);
        }
        if self.touch {
            out.push(InputModality::PointerTouch);
        }
        if self.gaze {
            out.push(InputModality::EyeGaze);
        }
        if self.switch_input {
            out.push(InputModality::SwitchAccess);
        }
        if self.api_agent {
            out.push(InputModality::Agent);
        }
        out
    }

    /// True when at least one input channel is available.
    #[must_use]
    pub const fn has_any(&self) -> bool {
        self.pointer
            || self.keyboard
            || self.voice
            || self.gesture
            || self.touch
            || self.gaze
            || self.switch_input
            || self.api_agent
    }

    /// Count of active input channels.
    #[must_use]
    pub const fn count(&self) -> u8 {
        self.pointer as u8
            + self.keyboard as u8
            + self.voice as u8
            + self.gesture as u8
            + self.touch as u8
            + self.gaze as u8
            + self.switch_input as u8
            + self.api_agent as u8
    }
}

/// What petalTongue can render to this user.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "each bool is an independent capability flag"
)]
pub struct OutputCapabilitySet {
    /// Graphical display (egui, web canvas).
    pub visual_gui: bool,
    /// Terminal UI (ratatui).
    pub terminal: bool,
    /// Audio sonification.
    pub audio: bool,
    /// Braille display.
    pub braille: bool,
    /// Haptic/force feedback.
    pub haptic: bool,
    /// Text description / narration (screen reader).
    pub description: bool,
    /// SVG static export.
    pub svg: bool,
    /// GPU-accelerated 3D rendering.
    pub gpu: bool,
    /// JSON/API machine-readable output.
    pub api_machine: bool,
}

impl OutputCapabilitySet {
    /// All active output modalities as `OutputModality` values.
    #[must_use]
    pub fn active_modalities(&self) -> Vec<OutputModality> {
        let mut out = Vec::new();
        if self.visual_gui {
            out.push(OutputModality::Gui);
        }
        if self.terminal {
            out.push(OutputModality::Tui);
        }
        if self.audio {
            out.push(OutputModality::Audio);
        }
        if self.braille {
            out.push(OutputModality::Braille);
        }
        if self.haptic {
            out.push(OutputModality::Haptic);
        }
        if self.description {
            out.push(OutputModality::Headless);
        }
        if self.svg {
            out.push(OutputModality::Svg);
        }
        if self.gpu {
            out.push(OutputModality::Gui);
        }
        if self.api_machine {
            out.push(OutputModality::Json);
        }
        out
    }

    /// True when at least one output channel is available.
    #[must_use]
    pub const fn has_any(&self) -> bool {
        self.visual_gui
            || self.terminal
            || self.audio
            || self.braille
            || self.haptic
            || self.description
            || self.svg
            || self.gpu
            || self.api_machine
    }

    /// Count of active output channels.
    #[must_use]
    pub const fn count(&self) -> u8 {
        self.visual_gui as u8
            + self.terminal as u8
            + self.audio as u8
            + self.braille as u8
            + self.haptic as u8
            + self.description as u8
            + self.svg as u8
            + self.gpu as u8
            + self.api_machine as u8
    }
}
