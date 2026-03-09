// SPDX-License-Identifier: AGPL-3.0-only
//! # petal-tongue-ui
//!
//! Desktop UI application for petalTongue

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::unused_async)]
// UI rendering: precision loss in casts is acceptable (f32 for coordinates, u8 for colors)
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
// Large structs and long functions common in UI code
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::ref_option)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::unnecessary_get_then_check)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::await_holding_lock)]
#![allow(clippy::default_trait_access)]
#![allow(clippy::format_push_string)]
#![allow(clippy::while_let_loop)]
#![allow(clippy::manual_let_else)]
#![allow(clippy::assigning_clones)]
#![allow(clippy::unused_self)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::needless_continue)]
#![allow(clippy::if_same_then_else)]
#![allow(clippy::self_only_used_in_recursion)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::unnecessary_debug_formatting)]
#![allow(clippy::case_sensitive_file_extension_comparisons)]
#![allow(missing_docs)]

// Re-export egui and eframe for use by parent crate
pub use eframe;
pub use egui;

/// Accessibility features and compliance (WCAG, universal design)
pub mod accessibility;
pub mod accessibility_panel;
pub mod adaptive_ui;
pub mod app;
pub mod app_panels;
pub mod awakening_overlay;
pub mod backend; // NEW: UI backend abstraction (ecoBlossom!)
pub mod event_loop;
pub mod focus_manager;
pub mod panel_registry;
pub mod panels;
pub mod scenario;
pub mod scenario_error;
pub mod sensor_discovery;
pub mod sensory_ui;
pub mod startup_audio;
pub mod tutorial_mode;
// Smart refactoring modules (app.rs → modular architecture) - TEMPORARILY DISABLED FOR INCREMENTAL REFACTORING
// pub mod app_state;    // Application state (Phase 1)
// pub mod app_ui;       // UI rendering (Phase 2)
// pub mod app_data;     // Data management (Phase 3)
// pub mod app_adapters; // Adapter management (Phase 4)
pub mod audio; // NEW: Substrate-agnostic audio system (TRUE PRIMAL evolution!)
pub mod audio_canvas; // Direct hardware access (like WGPU!)
pub mod audio_discovery; // v1.3.1: Audio discovery - PipeWire/PulseAudio Unix sockets (TRUE PRIMAL!)
#[cfg(feature = "legacy-audio")]
pub mod audio_providers;
pub mod audio_pure_rust;
pub mod interaction_adapters; // Concrete InputAdapter implementations (pointer, keyboard)
// bingocube is a primalTool (ecoPrimals/primalTools/bingoCube)
// Discovered at runtime via IPC, not compiled into petalTongue
pub mod biomeos_integration; // biomeOS UI Integration - Phase 1 (device management provider)
pub mod biomeos_ui_manager; // biomeOS UI Manager - Phase 5 (integration & wiring)
pub mod data_source;
pub mod device_panel; // Device Management UI - Phase 2
pub mod display; // Pure Rust display system
pub mod display_pure_rust;
pub mod display_verification; // Phase 4: Active display visibility verification
pub mod graph_canvas; // Graph builder canvas - visual graph construction (Neural API Phase 4)
pub mod graph_editor; // Collaborative Intelligence - Interactive graph editing
pub mod graph_manager; // Graph manager - save/load/execute via Neural API
pub mod graph_metrics_plotter;
pub mod human_entropy_window;
pub mod input_verification; // Universal input verification (keyboard, pointer, etc.)
pub mod keyboard_shortcuts;
pub mod live_data;
pub mod metrics_dashboard; // System metrics dashboard with sparklines (Neural API)
#[cfg(feature = "mock")]
pub mod mock_device_provider; // Mock provider - dev/test only, NEVER production (sovereignty)
pub mod mode_presets; // Mode presets — named bundles of motor commands (SAME DAVE efferent)
/// Multimodal data streaming (audio, visual, haptic, etc.)
pub mod multimodal_stream;
pub mod niche_designer; // Niche Designer UI - Phase 4
pub mod node_palette; // Node palette - available node types for graph builder
pub mod output_verification; // Universal output verification (visual, audio, haptic, etc.)
pub mod primal_panel; // Primal Status UI - Phase 3
pub mod process_viewer_integration;
pub mod property_panel; // Property panel - node parameter editor
pub mod proprioception; // SAME DAVE - Complete sensory-motor self-awareness
pub mod proprioception_panel; // SAME DAVE self-awareness visualization (Neural API)
pub mod protocol_selection; // Protocol priority: tarpc PRIMARY, JSON-RPC SECONDARY, HTTPS FALLBACK
pub mod sensors;
pub mod ui_events; // Event-driven architecture for real-time updates // Sensor implementations (bidirectional UUI)
// DEPRECATED: Old hardcoded discovery (remove after migration)
// pub mod rendering_discovery;

// NEW: Universal infant discovery (zero hardcoded knowledge)
#[cfg(any(test, feature = "mock"))]
pub mod sandbox_mock;
pub mod state;
pub mod status_reporter;
pub mod system_dashboard;
pub mod system_monitor_integration;
pub mod timeline_view;
// pub mod toadstool_bridge; // REMOVED: Deprecated Python bridge, superseded by tarpc integration (toadstool_v2.rs)
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
