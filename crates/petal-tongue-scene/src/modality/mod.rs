// SPDX-License-Identifier: AGPL-3.0-only
//! Modality compilers: scene graph to output formats.
//!
//! Each compiler produces a different output modality: SVG, terminal cells,
//! audio parameters, GPU commands, or text descriptions for accessibility.

pub mod audio;
pub mod braille;
pub mod description;
pub mod haptic;
pub mod svg;
pub mod terminal;

use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::render_plan::RenderPlan;
use crate::scene_graph::SceneGraph;

pub use braille::BrailleCell;
pub use haptic::{HapticCommand, HapticPattern};

/// Output of a modality compiler.
///
/// Uses `bytes::Bytes` for binary/text payloads to enable zero-copy sharing
/// across the visualization pipeline (`UNIVERSAL_VISUALIZATION_PIPELINE` spec).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModalityOutput {
    /// SVG document (UTF-8 bytes, zero-copy).
    Svg(Bytes),
    /// Terminal character grid.
    TerminalCells(Vec<Vec<char>>),
    /// Audio synthesis parameters.
    AudioParams(Vec<AudioParam>),
    /// Raw GPU command bytes (zero-copy).
    GpuCommands(Bytes),
    /// Text description for accessibility (UTF-8 bytes, zero-copy).
    Description(Bytes),
    /// Braille dot pattern grid for tactile displays.
    BrailleCells(Vec<Vec<BrailleCell>>),
    /// Haptic feedback commands for force-feedback devices.
    HapticCommands(Vec<HapticCommand>),
}

/// Audio parameter for a single datum.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioParam {
    /// Frequency in Hz (e.g. 200–2000).
    pub frequency: f64,
    /// Amplitude 0.0 to 1.0.
    pub amplitude: f64,
    /// Pan -1.0 (left) to 1.0 (right).
    pub pan: f64,
    /// Duration in seconds.
    pub duration_secs: f64,
}

/// Trait for compiling a scene graph to a specific output modality.
pub trait ModalityCompiler: Send + Sync {
    /// Compile the scene graph to output.
    fn compile(&self, scene: &SceneGraph) -> ModalityOutput;

    /// Compile from a full render plan (default: delegates to `compile` on the plan's scene).
    fn compile_plan(&self, plan: &RenderPlan) -> ModalityOutput {
        self.compile(&plan.scene)
    }

    /// Human-readable compiler name.
    fn name(&self) -> &'static str;
}

// Re-exports
pub use audio::AudioCompiler;
pub use braille::BrailleCompiler;
pub use description::DescriptionCompiler;
pub use haptic::HapticCompiler;
pub use svg::SvgCompiler;
pub use terminal::TerminalCompiler;
