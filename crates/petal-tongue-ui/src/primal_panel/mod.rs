// SPDX-License-Identifier: AGPL-3.0-or-later
//! Primal Panel - Primal Status Management UI
//!
//! Displays all discovered primals with their health, capabilities, load, and device assignments.
//! Provides drag-and-drop drop zones for device assignment.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │ PrimalPanel                                                 │
//! │  ├─ Filter Bar (All/Healthy/Degraded/Error)                 │
//! │  ├─ Search Box                                              │
//! │  └─ Primal List                                             │
//! │      ├─ PrimalCard (security-primal) [drop zone]            │
//! │      │   ├─ Health: Healthy                                 │
//! │      │   ├─ Load: 45%                                       │
//! │      │   ├─ Capabilities: 5                                 │
//! │      │   └─ Devices: GPU-0, CPU-1                           │
//! │      ├─ PrimalCard (discovery-primal) [drop zone]           │
//! │      └─ PrimalCard (compute-primal) [drop zone]             │
//! └─────────────────────────────────────────────────────────────┘
//! ```

mod display;
mod filter;
mod render;
mod state;
mod stats;

use crate::biomeos_integration::Primal;
use crate::ui_events::UIEventHandler;
use egui::Ui;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

// Re-export public API
pub use display::{health_display_data, load_bar_color, load_bar_color_rgb};
pub use filter::PrimalFilter;
pub use stats::compute_primal_stats;

/// Primal panel - main UI component for primal status management
pub struct PrimalPanel {
    /// All primals (updated from provider)
    pub(in crate::primal_panel) primals: Vec<Primal>,
    /// Selected primal ID
    pub(in crate::primal_panel) selected: Option<String>,
    /// Current filter
    pub(in crate::primal_panel) filter: PrimalFilter,
    /// Search query
    pub(in crate::primal_panel) search_query: String,
    /// Event handler for real-time updates
    pub(in crate::primal_panel) event_handler: Arc<RwLock<UIEventHandler>>,
    /// Last refresh time
    last_refresh: std::time::Instant,
}

impl PrimalPanel {
    /// Create a new primal panel
    #[must_use]
    pub fn new(event_handler: Arc<RwLock<UIEventHandler>>) -> Self {
        info!("🌸 Creating primal panel");

        Self {
            primals: Vec::new(),
            selected: None,
            filter: PrimalFilter::All,
            search_query: String::new(),
            event_handler,
            last_refresh: std::time::Instant::now(),
        }
    }

    /// Update primals from provider
    #[expect(clippy::unused_async, reason = "async for trait compatibility")]
    pub async fn refresh(&mut self, primals: Vec<Primal>) {
        debug!("🔄 Refreshing primal panel with {} primals", primals.len());
        self.primals = primals;
        self.last_refresh = std::time::Instant::now();
    }

    /// Process pending events
    pub async fn process_events(&mut self) {
        let events = self
            .event_handler
            .write()
            .await
            .consume_primal_panel_events()
            .await;

        state::apply_primal_events(&mut self.primals, &mut self.selected, events);
    }

    /// Render the primal panel
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.heading("🌸 Primals");
        ui.separator();

        // Filter bar
        render::render_filter_bar(self, ui);
        ui.add_space(8.0);

        // Search box
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.text_edit_singleline(&mut self.search_query);
        });
        ui.add_space(8.0);

        // Stats
        render::render_stats(self, ui);
        ui.add_space(8.0);

        // Primal list
        egui::ScrollArea::vertical()
            .id_salt("primal_list")
            .show(ui, |ui| {
                let filtered_primals: Vec<Primal> =
                    self.filtered_primals().into_iter().cloned().collect();

                if filtered_primals.is_empty() {
                    ui.colored_label(egui::Color32::GRAY, "No primals found");
                } else {
                    for primal in &filtered_primals {
                        render::render_primal_card(self, ui, primal, self.event_handler.clone());
                    }
                }
            });
    }

    /// Wrapper for backward compatibility - delegates to stats module
    #[must_use]
    pub fn compute_primal_stats(primals: &[Primal]) -> (usize, usize, usize, usize) {
        stats::compute_primal_stats(primals)
    }

    /// Wrapper for backward compatibility - delegates to display module
    #[must_use]
    pub fn load_bar_color(load: f64) -> egui::Color32 {
        display::load_bar_color(load)
    }

    fn filtered_primals(&self) -> Vec<&Primal> {
        state::filter_primals(&self.primals, self.filter, &self.search_query)
    }

    /// Get selected primal
    #[must_use]
    pub fn selected_primal(&self) -> Option<&Primal> {
        self.selected
            .as_ref()
            .and_then(|id| self.primals.iter().find(|p| &p.id == id))
    }
}

#[cfg(test)]
mod tests;
