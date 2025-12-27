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

// BingoCube tool integration
use bingocube_adapters::audio::BingoCubeAudioRenderer;
use bingocube_adapters::visual::BingoCubeVisualRenderer;
use bingocube_core::{BingoCube, Config as BingoCubeConfig};

/// Application state for petalTongue UI
///
/// This struct holds all the state needed for the application, including:
/// - Rendering engines (visual, audio, animation)
/// - BiomeOS client for live data
/// - UI state (panels, controls)
/// - BingoCube tool integration (demonstrating primal tool use)
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
    /// BiomeOS API client (capability-based discovery)
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

    // BingoCube tool integration (demonstrating primal tool use)
    /// Show BingoCube panel
    pub show_bingocube_panel: bool,
    /// BingoCube instance (tool being used)
    pub bingocube: Option<BingoCube>,
    /// BingoCube visual renderer (adapter)
    pub bingocube_renderer: Option<BingoCubeVisualRenderer>,
    /// BingoCube audio renderer (adapter)
    pub bingocube_audio_renderer: Option<BingoCubeAudioRenderer>,
    /// BingoCube seed input
    pub bingocube_seed: String,
    /// BingoCube reveal parameter (0.0-1.0)
    pub bingocube_x: f64,
    /// BingoCube configuration
    pub bingocube_config: BingoCubeConfig,
    /// BingoCube error message
    pub bingocube_error: Option<String>,
    /// Show BingoCube configuration panel
    pub show_bingocube_config: bool,
    /// Show BingoCube audio panel
    pub show_bingocube_audio: bool,
}

impl AppState {
    /// Create new application state
    ///
    /// Initializes all rendering engines, capability detection, and BiomeOS client.
    /// Uses environment variables for configuration (no hardcoding).
    ///
    /// # Environment Variables
    ///
    /// - `BIOMEOS_URL`: BiomeOS endpoint (default: `http://localhost:3000`)
    /// - `PETALTONGUE_MOCK_MODE`: Enable mock mode (`true`/`false`, default: `false`)
    #[must_use]
    pub fn new() -> Self {
        // Create BiomeOS client with runtime capability detection
        // Configuration is environment-driven, not hardcoded
        let biomeos_url =
            std::env::var("BIOMEOS_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

        // Mock mode is ONLY enabled via environment variable
        // Default: FALSE (try real connection first, fallback to mock only if unavailable)
        let mock_mode_requested = std::env::var("PETALTONGUE_MOCK_MODE")
            .unwrap_or_else(|_| "false".to_string())
            .to_lowercase()
            == "true";

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
        let audio_generator = AudioFileGenerator::new(graph.clone());
        let animation_engine = AnimationEngine::new();

        // BingoCube tool integration (demonstrating primal tool use)
        let bingocube_config = BingoCubeConfig {
            grid_size: 8,
            reveal_threshold: 0.5,
            ..Default::default()
        };

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
            show_animation: true,
            last_refresh: Instant::now(),
            auto_refresh: true,
            refresh_interval: 5.0,

            // BingoCube tool state
            show_bingocube_panel: false,
            bingocube: None,
            bingocube_renderer: None,
            bingocube_audio_renderer: None,
            bingocube_seed: String::new(),
            bingocube_x: 0.5,
            bingocube_config,
            bingocube_error: None,
            show_bingocube_config: false,
            show_bingocube_audio: false,
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

    #[test]
    fn test_state_creation() {
        let state = AppState::new();
        assert_eq!(state.current_layout, LayoutAlgorithm::ForceDirected);
        assert!(state.auto_refresh);
        assert!((state.refresh_interval - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_state_default() {
        let state = AppState::default();
        assert_eq!(state.current_layout, LayoutAlgorithm::ForceDirected);
    }

    #[test]
    fn test_bingocube_initial_state() {
        let state = AppState::new();
        assert!(state.bingocube.is_none());
        assert!(!state.show_bingocube_panel);
        assert_eq!(state.bingocube_config.grid_size, 8);
    }
}

