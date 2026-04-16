// SPDX-License-Identifier: AGPL-3.0-or-later
//! Enum dispatch for [`TelemetrySubscriber`](crate::types::TelemetrySubscriber).

use std::sync::{Arc, RwLock};

use crate::types::{TelemetryEvent, TelemetryMetrics, TelemetrySubscriber};

/// Counts events received (tests / fixtures).
pub struct TestSubscriber {
    /// Incremented on each [`TelemetrySubscriber::on_event`].
    pub events_received: Arc<RwLock<usize>>,
}

impl TelemetrySubscriber for TestSubscriber {
    fn on_event(&mut self, _event: &TelemetryEvent) {
        if let Ok(mut n) = self.events_received.write() {
            *n += 1;
        }
    }
}

/// Counts metrics updates (tests / fixtures).
pub struct MetricsUpdateSubscriber {
    /// Incremented on each [`TelemetrySubscriber::on_metrics_update`].
    pub updates: Arc<RwLock<usize>>,
}

impl TelemetrySubscriber for MetricsUpdateSubscriber {
    fn on_event(&mut self, _event: &TelemetryEvent) {}

    fn on_metrics_update(&mut self, metrics: &TelemetryMetrics) {
        if let Ok(mut n) = self.updates.write() {
            *n += 1;
        }
        let _ = metrics;
    }
}

/// Concrete telemetry subscribers (replaces `Box<dyn TelemetrySubscriber>`).
pub enum TelemetrySubscriberImpl {
    /// Test helper: count events.
    Test(TestSubscriber),
    /// Test helper: count metrics updates.
    MetricsUpdate(MetricsUpdateSubscriber),
}

impl TelemetrySubscriber for TelemetrySubscriberImpl {
    fn on_event(&mut self, event: &TelemetryEvent) {
        match self {
            Self::Test(s) => s.on_event(event),
            Self::MetricsUpdate(s) => s.on_event(event),
        }
    }

    fn on_metrics_update(&mut self, metrics: &TelemetryMetrics) {
        match self {
            Self::Test(s) => s.on_metrics_update(metrics),
            Self::MetricsUpdate(s) => s.on_metrics_update(metrics),
        }
    }
}
