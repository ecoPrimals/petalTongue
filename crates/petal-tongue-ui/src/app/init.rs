// SPDX-License-Identifier: AGPL-3.0-or-later
//! Application initialization logic
//!
//! Extracted from app.rs - handles scenario loading, provider discovery,
//! renderer setup, and construction of `PetalTongueApp`.

use super::PetalTongueApp;
use crate::accessibility_panel::AccessibilityPanel;
use crate::audio::AudioSystemV2;
use crate::awakening_overlay::AwakeningOverlay;
use crate::error::Result;
use crate::graph_canvas::GraphCanvas;
use crate::graph_metrics_plotter::GraphMetricsPlotter;
use crate::interaction_bridge::EguiInteractionBridge;
use crate::keyboard_shortcuts::KeyboardShortcuts;
use crate::metrics_dashboard::MetricsDashboard;
use crate::panel_registry::{PanelInstance, PanelRegistry};
#[cfg(feature = "doom")]
use crate::panels::create_doom_factory;
use crate::process_viewer_integration::ProcessViewerTool;
use crate::proprioception::initialize_standard_proprioception;
use crate::proprioception_panel::ProprioceptionPanel;
use crate::status_reporter::StatusReporter;
use crate::system_dashboard::SystemDashboard;
use crate::system_monitor_integration::SystemMonitorTool;
use crate::tool_integration::ToolManager;
use crate::trust_dashboard::TrustDashboard;
use petal_tongue_adapters::{
    AdapterRegistry, EcoPrimalCapabilityAdapter, EcoPrimalFamilyAdapter, EcoPrimalTrustAdapter,
};
use petal_tongue_animation::AnimationEngine;
use petal_tongue_core::{
    CapabilityDetector, GraphEngine, LayoutAlgorithm, Modality,
    channel::standard_channels,
    constants::{self, APP_DIR_NAME},
};
use petal_tongue_discovery::{
    NeuralApiProvider, VisualizationDataProvider, discover_visualization_providers,
};
use petal_tongue_graph::{AudioFileGenerator, AudioSonificationRenderer, Visual2DRenderer};
use petal_tongue_scene::game_loop::{TickClock, TickConfig};
use std::sync::mpsc;
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// Create a fully initialized `PetalTongueApp`.
///
/// This is the main initialization entry point - handles scenario loading,
/// provider discovery, renderer setup, and all component wiring.
pub(super) fn create_app(
    scenario_path: Option<std::path::PathBuf>,
    rendering_caps: petal_tongue_core::RenderingCapabilities,
    shared_graph: Arc<RwLock<GraphEngine>>,
) -> Result<PetalTongueApp> {
    tracing::info!("✅ Using shared graph from DataService - zero data duplication!");

    // Load scenario if provided
    let (scenario, scenario_path_for_provider) = scenario_path.map_or((None, None), |path| {
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
    });

    // Initialize tutorial mode (checks SHOWCASE_MODE environment variable)
    let tutorial_mode = if scenario.is_some() {
        tracing::info!("📋 Scenario mode: Disabling tutorial/mock data");
        crate::tutorial_mode::TutorialMode::disabled()
    } else {
        crate::tutorial_mode::TutorialMode::new()
    };

    tracing::info!(
        "🎨 Rendering: {} ({:?}) - {} modalities",
        rendering_caps.device_type,
        rendering_caps.ui_complexity,
        rendering_caps.modalities.len()
    );

    tracing::info!("Starting capability-based provider discovery...");
    let runtime = tokio::runtime::Runtime::new()?;

    let data_providers = discover_data_providers(
        &scenario,
        &scenario_path_for_provider,
        &tutorial_mode,
        &runtime,
    );

    let neural_api_provider = discover_neural_api(&runtime);
    let graph = shared_graph;

    let capabilities = CapabilityDetector::default();
    tracing::info!("Capability detection complete");
    tracing::info!("{}", capabilities.capability_report());

    let status_reporter = create_status_reporter(&capabilities);
    let (visual_renderer, audio_renderer, audio_generator, animation_engine) =
        create_renderers(Arc::clone(&graph), &scenario);

    let adapter_registry = create_adapter_registry();
    let tools = create_tool_manager();
    let audio_system = AudioSystemV2::new()?;

    let (_panel_registry, custom_panels) = create_panel_registry_and_panels(&scenario);

    if let Some(backend) = audio_system.active_backend() {
        tracing::info!("🎵 Active audio backend: {}", backend);
    }
    for backend in &audio_system.available_backends() {
        tracing::info!("  - {}", backend);
    }

    let needs_fallback = !tutorial_mode.is_enabled() && data_providers.is_empty();

    let (rendering_awareness, sensor_registry) = initialize_central_nervous_system(&runtime);

    let channel_registry = Arc::new(RwLock::new(standard_channels()));

    let (motor_tx, motor_rx) = mpsc::channel();

    let mut app = PetalTongueApp {
        capabilities,
        graph,
        visual_renderer,
        audio_renderer,
        audio_generator,
        animation_engine,
        current_layout: LayoutAlgorithm::ForceDirected,
        show_audio_panel: scenario
            .as_ref()
            .is_none_or(|s| s.ui_config.show_panels.audio_panel),
        show_capability_panel: false,
        show_controls: scenario
            .as_ref()
            .is_none_or(|s| s.ui_config.show_panels.left_sidebar),
        show_animation: true,
        last_refresh: Instant::now(),
        auto_refresh: true,
        refresh_interval: 5.0,
        tools,
        accessibility_panel: AccessibilityPanel::default(),
        system_dashboard: SystemDashboard::default(),
        show_dashboard: scenario
            .as_ref()
            .is_none_or(|s| s.ui_config.show_panels.system_dashboard),
        audio_system,
        status_reporter,
        keyboard_shortcuts: KeyboardShortcuts::default(),
        adapter_registry,
        trust_dashboard: TrustDashboard::new(),
        show_trust_dashboard: scenario
            .as_ref()
            .is_none_or(|s| s.ui_config.show_panels.trust_dashboard),
        awakening_overlay: AwakeningOverlay::new(),
        rendering_awareness,
        sensor_registry,
        channel_registry,
        frame_count: 0,
        last_display_verification: Instant::now(),
        proprioception: initialize_standard_proprioception(),
        neural_api_provider,
        neural_proprioception_panel: ProprioceptionPanel::new(),
        show_neural_proprioception: false,
        neural_metrics_dashboard: MetricsDashboard::new(),
        show_neural_metrics: false,
        tokio_runtime: runtime,
        graph_canvas: GraphCanvas::new("New Graph".to_string()),
        show_graph_builder: false,
        tick_clock: TickClock::new(TickConfig::default()),
        continuous_mode: false,
        physics_world: petal_tongue_scene::physics::PhysicsWorld::new(),
        animation_player: petal_tongue_scene::animation::AnimationPlayer::new(),
        active_scene: petal_tongue_scene::scene_graph::SceneGraph::new(),
        visualization_state: None,
        last_session_poll: Instant::now(),
        sensor_stream: None,
        interaction_subscribers: None,
        last_broadcast_selection: None,
        custom_panels,
        motor_rx,
        motor_tx,
        show_top_menu: true,
        interaction_bridge: EguiInteractionBridge::new(),
        squirrel_adapter: crate::squirrel_adapter::SquirrelAdapter::new_deferred(),
    };

    finalize_app_startup(&mut app, &scenario, &tutorial_mode, needs_fallback);

    Ok(app)
}

