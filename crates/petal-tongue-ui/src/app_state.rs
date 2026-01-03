//! Application state structure for petalTongue UI
//!
//! This module defines the core state of the petalTongue application.
//! Separated from behavior (methods) for clarity and testability.

use crate::accessibility_panel::AccessibilityPanel;
use crate::keyboard_shortcuts::KeyboardShortcuts;
use crate::system_dashboard::SystemDashboard;
use crate::tool_integration::ToolManager;
use petal_tongue_animation::AnimationEngine;
use petal_tongue_api::BiomeOSClient;
use petal_tongue_core::{CapabilityDetector, GraphEngine, LayoutAlgorithm};
use petal_tongue_discovery::VisualizationDataProvider;
use petal_tongue_graph::{AudioFileGenerator, AudioSonificationRenderer, Visual2DRenderer};
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// The main petalTongue UI application state
///
/// This struct contains all the state needed to run petalTongue.
/// Methods are implemented in other modules (`app_init.rs`, `app_render.rs`, etc.)
/// to keep concerns separated and files maintainable.
#[allow(clippy::struct_excessive_bools)]
pub struct PetalTongueApp {
    // ===== Core Capabilities =====
    /// Capability detector (knows what modalities are actually available)
    pub(crate) capabilities: CapabilityDetector,

    // ===== Graph & Visualization =====
    /// The graph engine (shared between renderers)
    pub(crate) graph: Arc<RwLock<GraphEngine>>,
    /// Visual renderer (2D graph display)
    pub(crate) visual_renderer: Visual2DRenderer,
    /// Audio renderer (sonification)
    pub(crate) audio_renderer: AudioSonificationRenderer,
    /// Audio file generator (pure Rust WAV export)
    pub(crate) audio_generator: AudioFileGenerator,
    /// Animation engine (flow particles and pulses)
    pub(crate) animation_engine: Arc<RwLock<AnimationEngine>>,

    // ===== Data Sources (Capability-Based) =====
    /// Visualization data providers (discovered at runtime - capability-based!)
    /// These could be: biomeOS, Songbird, custom aggregators, or any primal
    /// that advertises visualization data capability.
    pub(crate) data_providers: Vec<Box<dyn VisualizationDataProvider>>,
    
    /// Legacy BiomeOS client (DEPRECATED - kept for backward compatibility)
    /// New code should use `data_providers` instead.
    #[deprecated(note = "Use data_providers instead - biomeOS is just another primal!")]
    pub(crate) biomeos_client: BiomeOSClient,

    // ===== Layout & Display Settings =====
    /// Current layout algorithm (ForceDirected, Hierarchical, etc.)
    pub(crate) current_layout: LayoutAlgorithm,
    
    // ===== Panel Visibility =====
    /// Show audio description panel
    pub(crate) show_audio_panel: bool,
    /// Show capability status panel
    pub(crate) show_capability_panel: bool,
    /// Show controls panel (layout, animation, etc.)
    pub(crate) show_controls: bool,
    /// Show animation (flow particles and pulses)
    pub(crate) show_animation: bool,
    /// Show system dashboard sidebar
    pub(crate) show_dashboard: bool,

    // ===== Data Refresh =====
    /// Last time data was refreshed from providers
    pub(crate) last_refresh: Instant,
    /// Auto-refresh enabled
    pub(crate) auto_refresh: bool,
    /// Refresh interval in seconds
    pub(crate) refresh_interval: f32,

    // ===== Tool Integration (Capability-Based) =====
    /// Tool manager (handles all external tools dynamically)
    /// Tools are discovered at runtime based on capabilities.
    pub(crate) tools: ToolManager,

    // ===== Universal UI - Accessibility =====
    /// Accessibility settings panel (color schemes, font sizes, etc.)
    pub(crate) accessibility_panel: AccessibilityPanel,

    // ===== System Monitoring =====
    /// Live system dashboard (CPU, memory, etc.)
    pub(crate) system_dashboard: SystemDashboard,

    // ===== Keyboard Navigation =====
    /// Keyboard shortcuts system
    pub(crate) keyboard_shortcuts: KeyboardShortcuts,
}

impl PetalTongueApp {
    /// Get a reference to the graph engine
    #[must_use]
    pub fn graph(&self) -> &Arc<RwLock<GraphEngine>> {
        &self.graph
    }

    /// Get a mutable reference to the visual renderer
    pub fn visual_renderer_mut(&mut self) -> &mut Visual2DRenderer {
        &mut self.visual_renderer
    }

    /// Check if auto-refresh is enabled
    #[must_use]
    pub fn is_auto_refresh_enabled(&self) -> bool {
        self.auto_refresh
    }

    /// Get the current refresh interval in seconds
    #[must_use]
    pub fn refresh_interval(&self) -> f32 {
        self.refresh_interval
    }

    /// Set the refresh interval
    pub fn set_refresh_interval(&mut self, interval: f32) {
        self.refresh_interval = interval.max(1.0); // Min 1 second
    }

    /// Get capability detector reference
    #[must_use]
    pub fn capabilities(&self) -> &CapabilityDetector {
        &self.capabilities
    }
}

