// SPDX-License-Identifier: AGPL-3.0-only
//! Modality compilation: scene graph → output format (SVG, audio, description).
//!
//! Used by VisualizationState for grammar render and export.

use petal_tongue_scene::modality::{
    AudioCompiler, DescriptionCompiler, ModalityCompiler, ModalityOutput, SvgCompiler,
};
use petal_tongue_scene::scene_graph::SceneGraph;
use tracing::warn;

/// Compile a scene graph to the requested output modality.
pub(super) fn compile_modality(scene: &SceneGraph, modality: &str) -> (serde_json::Value, String) {
    match modality {
        "svg" => {
            let compiler = SvgCompiler;
            match compiler.compile(scene) {
                ModalityOutput::Svg(s) => (serde_json::Value::String(s), "svg".into()),
                _ => (serde_json::Value::Null, "error".into()),
            }
        }
        "audio" => {
            let compiler = AudioCompiler;
            match compiler.compile(scene) {
                ModalityOutput::AudioParams(params) => {
                    let v = serde_json::to_value(&params).unwrap_or(serde_json::Value::Null);
                    (v, "audio".into())
                }
                _ => (serde_json::Value::Null, "error".into()),
            }
        }
        "description" | "accessibility" => {
            let compiler = DescriptionCompiler;
            match compiler.compile(scene) {
                ModalityOutput::Description(s) => {
                    (serde_json::Value::String(s), "description".into())
                }
                _ => (serde_json::Value::Null, "error".into()),
            }
        }
        other => {
            warn!("Unknown modality '{other}', falling back to SVG");
            let compiler = SvgCompiler;
            match compiler.compile(scene) {
                ModalityOutput::Svg(s) => (serde_json::Value::String(s), "svg".into()),
                _ => (serde_json::Value::Null, "error".into()),
            }
        }
    }
}
