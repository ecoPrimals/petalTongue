// SPDX-License-Identifier: AGPL-3.0-or-later
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

#[cfg(test)]
mod test_utils;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::{GeometryType, GrammarExpr};
    use crate::render_plan::RenderPlan;
    use crate::scene_graph::SceneGraph;

    #[test]
    fn modality_output_svg_serde() {
        let out = ModalityOutput::Svg(Bytes::from_static(b"<svg/>"));
        let json = serde_json::to_string(&out).unwrap();
        let restored: ModalityOutput = serde_json::from_str(&json).unwrap();
        match (&out, &restored) {
            (ModalityOutput::Svg(a), ModalityOutput::Svg(b)) => assert_eq!(a, b),
            _ => panic!("variant mismatch"),
        }
    }

    #[test]
    fn modality_output_terminal_cells_serde() {
        let out = ModalityOutput::TerminalCells(vec![vec!['a', 'b'], vec!['c', 'd']]);
        let json = serde_json::to_string(&out).unwrap();
        let restored: ModalityOutput = serde_json::from_str(&json).unwrap();
        match (&out, &restored) {
            (ModalityOutput::TerminalCells(a), ModalityOutput::TerminalCells(b)) => {
                assert_eq!(a, b);
            }
            _ => panic!("variant mismatch"),
        }
    }

    #[test]
    fn modality_output_audio_params_serde() {
        let out = ModalityOutput::AudioParams(vec![AudioParam {
            frequency: 440.0,
            amplitude: 0.5,
            pan: 0.0,
            duration_secs: 0.1,
        }]);
        let json = serde_json::to_string(&out).unwrap();
        let restored: ModalityOutput = serde_json::from_str(&json).unwrap();
        match &restored {
            ModalityOutput::AudioParams(b) => {
                assert_eq!(b.len(), 1);
                assert!((b[0].frequency - 440.0).abs() < f64::EPSILON);
                assert!((b[0].amplitude - 0.5).abs() < f64::EPSILON);
            }
            _ => panic!("variant mismatch"),
        }
    }

    #[test]
    fn modality_output_gpu_commands_serde() {
        let out = ModalityOutput::GpuCommands(Bytes::from_static(b"gpu"));
        let json = serde_json::to_string(&out).unwrap();
        let restored: ModalityOutput = serde_json::from_str(&json).unwrap();
        match (&out, &restored) {
            (ModalityOutput::GpuCommands(a), ModalityOutput::GpuCommands(b)) => assert_eq!(a, b),
            _ => panic!("variant mismatch"),
        }
    }

    #[test]
    fn modality_output_description_serde() {
        let out = ModalityOutput::Description(Bytes::from_static(b"text"));
        let json = serde_json::to_string(&out).unwrap();
        let restored: ModalityOutput = serde_json::from_str(&json).unwrap();
        match (&out, &restored) {
            (ModalityOutput::Description(a), ModalityOutput::Description(b)) => assert_eq!(a, b),
            _ => panic!("variant mismatch"),
        }
    }

    #[test]
    fn modality_output_braille_cells_serde() {
        let out = ModalityOutput::BrailleCells(vec![vec![BrailleCell { dots: 1 }]]);
        let json = serde_json::to_string(&out).unwrap();
        let restored: ModalityOutput = serde_json::from_str(&json).unwrap();
        match (&out, &restored) {
            (ModalityOutput::BrailleCells(a), ModalityOutput::BrailleCells(b)) => assert_eq!(a, b),
            _ => panic!("variant mismatch"),
        }
    }

    #[test]
    fn modality_output_haptic_commands_serde() {
        let out = ModalityOutput::HapticCommands(vec![HapticCommand {
            intensity: 0.5,
            duration_secs: 0.1,
            position: [0.5, 0.5],
            pattern: HapticPattern::Pulse,
        }]);
        let json = serde_json::to_string(&out).unwrap();
        let restored: ModalityOutput = serde_json::from_str(&json).unwrap();
        match &restored {
            ModalityOutput::HapticCommands(b) => {
                assert_eq!(b.len(), 1);
                assert!((b[0].intensity - 0.5).abs() < f64::EPSILON);
                assert_eq!(b[0].pattern, HapticPattern::Pulse);
            }
            _ => panic!("variant mismatch"),
        }
    }

    #[test]
    fn audio_param_construction() {
        let param = AudioParam {
            frequency: 1000.0,
            amplitude: 0.8,
            pan: -0.5,
            duration_secs: 0.2,
        };
        assert!((param.frequency - 1000.0).abs() < f64::EPSILON);
        assert!((param.amplitude - 0.8).abs() < f64::EPSILON);
        assert!((param.pan + 0.5).abs() < f64::EPSILON);
        assert!((param.duration_secs - 0.2).abs() < f64::EPSILON);
    }

    #[test]
    fn audio_param_serde() {
        let param = AudioParam {
            frequency: 440.0,
            amplitude: 0.5,
            pan: 0.0,
            duration_secs: 0.1,
        };
        let json = serde_json::to_string(&param).unwrap();
        let restored: AudioParam = serde_json::from_str(&json).unwrap();
        assert!((param.frequency - restored.frequency).abs() < f64::EPSILON);
        assert!((param.amplitude - restored.amplitude).abs() < f64::EPSILON);
    }

    #[test]
    fn compile_plan_delegates_to_compile() {
        let scene = SceneGraph::new();
        let grammar = GrammarExpr::new("data", GeometryType::Point);
        let plan = RenderPlan::new(scene.clone(), grammar);
        let compiler = AudioCompiler::new();
        let out_plan = compiler.compile_plan(&plan);
        let out_direct = compiler.compile(&scene);
        match (&out_plan, &out_direct) {
            (ModalityOutput::AudioParams(a), ModalityOutput::AudioParams(b)) => {
                assert_eq!(a.len(), b.len());
            }
            _ => panic!("expected AudioParams"),
        }
    }

    #[test]
    fn audio_compiler_name() {
        assert_eq!(AudioCompiler::new().name(), "AudioCompiler");
    }

    #[test]
    fn braille_compiler_name() {
        assert_eq!(BrailleCompiler::new(40, 12).name(), "BrailleCompiler");
    }

    #[test]
    fn description_compiler_name() {
        assert_eq!(DescriptionCompiler::new().name(), "DescriptionCompiler");
    }

    #[test]
    fn haptic_compiler_name() {
        assert_eq!(HapticCompiler::new().name(), "HapticCompiler");
    }

    #[test]
    fn svg_compiler_name() {
        assert_eq!(SvgCompiler::new().name(), "SvgCompiler");
    }

    #[test]
    fn terminal_compiler_name() {
        assert_eq!(TerminalCompiler::new(80, 24).name(), "TerminalCompiler");
    }

    #[test]
    fn compile_plan_audio() {
        let scene = SceneGraph::new();
        let plan = RenderPlan::new(scene, GrammarExpr::new("data", GeometryType::Point));
        let out = AudioCompiler::new().compile_plan(&plan);
        match &out {
            ModalityOutput::AudioParams(p) => assert_eq!(p.len(), 0),
            _ => panic!("expected AudioParams"),
        }
    }

    #[test]
    fn compile_plan_braille() {
        let scene = SceneGraph::new();
        let plan = RenderPlan::new(scene, GrammarExpr::new("data", GeometryType::Point));
        let out = BrailleCompiler::new(40, 12).compile_plan(&plan);
        match &out {
            ModalityOutput::BrailleCells(g) => {
                assert_eq!(g.len(), 12);
                assert!(g[0].len() <= 40);
            }
            _ => panic!("expected BrailleCells"),
        }
    }

    #[test]
    fn compile_plan_description() {
        let scene = SceneGraph::new();
        let plan = RenderPlan::new(scene, GrammarExpr::new("data", GeometryType::Point));
        let out = DescriptionCompiler::new().compile_plan(&plan);
        match &out {
            ModalityOutput::Description(b) => assert!(!b.is_empty()),
            _ => panic!("expected Description"),
        }
    }

    #[test]
    fn compile_plan_haptic() {
        let scene = SceneGraph::new();
        let plan = RenderPlan::new(scene, GrammarExpr::new("data", GeometryType::Point));
        let out = HapticCompiler::new().compile_plan(&plan);
        match &out {
            ModalityOutput::HapticCommands(c) => assert_eq!(c.len(), 0),
            _ => panic!("expected HapticCommands"),
        }
    }

    #[test]
    fn compile_plan_svg() {
        let scene = SceneGraph::new();
        let plan = RenderPlan::new(scene, GrammarExpr::new("data", GeometryType::Point));
        let out = SvgCompiler::new().compile_plan(&plan);
        match &out {
            ModalityOutput::Svg(b) => {
                assert!(std::str::from_utf8(b.as_ref()).unwrap().contains("<svg"));
            }
            _ => panic!("expected Svg"),
        }
    }

    #[test]
    fn compile_plan_terminal() {
        let scene = SceneGraph::new();
        let plan = RenderPlan::new(scene, GrammarExpr::new("data", GeometryType::Point));
        let out = TerminalCompiler::new(80, 24).compile_plan(&plan);
        match &out {
            ModalityOutput::TerminalCells(g) => {
                assert_eq!(g.len(), 24);
                assert_eq!(g[0].len(), 80);
            }
            _ => panic!("expected TerminalCells"),
        }
    }
}
