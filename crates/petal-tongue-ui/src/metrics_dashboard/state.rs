// SPDX-License-Identifier: AGPL-3.0-or-later
//! Metrics dashboard widget state and async refresh.

use petal_tongue_core::{CpuHistory, MemoryHistory, SystemMetrics};
use petal_tongue_discovery::NeuralApiProvider;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Auto-refresh interval for metrics data (5 seconds)
pub(super) const REFRESH_INTERVAL: Duration = Duration::from_secs(5);

/// Metrics dashboard widget with sparklines
pub struct MetricsDashboard {
    /// Current metrics data (None if not yet fetched)
    pub(crate) data: Option<SystemMetrics>,

    /// CPU usage history for sparkline
    pub(crate) cpu_history: CpuHistory,

    /// Memory usage history for sparkline
    pub(crate) memory_history: MemoryHistory,

    /// Last update timestamp
    pub(crate) last_update: Instant,

    /// Whether data is currently being fetched
    pub(crate) fetching: bool,
}

impl MetricsDashboard {
    /// Create a new metrics dashboard
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: None,
            cpu_history: CpuHistory::new(),
            memory_history: MemoryHistory::new(),
            last_update: Instant::now()
                .checked_sub(REFRESH_INTERVAL)
                .unwrap_or_else(Instant::now),
            fetching: false,
        }
    }

    /// Update metrics data from Neural API (async)
    ///
    /// This should be called from an async context. The UI will show stale data
    /// while fetching new data.
    pub async fn update(&mut self, provider: &NeuralApiProvider) {
        if self.last_update.elapsed() < REFRESH_INTERVAL {
            return;
        }

        if self.fetching {
            return;
        }

        self.fetching = true;
        debug!("Fetching metrics data from Neural API...");

        match provider.get_metrics().await {
            Ok(result) => match serde_json::from_value::<SystemMetrics>(result) {
                Ok(metrics) => {
                    debug!(
                        "Metrics data received: CPU {:.1}%, Mem {:.1}%",
                        metrics.system.cpu_percent, metrics.system.memory_percent
                    );

                    self.cpu_history.push(metrics.system.cpu_percent);
                    self.memory_history.push(metrics.system.memory_percent);

                    self.data = Some(metrics);
                    self.last_update = Instant::now();
                }
                Err(e) => {
                    warn!("Failed to parse metrics data: {}", e);
                }
            },
            Err(e) => {
                warn!("Failed to fetch metrics data: {}", e);
            }
        }

        self.fetching = false;
    }
}

impl Default for MetricsDashboard {
    fn default() -> Self {
        Self::new()
    }
}
