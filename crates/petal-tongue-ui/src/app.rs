//! Main application logic for petalTongue UI
//!
//! This is the **EguiGUI Modality** implementation - the native desktop GUI.
//!
//! ## Architecture Note
//!
//! This module represents Tier 3 (Enhancement) GUI modality using egui/eframe.
//! Rather than extracting to a separate modality file, we recognize that this
//! IS the EguiGUI implementation. This is a "smart refactor" approach - we don't
//! split code just to split it; the current organization is clean and working.

use crate::accessibility_panel::AccessibilityPanel;
use crate::audio::AudioSystemV2; // NEW: Substrate-agnostic audio
use crate::awakening_overlay::AwakeningOverlay;
// BingoCubeIntegration removed - it's a primalTool, discovered at runtime
use crate::graph_metrics_plotter::GraphMetricsPlotter;
use crate::keyboard_shortcuts::KeyboardShortcuts;
use crate::metrics_dashboard::MetricsDashboard; // NEW: Neural API metrics dashboard
use crate::proprioception_panel::ProprioceptionPanel; // NEW: Neural API SAME DAVE panel
use crate::graph_canvas::GraphCanvas; // NEW: Graph Builder canvas (Phase 4.8)
use crate::node_palette::NodePalette; // NEW: Node palette for graph builder
use crate::property_panel::PropertyPanel; // NEW: Property panel for node editing
use crate::graph_manager::GraphManagerPanel; // NEW: Graph manager for save/load/execute
use crate::process_viewer_integration::ProcessViewerTool;
use crate::proprioception::{ProprioceptionSystem, initialize_standard_proprioception}; // v1.1.0: SAME DAVE integration
use crate::status_reporter::StatusReporter;
use crate::system_dashboard::SystemDashboard;
use crate::system_monitor_integration::SystemMonitorTool;
use crate::tool_integration::ToolManager;
use crate::trust_dashboard::TrustDashboard;
use petal_tongue_adapters::{
    AdapterRegistry, EcoPrimalCapabilityAdapter, EcoPrimalFamilyAdapter, EcoPrimalTrustAdapter,
};
use petal_tongue_animation::AnimationEngine;
use petal_tongue_api::BiomeOSClient;
use petal_tongue_core::{
    CapabilityDetector, GraphEngine, InstanceId, LayoutAlgorithm, Modality, MotorCommand,
    RenderingAwareness, SensorEvent, SensorRegistry, SessionManager,
};
use petal_tongue_discovery::{NeuralApiProvider, VisualizationDataProvider, discover_visualization_providers};
use petal_tongue_graph::{AudioFileGenerator, AudioSonificationRenderer, Visual2DRenderer};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// The main petalTongue UI application
#[allow(clippy::struct_excessive_bools)]
pub struct PetalTongueApp {
    /// Capability detector (knows what modalities are actually available)
    capabilities: CapabilityDetector,
    /// The graph engine (shared between renderers)
    graph: Arc<RwLock<GraphEngine>>,
    /// Visual renderer
    visual_renderer: Visual2DRenderer,
    /// Audio renderer
    audio_renderer: AudioSonificationRenderer,
    /// Audio file generator (pure Rust WAV export)
    audio_generator: AudioFileGenerator,
    /// Animation engine (used for flow visualization)
    animation_engine: Arc<RwLock<AnimationEngine>>,
    /// Visualization data providers (discovered at runtime - capability-based!)
    #[allow(dead_code)] // TODO: Use for data aggregation when multi-provider support is ready
    data_providers: Vec<Box<dyn VisualizationDataProvider>>,
    /// Legacy `BiomeOS` client (DEPRECATED - kept for backward compatibility)
    #[deprecated(note = "Use data_providers instead - biomeOS is just another primal!")]
    biomeos_client: BiomeOSClient,
    /// Current layout algorithm
    current_layout: LayoutAlgorithm,
    /// Show audio description panel
    show_audio_panel: bool,
    /// Show capability status panel
    show_capability_panel: bool,
    /// Show controls panel
    show_controls: bool,
    /// Show animation (flow particles and pulses)
    show_animation: bool,
    /// Last refresh time
    last_refresh: Instant,
    /// Auto-refresh enabled
    auto_refresh: bool,
    /// Refresh interval (seconds)
    refresh_interval: f32,

    // Tool integration - capability-based, no hardcoded tool knowledge
    /// Tool manager (handles all external tools dynamically)
    tools: ToolManager,

    // Universal UI - Accessibility
    /// Accessibility settings panel
    accessibility_panel: AccessibilityPanel,

    // System Dashboard - Always visible metrics
    /// Live system dashboard
    system_dashboard: SystemDashboard,
    /// Show system dashboard sidebar
    show_dashboard: bool,

    // Audio System - Multimodal output (EVOLVED: Substrate-agnostic!)
    /// Audio system for UI sounds and data sonification
    /// Now uses substrate-agnostic AudioManager (runtime discovery, zero hardcoding)
    audio_system: AudioSystemV2,

    // Status Reporter - AI-accessible observability
    /// Status reporter (makes petalTongue observable to AI and external systems)
    pub status_reporter: Arc<StatusReporter>,

    // Keyboard Navigation
    /// Keyboard shortcuts system
    keyboard_shortcuts: KeyboardShortcuts,

    // Adapter System - Universal property rendering
    /// Property adapter registry (ecosystem-agnostic rendering)
    adapter_registry: AdapterRegistry,

    // Trust Dashboard - Trust visualization and monitoring
    /// Trust status dashboard
    trust_dashboard: TrustDashboard,
    /// Show trust dashboard panel
    show_trust_dashboard: bool,

