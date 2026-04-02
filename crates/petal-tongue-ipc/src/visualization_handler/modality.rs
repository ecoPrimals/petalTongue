// SPDX-License-Identifier: AGPL-3.0-or-later
//! Modality compilation: scene graph → output format.
//!
//! Supports SVG, audio, description, haptic, GPU command, braille, and terminal.
//! Used by `VisualizationState` for grammar render and export.

use petal_tongue_core::DataBinding;
use petal_tongue_scene::GpuCompiler;
use petal_tongue_scene::modality::{
    AudioCompiler, BrailleCompiler, DescriptionCompiler, HapticCompiler, ModalityCompiler,
    ModalityOutput, SvgCompiler, TerminalCompiler,
};
use petal_tongue_scene::scene_graph::SceneGraph;
use tracing::warn;

/// Compile a scene graph to SVG and return the raw SVG string.
fn compile_svg(scene: &SceneGraph) -> (serde_json::Value, String) {
    let compiler = SvgCompiler;
    match compiler.compile(scene) {
        ModalityOutput::Svg(b) => (
            serde_json::Value::String(String::from_utf8_lossy(b.as_ref()).into_owned()),
            "svg".into(),
        ),
        _ => (serde_json::Value::Null, "error".into()),
    }
}

/// Compile a scene graph to an HTML document wrapping SVG (PT-04).
fn compile_html(scene: &SceneGraph) -> (serde_json::Value, String) {
    let compiler = SvgCompiler;
    match compiler.compile(scene) {
        ModalityOutput::Svg(b) => {
            let svg = String::from_utf8_lossy(b.as_ref());
            let html = format!(
                "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\
                 <meta charset=\"utf-8\">\
                 <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
                 <title>petalTongue Export</title>\
                 <style>body{{margin:0;display:flex;justify-content:center;\
                 align-items:center;min-height:100vh;background:#1a1a2e}}\
                 svg{{max-width:100%;height:auto}}</style>\
                 </head>\n<body>\n{svg}\n</body>\n</html>"
            );
            (serde_json::Value::String(html), "html".into())
        }
        _ => (serde_json::Value::Null, "error".into()),
    }
}

/// Compile a scene graph to the requested output modality.
pub(super) fn compile_modality(scene: &SceneGraph, modality: &str) -> (serde_json::Value, String) {
    match modality {
        "svg" => compile_svg(scene),
        "html" => compile_html(scene),
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
                ModalityOutput::Description(b) => (
                    serde_json::Value::String(String::from_utf8_lossy(b.as_ref()).into_owned()),
                    "description".into(),
                ),
                _ => (serde_json::Value::Null, "error".into()),
            }
        }
        "haptic" => {
            let compiler = HapticCompiler;
            match compiler.compile(scene) {
                ModalityOutput::HapticCommands(cmds) => {
                    let v = serde_json::to_value(&cmds).unwrap_or(serde_json::Value::Null);
                    (v, "haptic".into())
                }
                _ => (serde_json::Value::Null, "error".into()),
            }
        }
        "gpu" => {
            let compiler = GpuCompiler::new(1920.0, 1080.0);
            match compiler.compile(scene) {
                ModalityOutput::GpuCommands(b) => (
                    serde_json::Value::String(String::from_utf8_lossy(b.as_ref()).into_owned()),
                    "gpu".into(),
                ),
                _ => (serde_json::Value::Null, "error".into()),
            }
        }
        "braille" => {
            let compiler = BrailleCompiler::new(80, 25);
            match compiler.compile(scene) {
                ModalityOutput::BrailleCells(cells) => {
                    let v = serde_json::to_value(&cells).unwrap_or(serde_json::Value::Null);
                    (v, "braille".into())
                }
                _ => (serde_json::Value::Null, "error".into()),
            }
        }
        "terminal" => {
            let compiler = TerminalCompiler::new(120, 40);
            match compiler.compile(scene) {
                ModalityOutput::TerminalCells(cells) => {
                    let grid: Vec<String> = cells.iter().map(|row| row.iter().collect()).collect();
                    (
                        serde_json::to_value(&grid).unwrap_or(serde_json::Value::Null),
                        "terminal".into(),
                    )
                }
                _ => (serde_json::Value::Null, "error".into()),
            }
        }
        other => {
            warn!("Unknown modality '{other}', falling back to SVG");
            compile_svg(scene)
        }
    }
}

