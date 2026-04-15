// SPDX-License-Identifier: AGPL-3.0-or-later

//! Rolling aggregation of [`crate::types::TelemetryMetrics`] from [`crate::types::TelemetryEvent`].

use crate::types::{TelemetryEvent, TelemetryMetrics};

pub fn apply_event_to_metrics(metrics: &mut TelemetryMetrics, event: &TelemetryEvent) {
    match event {
        TelemetryEvent::PrimalDiscovered { primal_id, .. } => {
            metrics.total_primals += 1;
            metrics.active_primals += 1;
            metrics.primal_metrics.entry(primal_id.clone()).or_default();
        }

        TelemetryEvent::PrimalDisappeared { primal_id, .. } => {
            if metrics.active_primals > 0 {
                metrics.active_primals -= 1;
            }
            if let Some(pm) = metrics.primal_metrics.get_mut(primal_id) {
                let _ = pm;
            }
        }

        TelemetryEvent::ApiCall {
            from,
            to,
            latency_ms,
            ..
        } => {
            metrics.total_api_calls += 1;

            #[expect(
                clippy::cast_precision_loss,
                reason = "f64 mantissa covers u64 counts for running-average math"
            )]
            let total = metrics.total_api_calls as f64;
            let prev_avg = metrics.avg_latency_ms;
            metrics.avg_latency_ms = (prev_avg * (total - 1.0) + latency_ms) / total;

            if let Some(pm) = metrics.primal_metrics.get_mut(from) {
                pm.calls_made += 1;
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "f64 mantissa covers u64 counts for per-primal averages"
                )]
                let pm_total = pm.calls_made as f64;
                pm.avg_latency_ms = (pm.avg_latency_ms * (pm_total - 1.0) + latency_ms) / pm_total;
            }
            if let Some(pm) = metrics.primal_metrics.get_mut(to) {
                pm.calls_received += 1;
            }
        }

        TelemetryEvent::DataTransfer {
            from, to, bytes, ..
        } => {
            metrics.total_bytes += bytes;

            if let Some(pm) = metrics.primal_metrics.get_mut(from) {
                pm.bytes_sent += bytes;
            }
            if let Some(pm) = metrics.primal_metrics.get_mut(to) {
                pm.bytes_received += bytes;
            }
        }

        TelemetryEvent::HealthUpdate { .. } => {}
    }
}
