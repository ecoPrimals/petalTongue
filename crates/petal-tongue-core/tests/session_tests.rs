// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Comprehensive tests for session module
//!
//! Tests verify session state persistence, saving, loading, and XDG compliance.

#![allow(clippy::float_cmp)]

use petal_tongue_core::{
    InstanceId, LayoutAlgorithm, PrimalHealthStatus, PrimalInfo, SessionError, SessionManager,
    SessionState, property::PropertyValue,
};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_session_state_creation() {
    let instance_id = InstanceId::new();
    let state = SessionState::new(instance_id.clone());

    assert_eq!(state.instance_id, instance_id);
    assert_eq!(state.version, SessionState::VERSION);
    assert!(state.nodes.is_empty());
    assert!(state.edges.is_empty());
}

#[test]
fn test_session_state_with_graph_data() {
    let instance_id = InstanceId::new();
    let mut state = SessionState::new(instance_id);

    // Add some mock data using the modern PrimalInfo::new() API
    state.nodes.push(PrimalInfo::new(
        "test-node",
        "Test Node",
        "test",
        "http://localhost:8080",
        vec![],
        PrimalHealthStatus::Healthy,
        1_234_567_890,
    ));

    assert_eq!(state.nodes.len(), 1);
    assert_eq!(state.nodes[0].id, "test-node");
}

#[test]
fn test_session_state_serialization() {
    let instance_id = InstanceId::new();
    let state = SessionState::new(instance_id);

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
    let instance_id = InstanceId::new();
    let manager = SessionManager::new(&instance_id);
    assert!(manager.is_ok(), "SessionManager should create successfully");

    let manager = manager.unwrap();
    assert!(manager.current_state().is_none());
    assert!(!manager.is_dirty());
}

#[test]
fn test_load_or_create_session() {
    let instance_id = InstanceId::new();
    let mut manager = SessionManager::new(&instance_id).unwrap();

    // Load or create should succeed
    let state = manager.load_or_create(instance_id);
    assert!(state.is_ok(), "Should load or create session successfully");

    // Manager should now have state
    assert!(manager.current_state().is_some());
    assert!(manager.is_dirty()); // New session is dirty
}

#[test]
fn test_save_session() {
    let instance_id = InstanceId::new();
    let mut manager = SessionManager::new(&instance_id).unwrap();

    // Create session
    manager.load_or_create(instance_id).unwrap();

    // Save should succeed
    let save_result = manager.save();
    assert!(save_result.is_ok(), "Should save session successfully");

    // Should no longer be dirty
    assert!(!manager.is_dirty());
}

#[test]
fn test_session_dirty_tracking() {
    let instance_id = InstanceId::new();
    let mut manager = SessionManager::new(&instance_id).unwrap();

    manager.load_or_create(instance_id).unwrap();
    assert!(manager.is_dirty()); // New session is dirty

    manager.save().unwrap();
    assert!(!manager.is_dirty()); // Clean after save

    manager.mark_dirty();
    assert!(manager.is_dirty()); // Dirty after mark
}

#[test]
fn test_session_state_update() {
    let instance_id = InstanceId::new();
    let mut manager = SessionManager::new(&instance_id).unwrap();

    manager.load_or_create(instance_id.clone()).unwrap();

    // Modify state
    if let Some(state) = manager.current_state_mut() {
        state.layout = LayoutAlgorithm::Circular;
        state.zoom_level = 2.0;
    }

    // Should be dirty
    assert!(manager.is_dirty());

    // Save and reload to verify persistence
    manager.save().unwrap();

    let mut manager2 = SessionManager::new(&instance_id).unwrap();
    let reloaded = manager2.load_or_create(instance_id).unwrap();

    assert_eq!(reloaded.layout, LayoutAlgorithm::Circular);
    assert_eq!(reloaded.zoom_level, 2.0);
}

#[test]
fn test_session_export_import() {
    let temp_dir = TempDir::new().unwrap();
    let export_path = temp_dir.path().join("export.ron");

    let instance_id = InstanceId::new();
    let mut manager = SessionManager::new(&instance_id).unwrap();

    // Create session with data
    manager.load_or_create(instance_id).unwrap();
    if let Some(state) = manager.current_state_mut() {
        state.nodes.push(PrimalInfo::new(
            "export-test",
            "Export Test",
            "test",
            "http://localhost:8080",
            vec![],
            PrimalHealthStatus::Healthy,
            1_234_567_890,
        ));
    }
    manager.save().unwrap();

    // Export the session
    let export_result = manager.export(&export_path);
    assert!(export_result.is_ok(), "Should export session successfully");
    assert!(export_path.exists(), "Export file should exist");

    // Import into a new manager
    let instance_id2 = InstanceId::new();
    let mut manager2 = SessionManager::new(&instance_id2).unwrap();
    let import_result = manager2.import(&export_path);
    assert!(import_result.is_ok(), "Should import session successfully");

    // Verify imported data
    let imported_state = manager2.current_state().unwrap();
    assert_eq!(imported_state.nodes.len(), 1);
    assert_eq!(imported_state.nodes[0].id, "export-test");
}

