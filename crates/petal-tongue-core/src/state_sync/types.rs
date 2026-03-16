// SPDX-License-Identifier: AGPL-3.0-or-later
//! Device state types for cross-device synchronization.

use crate::adaptive_rendering::DeviceType;
use crate::dynamic_schema::DynamicValue;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Device state (serializable, transferable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceState {
    /// Unique device identifier (persistent across sessions)
    pub device_id: String,

    /// Device type
    pub device_type: DeviceType,

    /// Current UI state (device-agnostic)
    pub ui_state: HashMap<String, DynamicValue>,

    /// User preferences
    pub preferences: HashMap<String, DynamicValue>,

    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,

    /// Session metadata
    pub metadata: HashMap<String, String>,
}

impl DeviceState {
    /// Create a new device state
    #[must_use]
    pub fn new(device_id: impl Into<String>, device_type: DeviceType) -> Self {
        let device_id = device_id.into();
        Self {
            device_id,
            device_type,
            ui_state: HashMap::new(),
            preferences: HashMap::new(),
            last_updated: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Set UI state value
    pub fn set_ui_state(&mut self, key: impl Into<String>, value: DynamicValue) {
        self.ui_state.insert(key.into(), value);
        self.last_updated = Utc::now();
    }

    /// Get UI state value
    #[must_use]
    pub fn get_ui_state(&self, key: &str) -> Option<&DynamicValue> {
        self.ui_state.get(key)
    }

    /// Set preference
    pub fn set_preference(&mut self, key: impl Into<String>, value: DynamicValue) {
        self.preferences.insert(key.into(), value);
        self.last_updated = Utc::now();
    }

    /// Get preference
    #[must_use]
    pub fn get_preference(&self, key: &str) -> Option<&DynamicValue> {
        self.preferences.get(key)
    }

    /// Merge state from another device
    pub fn merge(&mut self, other: &Self) {
        // Merge UI state (prefer newer)
        if other.last_updated > self.last_updated {
            for (key, value) in &other.ui_state {
                self.ui_state.insert(key.clone(), value.clone());
            }
            self.last_updated = other.last_updated;
        }

        // Always merge preferences (prefer other if conflict)
        for (key, value) in &other.preferences {
            self.preferences.insert(key.clone(), value.clone());
        }
    }
}
