// SPDX-License-Identifier: AGPL-3.0-or-later

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use crate::cli_mode::gather::{gather_detailed_status, gather_status, get_total_memory};
use crate::cli_mode::output::print_status_text;
use crate::cli_mode::status;
use crate::cli_mode::types::{
    DependencyInfo, DetailedStatus, EcoBinStatus, ModeDetails, ModeInfo, SystemInfo, SystemStatus,
    UniBinStatus,
};

#[tokio::test]
async fn test_gather_status_snapshot_err_still_succeeds() {
    let data_service = Arc::new(crate::data_service::DataService::new());
    let graph = data_service.graph();
    let g2 = Arc::clone(&graph);
    let h = std::thread::spawn(move || {
        let _guard = g2.write().unwrap();
        panic!("intentional poison for snapshot branch coverage");
    });
    let _ = h.join();

    let status = gather_status(false, &data_service).await.unwrap();
    assert_eq!(status.version, env!("CARGO_PKG_VERSION"));
}

#[tokio::test]
async fn test_gather_status_concurrent() {
    let data_service = Arc::new(crate::data_service::DataService::new());
    let status = gather_status(false, &data_service).await.unwrap();

    assert_eq!(status.version, env!("CARGO_PKG_VERSION"));
    assert_eq!(status.unibin.binary_count, 1);
    assert_eq!(status.unibin.mode_count, 5);
    assert_eq!(status.ecobin.percentage, 80);
}

#[tokio::test]
async fn test_gather_status_verbose_concurrent() {
    let data_service = Arc::new(crate::data_service::DataService::new());
    let status = gather_status(true, &data_service).await.unwrap();

    assert!(status.detailed.is_some());
    let detailed = status.detailed.unwrap();
    assert_eq!(detailed.modes.len(), 5);
    assert!(!detailed.features.is_empty());
}

#[tokio::test]
async fn test_json_output() {
    let data_service = Arc::new(crate::data_service::DataService::new());
    let status = gather_status(false, &data_service).await.unwrap();
    let json = serde_json::to_string(&status).unwrap();

    assert!(json.contains("version"));
    assert!(json.contains("unibin"));
    assert!(json.contains("ecobin"));
}

#[tokio::test]
async fn test_memory_info_nonblocking() {
    let mem = get_total_memory().await;

    #[cfg(target_os = "linux")]
    {
        assert!(mem.is_some());
    }
}

#[tokio::test]
async fn test_status_json_serialization() {
    let data_service = Arc::new(crate::data_service::DataService::new());
    let status = gather_status(false, &data_service).await.unwrap();
    let json = serde_json::to_string(&status).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.get("version").is_some());
    assert!(parsed.get("unibin").is_some());
    assert!(parsed.get("ecobin").is_some());
    assert!(parsed.get("system").is_some());
}

#[tokio::test]
async fn test_status_verbose_has_detailed() {
    let data_service = Arc::new(crate::data_service::DataService::new());
    let status = gather_status(true, &data_service).await.unwrap();
    let detailed = status.detailed.unwrap();
    assert_eq!(detailed.modes.len(), 5);
    assert!(!detailed.features.is_empty());
    assert!(detailed.dependencies.total > 0);
}

#[tokio::test]
async fn test_status_system_info() {
    let data_service = Arc::new(crate::data_service::DataService::new());
    let status = gather_status(false, &data_service).await.unwrap();
    assert_eq!(status.system.os, std::env::consts::OS);
    assert_eq!(status.system.arch, std::env::consts::ARCH);
}

#[tokio::test]
async fn test_status_unibin_compliant() {
    let data_service = Arc::new(crate::data_service::DataService::new());
    let status = gather_status(false, &data_service).await.unwrap();
    assert!(status.unibin.compliant);
    assert_eq!(status.unibin.binary_count, 1);
    assert_eq!(status.unibin.mode_count, 5);
}

