// SPDX-License-Identifier: AGPL-3.0-or-later
//! Agent input adapter — formalizes agentic AI as an `InputAdapter`.
//!
//! While the existing `AiAdapter` bridges IPC events to the egui UI layer,
//! this adapter operates at the interaction engine level, translating
//! `SensorEvent::Generic` payloads (carrying JSON-RPC commands from AI agents)
//! into semantic `InteractionIntent` values.
//!
//! This allows agentic AI to participate in the same interaction pipeline
//! as human users — selections, navigation, inspection all go through
//! the same engine and produce the same `InteractionResult` broadcasts.

use crate::interaction::{
    DataObjectId, InputAdapter, InputModality, InteractionCapability, InteractionContext,
    InteractionIntent, InteractionResult, InteractionTarget, NavigationDirection, SelectionMode,
};
use crate::sensor::SensorEvent;

/// Adapts agentic AI commands (delivered as `SensorEvent::Generic`) to
/// semantic interaction intents.
///
/// The `Generic.data` field is expected to contain a JSON object with an
/// `"action"` key matching one of: `"select"`, `"focus"`, `"navigate"`,
/// `"inspect"`, `"command"`.
pub struct AgentInputAdapter {
    _private: (),
}

impl AgentInputAdapter {
    /// Create a new agent input adapter.
    #[must_use]
    pub const fn new() -> Self {
        Self { _private: () }
    }
}