#[test]
fn test_session_merge() {
    let id1 = InstanceId::new();
    let id2 = InstanceId::new();

    let mut state1 = SessionState::new(id1);
    let mut state2 = SessionState::new(id2);

    // Add different nodes to each
    state1.nodes.push(PrimalInfo::new(
        "node1",
        "Node 1",
        "Type1",
        "http://localhost:8001",
        vec![],
        PrimalHealthStatus::Healthy,
        1_234_567_890,
    ));

    state2.nodes.push(PrimalInfo::new(
        "node2",
        "Node 2",
        "Type2",
        "http://localhost:8002",
        vec![],
        PrimalHealthStatus::Healthy,
        1_234_567_890,
    ));

    // Merge state2 into state1
    state1.merge_graph(&state2);

    assert_eq!(state1.nodes.len(), 2);
    assert!(state1.nodes.iter().any(|n| n.id == "node1"));
    assert!(state1.nodes.iter().any(|n| n.id == "node2"));
}

#[test]
fn test_session_auto_save() {
    let instance_id = InstanceId::new();
    let mut manager = SessionManager::new(&instance_id).unwrap();

    manager.load_or_create(instance_id).unwrap();
    manager.save().unwrap(); // Save to clear dirty flag

    // Auto-save should not trigger if not dirty
    let not_saved = manager.auto_save_if_needed();
    assert!(not_saved.is_ok());
    assert!(!not_saved.unwrap()); // Should not have saved (not dirty)

    // Mark dirty
    manager.mark_dirty();
    assert!(manager.is_dirty());

    // Auto-save mechanism works (tested indirectly via force_save)
    // Note: Deterministic testing of time-based auto-save would require sleep,
    // which we avoid. Instead, we test the dirty flag logic.
    manager.force_save().unwrap();
    assert!(!manager.is_dirty());
}

#[test]
fn test_session_force_save() {
    let instance_id = InstanceId::new();
    let mut manager = SessionManager::new(&instance_id).unwrap();

    manager.load_or_create(instance_id).unwrap();

    // Force save even if not dirty
    let save_result = manager.force_save();
    assert!(save_result.is_ok());
}

#[test]
fn test_session_age() {
    let instance_id = InstanceId::new();
    let state = SessionState::new(instance_id);

    let age = state.age_seconds();
    assert!(age < 5, "New session should be very young");
}

#[test]
fn test_session_with_properties() {
    let instance_id = InstanceId::new();
    let mut state = SessionState::new(instance_id);

    // Add modern PrimalInfo with properties
    let mut primal = PrimalInfo::new(
        "modern-primal",
        "Modern Primal",
        "compute",
        "http://localhost:9000",
        vec!["capability1".to_string()],
        PrimalHealthStatus::Healthy,
        1_234_567_890,
    );

    primal
        .properties
        .insert("trust_level".to_string(), PropertyValue::Number(2.0));
    primal.properties.insert(
        "family_id".to_string(),
        PropertyValue::String("family-123".to_string()),
    );

    state.nodes.push(primal);

    // Serialize and deserialize
    let ron_str = ron::to_string(&state).unwrap();
    let deserialized: SessionState = ron::from_str(&ron_str).unwrap();

    assert_eq!(deserialized.nodes.len(), 1);
    assert_eq!(deserialized.nodes[0].id, "modern-primal");

    // Verify properties preserved
    let props = &deserialized.nodes[0].properties;
    assert!(props.contains_key("trust_level"));
    assert!(props.contains_key("family_id"));
}

#[test]
fn test_session_accessibility_settings() {
    let instance_id = InstanceId::new();
    let mut state = SessionState::new(instance_id);

    // Modify accessibility settings
    state.accessibility.color_scheme = "high_contrast".to_string();
    state.accessibility.font_size = 1.5;
    state.accessibility.audio_enabled = true;

    // Serialize and deserialize
    let ron_str = ron::to_string(&state).unwrap();
    let deserialized: SessionState = ron::from_str(&ron_str).unwrap();

    assert_eq!(deserialized.accessibility.color_scheme, "high_contrast");
    assert_eq!(deserialized.accessibility.font_size, 1.5);
    assert!(deserialized.accessibility.audio_enabled);
}

