// SPDX-License-Identifier: AGPL-3.0-or-later
//! Unit tests for rendering awareness.

use super::*;
use crate::frame_introspection::{
    BindingType, BoundDataObject, FrameIntrospection, PanelKind, PanelSnapshot,
};
use crate::sensor::MouseButton;
use std::time::{Duration, Instant};

#[test]
fn test_motor_command() {
    let mut awareness = RenderingAwareness::new();

    let cmd_id = awareness.motor_command(MotorCommand::RenderFrame { frame_id: 1 });
    assert_eq!(cmd_id, 1);
    assert_eq!(awareness.metrics().commands_sent, 1);
}

#[test]
fn test_sensory_feedback() {
    let mut awareness = RenderingAwareness::new();

    // Send a frame
    let frame_id = awareness.motor_command(MotorCommand::RenderFrame { frame_id: 1 });

    // Confirm it
    let event = SensorEvent::FrameAcknowledged {
        frame_id,
        timestamp: Instant::now(),
    };
    awareness.sensory_feedback(&event);

    assert_eq!(awareness.metrics.frames_confirmed, 1);
}

#[test]
fn test_user_interaction_tracking() {
    let mut awareness = RenderingAwareness::new();

    let click = SensorEvent::Click {
        x: 100.0,
        y: 200.0,
        button: MouseButton::Left,
        timestamp: Instant::now(),
    };

    awareness.sensory_feedback(&click);

    assert_eq!(awareness.metrics.user_interactions, 1);
    let assessment = awareness.assess_self();
    assert_eq!(assessment.user_interactivity, InteractivityState::Active);
}

#[test]
fn test_bidirectional_loop() {
    let mut awareness = RenderingAwareness::new();

    // Motor: send frame
    let frame_id = awareness.motor_command(MotorCommand::RenderFrame { frame_id: 1 });

    // Sensory: confirm frame
    awareness.sensory_feedback(&SensorEvent::FrameAcknowledged {
        frame_id,
        timestamp: Instant::now(),
    });

    // Sensory: heartbeat
    awareness.sensory_feedback(&SensorEvent::Heartbeat {
        latency: Duration::from_millis(10),
        timestamp: Instant::now(),
    });

    let assessment = awareness.assess_self();
    assert!(assessment.can_render);
    assert!(assessment.can_sense);
    assert!(assessment.is_complete_loop);
}

#[test]
fn test_health_percentage() {
    let mut awareness = RenderingAwareness::new();

    // Send and confirm multiple frames
    for i in 0..10 {
        let frame_id = awareness.motor_command(MotorCommand::RenderFrame { frame_id: i });
        awareness.sensory_feedback(&SensorEvent::FrameAcknowledged {
            frame_id,
            timestamp: Instant::now(),
        });
    }

    awareness.sensory_feedback(&SensorEvent::Heartbeat {
        latency: Duration::from_millis(5),
        timestamp: Instant::now(),
    });

    let assessment = awareness.assess_self();
    let health = assessment.health_percentage();

    // Should be near 100% (all checks pass)
    assert!(health > 95.0, "Health was {health}");
}

#[test]
fn test_visibility_state_confirmed() {
    let mut awareness = RenderingAwareness::new();
    for i in 0..10u64 {
        let fid = awareness.motor_command(MotorCommand::RenderFrame { frame_id: i });
        awareness.sensory_feedback(&SensorEvent::FrameAcknowledged {
            frame_id: fid,
            timestamp: Instant::now(),
        });
    }
    let assessment = awareness.assess_self();
    assert_eq!(assessment.user_visibility, VisibilityState::Confirmed);
}

#[test]
fn test_visibility_state_probable() {
    let mut awareness = RenderingAwareness::new();
    for i in 0..10u64 {
        awareness.motor_command(MotorCommand::RenderFrame { frame_id: i });
        if i < 6 {
            awareness.sensory_feedback(&SensorEvent::FrameAcknowledged {
                frame_id: i + 1,
                timestamp: Instant::now(),
            });
        }
    }
    let assessment = awareness.assess_self();
    assert_eq!(assessment.user_visibility, VisibilityState::Probable);
}

