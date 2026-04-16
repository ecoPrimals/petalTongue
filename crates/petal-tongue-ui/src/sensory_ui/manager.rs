// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sensory UI manager and renderer trait.

use eframe::egui;
use petal_tongue_core::sensory_capabilities::UIComplexity as SensoryUIComplexity;
use petal_tongue_core::{CapabilityError, SensoryCapabilities};
use std::time::Instant;

use crate::sensory_ui::renderers::{
    ImmersiveSensoryUI, MinimalSensoryUI, RichSensoryUI, SimpleSensoryUI, StandardSensoryUI,
};

/// Enum dispatch for sensory renderers (replaces `Box<dyn SensoryUIRenderer>`).
pub enum SensoryUIRendererImpl {
    /// Minimal UI.
    Minimal(MinimalSensoryUI),
    /// Simple UI.
    Simple(SimpleSensoryUI),
    /// Standard UI.
    Standard(StandardSensoryUI),
    /// Rich UI.
    Rich(RichSensoryUI),
    /// Immersive UI.
    Immersive(ImmersiveSensoryUI),
}

impl SensoryUIRenderer for SensoryUIRendererImpl {
    fn render_primal_list(&mut self, ui: &mut egui::Ui, primals: &[petal_tongue_core::PrimalInfo]) {
        match self {
            Self::Minimal(r) => r.render_primal_list(ui, primals),
            Self::Simple(r) => r.render_primal_list(ui, primals),
            Self::Standard(r) => r.render_primal_list(ui, primals),
            Self::Rich(r) => r.render_primal_list(ui, primals),
            Self::Immersive(r) => r.render_primal_list(ui, primals),
        }
    }

    fn render_topology(
        &mut self,
        ui: &mut egui::Ui,
        graph_engine: &petal_tongue_core::GraphEngine,
    ) {
        match self {
            Self::Minimal(r) => r.render_topology(ui, graph_engine),
            Self::Simple(r) => r.render_topology(ui, graph_engine),
            Self::Standard(r) => r.render_topology(ui, graph_engine),
            Self::Rich(r) => r.render_topology(ui, graph_engine),
            Self::Immersive(r) => r.render_topology(ui, graph_engine),
        }
    }

    fn render_metrics(
        &mut self,
        ui: &mut egui::Ui,
        metrics: Option<&petal_tongue_core::SystemMetrics>,
    ) {
        match self {
            Self::Minimal(r) => r.render_metrics(ui, metrics),
            Self::Simple(r) => r.render_metrics(ui, metrics),
            Self::Standard(r) => r.render_metrics(ui, metrics),
            Self::Rich(r) => r.render_metrics(ui, metrics),
            Self::Immersive(r) => r.render_metrics(ui, metrics),
        }
    }

    fn render_proprioception(
        &mut self,
        ui: &mut egui::Ui,
        proprioception: Option<&petal_tongue_core::ProprioceptionData>,
    ) {
        match self {
            Self::Minimal(r) => r.render_proprioception(ui, proprioception),
            Self::Simple(r) => r.render_proprioception(ui, proprioception),
            Self::Standard(r) => r.render_proprioception(ui, proprioception),
            Self::Rich(r) => r.render_proprioception(ui, proprioception),
            Self::Immersive(r) => r.render_proprioception(ui, proprioception),
        }
    }
}

/// Sensory-based adaptive UI manager
///
/// This replaces the old `AdaptiveUIManager` which used device types.
/// Now we use discovered capabilities instead.
pub struct SensoryUIManager {
    capabilities: SensoryCapabilities,
    ui_complexity: SensoryUIComplexity,
    renderer: SensoryUIRendererImpl,
    last_discovery: Instant,
}

impl SensoryUIManager {
    /// Create a new sensory UI manager with discovered capabilities
    ///
    /// # Errors
    ///
    /// Returns an error if capability discovery fails.
    pub fn new() -> Result<Self, CapabilityError> {
        let capabilities = SensoryCapabilities::discover()?;
        let ui_complexity = capabilities.determine_ui_complexity();

        let renderer = Self::create_renderer(ui_complexity);

        Ok(Self {
            capabilities,
            ui_complexity,
            renderer,
            last_discovery: Instant::now(),
        })
    }

