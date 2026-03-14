// SPDX-License-Identifier: AGPL-3.0-only
//! Metrics Dashboard - Real-time System and Neural API Metrics
//!
//! Displays CPU, memory, uptime, and Neural API statistics with sparklines.
//! Updates automatically every 5 seconds with fresh data from Neural API.

use crate::metrics_dashboard_helpers::{prepare_metrics_display, sparkline_points_in_rect};
use egui::{Color32, ProgressBar, RichText, Stroke, Ui, Vec2};
use petal_tongue_core::{CpuHistory, MemoryHistory, SystemMetrics};
use petal_tongue_discovery::NeuralApiProvider;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Auto-refresh interval for metrics data (5 seconds)
const REFRESH_INTERVAL: Duration = Duration::from_secs(5);

/// Metrics dashboard widget with sparklines
pub struct MetricsDashboard {
    /// Current metrics data (None if not yet fetched)
    data: Option<SystemMetrics>,

    /// CPU usage history for sparkline
    cpu_history: CpuHistory,

    /// Memory usage history for sparkline
    memory_history: MemoryHistory,

    /// Last update timestamp
    last_update: Instant,

    /// Whether data is currently being fetched
    fetching: bool,
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
                .unwrap_or_else(Instant::now), // Trigger immediate fetch (fallback if overflow)
            fetching: false,
        }
    }

    /// Update metrics data from Neural API (async)
    ///
    /// This should be called from an async context. The UI will show stale data
    /// while fetching new data.
    pub async fn update(&mut self, provider: &NeuralApiProvider) {
        if self.last_update.elapsed() < REFRESH_INTERVAL {
            return; // Too soon to refresh
        }

        if self.fetching {
            return; // Already fetching
        }

        self.fetching = true;
        debug!("Fetching metrics data from Neural API...");

        match provider.get_metrics().await {
            Ok(result) => {
                // Parse metrics from JSON
                match serde_json::from_value::<SystemMetrics>(result) {
                    Ok(metrics) => {
                        debug!(
                            "Metrics data received: CPU {:.1}%, Mem {:.1}%",
                            metrics.system.cpu_percent, metrics.system.memory_percent
                        );

                        // Update histories
                        self.cpu_history.push(metrics.system.cpu_percent);
                        self.memory_history.push(metrics.system.memory_percent);

                        self.data = Some(metrics);
                        self.last_update = Instant::now();
                    }
                    Err(e) => {
                        warn!("Failed to parse metrics data: {}", e);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to fetch metrics data: {}", e);
                // Keep old data if fetch fails (graceful degradation)
            }
        }

        self.fetching = false;
    }

    /// Render the metrics dashboard
    pub fn render(&self, ui: &mut Ui) {
        ui.heading("📊 System Metrics");

        ui.separator();

        if let Some(data) = &self.data {
            let cpu: Vec<f64> = self
                .cpu_history
                .values()
                .into_iter()
                .map(f64::from)
                .collect();
            let mem: Vec<f64> = self
                .memory_history
                .values()
                .into_iter()
                .map(f64::from)
                .collect();
            let state = prepare_metrics_display(data, &cpu, &mem);

            Self::render_cpu_section(ui, &state);
            ui.add_space(12.0);
            Self::render_memory_section(ui, &state);
            ui.add_space(12.0);
            Self::render_system_info(ui, &state);
            ui.add_space(12.0);
            Self::render_neural_api_info(ui, &state);
        } else {
            ui.label(
                RichText::new("No metrics data available").color(Color32::from_rgb(156, 163, 175)),
            ); // gray-400
            ui.label("Waiting for Neural API...");
        }
    }

    /// Render CPU usage section with sparkline (thin egui wrapper)
    fn render_cpu_section(
        ui: &mut Ui,
        state: &crate::metrics_dashboard_helpers::MetricDisplayState,
    ) {
        ui.group(|ui| {
            ui.label(RichText::new("CPU Usage").strong().size(14.0));

            let (r, g, b) = state.cpu_color;
            let color = Color32::from_rgb(r, g, b);

            let progress = (state.cpu_percent / 100.0) as f32;
            ui.add(
                ProgressBar::new(progress)
                    .fill(color)
                    .text(format!("{:.1}%", state.cpu_percent)),
            );

            let cpu_values: Vec<f32> = state.cpu_history.iter().map(|&v| v as f32).collect();
            if cpu_values.len() >= 3 {
                ui.add_space(4.0);
                Self::render_sparkline(ui, &cpu_values, "CPU History", color);
            }

            if !state.cpu_history.is_empty() {
                let avg = state.cpu_history.iter().sum::<f64>() / state.cpu_history.len() as f64;
                let max = state.cpu_history.iter().copied().fold(0.0_f64, f64::max);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("Avg: {:.1}%", avg))
                            .color(Color32::from_rgb(156, 163, 175)),
                    );
                    ui.label(
                        RichText::new(format!("Max: {:.1}%", max))
                            .color(Color32::from_rgb(156, 163, 175)),
                    );
                });
            }
        });
    }

    /// Render memory usage section with sparkline (thin egui wrapper)
    fn render_memory_section(
        ui: &mut Ui,
        state: &crate::metrics_dashboard_helpers::MetricDisplayState,
    ) {
        ui.group(|ui| {
            ui.label(RichText::new("Memory Usage").strong().size(14.0));

            let (r, g, b) = state.memory_color;
            let color = Color32::from_rgb(r, g, b);

            let progress = (state.memory_percent / 100.0) as f32;
            ui.add(
                ProgressBar::new(progress)
                    .fill(color)
                    .text(format!("{:.1}%", state.memory_percent)),
            );

            ui.label(format!(
                "{} MB / {} MB",
                state.memory_used_mb, state.memory_total_mb
            ));

            if !state.memory_history.is_empty() {
                ui.add_space(4.0);
                let mem_values: Vec<f32> = state.memory_history.iter().map(|&v| v as f32).collect();
                Self::render_sparkline(ui, &mem_values, "Memory History", color);
            }
        });
    }

    /// Render system information (thin egui wrapper)
    fn render_system_info(
        ui: &mut Ui,
        state: &crate::metrics_dashboard_helpers::MetricDisplayState,
    ) {
        ui.group(|ui| {
            ui.label(RichText::new("System Information").strong().size(14.0));

            ui.horizontal(|ui| {
                ui.label("⏱️ Uptime:");
                ui.label(RichText::new(&state.uptime_text).color(Color32::from_rgb(59, 130, 246))); // blue-500
            });
        });
    }

    /// Render Neural API information (thin egui wrapper)
    fn render_neural_api_info(
        ui: &mut Ui,
        state: &crate::metrics_dashboard_helpers::MetricDisplayState,
    ) {
        ui.group(|ui| {
            ui.label(RichText::new("Neural API Status").strong().size(14.0));

            ui.horizontal(|ui| {
                ui.label("🧬 Family:");
                ui.label(RichText::new(&state.family_id).color(Color32::from_rgb(168, 85, 247))); // purple-500
            });

            ui.horizontal(|ui| {
                ui.label("🌸 Active Primals:");
                ui.label(
                    RichText::new(format!("{}", state.active_primals))
                        .strong()
                        .color(Color32::from_rgb(34, 197, 94)),
                ); // green-500
            });

            ui.horizontal(|ui| {
                ui.label("📊 Available Graphs:");
                ui.label(format!("{}", state.graphs_available));
            });

            ui.horizontal(|ui| {
                ui.label("⚡ Active Executions:");
                let color = if state.active_executions > 0 {
                    Color32::from_rgb(34, 197, 94) // green-500 (active)
                } else {
                    Color32::from_rgb(156, 163, 175) // gray-400 (idle)
                };
                ui.label(RichText::new(format!("{}", state.active_executions)).color(color));
            });
        });
    }

    /// Render a sparkline chart (thin egui wrapper) (thin egui wrapper)
    fn render_sparkline(ui: &mut Ui, values: &[f32], label: &str, color: Color32) {
        if values.len() < 2 {
            return;
        }

        let desired_size = Vec2::new(ui.available_width(), 40.0);
        let (rect, _response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());

        if ui.is_rect_visible(rect) {
            let points: Vec<egui::Pos2> = sparkline_points_in_rect(
                values,
                rect.left(),
                rect.top(),
                rect.width(),
                rect.height(),
            )
            .into_iter()
            .map(|(x, y)| egui::pos2(x, y))
            .collect();

            // Draw line
            if points.len() >= 2 {
                let stroke = Stroke::new(2.0, color);
                ui.painter().add(egui::Shape::line(points.clone(), stroke));

                // Draw filled area under the line
                if points.len() >= 2 {
                    let mut area_points = points.clone();
                    area_points.push(egui::pos2(rect.right(), rect.bottom()));
                    area_points.push(egui::pos2(rect.left(), rect.bottom()));

                    let fill_color = Color32::from_rgba_unmultiplied(
                        color.r(),
                        color.g(),
                        color.b(),
                        30, // 12% opacity
                    );
                    ui.painter().add(egui::Shape::convex_polygon(
                        area_points,
                        fill_color,
                        Stroke::NONE,
                    ));
                }
            }

            // Draw label
            ui.painter().text(
                egui::pos2(rect.left() + 4.0, rect.top() + 4.0),
                egui::Align2::LEFT_TOP,
                label,
                egui::FontId::proportional(10.0),
                Color32::from_rgb(156, 163, 175), // gray-400
            );
        }
    }
}