#[test]
fn test_visibility_state_uncertain() {
    let mut awareness = RenderingAwareness::new();
    awareness.motor_command(MotorCommand::RenderFrame { frame_id: 1 });
    awareness.sensory_feedback(&SensorEvent::FrameAcknowledged {
        frame_id: 1,
        timestamp: Instant::now(),
    });
    for i in 2..10u64 {
        awareness.motor_command(MotorCommand::RenderFrame { frame_id: i });
    }
    let assessment = awareness.assess_self();
    assert_eq!(assessment.user_visibility, VisibilityState::Uncertain);
}

#[test]
fn test_visibility_state_unknown() {
    let awareness = RenderingAwareness::new();
    let assessment = awareness.assess_self();
    assert_eq!(assessment.user_visibility, VisibilityState::Unknown);
}

#[test]
fn test_self_assessment_is_healthy() {
    let mut awareness = RenderingAwareness::new();
    let fid = awareness.motor_command(MotorCommand::RenderFrame { frame_id: 1 });
    awareness.sensory_feedback(&SensorEvent::FrameAcknowledged {
        frame_id: fid,
        timestamp: Instant::now(),
    });
    awareness.sensory_feedback(&SensorEvent::Heartbeat {
        latency: Duration::from_millis(10),
        timestamp: Instant::now(),
    });
    let assessment = awareness.assess_self();
    assert!(assessment.is_healthy());
}

#[test]
fn test_self_assessment_unhealthy_no_motor() {
    let awareness = RenderingAwareness::new();
    let assessment = awareness.assess_self();
    assert!(!assessment.is_healthy());
}

#[test]
fn test_health_percentage_partial() {
    let mut awareness = RenderingAwareness::new();
    awareness.motor_command(MotorCommand::RenderFrame { frame_id: 1 });
    let assessment = awareness.assess_self();
    let health = assessment.health_percentage();
    assert!(health > 0.0 && health < 100.0);
}

#[test]
fn test_panel_id_variants() {
    assert_eq!(PanelId::LeftSidebar, PanelId::LeftSidebar);
    assert_eq!(
        PanelId::Custom("foo".to_string()),
        PanelId::Custom("foo".to_string())
    );
    assert!(PanelId::Custom("a".to_string()) != PanelId::Custom("b".to_string()));
}

#[test]
fn test_motor_command_ids_increment() {
    let mut awareness = RenderingAwareness::new();
    let id1 = awareness.motor_command(MotorCommand::UpdateDisplay);
    let id2 = awareness.motor_command(MotorCommand::ClearDisplay);
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
}

#[test]
fn test_time_since_last_interaction_no_interaction() {
    let awareness = RenderingAwareness::new();
    let elapsed = awareness.time_since_last_interaction();
    assert!(elapsed >= Duration::from_secs(9999));
}

#[test]
fn test_default_rendering_awareness() {
    let awareness = RenderingAwareness::default();
    assert_eq!(awareness.metrics().commands_sent, 0);
    assert_eq!(awareness.metrics().frames_confirmed, 0);
}

#[test]
fn test_interactivity_state_idle() {
    let mut awareness = RenderingAwareness::new();
    let click = SensorEvent::Click {
        x: 0.0,
        y: 0.0,
        button: MouseButton::Left,
        timestamp: Instant::now()
            .checked_sub(Duration::from_secs(60))
            .expect("60s ago should be representable"),
    };
    awareness.sensory_feedback(&click);
    let assessment = awareness.assess_self();
    assert_eq!(assessment.user_interactivity, InteractivityState::Idle);
}

#[test]
fn test_interactivity_state_recent() {
    let mut awareness = RenderingAwareness::new();
    let click = SensorEvent::Click {
        x: 0.0,
        y: 0.0,
        button: MouseButton::Left,
        timestamp: Instant::now()
            .checked_sub(Duration::from_secs(10))
            .expect("10s ago should be representable"),
    };
    awareness.sensory_feedback(&click);
    let assessment = awareness.assess_self();
    assert_eq!(assessment.user_interactivity, InteractivityState::Recent);
}

