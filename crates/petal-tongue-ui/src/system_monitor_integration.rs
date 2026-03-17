// SPDX-License-Identifier: AGPL-3.0-or-later
//! System Monitor Integration
//!
//! Real-time system resource monitoring via /proc parsing (ecoBin v3.0 compliant).
//! Demonstrates petalTongue integrating with external monitoring tool.
//! ALL DATA IS LIVE - timestamps and indicators prove it!
//!
//! Architecture: headless-first. All computation lives in pure functions
//! (`threshold_color`, `compute_sparkline_points`, `prepare_cpu_display`,
//! `prepare_memory_display`) that produce testable `DisplayState` structs.
//! Render methods are thin egui widget calls with zero logic.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use crate::live_data::{LiveGraphHeader, LiveMetric};
use crate::proc_stats::{ProcStats, SOURCE_ID};
use crate::tool_integration::{ToolCapability, ToolMetadata, ToolPanel};
use egui::Color32;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

// ============================================================================
// Display state types (headless-testable, no egui dependency)
// ============================================================================

/// Pre-computed CPU display data. Produced by `prepare_cpu_display()`,
/// consumed by `render_cpu()`. All formatting and threshold logic lives
/// in the producer so tests can validate without a UI context.
#[derive(Debug, Clone)]
pub struct CpuDisplayState {
    pub usage: f32,
    pub bar_fraction: f32,
    pub bar_color: Color32,
    pub label: String,
    pub core_count: usize,
    pub history_label: Option<String>,
}

/// Pre-computed memory display data.
#[derive(Debug, Clone)]
pub struct MemoryDisplayState {
    pub percent: f64,
    pub bar_fraction: f32,
    pub bar_color: Color32,
    pub label: String,
    pub used_gb_label: String,
    pub history_label: Option<String>,
}

// ============================================================================
// Pure functions (fully testable, no &self, no egui)
// ============================================================================

/// Map a metric value to a traffic-light color.
///
/// Returns red above `high`, amber above `mid`, and the provided `normal` color otherwise.
#[must_use]
pub fn threshold_color(value: f64, high: f64, mid: f64, normal: Color32) -> Color32 {
    if value > high {
        Color32::from_rgb(200, 50, 50)
    } else if value > mid {
        Color32::from_rgb(200, 150, 50)
    } else {
        normal
    }
}

/// Compute sparkline points in normalised coordinates.
///
/// Returns `Vec<[f32; 2]>` where x is in `[0, width]` and y is in `[0, height]`
/// (0 = top, height = bottom). The caller maps these into screen-space.
#[must_use]
pub fn compute_sparkline_points(
    data: &VecDeque<f32>,
    width: f32,
    height: f32,
    max_value: f32,
) -> Vec<[f32; 2]> {
    if data.len() < 2 {
        return Vec::new();
    }
    let len_minus_one = (data.len() - 1).max(1) as f32;
    data.iter()
        .enumerate()
        .map(|(i, &value)| {
            let x = (i as f32 / len_minus_one) * width;
            let y = (1.0 - (value / max_value).min(1.0)) * height;
            [x, y]
        })
        .collect()
}

/// Format bytes as GB with one decimal place.
#[must_use]
pub fn format_gb(bytes: u64) -> String {
    format!("{:.1}", bytes as f64 / 1_073_741_824.0)
}

/// Prepare CPU display state from current stats (pure computation).
#[must_use]
pub fn prepare_cpu_display(
    cpu_usage: f32,
    core_count: usize,
    history_len: usize,
) -> CpuDisplayState {
    CpuDisplayState {
        usage: cpu_usage,
        bar_fraction: cpu_usage / 100.0,
        bar_color: threshold_color(
            f64::from(cpu_usage),
            90.0,
            70.0,
            Color32::from_rgb(50, 150, 200),
        ),
        label: format!("{cpu_usage:.1}%"),
        core_count,
        history_label: if history_len > 0 {
            Some(format!("History ({history_len} samples) [LIVE DATA]"))
        } else {
            None
        },
    }
}

/// Prepare memory display state from current stats (pure computation).
#[must_use]
pub fn prepare_memory_display(
    used_bytes: u64,
    total_bytes: u64,
    history_len: usize,
) -> MemoryDisplayState {
    let percent = if total_bytes > 0 {
        (used_bytes as f64 / total_bytes as f64) * 100.0
    } else {
        0.0
    };
    MemoryDisplayState {
        percent,
        bar_fraction: percent as f32 / 100.0,
        bar_color: threshold_color(percent, 90.0, 70.0, Color32::from_rgb(50, 200, 150)),
        label: format!("{percent:.1}%"),
        used_gb_label: format!(
            "Used: {} / {} GB",
            format_gb(used_bytes),
            format_gb(total_bytes)
        ),
        history_label: if history_len > 0 {
            Some(format!("History ({history_len} samples) [LIVE DATA]"))
        } else {
            None
        },
    }
}

