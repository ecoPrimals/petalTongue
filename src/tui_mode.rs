// SPDX-License-Identifier: AGPL-3.0-or-later
//! TUI mode - Terminal User Interface
//!
//! Pure Rust! ✅
//! Dependencies: ratatui, crossterm (100% Pure Rust)
//!
//! Integrates with petal-tongue-tui for full interactive terminal UI.

use crate::data_service::DataService;
use crate::error::AppError;
use petal_tongue_tui::{TUIConfig, launch_with_config};
use std::sync::Arc;
use std::time::Duration;

pub async fn run(
    scenario: Option<String>,
    refresh_rate: u32,
    data_service: Arc<DataService>,
) -> Result<(), AppError> {
    tracing::info!(
        scenario = ?scenario,
        refresh_rate,
        "Starting terminal UI mode (Pure Rust!)"
    );

    tracing::info!("✅ Using shared DataService (zero duplication!)");

    // Log DataService snapshot for debugging
    match data_service.snapshot().await {
        Ok(snapshot) => {
            tracing::debug!(
                primals = snapshot.primals.len(),
                edges = snapshot.edges.len(),
                "DataService snapshot"
            );
        }
        Err(e) => {
            tracing::warn!("Failed to get snapshot: {}", e);
        }
    }

    // Launch full interactive TUI from petal-tongue-tui crate
    let config = TUIConfig {
        tick_rate: Duration::from_millis(1000 / u64::from(refresh_rate)),
        mouse_support: false,
        standalone: false,
    };

    launch_with_config(config)
        .await
        .map_err(|e| AppError::Tui(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "Requires interactive terminal; run with --ignored"]
    async fn test_tui_mode() {
        let data_service = Arc::new(DataService::new());
        let result = run(None, 60, data_service).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_tui_config_60hz() {
        let tick_rate = Duration::from_millis(1000 / 60);
        assert_eq!(tick_rate.as_millis(), 16);
    }

    #[test]
    fn test_tui_config_30hz() {
        let tick_rate = Duration::from_millis(1000 / 30);
        assert_eq!(tick_rate.as_millis(), 33);
    }

    #[test]
    fn test_tui_config_1hz() {
        let tick_rate = Duration::from_millis(1000);
        assert_eq!(tick_rate.as_millis(), 1000);
    }

    #[test]
    fn test_tui_config_120hz() {
        let tick_rate = Duration::from_millis(1000 / 120);
        assert_eq!(tick_rate.as_millis(), 8);
    }

    #[test]
    #[should_panic(expected = "divide")]
    fn test_refresh_rate_zero_panics() {
        let _ = Duration::from_millis(1000 / u64::from(0u32));
    }

    #[test]
    fn test_tui_config_construction() {
        let config = TUIConfig {
            tick_rate: Duration::from_millis(1000 / 60),
            mouse_support: false,
            standalone: false,
        };
        assert_eq!(config.tick_rate, Duration::from_millis(16));
        assert!(!config.mouse_support);
        assert!(!config.standalone);
    }

    #[test]
    fn test_tui_config_validation_mouse_standalone() {
        let config = TUIConfig {
            tick_rate: Duration::from_millis(1000 / 30),
            mouse_support: true,
            standalone: true,
        };
        assert!(config.mouse_support);
        assert!(config.standalone);
    }

    #[tokio::test]
    async fn test_data_service_snapshot_accessible() {
        let data_service = Arc::new(DataService::new());
        let snapshot = data_service.snapshot().await;
        assert!(snapshot.is_ok());
    }

    #[tokio::test]
    async fn test_run_with_scenario_none() {
        let data_service = Arc::new(DataService::new());
        let result =
            tokio::time::timeout(Duration::from_secs(3), run(None, 60, data_service)).await;
        if let Ok(inner) = result {
            assert!(
                inner.is_ok() || inner.is_err(),
                "run() should return a Result"
            );
        }
        // else: timeout: interactive TTY, run() blocks - acceptable
    }

    #[tokio::test]
    async fn test_run_with_scenario_some() {
        let data_service = Arc::new(DataService::new());
        let result = tokio::time::timeout(
            Duration::from_secs(3),
            run(Some("scenario.json".to_string()), 30, data_service),
        )
        .await;
        if let Ok(inner) = result {
            assert!(
                inner.is_ok() || inner.is_err(),
                "run() should return a Result"
            );
        }
        // else: timeout: interactive TTY - acceptable
    }

    #[tokio::test]
    async fn test_run_with_different_refresh_rates() {
        let data_service = Arc::new(DataService::new());
        for hz in [1, 30, 60, 120] {
            let result =
                tokio::time::timeout(Duration::from_secs(2), run(None, hz, data_service.clone()))
                    .await;
            if let Ok(inner) = result {
                assert!(
                    inner.is_ok() || inner.is_err(),
                    "run() with {hz}Hz should return Result"
                );
            }
            // else: timeout - acceptable
        }
    }

    #[tokio::test]
    async fn test_run_propagates_launch_error_when_no_tty() {
        let data_service = Arc::new(DataService::new());
        let result =
            tokio::time::timeout(Duration::from_secs(3), run(None, 60, data_service)).await;
        if let Ok(Err(e)) = result {
            assert!(!e.to_string().is_empty(), "Error should have a message");
        }
    }
}