#[test]
fn test_substrate_health_responsive() {
    let mut awareness = RenderingAwareness::new();
    awareness.motor_command(MotorCommand::RenderFrame { frame_id: 1 });
    awareness.sensory_feedback(&SensorEvent::Heartbeat {
        latency: Duration::from_millis(50),
        timestamp: Instant::now(),
    });
    let assessment = awareness.assess_self();
    assert!(assessment.substrate_responsive);
}

#[test]
fn test_substrate_health_slow() {
    let mut awareness = RenderingAwareness::new();
    awareness.motor_command(MotorCommand::RenderFrame { frame_id: 1 });
    awareness.sensory_feedback(&SensorEvent::Heartbeat {
        latency: Duration::from_millis(200),
        timestamp: Instant::now(),
    });
    let assessment = awareness.assess_self();
    assert!(!assessment.substrate_responsive);
}

#[test]
fn test_validation_pipeline_unhealthy_many_unconfirmed() {
    let mut awareness = RenderingAwareness::new();
    for i in 0..15u64 {
        awareness.motor_command(MotorCommand::RenderFrame { frame_id: i });
    }
    let assessment = awareness.assess_self();
    assert!(!assessment.is_complete_loop);
    assert!(assessment.confirmation_rate < 100.0);
}

#[test]
fn test_motor_command_variants() {
    let mut awareness = RenderingAwareness::new();
    let _ = awareness.motor_command(MotorCommand::UpdateDisplay);
    let _ = awareness.motor_command(MotorCommand::ClearDisplay);
    let _ = awareness.motor_command(MotorCommand::SetPanelVisibility {
        panel: PanelId::LeftSidebar,
        visible: true,
    });
    let _ = awareness.motor_command(MotorCommand::SetZoom { level: 1.5 });
    assert_eq!(awareness.metrics().commands_sent, 4);
}

#[test]
fn test_health_percentage_zero() {
    let awareness = RenderingAwareness::new();
    let assessment = awareness.assess_self();
    assert!(
        (assessment.health_percentage() - 0.0).abs() < f32::EPSILON,
        "health should be zero for fresh awareness"
    );
}

#[test]
fn test_visibility_state_unknown_no_frames() {
    let awareness = RenderingAwareness::new();
    let assessment = awareness.assess_self();
    assert_eq!(assessment.user_visibility, VisibilityState::Unknown);
}

#[test]
fn test_validation_health_unconfirmed_count() {
    let mut awareness = RenderingAwareness::new();
    for i in 0..5u64 {
        awareness.motor_command(MotorCommand::RenderFrame { frame_id: i });
    }
    let assessment = awareness.assess_self();
    assert!(assessment.confirmation_rate < 100.0);
}

#[test]
fn test_record_frame_content() {
    let mut awareness = RenderingAwareness::new();
    let mut frame = FrameIntrospection::empty(1);
    frame
        .visible_panels
        .push(PanelSnapshot::visible(PanelId::TopMenu, PanelKind::TopMenu));
    awareness.record_frame_content(frame);
    assert!(awareness.current_content().is_some());
    assert_eq!(awareness.visible_panels().len(), 1);
    assert!(awareness.is_panel_visible(PanelKind::TopMenu));
}

#[test]
fn test_is_showing_data() {
    let mut awareness = RenderingAwareness::new();
    let mut frame = FrameIntrospection::empty(1);
    frame.bound_data.push(BoundDataObject {
        panel_id: PanelId::TopMenu,
        data_object_id: "node-x".into(),
        binding_type: BindingType::GraphNode,
    });
    awareness.record_frame_content(frame);
    assert!(awareness.is_showing_data("node-x"));
    assert!(!awareness.is_showing_data("node-y"));
}

#[test]
fn test_content_accessor() {
    let awareness = RenderingAwareness::new();
    let content = awareness.content();
    assert!(!content.has_content());
}

#[test]
fn test_interactivity_state_unconfirmed() {
    let awareness = RenderingAwareness::new();
    let assessment = awareness.assess_self();
    assert_eq!(
        assessment.user_interactivity,
        InteractivityState::Unconfirmed
    );
}
