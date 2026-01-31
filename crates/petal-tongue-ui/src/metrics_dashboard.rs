//! Metrics Dashboard - Real-time System and Neural API Metrics
//!
//! Displays CPU, memory, uptime, and Neural API statistics with sparklines.
//! Updates automatically every 5 seconds with fresh data from Neural API.

use egui::{Color32, ProgressBar, RichText, Stroke, Ui, Vec2};
use petal_tongue_core::{CpuHistory, MemoryHistory, SystemMetrics, ThresholdLevel};
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
            last_update: Instant::now() - REFRESH_INTERVAL, // Trigger immediate fetch
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
            self.render_cpu_section(ui, data);
            ui.add_space(12.0);
            self.render_memory_section(ui, data);
            ui.add_space(12.0);
            self.render_system_info(ui, data);
            ui.add_space(12.0);
            self.render_neural_api_info(ui, data);
        } else {
            ui.label(
                RichText::new("No metrics data available").color(Color32::from_rgb(156, 163, 175)),
            ); // gray-400
            ui.label("Waiting for Neural API...");
        }
    }

    /// Render CPU usage section with sparkline
    fn render_cpu_section(&self, ui: &mut Ui, data: &SystemMetrics) {
        ui.group(|ui| {
            ui.label(RichText::new("CPU Usage").strong().size(14.0));

            // Progress bar with color coding
            let threshold = data.cpu_threshold();
            let (r, g, b) = threshold.color_rgb();
            let color = Color32::from_rgb(r, g, b);

            let progress = data.system.cpu_percent / 100.0;
            ui.add(
                ProgressBar::new(progress)
                    .fill(color)
                    .text(format!("{:.1}%", data.system.cpu_percent)),
            );

            // Sparkline
            if self.cpu_history.has_sufficient_data() {
                ui.add_space(4.0);
                self.render_sparkline(ui, &self.cpu_history.values(), "CPU History", color);
            }

            // Stats
            if !self.cpu_history.values().is_empty() {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("Avg: {:.1}%", self.cpu_history.average()))
                            .color(Color32::from_rgb(156, 163, 175)),
                    ); // gray-400
                    ui.label(
                        RichText::new(format!("Max: {:.1}%", self.cpu_history.max()))
                            .color(Color32::from_rgb(156, 163, 175)),
                    );
                });
            }
        });
    }

    /// Render memory usage section with sparkline
    fn render_memory_section(&self, ui: &mut Ui, data: &SystemMetrics) {
        ui.group(|ui| {
            ui.label(RichText::new("Memory Usage").strong().size(14.0));

            // Progress bar with color coding
            let threshold = data.memory_threshold();
            let (r, g, b) = threshold.color_rgb();
            let color = Color32::from_rgb(r, g, b);

            let progress = data.system.memory_percent / 100.0;
            ui.add(
                ProgressBar::new(progress)
                    .fill(color)
                    .text(format!("{:.1}%", data.system.memory_percent)),
            );

            // Memory details
            ui.label(format!(
                "{} MB / {} MB",
                data.system.memory_used_mb, data.system.memory_total_mb
            ));

            // Sparkline
            if self.memory_history.current().is_some() {
                ui.add_space(4.0);
                self.render_sparkline(ui, &self.memory_history.values(), "Memory History", color);
            }
        });
    }

    /// Render system information
    fn render_system_info(&self, ui: &mut Ui, data: &SystemMetrics) {
        ui.group(|ui| {
            ui.label(RichText::new("System Information").strong().size(14.0));

            ui.horizontal(|ui| {
                ui.label("⏱️ Uptime:");
                ui.label(
                    RichText::new(data.uptime_formatted()).color(Color32::from_rgb(59, 130, 246)),
                ); // blue-500
            });
        });
    }

    /// Render Neural API information
    fn render_neural_api_info(&self, ui: &mut Ui, data: &SystemMetrics) {
        ui.group(|ui| {
            ui.label(RichText::new("Neural API Status").strong().size(14.0));

            ui.horizontal(|ui| {
                ui.label("🧬 Family:");
                ui.label(
                    RichText::new(&data.neural_api.family_id)
                        .color(Color32::from_rgb(168, 85, 247)),
                ); // purple-500
            });

            ui.horizontal(|ui| {
                ui.label("🌸 Active Primals:");
                ui.label(
                    RichText::new(format!("{}", data.neural_api.active_primals))
                        .strong()
                        .color(Color32::from_rgb(34, 197, 94)),
                ); // green-500
            });

            ui.horizontal(|ui| {
                ui.label("📊 Available Graphs:");
                ui.label(format!("{}", data.neural_api.graphs_available));
            });

            ui.horizontal(|ui| {
                ui.label("⚡ Active Executions:");
                let color = if data.neural_api.active_executions > 0 {
                    Color32::from_rgb(34, 197, 94) // green-500 (active)
                } else {
                    Color32::from_rgb(156, 163, 175) // gray-400 (idle)
                };
                ui.label(
                    RichText::new(format!("{}", data.neural_api.active_executions)).color(color),
                );
            });
        });
    }

    /// Render a sparkline chart
    fn render_sparkline(&self, ui: &mut Ui, values: &[f32], label: &str, color: Color32) {
        if values.len() < 2 {
            return;
        }

        let desired_size = Vec2::new(ui.available_width(), 40.0);
        let (rect, _response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());

        if ui.is_rect_visible(rect) {
            // Find min/max for scaling
            let min_val = values.iter().copied().fold(f32::INFINITY, f32::min);
            let max_val = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            let range = (max_val - min_val).max(0.1); // Avoid division by zero

            // Calculate points
            let mut points = Vec::new();
            let width = rect.width();
            let height = rect.height();

            for (i, &value) in values.iter().enumerate() {
                let x = rect.left() + (i as f32 / (values.len() - 1) as f32) * width;
                let normalized = (value - min_val) / range;
                let y = rect.bottom() - normalized * height;
                points.push(egui::pos2(x, y));
            }

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
}
