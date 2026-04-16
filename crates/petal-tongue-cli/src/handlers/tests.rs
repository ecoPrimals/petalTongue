// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;
use crate::commands::Commands;
use petal_tongue_core::{Instance, InstanceId, InstanceRegistry};
use std::fs;

#[tokio::test(flavor = "multi_thread")]
async fn test_run_list_empty_registry() {
    let temp = tempfile::tempdir().unwrap();
    petal_tongue_core::test_fixtures::env_test_helpers::with_env_var(
        "XDG_DATA_HOME",
        temp.path().to_str().unwrap(),
        || {
            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(run(Commands::List))
            });
            assert!(result.is_ok());
        },
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_run_status_empty_registry() {
    let temp = tempfile::tempdir().unwrap();
    let data_home = temp.path().join("data");
    fs::create_dir_all(&data_home).unwrap();
    petal_tongue_core::test_fixtures::env_test_helpers::with_env_var(
        "XDG_DATA_HOME",
        data_home.to_str().unwrap(),
        || {
            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(run(Commands::Status))
            });
            assert!(result.is_ok());
        },
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_run_gc_empty_registry() {
    let temp = tempfile::tempdir().unwrap();
    let data_home = temp.path().join("data");
    fs::create_dir_all(&data_home).unwrap();
    petal_tongue_core::test_fixtures::env_test_helpers::with_env_var(
        "XDG_DATA_HOME",
        data_home.to_str().unwrap(),
        || {
            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(run(Commands::Gc { force: false }))
            });
            assert!(result.is_ok());
        },
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_run_gc_force_with_dead_instances() {
    let temp = tempfile::tempdir().unwrap();
    let app_dir = temp.path().join("petaltongue");
    fs::create_dir_all(&app_dir).unwrap();
    let reg_path = app_dir.join("instances.ron");
    petal_tongue_core::test_fixtures::env_test_helpers::with_env_vars(
        &[("XDG_DATA_HOME", Some(temp.path().to_str().unwrap()))],
        || {
            let id = InstanceId::parse("550e8400-e29b-41d4-a716-446655440000").unwrap();
            let mut inst = Instance::new(id, Some("dead-test".to_string())).unwrap();
            inst.pid = 99_999_999;
            let mut registry = InstanceRegistry::new();
            registry.register(inst).unwrap();

            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(run(Commands::Gc { force: true }))
            });
            assert!(result.is_ok());

            let loaded = InstanceRegistry::load_from(&reg_path).unwrap();
            assert_eq!(loaded.count(), 0);
        },
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_run_gc_dry_run_with_dead_instances() {
    let temp = tempfile::tempdir().unwrap();
    petal_tongue_core::test_fixtures::env_test_helpers::with_env_vars(
        &[("XDG_DATA_HOME", Some(temp.path().to_str().unwrap()))],
        || {
            let id = InstanceId::parse("550e8400-e29b-41d4-a716-446655440001").unwrap();
            let mut inst = Instance::new(id, Some("dead".to_string())).unwrap();
            inst.pid = 99_999_999;
            let mut registry = InstanceRegistry::new();
            registry.register(inst).unwrap();

            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(run(Commands::Gc { force: false }))
            });
            assert!(result.is_ok());

            let loaded = InstanceRegistry::load().unwrap();
            assert_eq!(loaded.count(), 1);
        },
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_run_list_with_instances() {
    let temp = tempfile::tempdir().unwrap();
    petal_tongue_core::test_fixtures::env_test_helpers::with_env_vars(
        &[("XDG_DATA_HOME", Some(temp.path().to_str().unwrap()))],
        || {
            let id = InstanceId::parse("550e8400-e29b-41d4-a716-446655440002").unwrap();
            let inst = Instance::new(id, Some("alive".to_string())).unwrap();
            let mut registry = InstanceRegistry::new();
            registry.register(inst).unwrap();

            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(run(Commands::List))
            });
            assert!(result.is_ok());
        },
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_run_list_with_instance_having_window_id() {
    let temp = tempfile::tempdir().unwrap();
    petal_tongue_core::test_fixtures::env_test_helpers::with_env_vars(
        &[("XDG_DATA_HOME", Some(temp.path().to_str().unwrap()))],
        || {
            let id = InstanceId::parse("550e8400-e29b-41d4-a716-446655440004").unwrap();
            let mut inst = Instance::new(id, Some("windowed".to_string())).unwrap();
            inst.set_window_id(0x0012_3456);
            let mut registry = InstanceRegistry::new();
            registry.register(inst).unwrap();

            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(run(Commands::List))
            });
            assert!(result.is_ok());
        },
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_run_list_with_instance_no_name() {
    let temp = tempfile::tempdir().unwrap();
    petal_tongue_core::test_fixtures::env_test_helpers::with_env_vars(
        &[("XDG_DATA_HOME", Some(temp.path().to_str().unwrap()))],
        || {
            let id = InstanceId::parse("550e8400-e29b-41d4-a716-446655440008").unwrap();
            let inst = Instance::new(id, None).unwrap();
            let mut registry = InstanceRegistry::new();
            registry.register(inst).unwrap();

            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(run(Commands::List))
            });
            assert!(result.is_ok());
        },
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_run_list_with_mixed_alive_and_dead_instances() {
    let temp = tempfile::tempdir().unwrap();
    petal_tongue_core::test_fixtures::env_test_helpers::with_env_vars(
        &[("XDG_DATA_HOME", Some(temp.path().to_str().unwrap()))],
        || {
            let id1 = InstanceId::parse("550e8400-e29b-41d4-a716-446655440006").unwrap();
            let inst1 = Instance::new(id1, Some("alive".to_string())).unwrap();
            let id2 = InstanceId::parse("550e8400-e29b-41d4-a716-446655440007").unwrap();
            let mut inst2 = Instance::new(id2, Some("dead".to_string())).unwrap();
            inst2.pid = 99_999_999;
            let mut registry = InstanceRegistry::new();
            registry.register(inst1).unwrap();
            registry.register(inst2).unwrap();

            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(run(Commands::List))
            });
            assert!(result.is_ok());
        },
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_run_status_with_alive_instance_unreachable() {
    let temp = tempfile::tempdir().unwrap();
    petal_tongue_core::test_fixtures::env_test_helpers::with_env_vars(
        &[("XDG_DATA_HOME", Some(temp.path().to_str().unwrap()))],
        || {
            let id = InstanceId::parse("550e8400-e29b-41d4-a716-446655440005").unwrap();
            let inst = Instance::new(id, Some("alive-unreachable".to_string())).unwrap();
            let mut registry = InstanceRegistry::new();
            registry.register(inst).unwrap();

            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(run(Commands::Status))
            });
            assert!(result.is_ok());
        },
    );
}

