// SPDX-License-Identifier: AGPL-3.0-only
//! Timeline View - Core view struct and rendering logic
//!
//! Displays temporal sequences of primal interactions with time scrubbing capabilities.
//! Implements Phase 4 of the UI specification.
//!
//! Architecture: headless-first. Pure functions live in `helpers`. The render method returns
//! `Vec<TimelineIntent>` rather than mutating state directly.

use chrono::{DateTime, Utc};
use egui::{Pos2, Rect, Stroke, Vec2};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use super::filtering::{filtered_events, get_primals};
use super::helpers::{
    build_primal_lanes, compute_lane_height, event_screen_rect, format_events_csv,
    prepare_event_detail, time_to_x, zoom_in, zoom_out,
};
use super::types::{TimelineEvent, TimelineIntent};

/// Timeline View - Sequence diagram of primal interactions
pub struct TimelineView {
    events: Vec<TimelineEvent>,
    selected_event: Option<String>,
    time_range_start: Option<DateTime<Utc>>,
    time_range_end: Option<DateTime<Utc>>,
    zoom: f32,
    show_details: bool,
    event_type_filter: Option<String>,
    primal_filter: Option<String>,
}

impl Default for TimelineView {
    fn default() -> Self {
        Self::new()
    }
}

impl TimelineView {
    #[must_use]
    pub const fn new() -> Self {
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

    pub fn add_event(&mut self, event: TimelineEvent) {
        self.events.push(event);
        self.events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    }

    pub fn clear(&mut self) {
        self.events.clear();
        self.selected_event = None;
    }

    pub const fn set_time_range(
        &mut self,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) {
        self.time_range_start = start;
        self.time_range_end = end;
    }

    pub fn set_event_type_filter(&mut self, filter: Option<String>) {
        self.event_type_filter = filter;
    }

    pub fn set_primal_filter(&mut self, filter: Option<String>) {
        self.primal_filter = filter;
    }

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

    #[cfg(test)]
    #[doc(hidden)]
    #[must_use]
    pub fn filtered_events_for_test(&self) -> Vec<&TimelineEvent> {
        self.filtered_events()
    }

    fn get_primals(&self) -> Vec<String> {
        get_primals(&self.events)
    }

    #[doc(hidden)]
    #[must_use]
    pub fn get_primals_for_test(&self) -> Vec<String> {
        get_primals(&self.events)
    }

    #[doc(hidden)]
    #[must_use]
    pub fn event_ids_ordered(&self) -> Vec<String> {
        self.events.iter().map(|e| e.id.clone()).collect()
    }

    /// Read-only access to the currently selected event ID (for headless testing).
    #[must_use]
    pub fn selected_event(&self) -> Option<&str> {
        self.selected_event.as_deref()
    }

    /// Current zoom level (for headless testing).
    #[must_use]
    pub const fn zoom(&self) -> f32 {
        self.zoom
    }

    #[cfg(test)]
    #[must_use]
    pub const fn show_details(&self) -> bool {
        self.show_details
    }

    /// Apply intents produced by `render()`. Call after render returns.
    pub fn apply_intents(&mut self, intents: &[TimelineIntent]) {
        for intent in intents {
            match intent {
                TimelineIntent::ZoomIn => self.zoom = zoom_in(self.zoom),
                TimelineIntent::ZoomOut => self.zoom = zoom_out(self.zoom),
                TimelineIntent::ToggleDetails => self.show_details = !self.show_details,
                TimelineIntent::SelectEvent(id) => self.selected_event = Some(id.clone()),
                TimelineIntent::DeselectEvent => self.selected_event = None,
                TimelineIntent::Clear => self.clear(),
                TimelineIntent::ExportCsv => self.export_csv(),
            }
        }
    }

    /// Render the timeline view. Returns intents for the caller to apply.
    pub fn render(&mut self, ui: &mut egui::Ui) -> Vec<TimelineIntent> {
        let mut intents = Vec::new();

        ui.horizontal(|ui| {
            ui.heading("📊 Timeline View");
            ui.separator();

            ui.label("Zoom:");
            if ui.button("➖").clicked() {
                intents.push(TimelineIntent::ZoomOut);
            }
            ui.label(format!("{:.0}%", self.zoom * 100.0));
            if ui.button("➕").clicked() {
                intents.push(TimelineIntent::ZoomIn);
            }

            ui.separator();

            let filtered_count = self.filtered_events().len();
            let total_count = self.events.len();
            ui.label(format!("Events: {filtered_count}/{total_count}"));

            ui.separator();

            if ui
                .button(if self.show_details {
                    "Hide Details"
                } else {
                    "Show Details"
                })
                .clicked()
            {
                intents.push(TimelineIntent::ToggleDetails);
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Clear").clicked() {
                    intents.push(TimelineIntent::Clear);
                }
                if ui.button("Export CSV").clicked() {
                    intents.push(TimelineIntent::ExportCsv);
                }
            });
        });

        ui.separator();

