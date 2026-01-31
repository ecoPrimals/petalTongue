//! State synchronization across devices
//!
//! This module enables petalTongue to maintain consistent state across
//! multiple devices (desktop, phone, watch).
//!
//! # Use Cases
//!
//! 1. **Desktop → Phone**: Start working on desktop, continue on phone
//! 2. **Phone → Watch**: Monitoring while driving, glance at watch
//! 3. **Session Resume**: Close app, reopen later, resume where you left off
//!
//! # Philosophy
//!
//! **State is portable, not device-specific.**
//!
//! Don't tie state to a device. Use device-agnostic data structures
//! that can be serialized, transferred, and deserialized on any device.

use crate::adaptive_rendering::DeviceType;
use crate::dynamic_schema::DynamicValue;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

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
    pub fn new(device_id: String, device_type: DeviceType) -> Self {
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
    pub fn set_ui_state(&mut self, key: String, value: DynamicValue) {
        self.ui_state.insert(key, value);
        self.last_updated = Utc::now();
    }

    /// Get UI state value
    pub fn get_ui_state(&self, key: &str) -> Option<&DynamicValue> {
        self.ui_state.get(key)
    }

    /// Set preference
    pub fn set_preference(&mut self, key: String, value: DynamicValue) {
        self.preferences.insert(key, value);
        self.last_updated = Utc::now();
    }

    /// Get preference
    pub fn get_preference(&self, key: &str) -> Option<&DynamicValue> {
        self.preferences.get(key)
    }

    /// Merge state from another device
    pub fn merge(&mut self, other: &DeviceState) {
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

/// Trait for state persistence
pub trait StatePersistence {
    /// Save state to persistent storage
    fn save(&self, state: &DeviceState) -> Result<()>;

    /// Load state from persistent storage
    fn load(&self, device_id: &str) -> Result<Option<DeviceState>>;

    /// Delete state
    fn delete(&self, device_id: &str) -> Result<()>;
}

/// Local file-based state persistence
pub struct LocalStatePersistence {
    /// Base directory for state files
    base_dir: PathBuf,
}

impl LocalStatePersistence {
    /// Create a new local state persistence
    pub fn new() -> Result<Self> {
        let base_dir = Self::default_state_dir()?;
        std::fs::create_dir_all(&base_dir)
            .with_context(|| format!("Failed to create state directory: {}", base_dir.display()))?;

        Ok(Self { base_dir })
    }

    /// Get default state directory
    ///
    /// Uses platform-specific directory resolution (Pure Rust, zero deps!)
    fn default_state_dir() -> Result<PathBuf> {
        crate::platform_dirs::config_dir()
            .map(|dir| dir.join("petalTongue").join("state"))
            .map_err(|e| anyhow::anyhow!("Could not determine config directory: {}", e))
    }

    /// Get state file path for device
    fn state_file(&self, device_id: &str) -> PathBuf {
        self.base_dir.join(format!("{}.json", device_id))
    }
}

impl StatePersistence for LocalStatePersistence {
    fn save(&self, state: &DeviceState) -> Result<()> {
        let path = self.state_file(&state.device_id);
        let json = serde_json::to_string_pretty(state).context("Failed to serialize state")?;

        std::fs::write(&path, json)
            .with_context(|| format!("Failed to write state file: {}", path.display()))?;

        Ok(())
    }

    fn load(&self, device_id: &str) -> Result<Option<DeviceState>> {
        let path = self.state_file(device_id);

        if !path.exists() {
            return Ok(None);
        }

        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read state file: {}", path.display()))?;

        let state: DeviceState =
            serde_json::from_str(&contents).context("Failed to deserialize state")?;

        Ok(Some(state))
    }

    fn delete(&self, device_id: &str) -> Result<()> {
        let path = self.state_file(device_id);

        if path.exists() {
            std::fs::remove_file(&path)
                .with_context(|| format!("Failed to delete state file: {}", path.display()))?;
        }

        Ok(())
    }
}

/// State synchronization coordinator
pub struct StateSync {
    /// Local persistence
    persistence: Box<dyn StatePersistence>,

    /// Current device state
    current_state: Option<DeviceState>,
}

impl StateSync {
    /// Create a new state sync with local persistence
    pub fn new() -> Result<Self> {
        Ok(Self {
            persistence: Box::new(LocalStatePersistence::new()?),
            current_state: None,
        })
    }

    /// Initialize state for this device
    pub fn init(&mut self, device_id: String, device_type: DeviceType) -> Result<DeviceState> {
        // Try to load existing state
        if let Some(mut state) = self.persistence.load(&device_id)? {
            tracing::info!("📂 Loaded existing state for device {}", device_id);
            state.device_type = device_type; // Update device type
            state.last_updated = Utc::now();
            self.current_state = Some(state.clone());
            return Ok(state);
        }

        // Create new state
        tracing::info!("🆕 Creating new state for device {}", device_id);
        let state = DeviceState::new(device_id, device_type);
        self.current_state = Some(state.clone());
        Ok(state)
    }

    /// Get current state
    pub fn current(&self) -> Option<&DeviceState> {
        self.current_state.as_ref()
    }

    /// Update current state
    pub fn update(&mut self, state: DeviceState) -> Result<()> {
        self.persistence.save(&state)?;
        self.current_state = Some(state);
        Ok(())
    }

    /// Set UI state value
    pub fn set_ui_state(&mut self, key: String, value: DynamicValue) -> Result<()> {
        if let Some(state) = &mut self.current_state {
            state.set_ui_state(key, value);
            self.persistence.save(state)?;
        }
        Ok(())
    }

    /// Get UI state value
    pub fn get_ui_state(&self, key: &str) -> Option<&DynamicValue> {
        self.current_state.as_ref()?.get_ui_state(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_state() {
        let mut state = DeviceState::new("test-device".to_string(), DeviceType::Desktop);

        state.set_ui_state(
            "selected_primal".to_string(),
            DynamicValue::String("beardog".to_string()),
        );
        assert_eq!(
            state
                .get_ui_state("selected_primal")
                .and_then(|v| v.as_str()),
            Some("beardog")
        );

        state.set_preference(
            "theme".to_string(),
            DynamicValue::String("dark".to_string()),
        );
        assert_eq!(
            state.get_preference("theme").and_then(|v| v.as_str()),
            Some("dark")
        );
    }

    #[test]
    fn test_state_merge() {
        let mut state1 = DeviceState::new("device1".to_string(), DeviceType::Desktop);
        state1.set_ui_state(
            "key1".to_string(),
            DynamicValue::String("value1".to_string()),
        );

        let mut state2 = DeviceState::new("device2".to_string(), DeviceType::Phone);
        state2.set_ui_state(
            "key2".to_string(),
            DynamicValue::String("value2".to_string()),
        );
        state2.last_updated = Utc::now(); // Make state2 newer

        state1.merge(&state2);

        // Should have both keys
        assert!(state1.get_ui_state("key1").is_some());
        assert!(state1.get_ui_state("key2").is_some());
    }
}
