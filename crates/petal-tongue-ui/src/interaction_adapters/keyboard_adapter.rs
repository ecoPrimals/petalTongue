// SPDX-License-Identifier: AGPL-3.0-or-later
//! Keyboard input adapter.
//!
//! Translates keyboard [`SensorEvent`] values into semantic
//! [`InteractionIntent`] values. Maps Enter -> Select, Tab -> Focus(next),
//! arrows -> Navigate, Delete -> Manipulate(Delete), Escape -> Dismiss.

use petal_tongue_core::interaction::{
    InputAdapter, InputModality, InteractionCapability, InteractionContext, InteractionIntent,
    InteractionResult, InteractionTarget, ManipulationOp, NavigationDirection, SelectionMode,
};
use petal_tongue_core::sensor::{Key, SensorEvent};

/// Adapts keyboard sensor events to semantic interaction intents.
pub struct KeyboardAdapter {
    _private: (),
}

impl KeyboardAdapter {
    /// Create a new keyboard adapter.
    #[must_use]
    pub const fn new() -> Self {
        Self { _private: () }
    }
}

impl Default for KeyboardAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl InputAdapter for KeyboardAdapter {
    fn name(&self) -> &'static str {
        "Keyboard"
    }

    fn modality(&self) -> InputModality {
        InputModality::Keyboard
    }

    fn capabilities(&self) -> &[InteractionCapability] {
        &[
            InteractionCapability::PointSelect,
            InteractionCapability::Navigate2D,
            InteractionCapability::TextInput,
            InteractionCapability::DiscreteChoice,
        ]
    }

    fn translate(
        &self,
        event: &SensorEvent,
        context: &InteractionContext,
    ) -> Option<InteractionIntent> {
        let SensorEvent::KeyPress { key, modifiers, .. } = event else {
            return None;
        };

        match key {
            Key::Enter => {
                let target = context
                    .current_focus
                    .clone()
                    .unwrap_or(InteractionTarget::Nothing);
                let mode = if modifiers.shift {
                    SelectionMode::Add
                } else if modifiers.ctrl {
                    SelectionMode::Toggle
                } else {
                    SelectionMode::Replace
                };
                Some(InteractionIntent::Select { target, mode })
            }

            Key::Tab => {
                let direction = if modifiers.shift {
                    NavigationDirection::Backward
                } else {
                    NavigationDirection::Forward
                };
                Some(InteractionIntent::Navigate {
                    direction,
                    magnitude: 1.0,
                })
            }

            Key::Escape => Some(InteractionIntent::Dismiss),

            Key::Delete | Key::Backspace if modifiers.ctrl => {
                let target = context
                    .current_focus
                    .clone()
                    .unwrap_or(InteractionTarget::Nothing);
                Some(InteractionIntent::Manipulate {
                    target,
                    operation: ManipulationOp::Delete,
                })
            }

            Key::Up => Some(InteractionIntent::Navigate {
                direction: if modifiers.ctrl {
                    NavigationDirection::In
                } else {
                    NavigationDirection::Up
                },
                magnitude: if modifiers.shift { 5.0 } else { 1.0 },
            }),

            Key::Down => Some(InteractionIntent::Navigate {
                direction: if modifiers.ctrl {
                    NavigationDirection::Out
                } else {
                    NavigationDirection::Down
                },
                magnitude: if modifiers.shift { 5.0 } else { 1.0 },
            }),

            Key::Left => Some(InteractionIntent::Navigate {
                direction: NavigationDirection::Left,
                magnitude: if modifiers.shift { 5.0 } else { 1.0 },
            }),

            Key::Right => Some(InteractionIntent::Navigate {
                direction: NavigationDirection::Right,
                magnitude: if modifiers.shift { 5.0 } else { 1.0 },
            }),

            Key::Char('/') if !modifiers.ctrl => Some(InteractionIntent::Command {
                verb: "search".into(),
                arguments: serde_json::Value::Null,
            }),

            Key::Char('i' | 'I') if modifiers.ctrl => {
                let target = context
                    .current_focus
                    .clone()
                    .unwrap_or(InteractionTarget::Nothing);
                Some(InteractionIntent::Inspect {
                    target,
                    depth: petal_tongue_core::interaction::InspectionDepth::Detail,
                })
            }

            _ => None,
        }
    }

    fn active_target(&self, context: &InteractionContext) -> Option<InteractionTarget> {
        context.current_focus.clone()
    }

    fn feedback(&mut self, _result: &InteractionResult) {
        // Keyboard adapter is stateless; no feedback handling needed.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::sensor::Modifiers;
    use std::time::Instant;

    fn make_key_event(key: Key, modifiers: Modifiers) -> SensorEvent {
        SensorEvent::KeyPress {
            key,
            modifiers,
            timestamp: Instant::now(),
        }
    }

    #[test]
    fn enter_produces_select() {
        let adapter = KeyboardAdapter::new();
        let ctx = InteractionContext::default_for_perspective(1);

        let intent = adapter.translate(&make_key_event(Key::Enter, Modifiers::none()), &ctx);
        assert!(matches!(
            intent,
            Some(InteractionIntent::Select {
                mode: SelectionMode::Replace,
                ..
            })
        ));
    }

    #[test]
    fn shift_enter_adds_to_selection() {
        let adapter = KeyboardAdapter::new();
        let ctx = InteractionContext::default_for_perspective(1);

        let mods = Modifiers {
            shift: true,
            ..Modifiers::none()
        };
        let intent = adapter.translate(&make_key_event(Key::Enter, mods), &ctx);
        assert!(matches!(
            intent,
            Some(InteractionIntent::Select {
                mode: SelectionMode::Add,
                ..
            })
        ));
    }

    #[test]
    fn escape_produces_dismiss() {
        let adapter = KeyboardAdapter::new();
        let ctx = InteractionContext::default_for_perspective(1);

        let intent = adapter.translate(&make_key_event(Key::Escape, Modifiers::none()), &ctx);
        assert!(matches!(intent, Some(InteractionIntent::Dismiss)));
    }

    #[test]
    fn tab_produces_navigate_forward() {
        let adapter = KeyboardAdapter::new();
        let ctx = InteractionContext::default_for_perspective(1);

        let intent = adapter.translate(&make_key_event(Key::Tab, Modifiers::none()), &ctx);
        assert!(matches!(
            intent,
            Some(InteractionIntent::Navigate {
                direction: NavigationDirection::Forward,
                ..
            })
        ));
    }

    #[test]
    fn shift_tab_produces_navigate_backward() {
        let adapter = KeyboardAdapter::new();
        let ctx = InteractionContext::default_for_perspective(1);

        let mods = Modifiers {
            shift: true,
            ..Modifiers::none()
        };
        let intent = adapter.translate(&make_key_event(Key::Tab, mods), &ctx);
        assert!(matches!(
            intent,
            Some(InteractionIntent::Navigate {
                direction: NavigationDirection::Backward,
                ..
            })
        ));
    }

    #[test]
    fn arrows_produce_navigation() {
        let adapter = KeyboardAdapter::new();
        let ctx = InteractionContext::default_for_perspective(1);

        let intent = adapter.translate(&make_key_event(Key::Up, Modifiers::none()), &ctx);
        assert!(matches!(
            intent,
            Some(InteractionIntent::Navigate {
                direction: NavigationDirection::Up,
                ..
            })
        ));

        let intent = adapter.translate(&make_key_event(Key::Left, Modifiers::none()), &ctx);
        assert!(matches!(
            intent,
            Some(InteractionIntent::Navigate {
                direction: NavigationDirection::Left,
                ..
            })
        ));
    }

    #[test]
    fn ctrl_up_zooms_in() {
        let adapter = KeyboardAdapter::new();
        let ctx = InteractionContext::default_for_perspective(1);

        let intent = adapter.translate(&make_key_event(Key::Up, Modifiers::ctrl()), &ctx);
        assert!(matches!(
            intent,
            Some(InteractionIntent::Navigate {
                direction: NavigationDirection::In,
                ..
            })
        ));
    }

    #[test]
    fn slash_produces_search_command() {
        let adapter = KeyboardAdapter::new();
        let ctx = InteractionContext::default_for_perspective(1);

        let intent = adapter.translate(&make_key_event(Key::Char('/'), Modifiers::none()), &ctx);
        match intent {
            Some(InteractionIntent::Command { verb, .. }) => {
                assert_eq!(verb, "search");
            }
            other => panic!("expected Command, got {other:?}"),
        }
    }

    #[test]
    fn unhandled_key_returns_none() {
        let adapter = KeyboardAdapter::new();
        let ctx = InteractionContext::default_for_perspective(1);

        let intent = adapter.translate(&make_key_event(Key::Char('q'), Modifiers::none()), &ctx);
        assert!(intent.is_none());
    }
}
