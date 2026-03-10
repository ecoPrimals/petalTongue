// SPDX-License-Identifier: AGPL-3.0-only
//! Timeline View - Core view struct and rendering logic
//!
//! Displays temporal sequences of primal interactions with time scrubbing capabilities.
//! Implements Phase 4 of the UI specification.

use chrono::{DateTime, Utc};
use egui::{Pos2, Rect, Stroke, Vec2};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use super::filtering::{filtered_events, get_primals};
use super::types::TimelineEvent;

/// Timeline View - Sequence diagram of primal interactions
pub struct TimelineView {
    /// Events to display
    events: Vec<TimelineEvent>,
    /// Selected event (for detail panel)
    selected_event: Option<String>,
    /// Time range start (None = auto)
    time_range_start: Option<DateTime<Utc>>,
    /// Time range end (None = auto)
    time_range_end: Option<DateTime<Utc>>,
    /// Zoom level (1.0 = default)
    zoom: f32,
    /// Show event details panel
    show_details: bool,
    /// Filter by event type
    event_type_filter: Option<String>,
    /// Filter by primal
    primal_filter: Option<String>,
}

impl Default for TimelineView {
    fn default() -> Self {
        Self::new()
    }
}

impl TimelineView {
    /// Create a new timeline view
    #[must_use]
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            selected_event: None,
            time_range_start: None,
            time_range_end: None,
            zoom: 1.0,
            show_details: true,
            event_type_filter: None,
            primal_filter: None,
        }
    }

    /// Add an event to the timeline
    pub fn add_event(&mut self, event: TimelineEvent) {
        self.events.push(event);
        // Sort by timestamp
        self.events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    }

    /// Clear all events
    pub fn clear(&mut self) {
        self.events.clear();
        self.selected_event = None;
    }

    /// Set time range for display
    pub fn set_time_range(&mut self, start: Option<DateTime<Utc>>, end: Option<DateTime<Utc>>) {
        self.time_range_start = start;
        self.time_range_end = end;
    }

    /// Set event type filter (for testing and UI)
    pub fn set_event_type_filter(&mut self, filter: Option<String>) {
        self.event_type_filter = filter;
    }

    /// Set primal filter (for testing and UI)
    pub fn set_primal_filter(&mut self, filter: Option<String>) {
        self.primal_filter = filter;
    }

    /// Get count of filtered events (for testing)
    #[must_use]
    pub fn filtered_event_count(&self) -> usize {
        self.filtered_events().len()
    }

    fn filtered_events(&self) -> Vec<&TimelineEvent> {
        filtered_events(
            &self.events,
            &self.event_type_filter,
            &self.primal_filter,
            &self.time_range_start,
            &self.time_range_end,
        )
    }

    fn get_primals(&self) -> Vec<String> {
        get_primals(&self.events)
    }

    /// Get primals (for testing).
    #[doc(hidden)]
    #[must_use]
    pub fn get_primals_for_test(&self) -> Vec<String> {
        get_primals(&self.events)
    }

    /// Get event IDs in timestamp order (for testing).
    #[doc(hidden)]
    #[must_use]
    pub fn event_ids_ordered(&self) -> Vec<String> {
        self.events.iter().map(|e| e.id.clone()).collect()
    }

    /// Render the timeline view
    pub fn render(&mut self, ui: &mut egui::Ui) {
        // Top control bar
        ui.horizontal(|ui| {
            ui.heading("📊 Timeline View");
            ui.separator();

            // Zoom controls
            ui.label("Zoom:");
            if ui.button("➖").clicked() {
                self.zoom = (self.zoom / 1.2).max(0.1);
            }
            ui.label(format!("{:.0}%", self.zoom * 100.0));
            if ui.button("➕").clicked() {
                self.zoom = (self.zoom * 1.2).min(10.0);
            }

            ui.separator();

            // Event count
            let filtered_count = self.filtered_events().len();
            let total_count = self.events.len();
            ui.label(format!("Events: {filtered_count}/{total_count}"));

            ui.separator();

            // Toggle details panel
            if ui
                .button(if self.show_details {
                    "Hide Details"
                } else {
                    "Show Details"
                })
                .clicked()
            {
                self.show_details = !self.show_details;
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Clear").clicked() {
                    self.clear();
                }
                if ui.button("Export CSV").clicked() {
                    self.export_csv();
                }
            });
        });

        ui.separator();

        // Main layout: Timeline + Details (if enabled)
        if self.show_details && self.selected_event.is_some() {
            ui.horizontal(|ui| {
                // Timeline (left side - 70%)
                ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width() * 0.7, ui.available_height()),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        self.render_timeline(ui);
                    },
                );

                ui.separator();

                // Details panel (right side - 30%)
                ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width(), ui.available_height()),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        self.render_details_panel(ui);
                    },
                );
            });
        } else {
            // Full-width timeline
            self.render_timeline(ui);
        }
    }

    /// Render the main timeline visualization
    fn render_timeline(&mut self, ui: &mut egui::Ui) {
        let available_size = ui.available_size();
        let (response, painter) = ui.allocate_painter(available_size, egui::Sense::click());

        let rect = response.rect;
        painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(20, 20, 25));

        let primals = self.get_primals();
        if primals.is_empty() {
            // No events yet
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "No events to display",
                egui::FontId::proportional(16.0),
                egui::Color32::GRAY,
            );
            return;
        }

        // Calculate layout
        let primal_count = primals.len();
        let lane_height = rect.height() / (primal_count as f32 + 1.0);
        let time_width = rect.width() - 100.0; // Reserve 100px for primal labels

        // Build primal -> lane mapping
        let primal_lanes: HashMap<_, _> = primals
            .iter()
            .enumerate()
            .map(|(i, p)| (p.clone(), i))
            .collect();

        // Draw primal lanes (horizontal lines)
        for (idx, primal) in primals.iter().enumerate() {
            let y = rect.min.y + lane_height * (idx as f32 + 1.0);

            // Lane line
            painter.line_segment(
                [Pos2::new(rect.min.x + 100.0, y), Pos2::new(rect.max.x, y)],
                Stroke::new(1.0, egui::Color32::from_rgb(50, 50, 60)),
            );

            // Primal label
            painter.text(
                Pos2::new(rect.min.x + 50.0, y),
                egui::Align2::CENTER_CENTER,
                primal,
                egui::FontId::monospace(12.0),
                egui::Color32::WHITE,
            );
        }

        // Draw events
        let filtered_events = self.filtered_events();
        if !filtered_events.is_empty() {
            // Calculate time range
            let min_time = filtered_events
                .first()
                .expect("checked non-empty above")
                .timestamp;
            let max_time = filtered_events
                .last()
                .expect("checked non-empty above")
                .timestamp;
            let time_range = (max_time - min_time).num_milliseconds().max(1) as f64;

            // Collect event IDs for click detection
            let mut clicked_event_id: Option<String> = None;

            for event in filtered_events {
                if let (Some(from_lane), Some(to_lane)) =
                    (primal_lanes.get(&event.from), primal_lanes.get(&event.to))
                {
                    // Calculate time position
                    let time_offset = (event.timestamp - min_time).num_milliseconds() as f64;
                    let time_fraction = time_offset / time_range;
                    let x = rect.min.x + 100.0 + time_width * time_fraction as f32;

                    let from_y = rect.min.y + lane_height * (*from_lane as f32 + 1.0);
                    let to_y = rect.min.y + lane_height * (*to_lane as f32 + 1.0);

                    // Draw event arrow
                    let from_pos = Pos2::new(x, from_y);
                    let to_pos = Pos2::new(x, to_y);

                    painter
                        .line_segment([from_pos, to_pos], Stroke::new(2.0, event.status.color()));

                    // Draw event marker (circle)
                    painter.circle_filled(from_pos, 4.0, event.status.color());
                    painter.circle_filled(to_pos, 4.0, event.status.color());

                    // Check for click
                    let click_rect = Rect::from_center_size(from_pos, Vec2::splat(10.0));
                    if response.clicked()
                        && let Some(pointer_pos) = response.interact_pointer_pos()
                        && click_rect.contains(pointer_pos)
                    {
                        clicked_event_id = Some(event.id.clone());
                    }
                }
            }

            // Apply click after borrowing events
            if let Some(event_id) = clicked_event_id {
                self.selected_event = Some(event_id);
            }
        }
    }

    /// Render event details panel
    fn render_details_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Event Details");
        ui.separator();

        if let Some(ref event_id) = self.selected_event {
            if let Some(event) = self.events.iter().find(|e| &e.id == event_id) {
                let event_from = event.from.clone();
                let event_to = event.to.clone();
                let event_type = event.event_type.clone();
                let event_timestamp = event.timestamp;
                let event_duration = event.duration_ms;
                let event_payload = event.payload_summary.clone();
                let event_status = event.status.clone();

                egui::Grid::new("event_details_grid")
                    .num_columns(2)
                    .spacing([10.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Status:");
                        ui.horizontal(|ui| {
                            ui.label(event_status.icon());
                            ui.label(format!("{event_status:?}"));
                        });
                        ui.end_row();

                        ui.label("From:");
                        ui.label(&event_from);
                        ui.end_row();

                        ui.label("To:");
                        ui.label(&event_to);
                        ui.end_row();

                        ui.label("Type:");
                        ui.label(&event_type);
                        ui.end_row();

                        ui.label("Time:");
                        ui.label(event_timestamp.format("%H:%M:%S%.3f").to_string());
                        ui.end_row();

                        if let Some(duration) = event_duration {
                            ui.label("Duration:");
                            ui.label(format!("{duration:.2}ms"));
                            ui.end_row();
                        }

                        if let Some(ref payload) = event_payload {
                            ui.label("Payload:");
                            ui.label(payload);
                            ui.end_row();
                        }
                    });

                ui.add_space(16.0);

                if ui.button("Close Details").clicked() {
                    self.selected_event = None;
                }
            }
        } else {
            ui.label("Click on an event to see details");
        }
    }

    /// Export events to CSV format
    ///
    /// Writes to `{XDG_DATA_HOME}/petalTongue/exports/timeline_events.csv`
    /// or temp dir if platform_dirs is unavailable.
    fn export_csv(&self) {
        let path = self.export_csv_path();
        let events = self.filtered_events();

        match self.write_events_csv(&path, events) {
            Ok(()) => tracing::info!("CSV exported to {}", path.display()),
            Err(e) => tracing::error!("CSV export failed: {}", e),
        }
    }

    fn export_csv_path(&self) -> PathBuf {
        petal_tongue_core::platform_dirs::data_dir().map_or_else(
            |_| std::env::temp_dir().join("petalTongue_timeline_events.csv"),
            |d| {
                d.join("petalTongue")
                    .join("exports")
                    .join("timeline_events.csv")
            },
        )
    }

    fn write_events_csv(&self, path: &PathBuf, events: Vec<&TimelineEvent>) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut f = File::create(path)?;

        // Header
        writeln!(
            f,
            "id,from,to,event_type,timestamp,duration_ms,status,payload_summary"
        )?;

        for event in events {
            let duration = event.duration_ms.map(|d| d.to_string()).unwrap_or_default();
            let payload = event.payload_summary.as_deref().unwrap_or("");
            writeln!(
                f,
                "{},{},{},{},{},{},{},{}",
                escape_csv(&event.id),
                escape_csv(&event.from),
                escape_csv(&event.to),
                escape_csv(&event.event_type),
                escape_csv(&event.timestamp.to_rfc3339()),
                escape_csv(&duration),
                escape_csv(&format!("{:?}", event.status)),
                escape_csv(payload),
            )?;
        }

        Ok(())
    }
}

/// Escape a CSV field (RFC 4180): wrap in quotes if contains comma, quote, or newline
fn escape_csv(s: &str) -> String {
    let needs_quotes = s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r');
    if needs_quotes {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
