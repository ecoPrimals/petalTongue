// SPDX-License-Identifier: AGPL-3.0-or-later

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
    ///
    /// # Errors
    ///
    /// Returns an error if the data directory cannot be determined or the
    /// sessions directory cannot be created.
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
    ///
    /// # Errors
    ///
    /// Returns an error if the parent directory cannot be created.
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
    ///
    /// # Errors
    ///
    /// Returns an error if the existing session file cannot be read or parsed.
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
    pub const fn current_state(&self) -> Option<&SessionState> {
        self.current_state.as_ref()
    }

    /// Returns a mutable reference to the current state; marks session dirty on access.
    pub const fn current_state_mut(&mut self) -> Option<&mut SessionState> {
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
    pub const fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Saves the current state to disk and clears the dirty flag.
    ///
    /// # Errors
    ///
    /// Returns an error if no state is loaded, or if saving to disk fails.
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
    ///
    /// # Errors
    ///
    /// Returns an error if a save occurred when no state is loaded and saving
    /// fails.
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
    ///
    /// # Errors
    ///
    /// Returns an error if no state is loaded, or if saving to disk fails.
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
    pub const fn set_auto_save(&mut self, enabled: bool) {
        self.auto_save_enabled = enabled;
    }

    /// Sets the auto-save interval in seconds (minimum 1).
    pub fn set_auto_save_interval(&mut self, seconds: u64) {
        self.auto_save_interval = seconds.max(1);
    }

    /// Returns whether the session has unsaved changes.
    #[must_use]
    pub const fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Exports the current state to the given path.
    ///
    /// # Errors
    ///
    /// Returns an error if no state is loaded, or if saving to the given path
    /// fails.
    pub fn export(&self, path: &Path) -> Result<(), SessionError> {
        self.current_state
            .as_ref()
            .map_or(Err(SessionError::NoState), |state| state.export(path))
    }

    /// Imports state from the given path, replacing the current state.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn import(&mut self, path: &Path) -> Result<(), SessionError> {
        let state = SessionState::import(path)?;
        self.current_state = Some(state);
        self.dirty = true;
        Ok(())
    }

    /// Returns whether there are unsaved changes (alias for `is_dirty`).
    #[must_use]
    pub const fn has_unsaved_changes(&self) -> bool {
        self.is_dirty()
    }

    /// Exports the session to the given path (alias for `export`).
    ///
    /// # Errors
    ///
    /// Returns an error if no state is loaded, or if saving to the given path
    /// fails.
    pub fn export_session(&self, path: &Path) -> Result<(), SessionError> {
        self.export(path)
    }

    /// Imports a session from the given path (alias for `import`).
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn import_session(&mut self, path: &Path) -> Result<(), SessionError> {
        self.import(path)
    }

    /// Merges nodes, edges, positions, and panels from another session file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed, or if no
    /// current state is loaded.
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test code")]

    use super::*;
    use crate::instance::InstanceId;
    use crate::session::state::SessionState;

    fn session_path_in(dir: &tempfile::TempDir, name: &str) -> PathBuf {
        dir.path().join("nested").join("sessions").join(name)
    }

    #[test]
    fn with_session_path_creates_parent_directories() {
        let dir = tempfile::tempdir().unwrap();
        let path = session_path_in(&dir, "session.ron");
        assert!(!path.parent().unwrap().exists());

        SessionManager::with_session_path(path.clone()).unwrap();

        assert!(path.parent().unwrap().exists());
    }

    #[test]
    fn session_path_returns_correct_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = session_path_in(&dir, "sess.ron");
        let manager = SessionManager::with_session_path(path.clone()).unwrap();

        assert_eq!(manager.session_path(), path.as_path());
    }

    #[test]
    fn load_or_create_creates_new_state_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = session_path_in(&dir, "new.ron");
        let mut manager = SessionManager::with_session_path(path.clone()).unwrap();
        let instance_id = InstanceId::new();

        assert!(manager.current_state().is_none());
        assert!(!path.exists());

        manager.load_or_create(instance_id.clone()).unwrap();

        assert!(manager.current_state().is_some());
        assert!(manager.is_dirty());
        assert_eq!(manager.current_state().unwrap().instance_id, instance_id);
    }

    #[test]
    fn load_or_create_save_reload_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = session_path_in(&dir, "roundtrip.ron");
        let instance_id = InstanceId::new();

        let mut manager = SessionManager::with_session_path(path.clone()).unwrap();
        manager.load_or_create(instance_id.clone()).unwrap();
        manager
            .current_state_mut()
            .unwrap()
            .add_metadata("marker", "saved");
        manager.save().unwrap();
        assert!(path.exists());

        let mut manager2 = SessionManager::with_session_path(path).unwrap();
        manager2.load_or_create(instance_id).unwrap();

        assert!(!manager2.is_dirty());
        assert_eq!(
            manager2.current_state().unwrap().metadata.get("marker"),
            Some(&"saved".to_owned())
        );
    }

    #[test]
    fn current_state_none_until_loaded() {
        let dir = tempfile::tempdir().unwrap();
        let path = session_path_in(&dir, "state.ron");
        let mut manager = SessionManager::with_session_path(path).unwrap();

        assert!(manager.current_state().is_none());

        manager.load_or_create(InstanceId::new()).unwrap();

        assert!(manager.current_state().is_some());
    }

    #[test]
    fn current_state_mut_marks_dirty() {
        let dir = tempfile::tempdir().unwrap();
        let path = session_path_in(&dir, "mut.ron");
        let mut manager = SessionManager::with_session_path(path).unwrap();
        manager.load_or_create(InstanceId::new()).unwrap();
        manager.save().unwrap();
        assert!(!manager.is_dirty());

        let _ = manager.current_state_mut();

        assert!(manager.is_dirty());
    }

    #[test]
    fn update_state_replaces_and_marks_dirty() {
        let dir = tempfile::tempdir().unwrap();
        let path = session_path_in(&dir, "update.ron");
        let mut manager = SessionManager::with_session_path(path).unwrap();
        manager.load_or_create(InstanceId::new()).unwrap();
        manager.save().unwrap();

        let mut replacement = SessionState::new(InstanceId::new());
        replacement.add_metadata("replaced", "yes");
        manager.update_state(replacement);

        assert!(manager.is_dirty());
        assert_eq!(
            manager.current_state().unwrap().metadata.get("replaced"),
            Some(&"yes".to_owned())
        );
    }

    #[test]
    fn mark_dirty_and_unsaved_change_aliases() {
        let dir = tempfile::tempdir().unwrap();
        let path = session_path_in(&dir, "dirty.ron");
        let mut manager = SessionManager::with_session_path(path).unwrap();
        manager.load_or_create(InstanceId::new()).unwrap();
        manager.save().unwrap();
        assert!(!manager.is_dirty());
        assert!(!manager.has_unsaved_changes());

        manager.mark_dirty();

        assert!(manager.is_dirty());
        assert!(manager.has_unsaved_changes());
    }

    #[test]
    fn save_clears_dirty_and_fails_without_state() {
        let dir = tempfile::tempdir().unwrap();
        let path = session_path_in(&dir, "save.ron");
        let mut manager = SessionManager::with_session_path(path.clone()).unwrap();

        let no_state = manager.save();
        assert!(matches!(no_state, Err(SessionError::NoState)));

        manager.load_or_create(InstanceId::new()).unwrap();
        assert!(manager.is_dirty());
        manager.save().unwrap();
        assert!(!manager.is_dirty());
        assert!(path.exists());
    }

    #[test]
    fn auto_save_if_needed_skips_when_clean() {
        let dir = tempfile::tempdir().unwrap();
        let path = session_path_in(&dir, "autosave.ron");
        let mut manager = SessionManager::with_session_path(path).unwrap();
        manager.load_or_create(InstanceId::new()).unwrap();
        manager.save().unwrap();
        assert!(!manager.is_dirty());

        let saved = manager.auto_save_if_needed().unwrap();

        assert!(!saved);
        assert!(!manager.is_dirty());
    }

    #[test]
    fn set_auto_save_interval_enforces_minimum_one() {
        let dir = tempfile::tempdir().unwrap();
        let path = session_path_in(&dir, "interval.ron");
        let mut manager = SessionManager::with_session_path(path).unwrap();
        manager.load_or_create(InstanceId::new()).unwrap();
        manager.save().unwrap();
        manager.mark_dirty();
        manager.set_auto_save_interval(0);

        let saved = manager.auto_save_if_needed().unwrap();

        assert!(
            !saved,
            "interval of 0 should clamp to 1 second, not save immediately"
        );
    }

    #[test]
    fn export_import_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let session_path = session_path_in(&dir, "main.ron");
        let export_path = dir.path().join("export.ron");
        let mut manager = SessionManager::with_session_path(session_path).unwrap();
        let instance_id = InstanceId::new();
        manager.load_or_create(instance_id.clone()).unwrap();
        manager
            .current_state_mut()
            .unwrap()
            .add_metadata("exported", "value");
        manager.save().unwrap();

        manager.export(&export_path).unwrap();
        assert!(export_path.exists());

        let mut manager2 = SessionManager::with_session_path(dir.path().join("other.ron")).unwrap();
        manager2.import(&export_path).unwrap();

        assert!(manager2.is_dirty());
        assert_eq!(manager2.current_state().unwrap().instance_id, instance_id);
        assert_eq!(
            manager2.current_state().unwrap().metadata.get("exported"),
            Some(&"value".to_owned())
        );
    }

    #[test]
    fn export_without_state_returns_no_state() {
        let dir = tempfile::tempdir().unwrap();
        let path = session_path_in(&dir, "no-export.ron");
        let export_path = dir.path().join("out.ron");
        let manager = SessionManager::with_session_path(path).unwrap();

        let result = manager.export(&export_path);

        assert!(matches!(result, Err(SessionError::NoState)));
    }
}
