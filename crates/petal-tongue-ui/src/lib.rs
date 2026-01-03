//! # petal-tongue-ui
//!
//! Desktop UI application for petalTongue

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

pub mod accessibility;
pub mod accessibility_panel;
pub mod app;
pub mod app_state;
pub mod audio_providers;
pub mod audio_pure_rust;
pub mod bingocube_integration;
pub mod data_source;
pub mod graph_metrics_plotter;
pub mod human_entropy_window;
pub mod keyboard_shortcuts;
pub mod live_data;
pub mod process_viewer_integration;
pub mod sandbox_mock;
pub mod state;
pub mod system_dashboard;
pub mod system_monitor_integration;
pub mod timeline_view;
pub mod toadstool_bridge;
pub mod tool_integration;
pub mod traffic_view;

pub use app::PetalTongueApp;
pub use human_entropy_window::HumanEntropyWindow;
pub use timeline_view::TimelineView;
pub use traffic_view::TrafficView;

// BingoCube demo is now standalone at bingoCube/demos
// Run it with: cd bingoCube/demos && cargo run --release
// Or: cargo run --manifest-path bingoCube/demos/Cargo.toml
