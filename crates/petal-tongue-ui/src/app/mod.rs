// SPDX-License-Identifier: AGPL-3.0-or-later
//! Main application logic for petalTongue UI
//!
//! This is the **`EguiGUI` Modality** implementation - the native desktop display.
//!
//! ## Architecture (`SMART_REFACTORING_POLICY`)
//!
//! This module is the application root - naturally centralized. Per smart
//! refactoring policy, app.rs may legitimately be large because it is the UI
//! integration point.
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

mod events;
mod init;
mod layout;
mod panel_init;
mod panels;
mod provider_init;
mod scenario_init;
mod sensory;
mod update;

use crate::accessibility_panel::AccessibilityPanel;
use crate::audio::AudioSystemV2;
use crate::awakening_overlay::AwakeningOverlay;
use crate::error::Result;
use crate::graph_canvas::GraphCanvas;
use crate::interaction_bridge::EguiInteractionBridge;
use crate::keyboard_shortcuts::KeyboardShortcuts;
use crate::metrics_dashboard::MetricsDashboard;
use crate::panel_registry::PanelInstance;
use crate::proprioception::ProprioceptionSystem;
use crate::proprioception_panel::ProprioceptionPanel;
use crate::system_dashboard::SystemDashboard;
use crate::tool_integration::ToolManager;
use crate::trust_dashboard::TrustDashboard;
use petal_tongue_adapters::AdapterRegistry;
use petal_tongue_animation::AnimationEngine;
use petal_tongue_core::{
    CapabilityDetector, FrameIntrospection, GraphEngine, InteractionCapability, InteractionKind,
    LayoutAlgorithm, MotorCommand, PanelId, PanelKind, PanelSnapshot, RenderingAwareness,
    SensorRegistry,
};
use petal_tongue_discovery::NeuralApiProvider;
use petal_tongue_graph::{AudioFileGenerator, AudioSonificationRenderer, Visual2DRenderer};
use petal_tongue_scene::game_loop::TickClock;
use std::sync::mpsc;
use std::sync::{Arc, RwLock};
use std::time::Instant;

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

    // === Game loop (fixed-timestep tick for physics/animation) ===
    /// Fixed-timestep tick clock for continuous rendering
    tick_clock: TickClock,
    /// Whether continuous 60 Hz mode is active
    continuous_mode: bool,
    /// Physics world for fixed-timestep simulation (barraCuda delegates here)
    physics_world: petal_tongue_scene::physics::PhysicsWorld,
    /// Animation player for scene-graph keyframe interpolation
    animation_player: petal_tongue_scene::animation::AnimationPlayer,
    /// Active scene graph — the single source of truth for all rendering
    active_scene: petal_tongue_scene::scene_graph::SceneGraph,

    // === IPC visualization bridge ===
    /// Shared visualization state (written by IPC server, read by UI)
    visualization_state: Option<Arc<RwLock<petal_tongue_ipc::VisualizationState>>>,
    /// Timestamp of last session poll (to detect changes)
    last_session_poll: Instant,

    // === IPC sensor/interaction bridge ===
    /// Shared sensor stream registry (UI broadcasts, IPC subscribers poll)
    sensor_stream: Option<Arc<RwLock<petal_tongue_ipc::SensorStreamRegistry>>>,
    /// Shared interaction subscriber registry (UI broadcasts selection changes)
    interaction_subscribers: Option<Arc<RwLock<petal_tongue_ipc::InteractionSubscriberRegistry>>>,
    /// Push-delivery sender for callback dispatches from GUI-originated events (PT-06)
    callback_tx: Option<tokio::sync::mpsc::UnboundedSender<petal_tongue_ipc::CallbackDispatch>>,
    /// Previously selected node (for change-detection-based broadcasting)
    last_broadcast_selection: Option<String>,

    // === SAME DAVE: Efferent motor command channel ===
    /// Receiver for motor commands (efferent channel sink).
    /// Drained every frame in `update()` to apply UI state changes.
    motor_rx: mpsc::Receiver<MotorCommand>,
    /// Sender for motor commands (efferent channel source).
    /// Cloned to IPC server, scenario loader, mode presets, etc.
    motor_tx: mpsc::Sender<MotorCommand>,
    /// Show top menu bar (controllable via motor commands)
    show_top_menu: bool,

    /// Bridge between egui events and the `InteractionEngine` (inverse pipeline for hit-target registration)
    interaction_bridge: EguiInteractionBridge,
    /// AI interaction adapter for AI-driven interaction commands
    ai_adapter: crate::ai_adapter::AiAdapter,
}

impl PetalTongueApp {
    /// Create a new application with a shared graph from `DataService`
    ///
    /// This ensures the display uses the SAME data as all other UI modes (TUI, Web, etc.)
    /// TRUE PRIMAL: Zero data duplication!
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails (scenario loading, provider discovery, or renderer setup).
    pub fn new_with_shared_graph(
        _cc: &eframe::CreationContext<'_>,
        scenario_path: Option<std::path::PathBuf>,
        rendering_caps: petal_tongue_core::RenderingCapabilities,
        shared_graph: Arc<RwLock<GraphEngine>>,
    ) -> Result<Self> {
        init::create_app(scenario_path, rendering_caps, shared_graph)
    }