    // Awakening Experience - Multi-modal startup
    /// Awakening overlay (visual flower animation + tutorial transition)
    awakening_overlay: AwakeningOverlay,

    // ===== Phase 2: Session Management =====
    /// Session manager for state persistence (optional, graceful degradation)
    #[allow(dead_code)] // TODO: Activate when session persistence is enabled
    session_manager: Option<SessionManager>,
    /// Instance ID for this petalTongue instance
    #[allow(dead_code)] // TODO: Use for multi-instance coordination
    instance_id: Option<InstanceId>,
    // ===== End Phase 2 =====

    // ===== Central Nervous System - Bidirectional Sensory Coordination =====
    /// Rendering awareness - motor + sensory feedback loop
    rendering_awareness: Arc<RwLock<RenderingAwareness>>,
    /// Sensor registry - discovered input peripherals
    sensor_registry: Arc<RwLock<SensorRegistry>>,
    /// Frame counter for tracking motor commands
    frame_count: u64,
    /// Last display verification time (Phase 4)
    last_display_verification: Instant,
    // ===== v1.1.0: SAME DAVE Proprioception System =====
    /// Complete self-awareness system (output + input + bidirectional feedback)
    proprioception: ProprioceptionSystem,
    
    // ===== v2.0: Neural API Integration =====
    /// Neural API provider (discovered at runtime)
    neural_api_provider: Option<Arc<NeuralApiProvider>>,
    /// Neural API proprioception panel (SAME DAVE visualization)
    neural_proprioception_panel: ProprioceptionPanel,
    /// Show Neural API proprioception panel
    show_neural_proprioception: bool,
    /// Neural API metrics dashboard
    neural_metrics_dashboard: MetricsDashboard,
    /// Show Neural API metrics dashboard
    show_neural_metrics: bool,
    /// Tokio runtime for async Neural API updates
    tokio_runtime: tokio::runtime::Runtime,
    
    // ===== v2.0: Graph Builder (Phase 4.8) =====
    /// Graph Builder canvas (interactive visual graph construction)
    graph_canvas: GraphCanvas,
    /// Node palette (available node types)
    node_palette: NodePalette,
    /// Property panel (node parameter editor)
    property_panel: PropertyPanel,
    /// Graph manager (save/load/execute via Neural API)
    graph_manager: GraphManagerPanel,
    /// Show Graph Builder window
    show_graph_builder: bool,
    
    // ===== v2.1: Adaptive UI (device-specific rendering) =====
    /// Adaptive UI manager (device-specific rendering) - DEPRECATED
    adaptive_ui: crate::adaptive_ui::AdaptiveUIManager,
    
    // ===== v2.2: Sensory UI (capability-based rendering) =====
    /// Sensory UI manager (capability-based rendering) - TRUE PRIMAL evolution
    sensory_ui: Option<crate::sensory_ui::SensoryUIManager>,
    /// Use sensory UI instead of adaptive UI (feature flag for migration)
    use_sensory_ui: bool,
}