// ============================================================================
// SystemMonitorTool
// ============================================================================

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
mod tests {
    use super::*;

    // === Pure function tests ===

    #[test]
    fn threshold_color_red_above_high() {
        let c = threshold_color(95.0, 90.0, 70.0, Color32::BLUE);
        assert_eq!(c, Color32::from_rgb(200, 50, 50));
    }

    #[test]
    fn threshold_color_amber_above_mid() {
        let c = threshold_color(75.0, 90.0, 70.0, Color32::BLUE);
        assert_eq!(c, Color32::from_rgb(200, 150, 50));
    }

    #[test]
    fn threshold_color_normal_below_mid() {
        let normal = Color32::from_rgb(50, 150, 200);
        let c = threshold_color(50.0, 90.0, 70.0, normal);
        assert_eq!(c, normal);
    }

    #[test]
    fn threshold_color_boundary_at_high() {
        let c = threshold_color(90.0, 90.0, 70.0, Color32::BLUE);
        assert_eq!(c, Color32::from_rgb(200, 150, 50), "boundary = mid color");
    }

    #[test]
    fn threshold_color_boundary_at_mid() {
        let c = threshold_color(70.0, 90.0, 70.0, Color32::BLUE);
        assert_eq!(c, Color32::BLUE, "boundary = normal color");
    }

    #[test]
    fn sparkline_points_empty_data() {
        let data = VecDeque::new();
        assert!(compute_sparkline_points(&data, 100.0, 50.0, 100.0).is_empty());
    }

    #[test]
    fn sparkline_points_single_datum() {
        let mut data = VecDeque::new();
        data.push_back(50.0);
        assert!(
            compute_sparkline_points(&data, 100.0, 50.0, 100.0).is_empty(),
            "need >= 2 points"
        );
    }

    #[test]
    fn sparkline_points_two_data() {
        let mut data = VecDeque::new();
        data.push_back(0.0);
        data.push_back(100.0);
        let pts = compute_sparkline_points(&data, 200.0, 100.0, 100.0);
        assert_eq!(pts.len(), 2);
        assert!((pts[0][0]).abs() < f32::EPSILON, "first x = 0");
        assert!(
            (pts[0][1] - 100.0).abs() < f32::EPSILON,
            "0% value = bottom"
        );
        assert!((pts[1][0] - 200.0).abs() < f32::EPSILON, "last x = width");
        assert!((pts[1][1]).abs() < f32::EPSILON, "100% value = top");
    }

    #[test]
    fn sparkline_points_mid_value() {
        let mut data = VecDeque::new();
        data.push_back(50.0);
        data.push_back(50.0);
        let pts = compute_sparkline_points(&data, 100.0, 100.0, 100.0);
        assert_eq!(pts.len(), 2);
        assert!((pts[0][1] - 50.0).abs() < f32::EPSILON, "50% = midpoint");
    }

    #[test]
    fn sparkline_clamps_above_max() {
        let mut data = VecDeque::new();
        data.push_back(200.0);
        data.push_back(200.0);
        let pts = compute_sparkline_points(&data, 100.0, 100.0, 100.0);
        assert!((pts[0][1]).abs() < f32::EPSILON, "clamped to top");
    }

    #[test]
    fn format_gb_zero() {
        assert_eq!(format_gb(0), "0.0");
    }

    #[test]
    fn format_gb_one_gb() {
        assert_eq!(format_gb(1_073_741_824), "1.0");
    }

    #[test]
    fn format_gb_sixteen_gb() {
        assert_eq!(format_gb(16 * 1_073_741_824), "16.0");
    }

    #[test]
    fn format_gb_fractional() {
        assert_eq!(format_gb(1_610_612_736), "1.5");
    }

    #[test]
    fn threshold_color_zero() {
        let normal = Color32::from_rgb(100, 100, 100);
        let c = threshold_color(0.0, 90.0, 70.0, normal);
        assert_eq!(c, normal);
    }

    #[test]
    fn sparkline_points_three_data() {
        let mut data = VecDeque::new();
        data.push_back(0.0);
        data.push_back(50.0);
        data.push_back(100.0);
        let pts = compute_sparkline_points(&data, 100.0, 50.0, 100.0);
        assert_eq!(pts.len(), 3);
        assert!((pts[1][0] - 50.0).abs() < f32::EPSILON);
    }

