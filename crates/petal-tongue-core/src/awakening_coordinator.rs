// SPDX-License-Identifier: AGPL-3.0-only
//! # Awakening Coordinator
//!
//! Coordinates visual, audio, and text across the 4-stage awakening timeline.

use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::awakening::{AwakeningConfig, AwakeningStage};
use crate::engine::UniversalRenderingEngine;
use crate::event::EngineEvent;

/// Timeline Event
///
/// Represents a synchronized event across all modalities.
#[derive(Debug, Clone)]
pub struct TimelineEvent {
    /// Time offset from start (seconds)
    pub time: f32,

    /// Stage this event belongs to
    pub stage: AwakeningStage,

    /// Event type
    pub event_type: TimelineEventType,
}

/// Timeline Event Type
#[derive(Debug, Clone)]
pub enum TimelineEventType {
    /// Stage transition
    StageTransition {
        /// New stage
        stage: AwakeningStage,
    },

    /// Visual frame update
    VisualFrame {
        /// Frame index
        frame: usize,
    },

    /// Audio layer start
    AudioStart {
        /// Layer name
        layer: String,
    },

    /// Audio layer stop
    AudioStop {
        /// Layer name
        layer: String,
    },

    /// Text message
    TextMessage {
        /// Message content
        message: String,
    },

    /// Discovery event (primal found)
    Discovery {
        /// Primal name
        primal: String,
        /// Index for audio chime
        index: u32,
    },
}

/// Awakening Timeline
///
/// Defines the complete awakening sequence with synchronized events.
pub struct AwakeningTimeline {
    /// All timeline events
    events: Vec<TimelineEvent>,

    /// Total duration (seconds)
    duration: f32,
}

impl AwakeningTimeline {
    /// Create standard awakening timeline
    #[must_use]
    pub fn standard(config: &AwakeningConfig) -> Self {
        let mut events = Vec::new();
        let mut time = 0.0;

        // Stage 1: Awakening (0-3s)
        events.push(TimelineEvent {
            time: 0.0,
            stage: AwakeningStage::Awakening,
            event_type: TimelineEventType::StageTransition {
                stage: AwakeningStage::Awakening,
            },
        });

        if config.audio_enabled {
            events.push(TimelineEvent {
                time: 0.0,
                stage: AwakeningStage::Awakening,
                event_type: TimelineEventType::AudioStart {
                    layer: "signature-tone".to_string(),
                },
            });
        }

        if config.text_enabled {
            events.push(TimelineEvent {
                time: 0.0,
                stage: AwakeningStage::Awakening,
                event_type: TimelineEventType::TextMessage {
                    message: "Awakening...".to_string(),
                },
            });
        }

        time += config.stage_1_duration as f32;

        // Stage 2: Self-Knowledge (3-6s)
        events.push(TimelineEvent {
            time,
            stage: AwakeningStage::SelfKnowledge,
            event_type: TimelineEventType::StageTransition {
                stage: AwakeningStage::SelfKnowledge,
            },
        });

        if config.audio_enabled {
            events.push(TimelineEvent {
                time,
                stage: AwakeningStage::SelfKnowledge,
                event_type: TimelineEventType::AudioStart {
                    layer: "heartbeat".to_string(),
                },
            });
        }

        if config.text_enabled {
            events.push(TimelineEvent {
                time,
                stage: AwakeningStage::SelfKnowledge,
                event_type: TimelineEventType::TextMessage {
                    message: "I am petalTongue. I know myself.".to_string(),
                },
            });
        }

        time += config.stage_2_duration as f32;

        // Stage 3: Discovery (6-10s)
        events.push(TimelineEvent {
            time,
            stage: AwakeningStage::Discovery,
            event_type: TimelineEventType::StageTransition {
                stage: AwakeningStage::Discovery,
            },
        });

        if config.audio_enabled {
            events.push(TimelineEvent {
                time,
                stage: AwakeningStage::Discovery,
                event_type: TimelineEventType::AudioStart {
                    layer: "wind".to_string(),
                },
            });
        }

        if config.text_enabled {
            events.push(TimelineEvent {
                time,
                stage: AwakeningStage::Discovery,
                event_type: TimelineEventType::TextMessage {
                    message: "Discovering...".to_string(),
                },
            });
        }

        // Discovery events at 1s intervals (simulated)
        // In real implementation, these would be triggered by actual discoveries
        for i in 0..3 {
            let discovery_time = time + (i as f32 + 1.0);
            events.push(TimelineEvent {
                time: discovery_time,
                stage: AwakeningStage::Discovery,
                event_type: TimelineEventType::Discovery {
                    primal: format!("primal-{i}"),
                    index: i as u32,
                },
            });

            if config.audio_enabled {
                events.push(TimelineEvent {
                    time: discovery_time,
                    stage: AwakeningStage::Discovery,
                    event_type: TimelineEventType::AudioStart {
                        layer: format!("chime-{i}"),
                    },
                });
            }
        }

        time += config.stage_3_duration as f32;

        // Stage 4: Tutorial (10-12s)
        events.push(TimelineEvent {
            time,
            stage: AwakeningStage::Tutorial,
            event_type: TimelineEventType::StageTransition {
                stage: AwakeningStage::Tutorial,
            },
        });

        if config.audio_enabled {
            events.push(TimelineEvent {
                time,
                stage: AwakeningStage::Tutorial,
                event_type: TimelineEventType::AudioStart {
                    layer: "completion".to_string(),
                },
            });
        }

        if config.text_enabled {
            events.push(TimelineEvent {
                time,
                stage: AwakeningStage::Tutorial,
                event_type: TimelineEventType::TextMessage {
                    message: "Ready. Let me show you.".to_string(),
                },
            });
        }

        time += config.stage_4_duration as f32;

        // Final event: Complete
        events.push(TimelineEvent {
            time,
            stage: AwakeningStage::Complete,
            event_type: TimelineEventType::StageTransition {
                stage: AwakeningStage::Complete,
            },
        });

        Self {
            events,
            duration: time,
        }
    }

