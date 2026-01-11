//! # petal-tongue-ui
//!
//! Desktop UI application for petalTongue

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

pub mod accessibility;
pub mod accessibility_panel;
pub mod app;
pub mod app_panels;
pub mod awakening_overlay;
pub mod event_loop;
pub mod sensor_discovery;
pub mod startup_audio;
pub mod tutorial_mode;
// Smart refactoring modules (app.rs → modular architecture) - TEMPORARILY DISABLED FOR INCREMENTAL REFACTORING
// pub mod app_state;    // Application state (Phase 1)
// pub mod app_ui;       // UI rendering (Phase 2)
// pub mod app_data;     // Data management (Phase 3)
// pub mod app_adapters; // Adapter management (Phase 4)
pub mod audio_canvas; // Direct hardware access (like WGPU!)
pub mod audio_discovery; // v1.3.1: Audio discovery - PipeWire/PulseAudio Unix sockets (TRUE PRIMAL!)
pub mod audio_providers;
pub mod audio_pure_rust;
pub mod bingocube_integration;
pub mod data_source;
pub mod display; // Pure Rust display system
pub mod display_pure_rust;
pub mod display_verification; // Phase 4: Active display visibility verification
pub mod graph_editor; // Collaborative Intelligence - Interactive graph editing
pub mod graph_metrics_plotter;
pub mod human_entropy_window;
pub mod input_verification; // Universal input verification (keyboard, pointer, etc.)
pub mod keyboard_shortcuts;
pub mod live_data;
pub mod multimodal_stream;
pub mod output_verification; // Universal output verification (visual, audio, haptic, etc.)
pub mod process_viewer_integration;
pub mod proprioception; // SAME DAVE - Complete sensory-motor self-awareness
pub mod protocol_selection; // Protocol priority: tarpc PRIMARY, JSON-RPC SECONDARY, HTTPS FALLBACK
pub mod sensors; // Sensor implementations (bidirectional UUI)
// DEPRECATED: Old hardcoded discovery (remove after migration)
// pub mod rendering_discovery;

// NEW: Universal infant discovery (zero hardcoded knowledge)
pub mod sandbox_mock;
pub mod state;
pub mod status_reporter;
pub mod system_dashboard;
pub mod system_monitor_integration;
pub mod timeline_view;
pub mod toadstool_bridge;
pub mod tool_integration;
pub mod traffic_view;
pub mod trust_dashboard;
pub mod universal_discovery; // NEW: Capability-based GPU rendering discovery via Songbird

pub use app::PetalTongueApp;
pub use human_entropy_window::HumanEntropyWindow;
pub use sensors::{AudioSensor, KeyboardSensor, MouseSensor, ScreenSensor, discover_all_sensors};
pub use timeline_view::TimelineView;
pub use traffic_view::TrafficView;

// BingoCube demo is now standalone at bingoCube/demos
// Run it with: cd bingoCube/demos && cargo run --release
// Or: cargo run --manifest-path bingoCube/demos/Cargo.toml
