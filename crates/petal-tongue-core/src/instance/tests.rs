// SPDX-License-Identifier: AGPL-3.0-or-later

use super::entity::Instance;
use super::registry::InstanceRegistry;
use super::types::{InstanceError, InstanceId};

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

    instance.heartbeat();
    let second_heartbeat = instance.last_heartbeat;
    instance.heartbeat();
    let third_heartbeat = instance.last_heartbeat;

    assert!(
        second_heartbeat >= first_heartbeat,
        "Heartbeat should update timestamp (first: {first_heartbeat}, second: {second_heartbeat})"
    );
    assert!(
        third_heartbeat >= second_heartbeat,
        "Heartbeat should maintain monotonicity (second: {second_heartbeat}, third: {third_heartbeat})"
    );
}

#[test]
fn test_instance_registry() {
    let dir = tempfile::tempdir().unwrap();
    crate::test_fixtures::env_test_helpers::with_env_vars(
        &[("XDG_DATA_HOME", Some(dir.path().to_str().unwrap()))],
        || {
            let mut registry = InstanceRegistry::new();
            let id = InstanceId::new();
            let instance = Instance::new(id.clone(), Some("test".to_string())).unwrap();

            registry.register(instance).unwrap();

            assert_eq!(registry.count(), 1);
            assert!(registry.get(&id).is_some());
            assert_eq!(registry.find_by_name("test").unwrap().id, id);

            registry.unregister(&id).unwrap();
            assert_eq!(registry.count(), 0);
        },
    );
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

#[test]
fn test_instance_window_id() {
    let id = InstanceId::new();
    let mut instance = Instance::new(id, None).unwrap();

    assert_eq!(instance.window_id, None);

    instance.set_window_id(0x0012_3456);
    assert_eq!(instance.window_id, Some(0x0012_3456));

    instance.set_window_id(0x00AB_CDEF);
    assert_eq!(instance.window_id, Some(0x00AB_CDEF));
}

#[test]
fn test_instance_age_seconds() {
    let id = InstanceId::new();
    let instance = Instance::new(id, None).unwrap();

    let age = instance.age_seconds();
    assert!(age < 2, "Age should be very small at creation: {age}");
}

#[test]
fn test_registry_multiple_instances() {
    let dir = tempfile::tempdir().unwrap();
    crate::test_fixtures::env_test_helpers::with_env_vars(
        &[("XDG_DATA_HOME", Some(dir.path().to_str().unwrap()))],
        || {
            let mut registry = InstanceRegistry::new();

            let id1 = InstanceId::new();
            let id2 = InstanceId::new();
            let id3 = InstanceId::new();

            let instance1 = Instance::new(id1, Some("test1".to_string())).unwrap();
            let instance2 = Instance::new(id2, Some("test2".to_string())).unwrap();
            let instance3 = Instance::new(id3, None).unwrap();

            registry.register(instance1).unwrap();
            registry.register(instance2).unwrap();
            registry.register(instance3).unwrap();

            assert_eq!(registry.count(), 3);
            assert_eq!(registry.count_alive(), 3);

            let all_instances = registry.list();
            assert_eq!(all_instances.len(), 3);

            let alive = registry.list_alive();
            assert_eq!(alive.len(), 3);
        },
    );
}

#[test]
fn test_registry_find_by_window() {
    let dir = tempfile::tempdir().unwrap();
    crate::test_fixtures::env_test_helpers::with_env_vars(
        &[("XDG_DATA_HOME", Some(dir.path().to_str().unwrap()))],
        || {
            let mut registry = InstanceRegistry::new();
            let id = InstanceId::new();

            let instance_result = Instance::new(id, None);
            let mut instance = match instance_result {
                Ok(i) => i,
                Err(InstanceError::IoError(msg)) if msg.contains("Permission denied") => {
                    eprintln!("Skipping test_registry_find_by_window: {msg}");
                    return;
                }
                Err(e) => panic!("Unexpected error: {e:?}"),
            };

            instance.set_window_id(0x0012_3456);
            registry.register(instance).unwrap();

            assert!(registry.find_by_window(0x12_3456).is_some());
            assert!(registry.find_by_window(0x0099_9999).is_none());
        },
    );
}

#[test]
fn test_registry_find_by_pid() {
    let dir = tempfile::tempdir().unwrap();
    crate::test_fixtures::env_test_helpers::with_env_vars(
        &[("XDG_DATA_HOME", Some(dir.path().to_str().unwrap()))],
        || {
            let mut registry = InstanceRegistry::new();
            let id = InstanceId::new();
            let instance = Instance::new(id, None).unwrap();
            let pid = instance.pid;

            registry.register(instance).unwrap();

            assert!(registry.find_by_pid(pid).is_some());
            assert!(registry.find_by_pid(99_999_999).is_none());
        },
    );
}

#[test]
fn test_registry_update() {
    let dir = tempfile::tempdir().unwrap();
    crate::test_fixtures::env_test_helpers::with_env_vars(
        &[("XDG_DATA_HOME", Some(dir.path().to_str().unwrap()))],
        || {
            let mut registry = InstanceRegistry::new();
            let id = InstanceId::new();
            let mut instance = Instance::new(id.clone(), Some("original".to_string())).unwrap();

            registry.register(instance.clone()).unwrap();

            instance.add_metadata("key", "value");
            instance.set_window_id(0x12_3456);

            registry.update(instance.clone()).unwrap();

            let retrieved = registry.get(&id).unwrap();
            assert_eq!(retrieved.metadata.get("key"), Some(&"value".to_string()));
            assert_eq!(retrieved.window_id, Some(0x12_3456));
        },
    );
}

#[test]
fn test_registry_update_nonexistent() {
    let mut registry = InstanceRegistry::new();
    let id = InstanceId::new();
    let instance = Instance::new(id, None).unwrap();

    let result = registry.update(instance);
    assert!(result.is_err());
}

#[test]
fn test_registry_get_mut() {
    let dir = tempfile::tempdir().unwrap();
    crate::test_fixtures::env_test_helpers::with_env_vars(
        &[("XDG_DATA_HOME", Some(dir.path().to_str().unwrap()))],
        || {
            let mut registry = InstanceRegistry::new();
            let id = InstanceId::new();
            let instance = Instance::new(id.clone(), Some("test".to_string())).unwrap();

            registry.register(instance).unwrap();

            if let Some(inst) = registry.get_mut(&id) {
                inst.add_metadata("key", "value");
                inst.heartbeat();
            }

            let retrieved = registry.get(&id).unwrap();
            assert_eq!(retrieved.metadata.get("key"), Some(&"value".to_string()));
        },
    );
}

#[test]
fn test_registry_count_methods() {
    let dir = tempfile::tempdir().unwrap();
    crate::test_fixtures::env_test_helpers::with_env_vars(
        &[("XDG_DATA_HOME", Some(dir.path().to_str().unwrap()))],
        || {
            let mut registry = InstanceRegistry::new();

            assert_eq!(registry.count(), 0);
            assert_eq!(registry.count_alive(), 0);

            let id1 = InstanceId::new();
            let id2 = InstanceId::new();
            let instance1 = Instance::new(id1.clone(), None).unwrap();
            let instance2 = Instance::new(id2, None).unwrap();

            registry.register(instance1).unwrap();
            assert_eq!(registry.count(), 1);
            assert_eq!(registry.count_alive(), 1);

            registry.register(instance2).unwrap();
            assert_eq!(registry.count(), 2);
            assert_eq!(registry.count_alive(), 2);

            registry.unregister(&id1).unwrap();
            assert_eq!(registry.count(), 1);
            assert_eq!(registry.count_alive(), 1);
        },
    );
}

#[test]
fn test_instance_id_invalid_parse() {
    assert!(InstanceId::parse("").is_err());
    assert!(InstanceId::parse("not-a-uuid").is_err());
    assert!(InstanceId::parse("12345678").is_err());
}

#[test]
fn test_instance_id_display() {
    let id = InstanceId::new();
    let displayed = format!("{id}");
    let as_str = id.as_str();

    assert_eq!(displayed, as_str);
}

#[test]
fn test_instance_paths_created() {
    let id = InstanceId::new();
    let instance = Instance::new(id.clone(), None).unwrap();

    assert!(instance.state_path.to_string_lossy().contains(&id.as_str()));
    assert!(instance.state_path.to_string_lossy().ends_with(".ron"));

    assert!(
        instance
            .socket_path
            .to_string_lossy()
            .contains(&id.as_str())
    );
    assert!(instance.socket_path.to_string_lossy().ends_with(".sock"));
}

#[test]
fn test_instance_id_default() {
    let id = InstanceId::default();
    assert!(!id.as_str().is_empty());
    assert!(uuid::Uuid::parse_str(&id.as_str()).is_ok());
}

#[test]
fn test_instance_id_parse_valid_uuid() {
    let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
    let id = InstanceId::parse(uuid_str).unwrap();
    assert_eq!(id.as_str(), uuid_str);
}

#[test]
fn test_instance_error_display() {
    let err = InstanceError::InvalidInstanceId("bad".to_string());
    let s = format!("{err}");
    assert!(s.contains("Invalid instance ID"));
    assert!(s.contains("bad"));

    let err = InstanceError::NotFound("x".to_string());
    assert!(format!("{err}").contains("not found"));

    let err = InstanceError::IoError("e".to_string());
    assert!(format!("{err}").contains("IO error"));
}

#[test]
fn test_registry_duplicate_registration_overwrites() {
    let dir = tempfile::tempdir().unwrap();
    crate::test_fixtures::env_test_helpers::with_env_vars(
        &[("XDG_DATA_HOME", Some(dir.path().to_str().unwrap()))],
        || {
            let mut registry = InstanceRegistry::new();
            let id = InstanceId::new();
            let inst1 = Instance::new(id.clone(), Some("first".to_string())).unwrap();
            let mut inst2 = Instance::new(id.clone(), Some("second".to_string())).unwrap();
            inst2.add_metadata("k", "v");

            registry.register(inst1).unwrap();
            assert_eq!(registry.get(&id).unwrap().name.as_deref(), Some("first"));

            registry.register(inst2).unwrap();
            assert_eq!(registry.get(&id).unwrap().name.as_deref(), Some("second"));
            assert_eq!(
                registry.get(&id).unwrap().metadata.get("k"),
                Some(&"v".to_string())
            );
        },
    );
}

#[test]
fn test_registry_unregister_nonexistent() {
    let dir = tempfile::tempdir().unwrap();
    crate::test_fixtures::env_test_helpers::with_env_vars(
        &[("XDG_DATA_HOME", Some(dir.path().to_str().unwrap()))],
        || {
            let mut registry = InstanceRegistry::new();
            let id = InstanceId::new();
            let result = registry.unregister(&id);
            assert!(result.is_ok());
            assert_eq!(registry.count(), 0);
        },
    );
}

#[test]
fn test_instance_metadata_overwrite() {
    let id = InstanceId::new();
    let mut instance = Instance::new(id, None).unwrap();
    instance.add_metadata("k", "v1");
    instance.add_metadata("k", "v2");
    assert_eq!(instance.metadata.get("k"), Some(&"v2".to_string()));
}

#[test]
fn test_registry_default() {
    let registry = InstanceRegistry::default();
    assert_eq!(registry.count(), 0);
}

#[test]
fn test_registry_gc_no_dead() {
    let dir = tempfile::tempdir().unwrap();
    crate::test_fixtures::env_test_helpers::with_env_vars(
        &[("XDG_DATA_HOME", Some(dir.path().to_str().unwrap()))],
        || {
            let mut registry = InstanceRegistry::new();
            let id = InstanceId::new();
            let instance = Instance::new(id, None).unwrap();
            registry.register(instance).unwrap();
            let removed = registry.gc().unwrap();
            assert_eq!(removed, 0);
            assert_eq!(registry.count(), 1);
        },
    );
}