        if self.show_details && self.selected_event.is_some() {
            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width() * 0.7, ui.available_height()),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        let diagram_intents = self.render_timeline(ui);
                        intents.extend(diagram_intents);
                    },
                );

                ui.separator();

                ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width(), ui.available_height()),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        let panel_intents = self.render_details_panel(ui);
                        intents.extend(panel_intents);
                    },
                );
            });
        } else {
            let diagram_intents = self.render_timeline(ui);
            intents.extend(diagram_intents);
        }

        intents
    }

    fn render_timeline(&self, ui: &mut egui::Ui) -> Vec<TimelineIntent> {
        let mut intents = Vec::new();
        let available_size = ui.available_size();
        let (response, painter) = ui.allocate_painter(available_size, egui::Sense::click());

        let rect = response.rect;
        painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(20, 20, 25));

        let primals = self.get_primals();
        if primals.is_empty() {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "No events to display",
                egui::FontId::proportional(16.0),
                egui::Color32::GRAY,
            );
            return intents;
        }

        let primal_count = primals.len();
        let lane_height = compute_lane_height(rect.height(), primal_count);
        let time_width = rect.width() - 100.0;
        let primal_lanes = build_primal_lanes(&self.events);

        for (idx, primal) in primals.iter().enumerate() {
            let y = lane_height.mul_add(idx as f32 + 1.0, rect.min.y);

            painter.line_segment(
                [Pos2::new(rect.min.x + 100.0, y), Pos2::new(rect.max.x, y)],
                Stroke::new(1.0, egui::Color32::from_rgb(50, 50, 60)),
            );

            painter.text(
                Pos2::new(rect.min.x + 50.0, y),
                egui::Align2::CENTER_CENTER,
                primal,
                egui::FontId::monospace(12.0),
                egui::Color32::WHITE,
            );
        }

        let filtered_events = self.filtered_events();
        let (Some(first), Some(last)) = (filtered_events.first(), filtered_events.last()) else {
            return intents;
        };
        let min_time = first.timestamp;
        let max_time = last.timestamp;

        let start_ms = min_time.timestamp_millis() as f64;
        let end_ms = max_time.timestamp_millis() as f64;
        let rect_min = (rect.min.x, rect.min.y);

        for event in filtered_events {
            if let (Some(from_lane), Some(to_lane)) =
                (primal_lanes.get(&event.from), primal_lanes.get(&event.to))
            {
                let time_ms = event.timestamp.timestamp_millis() as f64;
                let x = rect.min.x + 100.0 + time_to_x(time_ms, start_ms, end_ms, time_width);

                let from_y = lane_height.mul_add(*from_lane as f32 + 1.0, rect.min.y);
                let to_y = lane_height.mul_add(*to_lane as f32 + 1.0, rect.min.y);

                let from_pos = Pos2::new(x, from_y);
                let to_pos = Pos2::new(x, to_y);

                painter.line_segment([from_pos, to_pos], Stroke::new(2.0, event.status.color()));
                painter.circle_filled(from_pos, 4.0, event.status.color());
                painter.circle_filled(to_pos, 4.0, event.status.color());

                if let Some((ex, ey, ew, eh)) = event_screen_rect(
                    event,
                    start_ms,
                    end_ms,
                    rect_min,
                    time_width,
                    lane_height,
                    &primal_lanes,
                ) {
                    let event_rect = Rect::from_min_size(Pos2::new(ex, ey), Vec2::new(ew, eh));
                    if response.clicked()
                        && let Some(pointer_pos) = response.interact_pointer_pos()
                        && event_rect.contains(pointer_pos)
                    {
                        intents.push(TimelineIntent::SelectEvent(event.id.clone()));
                    }
                }
            }
        }

        intents
    }

    /// Render event details panel using pre-computed display state. Returns intents.
    fn render_details_panel(&self, ui: &mut egui::Ui) -> Vec<TimelineIntent> {
        let mut intents = Vec::new();

        ui.heading("Event Details");
        ui.separator();

        if let Some(ref event_id) = self.selected_event {
            if let Some(event) = self.events.iter().find(|e| &e.id == event_id) {
                let detail = prepare_event_detail(event);

                egui::Grid::new("event_details_grid")
                    .num_columns(2)
                    .spacing([10.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Status:");
                        ui.horizontal(|ui| {
                            ui.label(detail.status_icon);
                            ui.label(&detail.status_label);
                        });
                        ui.end_row();

                        ui.label("From:");
                        ui.label(&detail.from);
                        ui.end_row();

                        ui.label("To:");
                        ui.label(&detail.to);
                        ui.end_row();

                        ui.label("Type:");
                        ui.label(&detail.event_type);
                        ui.end_row();

                        ui.label("Time:");
                        ui.label(&detail.time_str);
                        ui.end_row();

                        if let Some(ref duration) = detail.duration_str {
                            ui.label("Duration:");
                            ui.label(duration);
                            ui.end_row();
                        }

                        if let Some(ref payload) = detail.payload {
                            ui.label("Payload:");
                            ui.label(payload);
                            ui.end_row();
                        }
                    });

                ui.add_space(16.0);

                if ui.button("Close Details").clicked() {
                    intents.push(TimelineIntent::DeselectEvent);
                }
            }
        } else {
            ui.label("Click on an event to see details");
        }

        intents
    }

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
        write!(f, "{}", format_events_csv(events))?;
        Ok(())
    }

    #[cfg(test)]
    #[doc(hidden)]
    #[must_use]
    pub fn export_csv_path_for_test(&self) -> PathBuf {
        self.export_csv_path()
    }

    #[cfg(test)]
    #[doc(hidden)]
    pub fn write_events_csv_for_test(
        &self,
        path: &PathBuf,
        events: Vec<&TimelineEvent>,
    ) -> std::io::Result<()> {
        self.write_events_csv(path, events)
    }
}
