// SPDX-License-Identifier: AGPL-3.0-only
//! Timeline View - Event Sequence Visualization
//!
//! Displays temporal sequences of primal interactions with time scrubbing capabilities.
//! Implements Phase 4 of the UI specification.

use chrono::{DateTime, Utc};
use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use std::collections::HashMap;

/// Event in the timeline
#[derive(Clone, Debug)]
pub struct TimelineEvent {
    /// Unique event ID
    pub id: String,
    /// Source primal ID
    pub from: String,
    /// Target primal ID
    pub to: String,
    /// Event type (capability name, message type, etc.)
    pub event_type: String,
    /// Timestamp when event occurred
    pub timestamp: DateTime<Utc>,
    /// Duration of the event (if applicable)
    pub duration_ms: Option<f64>,
    /// Status (success, failure, etc.)
    pub status: EventStatus,
    /// Optional payload summary
    pub payload_summary: Option<String>,
}

/// Status of a timeline event
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventStatus {
    /// Event completed successfully
    Success,
    /// Event failed
    Failure,
    /// Event is still in progress
    InProgress,
    /// Event timed out
    Timeout,
}

impl EventStatus {
    /// Get color for this status
    #[must_use]
    pub fn color(&self) -> Color32 {
        match self {
            Self::Success => Color32::from_rgb(100, 255, 100),
            Self::Failure => Color32::from_rgb(255, 100, 100),
            Self::InProgress => Color32::from_rgb(255, 200, 100),
            Self::Timeout => Color32::from_rgb(200, 100, 255),
        }
    }

