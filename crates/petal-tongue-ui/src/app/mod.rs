// SPDX-License-Identifier: AGPL-3.0-only
//! Main application logic for petalTongue UI
//!
//! This is the **`EguiGUI` Modality** implementation - the native desktop GUI.
//!
//! ## Architecture (SMART_REFACTORING_POLICY)
//!
//! This module is the application root - naturally centralized. Per
//! `docs/operations/SMART_REFACTORING_POLICY.md`, app.rs may legitimately be
//! large because it is the UI integration point.
//!
//! **Why we keep it cohesive:**
//! - Single responsibility: Application state and lifecycle
//! - Single type: `PetalTongueApp` with impl blocks
//! - All panels coordinate through single state; splitting would create circular deps
//! - Event flow is linear: sensory → motor drain → update → panel dispatch
//!
//! **What we extracted:**
//! - `init.rs` — scenario loading, provider discovery, renderer setup
//! - `sensory.rs` — sensory feedback and display verification
//! - `layout.rs` — pure layout algorithm parsing (testable)
//! - `app_panels/` — panel rendering (top menu, controls, audio, capability, primal details)

mod init;
mod layout;
mod sensory;

use crate::accessibility_panel::AccessibilityPanel;
use crate::audio::AudioSystemV2;
use crate::awakening_overlay::AwakeningOverlay;
use crate::graph_canvas::GraphCanvas;
use crate::keyboard_shortcuts::KeyboardShortcuts;
use crate::metrics_dashboard::MetricsDashboard;
use crate::panel_registry::PanelInstance;
use crate::proprioception::ProprioceptionSystem;
use crate::proprioception_panel::ProprioceptionPanel;
use crate::system_dashboard::SystemDashboard;
use crate::tool_integration::ToolManager;
use crate::trust_dashboard::TrustDashboard;
use anyhow::Result as AnyhowResult;
use petal_tongue_adapters::AdapterRegistry;
use petal_tongue_animation::AnimationEngine;
use petal_tongue_core::{
    CapabilityDetector, GraphEngine, LayoutAlgorithm, MotorCommand, PanelId, RenderingAwareness,
    SensorRegistry,
};
use petal_tongue_discovery::NeuralApiProvider;
use petal_tongue_graph::{AudioFileGenerator, AudioSonificationRenderer, Visual2DRenderer};
use std::sync::mpsc;
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

    /// Rendering awareness - motor + sensory feedback loop
    rendering_awareness: Arc<RwLock<RenderingAwareness>>,
    /// Sensor registry - discovered input peripherals
    sensor_registry: Arc<RwLock<SensorRegistry>>,
    /// Channel registry - local self-awareness of signal channels (SAME DAVE)
    channel_registry: Arc<RwLock<petal_tongue_core::channel::ChannelRegistry>>,
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
    /// Show Graph Builder window
    show_graph_builder: bool,

    /// Active custom panels
    custom_panels: Vec<Box<dyn PanelInstance>>,

    // === SAME DAVE: Efferent motor command channel ===
    /// Receiver for motor commands (efferent channel sink).
    /// Drained every frame in `update()` to apply UI state changes.
    motor_rx: mpsc::Receiver<MotorCommand>,
    /// Sender for motor commands (efferent channel source).
    /// Cloned to IPC server, scenario loader, mode presets, etc.
    motor_tx: mpsc::Sender<MotorCommand>,
    /// Show top menu bar (controllable via motor commands)
    show_top_menu: bool,
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

    /// Get a clone of the motor command sender (for IPC, init, external control).
    #[must_use]
    pub fn motor_sender(&self) -> mpsc::Sender<MotorCommand> {
        self.motor_tx.clone()
    }

    /// Get a handle to the shared graph engine (for IPC server).
    #[must_use]
    pub fn graph_handle(&self) -> Arc<RwLock<petal_tongue_core::GraphEngine>> {
        Arc::clone(&self.graph)
    }

    /// Drain all pending motor commands and apply them to UI state.
    fn drain_motor_commands(&mut self) {
        while let Ok(cmd) = self.motor_rx.try_recv() {
            self.apply_motor_command(cmd);
        }
    }

    /// Apply a single motor command to UI state (efferent signal → effector).
    fn apply_motor_command(&mut self, cmd: MotorCommand) {
        let cmd_description = format!("{cmd:?}");
        match cmd {
            MotorCommand::RenderFrame { .. }
            | MotorCommand::UpdateDisplay
            | MotorCommand::ClearDisplay => {
                // Rendering commands handled by the existing awareness system
            }
            MotorCommand::SetPanelVisibility { panel, visible } => {
                match panel {
                    PanelId::LeftSidebar => self.show_controls = visible,
                    PanelId::RightSidebar => {
                        self.show_audio_panel = visible;
                        self.show_dashboard = visible;
                        self.show_trust_dashboard = visible;
                    }
                    PanelId::TopMenu => self.show_top_menu = visible,
                    PanelId::SystemDashboard => self.show_dashboard = visible,
                    PanelId::AudioPanel => self.show_audio_panel = visible,
                    PanelId::TrustDashboard => self.show_trust_dashboard = visible,
                    PanelId::Proprioception => self.show_neural_proprioception = visible,
                    PanelId::GraphStats => self.visual_renderer.set_show_stats(visible),
                    PanelId::Custom(_) => {}
                }
                tracing::debug!("Motor: SetPanelVisibility({panel:?}, {visible})");
            }
            MotorCommand::SetZoom { level } => {
                self.visual_renderer.set_zoom(level);
                tracing::debug!("Motor: SetZoom({level})");
            }
            MotorCommand::FitToView => {
                self.visual_renderer.fit_to_view(&self.graph);
                tracing::debug!("Motor: FitToView");
            }
            MotorCommand::Navigate { ref target_node } => {
                self.visual_renderer
                    .navigate_to_node(target_node, &self.graph);
                tracing::debug!("Motor: Navigate({target_node})");
            }
            MotorCommand::SelectNode { ref node_id } => {
                if let Some(id) = node_id {
                    self.visual_renderer.select_node(Some(id));
                } else {
                    self.visual_renderer.select_node(None::<&str>);
                }
                tracing::debug!("Motor: SelectNode({node_id:?})");
            }
            MotorCommand::SetLayout { ref algorithm } => {
                let layout = layout::layout_from_str(algorithm);
                self.current_layout = layout;
                if let Ok(mut graph) = self.graph.write() {
                    graph.set_layout(layout);
                }
                tracing::debug!("Motor: SetLayout({algorithm})");
            }
            MotorCommand::SetMode { ref mode } => {
                tracing::info!("Motor: SetMode({mode})");
                self.neural_proprioception_panel.set_current_mode(mode);
                let commands = crate::mode_presets::commands_for_mode(mode);
                for sub_cmd in commands {
                    self.apply_motor_command(sub_cmd);
                }
            }
            MotorCommand::SetAwakening { enabled } => {
                if !enabled {
                    self.awakening_overlay.skip();
                }
                tracing::debug!("Motor: SetAwakening({enabled})");
            }
            MotorCommand::LoadScenario { ref path } => {
                tracing::info!("Motor: LoadScenario({path})");
            }
        }
        self.neural_proprioception_panel
            .record_motor_command(&cmd_description);
    }
}

