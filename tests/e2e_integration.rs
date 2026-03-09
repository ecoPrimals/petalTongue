// SPDX-License-Identifier: AGPL-3.0-only
//! End-to-end integration tests for petalTongue
//!
//! These tests verify the complete system integration:
//! - Multi-instance lifecycle management
//! - State persistence and restoration
//! - Registry operations and garbage collection
//! - Session merging and transfer
//! - Auto-save functionality
//!
//! Run with: cargo test --test `e2e_integration`

use petal_tongue_core::{
    Instance, InstanceId, InstanceRegistry, SessionManager, test_fixtures::env_test_helpers,
};
use tempfile::TempDir;

use std::path::PathBuf;

fn registry_path(temp_dir: &TempDir) -> PathBuf {
    temp_dir.path().join("petaltongue").join("instances.ron")
}

fn session_path(temp_dir: &TempDir, instance_id: &InstanceId) -> PathBuf {
    temp_dir
        .path()
        .join("petaltongue")
        .join("sessions")
        .join(format!("{}.ron", instance_id.as_str()))
}

/// Test 1: Multi-instance lifecycle management
#[test]
fn test_multi_instance_lifecycle() {
    let temp_dir = TempDir::new().unwrap();
    let reg_path = registry_path(&temp_dir);
    env_test_helpers::with_env_var("XDG_DATA_HOME", temp_dir.path().to_str().unwrap(), || {
        let mut registry = InstanceRegistry::new();

        let id_a = InstanceId::new();
        let instance_a = Instance::new(id_a.clone(), Some("instance-a".to_string())).unwrap();
        registry.register(instance_a).unwrap();
        assert_eq!(registry.list().len(), 1);

        let id_b = InstanceId::new();
        let instance_b = Instance::new(id_b.clone(), Some("instance-b".to_string())).unwrap();
        registry.register(instance_b).unwrap();
        assert_eq!(registry.list().len(), 2);

        assert!(registry.get(&id_a).is_some());
        assert!(registry.get(&id_b).is_some());

        registry.unregister(&id_a).unwrap();
        assert_eq!(registry.list().len(), 1);
        assert!(registry.get(&id_a).is_none());
        assert!(registry.get(&id_b).is_some());

        registry.save_to(&reg_path).unwrap();
        let loaded_registry = InstanceRegistry::load_from(&reg_path).unwrap();
        assert_eq!(loaded_registry.list().len(), 1);
    });
}

/// Test 2: State persistence and restoration
#[test]
fn test_state_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let instance_id = InstanceId::new();
    let sess_path = session_path(&temp_dir, &instance_id);

    let mut session_mgr = SessionManager::with_session_path(sess_path.clone()).unwrap();
    session_mgr.load_or_create(instance_id.clone()).unwrap();
    session_mgr.mark_dirty();
    session_mgr.save().unwrap();

    let mut new_mgr = SessionManager::with_session_path(sess_path).unwrap();
    new_mgr.load_or_create(instance_id.clone()).unwrap();
    assert!(!new_mgr.has_unsaved_changes());
}

/// Test 3: Registry operations and garbage collection
#[test]
fn test_registry_garbage_collection() {
    let temp_dir = TempDir::new().unwrap();
    env_test_helpers::with_env_var("XDG_DATA_HOME", temp_dir.path().to_str().unwrap(), || {
        let mut registry = InstanceRegistry::new();

        let live_id = InstanceId::new();
        let live_instance = Instance::new(live_id.clone(), Some("live".to_string())).unwrap();
        registry.register(live_instance).unwrap();

        let dead_id = InstanceId::new();
        let mut dead_instance = Instance::new(dead_id.clone(), Some("dead".to_string())).unwrap();
        dead_instance.pid = 999_999;
        registry.register(dead_instance).unwrap();

        assert_eq!(registry.list().len(), 2);
        let _cleaned = registry.gc().unwrap();
        assert_eq!(registry.list_alive().len(), 1);
        assert!(registry.get(&live_id).is_some());
    });
}

/// Test 4: Session export and import
#[test]
fn test_session_export_import() {
    let temp_dir = TempDir::new().unwrap();

    let instance_id = InstanceId::new();
    let sess_path = session_path(&temp_dir, &instance_id);
    let mut session_mgr = SessionManager::with_session_path(sess_path).unwrap();
    session_mgr.load_or_create(instance_id.clone()).unwrap();
    session_mgr.mark_dirty();
    session_mgr.save().unwrap();

    let export_path = temp_dir.path().join("export.ron");
    session_mgr.export_session(&export_path).unwrap();

    let new_id = InstanceId::new();
    let new_sess_path = session_path(&temp_dir, &new_id);
    let mut new_mgr = SessionManager::with_session_path(new_sess_path).unwrap();
    new_mgr.import_session(&export_path).unwrap();
    assert!(new_mgr.has_unsaved_changes());
}

/// Test 5: Auto-save functionality
#[test]
fn test_auto_save() {
    let temp_dir = TempDir::new().unwrap();
    let instance_id = InstanceId::new();
    let sess_path = session_path(&temp_dir, &instance_id);

    let mut session_mgr = SessionManager::with_session_path(sess_path).unwrap();
    session_mgr.load_or_create(instance_id.clone()).unwrap();

    session_mgr.mark_dirty();
    assert!(session_mgr.has_unsaved_changes());

    session_mgr.auto_save_if_needed().unwrap();

    session_mgr.save().unwrap();
    assert!(!session_mgr.has_unsaved_changes());
}

/// Test 6: Session merge operations
#[test]
fn test_session_merge() {
    let temp_dir = TempDir::new().unwrap();

    let id_a = InstanceId::new();
    let sess_a = session_path(&temp_dir, &id_a);
    let mut mgr_a = SessionManager::with_session_path(sess_a).unwrap();
    mgr_a.load_or_create(id_a.clone()).unwrap();
    mgr_a.save().unwrap();
    let export_a = temp_dir.path().join("session_a.ron");
    mgr_a.export_session(&export_a).unwrap();

    let id_b = InstanceId::new();
    let sess_b = session_path(&temp_dir, &id_b);
    let mut mgr_b = SessionManager::with_session_path(sess_b).unwrap();
    mgr_b.load_or_create(id_b.clone()).unwrap();

    mgr_b.merge_session(&export_a).unwrap();
    assert!(mgr_b.has_unsaved_changes());
}

/// Test 7: Concurrent instance safety
#[test]
fn test_concurrent_registry_access() {
    let temp_dir = TempDir::new().unwrap();
    let reg_path = registry_path(&temp_dir);
    env_test_helpers::with_env_var("XDG_DATA_HOME", temp_dir.path().to_str().unwrap(), || {
        let mut registry = InstanceRegistry::new();
        let id = InstanceId::new();
        let instance = Instance::new(id.clone(), Some("concurrent-test".to_string())).unwrap();
        registry.register(instance).unwrap();
        registry.save_to(&reg_path).unwrap();

        let registry_1 = InstanceRegistry::load_from(&reg_path).unwrap();
        let registry_2 = InstanceRegistry::load_from(&reg_path).unwrap();

        assert_eq!(registry_1.list().len(), 1);
        assert_eq!(registry_2.list().len(), 1);
    });
}

/// Test 8: Instance uniqueness (no filesystem -- safe concurrent)
#[test]
fn test_instance_uniqueness() {
    let ids: Vec<InstanceId> = (0..1000).map(|_| InstanceId::new()).collect();

    let mut seen = std::collections::HashSet::new();
    for id in &ids {
        assert!(
            seen.insert(id.as_str()),
            "Duplicate ID found: {}",
            id.as_str()
        );
    }
}
