// SPDX-License-Identifier: AGPL-3.0-only
#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
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

mod error;

use crate::error::HeadlessError;
use petal_tongue_core::GraphEngine;
use petal_tongue_core::constants;
use petal_tongue_ui_core::{
    CanvasUI, ExportFormat, SvgUI, TerminalUI, TextUI, UIMode, UniversalUI, detect_best_ui_mode,
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
                                eprintln!("Unknown mode: {m}");
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
                    eprintln!("Unknown argument: {arg}");
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
        r"
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
"
    );
}

fn main() -> Result<(), HeadlessError> {
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
fn load_graph_data(graph: &Arc<RwLock<GraphEngine>>) -> Result<(), HeadlessError> {
    use petal_tongue_core::{PrimalHealthStatus, PrimalInfo, TopologyEdge};

    // For now, create a simple example topology
    tracing::info!("📚 Loading demonstration topology");

    let mut g = graph.write()?;

    // Create some example primals
    let primals = vec![
        PrimalInfo::new(
            "petaltongue-headless",
            "petalTongue Headless",
            "Visualization",
            constants::default_headless_url(),
            vec!["visualization".to_string(), "export".to_string()],
            PrimalHealthStatus::Healthy,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        ),
        PrimalInfo::new(
            std::env::var("PETALTONGUE_HEADLESS_DEMO_HEALTH_ID")
                .unwrap_or_else(|_| "health-monitor-1".to_string()),
            std::env::var("PETALTONGUE_HEADLESS_DEMO_HEALTH_NAME")
                .unwrap_or_else(|_| "Health Monitor".to_string()),
            "Health Monitoring",
            constants::default_web_url(),
            vec!["health".to_string(), "monitoring".to_string()],
            PrimalHealthStatus::Healthy,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        ),
        PrimalInfo::new(
            "encryption-demo-1",
            "Encryption Primal",
            "Encrypted Communication",
            constants::default_sandbox_security_url(),
            vec!["encryption".to_string(), "messaging".to_string()],
            PrimalHealthStatus::Warning,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        ),
    ];

    // Add primals to graph
    for primal in primals {
        g.add_node(primal);
    }

    // Add some connections (use same env-driven IDs for edges)
    let health_id = std::env::var("PETALTONGUE_HEADLESS_DEMO_HEALTH_ID")
        .unwrap_or_else(|_| "health-monitor-1".to_string());
    g.add_edge(TopologyEdge {
        from: health_id.into(),
        to: "petaltongue-headless".into(),
        edge_type: "monitors".to_string(),
        label: Some("Health Monitoring".to_string()),
        capability: None,
        metrics: None,
    });
    g.add_edge(TopologyEdge {
        from: "encryption-demo-1".into(),
        to: "petaltongue-headless".into(),
        edge_type: "sends_data".to_string(),
        label: Some("Encrypted Messages".to_string()),
        capability: None,
        metrics: None,
    });

    // Apply layout (10 iterations for force-directed layout)
    g.layout(10);

    let node_count = g.nodes().len();
    let edge_count = g.edges().len();
    tracing::info!(
        "📊 Loaded: {} primals, {} connections",
        node_count,
        edge_count
    );

    Ok(())
}

/// Render terminal UI
fn render_terminal(graph: Arc<RwLock<GraphEngine>>) -> Result<(), HeadlessError> {
    let ui = TerminalUI::new(graph);
    let output = ui.render_to_string()?;
    println!("{output}");
    Ok(())
}

/// Render SVG
fn render_svg(graph: Arc<RwLock<GraphEngine>>, args: &Args) -> Result<(), HeadlessError> {
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
fn render_json(graph: Arc<RwLock<GraphEngine>>, args: &Args) -> Result<(), HeadlessError> {
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
fn render_dot(graph: Arc<RwLock<GraphEngine>>, args: &Args) -> Result<(), HeadlessError> {
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
fn render_png(graph: Arc<RwLock<GraphEngine>>, args: &Args) -> Result<(), HeadlessError> {
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
