// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::{Arc, RwLock};

use crate::types::{TelemetryEvent, TelemetryMetrics, TelemetrySubscriber};

pub(super) struct TestSubscriber {
    pub events_received: Arc<RwLock<usize>>,
}

impl TelemetrySubscriber for TestSubscriber {
    fn on_event(&mut self, _event: &TelemetryEvent) {
        *self.events_received.write().expect("SAFETY: Lock poisoned") += 1;
    }
}

pub(super) struct MetricsUpdateSubscriber {
    pub updates: Arc<RwLock<usize>>,
}

impl TelemetrySubscriber for MetricsUpdateSubscriber {
    fn on_event(&mut self, _event: &TelemetryEvent) {}

    fn on_metrics_update(&mut self, metrics: &TelemetryMetrics) {
        *self.updates.write().expect("lock") += 1;
        let _ = metrics;
    }
}

pub(super) fn event_primal_id(event: &TelemetryEvent) -> Option<&str> {
    match event {
        TelemetryEvent::PrimalDiscovered { primal_id, .. }
        | TelemetryEvent::PrimalDisappeared { primal_id, .. }
        | TelemetryEvent::HealthUpdate { primal_id, .. } => Some(primal_id),
        _ => None,
    }
}
