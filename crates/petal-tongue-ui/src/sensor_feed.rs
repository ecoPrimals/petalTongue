// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sensor event feed: converts egui input events to IPC-serializable sensor events.
//!
//! Pure function: reads `egui::InputState`, returns `Vec<SensorEventIpc>`.
//! Called each frame from `update_headless()` and broadcast to the
//! `SensorStreamRegistry` for external subscribers (ludoSpring, Squirrel, etc.).

use petal_tongue_core::{KeyModifiersIpc, SensorEventIpc};

/// Collect sensor events from the current egui frame's input state.
///
/// Reads pointer position, clicks, key presses/releases, and scroll deltas.
/// Returns an empty vec when no relevant input occurred.
#[must_use]
pub fn collect_sensor_events(ctx: &egui::Context) -> Vec<SensorEventIpc> {
    let mut events = Vec::new();
    let now_ms = epoch_ms();

    ctx.input(|input| {
        // Pointer movement (only when position changed and pointer is over the area)
        if let Some(pos) = input.pointer.latest_pos() {
            if input.pointer.is_moving() {
                events.push(SensorEventIpc::PointerMove {
                    x: pos.x,
                    y: pos.y,
                    timestamp_ms: now_ms,
                });
            }
        }

        // Pointer clicks
        if input.pointer.any_pressed() {
            if let Some(pos) = input.pointer.interact_pos() {
                let button = if input.pointer.button_pressed(egui::PointerButton::Primary) {
                    "left"
                } else if input.pointer.button_pressed(egui::PointerButton::Secondary) {
                    "right"
                } else {
                    "middle"
                };
                events.push(SensorEventIpc::Click {
                    x: pos.x,
                    y: pos.y,
                    button: button.to_string(),
                    timestamp_ms: now_ms,
                });
            }
        }

        // Scroll
        let scroll = input.smooth_scroll_delta;
        if scroll.x.abs() > 0.1 || scroll.y.abs() > 0.1 {
            events.push(SensorEventIpc::Scroll {
                delta_x: scroll.x,
                delta_y: scroll.y,
                timestamp_ms: now_ms,
            });
        }

        // Key presses and releases from egui events
        for event in &input.events {
            match event {
                egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } => {
                    events.push(SensorEventIpc::KeyPress {
                        key: format!("{key:?}"),
                        modifiers: KeyModifiersIpc {
                            ctrl: modifiers.ctrl,
                            alt: modifiers.alt,
                            shift: modifiers.shift,
                            meta: modifiers.command,
                        },
                        timestamp_ms: now_ms,
                    });
                }
                egui::Event::Key {
                    key,
                    pressed: false,
                    ..
                } => {
                    events.push(SensorEventIpc::KeyRelease {
                        key: format!("{key:?}"),
                        timestamp_ms: now_ms,
                    });
                }
                _ => {}
            }
        }
    });

    events
}

fn epoch_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_ms_returns_nonzero() {
        assert!(epoch_ms() > 0);
    }

    #[test]
    fn collect_on_fresh_context_returns_empty() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |_ctx| {});
        let events = collect_sensor_events(&ctx);
        assert!(events.is_empty(), "fresh context should produce no events");
    }

    #[test]
    fn key_press_event_produced() {
        let ctx = egui::Context::default();
        let mut raw = egui::RawInput::default();
        raw.events.push(egui::Event::Key {
            key: egui::Key::A,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers {
                ctrl: true,
                ..Default::default()
            },
        });
        let _ = ctx.run(raw, |_ctx| {});
        let events = collect_sensor_events(&ctx);
        assert!(
            events.iter().any(
                |e| matches!(e, SensorEventIpc::KeyPress { key, modifiers, .. }
                if key.contains('A') && modifiers.ctrl)
            ),
            "should produce KeyPress for A with ctrl"
        );
    }

    #[test]
    fn key_release_event_produced() {
        let ctx = egui::Context::default();
        let mut raw = egui::RawInput::default();
        raw.events.push(egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: false,
            repeat: false,
            modifiers: Default::default(),
        });
        let _ = ctx.run(raw, |_ctx| {});
        let events = collect_sensor_events(&ctx);
        assert!(
            events.iter().any(
                |e| matches!(e, SensorEventIpc::KeyRelease { key, .. } if key.contains("Escape"))
            ),
            "should produce KeyRelease for Escape"
        );
    }
}
