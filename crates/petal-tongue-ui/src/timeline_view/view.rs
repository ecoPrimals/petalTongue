// SPDX-License-Identifier: AGPL-3.0-only
//! Timeline View - Core view struct and rendering logic
//!
//! Displays temporal sequences of primal interactions with time scrubbing capabilities.
//! Implements Phase 4 of the UI specification.
//!
//! Architecture: headless-first. Pure functions (`time_to_x`, `prepare_event_detail`,
//! `format_events_csv`, `escape_csv`) produce testable data. The render method returns
//! `Vec<TimelineIntent>` rather than mutating state directly.

use chrono::{DateTime, Utc};
use egui::{Pos2, Rect, Stroke, Vec2};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use super::filtering::{filtered_events, get_primals};
use super::types::TimelineEvent;

// ============================================================================
// Intent and display state types (headless-testable)
// ============================================================================

/// User interaction intent produced by the timeline view render method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimelineIntent {
    ZoomIn,
    ZoomOut,
    ToggleDetails,
    SelectEvent(String),
    DeselectEvent,
    Clear,
    ExportCsv,
}

/// Pre-computed display data for a selected event detail panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventDetailDisplay {
    pub status_icon: &'static str,
    pub status_label: String,
    pub from: String,
    pub to: String,
    pub event_type: String,
    pub time_str: String,
    pub duration_str: Option<String>,
    pub payload: Option<String>,
}

// ============================================================================
// Pure functions (fully testable, no &self, no egui context)
// ============================================================================

/// Map a timestamp to an x-coordinate within the time area.
/// Returns x in [0, rect_width] for the given time range.
#[must_use]
pub fn time_to_x(time: f64, start_time: f64, end_time: f64, rect_width: f32) -> f32 {
    let range = end_time - start_time;
    let fraction = if range <= 0.0 {
        0.0
    } else {
        ((time - start_time) / range).clamp(0.0, 1.0)
    };
    fraction as f32 * rect_width
}

/// Prepare an `EventDetailDisplay` from a `TimelineEvent` (pure computation).
#[must_use]
pub fn prepare_event_detail(event: &TimelineEvent) -> EventDetailDisplay {
    EventDetailDisplay {
        status_icon: event.status.icon(),
        status_label: format!("{:?}", event.status),
        from: event.from.clone(),
        to: event.to.clone(),
        event_type: event.event_type.clone(),
        time_str: event.timestamp.format("%H:%M:%S%.3f").to_string(),
        duration_str: event.duration_ms.map(|d| format!("{d:.2}ms")),
        payload: event.payload_summary.clone(),
    }
}

/// Compute zoom level after a zoom-in action.
#[must_use]
pub fn zoom_in(current: f32) -> f32 {
    (current * 1.2).min(10.0)
}

/// Compute zoom level after a zoom-out action.
#[must_use]
pub fn zoom_out(current: f32) -> f32 {
    (current / 1.2).max(0.1)
}

