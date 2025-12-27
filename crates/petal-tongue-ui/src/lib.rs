//! # petal-tongue-ui
//!
//! Desktop UI application for petalTongue

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

pub mod app;
pub mod bingocube_integration;
pub mod data_source;
pub mod graph_metrics_plotter;
pub mod process_viewer_integration;
pub mod state;
pub mod system_monitor_integration;
pub mod timeline_view;
pub mod toadstool_bridge;
pub mod tool_integration;
pub mod traffic_view;

pub use app::PetalTongueApp;
pub use timeline_view::TimelineView;
pub use traffic_view::TrafficView;

// BingoCube demo is now standalone at bingoCube/demos
// Run it with: cd bingoCube/demos && cargo run --release
// Or: cargo run --manifest-path bingoCube/demos/Cargo.toml
