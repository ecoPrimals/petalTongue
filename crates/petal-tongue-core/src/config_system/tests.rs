// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::PathBuf;

use super::*;
use crate::test_fixtures::env_test_helpers;

#[test]
fn test_default_config() {
    let config = Config::default();
    assert_eq!(config.network.web_port, 3000);
    assert_eq!(config.network.headless_port, 8080);
    assert!((config.thresholds.health_threshold - 80.0).abs() < f32::EPSILON);
}

#[test]
fn test_config_validation() {
    let mut config = Config::default();
    assert!(config.validate().is_ok());

    config.network.web_port = 0;
    assert!(config.validate().is_err());

    config.network.web_port = 3000;
    config.thresholds.health_threshold = 150.0;
    assert!(config.validate().is_err());
}

#[test]
fn test_config_merge() {
    let base = Config::default();
    let mut override_cfg = Config::default();
    override_cfg.network.web_port = 9000;

    let merged = base.merge(override_cfg);
    assert_eq!(merged.network.web_port, 9000);
}

#[test]
fn test_network_socket_addrs() {
    let config = NetworkConfig::default();
    let web_addr = config.web_addr();
    assert_eq!(web_addr.port(), 3000);

    let headless_addr = config.headless_addr();
    assert_eq!(headless_addr.port(), 8080);
}

#[test]
fn test_config_from_file_valid() {
    let temp = std::env::temp_dir().join("petaltongue-config-test.toml");
    let contents = r#"
[network]
web_bind = "0.0.0.0"
web_port = 4000
headless_bind = "0.0.0.0"
headless_port = 9000
workers = 4

[thresholds]
health_threshold = 75.0
"#;
    std::fs::write(&temp, contents).expect("write temp config");
    let config = Config::from_file(&temp).expect("load config");
    assert_eq!(config.network.web_port, 4000);
    assert_eq!(config.network.headless_port, 9000);
    assert!((config.thresholds.health_threshold - 75.0).abs() < f32::EPSILON);
    let _ = std::fs::remove_file(&temp);
}

#[test]
fn test_config_from_file_nonexistent() {
    let result = Config::from_file("/nonexistent/path/config.toml");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ConfigError::IoError(_)));
}

#[test]
fn test_config_from_file_invalid_toml() {
    let temp = std::env::temp_dir().join("petaltongue-invalid-config.toml");
    std::fs::write(&temp, "invalid toml {{{").expect("write temp");
    let result = Config::from_file(&temp);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ConfigError::LoadError(_)));
    let _ = std::fs::remove_file(&temp);
}

#[test]
fn test_config_from_env_with_overrides() {
    let temp = std::env::temp_dir().join("petaltongue-env-override-test.toml");
    let contents = r#"
[network]
web_bind = "0.0.0.0"
web_port = 3000
headless_bind = "0.0.0.0"
headless_port = 8080
workers = 4
"#;
    std::fs::write(&temp, contents).expect("write");
    let path = temp.to_str().expect("path");
    env_test_helpers::with_env_vars(
        &[
            ("PETALTONGUE_CONFIG", Some(path)),
            ("PETALTONGUE_WEB_PORT", Some("5000")),
            ("PETALTONGUE_HEADLESS_PORT", Some("9090")),
            ("PETALTONGUE_DISCOVERY_TIMEOUT", Some("500")),
        ],
        || {
            let config = Config::from_env().expect("from_env");
            assert_eq!(config.network.web_port, 5000);
            assert_eq!(config.network.headless_port, 9090);
            assert_eq!(config.discovery.timeout.as_millis(), 500);
        },
    );
    let _ = std::fs::remove_file(&temp);
}

#[test]
fn test_config_from_env_invalid_web_port() {
    let temp = std::env::temp_dir().join("petaltongue-invalid-port-test.toml");
    let contents = r#"
[network]
web_bind = "0.0.0.0"
web_port = 3000
headless_bind = "0.0.0.0"
headless_port = 8080
workers = 4
"#;
    std::fs::write(&temp, contents).expect("write");
    let path = temp.to_str().expect("path");
    env_test_helpers::with_env_vars(
        &[
            ("PETALTONGUE_CONFIG", Some(path)),
            ("PETALTONGUE_WEB_PORT", Some("not-a-number")),
        ],
        || {
            let result = Config::from_env();
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(
                matches!(err, ConfigError::EnvError(_)),
                "expected EnvError, got {err:?}"
            );
        },
    );
    let _ = std::fs::remove_file(&temp);
}

#[test]
fn test_paths_config_runtime_dir_explicit() {
    let paths = PathsConfig {
        runtime_dir: Some(PathBuf::from("/custom/runtime")),
        data_dir: None,
        config_dir: None,
        cache_dir: None,
    };
    let dir = paths.runtime_dir().expect("runtime_dir");
    assert!(dir.to_string_lossy().contains("custom"));
}

#[test]
fn test_paths_config_data_dir_explicit() {
    let paths = PathsConfig {
        runtime_dir: None,
        data_dir: Some(PathBuf::from("/custom/data")),
        config_dir: None,
        cache_dir: None,
    };
    let dir = paths.data_dir().expect("data_dir");
    assert!(dir.to_string_lossy().contains("custom"));
}

#[test]
fn test_paths_config_merge_via_config() {
    let mut base = Config::default();
    base.paths.runtime_dir = Some(PathBuf::from("/base/runtime"));
    let mut other = Config::default();
    other.paths.runtime_dir = Some(PathBuf::from("/other/runtime"));
    other.paths.data_dir = Some(PathBuf::from("/other/data"));
    let merged = base.merge(other);
    assert!(
        merged
            .paths
            .runtime_dir
            .as_ref()
            .is_some_and(|p| p.to_string_lossy().contains("other"))
    );
    assert!(
        merged
            .paths
            .data_dir
            .as_ref()
            .is_some_and(|p| p.to_string_lossy().contains("other"))
    );
}

#[test]
fn test_discovery_config_default() {
    let d = DiscoveryConfig::default();
    assert_eq!(d.timeout.as_millis(), 200);
    assert_eq!(d.retry_attempts, 3);
    assert_eq!(d.retry_delay.as_millis(), 100);
    assert_eq!(d.cache_ttl.as_secs(), 60);
}

#[test]
fn test_thresholds_config_default() {
    let t = ThresholdsConfig::default();
    assert!((t.health_threshold - 80.0).abs() < f32::EPSILON);
    assert!((t.cpu_warning - 50.0).abs() < f32::EPSILON);
}

#[test]
fn test_performance_config_default() {
    let p = PerformanceConfig::default();
    assert_eq!(p.max_fps, 240);
    assert_eq!(p.max_width, 7680);
    assert_eq!(p.max_height, 4320);
}

#[test]
fn test_config_error_display() {
    let err = ConfigError::LoadError("parse error".to_string());
    assert!(err.to_string().contains("parse error"));

    let err = ConfigError::ValidationError("port 0".to_string());
    assert!(err.to_string().contains("port 0"));

    let err = ConfigError::EnvError("bad var".to_string());
    assert!(err.to_string().contains("bad var"));
}
