// SPDX-License-Identifier: AGPL-3.0-or-later
//! Application state management
//!
//! This module contains the application state struct and initialization logic.
//! State is separated from view logic for better testability and maintainability.

use petal_tongue_animation::AnimationEngine;
use petal_tongue_api::BiomeOSClient;
use petal_tongue_core::{CapabilityDetector, GraphEngine, LayoutAlgorithm};
use petal_tongue_graph::{AudioFileGenerator, AudioSonificationRenderer, Visual2DRenderer};
use std::sync::{Arc, RwLock};
use std::time::Instant;

// BingoCube is a primalTool (ecoPrimals/primalTools/bingoCube)
// Discovered at runtime via IPC/tarpc, not compiled into petalTongue
// TRUE PRIMAL: Zero compile-time knowledge of other primals/tools!

/// Application state for petalTongue UI
///
/// This struct holds all the state needed for the application, including:
/// - Rendering engines (visual, audio, animation)
/// - `BiomeOS` client for live data
/// - UI state (panels, controls)
/// - `PrimalTools` discovered at runtime via IPC (no compile-time dependencies)
pub struct AppState {
    // Core rendering engines
    /// Capability detector (knows what modalities are actually available)
    pub capabilities: CapabilityDetector,
    /// The graph engine (shared between renderers)
    pub graph: Arc<RwLock<GraphEngine>>,
    /// Visual renderer
    pub visual_renderer: Visual2DRenderer,
    /// Audio renderer
    pub audio_renderer: AudioSonificationRenderer,
    /// Audio file generator (pure Rust WAV export)
    pub audio_generator: AudioFileGenerator,
    /// Animation engine (used for flow visualization)
    pub animation_engine: AnimationEngine,

    // Data source
    /// `BiomeOS` API client (capability-based discovery)
    pub biomeos_client: BiomeOSClient,

    // UI state
    /// Current layout algorithm
    pub current_layout: LayoutAlgorithm,
    /// Show audio description panel
    pub show_audio_panel: bool,
    /// Show capability status panel
    pub show_capability_panel: bool,
    /// Show controls panel
    pub show_controls: bool,
    /// Show animation (flow particles and pulses)
    pub show_animation: bool,
    /// Last refresh time
    pub last_refresh: Instant,
    /// Auto-refresh enabled
    pub auto_refresh: bool,
    /// Refresh interval (seconds)
    pub refresh_interval: f32,
    // PrimalTools (like BingoCube) are discovered at runtime via IPC
    // No compile-time state needed - use tarpc/JSON-RPC discovery instead
    // See: petal-tongue-ipc for runtime tool discovery patterns
}

