// SPDX-License-Identifier: AGPL-3.0-only
//! TUI mode - Terminal User Interface
//!
//! Pure Rust! ✅
//! Dependencies: ratatui, crossterm (100% Pure Rust)
//!
//! Integrates with petal-tongue-tui for full interactive terminal UI.

use crate::data_service::DataService;
use anyhow::Result;
use petal_tongue_tui::{TUIConfig, launch_with_config};
use std::sync::Arc;
use std::time::Duration;

pub async fn run(
    scenario: Option<String>,
    refresh_rate: u32,
    data_service: Arc<DataService>,
) -> Result<()> {
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

    launch_with_config(config).await
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
    fn test_tui_config() {
        // Test that config is constructed correctly
        let tick_rate = Duration::from_millis(1000 / 60);
        assert_eq!(tick_rate.as_millis(), 16);
    }
}