/// Compile a `DataBinding` directly to the requested modality output.
///
/// For `GameScene` and `Soundscape` bindings, produces rich semantic output
/// (descriptions, sonification, haptics) that the generic SceneGraph path
/// cannot provide. For other binding types, delegates to `compile_modality`.
pub(super) fn compile_binding_modality(
    binding: &DataBinding,
    scene: &SceneGraph,
    modality: &str,
) -> (serde_json::Value, String) {
    match (binding, modality) {
        (
            DataBinding::GameScene { .. } | DataBinding::Soundscape { .. },
            "description" | "accessibility",
        ) => {
            let desc = petal_tongue_scene::describe_binding(binding);
            (serde_json::Value::String(desc), "description".into())
        }
        (DataBinding::GameScene { scene: json, .. }, "audio") => {
            let params = petal_tongue_scene::sonify_game_scene(json);
            let value = serde_json::to_value(&params).unwrap_or(serde_json::Value::Null);
            (value, "audio".into())
        }
        (DataBinding::GameScene { scene: json, .. }, "haptic") => {
            let cmds = petal_tongue_scene::hapticize_game_scene(json);
            let value = serde_json::to_value(&cmds).unwrap_or(serde_json::Value::Null);
            (value, "haptic".into())
        }
        (DataBinding::Soundscape { definition, .. }, "haptic") => {
            let cmds = petal_tongue_scene::hapticize_soundscape(definition);
            let value = serde_json::to_value(&cmds).unwrap_or(serde_json::Value::Null);
            (value, "haptic".into())
        }
        _ => compile_modality(scene, modality),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_scene() -> SceneGraph {
        SceneGraph::new()
    }

    fn game_scene_binding() -> DataBinding {
        DataBinding::GameScene {
            id: "test".into(),
            label: "Arena".into(),
            scene: serde_json::json!({
                "entities": [
                    {"entity_type": "player", "position": [10.0, 10.0], "health": 0.9, "label": "Hero"},
                    {"entity_type": "enemy", "position": [40.0, 10.0], "health": 0.3, "label": "Goblin"}
                ],
                "camera_center": [20.0, 10.0]
            }),
        }
    }

    fn soundscape_binding() -> DataBinding {
        DataBinding::Soundscape {
            id: "test".into(),
            label: "Forest".into(),
            definition: serde_json::json!({
                "name": "Forest",
                "duration_secs": 10.0,
                "layers": [{"id": "wind", "waveform": "white_noise", "frequency": 200.0, "amplitude": 0.3, "pan": -0.5}]
            }),
        }
    }

    #[test]
    fn compile_modality_svg() {
        let scene = minimal_scene();
        let (output, format) = compile_modality(&scene, "svg");
        assert_eq!(format, "svg");
        assert!(output.is_string());
        let s = output.as_str().unwrap();
        assert!(s.contains("<svg"));
    }

    #[test]
    fn compile_modality_audio() {
        let scene = minimal_scene();
        let (output, format) = compile_modality(&scene, "audio");
        assert_eq!(format, "audio");
        assert!(output.is_array());
    }

    #[test]
    fn compile_modality_description() {
        let scene = minimal_scene();
        let (output, format) = compile_modality(&scene, "description");
        assert_eq!(format, "description");
        assert!(output.is_string());
    }

    #[test]
    fn compile_modality_accessibility_alias() {
        let scene = minimal_scene();
        let (output, format) = compile_modality(&scene, "accessibility");
        assert_eq!(format, "description");
        assert!(output.is_string());
    }

    #[test]
    fn compile_modality_haptic() {
        let scene = minimal_scene();
        let (output, format) = compile_modality(&scene, "haptic");
        assert_eq!(format, "haptic");
        assert!(output.is_array());
    }

    #[test]
    fn compile_modality_gpu() {
        let scene = minimal_scene();
        let (output, format) = compile_modality(&scene, "gpu");
        assert_eq!(format, "gpu");
        assert!(output.is_string());
    }

    #[test]
    fn compile_modality_braille() {
        let scene = minimal_scene();
        let (output, format) = compile_modality(&scene, "braille");
        assert_eq!(format, "braille");
        assert!(output.is_array());
    }

    #[test]
    fn compile_modality_terminal() {
        let scene = minimal_scene();
        let (output, format) = compile_modality(&scene, "terminal");
        assert_eq!(format, "terminal");
        assert!(output.is_array());
    }

    #[test]
    fn compile_modality_unknown_fallback_svg() {
        let scene = minimal_scene();
        let (output, format) = compile_modality(&scene, "unknown-modality");
        assert_eq!(format, "svg");
        assert!(output.is_string());
        let s = output.as_str().unwrap();
        assert!(s.contains("<svg"));
    }

    #[test]
    fn compile_modality_empty_string_fallback() {
        let scene = minimal_scene();
        let (output, format) = compile_modality(&scene, "");
        assert_eq!(format, "svg");
        assert!(output.is_string());
    }

    #[test]
    fn compile_modality_html_wraps_svg() {
        let scene = minimal_scene();
        let (output, format) = compile_modality(&scene, "html");
        assert_eq!(format, "html");
        assert!(output.is_string());
        let s = output.as_str().unwrap();
        assert!(
            s.contains("<!DOCTYPE html>"),
            "should be full HTML document"
        );
        assert!(s.contains("<svg"), "should contain embedded SVG");
        assert!(s.contains("</html>"), "should close HTML");
    }

    #[test]
    fn binding_modality_game_scene_description() {
        let binding = game_scene_binding();
        let scene = minimal_scene();
        let (output, format) = compile_binding_modality(&binding, &scene, "description");
        assert_eq!(format, "description");
        let desc = output.as_str().unwrap();
        assert!(desc.contains("Hero"), "should mention entities");
        assert!(desc.contains("Goblin"), "should mention enemies");
        assert!(desc.contains("health"), "should report health");
    }

    #[test]
    fn binding_modality_game_scene_audio() {
        let binding = game_scene_binding();
        let scene = minimal_scene();
        let (output, format) = compile_binding_modality(&binding, &scene, "audio");
        assert_eq!(format, "audio");
        let params = output.as_array().unwrap();
        assert_eq!(params.len(), 2, "one tone per entity");
    }

    #[test]
    fn binding_modality_game_scene_haptic() {
        let binding = game_scene_binding();
        let scene = minimal_scene();
        let (output, format) = compile_binding_modality(&binding, &scene, "haptic");
        assert_eq!(format, "haptic");
        let cmds = output.as_array().unwrap();
        assert_eq!(cmds.len(), 2, "one pulse per entity");
    }

    #[test]
    fn binding_modality_soundscape_description() {
        let binding = soundscape_binding();
        let scene = minimal_scene();
        let (output, format) = compile_binding_modality(&binding, &scene, "description");
        assert_eq!(format, "description");
        let desc = output.as_str().unwrap();
        assert!(desc.contains("Forest"));
        assert!(desc.contains("wind"));
    }

    #[test]
    fn binding_modality_soundscape_haptic() {
        let binding = soundscape_binding();
        let scene = minimal_scene();
        let (output, format) = compile_binding_modality(&binding, &scene, "haptic");
        assert_eq!(format, "haptic");
        let cmds = output.as_array().unwrap();
        assert_eq!(cmds.len(), 1, "one haptic per layer");
    }

    #[test]
    fn binding_modality_timeseries_falls_through() {
        let binding = DataBinding::TimeSeries {
            id: "t".into(),
            label: "T".into(),
            x_label: String::new(),
            y_label: String::new(),
            unit: String::new(),
            x_values: vec![0.0],
            y_values: vec![1.0],
        };
        let scene = minimal_scene();
        let (output, format) = compile_binding_modality(&binding, &scene, "svg");
        assert_eq!(
            format, "svg",
            "non-GameScene/Soundscape should fall through"
        );
        assert!(output.as_str().unwrap().contains("<svg"));
    }
}