#[tokio::test]
async fn test_run_show_invalid_instance_id() {
    let result = run(Commands::Show {
        instance_id: "not-a-valid-uuid".to_string(),
    })
    .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_run_raise_invalid_instance_id() {
    let result = run(Commands::Raise {
        instance_id: "bad-id".to_string(),
    })
    .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_run_ping_invalid_instance_id() {
    let result = run(Commands::Ping {
        instance_id: "invalid".to_string(),
    })
    .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_run_show_valid_uuid_no_socket() {
    let result = run(Commands::Show {
        instance_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
    })
    .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_run_raise_valid_uuid_no_socket() {
    let result = run(Commands::Raise {
        instance_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
    })
    .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_run_ping_valid_uuid_no_socket() {
    let result = run(Commands::Ping {
        instance_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
    })
    .await;
    assert!(result.is_err());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_run_status_with_dead_instance() {
    let temp = tempfile::tempdir().unwrap();
    petal_tongue_core::test_fixtures::env_test_helpers::with_env_vars(
        &[("XDG_DATA_HOME", Some(temp.path().to_str().unwrap()))],
        || {
            let id = InstanceId::parse("550e8400-e29b-41d4-a716-446655440003").unwrap();
            let mut inst = Instance::new(id, None).unwrap();
            inst.pid = 99_999_999;
            let mut registry = InstanceRegistry::new();
            registry.register(inst).unwrap();

            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(run(Commands::Status))
            });
            assert!(result.is_ok());
        },
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_run_show_empty_instance_id() {
    let temp = tempfile::tempdir().unwrap();
    petal_tongue_core::test_fixtures::env_test_helpers::with_env_var(
        "XDG_DATA_HOME",
        temp.path().to_str().unwrap(),
        || {
            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(run(Commands::Show {
                    instance_id: String::new(),
                }))
            });
            assert!(result.is_err());
        },
    );
}

#[test]
fn test_commands_match_variants() {
    drop(Commands::List);
    drop(Commands::Show {
        instance_id: "test".to_string(),
    });
    drop(Commands::Raise {
        instance_id: "test".to_string(),
    });
    drop(Commands::Ping {
        instance_id: "test".to_string(),
    });
    drop(Commands::Gc { force: false });
    drop(Commands::Status);
}
