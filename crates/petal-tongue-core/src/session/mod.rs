// SPDX-License-Identifier: AGPL-3.0-only
//! Session state management for petalTongue
//!
//! This module provides comprehensive state persistence, enabling petalTongue to:
//! - Save complete application state to disk
//! - Restore state on launch
//! - Auto-save on changes
//! - Export/import sessions for transfer
//! - Merge sessions from multiple instances

mod persistence;
mod validation;

use crate::instance::InstanceId;
use crate::{LayoutAlgorithm, PrimalInfo, TopologyEdge};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Complete application state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// Format version for future compatibility
    pub version: u32,

    /// Instance that created this session
    pub instance_id: InstanceId,

    /// When this session was saved (Unix timestamp)
    pub timestamp: u64,

    /// Human-readable session name (optional)
    pub name: Option<String>,

    /// All nodes in the graph
    pub nodes: Vec<PrimalInfo>,

    /// All edges in the graph
    pub edges: Vec<TopologyEdge>,

    /// Current layout algorithm
    pub layout: LayoutAlgorithm,

    /// Node positions (after layout)
    pub node_positions: HashMap<String, (f32, f32)>,

    /// Window position (x, y)
    pub window_position: Option<(i32, i32)>,

    /// Window size (width, height)
    pub window_size: Option<(u32, u32)>,

    /// Current zoom level
    pub zoom_level: f32,

    /// Pan offset (x, y)
    pub pan_offset: (f32, f32),

    /// Which panels are open
    pub panels_open: HashSet<String>,

    /// Accessibility settings
    pub accessibility: AccessibilitySettings,

    /// Active scenario (if in showcase mode)
    pub active_scenario: Option<String>,

    /// Auto-refresh enabled
    pub auto_refresh: bool,

    /// Refresh interval (seconds)
    pub refresh_interval: f32,

    /// Trust dashboard summary
    pub trust_summary: Option<TrustSummary>,

    /// Custom metadata (extensible)
    pub metadata: HashMap<String, String>,
}

/// Accessibility settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilitySettings {
    /// Selected color scheme
    pub color_scheme: String,

    /// Font size multiplier
    pub font_size: f32,

    /// Audio enabled
    pub audio_enabled: bool,

    /// Audio volume (0.0-1.0)
    pub audio_volume: f32,
}

impl Default for AccessibilitySettings {
    fn default() -> Self {
        Self {
            color_scheme: "standard".to_string(),
            font_size: 1.0,
            audio_enabled: false,
            audio_volume: 0.7,
        }
    }
}

/// Trust dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustSummary {
    /// Total number of primals
    pub total_primals: usize,

    /// Trust level distribution (level -> count)
    pub trust_distribution: HashMap<u8, usize>,

    /// Unique family IDs
    pub unique_families: HashSet<String>,

    /// Average trust level
    pub average_trust: f32,
}

impl Default for TrustSummary {
    fn default() -> Self {
        Self {
            total_primals: 0,
            trust_distribution: HashMap::new(),
            unique_families: HashSet::new(),
            average_trust: 0.0,
        }
    }
}

impl SessionState {
    /// Current session format version
    pub const VERSION: u32 = 1;

    /// Create a new session state
    #[must_use]
    pub fn new(instance_id: InstanceId) -> Self {
        Self {
            version: Self::VERSION,
            instance_id,
            timestamp: persistence::current_timestamp(),
            name: None,
            nodes: Vec::new(),
            edges: Vec::new(),
            layout: LayoutAlgorithm::ForceDirected,
            node_positions: HashMap::new(),
            window_position: None,
            window_size: None,
            zoom_level: 1.0,
            pan_offset: (0.0, 0.0),
            panels_open: HashSet::new(),
            accessibility: AccessibilitySettings::default(),
            active_scenario: None,
            auto_refresh: true,
            refresh_interval: 5.0,
            trust_summary: None,
            metadata: HashMap::new(),
        }
    }

    /// Save session to disk (atomic write)
    pub fn save(&self, path: &Path) -> Result<(), SessionError> {
        persistence::save_session(self, path)
    }