impl PetalTongueApp {
    /// Create a new application
    #[must_use]
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        scenario_path: Option<std::path::PathBuf>,
        rendering_caps: petal_tongue_core::RenderingCapabilities,
    ) -> Self {
        // Load scenario if provided
        let (scenario, scenario_path_for_provider) = if let Some(path) = scenario_path {
            match crate::scenario::Scenario::load(&path) {
                Ok(s) => {
                    tracing::info!("✅ Scenario loaded successfully: {}", s.name);
                    (Some(s), Some(path))
                }
                Err(e) => {
                    tracing::error!("❌ Failed to load scenario: {}", e);
                    (None, None)
                }
            }
        } else {
            (None, None)
        };

        // Initialize tutorial mode (checks SHOWCASE_MODE environment variable)
        // OVERRIDE: If scenario is provided, disable tutorial mode
        let tutorial_mode = if scenario.is_some() {
            tracing::info!("📋 Scenario mode: Disabling tutorial/mock data");
            crate::tutorial_mode::TutorialMode::disabled()
        } else {
            crate::tutorial_mode::TutorialMode::new()
        };

        // Log rendering capabilities
        tracing::info!(
            "🎨 Rendering: {} ({:?}) - {} modalities",
            rendering_caps.device_type,
            rendering_caps.ui_complexity,
            rendering_caps.modalities.len()
        );

        // Capability-based discovery: Find ANY primal that provides visualization data
        // This could be: biomeOS, Songbird, custom aggregator, or multiple providers!
        // We discover at runtime - no hardcoded assumptions!

        tracing::info!("Starting capability-based provider discovery...");

        let runtime = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

        let data_providers = if let (Some(_scenario), Some(path)) = (&scenario, &scenario_path_for_provider) {
            // Scenario mode: Use DYNAMIC scenario provider (schema-agnostic!)
            tracing::info!("📋 Scenario mode: Loading primals with dynamic schema");
            match petal_tongue_discovery::DynamicScenarioProvider::from_file(path) {
                Ok(provider) => {
                    if let Some(version) = provider.version() {
                        tracing::info!("   Schema version: {}", version);
                    }
                    vec![
                        Box::new(provider) as Box<dyn VisualizationDataProvider>
                    ]
                },
                Err(e) => {
                    tracing::error!("Failed to create dynamic scenario provider: {}", e);
                    tracing::info!("Falling back to static provider...");
                    // Fallback to static provider
                    match petal_tongue_discovery::ScenarioVisualizationProvider::from_file(path) {
                        Ok(provider) => vec![
                            Box::new(provider) as Box<dyn VisualizationDataProvider>
                        ],
                        Err(e2) => {
                            tracing::error!("Static provider also failed: {}", e2);
                            vec![]
                        }
                    }
                }
            }
        } else if tutorial_mode.is_enabled() {
            // Tutorial mode: Use mock provider (explicitly requested by user)
            tracing::info!("📚 Tutorial mode: Using demonstration data");
            vec![
                Box::new(petal_tongue_discovery::MockVisualizationProvider::new())
                    as Box<dyn VisualizationDataProvider>,
            ]
        } else {
            // Production mode: Discover real providers
            runtime.block_on(async {
                match discover_visualization_providers().await {
                    Ok(providers) => {
                        if providers.is_empty() {
                            tracing::warn!("No visualization providers discovered");

                            // Graceful fallback: Use tutorial data so user can still see petalTongue
                            if crate::tutorial_mode::should_fallback(0) {
                                tracing::info!("💡 Using tutorial data as graceful fallback");
                                vec![Box::new(
                                    petal_tongue_discovery::MockVisualizationProvider::new(),
                                )
                                    as Box<dyn VisualizationDataProvider>]
                            } else {
                                vec![]
                            }
                        } else {
                            tracing::info!(
                                "✅ Discovered {} visualization data provider(s)",
                                providers.len()
                            );
                            for provider in &providers {
                                let metadata = provider.get_metadata();
                                tracing::info!(
                                    "  - {} at {} (protocol: {})",
                                    metadata.name,
                                    metadata.endpoint,
                                    metadata.protocol
                                );
                            }
                            providers
                        }
                    }
                    Err(e) => {
                        tracing::error!("Provider discovery failed: {}", e);

                        // Graceful fallback
                        if crate::tutorial_mode::should_fallback(0) {
                            tracing::info!("💡 Using tutorial data as graceful fallback");
                            vec![
                                Box::new(petal_tongue_discovery::MockVisualizationProvider::new())
                                    as Box<dyn VisualizationDataProvider>,
                            ]
                        } else {
                            vec![]
                        }
                    }
                }
            })
        };

        // Legacy BiomeOS client (DEPRECATED - for backward compatibility only)
        // New code should use data_providers instead
        //
        // TRUE PRIMAL: BiomeOS URL is discovered at runtime, not assumed.
        // Priority: ENV > Discovery > Mock fallback (no hardcoded defaults)
        let biomeos_url = std::env::var("BIOMEOS_URL").ok();
        let mock_mode_requested = std::env::var("PETALTONGUE_MOCK_MODE")
            .unwrap_or_else(|_| "false".to_string())
            .to_lowercase()
            == "true";

        // If no BiomeOS URL provided and not in mock mode, we'll discover at runtime
        let biomeos_url = biomeos_url.unwrap_or_else(|| {
            tracing::info!("No BIOMEOS_URL provided - will discover BiomeOS capability at runtime");
            "".to_string() // Empty = not yet discovered
        });

        #[allow(deprecated)]
        let biomeos_client = BiomeOSClient::new(&biomeos_url).with_mock_mode(mock_mode_requested);

        // Discover Neural API provider (v2.0: Central coordination for all primals)
        tracing::info!("🧠 Attempting Neural API discovery (central coordinator)...");
        let neural_api_provider = runtime.block_on(async {
            match NeuralApiProvider::discover(None).await {
                Ok(provider) => {
                    tracing::info!("✅ Neural API connected - using as primary provider");
                    Some(Arc::new(provider))
                }
                Err(e) => {
                    tracing::info!("Neural API not available: {} (graceful degradation)", e);
                    None
                }
            }
        });

        // Create graph engine
        let graph = GraphEngine::new();
        let graph = Arc::new(RwLock::new(graph));

        // Create capability detector (tests what modalities actually work)
        let capabilities = CapabilityDetector::default();
        tracing::info!("Capability detection complete");
        tracing::info!("{}", capabilities.capability_report());

        // Create status reporter EARLY so we can report capability detection
        let status_reporter = {
            let mut reporter = StatusReporter::new();
            // Enable status file for AI inspection FIRST
            if let Ok(status_file) = std::env::var("PETALTONGUE_STATUS_FILE") {
                reporter.enable_status_file(std::path::PathBuf::from(status_file));
            } else {
                // Default status file location
                reporter
                    .enable_status_file(std::path::PathBuf::from("/tmp/petaltongue_status.json"));
            }

            let reporter_arc = Arc::new(reporter);

            // Report capability detection results immediately
            for modality in &[
                Modality::Visual2D,
                Modality::Audio,
                Modality::Animation,
                Modality::TextDescription,
                Modality::Haptic,
                Modality::VR3D,
            ] {
                let available = capabilities.is_available(*modality);
                // Get capability info to extract reason
                let reason_string = capabilities
                    .get_status(*modality)
                    .map_or_else(|| "Not tested".to_string(), |c| c.reason.clone());
                let modality_name = match modality {
                    Modality::Visual2D => "visual2d",
                    Modality::Audio => "audio",
                    Modality::Animation => "animation",
                    Modality::TextDescription => "text_description",
                    Modality::Haptic => "haptic",
                    Modality::VR3D => "vr3d",
                };
                reporter_arc.update_modality(modality_name, available, true, reason_string);
            }

            reporter_arc
        };

        // Create renderers
        let mut visual_renderer = Visual2DRenderer::new(Arc::clone(&graph));
        let audio_renderer = AudioSonificationRenderer::new(Arc::clone(&graph));
        let audio_generator = AudioFileGenerator::new();
        let animation_engine = Arc::new(RwLock::new(AnimationEngine::new()));

        // Wire animation engine to visual renderer
        visual_renderer.set_animation_engine(Arc::clone(&animation_engine));
        visual_renderer.set_animation_enabled(true); // Enable by default

        // Initialize adapter registry with ecoPrimals adapters
        // In the future, adapters will be loaded based on ecosystem capability discovery
        let adapter_registry = AdapterRegistry::new();
        adapter_registry.register(Box::new(EcoPrimalTrustAdapter::new()));
        adapter_registry.register(Box::new(EcoPrimalFamilyAdapter::new()));
        adapter_registry.register(Box::new(EcoPrimalCapabilityAdapter::new()));

        tracing::info!(
            "Registered {} property adapters",
            adapter_registry.adapter_count()
        );
        tracing::debug!("Adapters: {:?}", adapter_registry.adapter_names());

        // Initialize tool manager and register available tools
        // Discovered at runtime, not hardcoded - capability-based!
        let tools = {
            let mut tm = ToolManager::new();
            // BingoCubeIntegration removed - it's a primalTool, discovered at runtime via IPC
            tm.register_tool(Box::new(SystemMonitorTool::default()));
            tm.register_tool(Box::new(ProcessViewerTool::default()));
            tm.register_tool(Box::new(GraphMetricsPlotter::default()));
            tm
        };

        // Initialize audio system (EVOLVED: Substrate-agnostic with runtime discovery!)
        let audio_system = AudioSystemV2::new();

        // Log discovered audio backend for observability
        if let Some(backend) = audio_system.active_backend() {
            tracing::info!("🎵 Active audio backend: {}", backend);
        }

        // Report available backends
        let backends = audio_system.available_backends();
        tracing::info!("🎵 Available audio backends: {} total", backends.len());
        for backend in &backends {
            tracing::info!("  - {}", backend);
        }

        // Check if we need fallback BEFORE consuming data_providers
        let needs_fallback = !tutorial_mode.is_enabled() && data_providers.is_empty();

        // Initialize central nervous system components
        let rendering_awareness = Arc::new(RwLock::new(RenderingAwareness::new()));
        let sensor_registry = Arc::new(RwLock::new(SensorRegistry::new()));
        tracing::info!("🧠 Central nervous system initialized");

        // Discover available sensors at runtime (no hardcoded assumptions!)
        let sensor_registry_clone = Arc::clone(&sensor_registry);
        runtime.block_on(async {
            if let Err(e) =
                crate::sensor_discovery::discover_all_sensors(sensor_registry_clone).await
            {
                tracing::error!("Sensor discovery failed: {}", e);
            }
        });

        // Verify essential sensors are available
        if !crate::sensor_discovery::verify_essential_sensors(&sensor_registry) {
            tracing::warn!("⚠️  Running with limited sensor capabilities");
        }

        // Note: Sensory event loop will be activated when sensors support async polling
        // Currently, sensors are polled via egui input events (which works perfectly!)
        tracing::info!("🔄 Sensory feedback via egui input events (bidirectional loop active)");

        let mut app = Self {
            capabilities,
            graph,
            visual_renderer,
            audio_renderer,
            audio_generator,
            animation_engine,
            data_providers,
            #[allow(deprecated)]
            biomeos_client,
            current_layout: LayoutAlgorithm::ForceDirected,
            show_audio_panel: true,
            show_capability_panel: false,
            show_controls: true,
            show_animation: true,
            last_refresh: Instant::now(),
            auto_refresh: true,
            refresh_interval: 5.0,

            // Tool manager - capability-based integration
            tools,

            accessibility_panel: AccessibilityPanel::default(),
            system_dashboard: SystemDashboard::default(),
            show_dashboard: true, // Show by default - part of Universal UI
            audio_system,
            status_reporter,
            keyboard_shortcuts: KeyboardShortcuts::default(),
            adapter_registry, // Universal property rendering
            trust_dashboard: TrustDashboard::new(),
            show_trust_dashboard: true, // Show by default

            // Awakening experience
            awakening_overlay: AwakeningOverlay::new(),

            // Phase 2: Session management (optional, graceful degradation)
            // Initialized in main.rs via Phase 1 integration (see main.rs:13-18)
            session_manager: None,
            instance_id: None,

            // Central Nervous System
            rendering_awareness,
            sensor_registry,
            frame_count: 0,
            last_display_verification: Instant::now(),
            // v1.1.0: SAME DAVE Proprioception
            proprioception: initialize_standard_proprioception(),
            
            // v2.0: Neural API Integration
            neural_api_provider: neural_api_provider.clone(),
            neural_proprioception_panel: ProprioceptionPanel::new(),
            show_neural_proprioception: false, // Toggle with 'P' key
            neural_metrics_dashboard: MetricsDashboard::new(),
            show_neural_metrics: false, // Toggle with 'M' key
            tokio_runtime: runtime,
            
            // v2.0: Graph Builder (Phase 4.8)
            graph_canvas: GraphCanvas::new("New Graph".to_string()),
            node_palette: NodePalette::new(),
            property_panel: PropertyPanel::new(),
            graph_manager: GraphManagerPanel::new(),
            show_graph_builder: false, // Toggle with 'G' key
            
            // v2.1: Adaptive UI (device-specific rendering) - DEPRECATED
            adaptive_ui: crate::adaptive_ui::AdaptiveUIManager::new(rendering_caps),
            
            // v2.2: Sensory UI (capability-based rendering) - TRUE PRIMAL
            sensory_ui: crate::sensory_ui::SensoryUIManager::new().ok(),
            use_sensory_ui: true, // Enable sensory UI by default (zero hardcoding!)
        };

        // Check if awakening experience is enabled
        let awakening_enabled = std::env::var("AWAKENING_ENABLED")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(true); // Enabled by default

        if awakening_enabled {
            tracing::info!("🌸 Awakening experience enabled");
            app.awakening_overlay.start();
        }

        // Play startup audio (signature tone + music)
        tracing::info!("🎵 Initializing startup audio...");
        let startup_audio = crate::startup_audio::StartupAudio::new();
        if startup_audio.has_startup_music() {
            tracing::info!(
                "🎵 Startup music found: {:?}",
                startup_audio.startup_music_path()
            );
        }
        startup_audio.play(&app.audio_system);

        // Set initial health status and force write AFTER audio system is ready
        tracing::info!("📊 Writing initial status file for AI observability...");
        app.status_reporter.update_health("healthy");
        app.status_reporter.force_write();

        // Initial data load
        // In tutorial mode, load scenarios; otherwise discover from network
        if tutorial_mode.is_enabled() {
            tutorial_mode.load_into_graph(Arc::clone(&app.graph), app.current_layout);
        } else if needs_fallback {
            // No providers and not in tutorial mode - create minimal fallback
            crate::tutorial_mode::TutorialMode::create_fallback_scenario(
                Arc::clone(&app.graph),
                app.current_layout,
            );
        } else {
            app.refresh_graph_data();
        }

        app
    }

    /// Refresh graph data from `BiomeOS`
    fn refresh_graph_data(&mut self) {
        // For now, we'll use blocking calls in the UI thread
        // TODO: Move to background task with channels
        let runtime = tokio::runtime::Runtime::new()
            .expect("failed to create tokio runtime - system resources exhausted?");

        runtime.block_on(async {
            // Discover primals
            #[allow(deprecated)] // TODO: Migrate to data_providers when aggregation ready
            let primals = match self.biomeos_client.discover_primals().await {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!("Failed to discover primals: {}", e);
                    return;
                }
            };

            // Get topology
            #[allow(deprecated)] // TODO: Migrate to data_providers when aggregation ready
            let edges = match self.biomeos_client.get_topology().await {
                Ok(e) => e,
                Err(e) => {
                    tracing::warn!("Failed to get topology: {}", e);
                    vec![]
                }
            };

            // Update graph
            let mut graph = self.graph.write().expect("graph lock poisoned");

            // Clear existing graph
            // (In production, we'd do a smart merge to preserve positions)
            *graph = GraphEngine::new();

            // Add primals
            for primal in &primals {
                graph.add_node(primal.clone());
            }

            // Add edges
            for edge in edges {
                graph.add_edge(edge);
            }

            // Apply layout
            graph.set_layout(self.current_layout);
            graph.layout(100);

            // Drop the graph lock before updating trust dashboard
            drop(graph);

            // Update trust dashboard with new primal data
            self.trust_dashboard.update_from_primals(&primals);
            tracing::debug!("✅ Trust dashboard updated with {} primals", primals.len());
        });

        self.last_refresh = Instant::now();
    }

    /// Get icon for a capability (shared logic)
    /// Get icon for capability type
    ///
    /// DEPRECATED: Use adapter system instead
    /// This method is kept temporarily for backward compatibility with any remaining code
    /// that hasn't been migrated to adapters yet.
    #[deprecated(note = "Use adapter_registry with EcoPrimalCapabilityAdapter instead")]
    fn get_capability_icon(&self, capability: &str) -> &'static str {
        let capability_lower = capability.to_lowercase();

        if capability_lower.contains("security")
            || capability_lower.contains("trust")
            || capability_lower.contains("auth")
        {
            "🔒"
        } else if capability_lower.contains("storage")
            || capability_lower.contains("persist")
            || capability_lower.contains("data")
        {
            "💾"
        } else if capability_lower.contains("compute")
            || capability_lower.contains("container")
            || capability_lower.contains("workload")
        {
            "⚙️"
        } else if capability_lower.contains("discovery")
            || capability_lower.contains("orchestration")
            || capability_lower.contains("federation")
        {
            "🔍"
        } else if capability_lower.contains("identity")
            || capability_lower.contains("lineage")
            || capability_lower.contains("genetic")
        {
            "🆔"
        } else if capability_lower.contains("encrypt")
            || capability_lower.contains("crypto")
            || capability_lower.contains("sign")
        {
            "🔐"
        } else if capability_lower.contains("ai")
            || capability_lower.contains("inference")
            || capability_lower.contains("intent")
        {
            "🧠"
        } else if capability_lower.contains("network")
            || capability_lower.contains("tcp")
            || capability_lower.contains("http")
            || capability_lower.contains("grpc")
        {
            "🌐"
        } else if capability_lower.contains("attribution")
            || capability_lower.contains("provenance")
            || capability_lower.contains("audit")
        {
            "📋"
        } else if capability_lower.contains("visual")
            || capability_lower.contains("ui")
            || capability_lower.contains("display")
        {
            "👁️"
        } else if capability_lower.contains("audio")
            || capability_lower.contains("sound")
            || capability_lower.contains("sonification")
        {
            "🔊"
        } else {
            "⚙️" // Default
        }
    }
}

