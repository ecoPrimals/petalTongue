// SPDX-License-Identifier: AGPL-3.0-or-later
//! Session state persistence (Phase 2): load, save, and manage primal topology sessions.

mod manager;
mod persistence;
mod state;
mod validation;

pub use manager::SessionManager;
pub use state::{AccessibilitySettings, SessionState, TrustSummary};

use thiserror::Error;

/// Errors that can occur during session load, save, or validation.
#[derive(Debug, Error)]
pub enum SessionError {
    /// Session file not found at the given path.
    #[error("Session not found: {0}")]
    NotFound(String),

    /// I/O error during file operations.
    #[error("IO error: {0}")]
    IoError(String),

    /// Failed to parse session file (e.g. invalid RON).
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Failed to serialize session to disk.
    #[error("Serialize error: {0}")]
    SerializeError(String),

    /// Session schema version does not match expected.
    #[error("Version mismatch: found {found}, expected {expected}")]
    VersionMismatch {
        /// Version found in the file.
        found: u32,
        /// Version expected by this build.
        expected: u32,
    },

    /// No session state is loaded (e.g. save before load).
    #[error("No current session state")]
    NoState,

    /// Error creating or accessing session directory.
    #[error("Directory error: {0}")]
    DirectoryError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PrimalHealthStatus;
    use crate::PrimalInfo;
    use crate::instance::InstanceId;
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

        state.save(&temp).expect("save session");
        assert!(temp.exists());

        let loaded = SessionState::load(&temp).expect("load session");
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

        state.export(&temp).expect("export session");
        let imported = SessionState::import(&temp).expect("import session");
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
        let manager = SessionManager::new(&id).expect("create manager");

        assert!(manager.current_state().is_none());
        assert!(!manager.is_dirty());
    }

    #[test]
    fn test_session_manager_with_session_path() {
        let temp = std::env::temp_dir()
            .join("petal-manager-test")
            .join("sess.ron");
        std::fs::create_dir_all(temp.parent().expect("parent dir")).expect("create dir");

        let manager = SessionManager::with_session_path(temp.clone()).expect("create manager");
        assert_eq!(manager.session_path(), temp.as_path());
        assert!(!manager.is_dirty());

        let _ = std::fs::remove_dir_all(temp.parent().unwrap());
    }

    #[test]
    fn test_session_dirty_tracking() {
        let id = InstanceId::new();
        let temp = std::env::temp_dir()
            .join("petal-dirty-track-test")
            .join(format!("{}.ron", id.as_str()));
        std::fs::create_dir_all(temp.parent().expect("parent dir")).expect("create dir");
        let mut manager = SessionManager::with_session_path(temp.clone()).expect("create manager");

        manager.load_or_create(id).expect("load or create");
        assert!(manager.is_dirty());

        manager.save().expect("save");
        assert!(!manager.is_dirty());

        manager.mark_dirty();
        assert!(manager.is_dirty());

        let _ = std::fs::remove_dir_all(temp.parent().unwrap());
    }

    #[test]
    fn test_session_manager_save_without_state() {
        let temp = std::env::temp_dir().join("petal-save-test").join("s.ron");
        std::fs::create_dir_all(temp.parent().expect("parent dir")).expect("create dir");
        let mut manager = SessionManager::with_session_path(temp.clone()).expect("create manager");

        let result = manager.save();
        assert!(result.is_err());
        assert!(matches!(result, Err(SessionError::NoState)));

        let _ = std::fs::remove_dir_all(temp.parent().unwrap());
    }

    #[test]
    fn test_session_manager_auto_save_disabled() {
        let temp = std::env::temp_dir().join("petal-autosave").join("s.ron");
        std::fs::create_dir_all(temp.parent().expect("parent dir")).expect("create dir");
        let mut manager = SessionManager::with_session_path(temp.clone()).expect("create manager");
        manager
            .load_or_create(InstanceId::new())
            .expect("load or create");
        manager.save().expect("save");
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
        std::fs::create_dir_all(temp1.parent().expect("parent dir")).expect("create dir");
        std::fs::create_dir_all(temp2.parent().expect("parent dir")).expect("create dir");

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
        state2.save(&temp2).expect("save state2");

        let mut manager = SessionManager::with_session_path(temp1.clone()).expect("create manager");
        manager.load_or_create(id).expect("load or create");
        manager.merge_session(&temp2).expect("merge session");

        assert_eq!(manager.current_state().expect("state").nodes.len(), 1);
        assert_eq!(manager.current_state().expect("state").nodes[0].id, "node2");

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

    #[test]
    fn test_session_manager_current_state_mut_marks_dirty() {
        let temp = std::env::temp_dir().join("petal-mut-test").join("s.ron");
        std::fs::create_dir_all(temp.parent().expect("parent")).expect("create dir");
        let mut manager = SessionManager::with_session_path(temp.clone()).expect("create manager");
        manager.load_or_create(InstanceId::new()).expect("load");
        manager.save().expect("save");
        assert!(!manager.is_dirty());
        let _ = manager.current_state_mut();
        assert!(manager.is_dirty());
        let _ = std::fs::remove_dir_all(temp.parent().expect("parent"));
    }

    #[test]
    fn test_session_manager_update_state() {
        let temp = std::env::temp_dir().join("petal-update-test").join("s.ron");
        std::fs::create_dir_all(temp.parent().expect("parent")).expect("create dir");
        let mut manager = SessionManager::with_session_path(temp.clone()).expect("create manager");
        let mut new_state = SessionState::new(InstanceId::new());
        new_state.nodes.push(PrimalInfo::new(
            "n",
            "N",
            "T",
            "http://x",
            vec![],
            PrimalHealthStatus::Healthy,
            0,
        ));
        manager.update_state(new_state);
        assert!(manager.is_dirty());
        assert_eq!(manager.current_state().expect("state").nodes.len(), 1);
        let _ = std::fs::remove_dir_all(temp.parent().expect("parent"));
    }

    #[test]
    fn test_session_manager_force_save() {
        let temp = std::env::temp_dir().join("petal-force-save").join("s.ron");
        std::fs::create_dir_all(temp.parent().expect("parent")).expect("create dir");
        let mut manager = SessionManager::with_session_path(temp.clone()).expect("create manager");
        manager.load_or_create(InstanceId::new()).expect("load");
        manager.save().expect("save");
        manager.force_save().expect("force save");
        assert!(!manager.is_dirty());
        let _ = std::fs::remove_dir_all(temp.parent().expect("parent"));
    }

    #[test]
    fn test_session_manager_set_auto_save_interval() {
        let temp = std::env::temp_dir().join("petal-interval").join("s.ron");
        std::fs::create_dir_all(temp.parent().expect("parent")).expect("create dir");
        let mut manager = SessionManager::with_session_path(temp.clone()).expect("create manager");
        manager.set_auto_save_interval(0);
        manager.set_auto_save_interval(60);
        let _ = std::fs::remove_dir_all(temp.parent().expect("parent"));
    }

    #[test]
    fn test_session_manager_has_unsaved_changes() {
        let id = InstanceId::new();
        let manager = SessionManager::new(&id).expect("create");
        assert!(!manager.has_unsaved_changes());
    }

    #[test]
    fn test_session_error_display() {
        let err = SessionError::NotFound("x".to_string());
        assert!(err.to_string().contains('x'));
        let err = SessionError::NoState;
        assert!(!err.to_string().is_empty());
    }
}
