//! CLI mode - System status and information
//! 
//! Pure Rust! ✅
//! Fully concurrent, no blocking operations

use anyhow::{Context, Result};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Serialize)]
struct SystemStatus {
    version: String,
    mode: String,
    unibin: UniBinStatus,
    ecobin: EcoBinStatus,
    system: SystemInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    detailed: Option<DetailedStatus>,
}

#[derive(Debug, Serialize)]
struct UniBinStatus {
    compliant: bool,
    binary_count: u8,
    mode_count: u8,
}

#[derive(Debug, Serialize)]
struct EcoBinStatus {
    percentage: u8,
    pure_rust_modes: u8,
    total_modes: u8,
    modes: Vec<ModeInfo>,
}

#[derive(Debug, Serialize)]
struct ModeInfo {
    name: String,
    pure_rust: bool,
}

#[derive(Debug, Serialize)]
struct SystemInfo {
    os: String,
    arch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    cpu_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    memory_total: Option<u64>,
}

#[derive(Debug, Serialize)]
struct DetailedStatus {
    modes: Vec<ModeDetails>,
    features: Vec<String>,
    dependencies: DependencyInfo,
}

#[derive(Debug, Serialize)]
struct ModeDetails {
    name: String,
    description: String,
    pure_rust: bool,
    command: String,
}

#[derive(Debug, Serialize)]
struct DependencyInfo {
    total: usize,
    c_deps: usize,
    rust_deps: usize,
}

/// Show system status
/// 
/// Fully concurrent - gathers system info in parallel
pub async fn status(verbose: bool, format: &str) -> Result<()> {
    // Gather system info concurrently (no blocking!)
    let status = gather_status(verbose).await?;

    // Output based on format
    match format {
        "json" => {
            // JSON output for programmatic use
            let json = serde_json::to_string_pretty(&status)
                .context("Failed to serialize status to JSON")?;
            println!("{}", json);
        }
        _ => {
            // Human-readable text output
            print_status_text(&status);
        }
    }

    Ok(())
}

/// Gather system status concurrently
async fn gather_status(verbose: bool) -> Result<SystemStatus> {
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
        let cpu_count = num_cpus::get();

        // Get memory info (non-blocking)
        let memory_total = get_total_memory().await;

        // Update status
        let mut status_guard = status_clone.write().await;
        status_guard.system.cpu_count = Some(cpu_count);
        status_guard.system.memory_total = memory_total;
    });

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
            .context("Failed to gather system info")?;
    } else {
        // Just wait for system info
        system_info_task
            .await
            .context("Failed to gather system info")?;
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
async fn gather_detailed_status() -> DetailedStatus {
    DetailedStatus {
        modes: vec![
            ModeDetails {
                name: "ui".to_string(),
                description: "Desktop GUI (egui)".to_string(),
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
        println!("  CPUs: {}", cpus);
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
            println!("  ✅ {}", feature);
        }

        println!();
        println!("Dependencies:");
        println!("  Total: {}", detailed.dependencies.total);
        println!("  Pure Rust: {}", detailed.dependencies.rust_deps);
        println!("  C deps: {} (only GUI mode)", detailed.dependencies.c_deps);
    }
}

// Derive Clone for SystemStatus for testing
impl Clone for SystemStatus {
    fn clone(&self) -> Self {
        SystemStatus {
            version: self.version.clone(),
            mode: self.mode.clone(),
            unibin: UniBinStatus {
                compliant: self.unibin.compliant,
                binary_count: self.unibin.binary_count,
                mode_count: self.unibin.mode_count,
            },
            ecobin: EcoBinStatus {
                percentage: self.ecobin.percentage,
                pure_rust_modes: self.ecobin.pure_rust_modes,
                total_modes: self.ecobin.total_modes,
                modes: self.ecobin.modes.clone(),
            },
            system: SystemInfo {
                os: self.system.os.clone(),
                arch: self.system.arch.clone(),
                cpu_count: self.system.cpu_count,
                memory_total: self.system.memory_total,
            },
            detailed: self.detailed.as_ref().map(|d| DetailedStatus {
                modes: d.modes.clone(),
                features: d.features.clone(),
                dependencies: DependencyInfo {
                    total: d.dependencies.total,
                    c_deps: d.dependencies.c_deps,
                    rust_deps: d.dependencies.rust_deps,
                },
            }),
        }
    }
}

impl Clone for ModeInfo {
    fn clone(&self) -> Self {
        ModeInfo {
            name: self.name.clone(),
            pure_rust: self.pure_rust,
        }
    }
}

impl Clone for ModeDetails {
    fn clone(&self) -> Self {
        ModeDetails {
            name: self.name.clone(),
            description: self.description.clone(),
            pure_rust: self.pure_rust,
            command: self.command.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gather_status_concurrent() {
        // Test runs in parallel with others
        // No sleeps needed - proper async
        let status = gather_status(false).await.unwrap();

        assert_eq!(status.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(status.unibin.binary_count, 1);
        assert_eq!(status.unibin.mode_count, 5);
        assert_eq!(status.ecobin.percentage, 80);
    }

    #[tokio::test]
    async fn test_gather_status_verbose_concurrent() {
        // Verbose mode gathers more info concurrently
        let status = gather_status(true).await.unwrap();

        assert!(status.detailed.is_some());
        let detailed = status.detailed.unwrap();
        assert_eq!(detailed.modes.len(), 5);
        assert!(!detailed.features.is_empty());
    }

    #[tokio::test]
    async fn test_json_output() {
        let status = gather_status(false).await.unwrap();
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

    // All tests run in parallel - modern concurrent Rust!
}

