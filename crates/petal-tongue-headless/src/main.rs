// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! Headless petalTongue - Pure Rust interface (no display dependencies)
//!
//! This binary demonstrates petalTongue's self-sovereignty:
//! - Zero display dependencies
//! - Works on servers, containers, CI/CD
//! - Exports to multiple formats
//! - Runs over SSH
//!
//! # Philosophy
//!
//! External display systems (egui) are enhancements, not dependencies.
//! This binary proves petalTongue can run anywhere Rust runs.

mod args;
mod baselines;
mod error;
mod graph_loader;
mod render;

use crate::args::{Args, OutputMode};
use crate::error::HeadlessError;
use petal_tongue_core::GraphEngine;
use petal_tongue_ui_core::{UIMode, detect_best_ui_mode};
use std::sync::{Arc, RwLock};

fn main() -> Result<(), HeadlessError> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    tracing::info!("🌸 petalTongue Headless - Pure Rust UI");
    tracing::info!("Zero display dependencies. Universal representation system.");

    // Parse arguments
    let args = Args::parse();

    // Create graph
    let graph = Arc::new(RwLock::new(GraphEngine::new()));

    graph_loader::load_graph_data(&graph, &args)?;

    // Determine UI mode
    let ui_mode = match args.mode {
        OutputMode::Auto => detect_best_ui_mode(),
        OutputMode::Terminal => UIMode::Terminal,
        _ => UIMode::Headless,
    };

    tracing::info!("UI Mode: {:?}", ui_mode);

    // Render based on mode
    match args.mode {
        OutputMode::Auto => match ui_mode {
            UIMode::Terminal => render::render_terminal(graph)?,
            _ => render::render_svg(graph, &args)?,
        },
        OutputMode::Terminal => render::render_terminal(graph)?,
        OutputMode::Svg => render::render_svg(graph, &args)?,
        OutputMode::Json => render::render_json(graph, &args)?,
        OutputMode::Dot => render::render_dot(graph, &args)?,
        OutputMode::Png => render::render_png(graph, &args)?,
        OutputMode::Html => render::render_html(graph, &args)?,
        OutputMode::Baselines => baselines::render_baselines(&args)?,
    }

    Ok(())
}