impl eframe::App for PetalTongueApp {
    #[expect(clippy::too_many_lines)]
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        sensory::process_sensory_feedback(self, ctx);
        self.drain_motor_commands();

        if let Ok(mut reg) = self.channel_registry.write() {
            if ctx.input(|i| !i.events.is_empty()) {
                if let Some(ch) = reg.get_mut("keyboard-afferent") {
                    ch.record_signal_in();
                    ch.record_signal_out();
                }
            }
            if ctx.input(|i| i.pointer.any_click() || i.pointer.any_down()) {
                if let Some(ch) = reg.get_mut("pointer-afferent") {
                    ch.record_signal_in();
                    ch.record_signal_out();
                }
            }
            if let Some(ch) = reg.get_mut("visual-efferent") {
                ch.record_signal_in();
                ch.record_signal_out();
            }
        }

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

        if self.show_top_menu {
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
        } // show_top_menu

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
            if let Ok(reg) = self.channel_registry.read() {
                let snapshots = reg.snapshots();
                let afferent: Vec<_> = snapshots
                    .iter()
                    .filter(|s| s.direction == petal_tongue_core::ChannelDirection::Afferent)
                    .cloned()
                    .collect();
                let efferent: Vec<_> = snapshots
                    .iter()
                    .filter(|s| s.direction == petal_tongue_core::ChannelDirection::Efferent)
                    .cloned()
                    .collect();
                self.neural_proprioception_panel
                    .merge_local_channels(afferent, efferent);
            }
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
                        self.neural_proprioception_panel.render(ui);
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

#[cfg(test)]
mod tests {
    #[test]
    fn mode_presets_produce_commands() {
        let cmds = crate::mode_presets::commands_for_mode("clinical");
        assert!(!cmds.is_empty());
        let cmds = crate::mode_presets::commands_for_mode("developer");
        assert!(!cmds.is_empty());
    }
}