    /// Load session from disk
    pub fn load(path: &Path) -> Result<Self, SessionError> {
        persistence::load_session(path)
    }

    /// Export session to a specific path (for sharing)
    pub fn export(&self, path: &Path) -> Result<(), SessionError> {
        self.save(path)
    }

    /// Import session from a specific path
    pub fn import(path: &Path) -> Result<Self, SessionError> {
        Self::load(path)
    }

    /// Merge another session's graph data into this one
    pub fn merge_graph(&mut self, other: &Self) {
        let existing_ids: HashSet<_> = self.nodes.iter().map(|n| n.id.clone()).collect();

        for node in &other.nodes {
            if !existing_ids.contains(&node.id) {
                self.nodes.push(node.clone());
            }
        }

        let existing_edges: HashSet<_> = self
            .edges
            .iter()
            .map(|e| (e.from.clone(), e.to.clone(), e.edge_type.clone()))
            .collect();

        for edge in &other.edges {
            let edge_key = (edge.from.clone(), edge.to.clone(), edge.edge_type.clone());
            if !existing_edges.contains(&edge_key) {
                self.edges.push(edge.clone());
            }
        }

        for (node_id, position) in &other.node_positions {
            self.node_positions.insert(node_id.clone(), *position);
        }

        self.timestamp = persistence::current_timestamp();

        tracing::info!(
            "Merged session: {} nodes, {} edges",
            self.nodes.len(),
            self.edges.len()
        );
    }

    /// Update timestamp to now
    pub fn touch(&mut self) {
        self.timestamp = persistence::current_timestamp();
    }

    /// Add custom metadata
    pub fn add_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }

    /// Get session age in seconds
    #[must_use]
    pub fn age_seconds(&self) -> u64 {
        let now = persistence::current_timestamp();
        now.saturating_sub(self.timestamp)
    }
}

/// Session manager for auto-save and restore operations
pub struct SessionManager {
    session_path: PathBuf,
    current_state: Option<SessionState>,
    auto_save_enabled: bool,
    last_save: u64,
    auto_save_interval: u64,
    dirty: bool,
}

impl SessionManager {
    /// Create a new session manager for an instance
    pub fn new(instance_id: &InstanceId) -> Result<Self, SessionError> {
        let session_path = persistence::get_session_path(instance_id)?;

        Ok(Self {
            session_path,
            current_state: None,
            auto_save_enabled: true,
            last_save: persistence::current_timestamp(),
            auto_save_interval: 30,
            dirty: false,
        })
    }

