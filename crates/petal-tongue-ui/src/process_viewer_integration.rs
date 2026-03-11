// SPDX-License-Identifier: AGPL-3.0-only
//! Process Viewer Integration
//!
//! Real-time process monitoring via /proc parsing (ecoBin v3.0 compliant).
//! Displays running processes with CPU, memory usage, and filtering.

#![allow(clippy::cast_precision_loss)]

use crate::proc_stats::{ProcStats, ProcessInfo as ProcProcessInfo};
use crate::tool_integration::{ToolCapability, ToolMetadata, ToolPanel};
use std::time::{Duration, Instant};

/// Process information for display (wraps proc_stats::ProcessInfo for tests)
#[derive(Clone, Debug)]
struct ProcessInfo {
    pid: u32,
    name: String,
    cpu_usage: f32,
    memory: u64, // in bytes
}

impl From<ProcProcessInfo> for ProcessInfo {
    fn from(p: ProcProcessInfo) -> Self {
        Self {
            pid: p.pid,
            name: p.name,
            cpu_usage: p.cpu_usage,
            memory: p.memory,
        }
    }
}

#[cfg(test)]
impl ProcessInfo {
    fn test_new(pid: u32, name: impl Into<String>, cpu_usage: f32, memory: u64) -> Self {
        Self {
            pid,
            name: name.into(),
            cpu_usage,
            memory,
        }
    }
}

/// Process Viewer tool integration
///
/// Provides real-time process monitoring with filtering and sorting.
/// Complements the System Monitor by showing per-process details.
pub struct ProcessViewerTool {
    show_panel: bool,
    stats: ProcStats,
    last_refresh: Instant,
    refresh_interval: Duration,
    processes: Vec<ProcessInfo>,
    filter_text: String,
    sort_by: SortColumn,
    max_display: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SortColumn {
    Name,
    Cpu,
    Memory,
}

fn filter_and_sort_processes(
    mut processes: Vec<ProcessInfo>,
    filter_text: &str,
    sort_by: SortColumn,
    max_display: usize,
) -> Vec<ProcessInfo> {
    if !filter_text.is_empty() {
        let filter_lower = filter_text.to_lowercase();
        processes.retain(|p| p.name.to_lowercase().contains(&filter_lower));
    }
    match sort_by {
        SortColumn::Name => processes.sort_by(|a, b| a.name.cmp(&b.name)),
        SortColumn::Cpu => processes.sort_by(|a, b| {
            b.cpu_usage
                .partial_cmp(&a.cpu_usage)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        SortColumn::Memory => processes.sort_by(|a, b| b.memory.cmp(&a.memory)),
    }
    processes.truncate(max_display);
    processes
}

impl Default for ProcessViewerTool {
    fn default() -> Self {
        Self {
            show_panel: false,
            stats: ProcStats::new(),
            last_refresh: Instant::now(),
            refresh_interval: Duration::from_secs(2), // 2 second refresh for processes
            processes: Vec::new(),
            filter_text: String::new(),
            sort_by: SortColumn::Cpu,
            max_display: 50, // Show top 50 processes
        }
    }
}

impl ProcessViewerTool {
    /// Refresh process information
    fn refresh(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_refresh) >= self.refresh_interval {
            let processes: Vec<ProcessInfo> = self
                .stats
                .processes()
                .into_iter()
                .map(ProcessInfo::from)
                .collect();

            self.processes = filter_and_sort_processes(
                processes,
                &self.filter_text,
                self.sort_by,
                self.max_display,
            );
            self.last_refresh = now;
        }
    }

    /// Render controls section
    fn render_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("🔍 Filter:");
            ui.text_edit_singleline(&mut self.filter_text);

            if ui.button("Clear").clicked() {
                self.filter_text.clear();
            }

            ui.separator();

            ui.label("Sort by:");
            ui.selectable_value(&mut self.sort_by, SortColumn::Name, "Name");
            ui.selectable_value(&mut self.sort_by, SortColumn::Cpu, "CPU");
            ui.selectable_value(&mut self.sort_by, SortColumn::Memory, "Memory");
        });

