// SPDX-License-Identifier: AGPL-3.0-only
//! System Metrics Panel - Display real-time system and biomeOS metrics
//!
//! Integrates with Neural API to show:
//! - CPU usage
//! - Memory usage and availability
//! - System uptime
//! - Active primals count
//! - Available graphs

use crate::panel_registry::{PanelFactory, PanelInstance};
use crate::panels::metrics_panel_display::prepare_metrics_panel_display;
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
}

impl PanelInstance for MetricsPanel {
    fn title(&self) -> &'static str {
        "System Metrics"
    }

    fn on_open(&mut self) -> crate::error::Result<()> {
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

    fn on_close(&mut self) -> crate::error::Result<()> {
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

        let display = prepare_metrics_panel_display(
            &self.last_metrics,
            self.last_update,
            &self.error_message,
        );

        ui.heading("📊 System Metrics");
        ui.separator();

        if let Some(error) = &display.error_message {
            ui.colored_label(egui::Color32::RED, format!("⚠️  {error}"));
            ui.separator();
        }

        if let Some(summary) = &display.metrics_summary {
            // System section
            ui.label("System:");

            // CPU bar
            let cpu = summary.cpu_percent;
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
            let mem = summary.memory_percent;
            ui.horizontal(|ui| {
                ui.label(format!("Memory: {mem:.1}%"));
            });
            ui.label(format!(
                "  {} / {} MB",
                summary.memory_used_mb, summary.memory_total_mb
            ));
            ui.add(
                egui::ProgressBar::new((mem / 100.0) as f32)
                    .show_percentage()
                    .animate(true),
            );

            ui.add_space(4.0);

            // Uptime
            ui.label(format!("Uptime: {}", summary.uptime_str));

            ui.separator();

            // Neural API section
            ui.label("biomeOS (Neural API):");
            ui.label(format!("  Family: {}", summary.family_id));
            ui.label(format!("  Active Primals: {}", summary.active_primals));
            ui.label(format!("  Graphs: {}", summary.graphs_available));

            if summary.active_executions > 0 {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    format!("  ⚡ Executions: {}", summary.active_executions),
                );
            }

            ui.add_space(4.0);

            // Last update time
            if let Some(age_text) = &display.update_age_text {
                if display.is_stale {
                    ui.colored_label(egui::Color32::YELLOW, format!("⏳ {age_text}"));
                } else {
                    ui.label(format!("📡 {age_text}"));
                }
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
    pub const fn new() -> Self {
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
    use crate::panels::metrics_panel_display::format_uptime;
    use std::time::Duration;

    #[test]
    fn test_metrics_panel_default() {
        let panel = MetricsPanel::default();
        assert_eq!(panel.title(), "System Metrics");
    }

    #[test]
    fn test_metrics_panel_factory_default() {
        let factory = MetricsPanelFactory;
        assert_eq!(factory.panel_type(), "metrics");
    }

    #[test]
    fn test_metrics_panel_update() {
        let mut panel = MetricsPanel::new();
        panel.update();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_metrics_panel_on_open_on_close() {
        let mut panel = MetricsPanel::new();
        assert!(panel.on_open().is_ok());
        assert!(panel.on_close().is_ok());
    }

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
        assert_eq!(format_uptime(Duration::from_secs(59)), "0m");
        assert_eq!(format_uptime(Duration::from_secs(60)), "1m");
        assert_eq!(format_uptime(Duration::from_secs(3600)), "1h 0m");
        assert_eq!(format_uptime(Duration::from_secs(3661)), "1h 1m");
        assert_eq!(format_uptime(Duration::from_secs(86400)), "1d 0h 0m");
        assert_eq!(format_uptime(Duration::from_secs(90061)), "1d 1h 1m");
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

    #[test]
    fn test_prepare_metrics_panel_display() {
        use crate::panels::metrics_panel_display::prepare_metrics_panel_display;

        let metrics = SystemMetrics {
            timestamp: "2026-01-15T22:00:00Z".to_string(),
            system: SystemStats {
                cpu_percent: 25.0,
                memory_used_mb: 4096,
                memory_total_mb: 16384,
                memory_percent: 25.0,
                uptime_seconds: 7200,
            },
            neural_api: NeuralApiStats {
                family_id: "test".to_string(),
                active_primals: 2,
                graphs_available: 4,
                active_executions: 1,
            },
        };

        let last_update = Instant::now() - Duration::from_secs(5);
        let display = prepare_metrics_panel_display(&Some(metrics), last_update, &None);
        assert!(display.metrics_summary.is_some());
        let s = display.metrics_summary.unwrap();
        assert_eq!(s.cpu_percent, 25.0);
        assert_eq!(s.uptime_str, "2h 0m");
        assert_eq!(s.active_executions, 1);
    }

    #[test]
    fn test_format_update_age() {
        use crate::panels::metrics_panel_display::format_update_age;

        assert_eq!(format_update_age(0), "Updated 0s ago");
        assert_eq!(format_update_age(29), "Updated 29s ago");
        assert_eq!(format_update_age(30), "Stale (>30s)");
    }
}
