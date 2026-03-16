// SPDX-License-Identifier: AGPL-3.0-or-later
//! Awakening timeline definition and event generation.

use super::types::{TimelineEvent, TimelineEventType};
use crate::awakening::{AwakeningConfig, AwakeningStage};

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
