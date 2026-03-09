// SPDX-License-Identifier: AGPL-3.0-only
//! Main application logic for petalTongue UI
//!
//! This is the **`EguiGUI` Modality** implementation - the native desktop GUI.
//!
//! ## Architecture Note
//!
//! This module represents Tier 3 (Enhancement) GUI modality using egui/eframe.
//! Rather than extracting to a separate modality file, we recognize that this
//! IS the `EguiGUI` implementation. This is a "smart refactor" approach - we don't
//! split code just to split it; the current organization is clean and working.

mod init;
mod sensory;

use crate::accessibility_panel::AccessibilityPanel;
use crate::audio::AudioSystemV2;
use crate::awakening_overlay::AwakeningOverlay;
use crate::graph_canvas::GraphCanvas;
use crate::graph_manager::GraphManagerPanel;
use crate::keyboard_shortcuts::KeyboardShortcuts;
use crate::metrics_dashboard::MetricsDashboard;
use crate::node_palette::NodePalette;
use crate::panel_registry::{PanelInstance, PanelRegistry};
use crate::property_panel::PropertyPanel;
use crate::proprioception::ProprioceptionSystem;
use crate::proprioception_panel::ProprioceptionPanel;
use crate::system_dashboard::SystemDashboard;
use crate::tool_integration::ToolManager;
use crate::trust_dashboard::TrustDashboard;
use anyhow::Result as AnyhowResult;
use petal_tongue_adapters::AdapterRegistry;
use petal_tongue_animation::AnimationEngine;
use petal_tongue_api::BiomeOSClient;
use petal_tongue_core::{
    CapabilityDetector, GraphEngine, InstanceId, LayoutAlgorithm, RenderingAwareness,
    SensorRegistry, SessionManager,
};
use petal_tongue_discovery::{NeuralApiProvider, VisualizationDataProvider};
use petal_tongue_graph::{AudioFileGenerator, AudioSonificationRenderer, Visual2DRenderer};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// The main petalTongue UI application
#[expect(clippy::struct_excessive_bools)]
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
    #[expect(dead_code)]
    data_providers: Vec<Box<dyn VisualizationDataProvider>>,
    /// Legacy `BiomeOS` client (DEPRECATED - kept for backward compatibility)
    #[deprecated(note = "Use data_providers instead - biomeOS is just another primal!")]
    #[allow(dead_code)]
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

    /// Tool manager (handles all external tools dynamically)
    tools: ToolManager,

    /// Accessibility settings panel
    accessibility_panel: AccessibilityPanel,

    /// Live system dashboard
    system_dashboard: SystemDashboard,
    /// Show system dashboard sidebar
    show_dashboard: bool,

    /// Audio system for UI sounds and data sonification
    audio_system: AudioSystemV2,

    /// Status reporter (makes petalTongue observable to AI and external systems)
    pub status_reporter: Arc<crate::status_reporter::StatusReporter>,

    /// Keyboard shortcuts system
    keyboard_shortcuts: KeyboardShortcuts,

    /// Property adapter registry (ecosystem-agnostic rendering)
    adapter_registry: AdapterRegistry,

    /// Trust status dashboard
    trust_dashboard: TrustDashboard,
    /// Show trust dashboard panel
    show_trust_dashboard: bool,

    /// Awakening overlay (visual flower animation + tutorial transition)
    awakening_overlay: AwakeningOverlay,

    #[expect(dead_code)]
    session_manager: Option<SessionManager>,
    #[expect(dead_code)]
    instance_id: Option<InstanceId>,

    /// Rendering awareness - motor + sensory feedback loop
    rendering_awareness: Arc<RwLock<RenderingAwareness>>,
    /// Sensor registry - discovered input peripherals
    sensor_registry: Arc<RwLock<SensorRegistry>>,
    /// Frame counter for tracking motor commands
    frame_count: u64,
    /// Last display verification time (Phase 4)
    last_display_verification: Instant,
    /// Complete self-awareness system (output + input + bidirectional feedback)
    proprioception: ProprioceptionSystem,

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

    /// Graph Builder canvas (interactive visual graph construction)
    graph_canvas: GraphCanvas,
    /// Node palette (available node types)
    #[allow(dead_code)]
    node_palette: NodePalette,
    /// Property panel (node parameter editor)
    #[allow(dead_code)]
    property_panel: PropertyPanel,
    /// Graph manager (save/load/execute via Neural API)
    #[allow(dead_code)]
    graph_manager: GraphManagerPanel,
    /// Show Graph Builder window
    show_graph_builder: bool,

    /// Adaptive UI manager (device-specific rendering) - DEPRECATED
    #[allow(dead_code)]
    adaptive_ui: crate::adaptive_ui::AdaptiveUIManager,

    /// Sensory UI manager (capability-based rendering)
    #[allow(dead_code)]
    sensory_ui: Option<crate::sensory_ui::SensoryUIManager>,
    /// Use sensory UI instead of adaptive UI (feature flag for migration)
    #[allow(dead_code)]
    use_sensory_ui: bool,

    /// Panel registry for custom panel types (Doom, web, video, etc.)
    #[allow(dead_code)]
    panel_registry: PanelRegistry,
    /// Active custom panels
    custom_panels: Vec<Box<dyn PanelInstance>>,
}