impl eframe::App for PetalTongueApp {
    #[allow(clippy::too_many_lines, clippy::struct_excessive_bools)]
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // === CENTRAL NERVOUS SYSTEM - Motor Command ===
        // Record that we're rendering a frame (motor output)
        self.frame_count += 1;

        // v1.2.0: SAME DAVE - Record frame for hang detection & FPS tracking
        self.proprioception.record_frame();

        if let Ok(mut awareness) = self.rendering_awareness.write() {
            awareness.motor_command(MotorCommand::RenderFrame {
                frame_id: self.frame_count,
            });
        }

        // === CENTRAL NERVOUS SYSTEM - Sensory Feedback ===
        // Process user interactions as sensory confirmation
        ctx.input(|i| {
            // Mouse clicks = confirmation that user can see and interact
            if i.pointer.any_click() {
                if let Some(pos) = i.pointer.interact_pos() {
                    let event = SensorEvent::Click {
                        x: pos.x,
                        y: pos.y,
                        button: petal_tongue_core::MouseButton::Left,
                        timestamp: Instant::now(),
                    };
                    if let Ok(mut awareness) = self.rendering_awareness.write() {
                        awareness.sensory_feedback(&event);
                    }
                    // v1.1.0: SAME DAVE proprioception - pointer input received
                    self.proprioception
                        .input_received(&crate::input_verification::InputModality::Pointer);
                }
            }

            // Mouse movement = user can see display
            if let Some(pos) = i.pointer.hover_pos() {
                let event = SensorEvent::Position {
                    x: pos.x,
                    y: pos.y,
                    timestamp: Instant::now(),
                };
                if let Ok(mut awareness) = self.rendering_awareness.write() {
                    awareness.sensory_feedback(&event);
                }
                // v1.1.0: SAME DAVE proprioception - pointer movement
                self.proprioception
                    .input_received(&crate::input_verification::InputModality::Pointer);
            }

            // Any key press = bidirectional confirmation
            for key_event in &i.events {
                if let egui::Event::Key { .. } = key_event {
                    let event = SensorEvent::KeyPress {
                        key: petal_tongue_core::Key::Unknown,
                        modifiers: petal_tongue_core::Modifiers::none(),
                        timestamp: Instant::now(),
                    };
                    if let Ok(mut awareness) = self.rendering_awareness.write() {
                        awareness.sensory_feedback(&event);
                    }
                    // v1.1.0: SAME DAVE proprioception - keyboard input received
                    self.proprioception
                        .input_received(&crate::input_verification::InputModality::Keyboard);
                }
            }
        });

