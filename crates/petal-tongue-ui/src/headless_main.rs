// SPDX-License-Identifier: AGPL-3.0-only
//! Headless petalTongue - Pure Rust UI (no GUI dependencies)
//!
//! This binary demonstrates petalTongue's self-sovereignty:
//! - Zero GUI dependencies
//! - Works on servers, containers, CI/CD
//! - Exports to multiple formats
//! - Runs over SSH
//!
//! # Philosophy
//!
//! External systems (egui) are enhancements, not dependencies.
//! This binary proves petalTongue can run anywhere Rust runs.

use anyhow::Result;
use petal_tongue_core::GraphEngine;
use petal_tongue_ui_core::{
    detect_best_ui_mode, ExportFormat, SvgUI, TerminalUI, TextUI, UIMode, UniversalUI,
};
use std::path::Path;
use std::sync::{Arc, RwLock};

/// Command-line arguments
#[derive(Debug)]
struct Args {
    mode: OutputMode,
    output: Option<String>,
    width: u32,
    height: u32,
}

#[derive(Debug, Clone, Copy)]
enum OutputMode {
    /// Auto-detect best mode
    Auto,
    /// Terminal output (stdout)
    Terminal,
    /// SVG export
    Svg,
    /// JSON export
    Json,
    /// DOT export (graphviz)
    Dot,
    /// PNG export
    Png,
}

impl Args {
    fn parse() -> Self {
        let mut args = std::env::args().skip(1);
        let mut mode = OutputMode::Auto;
        let mut output = None;
        let mut width = 1920;
        let mut height = 1080;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--mode" | "-m" => {
                    if let Some(m) = args.next() {
                        mode = match m.as_str() {
                            "auto" => OutputMode::Auto,
                            "terminal" | "tui" => OutputMode::Terminal,
                            "svg" => OutputMode::Svg,
                            "json" => OutputMode::Json,
                            "dot" => OutputMode::Dot,
                            "png" => OutputMode::Png,
                            _ => {
                                eprintln!("Unknown mode: {}", m);
                                std::process::exit(1);
                            }
                        }
                    }
                }
                "--output" | "-o" => {
                    output = args.next();
                }
                "--width" | "-w" => {
                    if let Some(w) = args.next() {
                        width = w.parse().unwrap_or(1920);
                    }
                }
                "--height" | "-h" => {
                    if let Some(h) = args.next() {
                        height = h.parse().unwrap_or(1080);
                    }
                }
                "--help" => {
                    print_help();
                    std::process::exit(0);
                }
                _ => {
                    eprintln!("Unknown argument: {}", arg);
                    print_help();
                    std::process::exit(1);
                }
            }
        }

        Self {
            mode,
            output,
            width,
            height,
        }
    }
}

fn print_help() {
    println!(
        r#"
petalTongue Headless - Pure Rust UI

USAGE:
    petal-tongue-headless [OPTIONS]

OPTIONS:
    -m, --mode <MODE>       Output mode [auto, terminal, svg, json, dot, png]
    -o, --output <FILE>     Output file (required for export modes)
    -w, --width <WIDTH>     Width in pixels (default: 1920)
    -h, --height <HEIGHT>   Height in pixels (default: 1080)
    --help                  Show this help message

MODES:
    auto        Auto-detect best mode for environment
    terminal    Terminal UI (ASCII art, works over SSH)
    svg         Export to SVG (browser-friendly)
    json        Export to JSON (API-friendly)
    dot         Export to DOT (graphviz-friendly)
    png         Export to PNG (report-friendly)

EXAMPLES:
    # Auto-detect mode (terminal if available)
    petal-tongue-headless

    # Terminal mode
    petal-tongue-headless --mode terminal

    # Export to SVG
    petal-tongue-headless --mode svg --output topology.svg

    # Export to JSON
    petal-tongue-headless --mode json --output topology.json

    # Over SSH
    ssh server petal-tongue-headless --mode terminal

ENVIRONMENT:
    SHOWCASE_MODE=true      Use tutorial data
    HEADLESS=true           Force headless mode
    PETALTONGUE_HEADLESS=1  Force headless mode

PHILOSOPHY:
    This binary proves petalTongue's self-sovereignty.
    Zero GUI dependencies. Works everywhere Rust runs.
    External systems (egui) are enhancements, not dependencies.
"#
    );
}

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    tracing::info!("🌸 petalTongue Headless - Pure Rust UI");
    tracing::info!("Zero GUI dependencies. Universal representation system.");

    // Parse arguments
    let args = Args::parse();

    // Create graph
    let graph = Arc::new(RwLock::new(GraphEngine::new()));

    // Load data (tutorial mode or discovery)
    load_graph_data(&graph)?;

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
            UIMode::Terminal => render_terminal(graph)?,
            _ => render_svg(graph, &args)?,
        },
        OutputMode::Terminal => render_terminal(graph)?,
        OutputMode::Svg => render_svg(graph, &args)?,
        OutputMode::Json => render_json(graph, &args)?,
        OutputMode::Dot => render_dot(graph, &args)?,
        OutputMode::Png => render_png(graph, &args)?,
    }

    Ok(())
}

