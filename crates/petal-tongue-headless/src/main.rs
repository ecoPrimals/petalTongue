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
    /// HTML export (SVG wrapped in standalone HTML document) (PT-04)
    Html,
    /// Export all baseline DataBindings as individual SVGs for validation
    Baselines,
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
                            "html" => OutputMode::Html,
                            "baselines" => OutputMode::Baselines,
                            _ => {
                                tracing::error!("Unknown mode: {m}");
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
                    tracing::error!("Unknown argument: {arg}");
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
    -m, --mode <MODE>       Output mode [auto, terminal, svg, json, dot, png, html, baselines]
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
    html        Export to HTML (browser-friendly, standalone)
    baselines   Export all baseline DataBindings as individual SVGs

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
    Zero display dependencies. Works everywhere Rust runs.
    External systems (egui) are enhancements, not dependencies.
"
    );
}

fn main() -> Result<(), HeadlessError> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    tracing::info!("🌸 petalTongue Headless - Pure Rust UI");
    tracing::info!("Zero display dependencies. Universal representation system.");

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
        OutputMode::Html => render_html(graph, &args)?,
        OutputMode::Baselines => render_baselines(&args)?,
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
    drop(g);
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
        tracing::error!("PNG mode requires --output option");
        std::process::exit(1);
    }

    Ok(())
}

