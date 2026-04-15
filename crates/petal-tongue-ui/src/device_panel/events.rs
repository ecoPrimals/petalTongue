// SPDX-License-Identifier: AGPL-3.0-or-later

//! Apply [`UIEvent`](crate::ui_events::UIEvent) streams to [`DevicePanel`](super::DevicePanel) state.

use tracing::{debug, info};

use crate::ui_events::UIEvent;

use super::DevicePanel;

pub(super) fn apply_ui_events(panel: &mut DevicePanel, events: Vec<UIEvent>) {
    for event in events {
        match event {
            UIEvent::DeviceDiscovered(device) => {
                info!("📥 Device discovered: {}", device.name);
                panel.devices.push(device);
            }
            UIEvent::DeviceRemoved(device_id) => {
                info!("📤 Device removed: {}", device_id);
                panel.devices.retain(|d| d.id != device_id);
                if panel.selected.as_ref() == Some(&device_id) {
                    panel.selected = None;
                }
            }
            UIEvent::DeviceStatusChanged(device_id, new_status) => {
                if let Some(device) = panel.devices.iter_mut().find(|d| d.id == device_id) {
                    debug!("🔄 Device {} status changed to {:?}", device_id, new_status);
                    device.status = new_status;
                }
            }
            UIEvent::DeviceUsageChanged(device_id, new_usage) => {
                if let Some(device) = panel.devices.iter_mut().find(|d| d.id == device_id) {
                    device.resource_usage = new_usage;
                }
            }
            UIEvent::DeviceAssigned(device_id, primal_id) => {
                if let Some(device) = panel.devices.iter_mut().find(|d| d.id == device_id) {
                    info!("✅ Device {} assigned to {}", device_id, primal_id);
                    device.assigned_to = Some(primal_id);
                }
            }
            UIEvent::DeviceUnassigned(device_id, _) => {
                if let Some(device) = panel.devices.iter_mut().find(|d| d.id == device_id) {
                    info!("❌ Device {} unassigned", device_id);
                    device.assigned_to = None;
                }
            }
            _ => {} // Other events not relevant to device panel
        }
    }
}
