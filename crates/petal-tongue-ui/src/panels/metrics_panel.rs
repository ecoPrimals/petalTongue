// SPDX-License-Identifier: AGPL-3.0-only
//! System Metrics Panel - Display real-time system and biomeOS metrics
//!
//! Phase 1.4: Integrates with Neural API to show:
//! - CPU usage
//! - Memory usage and availability
//! - System uptime
//! - Active primals count
//! - Available graphs

use crate::panel_registry::{PanelFactory, PanelInstance};
use crate::scenario::CustomPanelConfig;
use petal_tongue_discovery::NeuralApiProvider;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// System metrics from Neural API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// ISO 8601 timestamp when metrics were collected
    pub timestamp: String,
    /// System-level statistics (CPU, memory, uptime)
    pub system: SystemStats,
    /// Neural API specific statistics
    pub neural_api: NeuralApiStats,
}

/// System-level statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    /// CPU usage percentage (0-100)
    pub cpu_percent: f64,
    /// Memory currently in use (megabytes)
    pub memory_used_mb: u64,
    /// Total system memory (megabytes)
    pub memory_total_mb: u64,
    /// Memory usage percentage (0-100)
    pub memory_percent: f64,
    /// System uptime in seconds
    pub uptime_seconds: u64,
}

/// Neural API specific statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuralApiStats {
    /// Family identifier for this primal cluster
    pub family_id: String,
    /// Number of currently active primals
    pub active_primals: u64,
    /// Number of graphs available for execution
    pub graphs_available: u64,
    /// Number of graphs currently executing
    pub active_executions: u64,
}

/// Panel that displays system metrics
pub struct MetricsPanel {
    provider: Option<NeuralApiProvider>,
    last_metrics: Option<SystemMetrics>,
    last_update: Instant,
    update_interval: Duration,
    error_message: Option<String>,
}

impl MetricsPanel {
    /// Create a new metrics panel (provider connected later)
    #[must_use]
    pub fn new() -> Self {
        Self {
            provider: None,
            last_metrics: None,
            last_update: Instant::now(),
            update_interval: Duration::from_secs(1), // Update every second
            error_message: None,
        }
    }

    #[allow(dead_code)]
    async fn refresh_metrics(&mut self) {
        if let Some(provider) = &self.provider {
            match provider.get_metrics().await {
                Ok(json_value) => {
                    // Parse the JSON into our structs
                    match serde_json::from_value::<SystemMetrics>(json_value) {
                        Ok(metrics) => {
                            self.last_metrics = Some(metrics);
                            self.last_update = Instant::now();
                            self.error_message = None;
                        }
                        Err(e) => {
                            self.error_message = Some(format!("Parse error: {e}"));
                            tracing::warn!("Failed to parse metrics: {}", e);
                        }
                    }
                }
                Err(e) => {
                    self.error_message = Some(format!("API error: {e}"));
                    tracing::warn!("Failed to get metrics: {}", e);
                }
            }
        }
    }

    fn format_uptime(seconds: u64) -> String {
        let days = seconds / 86400;
        let hours = (seconds % 86400) / 3600;
        let minutes = (seconds % 3600) / 60;

        if days > 0 {
            format!("{days}d {hours}h {minutes}m")
        } else if hours > 0 {
            format!("{hours}h {minutes}m")
        } else {
            format!("{minutes}m")
        }
    }
}

