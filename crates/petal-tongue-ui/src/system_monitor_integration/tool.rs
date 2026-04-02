// SPDX-License-Identifier: AGPL-3.0-or-later
//! `/proc`-backed sampling, history buffers, and egui rendering for the System Monitor tool.

use super::display_compute::compute_sparkline_points;
use super::display_compute::{prepare_cpu_display, prepare_memory_display};
use super::display_state::{CpuDisplayState, MemoryDisplayState};
use crate::live_data::{LiveGraphHeader, LiveMetric};
use crate::proc_stats::{ProcStats, SOURCE_ID};
use crate::tool_integration::{ToolCapability, ToolMetadata, ToolPanel};
use egui::Color32;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// System Monitor tool integration
///
/// Provides real-time system resource monitoring (CPU, memory) via /proc parsing.
/// This demonstrates petalTongue's capability-based tool integration pattern.
/// Features LIVE data indicators to prove all metrics are real-time.
pub struct SystemMonitorTool {
    show_panel: bool,
    stats: ProcStats,
    last_refresh: Instant,
    refresh_interval: Duration,
    cpu_history: VecDeque<f32>,
    mem_history: VecDeque<f32>,
    max_history: usize,
    last_cpu_usage: f32,

    cpu_display: CpuDisplayState,
    mem_display: MemoryDisplayState,

    cpu_header: LiveGraphHeader,
    memory_header: LiveGraphHeader,
    cpu_metric: LiveMetric,
    memory_metric: LiveMetric,
}

impl Default for SystemMonitorTool {
    fn default() -> Self {
        Self {
            show_panel: false,
            stats: ProcStats::new(),
            last_refresh: Instant::now(),
            refresh_interval: Duration::from_secs(1),
            cpu_history: VecDeque::new(),
            mem_history: VecDeque::new(),
            max_history: 60,
            last_cpu_usage: 0.0,

            cpu_display: prepare_cpu_display(0.0, 0, 0),
            mem_display: prepare_memory_display(0, 0, 0),

            cpu_header: LiveGraphHeader::new(
                "💻 CPU Usage".to_string(),
                SOURCE_ID.to_string(),
                1.0,
            ),
            memory_header: LiveGraphHeader::new(
                "🧠 Memory Usage".to_string(),
                SOURCE_ID.to_string(),
                1.0,
            ),
            cpu_metric: LiveMetric::new("Current CPU".to_string(), SOURCE_ID.to_string(), 1.0),
            memory_metric: LiveMetric::new(
                "Current Memory".to_string(),
                SOURCE_ID.to_string(),
                1.0,
            ),
        }
    }
}

impl SystemMonitorTool {
    /// Refresh system information and rebuild display states.
    ///
    /// Must be called before `render_panel()` -- the render method is a
    /// dumb pipe that reads pre-computed display state.
    pub fn refresh(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_refresh) >= self.refresh_interval {
            let cpu_usage = self.stats.cpu_usage();
            self.last_cpu_usage = cpu_usage;
            self.last_refresh = now;

            self.cpu_history.push_back(cpu_usage);
            if self.cpu_history.len() > self.max_history {
                self.cpu_history.pop_front();
            }

            let used_mem = self.stats.used_memory() as f32;
            let total_mem = self.stats.total_memory() as f32;
            let mem_percent = if total_mem > 0.0 {
                (used_mem / total_mem) * 100.0
            } else {
                0.0
            };
            self.mem_history.push_back(mem_percent);
            if self.mem_history.len() > self.max_history {
                self.mem_history.pop_front();
            }

            self.cpu_header.mark_updated();
            self.memory_header.mark_updated();
            self.cpu_metric
                .update(format!("{cpu_usage:.1}"), Some("%".to_string()));
            self.memory_metric
                .update(format!("{mem_percent:.1}"), Some("%".to_string()));

            self.cpu_display =
                prepare_cpu_display(cpu_usage, self.stats.cpu_count(), self.cpu_history.len());
            self.mem_display = prepare_memory_display(
                self.stats.used_memory(),
                self.stats.total_memory(),
                self.mem_history.len(),
            );
        }
    }

    /// Read-only access to the current CPU display state (for headless testing).
    #[must_use]
    pub const fn cpu_display(&self) -> &CpuDisplayState {
        &self.cpu_display
    }

    /// Read-only access to the current memory display state (for headless testing).
    #[must_use]
    pub const fn mem_display(&self) -> &MemoryDisplayState {
        &self.mem_display
    }

    /// Thin render: CPU section. Reads pre-computed `cpu_display`.
    fn render_cpu(&mut self, ui: &mut egui::Ui) {
        self.cpu_header.render(ui);
        ui.add_space(5.0);

        self.cpu_metric.render(ui);
        ui.label(format!("Cores: {}", self.cpu_display.core_count));

        ui.add(
            egui::ProgressBar::new(self.cpu_display.bar_fraction)
                .text(&self.cpu_display.label)
                .fill(self.cpu_display.bar_color),
        );

        if let Some(ref history_label) = self.cpu_display.history_label {
            ui.label(history_label);
            Self::render_sparkline(ui, &self.cpu_history, 100.0);
        }

        ui.add_space(10.0);
    }

    /// Thin render: memory section. Reads pre-computed `mem_display`.
    fn render_memory(&mut self, ui: &mut egui::Ui) {
        self.memory_header.render(ui);
        ui.add_space(5.0);

        self.memory_metric.render(ui);
        ui.label(&self.mem_display.used_gb_label);

        ui.add(
            egui::ProgressBar::new(self.mem_display.bar_fraction)
                .text(&self.mem_display.label)
                .fill(self.mem_display.bar_color),
        );

        if let Some(ref history_label) = self.mem_display.history_label {
            ui.label(history_label);
            Self::render_sparkline(ui, &self.mem_history, 100.0);
        }

        ui.add_space(10.0);
    }

    /// Render a sparkline from pre-computed points.
    fn render_sparkline(ui: &mut egui::Ui, data: &VecDeque<f32>, max_value: f32) {
        use egui::{Pos2, Stroke};

        let height = 50.0;
        let (response, painter) = ui.allocate_painter(
            egui::Vec2::new(ui.available_width(), height),
            egui::Sense::hover(),
        );

        let rect = response.rect;
        let raw_points = compute_sparkline_points(data, rect.width(), rect.height(), max_value);

        if raw_points.len() < 2 {
            return;
        }

        let points: Vec<Pos2> = raw_points
            .iter()
            .map(|[x, y]| Pos2::new(rect.left() + x, rect.top() + y))
            .collect();

        painter.add(egui::Shape::line(
            points,
            Stroke::new(2.0, Color32::from_rgb(100, 200, 255)),
        ));
    }
}

