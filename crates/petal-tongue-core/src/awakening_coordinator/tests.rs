// SPDX-License-Identifier: AGPL-3.0-or-later
//! Awakening coordinator unit tests.

use super::*;
use crate::awakening::AwakeningStage;
use crate::engine::UniversalRenderingEngine;
use std::sync::Arc;

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

    let stage_transition_count = timeline
        .events()
        .iter()
        .filter(|e| matches!(e.event_type, TimelineEventType::StageTransition { .. }))
        .count();
    assert_eq!(stage_transition_count, 5);
}

#[test]
fn test_timeline_audio_events() {
    let config = AwakeningConfig::default();
    let timeline = AwakeningTimeline::standard(&config);

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

    let stage1_events = timeline.events_in_range(0.0, 3.0);
    assert!(!stage1_events.is_empty());

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

#[tokio::test]
async fn test_coordinator_run_completes_with_zero_duration() {
    let engine = Arc::new(UniversalRenderingEngine::new().unwrap());
    let config = AwakeningConfig {
        stage_1_duration: 0,
        stage_2_duration: 0,
        stage_3_duration: 0,
        stage_4_duration: 0,
        ..Default::default()
    };
    let coordinator = AwakeningCoordinator::new(engine, config);
    let result = coordinator.run().await;
    assert!(result.is_ok());
    assert_eq!(coordinator.current_stage().await, AwakeningStage::Complete);
}

#[tokio::test]
async fn test_coordinator_run_tutorial_mode_true() {
    let config = AwakeningConfig {
        stage_1_duration: 0,
        stage_2_duration: 0,
        stage_3_duration: 0,
        stage_4_duration: 0,
        ..Default::default()
    };
    let result = crate::test_fixtures::env_test_helpers::with_env_var_async(
        "SHOWCASE_MODE",
        "true",
        || async {
            let coord = AwakeningCoordinator::new(
                Arc::new(UniversalRenderingEngine::new().unwrap()),
                config,
            );
            coord.run().await
        },
    )
    .await;
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_timeline_event_type_discovery() {
    let event = TimelineEvent {
        time: 5.0,
        stage: AwakeningStage::Discovery,
        event_type: TimelineEventType::Discovery {
            primal: "test-primal".to_string(),
            index: 1,
        },
    };
    assert!(matches!(
        event.event_type,
        TimelineEventType::Discovery { primal, index } if primal == "test-primal" && index == 1
    ));
}

#[test]
fn test_timeline_event_type_text_message() {
    let event = TimelineEvent {
        time: 1.0,
        stage: AwakeningStage::Awakening,
        event_type: TimelineEventType::TextMessage {
            message: "test message".to_string(),
        },
    };
    assert!(matches!(
        event.event_type,
        TimelineEventType::TextMessage { message } if message == "test message"
    ));
}

#[test]
fn test_timeline_event_type_audio_start() {
    let event = TimelineEvent {
        time: 0.0,
        stage: AwakeningStage::Awakening,
        event_type: TimelineEventType::AudioStart {
            layer: "signature-tone".to_string(),
        },
    };
    assert!(matches!(
        event.event_type,
        TimelineEventType::AudioStart { layer } if layer == "signature-tone"
    ));
}

#[test]
fn test_timeline_event_type_stage_transition() {
    let event = TimelineEvent {
        time: 3.0,
        stage: AwakeningStage::SelfKnowledge,
        event_type: TimelineEventType::StageTransition {
            stage: AwakeningStage::SelfKnowledge,
        },
    };
    assert!(matches!(
        event.event_type,
        TimelineEventType::StageTransition { stage } if stage == AwakeningStage::SelfKnowledge
    ));
}

#[test]
fn test_timeline_events_in_range_overlap() {
    let config = AwakeningConfig::default();
    let timeline = AwakeningTimeline::standard(&config);
    let events = timeline.events_in_range(5.0, 8.0);
    assert!(!events.is_empty());
    for e in events {
        assert!(e.time >= 5.0 && e.time < 8.0);
    }
}

#[test]
fn test_timeline_events_at_exact_stage_boundary() {
    let config = AwakeningConfig::default();
    let timeline = AwakeningTimeline::standard(&config);
    let events_at_3 = timeline.events_in_range(2.99, 3.01);
    assert!(!events_at_3.is_empty());
    let has_self_knowledge = events_at_3.iter().any(|e| {
        matches!(
            e.event_type,
            TimelineEventType::StageTransition {
                stage: AwakeningStage::SelfKnowledge
            }
        )
    });
    assert!(has_self_knowledge);
}

#[test]
fn test_timeline_events_at_complete_boundary() {
    let config = AwakeningConfig::default();
    let timeline = AwakeningTimeline::standard(&config);
    let events_at_end =
        timeline.events_in_range(timeline.duration() - 0.01, timeline.duration() + 0.01);
    let has_complete = events_at_end.iter().any(|e| {
        matches!(
            e.event_type,
            TimelineEventType::StageTransition {
                stage: AwakeningStage::Complete
            }
        )
    });
    assert!(has_complete);
}

#[test]
fn test_timeline_event_time_properties() {
    let event = TimelineEvent {
        time: 1.5,
        stage: AwakeningStage::Awakening,
        event_type: TimelineEventType::StageTransition {
            stage: AwakeningStage::Awakening,
        },
    };
    assert!((event.time - 1.5).abs() < f32::EPSILON);
    assert_eq!(event.stage, AwakeningStage::Awakening);
}

#[test]
fn test_awakening_stage_all_variants() {
    let _ = AwakeningStage::SelfKnowledge;
    let _ = AwakeningStage::Discovery;
    let _ = AwakeningStage::Tutorial;
    let _ = AwakeningStage::Complete;
}

#[test]
fn test_timeline_duration_positive() {
    let config = AwakeningConfig::default();
    let timeline = AwakeningTimeline::standard(&config);
    assert!(timeline.duration() > 0.0);
}

#[test]
fn test_timeline_first_event_at_zero() {
    let config = AwakeningConfig::default();
    let timeline = AwakeningTimeline::standard(&config);
    let first = timeline.events().first().expect("events");
    assert!((first.time - 0.0).abs() < f32::EPSILON);
}

#[test]
fn test_timeline_last_event_at_duration() {
    let config = AwakeningConfig::default();
    let timeline = AwakeningTimeline::standard(&config);
    let last = timeline.events().last().expect("events");
    assert!((last.time - timeline.duration()).abs() < f32::EPSILON);
}
