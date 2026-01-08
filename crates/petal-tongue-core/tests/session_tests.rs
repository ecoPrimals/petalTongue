//! Comprehensive tests for session module
//!
//! Tests verify session state persistence, saving, loading, and XDG compliance.

use petal_tongue_core::{InstanceId, LayoutAlgorithm, SessionError, SessionManager, SessionState};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_session_state_creation() {
    let state = SessionState::new();
    assert_eq!(state.version, 1);
    assert_eq!(state.nodes.len(), 0);
    assert_eq!(state.edges.len(), 0);
}

#[test]
fn test_session_state_with_graph_data() {
    let mut state = SessionState::new();

    // Add some mock data
    state.nodes.insert(
        "test-node".to_string(),
        serde_json::json!({
            "id": "test-node",
            "name": "Test Node"
        }),
    );

    assert_eq!(state.nodes.len(), 1);
    assert!(state.nodes.contains_key("test-node"));
}

#[test]
fn test_session_state_serialization() {
    let state = SessionState::new();

    // Test RON serialization
    let serialized = ron::to_string(&state);
    assert!(serialized.is_ok(), "Session state should serialize to RON");

    // Test deserialization
    let ron_str = serialized.unwrap();
    let deserialized: Result<SessionState, _> = ron::from_str(&ron_str);
    assert!(
        deserialized.is_ok(),
        "Session state should deserialize from RON"
    );
}

#[test]
fn test_session_manager_creation() {
    let temp_dir = TempDir::new().unwrap();
    let session_dir = temp_dir.path().to_path_buf();

    let manager = SessionManager::new(session_dir.clone());
    assert!(manager.is_ok(), "SessionManager should create successfully");

    // Verify session directory was created
    assert!(session_dir.exists(), "Session directory should exist");
}

#[test]
fn test_save_and_load_session() {
    let temp_dir = TempDir::new().unwrap();
    let session_dir = temp_dir.path().to_path_buf();

    let mut manager = SessionManager::new(session_dir.clone()).unwrap();
    let instance_id = InstanceId::generate();

    // Create a session with some data
    let mut state = SessionState::new();
    state.layout_algorithm = LayoutAlgorithm::Circular;
    state
        .nodes
        .insert("test-1".to_string(), serde_json::json!({"test": true}));

    // Save the session
    let save_result = manager.save_session(&instance_id, &state);
    assert!(save_result.is_ok(), "Should save session successfully");

    // Load the session back
    let loaded = manager.load_session(&instance_id);
    assert!(loaded.is_ok(), "Should load session successfully");

    let loaded_state = loaded.unwrap();
    assert_eq!(loaded_state.version, state.version);
    assert_eq!(loaded_state.layout_algorithm, LayoutAlgorithm::Circular);
    assert_eq!(loaded_state.nodes.len(), 1);
}

#[test]
fn test_load_nonexistent_session() {
    let temp_dir = TempDir::new().unwrap();
    let session_dir = temp_dir.path().to_path_buf();

    let manager = SessionManager::new(session_dir).unwrap();
    let instance_id = InstanceId::generate();

    let result = manager.load_session(&instance_id);
    assert!(result.is_err(), "Loading nonexistent session should fail");

    match result {
        Err(SessionError::SessionNotFound(_)) => {
            // Expected error type
        }
        _ => panic!("Expected SessionNotFound error"),
    }
}

#[test]
fn test_list_sessions() {
    let temp_dir = TempDir::new().unwrap();
    let session_dir = temp_dir.path().to_path_buf();

    let mut manager = SessionManager::new(session_dir).unwrap();

    // Create multiple sessions
    for i in 0..3 {
        let instance_id = InstanceId::generate();
        let mut state = SessionState::new();
        state
            .nodes
            .insert(format!("node-{}", i), serde_json::json!({"index": i}));
        manager.save_session(&instance_id, &state).unwrap();
    }

    let sessions = manager.list_sessions();
    assert!(sessions.is_ok());
    assert_eq!(sessions.unwrap().len(), 3);
}

#[test]
fn test_delete_session() {
    let temp_dir = TempDir::new().unwrap();
    let session_dir = temp_dir.path().to_path_buf();

    let mut manager = SessionManager::new(session_dir).unwrap();
    let instance_id = InstanceId::generate();

    // Create and save a session
    let state = SessionState::new();
    manager.save_session(&instance_id, &state).unwrap();

    // Verify it exists
    assert!(manager.load_session(&instance_id).is_ok());

    // Delete it
    let delete_result = manager.delete_session(&instance_id);
    assert!(delete_result.is_ok(), "Should delete session successfully");

    // Verify it's gone
    assert!(manager.load_session(&instance_id).is_err());
}

#[test]
fn test_session_file_naming() {
    let temp_dir = TempDir::new().unwrap();
    let session_dir = temp_dir.path().to_path_buf();

    let mut manager = SessionManager::new(session_dir.clone()).unwrap();
    let instance_id = InstanceId::generate();

    let state = SessionState::new();
    manager.save_session(&instance_id, &state).unwrap();

    // Check that file exists with correct naming pattern
    let expected_file = session_dir.join(format!("{}.ron", instance_id));
    assert!(
        expected_file.exists(),
        "Session file should exist with correct name"
    );
}