#[tokio::test]
async fn test_status_text_format() {
    let data_service = Arc::new(crate::data_service::DataService::new());
    let result = status(false, "text", data_service).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_status_json_format() {
    let data_service = Arc::new(crate::data_service::DataService::new());
    let result = status(false, "json", data_service).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_status_unknown_format_defaults_to_text() {
    let data_service = Arc::new(crate::data_service::DataService::new());
    let result = status(false, "yaml", data_service).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_status_empty_format_defaults_to_text() {
    let data_service = Arc::new(crate::data_service::DataService::new());
    let result = status(false, "", data_service).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_status_verbose_text() {
    let data_service = Arc::new(crate::data_service::DataService::new());
    let result = status(true, "text", data_service).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_status_verbose_json() {
    let data_service = Arc::new(crate::data_service::DataService::new());
    let result = status(true, "json", data_service).await;
    assert!(result.is_ok());
}

#[test]
fn test_gather_detailed_status() {
    let detailed = gather_detailed_status();
    assert_eq!(detailed.modes.len(), 5);
    assert!(detailed.features.contains(&"UniBin".to_string()));
    assert!(detailed.features.contains(&"Concurrent".to_string()));
    assert_eq!(detailed.dependencies.c_deps, 1);
}

#[test]
fn test_print_status_text_no_panic() {
    let status = SystemStatus {
        version: "0.1.0".to_string(),
        mode: "status".to_string(),
        unibin: UniBinStatus {
            compliant: true,
            binary_count: 1,
            mode_count: 5,
        },
        ecobin: EcoBinStatus {
            percentage: 80,
            pure_rust_modes: 4,
            total_modes: 5,
            modes: vec![
                ModeInfo {
                    name: "tui".to_string(),
                    pure_rust: true,
                },
                ModeInfo {
                    name: "ui".to_string(),
                    pure_rust: false,
                },
            ],
        },
        system: SystemInfo {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            cpu_count: Some(8),
            memory_total: Some(16 * 1024 * 1024 * 1024),
        },
        detailed: None,
    };
    print_status_text(&status);
}

#[test]
fn test_print_status_text_with_detailed() {
    let status = SystemStatus {
        version: "0.1.0".to_string(),
        mode: "status".to_string(),
        unibin: UniBinStatus {
            compliant: true,
            binary_count: 1,
            mode_count: 5,
        },
        ecobin: EcoBinStatus {
            percentage: 80,
            pure_rust_modes: 4,
            total_modes: 5,
            modes: vec![],
        },
        system: SystemInfo {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            cpu_count: None,
            memory_total: None,
        },
        detailed: Some(DetailedStatus {
            modes: vec![ModeDetails {
                name: "tui".to_string(),
                description: "Terminal UI".to_string(),
                pure_rust: true,
                command: "petaltongue tui".to_string(),
            }],
            features: vec!["UniBin".to_string()],
            dependencies: DependencyInfo {
                total: 100,
                c_deps: 1,
                rust_deps: 99,
            },
        }),
    };
    print_status_text(&status);
}

#[tokio::test]
async fn test_ecobin_modes_correct() {
    let data_service = Arc::new(crate::data_service::DataService::new());
    let status = gather_status(false, &data_service).await.unwrap();
    let non_rust: Vec<_> = status
        .ecobin
        .modes
        .iter()
        .filter(|m| !m.pure_rust)
        .collect();
    assert_eq!(non_rust.len(), 1);
    assert_eq!(non_rust[0].name, "ui");
}

#[tokio::test]
async fn test_cpu_count_populated() {
    let data_service = Arc::new(crate::data_service::DataService::new());
    let status = gather_status(false, &data_service).await.unwrap();
    assert!(status.system.cpu_count.is_some());
    assert!(status.system.cpu_count.unwrap() > 0);
}

#[test]
fn test_mode_info_clone() {
    let mode = ModeInfo {
        name: "test".to_string(),
        pure_rust: true,
    };
    let cloned = Clone::clone(&mode);
    assert_eq!(cloned.name, "test");
    assert!(cloned.pure_rust);
}

#[test]
fn test_mode_details_clone() {
    let mode = ModeDetails {
        name: "tui".to_string(),
        description: "Terminal UI".to_string(),
        pure_rust: true,
        command: "petaltongue tui".to_string(),
    };
    let cloned = Clone::clone(&mode);
    assert_eq!(cloned.name, "tui");
    assert_eq!(cloned.command, "petaltongue tui");
}
