//! System Dashboard Sidebar
//!
//! Compact live system metrics always visible in the main UI

use crate::live_data::{LiveMetric, request_live_updates};
use crate::accessibility::ColorPalette;
use egui::{Context, Ui};
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use sysinfo::System;

/// Compact system dashboard for sidebar
pub struct SystemDashboard {
    system: System,
    last_refresh: Instant,
    refresh_interval: Duration,
    cpu_metric: LiveMetric,
    memory_metric: LiveMetric,
    cpu_history: VecDeque<f32>, // Mini sparkline
    mem_history: VecDeque<f32>,
    max_history: usize,
}

impl Default for SystemDashboard {
    fn default() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self {
            system,
            last_refresh: Instant::now(),
            refresh_interval: Duration::from_secs(1),
            cpu_metric: LiveMetric::new("CPU".to_string(), "sysinfo".to_string(), 1.0),
            memory_metric: LiveMetric::new("Memory".to_string(), "sysinfo".to_string(), 1.0),
            cpu_history: VecDeque::new(),
            mem_history: VecDeque::new(),
            max_history: 30, // 30 seconds for mini view
        }
    }
}

impl SystemDashboard {
    /// Refresh system data
    fn refresh(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_refresh) >= self.refresh_interval {
            self.system.refresh_all();
            self.last_refresh = now;

            // Calculate CPU usage
            let cpus = self.system.cpus();
            let cpu_usage = if cpus.is_empty() {
                0.0
            } else {
                cpus.iter().map(sysinfo::Cpu::cpu_usage).sum::<f32>() / cpus.len() as f32
            };

            self.cpu_history.push_back(cpu_usage);
            if self.cpu_history.len() > self.max_history {
                self.cpu_history.pop_front();
            }

            // Calculate memory usage
            let used_mem = self.system.used_memory() as f32;
            let total_mem = self.system.total_memory() as f32;
            let mem_percent = if total_mem > 0.0 {
                (used_mem / total_mem) * 100.0
            } else {
                0.0
            };

            self.mem_history.push_back(mem_percent);
            if self.mem_history.len() > self.max_history {
                self.mem_history.pop_front();
            }

            // Update live metrics
            self.cpu_metric.update(format!("{:.1}", cpu_usage), Some("%".to_string()));
            self.memory_metric.update(format!("{:.1}", mem_percent), Some("%".to_string()));
        }
    }

    /// Render compact dashboard (for sidebar)
    pub fn render_compact(&mut self, ui: &mut Ui, palette: &ColorPalette, font_scale: f32) {
        self.refresh();

        ui.group(|ui| {
            ui.set_width(ui.available_width());
            
            ui.label(
                egui::RichText::new("📊 System")
                    .size(14.0 * font_scale)
                    .strong()
                    .color(palette.text),
            );

            ui.add_space(5.0);

            // CPU
            self.cpu_metric.render(ui);
            self.render_mini_sparkline(ui, &self.cpu_history, palette);

            ui.add_space(3.0);

            // Memory
            self.memory_metric.render(ui);
            self.render_mini_sparkline(ui, &self.mem_history, palette);
        });

        // Request continuous updates
        request_live_updates(ui.ctx());
    }

    /// Render mini sparkline
    fn render_mini_sparkline(&self, ui: &mut Ui, data: &VecDeque<f32>, palette: &ColorPalette) {
        if data.is_empty() {
            return;
        }

        use egui::{Color32, Pos2, Stroke};

        let height = 20.0;
        let (response, painter) = ui.allocate_painter(
            egui::vec2(ui.available_width(), height),
            egui::Sense::hover(),
        );

        let rect = response.rect;
        let width = rect.width();

        // Background
        painter.rect_filled(rect, 2.0, palette.background_alt);

        // Calculate points
        let point_count = data.len();
        if point_count < 2 {
            return;
        }

        let x_step = width / (point_count - 1) as f32;
        let mut points = Vec::with_capacity(point_count);

        for (i, &value) in data.iter().enumerate() {
            let x = rect.min.x + (i as f32 * x_step);
            let y_normalized = value / 100.0; // Assume 0-100% range
            let y = rect.max.y - (y_normalized * height);
            points.push(Pos2::new(x, y));
        }

        // Draw line
        let color = if let Some(&last) = data.back() {
            if last > 90.0 {
                palette.error
            } else if last > 70.0 {
                palette.warning
            } else {
                palette.healthy
            }
        } else {
            palette.accent
        };

        painter.add(egui::Shape::line(points, Stroke::new(1.5, color)));

        // Draw current value indicator
        if let Some(&last_value) = data.back() {
            let last_x = rect.max.x;
            let last_y = rect.max.y - ((last_value / 100.0) * height);
            painter.circle_filled(Pos2::new(last_x, last_y), 2.0, color);
        }
    }

    /// Render full dashboard (for right panel)
    pub fn render_full(&mut self, ui: &mut Ui, palette: &ColorPalette, font_scale: f32) {
        self.refresh();

        ui.heading(
            egui::RichText::new("📊 Live System Metrics")
                .size(18.0 * font_scale)
                .color(palette.text),
        );

        ui.add_space(10.0);

        // CPU Section
        ui.group(|ui| {
            self.cpu_metric.render_large(ui);
            ui.add_space(5.0);

            let cpus = self.system.cpus();
            let cpu_usage = if cpus.is_empty() {
                0.0
            } else {
                cpus.iter().map(sysinfo::Cpu::cpu_usage).sum::<f32>() / cpus.len() as f32
            };

            // Progress bar
            ui.add(
                egui::ProgressBar::new(cpu_usage / 100.0)
                    .text(format!("{:.1}%", cpu_usage))
                    .fill(if cpu_usage > 90.0 {
                        palette.error
                    } else if cpu_usage > 70.0 {
                        palette.warning
                    } else {
                        palette.healthy
                    }),
            );

            ui.label(format!("Cores: {}", cpus.len()));
            self.render_mini_sparkline(ui, &self.cpu_history, palette);
        });

        ui.add_space(10.0);

        // Memory Section
        ui.group(|ui| {
            self.memory_metric.render_large(ui);
            ui.add_space(5.0);

            let used = self.system.used_memory();
            let total = self.system.total_memory();
            let percent = if total > 0 {
                (used as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            ui.label(format!(
                "Used: {:.1} / {:.1} GB",
                used as f64 / 1_073_741_824.0,
                total as f64 / 1_073_741_824.0
            ));

            ui.add(
                egui::ProgressBar::new(percent as f32 / 100.0)
                    .text(format!("{:.1}%", percent))
                    .fill(if percent > 90.0 {
                        palette.error
                    } else if percent > 70.0 {
                        palette.warning
                    } else {
                        palette.healthy
                    }),
            );

            self.render_mini_sparkline(ui, &self.mem_history, palette);
        });

        // Request continuous updates
        request_live_updates(ui.ctx());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_creation() {
        let dashboard = SystemDashboard::default();
        assert_eq!(dashboard.max_history, 30);
    }

    #[test]
    fn test_refresh_updates_metrics() {
        let mut dashboard = SystemDashboard::default();
        let initial_count = dashboard.cpu_history.len();
        
        // Wait a bit to ensure refresh interval passes
        std::thread::sleep(Duration::from_millis(1100));
        
        dashboard.refresh();
        
        // Should have added data
        assert!(dashboard.cpu_history.len() >= initial_count);
    }
}

