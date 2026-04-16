// SPDX-License-Identifier: AGPL-3.0-or-later
//! [`Instance`] metadata and lifecycle helpers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use super::lifecycle;
use super::types::{InstanceError, InstanceId};

/// Metadata about a petalTongue instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    /// Unique identifier for this instance
    pub id: InstanceId,

    /// Process ID (for liveness checking)
    pub pid: u32,

    /// X11/Wayland window ID (if known)
    pub window_id: Option<u64>,

    /// When this instance was created (Unix timestamp)
    pub created_at: u64,

    /// Last heartbeat timestamp (Unix timestamp)
    pub last_heartbeat: u64,

    /// Path to this instance's saved state
    pub state_path: PathBuf,

    /// Path to this instance's IPC socket
    pub socket_path: PathBuf,

    /// Optional human-readable name
    pub name: Option<String>,

    /// Instance configuration (optional metadata)
    pub metadata: HashMap<String, String>,
}

impl Instance {
    /// Create a new instance with the given ID
    ///
    /// # Errors
    ///
    /// Returns an error if the data directory cannot be determined, or if the
    /// state or socket directories cannot be created.
    pub fn new(id: InstanceId, name: Option<String>) -> Result<Self, InstanceError> {
        let pid = std::process::id();
        let now = lifecycle::current_timestamp();

        let base_dir = lifecycle::get_base_dir()?;

        let state_dir = base_dir.join("sessions");
        fs::create_dir_all(&state_dir).map_err(|e| {
            InstanceError::IoError(format!("Failed to create state directory: {e}"))
        })?;

        let state_path = state_dir.join(format!("{}.ron", id.as_str()));

        let socket_dir = lifecycle::get_socket_dir()?;
        fs::create_dir_all(&socket_dir).map_err(|e| {
            InstanceError::IoError(format!("Failed to create socket directory: {e}"))
        })?;

        let socket_path = socket_dir.join(format!("{}.sock", id.as_str()));

        Ok(Self {
            id,
            pid,
            window_id: None,
            created_at: now,
            last_heartbeat: now,
            state_path,
            socket_path,
            name,
            metadata: HashMap::new(),
        })
    }

    /// Update the heartbeat timestamp to now
    pub fn heartbeat(&mut self) {
        self.last_heartbeat = lifecycle::current_timestamp();
    }

    /// Set the window ID for this instance
    pub const fn set_window_id(&mut self, window_id: u64) {
        self.window_id = Some(window_id);
    }

    /// Check if this instance is likely still alive
    #[must_use]
    pub fn is_alive(&self) -> bool {
        if !lifecycle::process_exists(self.pid) {
            return false;
        }

        let now = lifecycle::current_timestamp();
        let heartbeat_age = now.saturating_sub(self.last_heartbeat);
        heartbeat_age < 30
    }

    /// Get age in seconds since creation
    #[must_use]
    pub fn age_seconds(&self) -> u64 {
        let now = lifecycle::current_timestamp();
        now.saturating_sub(self.created_at)
    }

    /// Add metadata to this instance
    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }
}
