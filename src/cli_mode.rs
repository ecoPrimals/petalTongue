// SPDX-License-Identifier: AGPL-3.0-or-later
//! CLI mode - System status and information
//!
//! Pure Rust! ✅
//! Fully concurrent, no blocking operations

use crate::error::AppError;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Debug, Serialize)]
struct SystemStatus {
    version: String,
    mode: String,
    unibin: UniBinStatus,
    ecobin: EcoBinStatus,
    system: SystemInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    detailed: Option<DetailedStatus>,
}

#[derive(Clone, Debug, Serialize)]
struct UniBinStatus {
    compliant: bool,
    binary_count: u8,
    mode_count: u8,
}

#[derive(Clone, Debug, Serialize)]
struct EcoBinStatus {
    percentage: u8,
    pure_rust_modes: u8,
    total_modes: u8,
    modes: Vec<ModeInfo>,
}

#[derive(Clone, Debug, Serialize)]
struct ModeInfo {
    name: String,
    pure_rust: bool,
}

#[derive(Clone, Debug, Serialize)]
struct SystemInfo {
    os: String,
    arch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    cpu_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    memory_total: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
struct DetailedStatus {
    modes: Vec<ModeDetails>,
    features: Vec<String>,
    dependencies: DependencyInfo,
}

#[derive(Clone, Debug, Serialize)]
struct ModeDetails {
    name: String,
    description: String,
    pure_rust: bool,
    command: String,
}

#[derive(Clone, Debug, Serialize)]
struct DependencyInfo {
    total: usize,
    c_deps: usize,
    rust_deps: usize,
}

/// Show system status
///
/// Fully concurrent - gathers system info in parallel
pub async fn status(
    verbose: bool,
    format: &str,
    data_service: Arc<crate::data_service::DataService>,
) -> Result<(), AppError> {
    // Gather system info concurrently (no blocking!)
    let status = gather_status(verbose, &data_service).await?;

    // Output based on format
    match format {
        "json" => {
            // JSON output for programmatic use
            let json = serde_json::to_string_pretty(&status)
                .map_err(|e| AppError::Other(format!("Failed to serialize status to JSON: {e}")))?;
            println!("{json}");
        }
        _ => {
            // Human-readable text output
            print_status_text(&status);
        }
    }

    Ok(())
}

/// Gather system status concurrently
async fn gather_status(
    verbose: bool,
    data_service: &Arc<crate::data_service::DataService>,
) -> Result<SystemStatus, AppError> {
    // Use Arc<RwLock<>> for concurrent access
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

    // Gather system info concurrently
    let status_clone = Arc::clone(&status);
    let system_info_task = tokio::spawn(async move {
        // Get CPU count (non-blocking)
        let cpu_count = std::thread::available_parallelism()
            .map(std::num::NonZero::get)
            .unwrap_or(1);

        // Get memory info (non-blocking)
        let memory_total = get_total_memory().await;

        // Update status
        let mut status_guard = status_clone.write().await;
        status_guard.system.cpu_count = Some(cpu_count);
        status_guard.system.memory_total = memory_total;
    });

    // Show data service info
    tracing::info!("✅ Using shared DataService");
    if let Ok(snapshot) = data_service.snapshot().await {
        tracing::info!(
            "📊 DataService has {} primals, {} edges",
            snapshot.primals.len(),
            snapshot.edges.len()
        );
    }

    // If verbose, gather detailed info concurrently
    if verbose {
        let status_clone = Arc::clone(&status);
        let detailed_task = tokio::spawn(async move {
            let detailed = gather_detailed_status().await;
            let mut status_guard = status_clone.write().await;
            status_guard.detailed = Some(detailed);
        });

        // Wait for both tasks
        tokio::try_join!(system_info_task, detailed_task)
            .map_err(|e| AppError::Other(format!("Failed to gather system info: {e}")))?;
    } else {
        // Just wait for system info
        system_info_task
            .await
            .map_err(|e| AppError::Other(format!("Failed to gather system info: {e}")))?;
    }

    // Extract final status
    let final_status = status.read().await.clone();
    Ok(final_status)
}

/// Get total system memory (non-blocking)
async fn get_total_memory() -> Option<u64> {
    // Use tokio::task::spawn_blocking for potentially blocking syscalls
    tokio::task::spawn_blocking(|| {
        #[cfg(target_os = "linux")]
        {
            // Parse /proc/meminfo (Pure Rust!)
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
                                .map(|kb| kb * 1024) // Convert to bytes
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

/// Gather detailed status (concurrent)
#[expect(
    clippy::unused_async,
    reason = "async for future concurrent status gathering"
)]
async fn gather_detailed_status() -> DetailedStatus {
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
            total: 150, // Approximate
            c_deps: 1,  // Only GUI mode
            rust_deps: 149,
        },
    }
}

/// Print status in human-readable format
fn print_status_text(status: &SystemStatus) {
    println!("🌸 petalTongue ecoBud v{}", status.version);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // UniBin status
    println!("UniBin:");
    println!(
        "  ✅ {} binary, {} modes",
        status.unibin.binary_count, status.unibin.mode_count
    );
    println!();

    // ecoBin status
    println!("ecoBin:");
    println!(
        "  ✅ {}% Pure Rust ({}/{} modes)",
        status.ecobin.percentage, status.ecobin.pure_rust_modes, status.ecobin.total_modes
    );
    for mode in &status.ecobin.modes {
        let check = if mode.pure_rust { "✅" } else { "⚠️ " };
        println!("     {} {}", check, mode.name);
    }
    println!();

    // System info
    println!("System:");
    println!("  OS: {}", status.system.os);
    println!("  Arch: {}", status.system.arch);
    if let Some(cpus) = status.system.cpu_count {
        println!("  CPUs: {cpus}");
    }
    if let Some(mem) = status.system.memory_total {
        println!("  Memory: {} GB", mem / 1024 / 1024 / 1024);
    }

    // Detailed info if available
    if let Some(detailed) = &status.detailed {
        println!();
        println!("Modes:");
        for mode in &detailed.modes {
            let check = if mode.pure_rust { "✅" } else { "⚠️ " };
            println!("  {} {} - {}", check, mode.name, mode.description);
            println!("     Command: {}", mode.command);
        }

        println!();
        println!("Features:");
        for feature in &detailed.features {
            println!("  ✅ {feature}");
        }

        println!();
        println!("Dependencies:");
        println!("  Total: {}", detailed.dependencies.total);
        println!("  Pure Rust: {}", detailed.dependencies.rust_deps);
        println!(
            "  C deps: {} (only display mode)",
            detailed.dependencies.c_deps
        );
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

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
        // Test runs in parallel with others
        // No sleeps needed - proper async
        let data_service = Arc::new(crate::data_service::DataService::new());
        let status = gather_status(false, &data_service).await.unwrap();

        assert_eq!(status.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(status.unibin.binary_count, 1);
        assert_eq!(status.unibin.mode_count, 5);
        assert_eq!(status.ecobin.percentage, 80);
    }

    #[tokio::test]
    async fn test_gather_status_verbose_concurrent() {
        // Verbose mode gathers more info concurrently
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

        // Should be valid JSON
        assert!(json.contains("version"));
        assert!(json.contains("unibin"));
        assert!(json.contains("ecobin"));
    }

    #[tokio::test]
    async fn test_memory_info_nonblocking() {
        // Memory info gathering is non-blocking
        let mem = get_total_memory().await;

        #[cfg(target_os = "linux")]
        {
            // On Linux, should get some value
            assert!(mem.is_some());
        }

        // Test completes quickly (no sleeps!)
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
        // Any format other than "json" uses human-readable text output
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

    #[tokio::test]
    async fn test_gather_detailed_status() {
        let detailed = gather_detailed_status().await;
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
}
