// SPDX-License-Identifier: AGPL-3.0-or-later
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! # petal-tongue-ui
//!
//! Desktop UI application for petalTongue

#![forbid(unsafe_code)]
#![expect(clippy::cast_sign_loss, reason = "UI rendering: f32 coords, u8 colors")]
#![expect(
    clippy::cast_precision_loss,
    reason = "UI rendering coordinate; f32/f64 precision is sufficient"
)]
#![expect(
    clippy::cast_possible_truncation,
    reason = "value is bounded by UI dimensions"
)]
#![expect(
    clippy::struct_excessive_bools,
    reason = "UI state structs need many flags"
)]
#![expect(
    clippy::too_many_lines,
    reason = "UI rendering functions are inherently long"
)]
#![expect(
    clippy::too_many_arguments,
    reason = "egui render callbacks receive many params"
)]
#![expect(
    clippy::match_same_arms,
    reason = "UI match arms kept explicit for readability per variant"
)]
#![expect(
    clippy::needless_pass_by_value,
    reason = "egui callbacks take owned types by convention"
)]
#![expect(
    clippy::format_push_string,
    reason = "format! into String is clearer for HTML/SVG builders"
)]
#![expect(
    clippy::unused_self,
    reason = "trait impls require &self even when unused"
)]
#![expect(
    clippy::unnecessary_wraps,
    reason = "Result return for API consistency across trait impls"
)]
#![expect(
    clippy::uninlined_format_args,
    reason = "explicit format args preferred for clarity in UI strings"
)]
#![expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "ref params for API consistency across trait impls"
)]
#![expect(
    missing_docs,
    reason = "UI module documentation tracked for incremental completion"
)]

// Re-export egui and eframe for use by parent crate
pub use eframe;
pub use egui;

/// Accessibility features and compliance (WCAG, universal design)
pub mod accessibility;
pub mod accessibility_panel;
pub mod adaptive_ui;
pub mod app;
pub mod app_panels;
pub mod audio;
pub mod audio_canvas;
pub mod audio_discovery;
pub mod audio_pure_rust;
pub mod awakening_overlay;
pub mod backend; // UI backend abstraction (ecoBlossom!)
pub mod error;
pub mod event_loop;
pub mod focus_manager;
pub mod interaction_adapters;
pub mod panel_registry;
pub mod panels;
pub mod scenario;
pub mod scenario_error;
pub mod sensor_discovery;
pub mod sensory_ui;
pub mod startup_audio;
pub mod tutorial_mode; // Concrete InputAdapter implementations (pointer, keyboard)
// bingocube is a primalTool (ecoPrimals/primalTools/bingoCube)
// Discovered at runtime via IPC, not compiled into petalTongue
pub mod biomeos_integration; // biomeOS UI Integration (device management provider)
pub mod biomeos_ui_manager; // biomeOS UI Manager - Phase 5 (integration & wiring)
pub mod data_source;
#[cfg(feature = "mock")]
pub mod demo_device_provider; // Demo fallback when biomeOS unavailable
pub mod device_panel; // Device Management UI - Phase 2
pub mod display; // Pure Rust display system
pub mod display_pure_rust;
pub mod display_verification; // Phase 4: Active display visibility verification
pub mod graph_canvas; // Graph builder canvas - visual graph construction (Neural API Phase 4)
pub mod graph_editor; // Collaborative Intelligence - Interactive graph editing
pub mod graph_manager; // Graph manager - save/load/execute via Neural API
pub mod graph_metrics_plotter;
pub mod headless_harness; // Headless egui harness for UI introspection and testing
pub mod human_entropy_window;
pub mod input_verification; // Universal input verification (keyboard, pointer, etc.)
pub mod keyboard_shortcuts;
pub mod live_data;
mod live_data_helpers;
pub mod live_sessions;
pub mod metrics_dashboard; // System metrics dashboard with sparklines (Neural API)
mod metrics_dashboard_helpers;
pub mod mode_presets; // Mode presets — named bundles of motor commands (SAME DAVE efferent)
/// Multimodal data streaming (audio, visual, haptic, etc.)
pub mod multimodal_stream;
pub mod niche_designer; // Niche Designer UI - Phase 4
pub mod node_palette; // Node palette - available node types for graph builder
pub mod output_verification; // Universal output verification (visual, audio, haptic, etc.)
pub mod primal_panel; // Primal Status UI - Phase 3
pub mod proc_stats;
pub mod process_viewer_integration;
pub mod property_panel; // Property panel - node parameter editor
pub mod proprioception; // SAME DAVE - Complete sensory-motor self-awareness
pub mod proprioception_panel; // SAME DAVE self-awareness visualization (Neural API)
pub mod protocol_selection; // Protocol priority: tarpc PRIMARY, JSON-RPC universal fallback, HTTPS optional
pub mod sensors;
pub mod ui_events; // Event-driven architecture for real-time updates // Sensor implementations (bidirectional UUI)
// Universal infant discovery (zero hardcoded knowledge)
pub mod ai_adapter;
pub mod egui_compiler;
pub mod game_data_channel;
pub mod interaction_bridge;
pub mod neural_registration;
#[cfg(any(test, feature = "mock"))]
pub mod sandbox_provider;
pub mod scene_bridge;
pub mod sensor_feed;
pub mod state;
pub mod status_reporter;
pub mod system_dashboard;
pub mod system_monitor_integration;
pub mod timeline_view;
pub mod tool_integration;
pub mod traffic_view;
pub mod trust_dashboard;
pub mod universal_discovery; // Capability-based GPU rendering discovery via registry provider

pub use app::PetalTongueApp;
pub use human_entropy_window::HumanEntropyWindow;
pub use sensors::{AudioSensor, KeyboardSensor, MouseSensor, ScreenSensor, discover_all_sensors};
pub use timeline_view::TimelineView;
pub use traffic_view::TrafficView;

// BingoCube demo is now standalone at bingoCube/demos
// Run it with: cd bingoCube/demos && cargo run --release
// Or: cargo run --manifest-path bingoCube/demos/Cargo.toml