    /// Get icon for this status
    #[must_use]
    pub const fn icon(&self) -> &'static str {
        match self {
            Self::Success => "✅",
            Self::Failure => "❌",
            Self::InProgress => "⏳",
            Self::Timeout => "⏱️",
        }
    }
}

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
    /// Scroll offset (for panning)
    scroll_offset: f32,
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
            scroll_offset: 0.0,
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

    /// Get filtered events based on current filters
    fn filtered_events(&self) -> Vec<&TimelineEvent> {
        self.events
            .iter()
            .filter(|e| {
                // Apply event type filter
                if let Some(ref filter) = self.event_type_filter
                    && &e.event_type != filter
                {
                    return false;
                }

                // Apply primal filter
                if let Some(ref filter) = self.primal_filter
                    && &e.from != filter
                    && &e.to != filter
                {
                    return false;
                }

                // Apply time range filter
                if let Some(start) = self.time_range_start
                    && e.timestamp < start
                {
                    return false;
                }
                if let Some(end) = self.time_range_end
                    && e.timestamp > end
                {
                    return false;
                }

                true
            })
            .collect()
    }

    /// Get list of unique primals involved in events
    fn get_primals(&self) -> Vec<String> {
        let mut primals = std::collections::HashSet::new();
        for event in &self.events {
            primals.insert(event.from.clone());
            primals.insert(event.to.clone());
        }
        let mut primal_vec: Vec<_> = primals.into_iter().collect();
        primal_vec.sort();
        primal_vec
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
        painter.rect_filled(rect, 0.0, Color32::from_rgb(20, 20, 25));

        let primals = self.get_primals();
        if primals.is_empty() {
            // No events yet
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "No events to display",
                egui::FontId::proportional(16.0),
                Color32::GRAY,
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
                Stroke::new(1.0, Color32::from_rgb(50, 50, 60)),
            );

            // Primal label
            painter.text(
                Pos2::new(rect.min.x + 50.0, y),
                egui::Align2::CENTER_CENTER,
                primal,
                egui::FontId::monospace(12.0),
                Color32::WHITE,
            );
        }

        // Draw events
        let filtered_events = self.filtered_events();
        if !filtered_events.is_empty() {
            // Calculate time range
            let min_time = filtered_events.first().unwrap().timestamp;
            let max_time = filtered_events.last().unwrap().timestamp;
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
    fn export_csv(&self) {
        // TODO: Implement CSV export
        // This would write events to a file in CSV format
        tracing::info!("CSV export not yet implemented");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeline_view_creation() {
        let view = TimelineView::new();
        assert_eq!(view.events.len(), 0);
        assert!(view.selected_event.is_none());
        assert_eq!(view.zoom, 1.0);
    }

    #[test]
    fn test_add_event() {
        let mut view = TimelineView::new();

        let event = TimelineEvent {
            id: "evt1".to_string(),
            from: "primal1".to_string(),
            to: "primal2".to_string(),
            event_type: "discover".to_string(),
            timestamp: Utc::now(),
            duration_ms: Some(10.5),
            status: EventStatus::Success,
            payload_summary: None,
        };

        view.add_event(event);
        assert_eq!(view.events.len(), 1);
    }

    #[test]
    fn test_clear_events() {
        let mut view = TimelineView::new();

        // Add some events
        for i in 0..5 {
            view.add_event(TimelineEvent {
                id: format!("evt{i}"),
                from: "primal1".to_string(),
                to: "primal2".to_string(),
                event_type: "test".to_string(),
                timestamp: Utc::now(),
                duration_ms: None,
                status: EventStatus::Success,
                payload_summary: None,
            });
        }

        assert_eq!(view.events.len(), 5);

        view.clear();
        assert_eq!(view.events.len(), 0);
        assert!(view.selected_event.is_none());
    }

    #[test]
    fn test_get_primals() {
        let mut view = TimelineView::new();

        view.add_event(TimelineEvent {
            id: "evt1".to_string(),
            from: "alice".to_string(),
            to: "bob".to_string(),
            event_type: "test".to_string(),
            timestamp: Utc::now(),
            duration_ms: None,
            status: EventStatus::Success,
            payload_summary: None,
        });

        view.add_event(TimelineEvent {
            id: "evt2".to_string(),
            from: "bob".to_string(),
            to: "charlie".to_string(),
            event_type: "test".to_string(),
            timestamp: Utc::now(),
            duration_ms: None,
            status: EventStatus::Success,
            payload_summary: None,
        });

        let primals = view.get_primals();
        assert_eq!(primals.len(), 3);
        assert!(primals.contains(&"alice".to_string()));
        assert!(primals.contains(&"bob".to_string()));
        assert!(primals.contains(&"charlie".to_string()));
    }

    #[test]
    fn test_event_status_colors() {
        assert_ne!(EventStatus::Success.color(), EventStatus::Failure.color());
        assert_ne!(
            EventStatus::Success.color(),
            EventStatus::InProgress.color()
        );
        assert_ne!(EventStatus::Failure.color(), EventStatus::Timeout.color());
    }

    #[test]
    fn test_event_sorting() {
        let mut view = TimelineView::new();

        let now = Utc::now();

        // Add events out of order
        view.add_event(TimelineEvent {
            id: "evt3".to_string(),
            from: "a".to_string(),
            to: "b".to_string(),
            event_type: "test".to_string(),
            timestamp: now + chrono::Duration::seconds(3),
            duration_ms: None,
            status: EventStatus::Success,
            payload_summary: None,
        });

        view.add_event(TimelineEvent {
            id: "evt1".to_string(),
            from: "a".to_string(),
            to: "b".to_string(),
            event_type: "test".to_string(),
            timestamp: now + chrono::Duration::seconds(1),
            duration_ms: None,
            status: EventStatus::Success,
            payload_summary: None,
        });

        view.add_event(TimelineEvent {
            id: "evt2".to_string(),
            from: "a".to_string(),
            to: "b".to_string(),
            event_type: "test".to_string(),
            timestamp: now + chrono::Duration::seconds(2),
            duration_ms: None,
            status: EventStatus::Success,
            payload_summary: None,
        });

        // Events should be sorted by timestamp
        assert_eq!(view.events[0].id, "evt1");
        assert_eq!(view.events[1].id, "evt2");
        assert_eq!(view.events[2].id, "evt3");
    }
}