    /// Create a new application (compatibility wrapper for standalone binary)
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails (scenario loading, provider discovery, or renderer setup).
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        scenario_path: Option<std::path::PathBuf>,
        rendering_caps: petal_tongue_core::RenderingCapabilities,
    ) -> Result<Self> {
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

    #[doc(hidden)]
    pub const fn tools_mut(&mut self) -> &mut crate::tool_integration::ToolManager {
        &mut self.tools
    }

    /// Inject the IPC sensor stream registry so egui events are broadcast to subscribers.
    pub fn set_sensor_stream(&mut self, reg: Arc<RwLock<petal_tongue_ipc::SensorStreamRegistry>>) {
        self.sensor_stream = Some(reg);
    }

    /// Inject the IPC interaction subscriber registry so UI selection changes are broadcast.
    pub fn set_interaction_subscribers(
        &mut self,
        reg: Arc<RwLock<petal_tongue_ipc::InteractionSubscriberRegistry>>,
    ) {
        self.interaction_subscribers = Some(reg);
    }

    /// Inject the push-delivery sender so GUI-originated interaction events
    /// reach subscribers with `callback_socket` (PT-06).
    pub fn set_callback_tx(
        &mut self,
        tx: tokio::sync::mpsc::UnboundedSender<petal_tongue_ipc::CallbackDispatch>,
    ) {
        self.callback_tx = Some(tx);
    }

    /// Create an application in headless mode (no display, no eframe context).
    ///
    /// Suitable for testing, introspection harnesses, and CI pipelines.
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails (scenario loading, provider discovery, or renderer setup).
    pub fn new_headless() -> Result<Self> {
        let shared_graph = Arc::new(RwLock::new(GraphEngine::new()));
        let caps = petal_tongue_core::RenderingCapabilities::detect();
        init::create_app(None, caps, shared_graph)
    }

    /// Snapshot of what this frame contains: perceivable panels, bound data, possible interactions.
    ///
    /// This closes the proprioceptive loop: the primal knows *what* it's presenting.
    #[must_use]
    pub fn introspect(&self) -> FrameIntrospection {
        let mut panels = Vec::new();

        panels.push(if self.show_top_menu {
            PanelSnapshot::visible(PanelId::TopMenu, PanelKind::TopMenu)
        } else {
            PanelSnapshot::hidden(PanelId::TopMenu, PanelKind::TopMenu)
        });

        panels.push(if self.show_controls {
            PanelSnapshot::visible(PanelId::LeftSidebar, PanelKind::Controls)
        } else {
            PanelSnapshot::hidden(PanelId::LeftSidebar, PanelKind::Controls)
        });

        panels.push(if self.show_audio_panel {
            PanelSnapshot::visible(PanelId::AudioPanel, PanelKind::AudioSonification)
        } else {
            PanelSnapshot::hidden(PanelId::AudioPanel, PanelKind::AudioSonification)
        });

        panels.push(if self.show_capability_panel {
            PanelSnapshot::visible(
                PanelId::Custom("capability".into()),
                PanelKind::CapabilityStatus,
            )
        } else {
            PanelSnapshot::hidden(
                PanelId::Custom("capability".into()),
                PanelKind::CapabilityStatus,
            )
        });

        panels.push(if self.show_dashboard {
            PanelSnapshot::visible(PanelId::SystemDashboard, PanelKind::Dashboard)
                .with_data_source("proc_stats")
        } else {
            PanelSnapshot::hidden(PanelId::SystemDashboard, PanelKind::Dashboard)
        });

        panels.push(if self.show_trust_dashboard {
            PanelSnapshot::visible(PanelId::TrustDashboard, PanelKind::TrustDashboard)
        } else {
            PanelSnapshot::hidden(PanelId::TrustDashboard, PanelKind::TrustDashboard)
        });

        panels.push(if self.show_neural_proprioception {
            PanelSnapshot::visible(PanelId::Proprioception, PanelKind::Proprioception)
        } else {
            PanelSnapshot::hidden(PanelId::Proprioception, PanelKind::Proprioception)
        });

        panels.push(if self.show_neural_metrics {
            PanelSnapshot::visible(PanelId::Custom("metrics".into()), PanelKind::Metrics)
        } else {
            PanelSnapshot::hidden(PanelId::Custom("metrics".into()), PanelKind::Metrics)
        });

        panels.push(if self.show_graph_builder {
            PanelSnapshot::visible(
                PanelId::Custom("graph_builder".into()),
                PanelKind::GraphBuilder,
            )
        } else {
            PanelSnapshot::hidden(
                PanelId::Custom("graph_builder".into()),
                PanelKind::GraphBuilder,
            )
        });

        // Central panel (always visible)
        panels.push(PanelSnapshot::visible(
            PanelId::Custom("graph_canvas".into()),
            PanelKind::GraphCanvas,
        ));

        panels.push(if self.accessibility_panel.show {
            PanelSnapshot::visible(
                PanelId::Custom("accessibility".into()),
                PanelKind::Accessibility,
            )
        } else {
            PanelSnapshot::hidden(
                PanelId::Custom("accessibility".into()),
                PanelKind::Accessibility,
            )
        });

        // Awakening overlay
        if self.awakening_overlay.is_active() {
            panels.push(PanelSnapshot::visible(
                PanelId::Custom("awakening".into()),
                PanelKind::Awakening,
            ));
        }

        // Collect bound data from the graph
        let bound_data = self.collect_bound_data();

        // Collect possible interactions
        let possible_interactions = self.collect_possible_interactions();

        FrameIntrospection {
            frame_id: self.frame_count,
            timestamp: std::time::Instant::now(),
            visible_panels: panels,
            bound_data,
            possible_interactions,
            active_modalities: vec![
                petal_tongue_core::interaction::perspective::OutputModality::Gui,
            ],
        }
    }

    /// Collect data objects currently bound to the graph canvas.
    fn collect_bound_data(&self) -> Vec<petal_tongue_core::BoundDataObject> {
        let mut bindings = Vec::new();
        if let Ok(graph) = self.graph.read() {
            for node in graph.nodes() {
                bindings.push(petal_tongue_core::BoundDataObject {
                    panel_id: PanelId::Custom("graph_canvas".into()),
                    data_object_id: node.info.id.to_string(),
                    binding_type: petal_tongue_core::BindingType::GraphNode,
                });
            }
            for edge in graph.edges() {
                bindings.push(petal_tongue_core::BoundDataObject {
                    panel_id: PanelId::Custom("graph_canvas".into()),
                    data_object_id: format!("{}->{}", edge.from, edge.to),
                    binding_type: petal_tongue_core::BindingType::GraphEdge,
                });
            }
        }
        bindings
    }

    /// Collect interactions currently available given UI state.
    fn collect_possible_interactions(&self) -> Vec<InteractionCapability> {
        let mut interactions = Vec::new();

        interactions.push(InteractionCapability {
            panel_id: PanelId::Custom("graph_canvas".into()),
            intent: InteractionKind::Navigate,
            target: None,
        });

        if let Some(selected) = self.visual_renderer.selected_node() {
            interactions.push(InteractionCapability {
                panel_id: PanelId::Custom("graph_canvas".into()),
                intent: InteractionKind::Inspect,
                target: Some(selected.to_string()),
            });
        }

        interactions.push(InteractionCapability {
            panel_id: PanelId::TopMenu,
            intent: InteractionKind::TogglePanel,
            target: None,
        });

        if self.show_controls {
            interactions.push(InteractionCapability {
                panel_id: PanelId::LeftSidebar,
                intent: InteractionKind::Configure,
                target: None,
            });
        }

        interactions
    }

    /// Run one update cycle without requiring `eframe::Frame`.
    ///
    /// This contains the full panel logic extracted from `eframe::App::update`
    /// so that the same code path can be exercised by both the real interface and
    /// the headless harness.
    pub fn update_headless(&mut self, ctx: &egui::Context) {
        update::run_update(self, ctx);
    }

    /// Record the current frame's content into the rendering awareness system.
    pub(crate) fn feed_introspection(&self) {
        let introspection = self.introspect();
        if let Ok(mut awareness) = self.rendering_awareness.write() {
            awareness.record_frame_content(introspection);
        }
    }

    /// Access the rendering awareness (read-only).
    #[must_use]
    pub const fn rendering_awareness(&self) -> &Arc<RwLock<RenderingAwareness>> {
        &self.rendering_awareness
    }

    /// Access the frame count.
    #[must_use]
    pub const fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Whether the keyboard shortcuts help overlay is perceivable.
    #[must_use]
    pub const fn is_help_visible(&self) -> bool {
        self.keyboard_shortcuts.show_help
    }

    /// Whether continuous 60 Hz mode is active.
    #[must_use]
    pub const fn is_continuous_mode(&self) -> bool {
        self.continuous_mode
    }

    /// Access the tick clock for introspection.
    #[must_use]
    pub const fn tick_clock(&self) -> &TickClock {
        &self.tick_clock
    }

    /// Inject the shared visualization state from the IPC server.
    pub fn set_visualization_state(
        &mut self,
        state: Arc<RwLock<petal_tongue_ipc::VisualizationState>>,
    ) {
        self.visualization_state = Some(state);
    }

    /// Number of active IPC visualization sessions.
    #[must_use]
    pub fn active_session_count(&self) -> usize {
        self.visualization_state
            .as_ref()
            .and_then(|vs| vs.read().ok())
            .map_or(0, |state| state.sessions().len())
    }
}

impl eframe::App for PetalTongueApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_headless(ctx);
    }
}

#[cfg(test)]
mod tests;
