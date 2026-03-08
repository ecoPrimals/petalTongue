// SPDX-License-Identifier: AGPL-3.0-only
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

use crate::biomeos_integration::{Device, DeviceStatus, DeviceType};
use crate::ui_events::{UIEvent, UIEventHandler};
use egui::{Color32, RichText, Ui};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

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
    devices: Vec<Device>,
    /// Selected device ID
    selected: Option<String>,
    /// Current filter
    filter: DeviceFilter,
    /// Search query
    search_query: String,
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
    pub async fn refresh(&mut self, devices: Vec<Device>) {
        debug!("🔄 Refreshing device panel with {} devices", devices.len());
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

        for event in events {
            match event {
                UIEvent::DeviceDiscovered(device) => {
                    info!("📥 Device discovered: {}", device.name);
                    self.devices.push(device);
                }
                UIEvent::DeviceRemoved(device_id) => {
                    info!("📤 Device removed: {}", device_id);
                    self.devices.retain(|d| d.id != device_id);
                    if self.selected.as_ref() == Some(&device_id) {
                        self.selected = None;
                    }
                }
                UIEvent::DeviceStatusChanged(device_id, new_status) => {
                    if let Some(device) = self.devices.iter_mut().find(|d| d.id == device_id) {
                        debug!("🔄 Device {} status changed to {:?}", device_id, new_status);
                        device.status = new_status;
                    }
                }
                UIEvent::DeviceUsageChanged(device_id, new_usage) => {
                    if let Some(device) = self.devices.iter_mut().find(|d| d.id == device_id) {
                        device.resource_usage = new_usage;
                    }
                }
                UIEvent::DeviceAssigned(device_id, primal_id) => {
                    if let Some(device) = self.devices.iter_mut().find(|d| d.id == device_id) {
                        info!("✅ Device {} assigned to {}", device_id, primal_id);
                        device.assigned_to = Some(primal_id);
                    }
                }
                UIEvent::DeviceUnassigned(device_id, _) => {
                    if let Some(device) = self.devices.iter_mut().find(|d| d.id == device_id) {
                        info!("❌ Device {} unassigned", device_id);
                        device.assigned_to = None;
                    }
                }
                _ => {} // Other events not relevant to device panel
            }
        }
    }

    /// Render the device panel
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.heading("🖥️ Devices");
        ui.separator();

        // Filter bar
        self.render_filter_bar(ui);
        ui.add_space(8.0);

        // Search box
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.text_edit_singleline(&mut self.search_query);
        });
        ui.add_space(8.0);

        // Stats
        self.render_stats(ui);
        ui.add_space(8.0);

        // Device list
        egui::ScrollArea::vertical()
            .id_source("device_list")
            .show(ui, |ui| {
                // Clone devices to avoid borrow checker issues with mutable UI rendering
                let filtered_devices: Vec<Device> =
                    self.filtered_devices().iter().map(|&d| d.clone()).collect();

                if filtered_devices.is_empty() {
                    ui.colored_label(Color32::GRAY, "No devices found");
                } else {
                    for device in &filtered_devices {
                        self.render_device_card(ui, device);
                    }
                }
            });
    }

    /// Render filter bar
    fn render_filter_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Filter:");

            if ui
                .selectable_label(self.filter == DeviceFilter::All, "All")
                .clicked()
            {
                self.filter = DeviceFilter::All;
            }

            if ui
                .selectable_label(self.filter == DeviceFilter::Available, "Available")
                .clicked()
            {
                self.filter = DeviceFilter::Available;
            }

            if ui
                .selectable_label(self.filter == DeviceFilter::Assigned, "Assigned")
                .clicked()
            {
                self.filter = DeviceFilter::Assigned;
            }
        });
    }

    /// Render stats
    fn render_stats(&self, ui: &mut Ui) {
        let total = self.devices.len();
        let online = self
            .devices
            .iter()
            .filter(|d| d.status == DeviceStatus::Online)
            .count();
        let assigned = self
            .devices
            .iter()
            .filter(|d| d.assigned_to.is_some())
            .count();

        ui.horizontal(|ui| {
            ui.label(format!("Total: {total}"));
            ui.separator();
            ui.colored_label(Color32::GREEN, format!("Online: {online}"));
            ui.separator();
            ui.label(format!("Assigned: {assigned}"));
        });
    }

    /// Render individual device card
    fn render_device_card(&mut self, ui: &mut Ui, device: &Device) {
        let is_selected = self.selected.as_ref() == Some(&device.id);

        let response = egui::Frame::none()
            .fill(if is_selected {
                ui.visuals().selection.bg_fill
            } else {
                ui.visuals().faint_bg_color
            })
            .inner_margin(egui::Margin::same(8.0))
            .rounding(4.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Device icon
                    ui.label(RichText::new(device_icon(device.device_type)).size(20.0));

                    // Device info
                    ui.vertical(|ui| {
                        ui.label(RichText::new(&device.name).strong());

                        ui.horizontal(|ui| {
                            // Status indicator
                            let (color, text) = match device.status {
                                DeviceStatus::Online => (Color32::GREEN, "● Online"),
                                DeviceStatus::Offline => (Color32::GRAY, "● Offline"),
                                DeviceStatus::Busy => (Color32::YELLOW, "● Busy"),
                                DeviceStatus::Error => (Color32::RED, "● Error"),
                            };
                            ui.colored_label(color, text);

                            // Assignment status
                            if let Some(primal_id) = &device.assigned_to {
                                ui.separator();
                                ui.label(format!("→ {primal_id}"));
                            }
                        });
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Resource usage bar
                        let usage = device.resource_usage;
                        let bar_color = if usage > 0.9 {
                            Color32::RED
                        } else if usage > 0.7 {
                            Color32::YELLOW
                        } else {
                            Color32::GREEN
                        };

                        ui.add(
                            egui::ProgressBar::new(usage as f32)
                                .fill(bar_color)
                                .show_percentage(),
                        );
                    });
                });
            })
            .response;

        // Selection
        if response.clicked() {
            self.selected = if is_selected {
                None
            } else {
                Some(device.id.clone())
            };
        }

        // Start drag if mouse is dragging
        let is_dragging = response.dragged();
        if is_dragging {
            // Store device ID in drag payload using egui's memory
            ui.memory_mut(|mem| {
                mem.data
                    .insert_temp(egui::Id::new("dragged_device"), device.id.clone());
            });
        }

        // Drag source (for device assignment) - show hover text
        if response.hovered() {
            response.on_hover_text("Drag to assign to a primal");
        }

        ui.add_space(4.0);
    }

    /// Get filtered devices based on current filter and search
    fn filtered_devices(&self) -> Vec<&Device> {
        self.devices
            .iter()
            .filter(|device| {
                // Apply filter
                let filter_match = match self.filter {
                    DeviceFilter::All => true,
                    DeviceFilter::Available => device.assigned_to.is_none(),
                    DeviceFilter::Assigned => device.assigned_to.is_some(),
                };

                // Apply search
                let search_match = if self.search_query.is_empty() {
                    true
                } else {
                    let query = self.search_query.to_lowercase();
                    device.name.to_lowercase().contains(&query)
                        || device.id.to_lowercase().contains(&query)
                };

                filter_match && search_match
            })
            .collect()
    }

    /// Get selected device
    #[must_use]
    pub fn selected_device(&self) -> Option<&Device> {
        self.selected
            .as_ref()
            .and_then(|id| self.devices.iter().find(|d| &d.id == id))
    }
}