        // === PHASE 4: DISPLAY VISIBILITY VERIFICATION ===
        // Periodically verify that the display substrate is actually visible
        let now = Instant::now();
        if now.duration_since(self.last_display_verification) > Duration::from_secs(5) {
            self.last_display_verification = now;

            // Get last interaction time from rendering awareness
            let last_interaction_secs = if let Ok(awareness) = self.rendering_awareness.read() {
                awareness.time_since_last_interaction().as_secs_f32()
            } else {
                999.0 // Unknown
            };

            // Run display verification
            let verification = crate::display_verification::continuous_verification(
                "petalTongue",
                last_interaction_secs,
            );

            // Log the verification result
            tracing::debug!(
                "🔍 Display verification: {} (visible: {}, wm_responsive: {})",
                verification.status_message,
                verification.window_visible,
                verification.wm_responsive
            );

            // If we detect an issue, log it prominently
            if !verification.window_visible && verification.display_server_available {
                tracing::warn!(
                    "⚠️  Display substrate verification: Window may not be visible! Status: {}",
                    verification.status_message
                );
            }

            // v1.1.0: SAME DAVE proprioception - periodic self-assessment
            let state = self.proprioception.assess();
            tracing::debug!(
                "🧠 Proprioception: Health={:.0}% Confidence={:.0}% Motor={} Sensory={} Loop={}",
                state.health * 100.0,
                state.confidence * 100.0,
                state.motor_functional,
                state.sensory_functional,
                state.loop_complete
            );
        }