impl AppState {
    /// Create new application state
    ///
    /// Initializes all rendering engines, capability detection, and `BiomeOS` client.
    /// Uses environment variables for configuration (no hardcoding).
    ///
    /// # Environment Variables
    ///
    /// - `BIOMEOS_URL`: `BiomeOS` endpoint (default: `http://localhost:3000`)
    /// - `PETALTONGUE_MOCK_MODE`: Enable mock mode (`true`/`false`, default: `false`)
    #[must_use]
    pub fn new() -> Self {
        // Use centralized configuration system - zero hardcoding
        let config = petal_tongue_core::PetalTongueConfig::default();

        // Get BiomeOS URL from config (environment-driven)
        let biomeos_url = config.biomeos_url();

        // Mock mode from config (NOT hardcoded)
        let mock_mode_requested = config.mock_mode;

        let biomeos_client = BiomeOSClient::new(&biomeos_url).with_mock_mode(mock_mode_requested);

        // Create graph engine (shared across renderers)
        let graph = GraphEngine::new();
        let graph = Arc::new(RwLock::new(graph));

        // Create capability detector (tests what modalities actually work)
        let capabilities = CapabilityDetector::default();
        tracing::info!("Capability detection complete");

        // Create renderers
        let visual_renderer = Visual2DRenderer::new(graph.clone());
        let audio_renderer = AudioSonificationRenderer::new(graph.clone());
        let audio_generator = AudioFileGenerator::new();
        let animation_engine = AnimationEngine::new();

        // PrimalTools are discovered at runtime (no compile-time config)

        Self {
            // Core engines
            capabilities,
            graph,
            visual_renderer,
            audio_renderer,
            audio_generator,
            animation_engine,

            // Data source
            biomeos_client,

            // UI state
            current_layout: LayoutAlgorithm::ForceDirected,
            show_audio_panel: false,
            show_capability_panel: false,
            show_controls: true,
            show_animation: config.audio_enabled, // From config, not hardcoded
            last_refresh: Instant::now(),
            auto_refresh: true,
            refresh_interval: config.refresh_interval_secs as f32,
            // PrimalTools state managed via runtime discovery (no compile-time state)
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::Modality;

    #[test]
    fn test_app_state_creation() {
        let state = AppState::new();

        // Verify initial state
        assert_eq!(state.current_layout, LayoutAlgorithm::ForceDirected);
        assert!(state.show_controls);
        assert!(!state.show_audio_panel);
        assert!(!state.show_capability_panel);
        assert!(state.auto_refresh);
        assert_eq!(state.refresh_interval, 5.0);
    }

    #[test]
    fn test_app_state_defaults() {
        let state = AppState::new();

        // Verify capability detector initialized
        assert!(state.capabilities.has_modality(Modality::Visual2D));

        // Verify graph engine initialized
        let graph = state.graph.read().expect("graph lock poisoned");
        assert_eq!(graph.nodes().len(), 0);
        assert_eq!(graph.edges().len(), 0);
    }

    #[test]
    fn test_layout_algorithm_default() {
        let state = AppState::new();
        assert_eq!(state.current_layout, LayoutAlgorithm::ForceDirected);

        // Graph should use the same layout
        let graph = state.graph.read().expect("graph lock poisoned");
        assert_eq!(graph.get_layout(), LayoutAlgorithm::ForceDirected);
    }

    #[test]
    fn test_biomeos_client_initialization() {
        // Test initialization with environment variables
        // Note: We don't actually remove env vars in tests as it's unsafe
        // and affects other tests. Instead, we test that initialization
        // handles missing vars gracefully.

        let state = AppState::new();

        // Client should be initialized
        // Verify it doesn't panic on creation or drop
        drop(state);
    }

    #[test]
    fn test_bingocube_initial_state() {
        let _state = AppState::new();

        // BingoCube is a primalTool, discovered at runtime (not a compile-time dependency)
        // No compile-time state for BingoCube in AppState - discovered via IPC
        // This test is deprecated - BingoCube integration is now runtime-only
    }

    #[test]
    fn test_panel_visibility_defaults() {
        let state = AppState::new();

        // Controls should be visible by default
        assert!(state.show_controls);

        // Other panels should be hidden by default
        assert!(!state.show_audio_panel);
        assert!(!state.show_capability_panel);
        // show_bingocube_panel removed - BingoCube is runtime-discovered primalTool
    }

    #[test]
    fn test_auto_refresh_defaults() {
        let state = AppState::new();

        // Auto-refresh should be enabled with 5 second interval
        assert!(state.auto_refresh);
        assert_eq!(state.refresh_interval, 5.0);

        // Last refresh should be initialized (approximately now)
        let elapsed = state.last_refresh.elapsed();
        assert!(elapsed.as_secs() < 1);
    }

    #[test]
    fn test_animation_defaults() {
        let state = AppState::new();

        // Animation is ENABLED by default (true, not false)
        assert!(state.show_animation);
    }

    #[test]
    fn test_capability_detector_visual() {
        let state = AppState::new();

        // Visual modality should always be available
        assert!(state.capabilities.has_modality(Modality::Visual2D));
        assert!(state.capabilities.has_capability("visual.2d"));
    }

    #[test]
    fn test_capability_detector_audio() {
        let state = AppState::new();

        // Audio modality availability depends on system
        // Just verify we can check it without panicking
        let _has_audio = state.capabilities.has_modality(Modality::Audio);
        let _has_sonification = state.capabilities.has_capability("audio.sonification");
    }

    #[test]
    fn test_state_drop_cleanup() {
        // Create and drop state - should not panic
        let state = AppState::new();
        drop(state);

        // Create another one - should work fine
        let state2 = AppState::new();
        drop(state2);
    }

    #[test]
    fn test_multiple_states_independent() {
        let state1 = AppState::new();
        let state2 = AppState::new();

        // States should be independent
        assert_ne!(Arc::as_ptr(&state1.graph), Arc::as_ptr(&state2.graph));

        drop(state1);
        drop(state2);
    }

    // BingoCube config test removed - it's a primalTool, discovered at runtime

    #[test]
    fn test_refresh_interval_bounds() {
        let state = AppState::new();

        // Refresh interval should be reasonable (1-60 seconds)
        assert!(state.refresh_interval >= 1.0);
        assert!(state.refresh_interval <= 60.0);
    }
}
