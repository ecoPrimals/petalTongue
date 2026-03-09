// SPDX-License-Identifier: AGPL-3.0-only
//! Core data types for biomeOS integration.
//!
//! Device, Primal, and NicheTemplate types used for device management UI.

use serde::{Deserialize, Serialize};

/// Device representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    /// Device identifier
    pub id: String,
    /// Human-readable device name
    pub name: String,
    /// Type of device
    pub device_type: DeviceType,
    /// Current device status
    pub status: DeviceStatus,
    /// Resource usage (0.0-1.0)
    pub resource_usage: f64,
    /// Primal ID if device is assigned
    pub assigned_to: Option<String>,
    /// Additional device metadata
    pub metadata: serde_json::Value,
}

/// Device type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceType {
    /// Graphics processing unit
    GPU,
    /// Central processing unit
    CPU,
    /// Storage device
    Storage,
    /// Network interface
    Network,
    /// Memory module
    Memory,
    /// Other device type
    Other,
}

/// Device status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceStatus {
    /// Device is online and available
    Online,
    /// Device is offline
    Offline,
    /// Device is busy with current task
    Busy,
    /// Device has an error
    Error,
}

/// Primal representation (extended from `PrimalInfo`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Primal {
    /// Primal identifier
    pub id: String,
    /// Human-readable primal name
    pub name: String,
    /// Capabilities provided by primal
    pub capabilities: Vec<String>,
    /// Current health status
    pub health: Health,
    /// Current load (0.0-1.0)
    pub load: f64,
    /// Device IDs assigned to this primal
    pub assigned_devices: Vec<String>,
    /// Additional primal metadata
    pub metadata: serde_json::Value,
}

/// Primal health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Health {
    /// Primal is healthy and functioning normally
    Healthy,
    /// Primal is degraded but still functional
    Degraded,
    /// Primal is offline
    Offline,
}

/// Niche template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NicheTemplate {
    /// Template identifier
    pub id: String,
    /// Template name
    pub name: String,
    /// Template description
    pub description: String,
    /// Required primal capabilities
    pub required_primals: Vec<String>,
    /// Optional primal capabilities
    pub optional_primals: Vec<String>,
    /// Additional template metadata
    pub metadata: serde_json::Value,
}
