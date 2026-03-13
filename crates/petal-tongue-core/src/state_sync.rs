// SPDX-License-Identifier: AGPL-3.0-only
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
    #[must_use]
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
    #[must_use]
    pub fn get_ui_state(&self, key: &str) -> Option<&DynamicValue> {
        self.ui_state.get(key)
    }

    /// Set preference
    pub fn set_preference(&mut self, key: String, value: DynamicValue) {
        self.preferences.insert(key, value);
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

    /// Create with explicit base directory (for testing)
    #[cfg(test)]
    pub const fn with_base_dir(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Get default state directory
    ///
    /// Uses platform-specific directory resolution (Pure Rust, zero deps!)
    fn default_state_dir() -> Result<PathBuf> {
        crate::platform_dirs::config_dir()
            .map(|dir| dir.join("petalTongue").join("state"))
            .map_err(|e| anyhow::anyhow!("Could not determine config directory: {e}"))
    }

    /// Get state file path for device
    fn state_file(&self, device_id: &str) -> PathBuf {
        self.base_dir.join(format!("{device_id}.json"))
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

    /// Create with custom persistence (for testing)
    #[cfg(test)]
    pub fn with_persistence(persistence: Box<dyn StatePersistence>) -> Self {
        Self {
            persistence,
            current_state: None,
        }
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
    #[must_use]
    pub const fn current(&self) -> Option<&DeviceState> {
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
    #[must_use]
    pub fn get_ui_state(&self, key: &str) -> Option<&DynamicValue> {
        self.current_state.as_ref()?.get_ui_state(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// In-memory StatePersistence for deterministic tests
    struct InMemoryPersistence {
        storage: std::sync::Arc<Mutex<HashMap<String, DeviceState>>>,
    }

    impl InMemoryPersistence {
        fn new() -> Self {
            Self {
                storage: std::sync::Arc::new(Mutex::new(HashMap::new())),
            }
        }

        /// Create a pair that shares storage (for tests that need to verify persistence)
        fn shared() -> (Self, Self) {
            let storage = std::sync::Arc::new(Mutex::new(HashMap::new()));
            (
                Self {
                    storage: std::sync::Arc::clone(&storage),
                },
                Self { storage },
            )
        }

        fn with_shared_storage(
            storage: &std::sync::Arc<Mutex<HashMap<String, DeviceState>>>,
        ) -> Self {
            Self {
                storage: std::sync::Arc::clone(storage),
            }
        }
    }

    impl StatePersistence for InMemoryPersistence {
        fn save(&self, state: &DeviceState) -> Result<()> {
            self.storage
                .lock()
                .expect("state_sync: lock poisoned — unrecoverable")
                .insert(state.device_id.clone(), state.clone());
            Ok(())
        }

        fn load(&self, device_id: &str) -> Result<Option<DeviceState>> {
            Ok(self
                .storage
                .lock()
                .expect("state_sync: lock poisoned — unrecoverable")
                .get(device_id)
                .cloned())
        }

        fn delete(&self, device_id: &str) -> Result<()> {
            self.storage
                .lock()
                .expect("state_sync: lock poisoned — unrecoverable")
                .remove(device_id);
            Ok(())
        }
    }

    #[test]
    fn test_device_state_new() {
        let state = DeviceState::new("dev-1".to_string(), DeviceType::Phone);
        assert_eq!(state.device_id, "dev-1");
        assert_eq!(state.device_type, DeviceType::Phone);
        assert!(state.ui_state.is_empty());
        assert!(state.preferences.is_empty());
        assert!(state.metadata.is_empty());
    }

    #[test]
    fn test_device_state_ui_state() {
        let mut state = DeviceState::new("test-device".to_string(), DeviceType::Desktop);

        state.set_ui_state(
            "selected_primal".to_string(),
            DynamicValue::String("gpu-display".to_string()),
        );
        assert_eq!(
            state
                .get_ui_state("selected_primal")
                .and_then(|v| v.as_str()),
            Some("gpu-display")
        );

        state.set_ui_state("count".to_string(), DynamicValue::Number(42.0));
        assert_eq!(
            state.get_ui_state("count").and_then(DynamicValue::as_f64),
            Some(42.0)
        );

        assert!(state.get_ui_state("nonexistent").is_none());
    }

    #[test]
    fn test_device_state_preferences() {
        let mut state = DeviceState::new("test-device".to_string(), DeviceType::Desktop);

        state.set_preference(
            "theme".to_string(),
            DynamicValue::String("dark".to_string()),
        );
        assert_eq!(
            state.get_preference("theme").and_then(|v| v.as_str()),
            Some("dark")
        );

        state.set_preference("volume".to_string(), DynamicValue::Number(0.8));
        assert_eq!(
            state
                .get_preference("volume")
                .and_then(DynamicValue::as_f64),
            Some(0.8)
        );

        assert!(state.get_preference("missing").is_none());
    }

    #[test]
    fn test_state_merge_newer_wins() {
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

        // Should have both keys (state2 is newer, so its UI state merges)
        assert!(state1.get_ui_state("key1").is_some());
        assert!(state1.get_ui_state("key2").is_some());
    }

    #[test]
    fn test_state_merge_older_ignored() {
        let mut state1 = DeviceState::new("device1".to_string(), DeviceType::Desktop);
        state1.set_ui_state(
            "key1".to_string(),
            DynamicValue::String("value1".to_string()),
        );
        state1.last_updated = Utc::now();

        let mut state2 = DeviceState::new("device2".to_string(), DeviceType::Phone);
        state2.set_ui_state(
            "key2".to_string(),
            DynamicValue::String("value2".to_string()),
        );
        state2.last_updated = state1.last_updated - chrono::Duration::seconds(1);

        state1.merge(&state2);

        // state2 is older - UI state from state2 should NOT be merged
        assert!(state1.get_ui_state("key1").is_some());
        assert!(state1.get_ui_state("key2").is_none());
    }

    #[test]
    fn test_state_merge_preferences_always() {
        let mut state1 = DeviceState::new("device1".to_string(), DeviceType::Desktop);
        state1.set_preference(
            "theme".to_string(),
            DynamicValue::String("light".to_string()),
        );

        let mut state2 = DeviceState::new("device2".to_string(), DeviceType::Phone);
        state2.set_preference(
            "theme".to_string(),
            DynamicValue::String("dark".to_string()),
        );
        state2.last_updated = state1.last_updated - chrono::Duration::seconds(1);

        state1.merge(&state2);

        // Preferences always merge (other wins on conflict)
        assert_eq!(
            state1.get_preference("theme").and_then(|v| v.as_str()),
            Some("dark")
        );
    }

    #[test]
    fn test_state_sync_with_persistence_init_new() {
        let persistence = InMemoryPersistence::new();
        let mut sync = StateSync::with_persistence(Box::new(persistence));

        let state = sync
            .init("device-1".to_string(), DeviceType::Desktop)
            .unwrap();

        assert_eq!(state.device_id, "device-1");
        assert_eq!(state.device_type, DeviceType::Desktop);
        assert!(sync.current().is_some());
        assert_eq!(sync.current().unwrap().device_id, "device-1");
    }

    #[test]
    fn test_state_sync_init_loads_existing() {
        let persistence = InMemoryPersistence::new();
        let mut existing = DeviceState::new("device-1".to_string(), DeviceType::Phone);
        existing.set_ui_state(
            "saved".to_string(),
            DynamicValue::String("value".to_string()),
        );
        persistence.save(&existing).unwrap();

        let mut sync = StateSync::with_persistence(Box::new(persistence));
        let state = sync
            .init("device-1".to_string(), DeviceType::Desktop)
            .unwrap();

        assert_eq!(state.device_id, "device-1");
        assert_eq!(state.device_type, DeviceType::Desktop); // Updated
        assert_eq!(
            state.get_ui_state("saved").and_then(|v| v.as_str()),
            Some("value")
        );
    }

    #[test]
    fn test_state_sync_update() {
        let (persistence, persistence_reader) = InMemoryPersistence::shared();
        let mut sync = StateSync::with_persistence(Box::new(persistence));
        sync.init("device-1".to_string(), DeviceType::Desktop)
            .unwrap();

        let mut state = sync.current().unwrap().clone();
        state.set_ui_state("key".to_string(), DynamicValue::String("val".to_string()));

        sync.update(state).unwrap();

        let loaded = persistence_reader.load("device-1").unwrap().unwrap();
        assert_eq!(
            loaded.get_ui_state("key").and_then(|v| v.as_str()),
            Some("val")
        );
    }

    #[test]
    fn test_state_sync_set_get_ui_state() {
        let persistence = InMemoryPersistence::new();
        let mut sync = StateSync::with_persistence(Box::new(persistence));
        sync.init("device-1".to_string(), DeviceType::Desktop)
            .unwrap();

        sync.set_ui_state("k1".to_string(), DynamicValue::String("v1".to_string()))
            .unwrap();
        assert_eq!(sync.get_ui_state("k1").and_then(|v| v.as_str()), Some("v1"));

        sync.set_ui_state("k2".to_string(), DynamicValue::Number(99.0))
            .unwrap();
        assert_eq!(
            sync.get_ui_state("k2").and_then(DynamicValue::as_f64),
            Some(99.0)
        );
    }

    #[test]
    fn test_state_sync_set_ui_state_no_current() {
        let persistence = InMemoryPersistence::new();
        let mut sync = StateSync::with_persistence(Box::new(persistence));

        sync.set_ui_state("k".to_string(), DynamicValue::String("v".to_string()))
            .unwrap();
        assert!(sync.get_ui_state("k").is_none());
    }

    #[test]
    fn test_state_sync_current_none() {
        let persistence = InMemoryPersistence::new();
        let sync = StateSync::with_persistence(Box::new(persistence));
        assert!(sync.current().is_none());
    }

    #[test]
    fn test_local_persistence_save_load_delete() {
        let temp = std::env::temp_dir().join("petal-state-test");
        let _ = std::fs::remove_dir_all(&temp);
        std::fs::create_dir_all(&temp).unwrap();

        let persistence = LocalStatePersistence::with_base_dir(temp.clone());

        let mut state = DeviceState::new("test-dev".to_string(), DeviceType::Desktop);
        state.set_ui_state("x".to_string(), DynamicValue::String("y".to_string()));

        persistence.save(&state).unwrap();
        let loaded = persistence.load("test-dev").unwrap().unwrap();
        assert_eq!(loaded.device_id, "test-dev");
        assert_eq!(loaded.get_ui_state("x").and_then(|v| v.as_str()), Some("y"));

        persistence.delete("test-dev").unwrap();
        assert!(persistence.load("test-dev").unwrap().is_none());

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_local_persistence_load_nonexistent() {
        let temp = std::env::temp_dir().join("petal-state-nonexistent");
        let _ = std::fs::remove_dir_all(&temp);
        std::fs::create_dir_all(&temp).unwrap();

        let persistence = LocalStatePersistence::with_base_dir(temp.clone());
        assert!(persistence.load("no-such-device").unwrap().is_none());

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_state_merge_ui_state_overwrite() {
        let mut state1 = DeviceState::new("device1".to_string(), DeviceType::Desktop);
        state1.set_ui_state("key".to_string(), DynamicValue::String("old".to_string()));
        state1.last_updated = Utc::now() - chrono::Duration::seconds(10);

        let mut state2 = DeviceState::new("device2".to_string(), DeviceType::Phone);
        state2.set_ui_state("key".to_string(), DynamicValue::String("new".to_string()));
        state2.last_updated = Utc::now();

        state1.merge(&state2);
        assert_eq!(
            state1.get_ui_state("key").and_then(|v| v.as_str()),
            Some("new")
        );
    }

    #[test]
    fn test_device_state_metadata() {
        let mut state = DeviceState::new("dev".to_string(), DeviceType::Desktop);
        assert!(state.metadata.is_empty());
        state
            .metadata
            .insert("source".to_string(), "test".to_string());
        assert_eq!(state.metadata.get("source"), Some(&"test".to_string()));
    }

    #[test]
    fn test_state_sync_get_ui_state_none_before_init() {
        let persistence = InMemoryPersistence::new();
        let sync = StateSync::with_persistence(Box::new(persistence));
        assert!(sync.get_ui_state("any").is_none());
    }

    #[test]
    fn test_device_state_merge_same_timestamp() {
        let mut state1 = DeviceState::new("device1".to_string(), DeviceType::Desktop);
        state1.set_ui_state("a".to_string(), DynamicValue::String("1".to_string()));

        let mut state2 = DeviceState::new("device2".to_string(), DeviceType::Phone);
        state2.set_ui_state("b".to_string(), DynamicValue::String("2".to_string()));
        state2.last_updated = state1.last_updated;

        state1.merge(&state2);
        assert!(state1.get_ui_state("a").is_some());
        assert!(state1.get_ui_state("b").is_none());
    }

    #[test]
    fn test_device_state_empty_merge() {
        let mut state1 = DeviceState::new("device1".to_string(), DeviceType::Desktop);
        let state2 = DeviceState::new("device2".to_string(), DeviceType::Phone);
        state1.merge(&state2);
        assert!(state1.ui_state.is_empty());
        assert!(state1.preferences.is_empty());
    }

    #[test]
    fn test_device_state_null_field_handling() {
        let mut state = DeviceState::new("dev".to_string(), DeviceType::Desktop);
        state.set_ui_state("null_key".to_string(), DynamicValue::Null);
        assert!(state.get_ui_state("null_key").unwrap().is_null());
        assert!(state.get_ui_state("null_key").is_some());
    }

    #[test]
    fn test_device_state_array_value() {
        let mut state = DeviceState::new("dev".to_string(), DeviceType::Desktop);
        state.set_ui_state(
            "items".to_string(),
            DynamicValue::Array(vec![
                DynamicValue::Number(1.0),
                DynamicValue::String("x".to_string()),
            ]),
        );
        let arr = state.get_ui_state("items").and_then(DynamicValue::as_array);
        assert_eq!(arr.map(<[_]>::len), Some(2));
    }

    #[test]
    fn test_device_state_object_value() {
        let mut obj = HashMap::new();
        obj.insert("nested".to_string(), DynamicValue::Number(99.0));
        let mut state = DeviceState::new("dev".to_string(), DeviceType::Desktop);
        state.set_ui_state("config".to_string(), DynamicValue::Object(obj));
        let inner = state
            .get_ui_state("config")
            .and_then(DynamicValue::as_object)
            .and_then(|o| o.get("nested"));
        assert_eq!(inner.and_then(DynamicValue::as_f64), Some(99.0));
    }

    #[test]
    fn test_state_sync_update_overwrites() {
        let persistence = InMemoryPersistence::new();
        let mut sync = StateSync::with_persistence(Box::new(persistence));
        sync.init("device-1".to_string(), DeviceType::Desktop)
            .unwrap();
        sync.set_ui_state("k".to_string(), DynamicValue::String("v1".to_string()))
            .unwrap();
        let mut state = sync.current().unwrap().clone();
        state.set_ui_state("k".to_string(), DynamicValue::String("v2".to_string()));
        sync.update(state).unwrap();
        assert_eq!(sync.get_ui_state("k").and_then(|v| v.as_str()), Some("v2"));
    }

    #[test]
    fn test_state_sync_get_ui_state_missing_key() {
        let persistence = InMemoryPersistence::new();
        let mut sync = StateSync::with_persistence(Box::new(persistence));
        sync.init("device-1".to_string(), DeviceType::Desktop)
            .unwrap();
        assert!(sync.get_ui_state("nonexistent").is_none());
    }

    #[test]
    fn test_device_state_merge_preferences_overwrite() {
        let mut state1 = DeviceState::new("device1".to_string(), DeviceType::Desktop);
        state1.set_preference("volume".to_string(), DynamicValue::Number(0.5));
        state1.last_updated = Utc::now();

        let mut state2 = DeviceState::new("device2".to_string(), DeviceType::Phone);
        state2.set_preference("volume".to_string(), DynamicValue::Number(1.0));
        state2.last_updated = state1.last_updated - chrono::Duration::seconds(1);

        state1.merge(&state2);
        assert_eq!(
            state1
                .get_preference("volume")
                .and_then(DynamicValue::as_f64),
            Some(1.0)
        );
    }

    #[test]
    fn test_local_persistence_delete_nonexistent() {
        let temp = std::env::temp_dir().join("petal-state-delete-test");
        let _ = std::fs::remove_dir_all(&temp);
        std::fs::create_dir_all(&temp).unwrap();

        let persistence = LocalStatePersistence::with_base_dir(temp.clone());
        let result = persistence.delete("never-existed");
        assert!(result.is_ok());

        let _ = std::fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_device_type_serialization() {
        let state = DeviceState::new("dev".to_string(), DeviceType::Watch);
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("device_id"));
        let restored: DeviceState = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.device_type, DeviceType::Watch);
    }

    #[test]
    fn test_concurrent_persistence_access() {
        let storage = std::sync::Arc::new(Mutex::new(HashMap::new()));
        let p1 = InMemoryPersistence::with_shared_storage(&storage);
        let p2 = InMemoryPersistence::with_shared_storage(&storage);
        let p3 = InMemoryPersistence::with_shared_storage(&storage);
        let h1 = std::thread::spawn(move || {
            let mut sync = StateSync::with_persistence(Box::new(p1));
            sync.init("dev-a".to_string(), DeviceType::Desktop).unwrap();
            sync.set_ui_state("k".to_string(), DynamicValue::String("v1".to_string()))
                .unwrap();
        });
        let h2 = std::thread::spawn(move || {
            let mut sync = StateSync::with_persistence(Box::new(p2));
            sync.init("dev-b".to_string(), DeviceType::Phone).unwrap();
            sync.set_ui_state("k".to_string(), DynamicValue::String("v2".to_string()))
                .unwrap();
        });
        h1.join().unwrap();
        h2.join().unwrap();
        let a = p3.load("dev-a").unwrap();
        let b = p3.load("dev-b").unwrap();
        assert!(a.is_some());
        assert!(b.is_some());
        assert_eq!(
            a.unwrap().get_ui_state("k").and_then(|v| v.as_str()),
            Some("v1")
        );
        assert_eq!(
            b.unwrap().get_ui_state("k").and_then(|v| v.as_str()),
            Some("v2")
        );
    }

    #[test]
    fn test_state_transition_init_update_cycle() {
        let (persistence, reader) = InMemoryPersistence::shared();
        let mut sync = StateSync::with_persistence(Box::new(persistence));
        sync.init("dev".to_string(), DeviceType::Desktop).unwrap();
        let mut state = sync.current().unwrap().clone();
        state.set_ui_state(
            "phase".to_string(),
            DynamicValue::String("running".to_string()),
        );
        sync.update(state).unwrap();
        let loaded = reader.load("dev").unwrap().unwrap();
        assert_eq!(
            loaded.get_ui_state("phase").and_then(|v| v.as_str()),
            Some("running")
        );
    }

    #[test]
    fn test_conflict_resolution_merge_timestamp() {
        let mut state1 = DeviceState::new("dev1".to_string(), DeviceType::Desktop);
        state1.set_ui_state("x".to_string(), DynamicValue::String("old".to_string()));
        state1.last_updated = Utc::now() - chrono::Duration::seconds(5);

        let mut state2 = DeviceState::new("dev2".to_string(), DeviceType::Phone);
        state2.set_ui_state("x".to_string(), DynamicValue::String("new".to_string()));
        state2.set_ui_state("y".to_string(), DynamicValue::Number(1.0));
        state2.last_updated = Utc::now();

        state1.merge(&state2);
        assert_eq!(
            state1.get_ui_state("x").and_then(|v| v.as_str()),
            Some("new")
        );
    }

    #[test]
    fn test_device_state_merge_multiple_preferences() {
        let mut state1 = DeviceState::new("dev1".to_string(), DeviceType::Desktop);
        state1.set_preference("a".to_string(), DynamicValue::String("1".to_string()));
        state1.set_preference("b".to_string(), DynamicValue::Number(2.0));

        let mut state2 = DeviceState::new("dev2".to_string(), DeviceType::Phone);
        state2.set_preference("b".to_string(), DynamicValue::Number(99.0));
        state2.set_preference("c".to_string(), DynamicValue::Boolean(true));
        state2.last_updated = state1.last_updated - chrono::Duration::seconds(1);

        state1.merge(&state2);
        assert_eq!(
            state1.get_preference("a").and_then(|v| v.as_str()),
            Some("1")
        );
        assert_eq!(
            state1.get_preference("b").and_then(DynamicValue::as_f64),
            Some(99.0)
        );
        assert_eq!(
            state1.get_preference("c").and_then(DynamicValue::as_bool),
            Some(true)
        );
    }

    #[test]
    fn test_in_memory_persistence_shared_isolation() {
        let (p1, p2) = InMemoryPersistence::shared();
        let state = DeviceState::new("dev".to_string(), DeviceType::Desktop);
        p1.save(&state).unwrap();
        let loaded = p2.load("dev").unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().device_id, "dev");
    }

    #[test]
    fn test_state_sync_init_twice_returns_cached() {
        let persistence = InMemoryPersistence::new();
        let mut sync = StateSync::with_persistence(Box::new(persistence));
        let s1 = sync.init("dev".to_string(), DeviceType::Desktop).unwrap();
        let s2 = sync.init("dev".to_string(), DeviceType::Phone).unwrap();
        assert_eq!(s1.device_id, s2.device_id);
        assert_eq!(s2.device_type, DeviceType::Phone);
    }
}

#[cfg(test)]
mod proptest_tests {
    use super::*;
    use crate::adaptive_rendering::DeviceType;
    use crate::dynamic_schema::DynamicValue;
    use proptest::prelude::*;

    proptest! {
        /// set_ui_state then get_ui_state returns the value (finite numbers; NaN != NaN).
        #[test]
        fn prop_set_get_ui_state_roundtrip(key in "\\PC{1,20}", val in proptest::num::f64::NORMAL) {
            let key = if key.is_empty() { "k".to_string() } else { key };
            let mut state = DeviceState::new("dev".to_string(), DeviceType::Desktop);
            state.set_ui_state(key.clone(), DynamicValue::Number(val));
            prop_assert_eq!(state.get_ui_state(&key).and_then(DynamicValue::as_f64), Some(val));
        }

        /// set_preference then get_preference returns the value (finite numbers; NaN != NaN).
        #[test]
        fn prop_set_get_preference_roundtrip(key in "\\PC{1,20}", val in proptest::num::f64::NORMAL) {
            let key = if key.is_empty() { "k".to_string() } else { key };
            let mut state = DeviceState::new("dev".to_string(), DeviceType::Desktop);
            state.set_preference(key.clone(), DynamicValue::Number(val));
            prop_assert_eq!(state.get_preference(&key).and_then(DynamicValue::as_f64), Some(val));
        }

        /// DeviceState JSON serialization roundtrip preserves device_id and type.
        #[test]
        fn prop_device_state_serialization_roundtrip(device_id in "\\w{1,30}") {
            let device_id = if device_id.is_empty() { "dev".to_string() } else { device_id };
            let state = DeviceState::new(device_id.clone(), DeviceType::Phone);
            let json = serde_json::to_string(&state).unwrap();
            let restored: DeviceState = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(restored.device_id, device_id);
            prop_assert_eq!(restored.device_type, DeviceType::Phone);
        }
    }
}
