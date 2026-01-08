//! Instance management for petalTongue
//!
//! This module provides the foundation for managing multiple petalTongue instances,
//! enabling instance tracking, discovery, and coordination.
//!
//! # Principles
//!
//! - **Self-knowledge only**: Each instance only knows about itself
//! - **Runtime discovery**: Instances discover each other via file-backed registry
//! - **Capability-based**: Features discovered at runtime, not hardcoded
//! - **No mocks**: Real file-backed registry in production
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                   InstanceRegistry                          │
//! │  File: ~/.local/share/petaltongue/instances.json           │
//! │                                                             │
//! │  {                                                          │
//! │    "instances": [                                           │
//! │      {                                                      │
//! │        "id": "uuid-here",                                   │
//! │        "pid": 12345,                                        │
//! │        "window_id": 0x123456,                               │
//! │        "created_at": 1704326400,                            │
//! │        "last_heartbeat": 1704326500,                        │
//! │        "state_path": "~/.local/share/petaltongue/sessions/uuid.ron",
//! │        "socket_path": "/tmp/petaltongue-uuid.sock"          │
//! │      }                                                      │
//! │    ]                                                        │
//! │  }                                                          │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use uuid::Uuid;

/// Unique identifier for a petalTongue instance
///
/// Each instance is assigned a unique UUID at startup. This enables:
/// - Tracking multiple instances
/// - Distinguishing between instances
/// - IPC communication
/// - State isolation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InstanceId(Uuid);

impl InstanceId {
    /// Generate a new unique instance ID
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the UUID as a string
    #[must_use]
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }

    /// Parse an instance ID from a string
    ///
    /// # Errors
    ///
    /// Returns error if the string is not a valid UUID
    /// Parse an `InstanceId` from a string representation
    ///
    /// # Errors
    ///
    /// Returns `InstanceError::InvalidInstanceId` if the string is not a valid UUID
    pub fn parse(s: &str) -> Result<Self, InstanceError> {
        Ok(Self(Uuid::parse_str(s).map_err(|e| {
            InstanceError::InvalidInstanceId(format!("Invalid UUID: {e}"))
        })?))
    }
}

impl Default for InstanceId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for InstanceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Metadata about a petalTongue instance
///
/// Contains all information needed to track, identify, and communicate
/// with a running petalTongue instance.
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
    /// Automatically sets up paths for state and IPC socket.
    ///
    /// # Errors
    ///
    /// Returns error if directories cannot be created
    pub fn new(id: InstanceId, name: Option<String>) -> Result<Self, InstanceError> {
        let pid = std::process::id();
        let now = current_timestamp();

        // Determine base directory (XDG-compliant)
        let base_dir = get_base_dir()?;

        // State directory: ~/.local/share/petaltongue/sessions/
        let state_dir = base_dir.join("sessions");
        fs::create_dir_all(&state_dir).map_err(|e| {
            InstanceError::IoError(format!("Failed to create state directory: {e}"))
        })?;

        let state_path = state_dir.join(format!("{}.ron", id.as_str()));

        // Socket directory: /tmp/petaltongue/ or /run/user/{uid}/petaltongue/
        let socket_dir = get_socket_dir()?;
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
        self.last_heartbeat = current_timestamp();
    }

    /// Set the window ID for this instance
    pub fn set_window_id(&mut self, window_id: u64) {
        self.window_id = Some(window_id);
    }

    /// Check if this instance is likely still alive
    ///
    /// Checks if:
    /// - The process ID exists
    /// - The heartbeat is recent (within 30 seconds)
    #[must_use]
    pub fn is_alive(&self) -> bool {
        // Check if process exists
        if !process_exists(self.pid) {
            return false;
        }

        // Check if heartbeat is recent (within 30 seconds)
        let now = current_timestamp();
        let heartbeat_age = now.saturating_sub(self.last_heartbeat);
        heartbeat_age < 30
    }

    /// Get age in seconds since creation
    #[must_use]
    pub fn age_seconds(&self) -> u64 {
        let now = current_timestamp();
        now.saturating_sub(self.created_at)
    }

    /// Add metadata to this instance
    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }
}

/// Registry of all petalTongue instances
///
/// File-backed singleton that tracks all running instances.
/// Enables instance discovery and coordination.
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

    /// Load registry from disk
    ///
    /// Creates a new empty registry if the file doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be read or parsed
    pub fn load() -> Result<Self, InstanceError> {
        let path = get_registry_path()?;

        if !path.exists() {
            return Ok(Self::new());
        }

        let contents = fs::read_to_string(&path)
            .map_err(|e| InstanceError::IoError(format!("Failed to read registry: {e}")))?;

        ron::from_str(&contents)
            .map_err(|e| InstanceError::ParseError(format!("Failed to parse registry: {e}")))
    }

    /// Save registry to disk
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be written
    pub fn save(&self) -> Result<(), InstanceError> {
        let path = get_registry_path()?;

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                InstanceError::IoError(format!("Failed to create registry directory: {e}"))
            })?;
        }

        let contents = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(|e| InstanceError::SerializeError(format!("Failed to serialize: {e}")))?;

        fs::write(&path, contents)
            .map_err(|e| InstanceError::IoError(format!("Failed to write registry: {e}")))?;

        Ok(())
    }

    /// Register a new instance
    ///
    /// Adds the instance to the registry and saves to disk.
    ///
    /// # Errors
    ///
    /// Returns error if instance cannot be saved
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
    /// Removes the instance from the registry and saves to disk.
    ///
    /// # Errors
    ///
    /// Returns error if registry cannot be saved
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
    /// Returns error if instance not found or cannot be saved
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
    /// Removes instances that are no longer alive and saves to disk.
    ///
    /// # Errors
    ///
    /// Returns error if registry cannot be saved
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

