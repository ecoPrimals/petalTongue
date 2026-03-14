// SPDX-License-Identifier: AGPL-3.0-only
//! Timeline View - Pure helper functions (fully testable, no &self, no egui context)

use super::types::TimelineEvent;

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
pub fn prepare_event_detail(event: &TimelineEvent) -> super::types::EventDetailDisplay {
    super::types::EventDetailDisplay {
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
