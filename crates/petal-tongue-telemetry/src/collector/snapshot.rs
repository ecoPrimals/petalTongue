// SPDX-License-Identifier: AGPL-3.0-or-later

//! Read-only snapshots of buffered events and aggregate metrics (telemetry “export” surface).

use std::collections::VecDeque;

use crate::types::{TelemetryEvent, TelemetryMetrics};

#[must_use]
pub fn clone_metrics(metrics: &TelemetryMetrics) -> TelemetryMetrics {
    metrics.clone()
}

#[must_use]
pub fn recent_events(buffer: &VecDeque<TelemetryEvent>, count: usize) -> Vec<TelemetryEvent> {
    buffer.iter().rev().take(count).cloned().collect()
}