impl ToolPanel for SystemMonitorTool {
    fn metadata(&self) -> &ToolMetadata {
        static METADATA: std::sync::OnceLock<ToolMetadata> = std::sync::OnceLock::new();
        METADATA.get_or_init(|| ToolMetadata {
            name: "System Monitor".to_string(),
            description: "Real-time system resource monitoring".to_string(),
            version: "0.1.0".to_string(),
            capabilities: vec![
                ToolCapability::Visual,
                ToolCapability::Custom("RealTime".to_string()),
            ],
            icon: "📡".to_string(),
            source: Some(
                "https://www.kernel.org/doc/html/latest/filesystems/proc.html".to_string(),
            ),
        })
    }

    fn is_visible(&self) -> bool {
        self.show_panel
    }

    fn toggle_visibility(&mut self) {
        self.show_panel = !self.show_panel;
    }

    fn render_panel(&mut self, ui: &mut egui::Ui) {
        self.refresh();

        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading(egui::RichText::new("📡 System Monitor").size(24.0));
            ui.label(
                egui::RichText::new("Real-time system resource monitoring")
                    .size(14.0)
                    .color(Color32::GRAY),
            );
            ui.add_space(10.0);
        });

        ui.separator();
        ui.add_space(10.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Frame::none()
                .fill(Color32::from_rgb(30, 30, 35))
                .inner_margin(12.0)
                .show(ui, |ui| {
                    self.render_cpu(ui);
                    ui.separator();
                    self.render_memory(ui);
                });
        });

        ui.ctx().request_repaint();
    }

    fn status_message(&self) -> Option<String> {
        let cpu = self.last_cpu_usage;
        let mem = if self.stats.total_memory() > 0 {
            (self.stats.used_memory() as f64 / self.stats.total_memory() as f64) * 100.0
        } else {
            0.0
        };
        Some(format!("CPU: {cpu:.1}% | MEM: {mem:.1}%"))
    }
}

#[cfg(test)]
mod tool_tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn test_refresh_populates_cpu_history() {
        let mut tool = SystemMonitorTool {
            last_refresh: Instant::now().checked_sub(Duration::from_secs(2)).unwrap(),
            ..Default::default()
        };
        assert!(tool.cpu_history.is_empty());
        tool.refresh();
        assert_eq!(tool.cpu_history.len(), 1);
    }

    #[test]
    fn test_refresh_populates_mem_history() {
        let mut tool = SystemMonitorTool {
            last_refresh: Instant::now().checked_sub(Duration::from_secs(2)).unwrap(),
            ..Default::default()
        };
        assert!(tool.mem_history.is_empty());
        tool.refresh();
        assert_eq!(tool.mem_history.len(), 1);
    }

    #[test]
    fn test_refresh_respects_interval() {
        let mut tool = SystemMonitorTool::default();
        tool.refresh();
        assert!(
            tool.cpu_history.is_empty(),
            "Should not refresh immediately after construction"
        );
    }

    #[test]
    fn test_refresh_caps_history_length() {
        let mut tool = SystemMonitorTool {
            max_history: 3,
            last_refresh: Instant::now().checked_sub(Duration::from_secs(2)).unwrap(),
            ..Default::default()
        };

        for _ in 0..5 {
            tool.last_refresh = Instant::now().checked_sub(Duration::from_secs(2)).unwrap();
            tool.refresh();
        }

        assert!(tool.cpu_history.len() <= 3);
        assert!(tool.mem_history.len() <= 3);
    }

    #[test]
    fn test_refresh_rebuilds_display_states() {
        let mut tool = SystemMonitorTool {
            last_refresh: Instant::now().checked_sub(Duration::from_secs(2)).unwrap(),
            ..Default::default()
        };
        tool.refresh();
        assert!(tool.cpu_display().usage >= 0.0);
        assert!(tool.mem_display().percent >= 0.0);
    }

    #[test]
    fn test_default_max_history() {
        let tool = SystemMonitorTool::default();
        assert_eq!(tool.max_history, 60);
    }

    #[test]
    fn test_default_refresh_interval() {
        let tool = SystemMonitorTool::default();
        assert_eq!(tool.refresh_interval, Duration::from_secs(1));
    }
}