    /// Get all events
    #[must_use]
    pub fn events(&self) -> &[TimelineEvent] {
        &self.events
    }

    /// Get total duration
    #[must_use]
    pub fn duration(&self) -> f32 {
        self.duration
    }

    /// Get events for a specific time range
    #[must_use]
    pub fn events_in_range(&self, start: f32, end: f32) -> Vec<&TimelineEvent> {
        self.events
            .iter()
            .filter(|e| e.time >= start && e.time < end)
            .collect()
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeline_creation() {
        let config = AwakeningConfig::default();
        let timeline = AwakeningTimeline::standard(&config);

        assert!(!timeline.events().is_empty());
        assert_eq!(timeline.duration(), 12.0); // 3+3+4+2 = 12 seconds
    }

    #[test]
    fn test_timeline_stages() {
        let config = AwakeningConfig::default();
        let timeline = AwakeningTimeline::standard(&config);

        // Check stage transitions exist
        let stage_transitions: Vec<_> = timeline
            .events()
            .iter()
            .filter(|e| matches!(e.event_type, TimelineEventType::StageTransition { .. }))
            .collect();

        assert_eq!(stage_transitions.len(), 5); // 4 stages + complete
    }

    #[test]
    fn test_timeline_audio_events() {
        let config = AwakeningConfig::default();
        let timeline = AwakeningTimeline::standard(&config);

        // Check audio events exist
        let audio_starts: Vec<_> = timeline
            .events()
            .iter()
            .filter(|e| matches!(e.event_type, TimelineEventType::AudioStart { .. }))
            .collect();

        assert!(!audio_starts.is_empty());
    }

    #[test]
    fn test_timeline_text_events() {
        let config = AwakeningConfig::default();
        let timeline = AwakeningTimeline::standard(&config);

        // Check text events exist
        let text_messages: Vec<_> = timeline
            .events()
            .iter()
            .filter(|e| matches!(e.event_type, TimelineEventType::TextMessage { .. }))
            .collect();

        assert!(!text_messages.is_empty());
    }

    #[test]
    fn test_timeline_events_in_range() {
        let config = AwakeningConfig::default();
        let timeline = AwakeningTimeline::standard(&config);

        // Get events in first 3 seconds (Stage 1)
        let stage1_events = timeline.events_in_range(0.0, 3.0);
        assert!(!stage1_events.is_empty());

        // All should be in Awakening stage
        for event in stage1_events {
            assert_eq!(event.stage, AwakeningStage::Awakening);
        }
    }

    #[tokio::test]
    async fn test_coordinator_creation() {
        let engine = Arc::new(UniversalRenderingEngine::new().unwrap());
        let config = AwakeningConfig::default();
        let coordinator = AwakeningCoordinator::new(engine, config);

        assert_eq!(coordinator.current_time().await, 0.0);
        assert_eq!(coordinator.current_stage().await, AwakeningStage::Awakening);
    }

    #[tokio::test]
    async fn test_coordinator_disabled() {
        let engine = Arc::new(UniversalRenderingEngine::new().unwrap());
        let mut config = AwakeningConfig::default();
        config.enabled = false;

        let coordinator = AwakeningCoordinator::new(engine, config);

        let result = coordinator.run().await;
        assert!(result.is_ok());
    }
}
