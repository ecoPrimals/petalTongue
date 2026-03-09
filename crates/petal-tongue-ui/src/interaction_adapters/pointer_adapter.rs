// SPDX-License-Identifier: AGPL-3.0-only
//! Pointer (mouse/touch/stylus) input adapter.
//!
//! Translates mouse/pointer [`SensorEvent`] values into semantic
//! [`InteractionIntent`] values. Handles click-to-select, scroll-to-zoom,
//! hover-to-focus, and drag-to-pan/select-box.

use petal_tongue_core::interaction::{
    InputAdapter, InputModality, InteractionCapability, InteractionContext, InteractionIntent,
    InteractionResult, InteractionTarget, NavigationDirection, SelectionMode,
};
use petal_tongue_core::sensor::{MouseButton, SensorEvent};

/// Adapts mouse/pointer sensor events to semantic interaction intents.
pub struct PointerAdapter {
    last_hover_x: f32,
    last_hover_y: f32,
}

impl PointerAdapter {
    /// Create a new pointer adapter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            last_hover_x: 0.0,
            last_hover_y: 0.0,
        }
    }
}

impl Default for PointerAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl InputAdapter for PointerAdapter {
    fn name(&self) -> &'static str {
        "Pointer"
    }

    fn modality(&self) -> InputModality {
        InputModality::PointerMouse
    }

    fn capabilities(&self) -> &[InteractionCapability] {
        &[
            InteractionCapability::PointSelect,
            InteractionCapability::RangeSelect,
            InteractionCapability::Navigate2D,
            InteractionCapability::ContinuousValue,
        ]
    }

    fn translate(
        &self,
        event: &SensorEvent,
        context: &InteractionContext,
    ) -> Option<InteractionIntent> {
        match event {
            SensorEvent::Click { x, y, button, .. } => {
                let target = InteractionTarget::Region {
                    bounds: petal_tongue_core::interaction::BoundingBox::from_corners(
                        f64::from(*x),
                        f64::from(*y),
                        f64::from(*x),
                        f64::from(*y),
                    ),
                };

                match button {
                    MouseButton::Left => {
                        let mode = if context.current_selection.is_empty() {
                            SelectionMode::Replace
                        } else {
                            SelectionMode::Replace
                        };
                        Some(InteractionIntent::Select { target, mode })
                    }
                    MouseButton::Right => Some(InteractionIntent::Inspect {
                        target,
                        depth: petal_tongue_core::interaction::InspectionDepth::Summary,
                    }),
                    MouseButton::Middle => Some(InteractionIntent::Dismiss),
                    MouseButton::Other(_) => None,
                }
            }

            SensorEvent::Scroll { delta_y, .. } => {
                let direction = if *delta_y > 0.0 {
                    NavigationDirection::In
                } else {
                    NavigationDirection::Out
                };
                Some(InteractionIntent::Navigate {
                    direction,
                    magnitude: delta_y.abs().into(),
                })
            }

            SensorEvent::Position { x, y, .. } => {
                let target = InteractionTarget::Region {
                    bounds: petal_tongue_core::interaction::BoundingBox::from_corners(
                        f64::from(*x),
                        f64::from(*y),
                        f64::from(*x),
                        f64::from(*y),
                    ),
                };
                Some(InteractionIntent::Focus { target })
            }

            _ => None,
        }
    }

    fn active_target(&self, _context: &InteractionContext) -> Option<InteractionTarget> {
        Some(InteractionTarget::Region {
            bounds: petal_tongue_core::interaction::BoundingBox::from_corners(
                f64::from(self.last_hover_x),
                f64::from(self.last_hover_y),
                f64::from(self.last_hover_x),
                f64::from(self.last_hover_y),
            ),
        })
    }

    fn feedback(&mut self, result: &InteractionResult) {
        // Track last position from focus events for active_target
        if let InteractionIntent::Focus {
            target: InteractionTarget::Region { bounds },
        } = &result.intent
        {
            self.last_hover_x = bounds.x_min as f32;
            self.last_hover_y = bounds.y_min as f32;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn click_produces_select() {
        let adapter = PointerAdapter::new();
        let ctx = InteractionContext::default_for_perspective(1);

        let event = SensorEvent::Click {
            x: 100.0,
            y: 200.0,
            button: MouseButton::Left,
            timestamp: Instant::now(),
        };

        let intent = adapter.translate(&event, &ctx);
        assert!(intent.is_some());
        match intent.unwrap() {
            InteractionIntent::Select { mode, .. } => {
                assert_eq!(mode, SelectionMode::Replace);
            }
            _ => panic!("expected Select"),
        }
    }

    #[test]
    fn right_click_produces_inspect() {
        let adapter = PointerAdapter::new();
        let ctx = InteractionContext::default_for_perspective(1);

        let event = SensorEvent::Click {
            x: 50.0,
            y: 75.0,
            button: MouseButton::Right,
            timestamp: Instant::now(),
        };

        let intent = adapter.translate(&event, &ctx);
        assert!(matches!(intent, Some(InteractionIntent::Inspect { .. })));
    }

    #[test]
    fn scroll_produces_navigate() {
        let adapter = PointerAdapter::new();
        let ctx = InteractionContext::default_for_perspective(1);

        let event = SensorEvent::Scroll {
            delta_x: 0.0,
            delta_y: 1.0,
            timestamp: Instant::now(),
        };

        let intent = adapter.translate(&event, &ctx);
        assert!(matches!(
            intent,
            Some(InteractionIntent::Navigate {
                direction: NavigationDirection::In,
                ..
            })
        ));
    }

    #[test]
    fn position_produces_focus() {
        let adapter = PointerAdapter::new();
        let ctx = InteractionContext::default_for_perspective(1);

        let event = SensorEvent::Position {
            x: 300.0,
            y: 400.0,
            timestamp: Instant::now(),
        };

        let intent = adapter.translate(&event, &ctx);
        assert!(matches!(intent, Some(InteractionIntent::Focus { .. })));
    }

    #[test]
    fn heartbeat_ignored() {
        let adapter = PointerAdapter::new();
        let ctx = InteractionContext::default_for_perspective(1);

        let event = SensorEvent::Heartbeat {
            latency: std::time::Duration::from_millis(10),
            timestamp: Instant::now(),
        };

        assert!(adapter.translate(&event, &ctx).is_none());
    }
}