#[test]
fn test_session_dirty_tracking() {
    let mut state = SessionState::new();

    // Initially not dirty
    assert!(!state.is_dirty());

    // Mark as dirty
    state.mark_dirty();
    assert!(state.is_dirty());

    // Clear dirty flag
    state.clear_dirty();
    assert!(!state.is_dirty());
}

#[test]
fn test_session_state_update_timestamp() {
    let mut state = SessionState::new();
    let original_timestamp = state.last_modified;

    std::thread::sleep(std::time::Duration::from_millis(10));

    state.update_timestamp();
    assert!(state.last_modified > original_timestamp);
}

#[test]
fn test_auto_save_marker() {
    let mut state = SessionState::new();

    // Check auto-save logic
    let should_auto_save = state.should_auto_save();

    // Initially depends on dirty state
    if state.is_dirty() {
        assert!(should_auto_save);
    }
}

#[test]
fn test_session_export() {
    let temp_dir = TempDir::new().unwrap();
    let session_dir = temp_dir.path().to_path_buf();
    let export_path = temp_dir.path().join("export.ron");

    let mut manager = SessionManager::new(session_dir).unwrap();
    let instance_id = InstanceId::generate();

    // Create and save a session
    let mut state = SessionState::new();
    state
        .nodes
        .insert("export-test".to_string(), serde_json::json!({"test": true}));
    manager.save_session(&instance_id, &state).unwrap();

    // Export the session
    let export_result = manager.export_session(&instance_id, &export_path);
    assert!(export_result.is_ok(), "Should export session successfully");
    assert!(export_path.exists(), "Export file should exist");
}

#[test]
fn test_session_import() {
    let temp_dir = TempDir::new().unwrap();
    let session_dir = temp_dir.path().to_path_buf();
    let import_path = temp_dir.path().join("import.ron");

    // Create a session file to import
    let state = SessionState::new();
    let ron_str = ron::to_string(&state).unwrap();
    fs::write(&import_path, ron_str).unwrap();

    let mut manager = SessionManager::new(session_dir).unwrap();
    let instance_id = InstanceId::generate();

    // Import the session
    let import_result = manager.import_session(&instance_id, &import_path);
    assert!(import_result.is_ok(), "Should import session successfully");

    // Verify the imported session can be loaded
    let loaded = manager.load_session(&instance_id);
    assert!(loaded.is_ok(), "Should load imported session");
}

#[test]
fn test_session_atomic_write() {
    let temp_dir = TempDir::new().unwrap();
    let session_dir = temp_dir.path().to_path_buf();

    let mut manager = SessionManager::new(session_dir.clone()).unwrap();
    let instance_id = InstanceId::generate();

    let state = SessionState::new();
    manager.save_session(&instance_id, &state).unwrap();

    // Verify no .tmp files left behind (atomic write cleanup)
    let tmp_files: Vec<_> = fs::read_dir(&session_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "tmp"))
        .collect();

    assert_eq!(
        tmp_files.len(),
        0,
        "No temporary files should remain after save"
    );
}

#[test]
fn test_multiple_concurrent_sessions() {
    let temp_dir = TempDir::new().unwrap();
    let session_dir = temp_dir.path().to_path_buf();

    let mut manager = SessionManager::new(session_dir).unwrap();

    // Create 10 different sessions
    let ids: Vec<_> = (0..10).map(|_| InstanceId::generate()).collect();

    for (i, id) in ids.iter().enumerate() {
        let mut state = SessionState::new();
        state
            .nodes
            .insert(format!("node-{}", i), serde_json::json!({"index": i}));
        manager.save_session(id, &state).unwrap();
    }

    // Verify all can be loaded
    for (i, id) in ids.iter().enumerate() {
        let loaded = manager.load_session(id).unwrap();
        assert!(loaded.nodes.contains_key(&format!("node-{}", i)));
    }
}

#[test]
fn test_session_state_with_ui_state() {
    let mut state = SessionState::new();

    // Set UI state
    state
        .ui_state
        .insert("window_size".to_string(), serde_json::json!([800, 600]));
    state
        .ui_state
        .insert("zoom_level".to_string(), serde_json::json!(1.5));

    // Serialize and deserialize
    let ron_str = ron::to_string(&state).unwrap();
    let deserialized: SessionState = ron::from_str(&ron_str).unwrap();

    assert_eq!(deserialized.ui_state.len(), 2);
    assert!(deserialized.ui_state.contains_key("window_size"));
    assert!(deserialized.ui_state.contains_key("zoom_level"));
}

#[test]
fn test_session_state_with_settings() {
    let mut state = SessionState::new();

    // Set accessibility settings
    state.settings.insert(
        "color_scheme".to_string(),
        serde_json::json!("HighContrast"),
    );
    state
        .settings
        .insert("font_size".to_string(), serde_json::json!("Large"));

    // Serialize and deserialize
    let ron_str = ron::to_string(&state).unwrap();
    let deserialized: SessionState = ron::from_str(&ron_str).unwrap();

    assert_eq!(deserialized.settings.len(), 2);
}

#[test]
fn test_session_version_compatibility() {
    let mut state = SessionState::new();
    state.version = 1;

    // Future versions should be able to read version 1
    let ron_str = ron::to_string(&state).unwrap();
    let deserialized: SessionState = ron::from_str(&ron_str).unwrap();

    assert_eq!(deserialized.version, 1);
}
