// SPDX-License-Identifier: AGPL-3.0-or-later
//! Device Panel - Device Management UI
//!
//! Displays all discovered devices in a tree view with filtering, status indicators,
//! and drag-and-drop support for device assignment.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │ DevicePanel                                                 │
//! │  ├─ Filter Bar (All/Available/Assigned)                     │
//! │  ├─ Search Box                                              │
//! │  └─ Device List                                             │
//! │      ├─ DeviceCard (GPU-0) [draggable]                      │
//! │      ├─ DeviceCard (CPU-1) [draggable]                      │
//! │      └─ DeviceCard (SSD-2) [draggable]                      │
//! └─────────────────────────────────────────────────────────────┘
//! ```

mod detail_view;
mod events;
mod list_view;

#[cfg(test)]
mod test_fixtures;
#[cfg(test)]
mod tests_builtins;
#[cfg(test)]
mod tests_events;

pub use detail_view::device_icon;

use crate::biomeos_integration::{Device, DeviceStatus};
use crate::ui_events::UIEventHandler;
use egui::{Color32, Ui};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Device filter options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceFilter {
    /// Show all devices
    All,
    /// Show only available (unassigned) devices
    Available,
    /// Show only assigned devices
    Assigned,
}

/// Device panel - main UI component for device management
pub struct DevicePanel {
    /// All devices (updated from provider)
    pub(super) devices: Vec<Device>,
    /// Selected device ID
    pub(super) selected: Option<String>,
    /// Current filter
    pub(super) filter: DeviceFilter,
    /// Search query
    pub(super) search_query: String,
    /// Event handler for real-time updates
    event_handler: Arc<RwLock<UIEventHandler>>,
    /// Last refresh time
    last_refresh: std::time::Instant,
}

impl DevicePanel {
    /// Create a new device panel
    #[must_use]
    pub fn new(event_handler: Arc<RwLock<UIEventHandler>>) -> Self {
        info!("🖥️ Creating device panel");

        Self {
            devices: Vec::new(),
            selected: None,
            filter: DeviceFilter::All,
            search_query: String::new(),
            event_handler,
            last_refresh: std::time::Instant::now(),
        }
    }

    /// Update devices from provider
    #[expect(clippy::unused_async, reason = "async for trait compatibility")]
    pub async fn refresh(&mut self, devices: Vec<Device>) {
        tracing::debug!("🔄 Refreshing device panel with {} devices", devices.len());
        self.devices = devices;
        self.last_refresh = std::time::Instant::now();
    }

    /// Process pending events
    pub async fn process_events(&mut self) {
        let events = self
            .event_handler
            .write()
            .await
            .consume_device_panel_events()
            .await;

        events::apply_ui_events(self, events);
    }

    /// Render the device panel
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.heading("🖥️ Devices");
        ui.separator();

        list_view::render_filter_bar(self, ui);
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.text_edit_singleline(&mut self.search_query);
        });
        ui.add_space(8.0);

        list_view::render_stats(self, ui);
        ui.add_space(8.0);

        list_view::render_scrollable_list(self, ui);
    }

    #[must_use]
    pub fn compute_device_stats(devices: &[Device]) -> (usize, usize, usize) {
        let total = devices.len();
        let online = devices
            .iter()
            .filter(|d| d.status == DeviceStatus::Online)
            .count();
        let assigned = devices.iter().filter(|d| d.assigned_to.is_some()).count();
        (total, online, assigned)
    }

    #[must_use]
    pub fn usage_bar_color(usage: f64) -> Color32 {
        if usage > 0.9 {
            Color32::RED
        } else if usage > 0.7 {
            Color32::YELLOW
        } else {
            Color32::GREEN
        }
    }

    /// Get selected device
    #[must_use]
    pub fn selected_device(&self) -> Option<&Device> {
        self.selected
            .as_ref()
            .and_then(|id| self.devices.iter().find(|d| &d.id == id))
    }
}
