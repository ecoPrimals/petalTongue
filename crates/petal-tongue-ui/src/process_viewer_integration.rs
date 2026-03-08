// SPDX-License-Identifier: AGPL-3.0-only
//! Process Viewer Integration
//!
//! Real-time process monitoring using sysinfo.
//! Displays running processes with CPU, memory usage, and filtering.

#![allow(clippy::cast_precision_loss)]

use crate::tool_integration::{ToolCapability, ToolMetadata, ToolPanel};
use std::time::{Duration, Instant};
use sysinfo::{ProcessRefreshKind, System};

/// Process information for display
#[derive(Clone, Debug)]
struct ProcessInfo {
    pid: u32,
    name: String,
    cpu_usage: f32,
    memory: u64, // in bytes
}

/// Process Viewer tool integration
///
/// Provides real-time process monitoring with filtering and sorting.
/// Complements the System Monitor by showing per-process details.
pub struct ProcessViewerTool {
    show_panel: bool,
    system: System,
    last_refresh: Instant,
    refresh_interval: Duration,
    processes: Vec<ProcessInfo>,
    filter_text: String,
    sort_by: SortColumn,
    #[expect(dead_code)] // TODO: Add toggle for showing system processes
    show_all: bool, // Show all processes or only user processes
    max_display: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SortColumn {
    Name,
    Cpu,
    Memory,
}

impl Default for ProcessViewerTool {
    fn default() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self {
            show_panel: false,
            system,
            last_refresh: Instant::now(),
            refresh_interval: Duration::from_secs(2), // 2 second refresh for processes
            processes: Vec::new(),
            filter_text: String::new(),
            sort_by: SortColumn::Cpu,
            show_all: false,
            max_display: 50, // Show top 50 processes
        }
    }
}

impl ProcessViewerTool {
    /// Refresh process information
    fn refresh(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_refresh) >= self.refresh_interval {
            // Refresh processes
            self.system
                .refresh_processes_specifics(ProcessRefreshKind::everything());
            self.last_refresh = now;

            // Collect process info
            let mut processes: Vec<ProcessInfo> = self
                .system
                .processes()
                .iter()
                .map(|(pid, process)| ProcessInfo {
                    pid: pid.as_u32(),
                    name: process.name().to_string(),
                    cpu_usage: process.cpu_usage(),
                    memory: process.memory(),
                })
                .collect();

            // Apply filter
            if !self.filter_text.is_empty() {
                let filter_lower = self.filter_text.to_lowercase();
                processes.retain(|p| p.name.to_lowercase().contains(&filter_lower));
            }

            // Sort by selected column
            match self.sort_by {
                SortColumn::Name => processes.sort_by(|a, b| a.name.cmp(&b.name)),
                SortColumn::Cpu => processes.sort_by(|a, b| {
                    b.cpu_usage
                        .partial_cmp(&a.cpu_usage)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }),
                SortColumn::Memory => processes.sort_by(|a, b| b.memory.cmp(&a.memory)),
            }

            // Limit to max display
            processes.truncate(self.max_display);

            self.processes = processes;
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

        let total_processes = self.system.processes().len();

        ui.label(format!(
            "Showing {} of {} processes",
            self.processes.len(),
            total_processes
        ));

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
            source: Some("https://github.com/GuillaumeGomez/sysinfo".to_string()),
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
        let total = self.system.processes().len();
        let showing = self.processes.len();
        Some(format!("Processes: {showing}/{total}"))
    }
}