/// Load graph data (tutorial mode or discovery)
fn load_graph_data(graph: &Arc<RwLock<GraphEngine>>) -> Result<()> {
    // Check if tutorial mode
    let tutorial_mode = std::env::var("SHOWCASE_MODE").unwrap_or_default() == "true";

    if tutorial_mode {
        tracing::info!("📚 Tutorial mode: Loading demonstration data");
        
        // Use tutorial mode to load data
        let tutorial = crate::tutorial_mode::TutorialMode::new();
        if tutorial.is_enabled() {
            tutorial.load_scenario_into_graph(graph)?;
            tracing::info!("✅ Tutorial scenario loaded");
        }
    } else {
        tracing::info!("🔍 Production mode: Discovering primals...");
        
        // TODO: Implement actual discovery (delegated to discovery provider)
        // For now, use tutorial as fallback
        let tutorial = crate::tutorial_mode::TutorialMode::new();
        if tutorial.should_fallback(0) {
            tracing::info!("💡 Using tutorial data as graceful fallback");
            tutorial.load_scenario_into_graph(graph)?;
        }
    }

    let node_count = graph.read().unwrap().nodes().len();
    let edge_count = graph.read().unwrap().edges().len();
    tracing::info!("📊 Loaded: {} primals, {} connections", node_count, edge_count);

    Ok(())
}

/// Render terminal UI
fn render_terminal(graph: Arc<RwLock<GraphEngine>>) -> Result<()> {
    let ui = TerminalUI::new(graph);
    let output = ui.render_to_string()?;
    println!("{}", output);
    Ok(())
}

/// Render SVG
fn render_svg(graph: Arc<RwLock<GraphEngine>>, args: &Args) -> Result<()> {
    let ui = SvgUI::new(graph, args.width, args.height);
    
    if let Some(ref output) = args.output {
        ui.export(Path::new(output), ExportFormat::Svg)?;
        tracing::info!("✅ Exported to {}", output);
    } else {
        let svg = ui.render_to_string()?;
        println!("{}", svg);
    }
    
    Ok(())
}

/// Render JSON
fn render_json(graph: Arc<RwLock<GraphEngine>>, args: &Args) -> Result<()> {
    let ui = TextUI::new(graph).with_format(ExportFormat::Json);
    
    if let Some(ref output) = args.output {
        ui.export(Path::new(output), ExportFormat::Json)?;
        tracing::info!("✅ Exported to {}", output);
    } else {
        let json = ui.render_to_string()?;
        println!("{}", json);
    }
    
    Ok(())
}

/// Render DOT
fn render_dot(graph: Arc<RwLock<GraphEngine>>, args: &Args) -> Result<()> {
    let ui = TextUI::new(graph).with_format(ExportFormat::Dot);
    
    if let Some(ref output) = args.output {
        ui.export(Path::new(output), ExportFormat::Dot)?;
        tracing::info!("✅ Exported to {}", output);
    } else {
        let dot = ui.render_to_string()?;
        println!("{}", dot);
    }
    
    Ok(())
}

/// Render PNG
fn render_png(graph: Arc<RwLock<GraphEngine>>, args: &Args) -> Result<()> {
    use petal_tongue_ui_core::CanvasUI;
    
    let ui = CanvasUI::new(graph, args.width, args.height);
    
    if let Some(ref output) = args.output {
        ui.export(Path::new(output), ExportFormat::Png)?;
        tracing::info!("✅ Exported to {}", output);
    } else {
        eprintln!("Error: PNG mode requires --output option");
        std::process::exit(1);
    }
    
    Ok(())
}