// ============================================================================
// TimelineView
// ============================================================================

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
        let lane_height = rect.height() / (primal_count as f32 + 1.0);
        let time_width = rect.width() - 100.0;

        let primal_lanes: HashMap<_, _> = primals
            .iter()
            .enumerate()
            .map(|(i, p)| (p.clone(), i))
            .collect();

        for (idx, primal) in primals.iter().enumerate() {
            let y = rect.min.y + lane_height * (idx as f32 + 1.0);

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

        for event in filtered_events {
            if let (Some(from_lane), Some(to_lane)) =
                (primal_lanes.get(&event.from), primal_lanes.get(&event.to))
            {
                let time_ms = event.timestamp.timestamp_millis() as f64;
                let x = rect.min.x + 100.0 + time_to_x(time_ms, start_ms, end_ms, time_width);

                let from_y = rect.min.y + lane_height * (*from_lane as f32 + 1.0);
                let to_y = rect.min.y + lane_height * (*to_lane as f32 + 1.0);

                let from_pos = Pos2::new(x, from_y);
                let to_pos = Pos2::new(x, to_y);

                painter.line_segment([from_pos, to_pos], Stroke::new(2.0, event.status.color()));
                painter.circle_filled(from_pos, 4.0, event.status.color());
                painter.circle_filled(to_pos, 4.0, event.status.color());

                let click_rect = Rect::from_center_size(from_pos, Vec2::splat(10.0));
                if response.clicked()
                    && let Some(pointer_pos) = response.interact_pointer_pos()
                    && click_rect.contains(pointer_pos)
                {
                    intents.push(TimelineIntent::SelectEvent(event.id.clone()));
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
}

/// Format timeline events as CSV content (for testing and export).
#[must_use]
pub fn format_events_csv(events: Vec<&TimelineEvent>) -> String {
    let mut out = String::new();
    out.push_str("id,from,to,event_type,timestamp,duration_ms,status,payload_summary\n");
    for event in events {
        let duration = event
            .duration_ms
            .map_or_else(String::new, |d| d.to_string());
        let payload = event.payload_summary.as_deref().unwrap_or("");
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{}\n",
            escape_csv(&event.id),
            escape_csv(&event.from),
            escape_csv(&event.to),
            escape_csv(&event.event_type),
            escape_csv(&event.timestamp.to_rfc3339()),
            escape_csv(&duration),
            escape_csv(&format!("{:?}", event.status)),
            escape_csv(payload),
        ));
    }
    out
}

/// Escape a CSV field (RFC 4180): wrap in quotes if contains comma, quote, or newline
#[must_use]
pub fn escape_csv(s: &str) -> String {
    let needs_quotes = s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r');
    if needs_quotes {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::EventStatus;
    use super::*;

    // === Pure function tests ===

    #[test]
    fn prepare_event_detail_complete() {
        let event = TimelineEvent {
            id: "evt1".to_string(),
            from: "alice".to_string(),
            to: "bob".to_string(),
            event_type: "discover".to_string(),
            timestamp: chrono::DateTime::parse_from_rfc3339("2025-01-15T12:30:45.123Z")
                .expect("valid datetime")
                .with_timezone(&chrono::Utc),
            duration_ms: Some(42.5),
            status: EventStatus::Success,
            payload_summary: Some("payload data".to_string()),
        };
        let d = prepare_event_detail(&event);
        assert_eq!(d.status_icon, "✅");
        assert_eq!(d.status_label, "Success");
        assert_eq!(d.from, "alice");
        assert_eq!(d.to, "bob");
        assert_eq!(d.event_type, "discover");
        assert_eq!(d.time_str, "12:30:45.123");
        assert_eq!(d.duration_str.as_deref(), Some("42.50ms"));
        assert_eq!(d.payload.as_deref(), Some("payload data"));
    }

    #[test]
    fn prepare_event_detail_no_optional_fields() {
        let event = TimelineEvent {
            id: "evt2".to_string(),
            from: "a".to_string(),
            to: "b".to_string(),
            event_type: "invoke".to_string(),
            timestamp: chrono::DateTime::parse_from_rfc3339("2025-06-01T00:00:00Z")
                .expect("valid datetime")
                .with_timezone(&chrono::Utc),
            duration_ms: None,
            status: EventStatus::Failure,
            payload_summary: None,
        };
        let d = prepare_event_detail(&event);
        assert_eq!(d.status_icon, "❌");
        assert_eq!(d.status_label, "Failure");
        assert!(d.duration_str.is_none());
        assert!(d.payload.is_none());
    }

    #[test]
    fn prepare_event_detail_timeout() {
        let event = TimelineEvent {
            id: "evt3".to_string(),
            from: "x".to_string(),
            to: "y".to_string(),
            event_type: "ping".to_string(),
            timestamp: chrono::Utc::now(),
            duration_ms: Some(5000.0),
            status: EventStatus::Timeout,
            payload_summary: None,
        };
        let d = prepare_event_detail(&event);
        assert_eq!(d.status_icon, "⏱️");
        assert_eq!(d.status_label, "Timeout");
        assert_eq!(d.duration_str.as_deref(), Some("5000.00ms"));
    }

    #[test]
    fn prepare_event_detail_in_progress() {
        let event = TimelineEvent {
            id: "evt4".to_string(),
            from: "x".to_string(),
            to: "y".to_string(),
            event_type: "stream".to_string(),
            timestamp: chrono::Utc::now(),
            duration_ms: None,
            status: EventStatus::InProgress,
            payload_summary: Some("streaming...".to_string()),
        };
        let d = prepare_event_detail(&event);
        assert_eq!(d.status_icon, "⏳");
        assert_eq!(d.payload.as_deref(), Some("streaming..."));
    }

    #[test]
    fn zoom_in_increases() {
        assert!(zoom_in(1.0) > 1.0);
    }

    #[test]
    fn zoom_in_caps_at_10() {
        assert!((zoom_in(10.0) - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn zoom_out_decreases() {
        assert!(zoom_out(1.0) < 1.0);
    }

    #[test]
    fn zoom_out_floors_at_0_1() {
        let result = zoom_out(0.1);
        assert!((result - 0.1).abs() < 0.01);
    }

    #[test]
    fn apply_intents_zoom() {
        let mut view = TimelineView::new();
        let initial = view.zoom();
        view.apply_intents(&[TimelineIntent::ZoomIn]);
        assert!(view.zoom() > initial);
        view.apply_intents(&[TimelineIntent::ZoomOut, TimelineIntent::ZoomOut]);
        assert!(view.zoom() < initial);
    }

    #[test]
    fn apply_intents_select_deselect() {
        let mut view = TimelineView::new();
        view.apply_intents(&[TimelineIntent::SelectEvent("evt1".to_string())]);
        assert_eq!(view.selected_event(), Some("evt1"));
        view.apply_intents(&[TimelineIntent::DeselectEvent]);
        assert!(view.selected_event().is_none());
    }

    #[test]
    fn apply_intents_clear() {
        let mut view = TimelineView::new();
        view.add_event(TimelineEvent {
            id: "e".to_string(),
            from: "a".to_string(),
            to: "b".to_string(),
            event_type: "t".to_string(),
            timestamp: chrono::Utc::now(),
            duration_ms: None,
            status: EventStatus::Success,
            payload_summary: None,
        });
        view.apply_intents(&[TimelineIntent::Clear]);
        assert_eq!(view.filtered_event_count(), 0);
    }

    // === Existing tests ===

    #[test]
    fn time_to_x_start() {
        let x = time_to_x(100.0, 100.0, 200.0, 500.0);
        assert!(x.abs() < f32::EPSILON);
    }

    #[test]
    fn time_to_x_end() {
        let x = time_to_x(200.0, 100.0, 200.0, 500.0);
        assert!((x - 500.0).abs() < f32::EPSILON);
    }

    #[test]
    fn time_to_x_mid() {
        let x = time_to_x(150.0, 100.0, 200.0, 500.0);
        assert!((x - 250.0).abs() < f32::EPSILON);
    }

    #[test]
    fn time_to_x_zero_range() {
        let x = time_to_x(100.0, 100.0, 100.0, 500.0);
        assert!(x.abs() < f32::EPSILON);
    }

    #[test]
    fn time_to_x_clamps_before_start() {
        let x = time_to_x(50.0, 100.0, 200.0, 500.0);
        assert!(x.abs() < f32::EPSILON);
    }

    #[test]
    fn time_to_x_clamps_after_end() {
        let x = time_to_x(250.0, 100.0, 200.0, 500.0);
        assert!((x - 500.0).abs() < f32::EPSILON);
    }

    #[test]
    fn escape_csv_plain() {
        assert_eq!(escape_csv("hello"), "hello");
    }

    #[test]
    fn escape_csv_with_comma() {
        assert_eq!(escape_csv("a,b"), "\"a,b\"");
    }

    #[test]
    fn escape_csv_with_quote() {
        assert_eq!(escape_csv("say \"hi\""), "\"say \"\"hi\"\"\"");
    }

    #[test]
    fn escape_csv_with_newline() {
        assert_eq!(escape_csv("line1\nline2"), "\"line1\nline2\"");
    }

    #[test]
    fn format_events_csv_empty() {
        let events: Vec<&TimelineEvent> = Vec::new();
        let csv = format_events_csv(events);
        assert_eq!(
            csv,
            "id,from,to,event_type,timestamp,duration_ms,status,payload_summary\n"
        );
    }

    #[test]
    fn format_events_csv_single_event() {
        let event = TimelineEvent {
            id: "ev1".to_string(),
            from: "primal-a".to_string(),
            to: "primal-b".to_string(),
            event_type: "message".to_string(),
            timestamp: chrono::DateTime::parse_from_rfc3339("2025-01-15T12:00:00Z")
                .expect("valid datetime")
                .with_timezone(&chrono::Utc),
            duration_ms: Some(42.5),
            status: EventStatus::Success,
            payload_summary: Some("summary".to_string()),
        };
        let csv = format_events_csv(vec![&event]);
        assert!(
            csv.starts_with("id,from,to,event_type,timestamp,duration_ms,status,payload_summary\n")
        );
        assert!(csv.contains("ev1"));
        assert!(csv.contains("primal-a"));
        assert!(csv.contains("primal-b"));
        assert!(csv.contains("message"));
        assert!(csv.contains("42.5"));
        assert!(csv.contains("Success"));
        assert!(csv.contains("summary"));
    }
}