/// Get icon for device type
fn device_icon(device_type: DeviceType) -> &'static str {
    match device_type {
        DeviceType::GPU => "🎮",
        DeviceType::CPU => "🧠",
        DeviceType::Storage => "💾",
        DeviceType::Network => "🌐",
        DeviceType::Memory => "🔲",
        DeviceType::Other => "❓",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biomeos_integration::DeviceType;

    #[tokio::test]
    async fn test_device_panel_creation() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let panel = DevicePanel::new(event_handler);

        assert_eq!(panel.devices.len(), 0);
        assert_eq!(panel.filter, DeviceFilter::All);
        assert!(panel.search_query.is_empty());
    }

    #[tokio::test]
    async fn test_device_panel_refresh() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut panel = DevicePanel::new(event_handler);

        let devices = vec![
            Device {
                id: "gpu-0".to_string(),
                name: "Test GPU".to_string(),
                device_type: DeviceType::GPU,
                status: DeviceStatus::Online,
                resource_usage: 0.5,
                assigned_to: None,
                metadata: serde_json::json!({}),
            },
            Device {
                id: "cpu-0".to_string(),
                name: "Test CPU".to_string(),
                device_type: DeviceType::CPU,
                status: DeviceStatus::Online,
                resource_usage: 0.3,
                assigned_to: None,
                metadata: serde_json::json!({}),
            },
        ];

        panel.refresh(devices).await;

        assert_eq!(panel.devices.len(), 2);
    }

    #[tokio::test]
    async fn test_device_panel_event_processing() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut panel = DevicePanel::new(event_handler.clone());

        // Add initial device
        panel
            .refresh(vec![Device {
                id: "gpu-0".to_string(),
                name: "Test GPU".to_string(),
                device_type: DeviceType::GPU,
                status: DeviceStatus::Online,
                resource_usage: 0.5,
                assigned_to: None,
                metadata: serde_json::json!({}),
            }])
            .await;

        // Send status change event
        event_handler
            .write()
            .await
            .handle_event(UIEvent::DeviceStatusChanged(
                "gpu-0".to_string(),
                DeviceStatus::Busy,
            ))
            .await;

        // Process events
        panel.process_events().await;

        // Check status was updated
        assert_eq!(panel.devices[0].status, DeviceStatus::Busy);
    }

    #[tokio::test]
    async fn test_device_panel_filtering() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut panel = DevicePanel::new(event_handler);

        let devices = vec![
            Device {
                id: "gpu-0".to_string(),
                name: "Test GPU".to_string(),
                device_type: DeviceType::GPU,
                status: DeviceStatus::Online,
                resource_usage: 0.5,
                assigned_to: Some("primal-1".to_string()),
                metadata: serde_json::json!({}),
            },
            Device {
                id: "cpu-0".to_string(),
                name: "Test CPU".to_string(),
                device_type: DeviceType::CPU,
                status: DeviceStatus::Online,
                resource_usage: 0.3,
                assigned_to: None,
                metadata: serde_json::json!({}),
            },
        ];

        panel.refresh(devices).await;

        // Test All filter
        panel.filter = DeviceFilter::All;
        assert_eq!(panel.filtered_devices().len(), 2);

        // Test Available filter
        panel.filter = DeviceFilter::Available;
        assert_eq!(panel.filtered_devices().len(), 1);
        assert_eq!(panel.filtered_devices()[0].id, "cpu-0");

        // Test Assigned filter
        panel.filter = DeviceFilter::Assigned;
        assert_eq!(panel.filtered_devices().len(), 1);
        assert_eq!(panel.filtered_devices()[0].id, "gpu-0");
    }

    #[tokio::test]
    async fn test_device_panel_search() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut panel = DevicePanel::new(event_handler);

        let devices = vec![
            Device {
                id: "gpu-0".to_string(),
                name: "NVIDIA GPU".to_string(),
                device_type: DeviceType::GPU,
                status: DeviceStatus::Online,
                resource_usage: 0.5,
                assigned_to: None,
                metadata: serde_json::json!({}),
            },
            Device {
                id: "cpu-0".to_string(),
                name: "AMD CPU".to_string(),
                device_type: DeviceType::CPU,
                status: DeviceStatus::Online,
                resource_usage: 0.3,
                assigned_to: None,
                metadata: serde_json::json!({}),
            },
        ];

        panel.refresh(devices).await;

        // Search for "NVIDIA"
        panel.search_query = "nvidia".to_string();
        assert_eq!(panel.filtered_devices().len(), 1);
        assert_eq!(panel.filtered_devices()[0].name, "NVIDIA GPU");

        // Search for "cpu"
        panel.search_query = "cpu".to_string();
        assert_eq!(panel.filtered_devices().len(), 1);
        assert_eq!(panel.filtered_devices()[0].name, "AMD CPU");

        // Empty search
        panel.search_query = String::new();
        assert_eq!(panel.filtered_devices().len(), 2);
    }

    #[test]
    fn test_device_icon() {
        assert_eq!(device_icon(DeviceType::GPU), "🎮");
        assert_eq!(device_icon(DeviceType::CPU), "🧠");
        assert_eq!(device_icon(DeviceType::Storage), "💾");
        assert_eq!(device_icon(DeviceType::Network), "🌐");
        assert_eq!(device_icon(DeviceType::Memory), "🔲");
        assert_eq!(device_icon(DeviceType::Other), "❓");
    }
}
