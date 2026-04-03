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
mod list_view;

pub use detail_view::device_icon;

use crate::biomeos_integration::{Device, DeviceStatus};
use crate::ui_events::{UIEvent, UIEventHandler};
use egui::{Color32, Ui};
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::biomeos_integration::{Device, DeviceStatus, DeviceType};
    use egui::Color32;

    fn make_device(
        id: &str,
        name: &str,
        device_type: DeviceType,
        status: DeviceStatus,
        usage: f64,
    ) -> Device {
        Device {
            id: id.to_string(),
            name: name.to_string(),
            device_type,
            status,
            resource_usage: usage,
            assigned_to: None,
            metadata: serde_json::json!({}),
        }
    }

    fn make_device_assigned(
        id: &str,
        name: &str,
        device_type: DeviceType,
        status: DeviceStatus,
        usage: f64,
        assigned: &str,
    ) -> Device {
        Device {
            assigned_to: Some(assigned.to_string()),
            ..make_device(id, name, device_type, status, usage)
        }
    }

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
            make_device(
                "gpu-0",
                "Test GPU",
                DeviceType::GPU,
                DeviceStatus::Online,
                0.5,
            ),
            make_device(
                "cpu-0",
                "Test CPU",
                DeviceType::CPU,
                DeviceStatus::Online,
                0.3,
            ),
        ];

        panel.refresh(devices).await;

        assert_eq!(panel.devices.len(), 2);
    }

    #[tokio::test]
    async fn test_device_panel_event_processing() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut panel = DevicePanel::new(event_handler.clone());

        panel
            .refresh(vec![make_device(
                "gpu-0",
                "Test GPU",
                DeviceType::GPU,
                DeviceStatus::Online,
                0.5,
            )])
            .await;

        event_handler
            .write()
            .await
            .handle_event(UIEvent::DeviceStatusChanged(
                "gpu-0".to_string(),
                DeviceStatus::Busy,
            ))
            .await;

        panel.process_events().await;

        assert_eq!(panel.devices[0].status, DeviceStatus::Busy);
    }

    #[tokio::test]
    async fn test_device_panel_filtering() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut panel = DevicePanel::new(event_handler);

        let devices = vec![
            make_device_assigned(
                "gpu-0",
                "Test GPU",
                DeviceType::GPU,
                DeviceStatus::Online,
                0.5,
                "primal-1",
            ),
            make_device(
                "cpu-0",
                "Test CPU",
                DeviceType::CPU,
                DeviceStatus::Online,
                0.3,
            ),
        ];

        panel.refresh(devices).await;

        panel.filter = DeviceFilter::All;
        assert_eq!(list_view::filtered_devices(&panel).len(), 2);

        panel.filter = DeviceFilter::Available;
        assert_eq!(list_view::filtered_devices(&panel).len(), 1);
        assert_eq!(list_view::filtered_devices(&panel)[0].id, "cpu-0");

        panel.filter = DeviceFilter::Assigned;
        assert_eq!(list_view::filtered_devices(&panel).len(), 1);
        assert_eq!(list_view::filtered_devices(&panel)[0].id, "gpu-0");
    }

    #[tokio::test]
    async fn test_device_panel_search() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut panel = DevicePanel::new(event_handler);

        let devices = vec![
            make_device(
                "gpu-0",
                "NVIDIA GPU",
                DeviceType::GPU,
                DeviceStatus::Online,
                0.5,
            ),
            make_device(
                "cpu-0",
                "AMD CPU",
                DeviceType::CPU,
                DeviceStatus::Online,
                0.3,
            ),
        ];

        panel.refresh(devices).await;

        panel.search_query = "nvidia".to_string();
        assert_eq!(list_view::filtered_devices(&panel).len(), 1);
        assert_eq!(list_view::filtered_devices(&panel)[0].name, "NVIDIA GPU");

        panel.search_query = "cpu".to_string();
        assert_eq!(list_view::filtered_devices(&panel).len(), 1);
        assert_eq!(list_view::filtered_devices(&panel)[0].name, "AMD CPU");

        panel.search_query = String::new();
        assert_eq!(list_view::filtered_devices(&panel).len(), 2);
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

    #[tokio::test]
    async fn test_device_panel_selected_device() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut panel = DevicePanel::new(event_handler);

        let devices = vec![make_device(
            "dev-1",
            "Device 1",
            DeviceType::GPU,
            DeviceStatus::Online,
            0.5,
        )];
        panel.refresh(devices).await;
        assert!(panel.selected_device().is_none());

        panel.selected = Some("dev-1".to_string());
        let selected = panel.selected_device().expect("selected");
        assert_eq!(selected.id, "dev-1");
    }

    #[tokio::test]
    async fn test_device_panel_device_filter_variants() {
        assert_eq!(DeviceFilter::All, DeviceFilter::All);
        assert_ne!(DeviceFilter::All, DeviceFilter::Available);
        assert_ne!(DeviceFilter::Available, DeviceFilter::Assigned);
    }

    #[tokio::test]
    async fn test_device_panel_device_removed_clears_selection() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut panel = DevicePanel::new(event_handler.clone());

        panel
            .refresh(vec![make_device(
                "gpu-0",
                "GPU",
                DeviceType::GPU,
                DeviceStatus::Online,
                0.5,
            )])
            .await;
        panel.selected = Some("gpu-0".to_string());

        event_handler
            .write()
            .await
            .handle_event(UIEvent::DeviceRemoved("gpu-0".to_string()))
            .await;
        panel.process_events().await;

        assert!(panel.selected.is_none());
        assert!(panel.devices.is_empty());
    }

    #[tokio::test]
    async fn test_device_panel_device_assigned_event() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut panel = DevicePanel::new(event_handler.clone());

        panel
            .refresh(vec![make_device(
                "cpu-0",
                "CPU",
                DeviceType::CPU,
                DeviceStatus::Online,
                0.0,
            )])
            .await;

        event_handler
            .write()
            .await
            .handle_event(UIEvent::DeviceAssigned(
                "cpu-0".to_string(),
                "primal-x".to_string(),
            ))
            .await;
        panel.process_events().await;

        assert_eq!(panel.devices[0].assigned_to, Some("primal-x".to_string()));
    }

    #[tokio::test]
    async fn test_device_panel_device_unassigned_event() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut panel = DevicePanel::new(event_handler.clone());

        panel
            .refresh(vec![make_device_assigned(
                "cpu-0",
                "CPU",
                DeviceType::CPU,
                DeviceStatus::Online,
                0.0,
                "primal-x",
            )])
            .await;

        event_handler
            .write()
            .await
            .handle_event(UIEvent::DeviceUnassigned(
                "cpu-0".to_string(),
                "primal-x".to_string(),
            ))
            .await;
        panel.process_events().await;

        assert!(panel.devices[0].assigned_to.is_none());
    }

    #[tokio::test]
    async fn test_device_panel_device_usage_changed_event() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut panel = DevicePanel::new(event_handler.clone());

        panel
            .refresh(vec![make_device(
                "gpu-0",
                "GPU",
                DeviceType::GPU,
                DeviceStatus::Online,
                0.3,
            )])
            .await;

        event_handler
            .write()
            .await
            .handle_event(UIEvent::DeviceUsageChanged("gpu-0".to_string(), 0.95))
            .await;
        panel.process_events().await;

        assert!((panel.devices[0].resource_usage - 0.95).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_device_panel_search_by_id() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut panel = DevicePanel::new(event_handler);

        panel
            .refresh(vec![make_device(
                "gpu-nvidia-0",
                "Graphics",
                DeviceType::GPU,
                DeviceStatus::Online,
                0.5,
            )])
            .await;

        panel.search_query = "nvidia".to_string();
        assert_eq!(list_view::filtered_devices(&panel).len(), 1);
        assert_eq!(list_view::filtered_devices(&panel)[0].id, "gpu-nvidia-0");
    }

    #[test]
    fn test_compute_device_stats() {
        let devices = vec![
            make_device_assigned("d1", "D1", DeviceType::GPU, DeviceStatus::Online, 0.5, "p1"),
            make_device("d2", "D2", DeviceType::CPU, DeviceStatus::Offline, 0.0),
            make_device("d3", "D3", DeviceType::Storage, DeviceStatus::Online, 0.3),
        ];
        let (total, online, assigned) = DevicePanel::compute_device_stats(&devices);
        assert_eq!(total, 3);
        assert_eq!(online, 2);
        assert_eq!(assigned, 1);
    }

    #[test]
    fn test_compute_device_stats_empty() {
        let (total, online, assigned) = DevicePanel::compute_device_stats(&[]);
        assert_eq!(total, 0);
        assert_eq!(online, 0);
        assert_eq!(assigned, 0);
    }

    #[test]
    fn test_usage_bar_color() {
        assert_eq!(DevicePanel::usage_bar_color(0.0), Color32::GREEN);
        assert_eq!(DevicePanel::usage_bar_color(0.7), Color32::GREEN);
        assert_eq!(DevicePanel::usage_bar_color(0.71), Color32::YELLOW);
        assert_eq!(DevicePanel::usage_bar_color(0.9), Color32::YELLOW);
        assert_eq!(DevicePanel::usage_bar_color(0.91), Color32::RED);
        assert_eq!(DevicePanel::usage_bar_color(1.0), Color32::RED);
    }

    #[tokio::test]
    async fn test_device_panel_ui_headless_empty() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut panel = DevicePanel::new(event_handler);
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                panel.ui(ui);
            });
        });
    }

    #[tokio::test]
    async fn test_device_panel_ui_headless_with_devices() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut panel = DevicePanel::new(event_handler);
        panel
            .refresh(vec![
                make_device("gpu-0", "GPU", DeviceType::GPU, DeviceStatus::Online, 0.5),
                make_device_assigned(
                    "cpu-0",
                    "CPU",
                    DeviceType::CPU,
                    DeviceStatus::Busy,
                    0.9,
                    "primal-1",
                ),
            ])
            .await;

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                panel.ui(ui);
            });
        });
    }

    #[tokio::test]
    async fn test_device_panel_ui_headless_all_statuses() {
        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));
        let mut panel = DevicePanel::new(event_handler);
        panel
            .refresh(vec![
                make_device("d1", "Online", DeviceType::GPU, DeviceStatus::Online, 0.3),
                make_device("d2", "Offline", DeviceType::CPU, DeviceStatus::Offline, 0.0),
                make_device("d3", "Error", DeviceType::Storage, DeviceStatus::Error, 1.0),
            ])
            .await;

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                panel.ui(ui);
            });
        });
    }
}
