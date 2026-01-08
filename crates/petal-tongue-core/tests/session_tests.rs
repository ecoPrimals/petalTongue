//! Comprehensive tests for session module
//!
//! Tests verify session state persistence, saving, loading, and XDG compliance.

use petal_tongue_core::{
    InstanceId, LayoutAlgorithm, PrimalHealthStatus, PrimalInfo, SessionError, SessionManager,
    SessionState,
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
        1234567890,
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
    let state = manager.load_or_create(instance_id.clone());
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
    manager.load_or_create(instance_id.clone()).unwrap();

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
    manager.load_or_create(instance_id.clone()).unwrap();
    if let Some(state) = manager.current_state_mut() {
        state.nodes.push(PrimalInfo::new(
            "export-test",
            "Export Test",
            "test",
            "http://localhost:8080",
            vec![],
            PrimalHealthStatus::Healthy,
            1234567890,
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
        1234567890,
    ));

    state2.nodes.push(PrimalInfo::new(
        "node2",
        "Node 2",
        "Type2",
        "http://localhost:8002",
        vec![],
        PrimalHealthStatus::Healthy,
        1234567890,
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

    // Mark dirty
    manager.mark_dirty();

    // Set short auto-save interval
    manager.set_auto_save_interval(1);

    // Wait for interval
    std::thread::sleep(std::time::Duration::from_secs(2));

    // Auto-save should trigger
    let saved = manager.auto_save_if_needed();
    assert!(saved.is_ok());
    assert!(saved.unwrap()); // Should have saved

    // Should no longer be dirty
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
        1234567890,
    );

    // Use the properties field (modern API)
    use petal_tongue_core::property::PropertyValue;
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
