// SPDX-License-Identifier: AGPL-3.0-or-later
//! Central Nervous System - sensory feedback and display verification
//!
//! Extracted from app.rs - handles motor command recording, sensory feedback
//! from user input, and periodic display visibility verification.

use super::PetalTongueApp;
use petal_tongue_core::constants::PRIMAL_NAME;
use std::time::{Duration, Instant};

/// Process sensory feedback from user input and run display verification.
pub(super) fn process_sensory_feedback(app: &mut PetalTongueApp, ctx: &egui::Context) {
    app.frame_count += 1;
    app.proprioception.record_frame();

    if let Ok(mut awareness) = app.rendering_awareness.write() {
        awareness.motor_command(petal_tongue_core::MotorCommand::RenderFrame {
            frame_id: app.frame_count,
        });
    }

    ctx.input(|i| {
        if i.pointer.any_click()
            && let Some(pos) = i.pointer.interact_pos()
        {
            let event = petal_tongue_core::SensorEvent::Click {
                x: pos.x,
                y: pos.y,
                button: petal_tongue_core::MouseButton::Left,
                timestamp: Instant::now(),
            };
            if let Ok(mut awareness) = app.rendering_awareness.write() {
                awareness.sensory_feedback(&event);
            }
            app.proprioception
                .input_received(&crate::input_verification::InputModality::Pointer);
        }

        if let Some(pos) = i.pointer.hover_pos() {
            let event = petal_tongue_core::SensorEvent::Position {
                x: pos.x,
                y: pos.y,
                timestamp: Instant::now(),
            };
            if let Ok(mut awareness) = app.rendering_awareness.write() {
                awareness.sensory_feedback(&event);
            }
            app.proprioception
                .input_received(&crate::input_verification::InputModality::Pointer);
        }

        for key_event in &i.events {
            if let egui::Event::Key { .. } = key_event {
                let event = petal_tongue_core::SensorEvent::KeyPress {
                    key: petal_tongue_core::Key::Unknown,
                    modifiers: petal_tongue_core::Modifiers::none(),
                    timestamp: Instant::now(),
                };
                if let Ok(mut awareness) = app.rendering_awareness.write() {
                    awareness.sensory_feedback(&event);
                }
                app.proprioception
                    .input_received(&crate::input_verification::InputModality::Keyboard);
            }
        }
    });

    let now = Instant::now();
    if now.duration_since(app.last_display_verification) > Duration::from_secs(5) {
        app.last_display_verification = now;

        let last_interaction_secs = app.rendering_awareness.read().map_or(999.0, |awareness| {
            awareness.time_since_last_interaction().as_secs_f32()
        });

        let verification = crate::display_verification::continuous_verification(
            PRIMAL_NAME,
            last_interaction_secs,
        );

        tracing::debug!(
            "🔍 Display verification: {} (visible: {}, wm_responsive: {})",
            verification.status_message,
            verification.window_visible,
            verification.wm_responsive
        );

        if !verification.window_visible && verification.display_server_available {
            tracing::warn!(
                "⚠️  Display substrate verification: Window may not be visible! Status: {}",
                verification.status_message
            );
        }

        let state = app.proprioception.assess();
        tracing::debug!(
            "🧠 Proprioception: Health={:.0}% Confidence={:.0}% Motor={} Sensory={} Loop={}",
            state.health * 100.0,
            state.confidence * 100.0,
            state.motor_functional,
            state.sensory_functional,
            state.loop_complete
        );
    }
}