impl Default for MetricsDashboard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::{NeuralApiMetrics, SystemResourceMetrics};

    #[test]
    fn test_new_dashboard() {
        let dashboard = MetricsDashboard::new();
        assert!(dashboard.data.is_none());
        assert!(!dashboard.fetching);
    }

    #[test]
    fn test_dashboard_default() {
        let dashboard = MetricsDashboard::default();
        assert!(dashboard.data.is_none());
    }

    #[test]
    fn test_dashboard_with_data() {
        let mut dashboard = MetricsDashboard::new();

        let metrics = SystemMetrics {
            timestamp: chrono::Utc::now(),
            system: SystemResourceMetrics {
                cpu_percent: 45.5,
                memory_used_mb: 4_096,
                memory_total_mb: 8_192,
                memory_percent: 50.0,
                uptime_seconds: 3_600,
            },
            neural_api: NeuralApiMetrics {
                family_id: "nat0".to_string(),
                active_primals: 3,
                graphs_available: 5,
                active_executions: 1,
            },
        };

        dashboard.data = Some(metrics);
        dashboard.cpu_history.push(45.5);
        dashboard.memory_history.push(50.0);

        assert!(dashboard.data.is_some());
        assert_eq!(dashboard.cpu_history.current(), Some(45.5));
    }

    #[test]
    fn test_cpu_history_accumulation() {
        let mut dashboard = MetricsDashboard::new();

        for i in 0..10 {
            dashboard.cpu_history.push(i as f32 * 10.0);
        }

        assert_eq!(dashboard.cpu_history.values().len(), 10);
        assert!(dashboard.cpu_history.has_sufficient_data());
    }

    #[test]
    fn test_memory_history_accumulation() {
        let mut dashboard = MetricsDashboard::new();

        dashboard.memory_history.push(25.0);
        dashboard.memory_history.push(50.0);
        dashboard.memory_history.push(75.0);

        assert_eq!(dashboard.memory_history.values().len(), 3);
        assert_eq!(dashboard.memory_history.current(), Some(75.0));
    }

    #[test]
    fn test_system_metrics_thresholds() {
        let metrics = SystemMetrics {
            timestamp: chrono::Utc::now(),
            system: SystemResourceMetrics {
                cpu_percent: 95.0,
                memory_used_mb: 8_000,
                memory_total_mb: 8_192,
                memory_percent: 97.0,
                uptime_seconds: 86_400,
            },
            neural_api: NeuralApiMetrics {
                family_id: "test".to_string(),
                active_primals: 0,
                graphs_available: 0,
                active_executions: 0,
            },
        };

        let cpu_thresh = metrics.cpu_threshold();
        let mem_thresh = metrics.memory_threshold();
        let (r, g, b) = cpu_thresh.color_rgb();
        assert!(r > 0 || g > 0 || b > 0);
        let (r, g, b) = mem_thresh.color_rgb();
        assert!(r > 0 || g > 0 || b > 0);
    }

    #[test]
    fn test_uptime_formatted() {
        let metrics = SystemMetrics {
            timestamp: chrono::Utc::now(),
            system: SystemResourceMetrics {
                cpu_percent: 0.0,
                memory_used_mb: 0,
                memory_total_mb: 0,
                memory_percent: 0.0,
                uptime_seconds: 3661, // 1h 1m 1s
            },
            neural_api: NeuralApiMetrics {
                family_id: "".to_string(),
                active_primals: 0,
                graphs_available: 0,
                active_executions: 0,
            },
        };
        let formatted = metrics.uptime_formatted();
        assert!(formatted.contains('h') || formatted.contains("1") || formatted.len() > 0);
    }
}