    /// Create a `SessionManager` that stores its session at an explicit path
    pub fn with_session_path(session_path: PathBuf) -> Result<Self, SessionError> {
        if let Some(parent) = session_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                SessionError::IoError(format!("Failed to create session directory: {e}"))
            })?;
        }
        Ok(Self {
            session_path,
            current_state: None,
            auto_save_enabled: true,
            last_save: persistence::current_timestamp(),
            auto_save_interval: 30,
            dirty: false,
        })
    }

    /// Get the session path
    #[must_use]
    pub fn session_path(&self) -> &Path {
        &self.session_path
    }

    /// Load or create session
    pub fn load_or_create(
        &mut self,
        instance_id: InstanceId,
    ) -> Result<&SessionState, SessionError> {
        if self.session_path.exists() {
            tracing::info!("Restoring session from: {}", self.session_path.display());
            self.current_state = Some(SessionState::load(&self.session_path)?);
        } else {
            tracing::info!("Creating new session");
            self.current_state = Some(SessionState::new(instance_id));
            self.dirty = true;
        }

        self.current_state.as_ref().ok_or(SessionError::NoState)
    }

    /// Get the current session state
    #[must_use]
    pub fn current_state(&self) -> Option<&SessionState> {
        self.current_state.as_ref()
    }

    /// Get mutable access to the current session state
    pub fn current_state_mut(&mut self) -> Option<&mut SessionState> {
        if self.current_state.is_some() {
            self.dirty = true;
        }
        self.current_state.as_mut()
    }

    /// Update the session state
    pub fn update_state(&mut self, state: SessionState) {
        self.current_state = Some(state);
        self.dirty = true;
    }

    /// Mark session as dirty (needing save)
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Save the current session
    pub fn save(&mut self) -> Result<(), SessionError> {
        if let Some(state) = &self.current_state {
            state.save(&self.session_path)?;
            self.last_save = persistence::current_timestamp();
            self.dirty = false;
            Ok(())
        } else {
            Err(SessionError::NoState)
        }
    }

    /// Auto-save if needed
    pub fn auto_save_if_needed(&mut self) -> Result<bool, SessionError> {
        if !self.auto_save_enabled || !self.dirty {
            return Ok(false);
        }

        let now = persistence::current_timestamp();
        let time_since_save = now.saturating_sub(self.last_save);

        if time_since_save >= self.auto_save_interval {
            self.save()?;
            tracing::debug!("Auto-saved session");
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Force save immediately (even if not dirty)
    pub fn force_save(&mut self) -> Result<(), SessionError> {
        if let Some(state) = &self.current_state {
            state.save(&self.session_path)?;
            self.last_save = persistence::current_timestamp();
            self.dirty = false;
            Ok(())
        } else {
            Err(SessionError::NoState)
        }
    }

    /// Enable or disable auto-save
    pub fn set_auto_save(&mut self, enabled: bool) {
        self.auto_save_enabled = enabled;
    }

    /// Set auto-save interval in seconds
    pub fn set_auto_save_interval(&mut self, seconds: u64) {
        self.auto_save_interval = seconds.max(1);
    }

    /// Check if session has unsaved changes
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Export current session to a path
    pub fn export(&self, path: &Path) -> Result<(), SessionError> {
        if let Some(state) = &self.current_state {
            state.export(path)
        } else {
            Err(SessionError::NoState)
        }
    }

    /// Import session from a path and make it current
    pub fn import(&mut self, path: &Path) -> Result<(), SessionError> {
        let state = SessionState::import(path)?;
        self.current_state = Some(state);
        self.dirty = true;
        Ok(())
    }

    /// Check if session has unsaved changes (alias for `is_dirty`)
    #[must_use]
    pub fn has_unsaved_changes(&self) -> bool {
        self.is_dirty()
    }

    /// Export current session to a path (alias for export)
    pub fn export_session(&self, path: &Path) -> Result<(), SessionError> {
        self.export(path)
    }

    /// Import session from a path (alias for import)
    pub fn import_session(&mut self, path: &Path) -> Result<(), SessionError> {
        self.import(path)
    }

    /// Merge another session into the current one
    pub fn merge_session(&mut self, path: &Path) -> Result<(), SessionError> {
        let other_state = SessionState::import(path)?;

        if let Some(current_state) = &mut self.current_state {
            let existing_ids: std::collections::HashSet<_> =
                current_state.nodes.iter().map(|n| n.id.clone()).collect();

            for node in other_state.nodes {
                if !existing_ids.contains(&node.id) {
                    current_state.nodes.push(node);
                }
            }

            let existing_edges: std::collections::HashSet<_> = current_state
                .edges
                .iter()
                .map(|e| (e.from.clone(), e.to.clone()))
                .collect();

            for edge in other_state.edges {
                let edge_key = (edge.from.clone(), edge.to.clone());
                if !existing_edges.contains(&edge_key) {
                    current_state.edges.push(edge);
                }
            }

            current_state
                .node_positions
                .extend(other_state.node_positions);

            current_state.panels_open.extend(other_state.panels_open);

            self.dirty = true;
            Ok(())
        } else {
            Err(SessionError::NoState)
        }
    }
}

/// Errors that can occur during session management
#[derive(Debug, Error)]
pub enum SessionError {
    /// Session file not found
    #[error("Session not found: {0}")]
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

    /// Version mismatch
    #[error("Version mismatch: found {found}, expected {expected}")]
    VersionMismatch {
        /// The version found in the session file
        found: u32,
        /// The expected version for this build
        expected: u32,
    },

    /// No current state
    #[error("No current session state")]
    NoState,

    /// Directory error
    #[error("Directory error: {0}")]
    DirectoryError(String),
}

// Need fs for with_session_path
use std::fs;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PrimalHealthStatus;
    use std::path::PathBuf;

    #[test]
    fn test_session_state_creation() {
        let id = InstanceId::new();
        let state = SessionState::new(id.clone());

        assert_eq!(state.instance_id, id);
        assert_eq!(state.version, SessionState::VERSION);
        assert!(state.nodes.is_empty());
        assert!(state.edges.is_empty());
    }

    #[test]
    fn test_session_state_save_load_roundtrip() {
        let id = InstanceId::new();
        let state = SessionState::new(id);
        let temp = std::env::temp_dir().join("petal-session-test.ron");
        let _ = std::fs::remove_file(&temp);

        state.save(&temp).unwrap();
        assert!(temp.exists());

        let loaded = SessionState::load(&temp).unwrap();
        assert_eq!(loaded.version, state.version);
        assert_eq!(loaded.instance_id, state.instance_id);

        let _ = std::fs::remove_file(&temp);
    }

    #[test]
    fn test_session_state_load_nonexistent() {
        let result = SessionState::load(PathBuf::from("/nonexistent/session.ron").as_path());
        assert!(result.is_err());
        assert!(matches!(result, Err(SessionError::NotFound(_))));
    }

    #[test]
    fn test_session_state_export_import() {
        let id = InstanceId::new();
        let mut state = SessionState::new(id);
        state.nodes.push(PrimalInfo::new(
            "n1",
            "Node 1",
            "T1",
            "http://localhost:1",
            vec![],
            PrimalHealthStatus::Healthy,
            0,
        ));

        let temp = std::env::temp_dir().join("petal-export-test.ron");
        let _ = std::fs::remove_file(&temp);

        state.export(&temp).unwrap();
        let imported = SessionState::import(&temp).unwrap();
        assert_eq!(imported.nodes.len(), 1);
        assert_eq!(imported.nodes[0].id, "n1");

        let _ = std::fs::remove_file(&temp);
    }

    #[test]
    fn test_session_state_merge_graph_dedup() {
        let id1 = InstanceId::new();
        let id2 = InstanceId::new();

        let mut state1 = SessionState::new(id1);
        let mut state2 = SessionState::new(id2);

        state1.nodes.push(PrimalInfo::new(
            "node1",
            "Node 1",
            "Type1",
            "http://localhost:8001",
            vec![],
            PrimalHealthStatus::Healthy,
            0,
        ));
        state2.nodes.push(PrimalInfo::new(
            "node1",
            "Node 1 dup",
            "Type1",
            "http://localhost:8001",
            vec![],
            PrimalHealthStatus::Healthy,
            0,
        ));
        state2.nodes.push(PrimalInfo::new(
            "node2",
            "Node 2",
            "Type2",
            "http://localhost:8002",
            vec![],
            PrimalHealthStatus::Healthy,
            0,
        ));

        state1.merge_graph(&state2);

        assert_eq!(state1.nodes.len(), 2);
        assert!(state1.nodes.iter().any(|n| n.id == "node1"));
        assert!(state1.nodes.iter().any(|n| n.id == "node2"));
    }

    #[test]
    fn test_session_state_touch_and_age() {
        let id = InstanceId::new();
        let mut state = SessionState::new(id);
        let age_before = state.age_seconds();

        state.touch();
        let age_after = state.age_seconds();
        assert!(age_after <= age_before);
    }

    #[test]
    fn test_session_state_add_metadata() {
        let id = InstanceId::new();
        let mut state = SessionState::new(id);
        state.add_metadata("key", "value");
        assert_eq!(state.metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_session_manager_creation() {
        let id = InstanceId::new();
        let manager = SessionManager::new(&id).unwrap();

        assert!(manager.current_state().is_none());
        assert!(!manager.is_dirty());
    }

    #[test]
    fn test_session_manager_with_session_path() {
        let temp = std::env::temp_dir().join("petal-manager-test").join("sess.ron");
        std::fs::create_dir_all(temp.parent().unwrap()).unwrap();

        let manager = SessionManager::with_session_path(temp.clone()).unwrap();
        assert_eq!(manager.session_path(), temp.as_path());
        assert!(!manager.is_dirty());

        let _ = std::fs::remove_dir_all(temp.parent().unwrap());
    }

    #[test]
    fn test_session_dirty_tracking() {
        let id = InstanceId::new();
        let mut manager = SessionManager::new(&id).unwrap();

        manager.load_or_create(id).unwrap();
        assert!(manager.is_dirty());

        manager.save().unwrap();
        assert!(!manager.is_dirty());

        manager.mark_dirty();
        assert!(manager.is_dirty());
    }

    #[test]
    fn test_session_manager_save_without_state() {
        let temp = std::env::temp_dir().join("petal-save-test").join("s.ron");
        std::fs::create_dir_all(temp.parent().unwrap()).unwrap();
        let mut manager = SessionManager::with_session_path(temp.clone()).unwrap();

        let result = manager.save();
        assert!(result.is_err());
        assert!(matches!(result, Err(SessionError::NoState)));

        let _ = std::fs::remove_dir_all(temp.parent().unwrap());
    }

    #[test]
    fn test_session_manager_auto_save_disabled() {
        let temp = std::env::temp_dir().join("petal-autosave").join("s.ron");
        std::fs::create_dir_all(temp.parent().unwrap()).unwrap();
        let mut manager = SessionManager::with_session_path(temp.clone()).unwrap();
        manager.load_or_create(InstanceId::new()).unwrap();
        manager.save().unwrap();
        manager.set_auto_save(false);
        manager.mark_dirty();

        let saved = manager.auto_save_if_needed().unwrap();
        assert!(!saved);

        let _ = std::fs::remove_dir_all(temp.parent().unwrap());
    }

    #[test]
    fn test_session_manager_merge_session() {
        let id = InstanceId::new();
        let temp1 = std::env::temp_dir().join("petal-merge1").join("a.ron");
        let temp2 = std::env::temp_dir().join("petal-merge2").join("b.ron");
        std::fs::create_dir_all(temp1.parent().unwrap()).unwrap();
        std::fs::create_dir_all(temp2.parent().unwrap()).unwrap();

        let mut state2 = SessionState::new(InstanceId::new());
        state2.nodes.push(PrimalInfo::new(
            "node2",
            "N2",
            "T2",
            "http://localhost:2",
            vec![],
            PrimalHealthStatus::Healthy,
            0,
        ));
        state2.save(&temp2).unwrap();

        let mut manager = SessionManager::with_session_path(temp1.clone()).unwrap();
        manager.load_or_create(id).unwrap();
        manager.merge_session(&temp2).unwrap();

        assert_eq!(manager.current_state().unwrap().nodes.len(), 1);
        assert_eq!(manager.current_state().unwrap().nodes[0].id, "node2");

        let _ = std::fs::remove_dir_all(temp1.parent().unwrap());
        let _ = std::fs::remove_dir_all(temp2.parent().unwrap());
    }

    #[test]
    fn test_session_merge() {
        let id1 = InstanceId::new();
        let id2 = InstanceId::new();

        let mut state1 = SessionState::new(id1);
        let mut state2 = SessionState::new(id2);

        state1.nodes.push(PrimalInfo::new(
            "node1",
            "Node 1",
            "Type1",
            "http://localhost:8001",
            vec![],
            PrimalHealthStatus::Healthy,
            0,
        ));

        state2.nodes.push(PrimalInfo::new(
            "node2",
            "Node 2",
            "Type2",
            "http://localhost:8002",
            vec![],
            PrimalHealthStatus::Healthy,
            0,
        ));

        state1.merge_graph(&state2);

        assert_eq!(state1.nodes.len(), 2);
        assert!(state1.nodes.iter().any(|n| n.id == "node1"));
        assert!(state1.nodes.iter().any(|n| n.id == "node2"));
    }
}