    /// Create appropriate renderer for UI complexity level
    const fn create_renderer(complexity: SensoryUIComplexity) -> SensoryUIRendererImpl {
        match complexity {
            SensoryUIComplexity::Minimal => SensoryUIRendererImpl::Minimal(MinimalSensoryUI::new()),
            SensoryUIComplexity::Simple => SensoryUIRendererImpl::Simple(SimpleSensoryUI::new()),
            SensoryUIComplexity::Standard => {
                SensoryUIRendererImpl::Standard(StandardSensoryUI::new())
            }
            SensoryUIComplexity::Rich => SensoryUIRendererImpl::Rich(RichSensoryUI::new()),
            SensoryUIComplexity::Immersive => {
                SensoryUIRendererImpl::Immersive(ImmersiveSensoryUI::new())
            }
        }
    }

    /// Get current UI complexity
    #[must_use]
    pub const fn ui_complexity(&self) -> SensoryUIComplexity {
        self.ui_complexity
    }

    /// Get capabilities description
    #[must_use]
    pub fn capabilities_description(&self) -> String {
        self.capabilities.describe()
    }

    /// Re-discover capabilities (for hot-reload when hardware changes)
    ///
    /// # Errors
    ///
    /// Returns an error if capability discovery fails.
    pub fn rediscover(&mut self) -> Result<(), CapabilityError> {
        // Only rediscover every 5 seconds to avoid overhead
        if self.last_discovery.elapsed().as_secs() < 5 {
            return Ok(());
        }

        let new_capabilities = SensoryCapabilities::discover()?;
        let new_complexity = new_capabilities.determine_ui_complexity();

        // Hot-swap renderer if complexity changed
        if new_complexity != self.ui_complexity {
            tracing::info!(
                "Capability change detected: {} → {}",
                self.ui_complexity,
                new_complexity
            );

            self.renderer = Self::create_renderer(new_complexity);
            self.ui_complexity = new_complexity;
        }

        self.capabilities = new_capabilities;
        self.last_discovery = Instant::now();

        Ok(())
    }

    /// Create manager with given capabilities (for testing - bypasses discovery)
    #[cfg(test)]
    #[must_use]
    pub fn with_capabilities(capabilities: SensoryCapabilities) -> Self {
        let ui_complexity = capabilities.determine_ui_complexity();
        let renderer = Self::create_renderer(ui_complexity);
        Self {
            capabilities,
            ui_complexity,
            renderer,
            last_discovery: Instant::now(),
        }
    }

    /// Render the primal list
    pub fn render_primal_list(
        &mut self,
        ui: &mut egui::Ui,
        primals: &[petal_tongue_core::PrimalInfo],
    ) {
        self.renderer.render_primal_list(ui, primals);
    }

    /// Render the topology view
    pub fn render_topology(
        &mut self,
        ui: &mut egui::Ui,
        graph_engine: &petal_tongue_core::GraphEngine,
    ) {
        self.renderer.render_topology(ui, graph_engine);
    }

    /// Render the metrics panel
    pub fn render_metrics(
        &mut self,
        ui: &mut egui::Ui,
        metrics: Option<&petal_tongue_core::SystemMetrics>,
    ) {
        self.renderer.render_metrics(ui, metrics);
    }

    /// Render the proprioception panel
    pub fn render_proprioception(
        &mut self,
        ui: &mut egui::Ui,
        proprioception: Option<&petal_tongue_core::ProprioceptionData>,
    ) {
        self.renderer.render_proprioception(ui, proprioception);
    }
}

/// Trait for sensory-based UI renderers
///
/// Each complexity level has a different renderer implementation
/// that adapts to the available capabilities.
pub trait SensoryUIRenderer: Send {
    /// Render the primal list
    fn render_primal_list(&mut self, ui: &mut egui::Ui, primals: &[petal_tongue_core::PrimalInfo]);

    /// Render the topology view
    fn render_topology(&mut self, ui: &mut egui::Ui, graph_engine: &petal_tongue_core::GraphEngine);

    /// Render the metrics panel
    fn render_metrics(
        &mut self,
        ui: &mut egui::Ui,
        metrics: Option<&petal_tongue_core::SystemMetrics>,
    );

    /// Render the proprioception panel
    fn render_proprioception(
        &mut self,
        ui: &mut egui::Ui,
        proprioception: Option<&petal_tongue_core::ProprioceptionData>,
    );
}
