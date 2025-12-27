//! # petal-tongue-ui
//!
//! Desktop UI application for petalTongue

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

pub mod app;
pub mod bingocube_integration;
pub mod graph_metrics_plotter;
pub mod process_viewer_integration;
pub mod system_monitor_integration;
pub mod toadstool_bridge;
pub mod tool_integration;

pub use app::PetalTongueApp;

// BingoCube demo is now standalone at bingoCube/demos
// Run it with: cd bingoCube/demos && cargo run --release
// Or: cargo run --manifest-path bingoCube/demos/Cargo.toml
