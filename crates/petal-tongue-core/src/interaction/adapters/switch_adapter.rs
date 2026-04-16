// SPDX-License-Identifier: AGPL-3.0-or-later
//! Switch input adapter for binary switch access.
//!
//! Enables motor-impaired users to interact via single-switch devices
//! (sip-and-puff, head switch, eye blink, BCI binary intent). The adapter
//! implements a scanning pattern: it cycles through focusable elements,
//! and each switch activation either advances focus or selects the
//! currently focused item.
//!
//! Two-switch mode: switch A = advance, switch B = select.
//! Single-switch mode: auto-advance on timer, switch = select.

use crate::interaction::{
    InputAdapter, InputModality, InteractionCapability, InteractionContext, InteractionIntent,
    InteractionResult, InteractionTarget, NavigationDirection, SelectionMode,
};
use crate::sensor::SensorEvent;

/// Scanning mode for switch access.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanMode {
    /// Single switch: auto-advance, switch = select.
    SingleSwitch,
    /// Two switches: button 0 = advance, button 1 = select.
    TwoSwitch,
}

/// Adapts binary switch sensor events to semantic interaction intents.
pub struct SwitchInputAdapter {
    mode: ScanMode,
    /// Tracks whether the next single-switch press should select (true)
    /// or advance (false). Toggles on each activation in `SingleSwitch` mode
    /// when auto-advance is not yet implemented.
    next_is_select: bool,
}

impl SwitchInputAdapter {
    /// Create a switch adapter in the given scanning mode.
    #[must_use]
    pub const fn new(mode: ScanMode) -> Self {
        Self {
            mode,
            next_is_select: false,
        }
    }
}

impl Default for SwitchInputAdapter {
    fn default() -> Self {
        Self::new(ScanMode::SingleSwitch)
    }
}

impl InputAdapter for SwitchInputAdapter {
    fn name(&self) -> &'static str {
        "SwitchAccess"
    }

    fn modality(&self) -> InputModality {
        InputModality::SwitchAccess
    }

    fn capabilities(&self) -> &[InteractionCapability] {
        &[
            InteractionCapability::PointSelect,
            InteractionCapability::Navigate2D,
            InteractionCapability::DiscreteChoice,
        ]
    }

    fn translate(
        &self,
        event: &SensorEvent,
        context: &InteractionContext,
    ) -> Option<InteractionIntent> {
        let SensorEvent::ButtonPress { button, .. } = event else {
            return None;
        };

        match self.mode {
            ScanMode::TwoSwitch => {
                if *button == 0 {
                    Some(InteractionIntent::Navigate {
                        direction: NavigationDirection::Forward,
                        magnitude: 1.0,
                    })
                } else {
                    let target = context
                        .current_focus
                        .clone()
                        .unwrap_or(InteractionTarget::Nothing);
                    Some(InteractionIntent::Select {
                        target,
                        mode: SelectionMode::Replace,
                    })
                }
            }
            ScanMode::SingleSwitch => {
                if self.next_is_select {
                    let target = context
                        .current_focus
                        .clone()
                        .unwrap_or(InteractionTarget::Nothing);
                    Some(InteractionIntent::Select {
                        target,
                        mode: SelectionMode::Replace,
                    })
                } else {
                    Some(InteractionIntent::Navigate {
                        direction: NavigationDirection::Forward,
                        magnitude: 1.0,
                    })
                }
            }
        }
    }

    fn active_target(&self, context: &InteractionContext) -> Option<InteractionTarget> {
        context.current_focus.clone()
    }

    fn feedback(&mut self, result: &InteractionResult) {
        if self.mode == ScanMode::SingleSwitch {
            self.next_is_select = matches!(result.intent, InteractionIntent::Navigate { .. });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interaction::InteractionTarget;
    use std::time::Instant;

    fn button_event(button: u8) -> SensorEvent {
        SensorEvent::ButtonPress {
            button,
            timestamp: Instant::now(),
        }
    }

    fn context_with_focus() -> InteractionContext {
        let mut ctx = InteractionContext::default_for_perspective(1);
        ctx.current_focus = Some(InteractionTarget::Nothing);
        ctx
    }

    #[test]
    fn two_switch_button_0_navigates() {
        let adapter = SwitchInputAdapter::new(ScanMode::TwoSwitch);
        let ctx = InteractionContext::default_for_perspective(1);
        let intent = adapter.translate(&button_event(0), &ctx);
        assert!(matches!(intent, Some(InteractionIntent::Navigate { .. })));
    }

    #[test]
    fn two_switch_button_1_selects() {
        let adapter = SwitchInputAdapter::new(ScanMode::TwoSwitch);
        let ctx = context_with_focus();
        let intent = adapter.translate(&button_event(1), &ctx);
        assert!(matches!(intent, Some(InteractionIntent::Select { .. })));
    }

    #[test]
    fn single_switch_alternates_navigate_select() {
        let mut adapter = SwitchInputAdapter::new(ScanMode::SingleSwitch);
        let ctx = context_with_focus();

        let first = adapter.translate(&button_event(0), &ctx);
        assert!(
            matches!(first, Some(InteractionIntent::Navigate { .. })),
            "first press should navigate"
        );

        adapter.feedback(&InteractionResult {
            intent: InteractionIntent::Navigate {
                direction: NavigationDirection::Forward,
                magnitude: 1.0,
            },
            resolved_targets: vec![],
            state_changes: vec![],
            perspective_id: 1,
            timestamp_ms: 0,
        });

        let second = adapter.translate(&button_event(0), &ctx);
        assert!(
            matches!(second, Some(InteractionIntent::Select { .. })),
            "second press should select"
        );
    }

    #[test]
    fn ignores_non_button_events() {
        let adapter = SwitchInputAdapter::new(ScanMode::TwoSwitch);
        let ctx = InteractionContext::default_for_perspective(1);
        let event = SensorEvent::Position {
            x: 10.0,
            y: 20.0,
            timestamp: Instant::now(),
        };
        assert!(adapter.translate(&event, &ctx).is_none());
    }

    #[test]
    fn modality_is_switch_access() {
        let adapter = SwitchInputAdapter::new(ScanMode::TwoSwitch);
        assert_eq!(adapter.modality(), InputModality::SwitchAccess);
    }

    #[test]
    fn name_is_switch_access() {
        let adapter = SwitchInputAdapter::default();
        assert_eq!(adapter.name(), "SwitchAccess");
    }

    #[test]
    fn active_target_returns_focus() {
        let adapter = SwitchInputAdapter::default();
        let ctx = context_with_focus();
        assert!(adapter.active_target(&ctx).is_some());

        let empty_ctx = InteractionContext::default_for_perspective(1);
        assert!(adapter.active_target(&empty_ctx).is_none());
    }
}