#[test]
fn test_session_metadata() {
    let instance_id = InstanceId::new();
    let mut state = SessionState::new(instance_id);

    // Add custom metadata
    state.add_metadata("environment", "test");
    state.add_metadata("version", "0.3.3");

    assert_eq!(state.metadata.get("environment"), Some(&"test".to_string()));
    assert_eq!(state.metadata.get("version"), Some(&"0.3.3".to_string()));

    // Serialize and deserialize
    let ron_str = ron::to_string(&state).unwrap();
    let deserialized: SessionState = ron::from_str(&ron_str).unwrap();

    assert_eq!(deserialized.metadata.len(), 2);
}

#[test]
fn test_session_path() {
    let instance_id = InstanceId::new();
    let manager = SessionManager::new(&instance_id).unwrap();

    let path = manager.session_path();
    assert!(path.to_string_lossy().contains(&instance_id.as_str()));
    assert!(path.to_string_lossy().ends_with(".ron"));
}

#[test]
fn test_no_state_error() {
    let instance_id = InstanceId::new();
    let mut manager = SessionManager::new(&instance_id).unwrap();

    // Try to save without loading/creating state first
    let result = manager.save();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), SessionError::NoState));
}

#[test]
fn test_session_state_version() {
    let instance_id = InstanceId::new();
    let state = SessionState::new(instance_id);

    // Version should be consistent
    assert_eq!(state.version, SessionState::VERSION);
    assert_eq!(SessionState::VERSION, 1); // Current version
}

#[test]
fn test_session_empty_state() {
    let instance_id = InstanceId::new();
    let state = SessionState::new(instance_id);

    // New state should be empty
    assert!(state.nodes.is_empty());
    assert!(state.edges.is_empty());
    assert!(state.node_positions.is_empty());
    assert!(state.panels_open.is_empty());
    assert!(state.metadata.is_empty());
    assert_eq!(state.name, None);
    assert_eq!(state.window_position, None);
    assert_eq!(state.window_size, None);
    assert_eq!(state.zoom_level, 1.0);
    assert_eq!(state.pan_offset, (0.0, 0.0));
    assert!(state.auto_refresh);
    assert_eq!(state.refresh_interval, 5.0);
}

#[test]
fn test_session_ui_state() {
    let instance_id = InstanceId::new();
    let mut state = SessionState::new(instance_id);

    // Modify UI state
    state.window_position = Some((100, 200));
    state.window_size = Some((1920, 1080));
    state.zoom_level = 1.5;
    state.pan_offset = (50.0, -30.0);
    state.panels_open.insert("left_panel".to_string());
    state.panels_open.insert("right_panel".to_string());

    // Serialize and deserialize
    let ron_str = ron::to_string(&state).unwrap();
    let deserialized: SessionState = ron::from_str(&ron_str).unwrap();

    assert_eq!(deserialized.window_position, Some((100, 200)));
    assert_eq!(deserialized.window_size, Some((1920, 1080)));
    assert_eq!(deserialized.zoom_level, 1.5);
    assert_eq!(deserialized.pan_offset, (50.0, -30.0));
    assert_eq!(deserialized.panels_open.len(), 2);
    assert!(deserialized.panels_open.contains("left_panel"));
    assert!(deserialized.panels_open.contains("right_panel"));
}

#[test]
fn test_session_node_positions() {
    let instance_id = InstanceId::new();
    let mut state = SessionState::new(instance_id);

    // Add node positions
    state
        .node_positions
        .insert("node1".to_string(), (100.0, 200.0));
    state
        .node_positions
        .insert("node2".to_string(), (-50.0, 75.5));

    // Serialize and deserialize
    let ron_str = ron::to_string(&state).unwrap();
    let deserialized: SessionState = ron::from_str(&ron_str).unwrap();

    assert_eq!(deserialized.node_positions.len(), 2);
    assert_eq!(
        deserialized.node_positions.get("node1"),
        Some(&(100.0, 200.0))
    );
    assert_eq!(
        deserialized.node_positions.get("node2"),
        Some(&(-50.0, 75.5))
    );
}

#[test]
fn test_session_layout_algorithms() {
    let instance_id = InstanceId::new();
    let mut state = SessionState::new(instance_id);

    // Test different layout algorithms
    state.layout = LayoutAlgorithm::ForceDirected;
    let ron1 = ron::to_string(&state).unwrap();
    let des1: SessionState = ron::from_str(&ron1).unwrap();
    assert_eq!(des1.layout, LayoutAlgorithm::ForceDirected);

    state.layout = LayoutAlgorithm::Circular;
    let ron2 = ron::to_string(&state).unwrap();
    let des2: SessionState = ron::from_str(&ron2).unwrap();
    assert_eq!(des2.layout, LayoutAlgorithm::Circular);

    state.layout = LayoutAlgorithm::Hierarchical;
    let ron3 = ron::to_string(&state).unwrap();
    let des3: SessionState = ron::from_str(&ron3).unwrap();
    assert_eq!(des3.layout, LayoutAlgorithm::Hierarchical);
}

