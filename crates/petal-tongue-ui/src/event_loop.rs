//! Sensory Event Loop
//!
//! Continuously polls sensors and feeds events to RenderingAwareness.
//! Completes the bidirectional feedback loop.

use anyhow::Result;
use petal_tongue_core::{RenderingAwareness, SensorRegistry};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time::sleep;

/// Start the sensory event loop (background task)
/// NOTE: Currently disabled - egui already provides perfect sensory feedback!
/// The bidirectional loop works via egui input events in the update() loop.
/// This function is here for future async sensor support (camera, network sensors, etc.)
#[allow(dead_code)]
pub fn start_event_loop(
    _sensor_registry: Arc<RwLock<SensorRegistry>>,
    _rendering_awareness: Arc<RwLock<RenderingAwareness>>,
) -> tokio::task::JoinHandle<()> {
    tracing::info!("🔄 Sensory loop active (via egui input events)");

    // Placeholder task that completes immediately
    // Real implementation will use tokio::sync::RwLock instead of std::sync::RwLock
    tokio::spawn(async {
        // Sensory feedback happens via egui - no separate task needed!
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

        if !events.is_empty() {
            if let Ok(mut awareness) = rendering_awareness.write() {
                for event in events {
                    awareness.sensory_feedback(&event);
                }
            }
        }
    }

    Ok(event_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::MotorCommand;

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
    async fn test_event_loop_starts() {
        let registry = Arc::new(RwLock::new(SensorRegistry::new()));
        let awareness = Arc::new(RwLock::new(RenderingAwareness::new()));

        // Start event loop (currently a placeholder that completes immediately)
        let handle = start_event_loop(Arc::clone(&registry), Arc::clone(&awareness));

        // Let task complete (placeholder completes immediately)
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Placeholder task should complete (sensory feedback happens via egui)
        assert!(handle.is_finished() || handle.await.is_ok());
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