    // === Display state tests ===

    #[test]
    fn cpu_display_state_low_usage() {
        let state = prepare_cpu_display(25.0, 8, 10);
        assert_eq!(state.usage, 25.0);
        assert!((state.bar_fraction - 0.25).abs() < f32::EPSILON);
        assert_eq!(state.bar_color, Color32::from_rgb(50, 150, 200));
        assert_eq!(state.label, "25.0%");
        assert_eq!(state.core_count, 8);
        assert_eq!(
            state.history_label.as_deref(),
            Some("History (10 samples) [LIVE DATA]")
        );
    }

    #[test]
    fn cpu_display_state_high_usage() {
        let state = prepare_cpu_display(95.0, 4, 0);
        assert_eq!(state.bar_color, Color32::from_rgb(200, 50, 50));
        assert!(state.history_label.is_none());
    }

    #[test]
    fn cpu_display_state_mid_usage() {
        let state = prepare_cpu_display(80.0, 4, 5);
        assert_eq!(state.bar_color, Color32::from_rgb(200, 150, 50));
    }

    #[test]
    fn memory_display_state_normal() {
        let total = 16 * 1_073_741_824_u64;
        let used = 4 * 1_073_741_824_u64;
        let state = prepare_memory_display(used, total, 30);
        assert!((state.percent - 25.0).abs() < 0.01);
        assert_eq!(state.bar_color, Color32::from_rgb(50, 200, 150));
        assert!(state.used_gb_label.contains("4.0"));
        assert!(state.used_gb_label.contains("16.0"));
        assert!(state.history_label.is_some());
    }

    #[test]
    fn memory_display_state_critical() {
        let total = 16 * 1_073_741_824_u64;
        let used = 15 * 1_073_741_824_u64;
        let state = prepare_memory_display(used, total, 0);
        assert_eq!(state.bar_color, Color32::from_rgb(200, 50, 50));
        assert!(state.history_label.is_none());
    }

    #[test]
    fn memory_display_state_zero_total() {
        let state = prepare_memory_display(0, 0, 0);
        assert!((state.percent).abs() < f64::EPSILON);
        assert_eq!(state.label, "0.0%");
    }

    // === Integration tests (tool-level) ===

    #[test]
    fn test_system_monitor_default() {
        let tool = SystemMonitorTool::default();
        assert!(!tool.is_visible());
        let meta = tool.metadata();
        assert_eq!(meta.name, "System Monitor");
        assert!(meta.description.contains("Real-time"));
        assert!(meta.capabilities.contains(&ToolCapability::Visual));
        assert!(
            meta.capabilities
                .contains(&ToolCapability::Custom("RealTime".to_string()))
        );
    }

    #[test]
    fn test_system_monitor_toggle_visibility() {
        let mut tool = SystemMonitorTool::default();
        assert!(!tool.is_visible());
        tool.toggle_visibility();
        assert!(tool.is_visible());
        tool.toggle_visibility();
        assert!(!tool.is_visible());
    }

    #[test]
    fn test_system_monitor_status_message() {
        let tool = SystemMonitorTool::default();
        let msg = tool.status_message();
        assert!(msg.is_some());
        let msg = msg.unwrap();
        assert!(msg.contains("CPU:"));
        assert!(msg.contains("MEM:"));
        assert!(msg.contains("%"));
    }

    #[test]
    fn test_system_monitor_metadata_source() {
        let tool = SystemMonitorTool::default();
        let meta = tool.metadata();
        assert_eq!(meta.icon, "📡");
        assert!(meta.source.is_some());
        assert!(meta.source.as_ref().unwrap().contains("kernel.org"));
    }

    #[test]
    fn test_system_monitor_metadata_version() {
        let tool = SystemMonitorTool::default();
        let meta = tool.metadata();
        assert_eq!(meta.version, "0.1.0");
    }

    #[test]
    fn test_system_monitor_toggle_multiple() {
        let mut tool = SystemMonitorTool::default();
        for _ in 0..4 {
            tool.toggle_visibility();
        }
        assert!(!tool.is_visible());
    }

    #[test]
    fn test_system_monitor_status_message_format() {
        let tool = SystemMonitorTool::default();
        let msg = tool.status_message().expect("status message");
        assert!(msg.starts_with("CPU:"));
        assert!(msg.contains("MEM:"));
        assert!(msg.contains('%'));
        let parts: Vec<&str> = msg.split('|').collect();
        assert_eq!(parts.len(), 2);
    }