impl PetalTongueApp {
    /// Create a new application with a shared graph from `DataService`
    ///
    /// This ensures the GUI uses the SAME data as all other UI modes (TUI, Web, etc.)
    /// TRUE PRIMAL: Zero data duplication!
    pub fn new_with_shared_graph(
        _cc: &eframe::CreationContext<'_>,
        scenario_path: Option<std::path::PathBuf>,
        rendering_caps: petal_tongue_core::RenderingCapabilities,
        shared_graph: Arc<RwLock<GraphEngine>>,
    ) -> AnyhowResult<Self> {
        init::create_app(scenario_path, rendering_caps, shared_graph)
    }

    /// Create a new application (compatibility wrapper for standalone binary)
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        scenario_path: Option<std::path::PathBuf>,
        rendering_caps: petal_tongue_core::RenderingCapabilities,
    ) -> AnyhowResult<Self> {
        tracing::info!("⚠️  Creating standalone graph (not shared with DataService)");
        let shared_graph = Arc::new(RwLock::new(GraphEngine::new()));
        Self::new_with_shared_graph(cc, scenario_path, rendering_caps, shared_graph)
    }

    /// Refresh graph data from `DataService`
    pub(crate) fn refresh_graph_data(&mut self) {
        tracing::debug!("✅ Graph refresh requested - DataService handles this automatically");
        self.last_refresh = Instant::now();
    }
}

impl eframe::App for PetalTongueApp {
    #[expect(clippy::too_many_lines)]
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        sensory::process_sensory_feedback(self, ctx);

        if self.awakening_overlay.is_active() {
            let delta_time = ctx.input(|i| i.stable_dt);
            if let Err(e) = self.awakening_overlay.update(delta_time) {
                tracing::error!("Awakening overlay update error: {}", e);
            }
            self.awakening_overlay.render(ctx);
            if self.awakening_overlay.should_transition_to_tutorial() {
                tracing::info!("🎓 Transitioning to tutorial mode");
                let tutorial = crate::tutorial_mode::TutorialMode::new();
                if tutorial.is_enabled() {
                    tutorial.load_into_graph(self.graph.clone(), self.current_layout);
                }
            }
            ctx.request_repaint();
            return;
        }

        ctx.input(|i| {
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

        self.system_dashboard
            .set_audio_enabled(self.accessibility_panel.settings.audio_enabled);
        self.system_dashboard
            .set_audio_volume(self.accessibility_panel.settings.audio_volume);

        if self.show_animation
            && let Ok(mut engine) = self.animation_engine.write()
        {
            engine.update();
        }

        let palette = self.accessibility_panel.get_palette();
        let mut style = (*ctx.style()).clone();
        style.visuals.dark_mode = true;
        style.visuals.override_text_color = Some(palette.text);
        style.visuals.window_fill = palette.background;
        style.visuals.panel_fill = palette.background_alt;
        ctx.set_style(style);

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

        if self.show_capability_panel {
            crate::app_panels::render_capability_panel(ctx, &palette, &self.capabilities);
        }

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
                    self.system_dashboard.render_compact(
                        ui,
                        &palette,
                        font_scale,
                        Some(&self.audio_system),
                    );
                    ui.add_space(8.0);
                    crate::system_dashboard::SystemDashboard::render_sensory_status(
                        ui,
                        &palette,
                        font_scale,
                        &self.rendering_awareness,
                        &self.sensor_registry,
                    );
                    ui.add_space(8.0);
                    crate::system_dashboard::SystemDashboard::render_proprioception_status(
                        ui,
                        &palette,
                        font_scale,
                        &mut self.proprioception,
                    );
                });
        }

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

        if self.show_neural_proprioception {
            egui::Window::new("🧠 Neural API Proprioception")
                .default_width(500.0)
                .default_height(600.0)
                .default_pos([100.0, 100.0])
                .show(ctx, |ui| {
                    if let Some(provider) = &self.neural_api_provider {
                        self.tokio_runtime.block_on(async {
                            self.neural_proprioception_panel
                                .update(provider.as_ref())
                                .await;
                        });
                        self.neural_proprioception_panel.render(ui);
                    } else {
                        ui.label("❌ Neural API not available");
                        ui.label("Start biomeOS nucleus to enable proprioception data.");
                    }
                });
        }

        if self.show_neural_metrics {
            egui::Window::new("📊 Neural API Metrics")
                .default_width(600.0)
                .default_height(500.0)
                .default_pos([150.0, 150.0])
                .show(ctx, |ui| {
                    if let Some(provider) = &self.neural_api_provider {
                        self.tokio_runtime.block_on(async {
                            self.neural_metrics_dashboard
                                .update(provider.as_ref())
                                .await;
                        });
                        self.neural_metrics_dashboard.render(ui);
                    } else {
                        ui.label("❌ Neural API not available");
                        ui.label("Start biomeOS nucleus to enable metrics data.");
                    }
                });
        }

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
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.heading("Canvas");
                                ui.separator();
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

        for (idx, panel) in self.custom_panels.iter_mut().enumerate() {
            egui::Window::new(panel.title())
                .id(egui::Id::new(format!("custom_panel_{idx}")))
                .default_width(640.0)
                .default_height(480.0)
                .resizable(true)
                .show(ctx, |ui| {
                    panel.update();
                    panel.render(ui);
                });
        }

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

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(tool) = self.tools.visible_tool() {
                tool.render_panel(ui);
            } else {
                self.visual_renderer.render(ui);
            }
        });

        self.accessibility_panel.show(ctx);
        self.keyboard_shortcuts.render_help(ctx, &palette);

        if self.auto_refresh {
            let elapsed = self.last_refresh.elapsed();
            if elapsed >= Duration::from_secs_f32(self.refresh_interval) {
                self.refresh_graph_data();
            }
            ctx.request_repaint();
        }
    }
}