/// Render HTML (SVG wrapped in a standalone HTML document) (PT-04)
fn render_html(graph: Arc<RwLock<GraphEngine>>, args: &Args) -> Result<(), HeadlessError> {
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

/// Export all baseline DataBindings as individual SVGs for human review.
fn render_baselines(args: &Args) -> Result<(), HeadlessError> {
    use petal_tongue_scene::compiler::GrammarCompiler;
    use petal_tongue_scene::data_binding::DataBindingCompiler;
    use petal_tongue_scene::modality::{ModalityCompiler, ModalityOutput, SvgCompiler};

    let output_dir = args
        .output
        .as_deref()
        .unwrap_or("baselines_svg");
    let out_path = Path::new(output_dir);
    std::fs::create_dir_all(out_path).map_err(|e| HeadlessError::IoError(e.to_string()))?;

    let bindings = baseline_bindings();
    let compiler = GrammarCompiler::new();
    let svg_compiler = SvgCompiler::new();

    let mut success_count = 0u32;
    let mut error_count = 0u32;

    for (name, binding) in &bindings {
        let (expr, data) = DataBindingCompiler::compile(binding, Some("genomics"));
        let scene = compiler.compile(&expr, &data);

        if scene.total_primitives() == 0 {
            tracing::warn!("  {name}: empty scene (0 primitives), skipping");
            error_count += 1;
            continue;
        }

        let output = svg_compiler.compile(&scene);
        let svg_bytes = match output {
            ModalityOutput::Svg(bytes) => bytes,
            _ => {
                tracing::warn!("  {name}: unexpected modality output");
                error_count += 1;
                continue;
            }
        };

        let file_name = format!("{name}.svg");
        let file_path = out_path.join(&file_name);
        std::fs::write(&file_path, &svg_bytes)
            .map_err(|e| HeadlessError::IoError(e.to_string()))?;

        tracing::info!(
            "  {name}: {} primitives -> {}",
            scene.total_primitives(),
            file_path.display()
        );
        success_count += 1;
    }

    println!(
        "Baselines export: {success_count} SVGs exported, {error_count} errors -> {}",
        out_path.display()
    );

    Ok(())
}

/// All baseline DataBindings for Barrick Lab tools validation.
fn baseline_bindings() -> Vec<(&'static str, petal_tongue_core::DataBinding)> {
    use petal_tongue_core::DataBinding;
    use serde_json::json;

    vec![
        ("breseq_genome_track", DataBinding::GenomeTrack {
            id: "bl_breseq_genome".into(), label: "breseq Genome Track".into(),
            sequence_length: 4_629_812.0,
            tracks: vec!["SNP".into(), "IS Element".into(), "Large Deletion".into()],
            segments: vec![
                json!({"track": "SNP", "start": 70867.0, "end": 70868.0, "strand": ".", "label": "SNP"}),
                json!({"track": "SNP", "start": 1234567.0, "end": 1234568.0, "strand": ".", "label": "SNP2"}),
                json!({"track": "IS Element", "start": 776697.0, "end": 778028.0, "strand": "+", "label": "IS1"}),
                json!({"track": "Large Deletion", "start": 1298712.0, "end": 1330044.0, "strand": ".", "label": "DEL1"}),
            ],
            unit: "bp".into(),
        }),
        ("breseq_evidence_bar", DataBinding::Bar {
            id: "bl_breseq_evidence".into(), label: "Evidence Types".into(),
            categories: vec!["RA".into(), "MC".into(), "JC".into(), "UN".into()],
            values: vec![42.0, 12.0, 23.0, 17.0], unit: "count".into(),
        }),
        ("breseq_mutations_gauge", DataBinding::Gauge {
            id: "bl_breseq_mutations".into(), label: "Total Mutations".into(),
            value: 94.0, min: 0.0, max: 200.0, unit: "mutations".into(),
            normal_range: [0.0, 50.0], warning_range: [50.0, 150.0],
        }),
        ("breseq_coverage_ts", DataBinding::TimeSeries {
            id: "bl_breseq_coverage".into(), label: "Read Coverage".into(),
            x_label: "Position (bp)".into(), y_label: "Coverage".into(),
            x_values: vec![0.0, 500_000.0, 1_000_000.0, 2_000_000.0, 3_000_000.0, 4_000_000.0],
            y_values: vec![45.0, 52.0, 48.0, 55.0, 43.0, 50.0],
            unit: "reads".into(),
        }),
        ("plannotate_circular_map", DataBinding::CircularMap {
            id: "bl_plannotate_map".into(), label: "pUC19".into(),
            sequence_length: 2686.0, rings: vec!["features".into()],
            arcs: vec![
                json!({"start_angle": 0.0, "end_angle": 90.0, "ring": 0, "label": "ori"}),
                json!({"start_angle": 120.0, "end_angle": 200.0, "ring": 0, "label": "AmpR"}),
                json!({"start_angle": 240.0, "end_angle": 280.0, "ring": 0, "label": "lacZ"}),
            ],
            unit: "bp".into(),
        }),
        ("plannotate_features_bar", DataBinding::Bar {
            id: "bl_plannotate_features".into(), label: "Feature Lengths".into(),
            categories: vec!["ori".into(), "AmpR".into(), "lacZ".into()],
            values: vec![600.0, 860.0, 510.0], unit: "bp".into(),
        }),
        ("plannotate_confidence", DataBinding::Scatter {
            id: "bl_plannotate_conf".into(), label: "Annotation Confidence".into(),
            x: vec![600.0, 860.0, 510.0], y: vec![0.99, 0.95, 0.87],
            point_labels: vec!["ori".into(), "AmpR".into(), "lacZ".into()],
            x_label: "Length (bp)".into(), y_label: "Confidence".into(), unit: "".into(),
        }),
        ("ostir_tir_scatter", DataBinding::Scatter {
            id: "bl_ostir_tir".into(), label: "OSTIR TIR Predictions".into(),
            x: vec![42.0, 156.0, 891.0, 12345.0, 72891.0],
            y: vec![1200.0, 45000.0, 8900.0, 120.0, 67000.0],
            point_labels: vec!["RBS1".into(), "RBS2".into(), "RBS3".into(), "RBS4".into(), "RBS5".into()],
            x_label: "Position (nt)".into(), y_label: "TIR (au)".into(), unit: "au".into(),
        }),
        ("ostir_rate_dist", DataBinding::Distribution {
            id: "bl_ostir_rate_dist".into(), label: "TIR Distribution".into(),
            values: vec![42.8, 127.5, 85.2, 43.1, 99.8, 150.3, 67.4],
            mean: 85.15, std: 59.9, comparison_value: 100.0, unit: "au".into(),
        }),
        ("ostir_energy_bar", DataBinding::Bar {
            id: "bl_ostir_energy".into(), label: "dG Decomposition".into(),
            categories: vec!["dG_total".into(), "dG_mRNA".into(), "dG_spacing".into(), "dG_standby".into()],
            values: vec![-8.2, -3.1, -2.8, -2.3], unit: "kcal/mol".into(),
        }),
        ("cryptkeeper_track", DataBinding::GenomeTrack {
            id: "bl_cryptkeeper".into(), label: "CryptKeeper Analysis".into(),
            sequence_length: 4_629_812.0,
            tracks: vec!["ORFs/Features".into(), "Cryptic Promoters".into()],
            segments: vec![
                json!({"track": "ORFs/Features", "start": 100000.0, "end": 102000.0, "strand": "+", "label": "lacZ"}),
                json!({"track": "ORFs/Features", "start": 500000.0, "end": 501500.0, "strand": "-", "label": "araC"}),
                json!({"track": "Cryptic Promoters", "start": 101800.0, "end": 102200.0, "strand": "+", "label": "P_crypto_1"}),
                json!({"track": "Cryptic Promoters", "start": 800000.0, "end": 800400.0, "strand": "+", "label": "P_crypto_2"}),
            ],
            unit: "bp".into(),
        }),
        ("cryptkeeper_burden_heatmap", DataBinding::Heatmap {
            id: "bl_cryptkeeper_burden".into(), label: "Promoter Burden".into(),
            x_labels: vec!["lacZ".into(), "araC".into(), "galK".into()],
            y_labels: vec!["strong".into(), "medium".into(), "weak".into()],
            values: vec![0.9, 0.3, 0.5, 0.6, 0.7, 0.4, 0.2, 0.8, 0.1],
            unit: "burden".into(),
        }),
        ("efm_rate_track", DataBinding::GenomeTrack {
            id: "bl_efm_track".into(), label: "EFM Rate-Colored Features".into(),
            sequence_length: 4_629_812.0,
            tracks: vec!["IS Target".into(), "Repeat Indel".into(), "Base Sub Hotspot".into()],
            segments: vec![
                json!({"track": "IS Target", "start": 776697.0, "end": 778028.0, "strand": "+", "label": "IS1"}),
                json!({"track": "IS Target", "start": 1500000.0, "end": 1501200.0, "strand": "+", "label": "IS5"}),
                json!({"track": "Repeat Indel", "start": 1200000.0, "end": 1200500.0, "strand": ".", "label": "repeat1"}),
                json!({"track": "Base Sub Hotspot", "start": 3500000.0, "end": 3500100.0, "strand": ".", "label": "hotspot1"}),
            ],
            unit: "bp".into(),
        }),
        ("efm_rate_bar", DataBinding::Bar {
            id: "bl_efm_rate_bar".into(), label: "Mutation Rates by Category".into(),
            categories: vec!["IS Insertion".into(), "Repeat Indel".into(), "Base Sub".into()],
            values: vec![2.3e-6, 8.7e-7, 1.1e-9], unit: "per bp per gen".into(),
        }),
        ("md_divergence_scatter", DataBinding::Scatter {
            id: "bl_md_divergence".into(), label: "Marker Divergence".into(),
            x: vec![0.0, 100.0, 200.0, 500.0, 1000.0],
            y: vec![0.0, 0.15, 0.28, 0.52, 0.78],
            point_labels: vec!["t0".into(), "t100".into(), "t200".into(), "t500".into(), "t1000".into()],
            x_label: "Generations".into(), y_label: "Divergence".into(), unit: "rel".into(),
        }),
        ("md_trajectory_ts", DataBinding::TimeSeries {
            id: "bl_md_trajectory".into(), label: "Population Trajectory".into(),
            x_label: "Generations".into(), y_label: "Frequency".into(),
            x_values: vec![0.0, 100.0, 200.0, 300.0, 500.0, 750.0, 1000.0],
            y_values: vec![0.5, 0.52, 0.55, 0.61, 0.68, 0.73, 0.78],
            unit: "freq".into(),
        }),
        ("rna_mi_covariance", DataBinding::Heatmap {
            id: "bl_rna_mi_cov".into(), label: "RNA Mutual Information".into(),
            x_labels: vec!["pos1".into(), "pos2".into(), "pos3".into(), "pos4".into(), "pos5".into()],
            y_labels: vec!["pos1".into(), "pos2".into(), "pos3".into(), "pos4".into(), "pos5".into()],
            values: vec![
                1.0, 0.8, 0.2, 0.1, 0.05,
                0.8, 1.0, 0.3, 0.15, 0.1,
                0.2, 0.3, 1.0, 0.7, 0.4,
                0.1, 0.15, 0.7, 1.0, 0.6,
                0.05, 0.1, 0.4, 0.6, 1.0,
            ],
            unit: "bits".into(),
        }),
        ("rna_mi_entropy", DataBinding::Spectrum {
            id: "bl_rna_mi_entropy".into(), label: "Positional Entropy".into(),
            frequencies: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0],
            amplitudes: vec![0.3, 0.7, 0.5, 0.9, 0.2, 0.6, 0.4, 0.8],
            unit: "bits".into(),
        }),
    ]
}