fn discover_data_providers(
    scenario: &Option<crate::scenario::Scenario>,
    scenario_path_for_provider: &Option<std::path::PathBuf>,
    tutorial_mode: &crate::tutorial_mode::TutorialMode,
    runtime: &tokio::runtime::Runtime,
) -> Vec<Box<dyn VisualizationDataProvider>> {
    if let (Some(_scenario), Some(path)) = (scenario, scenario_path_for_provider) {
        tracing::info!("📋 Scenario mode: Loading primals with dynamic schema");
        match petal_tongue_discovery::DynamicScenarioProvider::from_file(path) {
            Ok(provider) => {
                if let Some(version) = provider.version() {
                    tracing::info!("   Schema version: {}", version);
                }
                vec![Box::new(provider) as Box<dyn VisualizationDataProvider>]
            }
            Err(e) => {
                tracing::error!("Failed to create dynamic scenario provider: {}", e);
                tracing::info!("Falling back to static provider...");
                match petal_tongue_discovery::ScenarioVisualizationProvider::from_file(path) {
                    Ok(provider) => vec![Box::new(provider) as Box<dyn VisualizationDataProvider>],
                    Err(e2) => {
                        tracing::error!("Static provider also failed: {}", e2);
                        vec![]
                    }
                }
            }
        }
    } else if tutorial_mode.is_enabled() {
        tracing::info!("📚 Tutorial mode: Using demonstration data");
        #[cfg(feature = "mock")]
        {
            vec![
                Box::new(petal_tongue_discovery::DemoVisualizationProvider::new())
                    as Box<dyn VisualizationDataProvider>,
            ]
        }
        #[cfg(not(feature = "mock"))]
        {
            tracing::info!("💡 Mock feature disabled - start with --features mock for demo data");
            vec![]
        }
    } else {
        runtime.block_on(async {
            match discover_visualization_providers().await {
                Ok(providers) => {
                    if providers.is_empty() {
                        tracing::warn!("No visualization providers discovered");
                        if crate::tutorial_mode::should_fallback(0) {
                            #[cfg(feature = "mock")]
                            {
                                tracing::info!("💡 Using tutorial data as graceful fallback");
                                vec![Box::new(
                                    petal_tongue_discovery::DemoVisualizationProvider::new(),
                                )
                                    as Box<dyn VisualizationDataProvider>]
                            }
                            #[cfg(not(feature = "mock"))]
                            {
                                tracing::info!("💡 Use --features mock for demo data fallback");
                                vec![]
                            }
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
                    if crate::tutorial_mode::should_fallback(0) {
                        #[cfg(feature = "mock")]
                        {
                            tracing::info!("💡 Using tutorial data as graceful fallback");
                            vec![
                                Box::new(petal_tongue_discovery::DemoVisualizationProvider::new())
                                    as Box<dyn VisualizationDataProvider>,
                            ]
                        }
                        #[cfg(not(feature = "mock"))]
                        {
                            tracing::info!("💡 Use --features mock for demo data fallback");
                            vec![]
                        }
                    } else {
                        vec![]
                    }
                }
            }
        })
    }
}

fn discover_neural_api(runtime: &tokio::runtime::Runtime) -> Option<Arc<NeuralApiProvider>> {
    tracing::info!("🧠 Attempting Neural API discovery (central coordinator)...");
    runtime.block_on(async {
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
    })
}

fn create_status_reporter(capabilities: &CapabilityDetector) -> Arc<StatusReporter> {
    let mut reporter = StatusReporter::new();
    if let Ok(status_file) = std::env::var("PETALTONGUE_STATUS_FILE") {
        reporter.enable_status_file(std::path::PathBuf::from(status_file));
    } else {
        reporter.enable_status_file(
            std::path::PathBuf::from(constants::LEGACY_TMP_PREFIX)
                .join(format!("{APP_DIR_NAME}_status.json")),
        );
    }

    let reporter_arc = Arc::new(reporter);
    for modality in &[
        Modality::Visual2D,
        Modality::Audio,
        Modality::Animation,
        Modality::TextDescription,
        Modality::Haptic,
        Modality::VR3D,
    ] {
        let available = capabilities.is_available(*modality);
        let reason_string = capabilities
            .get_status(*modality)
            .map_or_else(|| "Not tested".to_string(), |c| c.reason);
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
}

fn create_renderers(
    graph: Arc<RwLock<GraphEngine>>,
    scenario: &Option<crate::scenario::Scenario>,
) -> (
    Visual2DRenderer,
    AudioSonificationRenderer,
    AudioFileGenerator,
    Arc<RwLock<AnimationEngine>>,
) {
    let mut visual_renderer = Visual2DRenderer::new(Arc::clone(&graph));
    let audio_renderer = AudioSonificationRenderer::new(Arc::clone(&graph));
    let audio_generator = AudioFileGenerator::new();
    let animation_engine = Arc::new(RwLock::new(AnimationEngine::new()));

    visual_renderer.set_animation_engine(Arc::clone(&animation_engine));
    visual_renderer.set_animation_enabled(true);

    if let Some(s) = scenario {
        visual_renderer.set_show_stats(s.ui_config.show_panels.graph_stats);
        if s.ui_config.layout == "canvas-only" || s.mode.contains("paint") {
            visual_renderer.set_interactive_mode(true);
            tracing::info!("✅ Interactive mode enabled for {} scenario", s.mode);
        }
    }

    (
        visual_renderer,
        audio_renderer,
        audio_generator,
        animation_engine,
    )
}

fn create_adapter_registry() -> AdapterRegistry {
    let adapter_registry = AdapterRegistry::new();
    adapter_registry.register(Box::new(EcoPrimalTrustAdapter::new()));
    adapter_registry.register(Box::new(EcoPrimalFamilyAdapter::new()));
    adapter_registry.register(Box::new(EcoPrimalCapabilityAdapter::new()));
    tracing::info!(
        "Registered {} property adapters",
        adapter_registry.adapter_count()
    );
    tracing::debug!("Adapters: {:?}", adapter_registry.adapter_names());
    adapter_registry
}

fn create_tool_manager() -> ToolManager {
    let mut tm = ToolManager::new();
    tm.register_tool(Box::new(SystemMonitorTool::default()));
    tm.register_tool(Box::new(ProcessViewerTool::default()));
    tm.register_tool(Box::new(GraphMetricsPlotter::default()));
    tm
}

fn create_panel_registry_and_panels(
    scenario: &Option<crate::scenario::Scenario>,
) -> (PanelRegistry, Vec<Box<dyn PanelInstance>>) {
    let mut panel_registry = PanelRegistry::new();
    #[cfg(feature = "doom")]
    panel_registry.register(create_doom_factory());
    panel_registry.register(Arc::new(crate::panels::MetricsPanelFactory::new()));
    panel_registry.register(Arc::new(crate::panels::ProprioceptionPanelFactory::new()));

    tracing::info!("✅ Panel registry initialized");
    tracing::info!(
        "   Available panel types: {:?}",
        panel_registry.available_types()
    );

    let mut custom_panels: Vec<Box<dyn PanelInstance>> = Vec::new();
    if let Some(s) = scenario {
        for panel_config in &s.ui_config.custom_panels {
            match panel_registry.create(panel_config) {
                Ok(panel) => {
                    tracing::info!(
                        "✅ Created custom panel: {} (type: {})",
                        panel_config.title,
                        panel_config.panel_type
                    );
                    custom_panels.push(panel);
                }
                Err(e) => {
                    tracing::error!("❌ Failed to create panel '{}': {}", panel_config.title, e);
                }
            }
        }
    }
    tracing::info!(
        "✅ Custom panels initialized: {} panels",
        custom_panels.len()
    );

    (panel_registry, custom_panels)
}

fn initialize_central_nervous_system(
    runtime: &tokio::runtime::Runtime,
) -> (
    Arc<RwLock<petal_tongue_core::RenderingAwareness>>,
    Arc<RwLock<petal_tongue_core::SensorRegistry>>,
) {
    let rendering_awareness = Arc::new(RwLock::new(petal_tongue_core::RenderingAwareness::new()));
    let sensor_registry = Arc::new(RwLock::new(petal_tongue_core::SensorRegistry::new()));
    tracing::info!("🧠 Central nervous system initialized");

    let sensor_registry_clone = Arc::clone(&sensor_registry);
    runtime.block_on(async {
        if let Err(e) = crate::sensor_discovery::discover_all_sensors(sensor_registry_clone).await {
            tracing::error!("Sensor discovery failed: {}", e);
        }
    });

    if !crate::sensor_discovery::verify_essential_sensors(&sensor_registry) {
        tracing::warn!("⚠️  Running with limited sensor capabilities");
    }

    tracing::info!("🔄 Sensory feedback via egui input events (bidirectional loop active)");

    (rendering_awareness, sensor_registry)
}

fn finalize_app_startup(
    app: &mut PetalTongueApp,
    scenario: &Option<crate::scenario::Scenario>,
    tutorial_mode: &crate::tutorial_mode::TutorialMode,
    needs_fallback: bool,
) {
    use petal_tongue_core::MotorCommand;

    // Awakening: respect env var, then scenario config
    let env_awakening = std::env::var("AWAKENING_ENABLED")
        .ok()
        .and_then(|v| v.parse::<bool>().ok());
    let scenario_awakening = scenario.as_ref().map(|s| s.ui_config.awakening_enabled);
    let awakening_enabled = env_awakening.unwrap_or_else(|| scenario_awakening.unwrap_or(true));

    if awakening_enabled {
        tracing::info!("🌸 Awakening experience enabled");
        app.awakening_overlay.start();
    }

    tracing::info!("🎵 Initializing startup audio...");
    let startup_audio = crate::startup_audio::StartupAudio::new();
    if startup_audio.has_startup_music() {
        tracing::info!(
            "🎵 Startup music found: {:?}",
            startup_audio.startup_music_path()
        );
    }
    startup_audio.play(&app.audio_system);

    tracing::info!("📊 Writing initial status file for AI observability...");
    app.status_reporter.update_health("healthy");
    app.status_reporter.force_write();

    if let Some(loaded_scenario) = scenario {
        tracing::info!(
            "📋 Loading {} primals and {} edges from scenario",
            loaded_scenario.primal_count(),
            loaded_scenario.edge_count()
        );
        let primals = loaded_scenario.to_primal_infos();
        let Ok(mut graph) = app.graph.write() else {
            tracing::error!("graph lock poisoned");
            return;
        };
        graph.clear();
        for primal in &primals {
            graph.add_node(primal.clone());
        }
        for edge in &loaded_scenario.edges {
            graph.add_edge(edge.clone());
        }
        tracing::info!(
            "✅ Scenario loaded: {} nodes, {} edges",
            primals.len(),
            loaded_scenario.edge_count()
        );
        drop(graph);

        // Wire remaining PanelVisibility fields via motor commands
        let panels = &loaded_scenario.ui_config.show_panels;
        app.show_top_menu = panels.top_menu;
        app.show_neural_proprioception = panels.proprioception && app.show_neural_proprioception;

        // Apply scenario mode through the motor channel (efferent)
        let mode = &loaded_scenario.mode;
        if !mode.is_empty() && mode != "live-ecosystem" && mode != "doom-showcase" {
            let cmds = crate::mode_presets::commands_for_mode(mode);
            if !cmds.is_empty() {
                tracing::info!(
                    "🎛️  Applying scenario mode '{mode}' ({} commands)",
                    cmds.len()
                );
                for cmd in cmds {
                    super::events::apply_motor_command(app, cmd);
                }
            }
        }

        // Apply initial zoom via motor command
        let zoom_str = &loaded_scenario.ui_config.initial_zoom;
        if zoom_str == "fit" {
            super::events::apply_motor_command(app, MotorCommand::FitToView);
        } else if let Ok(level) = zoom_str.parse::<f32>() {
            super::events::apply_motor_command(app, MotorCommand::SetZoom { level });
        }
    } else if tutorial_mode.is_enabled() {
        tutorial_mode.load_into_graph(Arc::clone(&app.graph), app.current_layout);
    } else if needs_fallback {
        crate::tutorial_mode::TutorialMode::create_fallback_scenario(
            Arc::clone(&app.graph),
            app.current_layout,
        );
    } else {
        app.refresh_graph_data();
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_create_adapter_registry() {
        let registry = create_adapter_registry();
        assert_eq!(registry.adapter_count(), 3);
        let names = registry.adapter_names();
        assert!(names.iter().any(|t| t.contains("trust")));
        assert!(names.iter().any(|t| t.contains("family")));
        assert!(names.iter().any(|t| t.contains("capabilities")));
    }

    #[test]
    fn test_create_tool_manager() {
        let tm = create_tool_manager();
        assert!(!tm.tools().is_empty());
    }

    #[test]
    fn test_create_renderers_empty_scenario() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let (visual, audio, _gen, anim) = create_renderers(graph, &None);
        let _ = visual;
        let _ = audio;
        let _guard = anim.read().unwrap();
    }

    #[test]
    fn test_create_status_reporter() {
        let caps = CapabilityDetector::default();
        let reporter = create_status_reporter(&caps);
        assert!(Arc::strong_count(&reporter) >= 1);
    }

    #[test]
    fn test_adapter_registry_names() {
        let registry = create_adapter_registry();
        let names = registry.adapter_names();
        assert_eq!(names.len(), 3);
    }

    #[test]
    fn test_tool_manager_has_graph_metrics() {
        let tm = create_tool_manager();
        let tools = tm.tools();
        assert!(tools.iter().any(|t| t.metadata().name.contains("Graph")));
    }

    #[test]
    fn test_create_panel_registry_empty_scenario() {
        let (registry, custom_panels) = create_panel_registry_and_panels(&None);
        assert!(!registry.available_types().is_empty());
        assert!(custom_panels.is_empty());
    }

    #[test]
    fn test_create_panel_registry_with_scenario_no_custom_panels() {
        use crate::scenario::config::UiConfig;
        use crate::scenario::ecosystem::Ecosystem;
        use crate::scenario::sensory::SensoryConfig;
        use crate::scenario::types::{NeuralApiConfig, Scenario};

        let scenario = Scenario {
            name: "Test".to_string(),
            description: "Test scenario".to_string(),
            version: "2.0.0".to_string(),
            mode: "test".to_string(),
            ui_config: UiConfig {
                custom_panels: vec![],
                ..Default::default()
            },
            ecosystem: Ecosystem::default(),
            neural_api: NeuralApiConfig::default(),
            sensory_config: SensoryConfig::default(),
            edges: vec![],
        };
        let (registry, custom_panels) = create_panel_registry_and_panels(&Some(scenario));
        assert!(!registry.available_types().is_empty());
        assert!(custom_panels.is_empty());
    }

    #[test]
    fn test_create_panel_registry_with_metrics_panel() {
        use crate::scenario::config::{CustomPanelConfig, UiConfig};
        use crate::scenario::ecosystem::Ecosystem;
        use crate::scenario::sensory::SensoryConfig;
        use crate::scenario::types::{NeuralApiConfig, Scenario};

        let scenario = Scenario {
            name: "Test".to_string(),
            description: "Test".to_string(),
            version: "2.0.0".to_string(),
            mode: "test".to_string(),
            ui_config: UiConfig {
                custom_panels: vec![CustomPanelConfig {
                    panel_type: "metrics".to_string(),
                    title: "Metrics".to_string(),
                    width: Some(400),
                    height: Some(300),
                    fullscreen: false,
                    config: serde_json::Value::Null,
                }],
                ..Default::default()
            },
            ecosystem: Ecosystem::default(),
            neural_api: NeuralApiConfig::default(),
            sensory_config: SensoryConfig::default(),
            edges: vec![],
        };
        let (_registry, custom_panels) = create_panel_registry_and_panels(&Some(scenario));
        assert_eq!(custom_panels.len(), 1);
        assert_eq!(custom_panels[0].title(), "System Metrics");
    }

    #[test]
    fn test_create_renderers_with_scenario_interactive() {
        use crate::scenario::config::UiConfig;
        use crate::scenario::ecosystem::Ecosystem;
        use crate::scenario::sensory::SensoryConfig;
        use crate::scenario::types::{NeuralApiConfig, Scenario};

        let scenario = Scenario {
            name: "Paint".to_string(),
            description: "Paint mode".to_string(),
            version: "2.0.0".to_string(),
            mode: "paint".to_string(),
            ui_config: UiConfig {
                layout: "canvas-only".to_string(),
                ..Default::default()
            },
            ecosystem: Ecosystem::default(),
            neural_api: NeuralApiConfig::default(),
            sensory_config: SensoryConfig::default(),
            edges: vec![],
        };
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let (visual, _audio, _gen, _anim) = create_renderers(graph, &Some(scenario));
        assert!(visual.is_interactive());
    }

    #[test]
    fn test_create_renderers_with_scenario_show_stats() {
        use crate::scenario::config::{PanelVisibility, UiConfig};
        use crate::scenario::ecosystem::Ecosystem;
        use crate::scenario::sensory::SensoryConfig;
        use crate::scenario::types::{NeuralApiConfig, Scenario};

        let scenario = Scenario {
            name: "Stats".to_string(),
            description: "Stats scenario".to_string(),
            version: "2.0.0".to_string(),
            mode: "test".to_string(),
            ui_config: UiConfig {
                show_panels: PanelVisibility {
                    graph_stats: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            ecosystem: Ecosystem::default(),
            neural_api: NeuralApiConfig::default(),
            sensory_config: SensoryConfig::default(),
            edges: vec![],
        };
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let (visual, _audio, _gen, _anim) = create_renderers(graph, &Some(scenario));
        assert!(visual.show_stats());
    }

    #[test]
    fn test_create_panel_registry_with_proprioception_panel() {
        use crate::scenario::config::{CustomPanelConfig, UiConfig};
        use crate::scenario::ecosystem::Ecosystem;
        use crate::scenario::sensory::SensoryConfig;
        use crate::scenario::types::{NeuralApiConfig, Scenario};

        let scenario = Scenario {
            name: "Proprio".to_string(),
            description: "Proprioception".to_string(),
            version: "2.0.0".to_string(),
            mode: "test".to_string(),
            ui_config: UiConfig {
                custom_panels: vec![CustomPanelConfig {
                    panel_type: "proprioception".to_string(),
                    title: "Proprio".to_string(),
                    width: Some(400),
                    height: Some(300),
                    fullscreen: false,
                    config: serde_json::Value::Null,
                }],
                ..Default::default()
            },
            ecosystem: Ecosystem::default(),
            neural_api: NeuralApiConfig::default(),
            sensory_config: SensoryConfig::default(),
            edges: vec![],
        };
        let (_registry, custom_panels) = create_panel_registry_and_panels(&Some(scenario));
        assert_eq!(custom_panels.len(), 1);
    }

    #[test]
    fn test_create_status_reporter_modalities() {
        let caps = CapabilityDetector::default();
        let reporter = create_status_reporter(&caps);
        assert!(Arc::strong_count(&reporter) >= 1);
    }

    #[test]
    fn test_create_tool_manager_has_system_monitor() {
        let tm = create_tool_manager();
        let tools = tm.tools();
        assert!(tools.iter().any(|t| t.metadata().name.contains("System")));
    }

    #[test]
    fn test_create_tool_manager_has_process_viewer() {
        let tm = create_tool_manager();
        let tools = tm.tools();
        assert!(tools.iter().any(|t| t.metadata().name.contains("Process")));
    }

    #[test]
    fn test_adapter_registry_adapter_count() {
        let registry = create_adapter_registry();
        assert!(registry.adapter_count() >= 3);
    }

    #[test]
    fn test_create_panel_registry_doom_when_feature() {
        let (registry, _) = create_panel_registry_and_panels(&None);
        let types = registry.available_types();
        assert!(!types.is_empty());
    }
}
