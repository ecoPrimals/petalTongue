//! Application state management
//!
//! This module centralizes all application state for the petalTongue UI.
//! It separates state from UI rendering logic and coordination.

use crate::accessibility::ColorPalette;
use crate::accessibility_panel::AccessibilityPanel;
use crate::audio_providers::AudioSystem;
use crate::keyboard_shortcuts::KeyboardShortcuts;
use crate::status_reporter::StatusReporter;
use crate::system_dashboard::SystemDashboard;
use crate::tool_integration::ToolManager;
use crate::trust_dashboard::TrustDashboard;
use petal_tongue_adapters::AdapterRegistry;
use petal_tongue_animation::AnimationEngine;
use petal_tongue_api::BiomeOSClient;
use petal_tongue_core::{CapabilityDetector, GraphEngine, InstanceId, LayoutAlgorithm, SessionManager};
use petal_tongue_discovery::VisualizationDataProvider;
use petal_tongue_graph::{AudioFileGenerator, AudioSonificationRenderer, Visual2DRenderer};
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// Centralized application state
///
/// This struct contains all the state for the petalTongue application,
/// organized into logical groups:
/// - Core engine state (graph, animation)
/// - Rendering state (visual, audio)
/// - UI state (panels, toggles, layout)
/// - Data state (providers, refresh timing)
/// - Session state (persistence, instance ID)
#[allow(clippy::struct_excessive_bools)]
pub struct AppState {
    // ===== Core Engine State =====
    /// Capability detector (knows what modalities are actually available)
    pub capabilities: CapabilityDetector,
    
    /// The graph engine (shared between renderers)
    pub graph: Arc<RwLock<GraphEngine>>,
    
    /// Animation engine (used for flow visualization)
    pub animation_engine: Arc<RwLock<AnimationEngine>>,

    // ===== Rendering State =====
    /// Visual renderer
    pub visual_renderer: Visual2DRenderer,
    
    /// Audio renderer
    pub audio_renderer: AudioSonificationRenderer,
    
    /// Audio file generator (pure Rust WAV export)
    pub audio_generator: AudioFileGenerator,

    // ===== Data Provider State =====
    /// Visualization data providers (discovered at runtime - capability-based!)
    pub data_providers: Vec<Box<dyn VisualizationDataProvider>>,
    
    /// Legacy `BiomeOS` client (DEPRECATED - kept for backward compatibility)
    #[deprecated(note = "Use data_providers instead - biomeOS is just another primal!")]
    pub biomeos_client: BiomeOSClient,
    
    /// Last refresh time
    pub last_refresh: Instant,
    
    /// Auto-refresh enabled
    pub auto_refresh: bool,
    
    /// Refresh interval (seconds)
    pub refresh_interval: f32,

    // ===== Layout & View State =====
    /// Current layout algorithm
    pub current_layout: LayoutAlgorithm,

    // ===== UI Panel Toggles =====
    /// Show audio description panel
    pub show_audio_panel: bool,
    
    /// Show capability status panel
    pub show_capability_panel: bool,
    
    /// Show controls panel
    pub show_controls: bool,
    
    /// Show animation (flow particles and pulses)
    pub show_animation: bool,
    
    /// Show system dashboard sidebar
    pub show_dashboard: bool,
    
    /// Show trust dashboard panel
    pub show_trust_dashboard: bool,

    // ===== Component State =====
    /// Tool manager (handles all external tools dynamically)
    pub tools: ToolManager,
    
    /// Accessibility settings panel
    pub accessibility_panel: AccessibilityPanel,
    
    /// Live system dashboard
    pub system_dashboard: SystemDashboard,
    
    /// Audio system for UI sounds and data sonification
    pub audio_system: AudioSystem,
    
    /// Status reporter (makes petalTongue observable to AI and external systems)
    pub status_reporter: Arc<StatusReporter>,
    
    /// Keyboard shortcuts system
    pub keyboard_shortcuts: KeyboardShortcuts,
    
    /// Property adapter registry (ecosystem-agnostic rendering)
    pub adapter_registry: AdapterRegistry,
    
    /// Trust status dashboard
    pub trust_dashboard: TrustDashboard,

    // ===== Session Management State =====
    /// Session manager for state persistence (optional, graceful degradation)
    pub session_manager: Option<SessionManager>,
    
    /// Instance ID for this petalTongue instance
    pub instance_id: Option<InstanceId>,
}

impl AppState {
    /// Create a new application state
    ///
    /// This initializes all components and discovers visualization providers.
    ///
    /// # Arguments
    ///
    /// * `data_providers` - Discovered visualization data providers
    /// * `biomeos_client` - Legacy BiomeOS client (deprecated)
    #[allow(deprecated)]
    pub fn new(
        data_providers: Vec<Box<dyn VisualizationDataProvider>>,
        biomeos_client: BiomeOSClient,
    ) -> Self {
        let capabilities = CapabilityDetector::default();
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let animation_engine = Arc::new(RwLock::new(AnimationEngine::new()));

        // Initialize renderers
        let visual_renderer = Visual2DRenderer::new(Arc::clone(&graph));
        let audio_renderer = AudioSonificationRenderer::new(Arc::clone(&graph));
        let audio_generator = AudioFileGenerator::new();

        // Initialize components
        let tools = ToolManager::default();
        let accessibility_panel = AccessibilityPanel::default();
        let system_dashboard = SystemDashboard::default();
        let audio_system = AudioSystem::default();
        let status_reporter = Arc::new(StatusReporter::default());
        let keyboard_shortcuts = KeyboardShortcuts::default();
        let adapter_registry = AdapterRegistry::new();
        let trust_dashboard = TrustDashboard::default();

        // Initialize session management (TODO: currently not initialized from main.rs)
        // Session management will be fully implemented in future phases
        let session_manager = None;
        let instance_id = None;

        Self {
            capabilities,
            graph,
            animation_engine,
            visual_renderer,
            audio_renderer,
            audio_generator,
            data_providers,
            biomeos_client,
            last_refresh: Instant::now(),
            auto_refresh: true,
            refresh_interval: 5.0,
            current_layout: LayoutAlgorithm::ForceDirected,
            show_audio_panel: false,
            show_capability_panel: false,
            show_controls: true,
            show_animation: true,
            show_dashboard: true,
            show_trust_dashboard: false,
            tools,
            accessibility_panel,
            system_dashboard,
            audio_system,
            status_reporter,
            keyboard_shortcuts,
            adapter_registry,
            trust_dashboard,
            session_manager,
            instance_id,
        }
    }

    /// Get the current color palette from accessibility settings
    pub fn color_palette(&self) -> ColorPalette {
        self.accessibility_panel.get_palette()
    }

    /// Check if it's time to refresh data
    pub fn should_refresh(&self) -> bool {
        self.auto_refresh && self.last_refresh.elapsed().as_secs_f32() >= self.refresh_interval
    }

    /// Mark that a refresh has occurred
    pub fn mark_refreshed(&mut self) {
        self.last_refresh = Instant::now();
    }

    /// Get mutable access to the graph engine
    pub fn graph_mut(&self) -> std::sync::RwLockWriteGuard<'_, GraphEngine> {
        self.graph.write().expect("Graph lock poisoned")
    }

    /// Get read access to the graph engine
    pub fn graph(&self) -> std::sync::RwLockReadGuard<'_, GraphEngine> {
        self.graph.read().expect("Graph lock poisoned")
    }
}

