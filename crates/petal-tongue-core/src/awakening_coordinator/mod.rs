// SPDX-License-Identifier: AGPL-3.0-or-later
//! # Awakening Coordinator
//!
//! Coordinates visual, audio, and text across the 4-stage awakening timeline.

mod timeline;
mod types;

use crate::awakening::{AwakeningConfig, AwakeningStage};
use crate::engine::UniversalRenderingEngine;
use crate::error::Result;
use crate::event::EngineEvent;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

#[cfg(test)]
mod tests;

pub use timeline::AwakeningTimeline;
pub use types::{TimelineEvent, TimelineEventType};

/// Awakening Coordinator
///
/// Executes the awakening timeline and broadcasts events to all modalities.
pub struct AwakeningCoordinator {
    /// Reference to engine
    engine: Arc<UniversalRenderingEngine>,

    /// Timeline
    timeline: AwakeningTimeline,

    /// Current time (seconds)
    current_time: Arc<RwLock<f32>>,

    /// Current stage
    current_stage: Arc<RwLock<AwakeningStage>>,

    /// Configuration
    config: AwakeningConfig,
}

impl AwakeningCoordinator {
    /// Create new coordinator
    #[must_use]
    pub fn new(engine: Arc<UniversalRenderingEngine>, config: AwakeningConfig) -> Self {
        let timeline = AwakeningTimeline::standard(&config);

        Self {
            engine,
            timeline,
            current_time: Arc::new(RwLock::new(0.0)),
            current_stage: Arc::new(RwLock::new(AwakeningStage::Awakening)),
            config,
        }
    }

    /// Get current time
    pub async fn current_time(&self) -> f32 {
        *self.current_time.read().await
    }

    /// Get current stage
    pub async fn current_stage(&self) -> AwakeningStage {
        *self.current_stage.read().await
    }

    /// Run the complete awakening sequence
    ///
    /// Returns `true` if tutorial mode should be activated after awakening.
    ///
    /// # Errors
    ///
    /// Never returns an error; event broadcast failures are logged but not propagated.
    pub async fn run(&self) -> Result<bool> {
        if !self.config.enabled {
            tracing::info!("Awakening experience disabled, skipping");
            return Ok(false);
        }

        tracing::info!("🌸 Starting coordinated awakening experience...");

        let start_time = std::time::Instant::now();
        let mut last_processed_index = 0;

        loop {
            let elapsed = start_time.elapsed().as_secs_f32();

            // Update current time
            {
                let mut time = self.current_time.write().await;
                *time = elapsed;
            }

            // Check if we've reached the end
            if elapsed >= self.timeline.duration() {
                break;
            }

            // Process events that should have occurred
            let events_to_process: Vec<_> = self.timeline.events()[last_processed_index..]
                .iter()
                .take_while(|e| e.time <= elapsed)
                .collect();

            for event in events_to_process {
                self.process_event(event).await?;
                last_processed_index += 1;
            }

            // Sleep for a short duration (60 FPS = ~16ms)
            tokio::time::sleep(Duration::from_millis(16)).await;
        }

        // Mark as complete
        {
            let mut stage = self.current_stage.write().await;
            *stage = AwakeningStage::Complete;
        }

        tracing::info!("✅ Awakening experience complete");

        // Check if tutorial mode should be activated
        let tutorial_mode = std::env::var("SHOWCASE_MODE")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(false);

        if tutorial_mode {
            tracing::info!("🎭 Tutorial mode detected - will load demonstration scenarios");
        }

        Ok(tutorial_mode)
    }

    /// Process a single timeline event
    async fn process_event(&self, event: &TimelineEvent) -> Result<()> {
        match &event.event_type {
            TimelineEventType::StageTransition { stage } => {
                tracing::info!("🌸 Stage transition: {:?}", stage);

                // Update current stage
                {
                    let mut current = self.current_stage.write().await;
                    *current = *stage;
                }

                // Broadcast state update
                self.engine
                    .events()
                    .broadcast(EngineEvent::StateUpdate {
                        key: "awakening_stage".to_string(),
                        value: serde_json::json!({
                            "stage": format!("{:?}", stage),
                            "time": event.time,
                        }),
                    })
                    .await
                    .ok();
            }

            TimelineEventType::VisualFrame { frame } => {
                tracing::debug!("🎨 Visual frame: {}", frame);

                // Broadcast visual update
                self.engine
                    .events()
                    .broadcast(EngineEvent::StateUpdate {
                        key: "awakening_visual_frame".to_string(),
                        value: serde_json::json!({
                            "frame": frame,
                            "time": event.time,
                        }),
                    })
                    .await
                    .ok();
            }

            TimelineEventType::AudioStart { layer } => {
                tracing::debug!("🔊 Audio start: {}", layer);

                // Broadcast audio start
                self.engine
                    .events()
                    .broadcast(EngineEvent::StateUpdate {
                        key: "awakening_audio".to_string(),
                        value: serde_json::json!({
                            "action": "start",
                            "layer": layer,
                            "time": event.time,
                        }),
                    })
                    .await
                    .ok();
            }

            TimelineEventType::AudioStop { layer } => {
                tracing::debug!("🔇 Audio stop: {}", layer);

                // Broadcast audio stop
                self.engine
                    .events()
                    .broadcast(EngineEvent::StateUpdate {
                        key: "awakening_audio".to_string(),
                        value: serde_json::json!({
                            "action": "stop",
                            "layer": layer,
                            "time": event.time,
                        }),
                    })
                    .await
                    .ok();
            }

            TimelineEventType::TextMessage { message } => {
                tracing::info!("📄 {}", message);

                // Broadcast text message
                self.engine
                    .events()
                    .broadcast(EngineEvent::StateUpdate {
                        key: "awakening_text".to_string(),
                        value: serde_json::json!({
                            "message": message,
                            "stage": format!("{:?}", event.stage),
                            "time": event.time,
                        }),
                    })
                    .await
                    .ok();
            }

            TimelineEventType::Discovery { primal, index } => {
                tracing::info!("🔍 Discovery: {}", primal);

                // Broadcast discovery event
                self.engine
                    .events()
                    .broadcast(EngineEvent::StateUpdate {
                        key: "awakening_discovery".to_string(),
                        value: serde_json::json!({
                            "primal": primal,
                            "index": index,
                            "time": event.time,
                        }),
                    })
                    .await
                    .ok();
            }
        }

        Ok(())
    }
}