impl Default for AgentInputAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl InputAdapter for AgentInputAdapter {
    fn name(&self) -> &'static str {
        "Agent"
    }

    fn modality(&self) -> InputModality {
        InputModality::Agent
    }

    fn capabilities(&self) -> &[InteractionCapability] {
        &[
            InteractionCapability::PointSelect,
            InteractionCapability::Navigate2D,
            InteractionCapability::Navigate3D,
            InteractionCapability::TextInput,
            InteractionCapability::DiscreteChoice,
        ]
    }

    fn translate(
        &self,
        event: &SensorEvent,
        context: &InteractionContext,
    ) -> Option<InteractionIntent> {
        let SensorEvent::Generic { data, .. } = event else {
            return None;
        };

        let parsed: serde_json::Value = serde_json::from_str(data).ok()?;
        let action = parsed.get("action")?.as_str()?;

        match action {
            "select" => {
                let target_str = parsed
                    .get("target")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("");
                let target = if target_str.is_empty() {
                    context
                        .current_focus
                        .clone()
                        .unwrap_or(InteractionTarget::Nothing)
                } else {
                    InteractionTarget::DataRow {
                        data_id: DataObjectId::new("agent", serde_json::json!(target_str)),
                    }
                };
                Some(InteractionIntent::Select {
                    target,
                    mode: SelectionMode::Replace,
                })
            }
            "focus" => {
                let target_str = parsed.get("target")?.as_str()?;
                Some(InteractionIntent::Focus {
                    target: InteractionTarget::DataRow {
                        data_id: DataObjectId::new("agent", serde_json::json!(target_str)),
                    },
                })
            }
            "navigate" => {
                let direction = match parsed
                    .get("direction")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("forward")
                {
                    "backward" => NavigationDirection::Backward,
                    "up" => NavigationDirection::Up,
                    "down" => NavigationDirection::Down,
                    "left" => NavigationDirection::Left,
                    "right" => NavigationDirection::Right,
                    "in" => NavigationDirection::In,
                    "out" => NavigationDirection::Out,
                    // "forward" and any unknown direction default to forward navigation
                    _ => NavigationDirection::Forward,
                };
                let magnitude = parsed
                    .get("magnitude")
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(1.0);
                Some(InteractionIntent::Navigate {
                    direction,
                    magnitude,
                })
            }
            "inspect" => {
                let target_str = parsed
                    .get("target")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("");
                let target = if target_str.is_empty() {
                    context
                        .current_focus
                        .clone()
                        .unwrap_or(InteractionTarget::Nothing)
                } else {
                    InteractionTarget::DataRow {
                        data_id: DataObjectId::new("agent", serde_json::json!(target_str)),
                    }
                };
                Some(InteractionIntent::Inspect {
                    target,
                    depth: crate::interaction::InspectionDepth::Detail,
                })
            }
            "command" => {
                let verb = parsed
                    .get("verb")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("unknown")
                    .to_string();
                let arguments = parsed
                    .get("arguments")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
                Some(InteractionIntent::Command { verb, arguments })
            }
            "dismiss" => Some(InteractionIntent::Dismiss),
            _ => None,
        }
    }

    fn active_target(&self, context: &InteractionContext) -> Option<InteractionTarget> {
        context.current_focus.clone()
    }

    fn feedback(&mut self, _result: &InteractionResult) {
        // Agent-side feedback is delivered via IPC (interaction.subscribe),
        // not through this in-process callback.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interaction::InteractionContext;
    use std::time::Instant;

    fn generic_event(json: &str) -> SensorEvent {
        SensorEvent::Generic {
            data: json.to_string(),
            timestamp: Instant::now(),
        }
    }

    #[test]
    fn select_with_target() {
        let adapter = AgentInputAdapter::new();
        let ctx = InteractionContext::default_for_perspective(1);
        let event = generic_event(r#"{"action":"select","target":"node-42"}"#);
        let intent = adapter.translate(&event, &ctx);
        assert!(matches!(intent, Some(InteractionIntent::Select { .. })));
    }

    #[test]
    fn select_without_target_uses_focus() {
        let adapter = AgentInputAdapter::new();
        let mut ctx = InteractionContext::default_for_perspective(1);
        ctx.current_focus = Some(InteractionTarget::Nothing);
        let event = generic_event(r#"{"action":"select"}"#);
        let intent = adapter.translate(&event, &ctx);
        assert!(matches!(intent, Some(InteractionIntent::Select { .. })));
    }

    #[test]
    fn focus_command() {
        let adapter = AgentInputAdapter::new();
        let ctx = InteractionContext::default_for_perspective(1);
        let event = generic_event(r#"{"action":"focus","target":"node-1"}"#);
        let intent = adapter.translate(&event, &ctx);
        assert!(matches!(intent, Some(InteractionIntent::Focus { .. })));
    }

    #[test]
    fn navigate_with_direction() {
        let adapter = AgentInputAdapter::new();
        let ctx = InteractionContext::default_for_perspective(1);
        let event = generic_event(r#"{"action":"navigate","direction":"up","magnitude":2.0}"#);
        let intent = adapter.translate(&event, &ctx);
        assert!(matches!(intent, Some(InteractionIntent::Navigate { .. })));
    }

    #[test]
    fn navigate_defaults_forward() {
        let adapter = AgentInputAdapter::new();
        let ctx = InteractionContext::default_for_perspective(1);
        let event = generic_event(r#"{"action":"navigate"}"#);
        let intent = adapter.translate(&event, &ctx);
        assert!(matches!(intent, Some(InteractionIntent::Navigate { .. })));
    }

    #[test]
    fn inspect_command() {
        let adapter = AgentInputAdapter::new();
        let ctx = InteractionContext::default_for_perspective(1);
        let event = generic_event(r#"{"action":"inspect","target":"node-5"}"#);
        let intent = adapter.translate(&event, &ctx);
        assert!(matches!(intent, Some(InteractionIntent::Inspect { .. })));
    }

    #[test]
    fn command_verb() {
        let adapter = AgentInputAdapter::new();
        let ctx = InteractionContext::default_for_perspective(1);
        let event =
            generic_event(r#"{"action":"command","verb":"filter","arguments":{"field":"temp"}}"#);
        let intent = adapter.translate(&event, &ctx);
        match intent {
            Some(InteractionIntent::Command { verb, arguments }) => {
                assert_eq!(verb, "filter");
                assert_eq!(arguments["field"], "temp");
            }
            other => panic!("expected Command, got {other:?}"),
        }
    }

    #[test]
    fn dismiss_command() {
        let adapter = AgentInputAdapter::new();
        let ctx = InteractionContext::default_for_perspective(1);
        let event = generic_event(r#"{"action":"dismiss"}"#);
        assert!(matches!(
            adapter.translate(&event, &ctx),
            Some(InteractionIntent::Dismiss)
        ));
    }

    #[test]
    fn unknown_action_returns_none() {
        let adapter = AgentInputAdapter::new();
        let ctx = InteractionContext::default_for_perspective(1);
        let event = generic_event(r#"{"action":"unknown_action"}"#);
        assert!(adapter.translate(&event, &ctx).is_none());
    }

    #[test]
    fn invalid_json_returns_none() {
        let adapter = AgentInputAdapter::new();
        let ctx = InteractionContext::default_for_perspective(1);
        let event = generic_event("not json");
        assert!(adapter.translate(&event, &ctx).is_none());
    }

    #[test]
    fn non_generic_event_returns_none() {
        let adapter = AgentInputAdapter::new();
        let ctx = InteractionContext::default_for_perspective(1);
        let event = SensorEvent::Position {
            x: 0.0,
            y: 0.0,
            timestamp: Instant::now(),
        };
        assert!(adapter.translate(&event, &ctx).is_none());
    }

    #[test]
    fn modality_is_custom_agent() {
        let adapter = AgentInputAdapter::new();
        assert_eq!(adapter.modality(), InputModality::Agent);
    }

    #[test]
    fn name_is_agent() {
        let adapter = AgentInputAdapter::default();
        assert_eq!(adapter.name(), "Agent");
    }
}