        // Update awakening overlay (if active)
        if self.awakening_overlay.is_active() {
            let delta_time = ctx.input(|i| i.stable_dt);
            if let Err(e) = self.awakening_overlay.update(delta_time) {
                tracing::error!("Awakening overlay update error: {}", e);
            }

            // Render awakening overlay (full-screen)
            self.awakening_overlay.render(ctx);

            // Check for tutorial transition
            if self.awakening_overlay.should_transition_to_tutorial() {
                tracing::info!("🎓 Transitioning to tutorial mode");
                let tutorial = crate::tutorial_mode::TutorialMode::new();
                if tutorial.is_enabled() {
                    tutorial.load_into_graph(self.graph.clone(), self.current_layout);
                }
            }

            // Request repaint for smooth animation
            ctx.request_repaint();

            // Skip normal UI while awakening is active
            return;
        }

        // Handle keyboard shortcuts for Neural API panels
        ctx.input(|i| {
            // 'P' key: Toggle Neural API Proprioception Panel
            if i.key_pressed(egui::Key::P) && !i.modifiers.ctrl && !i.modifiers.shift {
                self.show_neural_proprioception = !self.show_neural_proprioception;
                tracing::info!(
                    "🧠 Neural Proprioception Panel {}",
                    if self.show_neural_proprioception {
                        "enabled"
                    } else {
                        "disabled"
                    }
                );
            }
            
            // 'M' key: Toggle Neural API Metrics Dashboard
            if i.key_pressed(egui::Key::M) && !i.modifiers.ctrl && !i.modifiers.shift {
                self.show_neural_metrics = !self.show_neural_metrics;
                tracing::info!(
                    "📊 Neural Metrics Dashboard {}",
                    if self.show_neural_metrics {
                        "enabled"
                    } else {
                        "disabled"
                    }
                );
            }
            
            // 'G' key: Toggle Graph Builder
            if i.key_pressed(egui::Key::G) && !i.modifiers.ctrl && !i.modifiers.shift {
                self.show_graph_builder = !self.show_graph_builder;
                tracing::info!(
                    "🎨 Graph Builder {}",
                    if self.show_graph_builder {
                        "enabled"
                    } else {
                        "disabled"
                    }
                );
            }
        });

