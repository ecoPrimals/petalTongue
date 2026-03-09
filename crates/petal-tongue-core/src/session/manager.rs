// SPDX-License-Identifier: AGPL-3.0-only

use std::fs;
use std::path::{Path, PathBuf};

use super::SessionError;
use super::persistence;
use super::state::SessionState;

/// Manages session lifecycle: load, save, auto-save, and import/export.
pub struct SessionManager {
    /// Path to the session file on disk.
    pub(super) session_path: PathBuf,
    /// Loaded session state, if any.
    pub(super) current_state: Option<SessionState>,
    /// Whether auto-save is enabled.
    pub(super) auto_save_enabled: bool,
    /// Unix timestamp of last save.
    pub(super) last_save: u64,
    /// Seconds between auto-saves.
    pub(super) auto_save_interval: u64,
    /// Whether state has unsaved changes.
    pub(super) dirty: bool,
}

impl SessionManager {
    /// Creates a manager with session path derived from `instance_id`.
    pub fn new(instance_id: &crate::instance::InstanceId) -> Result<Self, SessionError> {
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

    /// Creates a manager with an explicit session path; creates parent dirs if needed.
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

    /// Returns the path to the session file.
    #[must_use]
    pub fn session_path(&self) -> &Path {
        &self.session_path
    }

    /// Loads session from disk if it exists, otherwise creates a new one.
    pub fn load_or_create(
        &mut self,
        instance_id: crate::instance::InstanceId,
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

    /// Returns a reference to the current session state, if loaded.
    #[must_use]
    pub fn current_state(&self) -> Option<&SessionState> {
        self.current_state.as_ref()
    }

    /// Returns a mutable reference to the current state; marks session dirty on access.
    pub fn current_state_mut(&mut self) -> Option<&mut SessionState> {
        if self.current_state.is_some() {
            self.dirty = true;
        }
        self.current_state.as_mut()
    }

    /// Replaces the current state with `state` and marks the session dirty.
    pub fn update_state(&mut self, state: SessionState) {
        self.current_state = Some(state);
        self.dirty = true;
    }

    /// Marks the session as having unsaved changes.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Saves the current state to disk and clears the dirty flag.
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

    /// Saves if dirty and interval elapsed; returns `true` if a save occurred.
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

    /// Saves immediately regardless of dirty flag; clears dirty.
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

    /// Enables or disables auto-save.
    pub fn set_auto_save(&mut self, enabled: bool) {
        self.auto_save_enabled = enabled;
    }

    /// Sets the auto-save interval in seconds (minimum 1).
    pub fn set_auto_save_interval(&mut self, seconds: u64) {
        self.auto_save_interval = seconds.max(1);
    }

    /// Returns whether the session has unsaved changes.
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Exports the current state to the given path.
    pub fn export(&self, path: &Path) -> Result<(), SessionError> {
        if let Some(state) = &self.current_state {
            state.export(path)
        } else {
            Err(SessionError::NoState)
        }
    }

    /// Imports state from the given path, replacing the current state.
    pub fn import(&mut self, path: &Path) -> Result<(), SessionError> {
        let state = SessionState::import(path)?;
        self.current_state = Some(state);
        self.dirty = true;
        Ok(())
    }

    /// Returns whether there are unsaved changes (alias for `is_dirty`).
    #[must_use]
    pub fn has_unsaved_changes(&self) -> bool {
        self.is_dirty()
    }

    /// Exports the session to the given path (alias for `export`).
    pub fn export_session(&self, path: &Path) -> Result<(), SessionError> {
        self.export(path)
    }

    /// Imports a session from the given path (alias for `import`).
    pub fn import_session(&mut self, path: &Path) -> Result<(), SessionError> {
        self.import(path)
    }

    /// Merges nodes, edges, positions, and panels from another session file.
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
