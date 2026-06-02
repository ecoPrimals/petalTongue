// SPDX-License-Identifier: AGPL-3.0-or-later
//! Output rendering: terminal, SVG, JSON, DOT, PNG, and HTML export modes.

use crate::args::Args;
use crate::error::HeadlessError;
use petal_tongue_core::GraphEngine;
use petal_tongue_ui_core::{
    CanvasUI, ExportFormat, SvgUI, TerminalUI, TextUI, UniversalUI,
};
use std::path::Path;
use std::sync::{Arc, RwLock};

/// Render terminal UI
pub fn render_terminal(graph: Arc<RwLock<GraphEngine>>) -> Result<(), HeadlessError> {
    let ui = TerminalUI::new(graph);
    let output = ui.render_to_string()?;
    println!("{output}");
    Ok(())
}

/// Render SVG
pub fn render_svg(
    graph: Arc<RwLock<GraphEngine>>,
    args: &Args,
) -> Result<(), HeadlessError> {
    let ui = SvgUI::new(graph, args.width, args.height);

    if let Some(ref output) = args.output {
        ui.export(Path::new(output), ExportFormat::Svg)?;
        tracing::info!("✅ Exported to {}", output);
    } else {
        let svg = ui.render_to_string()?;
        println!("{svg}");
    }

    Ok(())
}

/// Render JSON
pub fn render_json(
    graph: Arc<RwLock<GraphEngine>>,
    args: &Args,
) -> Result<(), HeadlessError> {
    let ui = TextUI::new(graph).with_format(ExportFormat::Json);

    if let Some(ref output) = args.output {
        ui.export(Path::new(output), ExportFormat::Json)?;
        tracing::info!("✅ Exported to {}", output);
    } else {
        let json = ui.render_to_string()?;
        println!("{json}");
    }

    Ok(())
}

/// Render DOT
pub fn render_dot(
    graph: Arc<RwLock<GraphEngine>>,
    args: &Args,
) -> Result<(), HeadlessError> {
    let ui = TextUI::new(graph).with_format(ExportFormat::Dot);

    if let Some(ref output) = args.output {
        ui.export(Path::new(output), ExportFormat::Dot)?;
        tracing::info!("✅ Exported to {}", output);
    } else {
        let dot = ui.render_to_string()?;
        println!("{dot}");
    }

    Ok(())
}

/// Render PNG
pub fn render_png(
    graph: Arc<RwLock<GraphEngine>>,
    args: &Args,
) -> Result<(), HeadlessError> {
    let ui = CanvasUI::new(graph, args.width, args.height);

    if let Some(ref output) = args.output {
        ui.export(Path::new(output), ExportFormat::Png)?;
        tracing::info!("✅ Exported to {}", output);
    } else {
        tracing::error!("PNG mode requires --output option");
        std::process::exit(1);
    }

    Ok(())
}

/// Render HTML (SVG wrapped in a standalone HTML document) (PT-04)
pub fn render_html(
    graph: Arc<RwLock<GraphEngine>>,
    args: &Args,
) -> Result<(), HeadlessError> {
    let ui = SvgUI::new(graph, args.width, args.height);

    if let Some(ref output) = args.output {
        ui.export(Path::new(output), ExportFormat::Html)?;
        tracing::info!("✅ Exported to {}", output);
    } else {
        let svg = ui.render_to_string()?;
        let html =
            String::from_utf8(petal_tongue_ui_core::wrap_svg_in_html(&svg)).unwrap_or_default();
        println!("{html}");
    }

    Ok(())
}
