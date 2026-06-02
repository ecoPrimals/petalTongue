// SPDX-License-Identifier: AGPL-3.0-or-later
//! Scene graph compilation to output modalities (SVG, description, terminal).

use petal_tongue_scene::modality::svg::SvgCompiler;
use petal_tongue_scene::modality::{ModalityCompiler, ModalityOutput};
use petal_tongue_scene::scene_graph::SceneGraph;
use petal_tongue_scene::tufte::TufteConstraintImpl;

pub fn all_tufte_constraints() -> Vec<TufteConstraintImpl> {
    vec![
        TufteConstraintImpl::DataInkRatio,
        TufteConstraintImpl::LieFactor,
        TufteConstraintImpl::ChartjunkDetection,
        TufteConstraintImpl::ColorAccessibility,
        TufteConstraintImpl::DataDensity,
        TufteConstraintImpl::SmallestEffectiveDifference,
        TufteConstraintImpl::SmallMultiplesPreference,
    ]
}

pub fn scene_to_svg(scene: &SceneGraph) -> String {
    let compiler = SvgCompiler::new();
    match compiler.compile(scene) {
        ModalityOutput::Svg(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        _ => "Error: SVG compilation produced unexpected modality".to_owned(),
    }
}

pub fn compile_scene_to_modality(scene: &SceneGraph, modality: &str) -> String {
    match modality {
        "svg" | "" => scene_to_svg(scene),
        "description" => {
            let compiler = petal_tongue_scene::modality::description::DescriptionCompiler::new();
            match compiler.compile(scene) {
                ModalityOutput::Description(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
                _ => "Error: description compilation produced unexpected modality".to_owned(),
            }
        }
        "terminal" => {
            let compiler = petal_tongue_scene::modality::terminal::TerminalCompiler::new(80, 24);
            match compiler.compile(scene) {
                ModalityOutput::TerminalCells(cells) => cells
                    .iter()
                    .map(|row| row.iter().collect::<String>())
                    .collect::<Vec<_>>()
                    .join("\n"),
                _ => "Error: terminal compilation produced unexpected modality".to_owned(),
            }
        }
        other => format!("Error: unsupported modality: {other}"),
    }
}