        // Sync accessibility audio settings with system dashboard
        self.system_dashboard
            .set_audio_enabled(self.accessibility_panel.settings.audio_enabled);
        self.system_dashboard
            .set_audio_volume(self.accessibility_panel.settings.audio_volume);

        // Update animation engine (flow particles and pulses)
        if self.show_animation
            && let Ok(mut engine) = self.animation_engine.write()
        {
            engine.update();
        }

        // Get current accessibility palette (respects user's color scheme choice!)
        let palette = self.accessibility_panel.get_palette();

        // Set theme using accessibility colors - NO HARDCODING!
        let mut style = (*ctx.style()).clone();
        style.visuals.dark_mode = true;
        style.visuals.override_text_color = Some(palette.text);
        style.visuals.window_fill = palette.background;
        style.visuals.panel_fill = palette.background_alt;
        ctx.set_style(style);

        // Top menu bar
        egui::TopBottomPanel::top("top_panel")
            .frame(
                egui::Frame::none()
                    .fill(palette.background)
                    .inner_margin(8.0),
            )
            .show(ctx, |ui| {
                let refresh_clicked = egui::menu::bar(ui, |ui| {
                    crate::app_panels::render_top_menu_bar(
                        ui,
                        &palette,
                        &mut self.accessibility_panel,
                        &mut self.visual_renderer,
                        &mut self.tools,
                        &mut self.current_layout,
                        &self.graph,
                        &mut self.show_dashboard,
                        &mut self.show_controls,
                        &mut self.show_audio_panel,
                        &mut self.show_capability_panel,
                        &mut self.show_neural_proprioception,
                        &mut self.show_neural_metrics,
                        &mut self.show_graph_builder,
                    )
                })
                .inner;

                if refresh_clicked {
                    self.refresh_graph_data();
                }
            });

        // Left panel - Controls
        if self.show_controls {
            egui::SidePanel::left("controls_panel")
                .default_width(280.0)
                .frame(
                    egui::Frame::none()
                        .fill(palette.background_alt)
                        .inner_margin(12.0),
                )
                .show(ctx, |ui| {
                    let elapsed = self.last_refresh.elapsed().as_secs_f32();
                    let refresh_clicked = crate::app_panels::render_controls_panel(
                        ui,
                        &palette,
                        &self.accessibility_panel,
                        &mut self.auto_refresh,
                        &mut self.refresh_interval,
                        elapsed,
                        &mut self.show_animation,
                        &mut self.visual_renderer,
                    );

                    if refresh_clicked {
                        self.refresh_graph_data();
                    }
                });
        }