        ui.add_space(5.0);
    }

    /// Render process table
    fn render_process_table(&self, ui: &mut egui::Ui) {
        use egui_extras::{Column, TableBuilder};

        ui.label(format!("Showing {} processes", self.processes.len(),));

        ui.add_space(5.0);

        // Table with fixed header
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::exact(80.0)) // PID
            .column(Column::remainder()) // Name
            .column(Column::exact(80.0)) // CPU
            .column(Column::exact(100.0)) // Memory
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("PID");
                });
                header.col(|ui| {
                    ui.strong("Name");
                });
                header.col(|ui| {
                    ui.strong("CPU %");
                });
                header.col(|ui| {
                    ui.strong("Memory");
                });
            })
            .body(|mut body| {
                for process in &self.processes {
                    body.row(18.0, |mut row| {
                        row.col(|ui| {
                            ui.label(format!("{}", process.pid));
                        });
                        row.col(|ui| {
                            ui.label(&process.name);
                        });
                        row.col(|ui| {
                            let color = if process.cpu_usage > 50.0 {
                                egui::Color32::from_rgb(200, 50, 50)
                            } else if process.cpu_usage > 20.0 {
                                egui::Color32::from_rgb(200, 150, 50)
                            } else {
                                egui::Color32::LIGHT_GRAY
                            };
                            ui.colored_label(color, format!("{:.1}", process.cpu_usage));
                        });
                        row.col(|ui| {
                            let mb = process.memory as f64 / 1_048_576.0;
                            ui.label(format!("{mb:.1} MB"));
                        });
                    });
                }
            });
    }
}

impl ToolPanel for ProcessViewerTool {
    fn metadata(&self) -> &ToolMetadata {
        static METADATA: std::sync::OnceLock<ToolMetadata> = std::sync::OnceLock::new();
        METADATA.get_or_init(|| ToolMetadata {
            name: "Process Viewer".to_string(),
            description: "View and monitor running processes".to_string(),
            version: "0.1.0".to_string(),
            capabilities: vec![
                ToolCapability::Visual,
                ToolCapability::Custom("RealTime".to_string()),
                ToolCapability::Custom("Filtering".to_string()),
            ],
            icon: "📋".to_string(),
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
        // Refresh process data
        self.refresh();

        // Header
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.heading(egui::RichText::new("📋 Process Viewer").size(24.0));
            ui.label(
                egui::RichText::new("Monitor running processes with filtering and sorting")
                    .size(14.0)
                    .color(egui::Color32::GRAY),
            );
            ui.add_space(10.0);
        });

        ui.separator();
        ui.add_space(10.0);

        // Controls
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(30, 30, 35))
            .inner_margin(12.0)
            .show(ui, |ui| {
                self.render_controls(ui);
            });

        ui.add_space(10.0);

