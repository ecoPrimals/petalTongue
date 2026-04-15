// SPDX-License-Identifier: AGPL-3.0-or-later

//! Shared device fixtures for device panel unit tests.

use crate::biomeos_integration::{Device, DeviceStatus, DeviceType};

pub(super) fn make_device(
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

pub(super) fn make_device_assigned(
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