    #[test]
    fn test_system_monitor_metadata_capabilities_count() {
        let tool = SystemMonitorTool::default();
        let meta = tool.metadata();
        assert!(meta.capabilities.len() >= 2);
    }

    #[test]
    fn test_refresh_populates_cpu_history() {
        let mut tool = SystemMonitorTool {
            last_refresh: Instant::now() - Duration::from_secs(2),
            ..Default::default()
        };
        assert!(tool.cpu_history.is_empty());
        tool.refresh();
        assert_eq!(tool.cpu_history.len(), 1);
    }

    #[test]
    fn test_refresh_populates_mem_history() {
        let mut tool = SystemMonitorTool {
            last_refresh: Instant::now() - Duration::from_secs(2),
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
            last_refresh: Instant::now() - Duration::from_secs(2),
            ..Default::default()
        };

        for _ in 0..5 {
            tool.last_refresh = Instant::now() - Duration::from_secs(2);
            tool.refresh();
        }

        assert!(tool.cpu_history.len() <= 3);
        assert!(tool.mem_history.len() <= 3);
    }

    #[test]
    fn test_refresh_rebuilds_display_states() {
        let mut tool = SystemMonitorTool {
            last_refresh: Instant::now() - Duration::from_secs(2),
            ..Default::default()
        };
        tool.refresh();
        assert!(tool.cpu_display.usage >= 0.0);
        assert!(tool.mem_display.percent >= 0.0);
    }

    #[test]
    fn test_cpu_display_accessor() {
        let tool = SystemMonitorTool::default();
        let display = tool.cpu_display();
        assert_eq!(display.label, "0.0%");
    }

    #[test]
    fn test_mem_display_accessor() {
        let tool = SystemMonitorTool::default();
        let display = tool.mem_display();
        assert_eq!(display.label, "0.0%");
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

    #[test]
    fn test_status_message_zero_memory() {
        let tool = SystemMonitorTool::default();
        let msg = tool.status_message();
        assert!(msg.is_some());
        let msg = msg.unwrap();
        assert!(msg.contains("CPU:"));
        assert!(msg.contains("MEM:"));
    }

    #[test]
    fn test_prepare_memory_display_amber_threshold() {
        let total = 16 * 1_073_741_824_u64;
        let used = 12 * 1_073_741_824_u64;
        let state = prepare_memory_display(used, total, 5);
        assert_eq!(state.bar_color, Color32::from_rgb(200, 150, 50));
    }

    #[test]
    fn test_prepare_cpu_display_amber_threshold() {
        let state = prepare_cpu_display(75.0, 8, 20);
        assert_eq!(state.bar_color, Color32::from_rgb(200, 150, 50));
    }

    #[test]
    fn test_threshold_color_exactly_mid() {
        let c = threshold_color(70.0, 90.0, 70.0, Color32::GREEN);
        assert_eq!(c, Color32::GREEN);
    }

    #[test]
    fn test_threshold_color_exactly_high() {
        let c = threshold_color(91.0, 90.0, 70.0, Color32::BLUE);
        assert_eq!(c, Color32::from_rgb(200, 50, 50));
    }

    #[test]
    fn test_sparkline_points_four_data() {
        let mut data = VecDeque::new();
        data.push_back(0.0);
        data.push_back(25.0);
        data.push_back(50.0);
        data.push_back(100.0);
        let pts = compute_sparkline_points(&data, 300.0, 100.0, 100.0);
        assert_eq!(pts.len(), 4);
        assert!((pts[0][0]).abs() < f32::EPSILON);
        assert!((pts[3][0] - 300.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_format_gb_small() {
        assert_eq!(format_gb(107_374_182), "0.1");
    }

    #[test]
    fn test_memory_display_state_amber() {
        let total = 16 * 1_073_741_824_u64;
        let used = 12 * 1_073_741_824_u64;
        let state = prepare_memory_display(used, total, 10);
        assert!((state.percent - 75.0).abs() < 0.01);
        assert_eq!(state.bar_color, Color32::from_rgb(200, 150, 50));
    }

    #[test]
    fn test_cpu_display_state_no_history() {
        let state = prepare_cpu_display(50.0, 4, 0);
        assert!(state.history_label.is_none());
    }

    #[test]
    fn test_metadata_icon() {
        let tool = SystemMonitorTool::default();
        let meta = tool.metadata();
        assert!(!meta.icon.is_empty());
    }

    #[test]
    fn test_metadata_name() {
        let tool = SystemMonitorTool::default();
        let meta = tool.metadata();
        assert_eq!(meta.name, "System Monitor");
    }
}
