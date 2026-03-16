// SPDX-License-Identifier: AGPL-3.0-or-later
//! Instance registry - file-backed tracking of running instances.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::lifecycle::{current_timestamp, get_registry_path};
use super::{Instance, InstanceError, InstanceId};

/// Registry of all petalTongue instances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceRegistry {
    /// Map of instance ID to instance metadata
    instances: HashMap<InstanceId, Instance>,

    /// Last time garbage collection was run
    last_gc: u64,
}

impl Default for InstanceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl InstanceRegistry {
    /// Create a new empty registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            instances: HashMap::new(),
            last_gc: current_timestamp(),
        }
    }

    /// Load registry from the default platform path
    ///
    /// # Errors
    ///
    /// Returns an error if the data directory cannot be determined, if the
    /// registry file cannot be read, or if its contents are invalid RON.
    pub fn load() -> Result<Self, InstanceError> {
        Self::load_from(&get_registry_path()?)
    }

    /// Load registry from an explicit path
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or its contents are invalid RON.
    pub fn load_from(path: &Path) -> Result<Self, InstanceError> {
        if !path.exists() {
            return Ok(Self::new());
        }

        let contents = fs::read_to_string(path)
            .map_err(|e| InstanceError::IoError(format!("Failed to read registry: {e}")))?;

        ron::from_str(&contents)
            .map_err(|e| InstanceError::ParseError(format!("Failed to parse registry: {e}")))
    }

    /// Save registry to the default platform path
    ///
    /// # Errors
    ///
    /// Returns an error if the data directory cannot be determined, if the
    /// registry directory cannot be created, if serialization fails, or if the
    /// file cannot be written.
    pub fn save(&self) -> Result<(), InstanceError> {
        self.save_to(&get_registry_path()?)
    }

    /// Save registry to an explicit path
    ///
    /// # Errors
    ///
    /// Returns an error if the parent directory cannot be created, if
    /// serialization fails, or if the file cannot be written.
    pub fn save_to(&self, path: &Path) -> Result<(), InstanceError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                InstanceError::IoError(format!("Failed to create registry directory: {e}"))
            })?;
        }

        let contents = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(|e| InstanceError::SerializeError(format!("Failed to serialize: {e}")))?;

        fs::write(path, contents)
            .map_err(|e| InstanceError::IoError(format!("Failed to write registry: {e}")))?;

        Ok(())
    }

    /// Register a new instance
    ///
    /// # Errors
    ///
    /// Returns an error if saving the registry to disk fails.
    pub fn register(&mut self, instance: Instance) -> Result<(), InstanceError> {
        tracing::info!(
            "Registering instance: {} (PID: {})",
            instance.id,
            instance.pid
        );
        self.instances.insert(instance.id.clone(), instance);
        self.save()?;
        Ok(())
    }

    /// Unregister an instance
    ///
    /// # Errors
    ///
    /// Returns an error if saving the registry to disk fails.
    pub fn unregister(&mut self, instance_id: &InstanceId) -> Result<(), InstanceError> {
        tracing::info!("Unregistering instance: {}", instance_id);
        self.instances.remove(instance_id);
        self.save()?;
        Ok(())
    }

    /// Update an existing instance
    ///
    /// # Errors
    ///
    /// Returns an error if the instance is not in the registry, or if saving
    /// the registry to disk fails.
    pub fn update(&mut self, instance: Instance) -> Result<(), InstanceError> {
        if !self.instances.contains_key(&instance.id) {
            return Err(InstanceError::NotFound(instance.id.to_string()));
        }

        self.instances.insert(instance.id.clone(), instance);
        self.save()?;
        Ok(())
    }

    /// Get an instance by ID
    #[must_use]
    pub fn get(&self, instance_id: &InstanceId) -> Option<&Instance> {
        self.instances.get(instance_id)
    }

    /// Get a mutable reference to an instance by ID
    pub fn get_mut(&mut self, instance_id: &InstanceId) -> Option<&mut Instance> {
        self.instances.get_mut(instance_id)
    }

    /// List all instances
    #[must_use]
    pub fn list(&self) -> Vec<&Instance> {
        self.instances.values().collect()
    }

    /// List all alive instances
    #[must_use]
    pub fn list_alive(&self) -> Vec<&Instance> {
        self.instances.values().filter(|i| i.is_alive()).collect()
    }

    /// Find instance by window ID
    #[must_use]
    pub fn find_by_window(&self, window_id: u64) -> Option<&Instance> {
        self.instances
            .values()
            .find(|i| i.window_id == Some(window_id))
    }

    /// Find instance by PID
    #[must_use]
    pub fn find_by_pid(&self, pid: u32) -> Option<&Instance> {
        self.instances.values().find(|i| i.pid == pid)
    }

    /// Find instance by name
    #[must_use]
    pub fn find_by_name(&self, name: &str) -> Option<&Instance> {
        self.instances
            .values()
            .find(|i| i.name.as_deref() == Some(name))
    }

    /// Garbage collect dead instances
    ///
    /// # Errors
    ///
    /// Returns an error if any dead instances were removed and saving the
    /// registry to disk fails.
    pub fn gc(&mut self) -> Result<usize, InstanceError> {
        let before = self.instances.len();

        self.instances.retain(|_, instance| instance.is_alive());

        let removed = before - self.instances.len();

        if removed > 0 {
            tracing::info!("Garbage collected {} dead instances", removed);
            self.save()?;
        }

        self.last_gc = current_timestamp();

        Ok(removed)
    }

    /// Get the number of registered instances
    #[must_use]
    pub fn count(&self) -> usize {
        self.instances.len()
    }

    /// Get the number of alive instances
    #[must_use]
    pub fn count_alive(&self) -> usize {
        self.instances.values().filter(|i| i.is_alive()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn load_from_nonexistent_returns_new() {
        let path = std::path::Path::new("/nonexistent/path/instances.ron");
        let registry = InstanceRegistry::load_from(path).unwrap();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn load_from_invalid_ron_fails() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("instances.ron");
        fs::write(&path, "not valid ron {{{").unwrap();
        let result = InstanceRegistry::load_from(&path);
        assert!(result.is_err());
    }

    #[test]
    fn save_to_and_load_from_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("instances.ron");

        crate::test_fixtures::env_test_helpers::with_env_vars(
            &[("XDG_DATA_HOME", Some(dir.path().to_str().unwrap()))],
            || {
                let id = InstanceId::new();
                let instance = Instance::new(id.clone(), Some("roundtrip".to_string())).unwrap();
                let mut registry = InstanceRegistry::new();
                registry.instances.insert(id.clone(), instance);
                registry.save_to(&path).unwrap();

                let loaded = InstanceRegistry::load_from(&path).unwrap();
                assert_eq!(loaded.count(), 1);
                assert!(loaded.get(&id).is_some());
                assert_eq!(loaded.find_by_name("roundtrip").unwrap().id, id);
            },
        );
    }
}