impl PanelInstance for MetricsPanel {
    fn title(&self) -> &'static str {
        "System Metrics"
    }

    fn on_open(&mut self) -> anyhow::Result<()> {
        tracing::info!("Metrics Panel opened - discovering Neural API");

        // Discover Neural API in blocking context
        let provider = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { NeuralApiProvider::discover(None).await.ok() })
        });

        if provider.is_some() {
            tracing::info!("✅ Neural API discovered for metrics panel");
        } else {
            tracing::warn!("⚠️  Neural API not available - metrics will not update");
            self.error_message = Some("Neural API not available".to_string());
        }

        self.provider = provider;
        Ok(())
    }

    fn on_close(&mut self) -> anyhow::Result<()> {
        tracing::info!("Metrics Panel closed");
        Ok(())
    }

    fn update(&mut self) {
        // Updates are handled in render() to keep it simple
        // In a more complex system, we'd use channels to update in background
    }

    fn render(&mut self, ui: &mut egui::Ui) {
        // Try to refresh if needed (blocking is acceptable in render since it's fast)
        if self.last_update.elapsed() > self.update_interval && self.provider.is_some() {
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    if let Some(provider) = &self.provider
                        && let Ok(json_value) = provider.get_metrics().await
                        && let Ok(metrics) = serde_json::from_value::<SystemMetrics>(json_value)
                    {
                        self.last_metrics = Some(metrics);
                        self.last_update = Instant::now();
                        self.error_message = None;
                    }
                });
            });
        }

        ui.heading("📊 System Metrics");
        ui.separator();

        if let Some(error) = &self.error_message {
            ui.colored_label(egui::Color32::RED, format!("⚠️  {error}"));
            ui.separator();
        }

        if let Some(metrics) = &self.last_metrics {
            // System section
            ui.label("System:");

            // CPU bar
            let cpu = metrics.system.cpu_percent;
            ui.horizontal(|ui| {
                ui.label(format!("CPU: {cpu:.1}%"));
            });
            ui.add(
                egui::ProgressBar::new((cpu / 100.0) as f32)
                    .show_percentage()
                    .animate(true),
            );

            ui.add_space(4.0);

            // Memory bar
            let mem = metrics.system.memory_percent;
            ui.horizontal(|ui| {
                ui.label(format!("Memory: {mem:.1}%"));
            });
            ui.label(format!(
                "  {} / {} MB",
                metrics.system.memory_used_mb, metrics.system.memory_total_mb
            ));
            ui.add(
                egui::ProgressBar::new((mem / 100.0) as f32)
                    .show_percentage()
                    .animate(true),
            );

            ui.add_space(4.0);

            // Uptime
            let uptime_str = Self::format_uptime(metrics.system.uptime_seconds);
            ui.label(format!("Uptime: {uptime_str}"));

            ui.separator();

            // Neural API section
            ui.label("biomeOS (Neural API):");
            ui.label(format!("  Family: {}", metrics.neural_api.family_id));
            ui.label(format!(
                "  Active Primals: {}",
                metrics.neural_api.active_primals
            ));
            ui.label(format!("  Graphs: {}", metrics.neural_api.graphs_available));

            if metrics.neural_api.active_executions > 0 {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    format!("  ⚡ Executions: {}", metrics.neural_api.active_executions),
                );
            }

            ui.add_space(4.0);

            // Last update time
            let age = self.last_update.elapsed().as_secs();
            if age < 5 {
                ui.label(format!("📡 Updated {age}s ago"));
            } else {
                ui.colored_label(egui::Color32::YELLOW, format!("⏳ Updated {age}s ago"));
            }
        } else if self.provider.is_none() {
            ui.label("⏳ Neural API not available");
            ui.label("Start NUCLEUS to see metrics:");
            ui.code("nucleus serve --family nat0");
        } else {
            ui.label("⏳ Connecting to Neural API...");
        }
    }

    fn wants_keyboard_input(&self) -> bool {
        false
    }

    fn wants_mouse_input(&self) -> bool {
        false
    }
}

impl Default for MetricsPanel {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory for creating Metrics panels
pub struct MetricsPanelFactory;

impl MetricsPanelFactory {
    /// Create a new factory for metrics panels
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl PanelFactory for MetricsPanelFactory {
    fn panel_type(&self) -> &'static str {
        "metrics"
    }

    fn create(
        &self,
        _config: &CustomPanelConfig,
    ) -> crate::panel_registry::Result<Box<dyn PanelInstance>> {
        Ok(Box::new(MetricsPanel::new()))
    }

    fn description(&self) -> &'static str {
        "Displays real-time system and biomeOS metrics from Neural API"
    }
}

impl Default for MetricsPanelFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_panel_creation() {
        let panel = MetricsPanel::new();
        assert_eq!(panel.title(), "System Metrics");
        assert!(!panel.wants_keyboard_input());
        assert!(!panel.wants_mouse_input());
    }

    #[test]
    fn test_metrics_panel_factory() {
        let factory = MetricsPanelFactory::new();
        assert_eq!(factory.panel_type(), "metrics");
        assert_eq!(
            factory.description(),
            "Displays real-time system and biomeOS metrics from Neural API"
        );

        let config = CustomPanelConfig {
            panel_type: "metrics".to_string(),
            title: "Test Metrics".to_string(),
            width: None,
            height: None,
            fullscreen: false,
            config: serde_json::json!({}),
        };
        let panel = factory.create(&config);
        assert!(panel.is_ok());
    }

    #[test]
    fn test_uptime_formatting() {
        assert_eq!(MetricsPanel::format_uptime(59), "0m");
        assert_eq!(MetricsPanel::format_uptime(60), "1m");
        assert_eq!(MetricsPanel::format_uptime(3600), "1h 0m");
        assert_eq!(MetricsPanel::format_uptime(3661), "1h 1m");
        assert_eq!(MetricsPanel::format_uptime(86400), "1d 0h 0m");
        assert_eq!(MetricsPanel::format_uptime(90061), "1d 1h 1m");
    }

    #[test]
    fn test_metrics_parsing() {
        let json = serde_json::json!({
            "timestamp": "2026-01-15T22:00:00Z",
            "system": {
                "cpu_percent": 16.5,
                "memory_used_mb": 32768,
                "memory_total_mb": 49152,
                "memory_percent": 66.7,
                "uptime_seconds": 86400
            },
            "neural_api": {
                "family_id": "nat0",
                "active_primals": 3,
                "graphs_available": 5,
                "active_executions": 0
            }
        });

        let metrics: Result<SystemMetrics, _> = serde_json::from_value(json);
        assert!(metrics.is_ok());

        let metrics = metrics.unwrap();
        assert_eq!(metrics.system.cpu_percent, 16.5);
        assert_eq!(metrics.neural_api.active_primals, 3);
    }
}