#[test]
fn test_session_merge_deduplication() {
    let id1 = InstanceId::new();
    let id2 = InstanceId::new();

    let mut state1 = SessionState::new(id1);
    let mut state2 = SessionState::new(id2);

    // Add same node to both states
    let same_primal = PrimalInfo::new(
        "duplicate",
        "Duplicate",
        "test",
        "http://localhost:8000",
        vec![],
        PrimalHealthStatus::Healthy,
        1_234_567_890,
    );

    state1.nodes.push(same_primal.clone());
    state2.nodes.push(same_primal);

    // Add unique node to state2
    state2.nodes.push(PrimalInfo::new(
        "unique",
        "Unique",
        "test",
        "http://localhost:8001",
        vec![],
        PrimalHealthStatus::Healthy,
        1_234_567_890,
    ));

    // Merge should deduplicate
    state1.merge_graph(&state2);

    // Should have 2 nodes (1 duplicate removed)
    assert_eq!(state1.nodes.len(), 2);
    assert!(state1.nodes.iter().any(|n| n.id == "duplicate"));
    assert!(state1.nodes.iter().any(|n| n.id == "unique"));
}

#[test]
fn test_session_current_state_mut() {
    let instance_id = InstanceId::new();
    let mut manager = SessionManager::new(&instance_id).unwrap();

    // Should be None before load/create
    assert!(manager.current_state_mut().is_none());

    manager.load_or_create(instance_id).unwrap();

    // Should be Some after load/create
    assert!(manager.current_state_mut().is_some());

    // Modify through mutable reference
    if let Some(state) = manager.current_state_mut() {
        state.zoom_level = 3.0;
        state.add_metadata("test", "value");
    }

    // Verify changes
    let state = manager.current_state().unwrap();
    assert_eq!(state.zoom_level, 3.0);
    assert_eq!(state.metadata.get("test"), Some(&"value".to_string()));
}

#[test]
fn test_session_refresh_settings() {
    let instance_id = InstanceId::new();
    let mut state = SessionState::new(instance_id);

    // Modify refresh settings
    state.auto_refresh = false;
    state.refresh_interval = 10.0;

    // Serialize and deserialize
    let ron_str = ron::to_string(&state).unwrap();
    let deserialized: SessionState = ron::from_str(&ron_str).unwrap();

    assert!(!deserialized.auto_refresh);
    assert_eq!(deserialized.refresh_interval, 10.0);
}

#[test]
fn test_session_active_scenario() {
    let instance_id = InstanceId::new();
    let mut state = SessionState::new(instance_id);

    // Set active scenario
    state.active_scenario = Some("showcase_demo".to_string());

    // Serialize and deserialize
    let ron_str = ron::to_string(&state).unwrap();
    let deserialized: SessionState = ron::from_str(&ron_str).unwrap();

    assert_eq!(
        deserialized.active_scenario,
        Some("showcase_demo".to_string())
    );
}

#[test]
fn test_session_accessibility_defaults() {
    let instance_id = InstanceId::new();
    let state = SessionState::new(instance_id);

    // Verify default accessibility settings
    assert_eq!(state.accessibility.color_scheme, "standard");
    assert_eq!(state.accessibility.font_size, 1.0);
    assert!(!state.accessibility.audio_enabled);
    assert_eq!(state.accessibility.audio_volume, 0.7);
}

#[test]
fn test_session_manager_multiple_saves() {
    let instance_id = InstanceId::new();
    let mut manager = SessionManager::new(&instance_id).unwrap();

    manager.load_or_create(instance_id.clone()).unwrap();

    // Save multiple times
    manager.save().unwrap();
    manager.mark_dirty();
    manager.save().unwrap();
    manager.mark_dirty();
    manager.save().unwrap();

    // Should be able to load
    let mut manager2 = SessionManager::new(&instance_id).unwrap();
    let loaded = manager2.load_or_create(instance_id).unwrap();

    assert!(loaded.version > 0);
}

#[test]
fn test_export_nonexistent_path() {
    let instance_id = InstanceId::new();
    let mut manager = SessionManager::new(&instance_id).unwrap();

    manager.load_or_create(instance_id).unwrap();

    // Export to a path in a non-existent directory should create it
    let temp_dir = TempDir::new().unwrap();
    let nested_path = temp_dir.path().join("nested").join("export.ron");

    let result = manager.export(&nested_path);
    assert!(result.is_ok());
    assert!(nested_path.exists());
}

#[test]
fn test_import_invalid_file() {
    let instance_id = InstanceId::new();
    let mut manager = SessionManager::new(&instance_id).unwrap();

    let temp_dir = TempDir::new().unwrap();
    let invalid_path = temp_dir.path().join("invalid.ron");

    // Write invalid RON
    fs::write(&invalid_path, "this is not valid RON").unwrap();

    let result = manager.import(&invalid_path);
    assert!(result.is_err());
}
