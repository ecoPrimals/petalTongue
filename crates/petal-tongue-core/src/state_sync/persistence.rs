// SPDX-License-Identifier: AGPL-3.0-only
//! State persistence backends (local file, in-memory for tests).

use std::path::PathBuf;

use crate::error::{PetalTongueError, Result};

use super::types::DeviceState;

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
        std::fs::create_dir_all(&base_dir).map_err(|e| {
            PetalTongueError::Io(std::io::Error::other(format!(
                "Failed to create state directory: {}: {e}",
                base_dir.display()
            )))
        })?;

        Ok(Self { base_dir })
    }

    /// Create with explicit base directory (for testing)
    #[cfg(test)]
    pub const fn with_base_dir(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Get default state directory
    fn default_state_dir() -> Result<PathBuf> {
        crate::platform_dirs::config_dir()
            .map(|dir| dir.join("petalTongue").join("state"))
            .map_err(|e| PetalTongueError::ConfigDir(e.to_string()))
    }

    /// Get state file path for device
    fn state_file(&self, device_id: &str) -> PathBuf {
        self.base_dir.join(format!("{device_id}.json"))
    }
}

impl StatePersistence for LocalStatePersistence {
    fn save(&self, state: &DeviceState) -> Result<()> {
        let path = self.state_file(&state.device_id);
        let json = serde_json::to_string_pretty(state)
            .map_err(|e| PetalTongueError::Json(format!("Failed to serialize state: {e}")))?;

        std::fs::write(&path, json)?;

        Ok(())
    }

    fn load(&self, device_id: &str) -> Result<Option<DeviceState>> {
        let path = self.state_file(device_id);

        if !path.exists() {
            return Ok(None);
        }

        let contents = std::fs::read_to_string(&path)?;

        let state: DeviceState = serde_json::from_str(&contents)
            .map_err(|e| PetalTongueError::Json(format!("Failed to deserialize state: {e}")))?;

        Ok(Some(state))
    }

    fn delete(&self, device_id: &str) -> Result<()> {
        let path = self.state_file(device_id);

        if path.exists() {
            std::fs::remove_file(&path)?;
        }

        Ok(())
    }
}
