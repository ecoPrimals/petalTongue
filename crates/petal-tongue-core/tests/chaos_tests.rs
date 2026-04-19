// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Chaos and fault injection tests for petal-tongue-core.

use petal_tongue_core::{
    Instance, InstanceId, InstanceRegistry, LoadedScenario, config_system::Config,
    config_system::ConfigError, platform_dirs,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

fn with_env_vars<F, R>(vars: &[(&str, Option<&str>)], f: F) -> R
where
    F: FnOnce() -> R,
{
    let owned: Vec<(String, Option<String>)> = vars
        .iter()
        .map(|(k, v)| ((*k).to_string(), v.map(|s| (*s).to_string())))
        .collect();
    temp_env::with_vars(owned, f)
}

fn temp_scenario_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "petaltongue_chaos_{}_{}.json",
        name,
        std::process::id()
    ))
}

fn temp_data_home() -> PathBuf {
    let p = std::env::temp_dir().join(format!("petaltongue_chaos_xdg_{}", std::process::id()));
    fs::create_dir_all(&p).unwrap();
    p
}

const fn minimal_config_toml() -> &'static str {
    r#"
[network]
web_bind = "0.0.0.0"
web_port = 3000
headless_bind = "0.0.0.0"
headless_port = 8080
workers = 4
"#
}

#[test]
fn chaos_concurrent_registry_access() {
    let dir = temp_data_home();
    with_env_vars(&[("XDG_DATA_HOME", Some(dir.to_str().unwrap()))], || {
        let reg = Arc::new(Mutex::new(InstanceRegistry::new()));
        thread::scope(|s| {
            for t in 0..8 {
                let reg = Arc::clone(&reg);
                s.spawn(move || {
                    for i in 0..20 {
                        let id = InstanceId::new();
                        let inst = Instance::new(id, Some(format!("chaos-{t}-{i}"))).unwrap();
                        reg.lock().unwrap().register(inst).unwrap();
                        let _ = reg.lock().unwrap().list();
                    }
                });
            }
        });
        assert_eq!(reg.lock().unwrap().count(), 160);
    });
}

#[test]
fn chaos_registry_lock_poison_recovery() {
    let reg = Arc::new(Mutex::new(InstanceRegistry::new()));
    let reg2 = Arc::clone(&reg);
    let handle = thread::spawn(move || {
        let _guard = reg2.lock().unwrap();
        panic!("intentional poison");
    });
    assert!(handle.join().is_err());
    let recovered = reg.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    assert_eq!(recovered.count(), 0);
}

#[test]
fn chaos_rapid_config_reload_under_load() {
    let path = temp_scenario_path("cfg");
    fs::write(&path, minimal_config_toml()).unwrap();
    let path_str = path.to_str().unwrap();
    with_env_vars(&[("PETALTONGUE_CONFIG", Some(path_str))], || {
        thread::scope(|s| {
            for _ in 0..8 {
                s.spawn(|| {
                    for _ in 0..32 {
                        let cfg = Config::from_env().unwrap();
                        assert_eq!(cfg.network.web_port, 3000);
                    }
                });
            }
        });
    });
    let _ = fs::remove_file(&path);
}

#[test]
fn chaos_config_from_env_malformed_port() {
    let path = temp_scenario_path("cfg_bad");
    fs::write(&path, minimal_config_toml()).unwrap();
    let path_str = path.to_str().unwrap();
    with_env_vars(
        &[
            ("PETALTONGUE_CONFIG", Some(path_str)),
            ("PETALTONGUE_WEB_PORT", Some("not-a-port")),
        ],
        || {
            let err = Config::from_env().unwrap_err();
            assert!(matches!(err, ConfigError::EnvError(_)));
        },
    );
    let _ = fs::remove_file(&path);
}

#[cfg(unix)]
#[test]
fn chaos_missing_xdg_home_graceful_error() {
    with_env_vars(
        &[
            ("HOME", None),
            ("XDG_DATA_HOME", None),
            ("XDG_CONFIG_HOME", None),
        ],
        || {
            assert!(platform_dirs::data_dir().is_err());
            assert!(platform_dirs::config_dir().is_err());
        },
    );
}

#[test]
fn chaos_empty_scenario_file() {
    let path = temp_scenario_path("empty");
    fs::write(&path, "").unwrap();
    assert!(LoadedScenario::from_file(&path).is_err());
    let _ = fs::remove_file(&path);
}

#[test]
fn chaos_corrupt_scenario_file() {
    let path = temp_scenario_path("bad");
    fs::write(&path, "{ not valid json").unwrap();
    assert!(LoadedScenario::from_file(&path).is_err());
    let _ = fs::remove_file(&path);
}

#[test]
fn chaos_missing_scenario_path() {
    let path = Path::new("/nonexistent/petalTongue/chaos_scenario.json");
    assert!(LoadedScenario::from_file(path).is_err());
}
