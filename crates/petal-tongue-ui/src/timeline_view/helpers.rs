// SPDX-License-Identifier: AGPL-3.0-only
//! Timeline View - Pure helper functions (fully testable, no &self, no egui context)

use std::collections::HashMap;

use super::filtering::get_primals;
use super::types::TimelineEvent;

#[must_use]
pub fn build_primal_lanes(events: &[TimelineEvent]) -> HashMap<String, usize> {
    let primals = get_primals(events);
    primals
        .into_iter()
        .enumerate()
        .map(|(i, p)| (p, i))
        .collect()
}

#[must_use]
pub fn compute_lane_height(rect_height: f32, lane_count: usize) -> f32 {
    rect_height / (lane_count as f32 + 1.0)
}

#[must_use]
pub fn event_screen_rect<S: ::std::hash::BuildHasher>(
    event: &TimelineEvent,
    start_ms: f64,
    end_ms: f64,
    rect_min: (f32, f32),
    time_width: f32,
    lane_height: f32,
    primal_lanes: &HashMap<String, usize, S>,
) -> Option<(f32, f32, f32, f32)> {
    let from_lane = *primal_lanes.get(&event.from)?;
    let to_lane = *primal_lanes.get(&event.to)?;
    let time_ms = event.timestamp.timestamp_millis() as f64;
    let x_offset = time_to_x(time_ms, start_ms, end_ms, time_width);
    let x_center = rect_min.0 + 100.0 + x_offset;
    let from_y = lane_height.mul_add(from_lane as f32 + 1.0, rect_min.1);
    let to_y = lane_height.mul_add(to_lane as f32 + 1.0, rect_min.1);
    let y_min = from_y.min(to_y) - 4.0;
    let height = (from_y - to_y).abs() + 8.0;
    Some((x_center - 4.0, y_min, 8.0, height))
}

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