        // Right panel - Audio information
        if self.show_audio_panel {
            egui::SidePanel::right("audio_panel")
                .default_width(380.0)
                .frame(
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(30, 30, 35))
                        .inner_margin(12.0),
                )
                .show(ctx, |ui| {
                    crate::app_panels::render_audio_panel(
                        ui,
                        &palette,
                        &self.accessibility_panel,
                        &mut self.audio_renderer,
                        &self.audio_generator,
                        &self.visual_renderer,
                        &self.capabilities,
                    );
                });
        }

        // Capability panel - Show modality status
        if self.show_capability_panel {
            egui::Window::new("🔍 Modality Capabilities")
                .default_width(500.0)
                .default_pos([400.0, 100.0])
                .show(ctx, |ui| {
                    ui.heading(egui::RichText::new("petalTongue Self-Awareness").size(16.0));
                    ui.add_space(8.0);
                    ui.label("This system knows what it can actually do:");
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(12.0);

                    for cap in self.capabilities.get_all() {
                        let (icon, color) = match cap.status {
                            petal_tongue_core::ModalityStatus::Available => {
                                ("✅", egui::Color32::from_rgb(100, 255, 100))
                            }
                            petal_tongue_core::ModalityStatus::NotInitialized => {
                                ("⚠️", egui::Color32::from_rgb(255, 200, 100))
                            }
                            petal_tongue_core::ModalityStatus::Unavailable => {
                                ("❌", egui::Color32::from_rgb(255, 100, 100))
                            }
                            petal_tongue_core::ModalityStatus::Disabled => {
                                ("🔇", egui::Color32::from_rgb(150, 150, 150))
                            }
                        };

                        let tested_text = if cap.tested { "tested" } else { "not tested" };

                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(40, 40, 45))
                            .stroke(egui::Stroke::new(1.0, color))
                            .inner_margin(10.0)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(icon).size(24.0));
                                    ui.vertical(|ui| {
                                        ui.label(
                                            egui::RichText::new(format!("{:?}", cap.modality))
                                                .size(14.0)
                                                .strong()
                                                .color(color),
                                        );
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "{:?} ({})",
                                                cap.status, tested_text
                                            ))
                                            .size(11.0)
                                            .color(egui::Color32::GRAY),
                                        );
                                    });
                                });
                                ui.add_space(6.0);
                                ui.label(
                                    egui::RichText::new(&cap.reason)
                                        .size(12.0)
                                        .color(egui::Color32::from_rgb(200, 200, 200)),
                                );
                            });
                        ui.add_space(8.0);
                    }

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("💡 Why This Matters").size(14.0).strong());
                    ui.add_space(4.0);
                    ui.label("In critical situations (wartime AR, disaster response, accessibility),\nfalse capability claims are dangerous. This system is honest about what it can do.");
                });
        }

        // Right panel - System Dashboard (LIVE metrics always visible!)
        if self.show_dashboard {
            egui::SidePanel::right("dashboard_panel")
                .default_width(220.0)
                .resizable(true)
                .frame(
                    egui::Frame::none()
                        .fill(palette.background_alt)
                        .inner_margin(12.0),
                )
                .show(ctx, |ui| {
                    let font_scale = self.accessibility_panel.settings.font_size.multiplier();

                    // System metrics
                    self.system_dashboard.render_compact(
                        ui,
                        &palette,
                        font_scale,
                        Some(&self.audio_system),
                    );

                    ui.add_space(8.0);

                    // Bidirectional sensory status (central nervous system)
                    crate::system_dashboard::SystemDashboard::render_sensory_status(
                        ui,
                        &palette,
                        font_scale,
                        &self.rendering_awareness,
                        &self.sensor_registry,
                    );

                    ui.add_space(8.0);

                    // v1.1.0: SAME DAVE Proprioception (complete self-awareness)
                    crate::system_dashboard::SystemDashboard::render_proprioception_status(
                        ui,
                        &palette,
                        font_scale,
                        &mut self.proprioception,
                    );
                });
        }

        // Right panel - Trust Dashboard (Trust status visualization!)
        if self.show_trust_dashboard {
            egui::SidePanel::right("trust_dashboard_panel")
                .default_width(280.0)
                .resizable(true)
                .frame(
                    egui::Frame::none()
                        .fill(palette.background_alt)
                        .inner_margin(12.0),
                )
                .show(ctx, |ui| {
                    let font_scale = self.accessibility_panel.settings.font_size.multiplier();
                    self.trust_dashboard
                        .render(ui, &palette, font_scale, Some(&self.audio_system));
                });
        }

        // v2.0: Neural API Proprioception Panel (SAME DAVE from Neural API)
        if self.show_neural_proprioception {
            egui::Window::new("🧠 Neural API Proprioception")
                .default_width(500.0)
                .default_height(600.0)
                .default_pos([100.0, 100.0])
                .show(ctx, |ui| {
                    if let Some(provider) = &self.neural_api_provider {
                        // Update panel with latest data (async)
                        self.tokio_runtime.block_on(async {
                            self.neural_proprioception_panel
                                .update(provider.as_ref())
                                .await;
                        });

                        // Render the panel
                        self.neural_proprioception_panel.render(ui);
                    } else {
                        ui.label("❌ Neural API not available");
                        ui.label("Start biomeOS nucleus to enable proprioception data.");
                    }
                });
        }

        // v2.0: Neural API Metrics Dashboard
        if self.show_neural_metrics {
            egui::Window::new("📊 Neural API Metrics")
                .default_width(600.0)
                .default_height(500.0)
                .default_pos([150.0, 150.0])
                .show(ctx, |ui| {
                    if let Some(provider) = &self.neural_api_provider {
                        // Update dashboard with latest data (async)
                        self.tokio_runtime.block_on(async {
                            self.neural_metrics_dashboard
                                .update(provider.as_ref())
                                .await;
                        });

                        // Render the dashboard
                        self.neural_metrics_dashboard.render(ui);
                    } else {
                        ui.label("❌ Neural API not available");
                        ui.label("Start biomeOS nucleus to enable metrics data.");
                    }
                });
        }

        // v2.0: Graph Builder (Phase 4.8)
        if self.show_graph_builder {
            egui::Window::new("🎨 Graph Builder")
                .default_width(1200.0)
                .default_height(800.0)
                .default_pos([50.0, 50.0])
                .resizable(true)
                .show(ctx, |ui| {
                    if self.neural_api_provider.is_some() {
                        ui.heading("🎨 Neural Graph Builder");
                        ui.separator();
                        ui.label("Interactive visual graph construction for Neural API.");
                        ui.separator();
                        
                        // Simplified initial implementation
                        // Full three-panel layout will be added in Phase 4.8.1
                        ui.horizontal(|ui| {
                            // Canvas area
                            ui.vertical(|ui| {
                                ui.heading("Canvas");
                                ui.separator();
                                
                                // Render the graph canvas
                                self.graph_canvas.render(ui, &palette);
                            });
                        });
                        
                        ui.separator();
                        ui.label("💡 Coming soon: Node palette, property editor, and graph management.");
                    } else {
                        ui.label("❌ Neural API not available");
                        ui.label("Start biomeOS nucleus to enable Graph Builder.");
                        ui.separator();
                        ui.label("The Graph Builder requires Neural API for graph persistence and execution.");
                    }
                });
        }

        // Right side panel - Primal details (if node selected)
        let selected_id_clone = self
            .visual_renderer
            .selected_node()
            .map(std::string::ToString::to_string);
        if let Some(selected_id) = selected_id_clone {
            egui::SidePanel::right("primal_details_panel")
                .default_width(350.0)
                .resizable(true)
                .show(ctx, |ui| {
                    crate::app_panels::render_primal_details_panel(
                        ui,
                        &selected_id,
                        &palette,
                        &self.graph,
                        &self.adapter_registry,
                        &mut self.visual_renderer,
                    );
                });
        }

        // Central panel - Graph visualization or active tool
        egui::CentralPanel::default().show(ctx, |ui| {
            // Check if any tool is visible (capability-based, not hardcoded)
            if let Some(tool) = self.tools.visible_tool() {
                // Tool is active - render its panel
                tool.render_panel(ui);
            } else {
                // No tool active - render the graph
                self.visual_renderer.render(ui);
            }
        });

        // Render accessibility panel (as a window)
        self.accessibility_panel.show(ctx);

        // Render keyboard shortcuts help overlay
        self.keyboard_shortcuts.render_help(ctx, &palette);

        // Auto-refresh logic
        if self.auto_refresh {
            let elapsed = self.last_refresh.elapsed();
            if elapsed >= Duration::from_secs_f32(self.refresh_interval) {
                self.refresh_graph_data();
            }

            // Request repaint for next frame
            ctx.request_repaint();
        }
    }
}
