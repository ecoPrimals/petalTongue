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
    fn add_stage_events(
        events: &mut Vec<TimelineEvent>,
        time: f32,
        stage: AwakeningStage,
        audio_layer: Option<&str>,
        text_message: Option<&str>,
        config: &AwakeningConfig,
    ) {
        events.push(TimelineEvent {
            time,
            stage,
            event_type: TimelineEventType::StageTransition { stage },
        });
        if config.modality.audio_enabled
            && let Some(layer) = audio_layer
        {
            events.push(TimelineEvent {
                time,
                stage,
                event_type: TimelineEventType::AudioStart {
                    layer: layer.to_string(),
                },
            });
        }
        if config.modality.text_enabled
            && let Some(message) = text_message
        {
            events.push(TimelineEvent {
                time,
                stage,
                event_type: TimelineEventType::TextMessage {
                    message: message.to_string(),
                },
            });
        }
    }

    #[expect(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    fn add_duration(time: &mut f32, duration: u64) {
        *time += duration as f64 as f32;
    }

    /// Create standard awakening timeline
    #[must_use]
    pub fn standard(config: &AwakeningConfig) -> Self {
        let mut events = Vec::new();
        let mut time = 0.0;

        Self::add_stage_events(
            &mut events,
            0.0,
            AwakeningStage::Awakening,
            Some("signature-tone"),
            Some("Awakening..."),
            config,
        );
        Self::add_duration(&mut time, config.stage_1_duration);

        Self::add_stage_events(
            &mut events,
            time,
            AwakeningStage::SelfKnowledge,
            Some("heartbeat"),
            Some("I am petalTongue. I know myself."),
            config,
        );
        Self::add_duration(&mut time, config.stage_2_duration);

        Self::add_stage_events(
            &mut events,
            time,
            AwakeningStage::Discovery,
            Some("wind"),
            Some("Discovering..."),
            config,
        );

        #[expect(clippy::cast_possible_truncation)]
        for i in 0u32..3 {
            let discovery_time = time + (f64::from(i) as f32 + 1.0);
            events.push(TimelineEvent {
                time: discovery_time,
                stage: AwakeningStage::Discovery,
                event_type: TimelineEventType::Discovery {
                    primal: format!("primal-{i}"),
                    index: i,
                },
            });
            if config.modality.audio_enabled {
                events.push(TimelineEvent {
                    time: discovery_time,
                    stage: AwakeningStage::Discovery,
                    event_type: TimelineEventType::AudioStart {
                        layer: format!("chime-{i}"),
                    },
                });
            }
        }
        Self::add_duration(&mut time, config.stage_3_duration);

        Self::add_stage_events(
            &mut events,
            time,
            AwakeningStage::Tutorial,
            Some("completion"),
            Some("Ready. Let me show you."),
            config,
        );
        Self::add_duration(&mut time, config.stage_4_duration);

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
    pub const fn duration(&self) -> f32 {
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
        assert!((timeline.duration() - 12.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_timeline_stages() {
        let config = AwakeningConfig::default();
        let timeline = AwakeningTimeline::standard(&config);

        // Check stage transitions exist
        let stage_transition_count = timeline
            .events()
            .iter()
            .filter(|e| matches!(e.event_type, TimelineEventType::StageTransition { .. }))
            .count();
        assert_eq!(stage_transition_count, 5); // 4 stages + complete
    }

    #[test]
    fn test_timeline_audio_events() {
        let config = AwakeningConfig::default();
        let timeline = AwakeningTimeline::standard(&config);

        // Check audio events exist
        let has_audio_starts = timeline
            .events()
            .iter()
            .any(|e| matches!(e.event_type, TimelineEventType::AudioStart { .. }));
        assert!(has_audio_starts);
    }

    #[test]
    fn test_timeline_text_events() {
        let config = AwakeningConfig::default();
        let timeline = AwakeningTimeline::standard(&config);

        // Check text events exist
        let has_text_messages = timeline
            .events()
            .iter()
            .any(|e| matches!(e.event_type, TimelineEventType::TextMessage { .. }));
        assert!(has_text_messages);
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

        assert!((coordinator.current_time().await - 0.0).abs() < f32::EPSILON);
        assert_eq!(coordinator.current_stage().await, AwakeningStage::Awakening);
    }

    #[tokio::test]
    async fn test_coordinator_disabled() {
        let engine = Arc::new(UniversalRenderingEngine::new().unwrap());
        let config = AwakeningConfig {
            enabled: false,
            ..Default::default()
        };

        let coordinator = AwakeningCoordinator::new(engine, config);

        let result = coordinator.run().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_timeline_events_in_range_empty() {
        let config = AwakeningConfig::default();
        let timeline = AwakeningTimeline::standard(&config);
        let events = timeline.events_in_range(1000.0, 2000.0);
        assert!(events.is_empty());
    }

    #[test]
    fn test_timeline_events_in_range_boundary() {
        let config = AwakeningConfig::default();
        let timeline = AwakeningTimeline::standard(&config);
        let events = timeline.events_in_range(0.0, 0.001);
        assert!(!events.is_empty());
        for e in events {
            assert!(e.time >= 0.0 && e.time < 0.001);
        }
    }

    #[test]
    fn test_timeline_discovery_events() {
        let config = AwakeningConfig::default();
        let timeline = AwakeningTimeline::standard(&config);
        let discovery_count = timeline
            .events()
            .iter()
            .filter(|e| matches!(e.event_type, TimelineEventType::Discovery { .. }))
            .count();
        assert_eq!(discovery_count, 3);
    }

    #[test]
    fn test_timeline_events_ordered_by_time() {
        let config = AwakeningConfig::default();
        let timeline = AwakeningTimeline::standard(&config);
        let events = timeline.events();
        for i in 1..events.len() {
            assert!(events[i].time >= events[i - 1].time);
        }
    }

    #[test]
    fn test_timeline_audio_disabled() {
        let config = AwakeningConfig {
            modality: crate::awakening::AwakeningModalityFlags {
                visual_enabled: true,
                audio_enabled: false,
                text_enabled: true,
            },
            ..Default::default()
        };
        let timeline = AwakeningTimeline::standard(&config);
        let has_audio_starts = timeline
            .events()
            .iter()
            .any(|e| matches!(e.event_type, TimelineEventType::AudioStart { .. }));
        assert!(!has_audio_starts);
    }

    #[test]
    fn test_timeline_text_disabled() {
        let config = AwakeningConfig {
            modality: crate::awakening::AwakeningModalityFlags {
                visual_enabled: true,
                audio_enabled: true,
                text_enabled: false,
            },
            ..Default::default()
        };
        let timeline = AwakeningTimeline::standard(&config);
        let has_text_events = timeline
            .events()
            .iter()
            .any(|e| matches!(e.event_type, TimelineEventType::TextMessage { .. }));
        assert!(!has_text_events);
    }

    #[test]
    fn test_timeline_duration_sum() {
        let config = AwakeningConfig {
            stage_1_duration: 1,
            stage_2_duration: 1,
            stage_3_duration: 1,
            stage_4_duration: 1,
            ..Default::default()
        };
        let timeline = AwakeningTimeline::standard(&config);
        assert!((timeline.duration() - 4.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_timeline_all_modalities_disabled() {
        let config = AwakeningConfig {
            modality: crate::awakening::AwakeningModalityFlags {
                visual_enabled: false,
                audio_enabled: false,
                text_enabled: false,
            },
            ..Default::default()
        };
        let timeline = AwakeningTimeline::standard(&config);
        // Should still have stage transitions and complete
        let stage_transition_count = timeline
            .events()
            .iter()
            .filter(|e| matches!(e.event_type, TimelineEventType::StageTransition { .. }))
            .count();
        assert_eq!(stage_transition_count, 5);
        assert!(!timeline.events().is_empty());
    }

    #[test]
    fn test_timeline_stage_order() {
        let config = AwakeningConfig::default();
        let timeline = AwakeningTimeline::standard(&config);
        let stages: Vec<_> = timeline
            .events()
            .iter()
            .filter_map(|e| {
                if let TimelineEventType::StageTransition { stage } = &e.event_type {
                    Some(*stage)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(stages[0], AwakeningStage::Awakening);
        assert_eq!(stages[1], AwakeningStage::SelfKnowledge);
        assert_eq!(stages[2], AwakeningStage::Discovery);
        assert_eq!(stages[3], AwakeningStage::Tutorial);
        assert_eq!(stages[4], AwakeningStage::Complete);
    }

    #[test]
    fn test_timeline_events_in_range_exclusive_end() {
        let config = AwakeningConfig::default();
        let timeline = AwakeningTimeline::standard(&config);
        let events_at_0 = timeline.events_in_range(0.0, 0.0);
        assert!(events_at_0.is_empty());
        let events_at_start = timeline.events_in_range(0.0, 0.01);
        assert!(!events_at_start.is_empty());
    }

    #[test]
    fn test_timeline_visual_frame_event_type() {
        let event = TimelineEvent {
            time: 1.0,
            stage: AwakeningStage::Awakening,
            event_type: TimelineEventType::VisualFrame { frame: 42 },
        };
        assert!((event.time - 1.0).abs() < f32::EPSILON);
        assert_eq!(event.stage, AwakeningStage::Awakening);
        assert!(matches!(
            event.event_type,
            TimelineEventType::VisualFrame { frame: 42 }
        ));
    }

    #[test]
    fn test_timeline_audio_stop_event_type() {
        let event = TimelineEvent {
            time: 2.0,
            stage: AwakeningStage::SelfKnowledge,
            event_type: TimelineEventType::AudioStop {
                layer: "heartbeat".to_string(),
            },
        };
        assert!(matches!(
            event.event_type,
            TimelineEventType::AudioStop { layer } if layer == "heartbeat"
        ));
    }

    #[test]
    fn test_timeline_zero_duration_stages() {
        let config = AwakeningConfig {
            stage_1_duration: 0,
            stage_2_duration: 0,
            stage_3_duration: 0,
            stage_4_duration: 0,
            ..Default::default()
        };
        let timeline = AwakeningTimeline::standard(&config);
        assert!((timeline.duration() - 0.0).abs() < f32::EPSILON);
        assert!(!timeline.events().is_empty());
    }

    #[test]
    fn test_awakening_stage_equality() {
        assert_eq!(AwakeningStage::Awakening, AwakeningStage::Awakening);
        assert_ne!(AwakeningStage::Awakening, AwakeningStage::Complete);
    }
}
