// SPDX-License-Identifier: AGPL-3.0-or-later

mod metrics;
mod snapshot;

#[cfg(test)]
mod test_fixtures;
#[cfg(test)]
mod tests_a;
#[cfg(test)]
mod tests_b;

use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use crate::subscriber_impl::TelemetrySubscriberImpl;
use crate::types::{TelemetryEvent, TelemetryMetrics, TelemetrySubscriber};

pub struct TelemetryCollector {
    buffer: Arc<RwLock<VecDeque<TelemetryEvent>>>,
    metrics: Arc<RwLock<TelemetryMetrics>>,
    subscribers: Arc<RwLock<Vec<TelemetrySubscriberImpl>>>,
    max_buffer_size: usize,
    aggregation_interval: Duration,
}

impl TelemetryCollector {
    #[must_use]
    pub fn new(max_buffer_size: usize, aggregation_interval: Duration) -> Self {
        Self {
            buffer: Arc::new(RwLock::new(VecDeque::with_capacity(max_buffer_size))),
            metrics: Arc::new(RwLock::new(TelemetryMetrics::default())),
            subscribers: Arc::new(RwLock::new(Vec::new())),
            max_buffer_size,
            aggregation_interval,
        }
    }

    pub fn push_event(&self, event: &TelemetryEvent) {
        {
            let Ok(mut buffer) = self.buffer.write() else {
                tracing::error!("Telemetry buffer lock poisoned");
                return;
            };
            buffer.push_back(event.clone());

            if buffer.len() > self.max_buffer_size {
                buffer.pop_front();
            }
        }

        {
            let Ok(mut subscribers) = self.subscribers.write() else {
                tracing::error!("Telemetry subscribers lock poisoned");
                return;
            };
            for subscriber in subscribers.iter_mut() {
                subscriber.on_event(event);
            }
        }

        self.update_metrics(event);
    }

    pub fn add_subscriber(&self, subscriber: TelemetrySubscriberImpl) {
        let Ok(mut subscribers) = self.subscribers.write() else {
            tracing::error!("Telemetry subscribers lock poisoned");
            return;
        };
        subscribers.push(subscriber);
    }

    #[must_use]
    pub fn get_metrics(&self) -> TelemetryMetrics {
        let Ok(metrics) = self.metrics.read() else {
            tracing::error!("Telemetry metrics lock poisoned");
            return TelemetryMetrics::default();
        };
        snapshot::clone_metrics(&metrics)
    }

    #[must_use]
    pub fn get_recent_events(&self, count: usize) -> Vec<TelemetryEvent> {
        let Ok(buffer) = self.buffer.read() else {
            tracing::error!("Telemetry buffer lock poisoned");
            return Vec::new();
        };
        snapshot::recent_events(&buffer, count)
    }

    fn update_metrics(&self, event: &TelemetryEvent) {
        let Ok(mut metrics) = self.metrics.write() else {
            tracing::error!("Telemetry metrics lock poisoned");
            return;
        };

        metrics::apply_event_to_metrics(&mut metrics, event);
    }

    pub fn clear(&self) {
        let Ok(mut buffer) = self.buffer.write() else {
            tracing::error!("Telemetry buffer lock poisoned");
            return;
        };
        buffer.clear();
    }

    #[must_use]
    pub const fn aggregation_interval(&self) -> Duration {
        self.aggregation_interval
    }
}

impl Default for TelemetryCollector {
    fn default() -> Self {
        Self::new(
            petal_tongue_core::constants::default_telemetry_buffer(),
            Duration::from_secs(1),
        )
    }
}