/// Errors that can occur during instance management
#[derive(Debug, Error)]
pub enum InstanceError {
    /// Invalid instance ID
    #[error("Invalid instance ID: {0}")]
    InvalidInstanceId(String),

    /// Instance not found
    #[error("Instance not found: {0}")]
    NotFound(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(String),

    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Serialize error
    #[error("Serialize error: {0}")]
    SerializeError(String),

    /// Directory error
    #[error("Directory error: {0}")]
    DirectoryError(String),
}

// ===== Helper Functions =====

/// Get the current Unix timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

/// Check if a process exists
fn process_exists(pid: u32) -> bool {
    // On Unix, we can check if the process exists by sending signal 0 (null signal)
    // Signal 0 doesn't actually send a signal, it just checks if the process exists
    #[cfg(unix)]
    {
        use nix::sys::signal::kill;
        use nix::unistd::Pid;

        // SAFETY: Converting u32 PID to i32. PIDs are typically small positive numbers.
        // On Linux, max PID is ~4M by default (well within i32 range).
        #[allow(clippy::cast_possible_wrap)]
        match kill(Pid::from_raw(pid as i32), None) {
            Ok(()) | Err(nix::errno::Errno::EPERM) => true, // Process exists (with or without permission)
            Err(nix::errno::Errno::ESRCH) => false,          // No such process
            Err(_) => false,                                  // Other error, assume dead
        }
    }

    // Fallback for non-Unix systems
    #[cfg(not(unix))]
    {
        // On non-Unix, check if /proc/{pid} exists
        std::path::Path::new(&format!("/proc/{}", pid)).exists()
    }
}

/// Get the base directory for petalTongue data
///
/// Uses `XDG_DATA_HOME` or ~/.local/share
fn get_base_dir() -> Result<PathBuf, InstanceError> {
    if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
        Ok(PathBuf::from(xdg_data).join("petaltongue"))
    } else if let Some(home) = dirs::home_dir() {
        Ok(home.join(".local/share/petaltongue"))
    } else {
        Err(InstanceError::DirectoryError(
            "Could not determine home directory".to_string(),
        ))
    }
}

/// Get the socket directory
///
/// Uses /run/user/{uid}/petaltongue or /tmp/petaltongue
///
/// Currently always succeeds, but returns Result for future extensibility
/// (e.g., permission checks, validation)
#[allow(clippy::unnecessary_wraps)]
fn get_socket_dir() -> Result<PathBuf, InstanceError> {
    // Try /run/user/{uid}/petaltongue first (more secure)
    if let Ok(uid) = std::env::var("UID") {
        let run_dir = PathBuf::from(format!("/run/user/{uid}/petaltongue"));
        if run_dir.parent().is_some_and(std::path::Path::exists) {
            return Ok(run_dir);
        }
    }

    // Fall back to /tmp/petaltongue
    Ok(PathBuf::from("/tmp/petaltongue"))
}

/// Get the path to the instance registry file
fn get_registry_path() -> Result<PathBuf, InstanceError> {
    Ok(get_base_dir()?.join("instances.ron"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_id_creation() {
        let id1 = InstanceId::new();
        let id2 = InstanceId::new();
        assert_ne!(id1, id2, "Instance IDs should be unique");
    }

    #[test]
    fn test_instance_id_string_conversion() {
        let id = InstanceId::new();
        let id_str = id.as_str();
        let parsed = InstanceId::parse(&id_str).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_instance_creation() {
        let id = InstanceId::new();
        let instance = Instance::new(id.clone(), Some("test".to_string())).unwrap();

        assert_eq!(instance.id, id);
        assert_eq!(instance.pid, std::process::id());
        assert_eq!(instance.name, Some("test".to_string()));
        assert!(instance.is_alive());
    }

    #[test]
    fn test_instance_heartbeat() {
        let id = InstanceId::new();
        let mut instance = Instance::new(id, None).unwrap();

        let first_heartbeat = instance.last_heartbeat;
        std::thread::sleep(std::time::Duration::from_millis(10));
        instance.heartbeat();

        assert!(instance.last_heartbeat > first_heartbeat);
    }

    #[test]
    fn test_instance_registry() {
        let mut registry = InstanceRegistry::new();
        let id = InstanceId::new();
        let instance = Instance::new(id.clone(), Some("test".to_string())).unwrap();

        registry.register(instance.clone()).unwrap();

        assert_eq!(registry.count(), 1);
        assert!(registry.get(&id).is_some());
        assert_eq!(registry.find_by_name("test").unwrap().id, id);

        registry.unregister(&id).unwrap();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_instance_metadata() {
        let id = InstanceId::new();
        let mut instance = Instance::new(id, None).unwrap();

        instance.add_metadata("key1", "value1");
        instance.add_metadata("key2", "value2");

        assert_eq!(instance.metadata.get("key1"), Some(&"value1".to_string()));
        assert_eq!(instance.metadata.get("key2"), Some(&"value2".to_string()));
    }
}
