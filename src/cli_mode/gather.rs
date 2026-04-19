// SPDX-License-Identifier: AGPL-3.0-or-later
//! Concurrent status gathering (system info, optional detailed block).

use crate::error::AppError;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::types::{
    DependencyInfo, DetailedStatus, EcoBinStatus, ModeDetails, ModeInfo, SystemInfo, SystemStatus,
    UniBinStatus,
};

/// Gather system status concurrently
pub async fn gather_status(
    verbose: bool,
    data_service: &Arc<crate::data_service::DataService>,
) -> Result<SystemStatus, AppError> {
    let status = Arc::new(RwLock::new(SystemStatus {
        version: env!("CARGO_PKG_VERSION").to_string(),
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
                    name: "ui".to_string(),
                    pure_rust: false,
                },
                ModeInfo {
                    name: "tui".to_string(),
                    pure_rust: true,
                },
                ModeInfo {
                    name: "web".to_string(),
                    pure_rust: true,
                },
                ModeInfo {
                    name: "headless".to_string(),
                    pure_rust: true,
                },
                ModeInfo {
                    name: "status".to_string(),
                    pure_rust: true,
                },
            ],
        },
        system: SystemInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            cpu_count: None,
            memory_total: None,
        },
        detailed: None,
    }));

    let status_clone = Arc::clone(&status);
    let system_info_task = tokio::spawn(async move {
        let cpu_count = std::thread::available_parallelism()
            .map(std::num::NonZero::get)
            .unwrap_or(1);

        let memory_total = get_total_memory().await;

        let mut status_guard = status_clone.write().await;
        status_guard.system.cpu_count = Some(cpu_count);
        status_guard.system.memory_total = memory_total;
    });

    tracing::info!("✅ Using shared DataService");
    if let Ok(snapshot) = data_service.snapshot().await {
        tracing::info!(
            "📊 DataService has {} primals, {} edges",
            snapshot.primals.len(),
            snapshot.edges.len()
        );
    }

    if verbose {
        let status_clone = Arc::clone(&status);
        let detailed_task = tokio::spawn(async move {
            let detailed = gather_detailed_status();
            let mut status_guard = status_clone.write().await;
            status_guard.detailed = Some(detailed);
        });

        tokio::try_join!(system_info_task, detailed_task)
            .map_err(|e| AppError::Other(format!("Failed to gather system info: {e}")))?;
    } else {
        system_info_task
            .await
            .map_err(|e| AppError::Other(format!("Failed to gather system info: {e}")))?;
    }

    let final_status = status.read().await.clone();
    Ok(final_status)
}

/// Get total system memory (non-blocking)
pub async fn get_total_memory() -> Option<u64> {
    tokio::task::spawn_blocking(|| {
        #[cfg(target_os = "linux")]
        {
            std::fs::read_to_string("/proc/meminfo")
                .ok()
                .and_then(|content| {
                    content
                        .lines()
                        .find(|line| line.starts_with("MemTotal:"))
                        .and_then(|line| {
                            line.split_whitespace()
                                .nth(1)
                                .and_then(|s| s.parse::<u64>().ok())
                                .map(|kb| kb * 1024)
                        })
                })
        }
        #[cfg(not(target_os = "linux"))]
        {
            None
        }
    })
    .await
    .ok()
    .flatten()
}

pub fn gather_detailed_status() -> DetailedStatus {
    DetailedStatus {
        modes: vec![
            ModeDetails {
                name: "ui".to_string(),
                description: "Desktop display (egui)".to_string(),
                pure_rust: false,
                command: "petaltongue ui".to_string(),
            },
            ModeDetails {
                name: "tui".to_string(),
                description: "Terminal UI (ratatui)".to_string(),
                pure_rust: true,
                command: "petaltongue tui".to_string(),
            },
            ModeDetails {
                name: "web".to_string(),
                description: "Web server (axum)".to_string(),
                pure_rust: true,
                command: "petaltongue web".to_string(),
            },
            ModeDetails {
                name: "headless".to_string(),
                description: "API server".to_string(),
                pure_rust: true,
                command: "petaltongue headless".to_string(),
            },
            ModeDetails {
                name: "status".to_string(),
                description: "System info".to_string(),
                pure_rust: true,
                command: "petaltongue status".to_string(),
            },
        ],
        features: vec![
            "UniBin".to_string(),
            "ecoBin 80%".to_string(),
            "Concurrent".to_string(),
            "No sleeps".to_string(),
            "Modern Rust".to_string(),
        ],
        dependencies: DependencyInfo {
            total: 150,
            c_deps: 1,
            rust_deps: 149,
        },
    }
}
