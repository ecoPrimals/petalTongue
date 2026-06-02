// SPDX-License-Identifier: AGPL-3.0-or-later
//! Command-line argument parsing and help text for the headless binary.

/// Command-line arguments
#[derive(Debug)]
pub struct Args {
    pub mode: OutputMode,
    pub output: Option<String>,
    pub width: u32,
    pub height: u32,
    pub scenario: Option<String>,
    pub demo: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputMode {
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
    pub fn parse() -> Self {
        let mut args = std::env::args().skip(1);
        let mut mode = OutputMode::Auto;
        let mut output = None;
        let mut width = 1920;
        let mut height = 1080;
        let mut scenario = None;
        let mut demo = std::env::var("SHOWCASE_MODE")
            .ok()
            .is_some_and(|v| v == "true" || v == "1");

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
                "--scenario" | "-s" => {
                    scenario = args.next();
                }
                "--demo" => {
                    demo = true;
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
            scenario,
            demo,
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
    -s, --scenario <FILE>   Load graph data from scenario JSON file
    --demo                  Load built-in demonstration topology (also via SHOWCASE_MODE=true)
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
