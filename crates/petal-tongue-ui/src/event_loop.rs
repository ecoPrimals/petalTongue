// SPDX-License-Identifier: AGPL-3.0-only
//! Sensory Event Loop
//!
//! Continuously polls sensors and feeds events to `RenderingAwareness`.
//! Completes the bidirectional feedback loop.

use anyhow::Result;
use petal_tongue_core::{RenderingAwareness, SensorRegistry};
use std::sync::{Arc, RwLock};

/// Start the sensory event loop (background task).
///
/// For GUI mode, egui already provides input events in the `update()` loop,
/// making a separate poll unnecessary. This function spawns a lightweight
/// task that bridges async sensors (network, external devices) into
/// `RenderingAwareness` for non-egui sensor sources.
///
/// Uses `spawn_blocking` because `std::sync::RwLock` guards are `!Send`.
pub fn start_event_loop(
    sensor_registry: Arc<RwLock<SensorRegistry>>,
    rendering_awareness: Arc<RwLock<RenderingAwareness>>,
) -> tokio::task::JoinHandle<()> {
    tracing::info!("Sensory event loop started");

    tokio::spawn(async move {
        let reg = sensor_registry;
        let aw = rendering_awareness;
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

            let reg_clone = Arc::clone(&reg);
            let aw_clone = Arc::clone(&aw);
            let result = tokio::task::spawn_blocking(move || {
                reg_clone.write().map_or(0, |mut registry| {
                    let rt = tokio::runtime::Handle::current();
                    match rt.block_on(registry.poll_all()) {
                        Ok(events) if !events.is_empty() => {
                            if let Ok(mut awareness) = aw_clone.write() {
                                for event in &events {
                                    awareness.sensory_feedback(event);
                                }
                            }
                            events.len()
                        }
                        _ => 0,
                    }
                })
            })
            .await;

            match result {
                Ok(0) => {}
                Ok(n) => tracing::trace!("Polled {n} sensor events"),
                Err(e) => tracing::debug!("Sensor poll task error: {e}"),
            }
        }
    })
}

/// Poll sensors once (for synchronous contexts)
pub async fn poll_sensors_once(
    sensor_registry: &Arc<RwLock<SensorRegistry>>,
    rendering_awareness: &Arc<RwLock<RenderingAwareness>>,
) -> Result<usize> {
    let mut event_count = 0;

    if let Ok(mut registry) = sensor_registry.write() {
        let events = registry.poll_all().await?;
        event_count = events.len();

        if !events.is_empty()
            && let Ok(mut awareness) = rendering_awareness.write()
        {
            for event in events {
                awareness.sensory_feedback(&event);
            }
        }
    }

    Ok(event_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::MotorCommand;
    use tokio::time::Duration;

    #[tokio::test]
    async fn test_poll_sensors_once() {
        let registry = Arc::new(RwLock::new(SensorRegistry::new()));
        let awareness = Arc::new(RwLock::new(RenderingAwareness::new()));

        // Should not fail even with empty registry
        let result = poll_sensors_once(&registry, &awareness).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0); // No events from empty registry
    }

    #[tokio::test]
    async fn test_event_loop_starts_and_polls() {
        let registry = Arc::new(RwLock::new(SensorRegistry::new()));
        let awareness = Arc::new(RwLock::new(RenderingAwareness::new()));

        let handle = start_event_loop(Arc::clone(&registry), Arc::clone(&awareness));

        // Let the loop run a few poll cycles, then abort
        tokio::time::sleep(Duration::from_millis(250)).await;
        handle.abort();
        let result = handle.await;
        assert!(
            result.is_err(),
            "Aborted task should return JoinError (cancelled)"
        );
    }

    #[tokio::test]
    async fn test_motor_sensory_correlation() {
        let awareness = Arc::new(RwLock::new(RenderingAwareness::new()));

        // Send motor command
        if let Ok(mut aw) = awareness.write() {
            aw.motor_command(MotorCommand::RenderFrame { frame_id: 1 });
        }

        // Check initial state
        if let Ok(aw) = awareness.read() {
            let assessment = aw.assess_self();
            assert!(assessment.can_render); // Motor works
            assert!(!assessment.can_sense); // No sensory input yet
            assert!(!assessment.is_complete_loop); // Loop not complete
        }

        // Simulate sensory feedback (Phase 3 will do this automatically)
        // For now, we just verify the infrastructure is ready
    }
}