        // Process table in scroll area
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(25, 25, 30))
                .inner_margin(12.0)
                .show(ui, |ui| {
                    self.render_process_table(ui);
                });
        });

        // Request continuous repaint for live updates
        ui.ctx().request_repaint();
    }

    fn status_message(&self) -> Option<String> {
        let showing = self.processes.len();
        Some(format!("Processes: {showing}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn format_memory_mb(memory: u64) -> String {
        let mb = memory as f64 / 1_048_576.0;
        format!("{mb:.1} MB")
    }

    #[test]
    fn process_viewer_default() {
        let pv = ProcessViewerTool::default();
        assert!(!pv.show_panel);
        assert_eq!(pv.filter_text, String::new());
        assert_eq!(pv.sort_by, SortColumn::Cpu);
        assert_eq!(pv.max_display, 50);
    }

    #[test]
    fn process_info_creation() {
        let p = ProcessInfo::test_new(1234, "test_process", 5.5, 10_485_760);
        assert_eq!(p.pid, 1234);
        assert_eq!(p.name, "test_process");
        assert!((p.cpu_usage - 5.5).abs() < f32::EPSILON);
        assert_eq!(p.memory, 10_485_760);
    }

    #[test]
    fn format_memory_mb_display() {
        assert_eq!(format_memory_mb(0), "0.0 MB");
        assert_eq!(format_memory_mb(1_048_576), "1.0 MB");
        assert_eq!(format_memory_mb(10_485_760), "10.0 MB");
    }

    #[test]
    fn filter_processes_by_name() {
        let processes = vec![
            ProcessInfo::test_new(1, "chrome", 10.0, 100),
            ProcessInfo::test_new(2, "firefox", 5.0, 200),
            ProcessInfo::test_new(3, "chromium", 3.0, 150),
        ];
        let result = filter_and_sort_processes(processes, "chrom", SortColumn::Name, 10);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "chrome");
        assert_eq!(result[1].name, "chromium");
    }

    #[test]
    fn filter_case_insensitive() {
        let processes = vec![
            ProcessInfo::test_new(1, "Chrome", 10.0, 100),
            ProcessInfo::test_new(2, "FIREFOX", 5.0, 200),
        ];
        let result = filter_and_sort_processes(processes, "chrome", SortColumn::Name, 10);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Chrome");
    }

    #[test]
    fn sort_by_name() {
        let processes = vec![
            ProcessInfo::test_new(1, "zebra", 1.0, 100),
            ProcessInfo::test_new(2, "alpha", 1.0, 100),
            ProcessInfo::test_new(3, "middle", 1.0, 100),
        ];
        let result = filter_and_sort_processes(processes, "", SortColumn::Name, 10);
        assert_eq!(result[0].name, "alpha");
        assert_eq!(result[1].name, "middle");
        assert_eq!(result[2].name, "zebra");
    }

    #[test]
    fn sort_by_cpu_descending() {
        let processes = vec![
            ProcessInfo::test_new(1, "a", 5.0, 100),
            ProcessInfo::test_new(2, "b", 20.0, 100),
            ProcessInfo::test_new(3, "c", 10.0, 100),
        ];
        let result = filter_and_sort_processes(processes, "", SortColumn::Cpu, 10);
        assert_eq!(result[0].name, "b");
        assert_eq!(result[1].name, "c");
        assert_eq!(result[2].name, "a");
    }

    #[test]
    fn sort_by_memory_descending() {
        let processes = vec![
            ProcessInfo::test_new(1, "a", 1.0, 100),
            ProcessInfo::test_new(2, "b", 1.0, 500),
            ProcessInfo::test_new(3, "c", 1.0, 250),
        ];
        let result = filter_and_sort_processes(processes, "", SortColumn::Memory, 10);
        assert_eq!(result[0].memory, 500);
        assert_eq!(result[1].memory, 250);
        assert_eq!(result[2].memory, 100);
    }

    #[test]
    fn max_display_truncates() {
        let processes = vec![
            ProcessInfo::test_new(1, "a", 1.0, 100),
            ProcessInfo::test_new(2, "b", 2.0, 200),
            ProcessInfo::test_new(3, "c", 3.0, 300),
        ];
        let result = filter_and_sort_processes(processes, "", SortColumn::Cpu, 2);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn filter_empty_shows_all() {
        let processes = vec![
            ProcessInfo::test_new(1, "a", 1.0, 100),
            ProcessInfo::test_new(2, "b", 2.0, 200),
        ];
        let result = filter_and_sort_processes(processes, "", SortColumn::Name, 10);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn filter_no_match_returns_empty() {
        let processes = vec![
            ProcessInfo::test_new(1, "chrome", 10.0, 100),
            ProcessInfo::test_new(2, "firefox", 5.0, 200),
        ];
        let result = filter_and_sort_processes(processes, "xyz", SortColumn::Name, 10);
        assert!(result.is_empty());
    }

    #[test]
    fn status_message_format() {
        let pv = ProcessViewerTool::default();
        let msg = pv.status_message().expect("should have message");
        assert!(msg.starts_with("Processes: "));
        assert!(msg.contains('0'));
    }

    #[test]
    fn process_info_from_proc() {
        use crate::proc_stats::ProcessInfo as ProcProcessInfo;
        let proc_info = ProcProcessInfo {
            pid: 999,
            name: "test_proc".to_string(),
            cpu_usage: 12.5,
            memory: 2_097_152,
        };
        let info = ProcessInfo::from(proc_info);
        assert_eq!(info.pid, 999);
        assert_eq!(info.name, "test_proc");
        assert!((info.cpu_usage - 12.5).abs() < f32::EPSILON);
        assert_eq!(info.memory, 2_097_152);
    }

    #[test]
    fn sort_by_cpu_nan_safe() {
        let processes = vec![
            ProcessInfo::test_new(1, "a", f32::NAN, 100),
            ProcessInfo::test_new(2, "b", 5.0, 100),
        ];
        let result = filter_and_sort_processes(processes, "", SortColumn::Cpu, 10);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn process_viewer_tool_panel_metadata() {
        let pv = ProcessViewerTool::default();
        let meta = pv.metadata();
        assert_eq!(meta.name, "Process Viewer");
        assert!(meta.description.contains("process"));
        assert_eq!(meta.version, "0.1.0");
        assert!(meta.capabilities.len() >= 2);
        assert_eq!(meta.icon, "📋");
    }

    #[test]
    fn process_viewer_tool_panel_toggle() {
        let mut pv = ProcessViewerTool::default();
        assert!(!pv.is_visible());
        pv.toggle_visibility();
        assert!(pv.is_visible());
        pv.toggle_visibility();
        assert!(!pv.is_visible());
    }

    #[test]
    fn process_viewer_max_display_default() {
        let pv = ProcessViewerTool::default();
        assert_eq!(pv.max_display, 50);
    }

    #[test]
    fn process_viewer_refresh_interval() {
        let pv = ProcessViewerTool::default();
        assert_eq!(pv.refresh_interval, std::time::Duration::from_secs(2));
    }
}
