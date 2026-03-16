// SPDX-License-Identifier: AGPL-3.0-only
//! SQUIRREL interaction adapter: AI-driven interaction adaptation.
//!
//! SQUIRREL (or any AI agent) subscribes to interaction events via
//! `interaction.subscribe` and can push interaction intents back via
//! `visualization.interact.apply`. This module bridges those IPC events
//! with the `EguiInteractionBridge` so the UI responds to AI-suggested
//! focus changes, selections, and navigation.
//!
//! ## Flow
//!
//! ```text
//! SQUIRREL ─→ interaction.subscribe (IPC) ─→ InteractionSubscriberRegistry
//!                                                       │
//!             SquirrelAdapter.poll_and_apply() ←─────────┘
//!                       │
//!                       ▼
//!              EguiInteractionBridge (focus/select/navigate)
//! ```

use petal_tongue_core::interaction::DataObjectId;
use petal_tongue_ipc::visualization_handler::InteractionEventNotification;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

/// AI-initiated interaction command that the UI should apply.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum AiInteractionCommand {
    /// AI suggests focusing on a specific data object.
    #[serde(rename = "focus")]
    Focus {
        /// Target data object identifier.
        target: String,
    },
    /// AI suggests selecting one or more data objects.
    #[serde(rename = "select")]
    Select {
        /// Target data object identifiers.
        targets: Vec<String>,
    },
    /// AI suggests clearing the current selection.
    #[serde(rename = "deselect")]
    Deselect,
    /// AI suggests highlighting a region or object for attention.
    #[serde(rename = "highlight")]
    Highlight {
        /// Targets to highlight.
        targets: Vec<String>,
        /// Duration hint in milliseconds (0 = indefinite).
        #[serde(default)]
        duration_ms: u64,
    },
    /// AI suggests navigating to a specific view or panel.
    #[serde(rename = "navigate")]
    Navigate {
        /// Navigation target (panel ID or view name).
        destination: String,
    },
}

/// Adapter that polls AI-originated interaction events and converts them
/// to `AiInteractionCommand`s for the `EguiInteractionBridge`.
pub struct SquirrelAdapter {
    subscriber_id: String,
    registry: Arc<RwLock<petal_tongue_ipc::InteractionSubscriberRegistry>>,
    pending_commands: Vec<AiInteractionCommand>,
}

impl SquirrelAdapter {
    /// Create a new adapter and register as a subscriber.
    pub async fn new(
        registry: Arc<RwLock<petal_tongue_ipc::InteractionSubscriberRegistry>>,
    ) -> Self {
        let subscriber_id = "squirrel-ui-adapter".to_string();
        {
            let mut reg = registry.write().await;
            reg.subscribe_with_filter(
                &subscriber_id,
                vec![
                    "ai.focus".to_string(),
                    "ai.select".to_string(),
                    "ai.deselect".to_string(),
                    "ai.highlight".to_string(),
                    "ai.navigate".to_string(),
                ],
                None,
                None,
            );
        }
        Self {
            subscriber_id,
            registry,
            pending_commands: Vec::new(),
        }
    }

    /// Create without registering (for use when registry is not yet available).
    #[must_use]
    pub fn new_deferred() -> Self {
        Self {
            subscriber_id: "squirrel-ui-adapter".to_string(),
            registry: Arc::new(RwLock::new(
                petal_tongue_ipc::InteractionSubscriberRegistry::new(),
            )),
            pending_commands: Vec::new(),
        }
    }

    /// Connect to a live registry (deferred initialization).
    pub async fn connect(
        &mut self,
        registry: Arc<RwLock<petal_tongue_ipc::InteractionSubscriberRegistry>>,
    ) {
        {
            let mut reg = registry.write().await;
            reg.subscribe_with_filter(
                &self.subscriber_id,
                vec![
                    "ai.focus".to_string(),
                    "ai.select".to_string(),
                    "ai.deselect".to_string(),
                    "ai.highlight".to_string(),
                    "ai.navigate".to_string(),
                ],
                None,
                None,
            );
        }
        self.registry = registry;
    }

    /// Poll the IPC registry for AI interaction events and convert to commands.
    pub fn poll(&mut self, registry: &mut petal_tongue_ipc::InteractionSubscriberRegistry) {
        let events = registry.poll(&self.subscriber_id);
        for event in events {
            if let Some(cmd) = Self::event_to_command(&event) {
                debug!("SQUIRREL adapter: {:?}", cmd);
                self.pending_commands.push(cmd);
            }
        }
    }

    /// Drain pending AI commands (call once per frame).
    pub fn drain_commands(&mut self) -> Vec<AiInteractionCommand> {
        std::mem::take(&mut self.pending_commands)
    }

    /// Apply a single AI command to the interaction bridge.
    pub fn apply_command(
        bridge: &mut crate::interaction_bridge::EguiInteractionBridge,
        command: &AiInteractionCommand,
    ) {
        match command {
            AiInteractionCommand::Focus { target } => {
                let data_id = DataObjectId::new("ai", serde_json::json!(target));
                if let Some(p) = bridge.engine.perspective_mut(bridge.perspective_id) {
                    p.focus = Some(data_id);
                }
            }
            AiInteractionCommand::Select { targets } => {
                if let Some(p) = bridge.engine.perspective_mut(bridge.perspective_id) {
                    p.clear_selection();
                    for t in targets {
                        let data_id = DataObjectId::new("ai", serde_json::json!(t));
                        p.add_to_selection(data_id);
                    }
                }
            }
            AiInteractionCommand::Deselect => {
                if let Some(p) = bridge.engine.perspective_mut(bridge.perspective_id) {
                    p.clear_selection();
                    p.focus = None;
                }
            }
            AiInteractionCommand::Highlight { .. } | AiInteractionCommand::Navigate { .. } => {
                // Highlight and navigate are higher-level commands; the bridge
                // records the intent, and the renderer reads it each frame.
                // No direct perspective mutation needed here.
            }
        }
    }

    /// Convert an IPC event notification to an AI command.
    fn event_to_command(event: &InteractionEventNotification) -> Option<AiInteractionCommand> {
        match event.event_type.as_str() {
            "ai.focus" => event
                .targets
                .first()
                .map(|t| AiInteractionCommand::Focus { target: t.clone() }),
            "ai.select" => Some(AiInteractionCommand::Select {
                targets: event.targets.clone(),
            }),
            "ai.deselect" => Some(AiInteractionCommand::Deselect),
            "ai.highlight" => Some(AiInteractionCommand::Highlight {
                targets: event.targets.clone(),
                duration_ms: 0,
            }),
            "ai.navigate" => event
                .targets
                .first()
                .map(|t| AiInteractionCommand::Navigate {
                    destination: t.clone(),
                }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_event(event_type: &str, targets: Vec<&str>) -> InteractionEventNotification {
        InteractionEventNotification {
            event_type: event_type.to_string(),
            targets: targets.into_iter().map(String::from).collect(),
            timestamp: "2026-03-14T00:00:00Z".to_string(),
            perspective_id: None,
        }
    }

    #[test]
    fn event_to_command_focus() {
        let event = make_event("ai.focus", vec!["node-42"]);
        let cmd = SquirrelAdapter::event_to_command(&event).unwrap();
        match cmd {
            AiInteractionCommand::Focus { target } => assert_eq!(target, "node-42"),
            _ => panic!("expected Focus"),
        }
    }

    #[test]
    fn event_to_command_select() {
        let event = make_event("ai.select", vec!["a", "b"]);
        let cmd = SquirrelAdapter::event_to_command(&event).unwrap();
        match cmd {
            AiInteractionCommand::Select { targets } => {
                assert_eq!(targets, vec!["a", "b"]);
            }
            _ => panic!("expected Select"),
        }
    }

    #[test]
    fn event_to_command_deselect() {
        let event = make_event("ai.deselect", vec![]);
        let cmd = SquirrelAdapter::event_to_command(&event).unwrap();
        assert!(matches!(cmd, AiInteractionCommand::Deselect));
    }

    #[test]
    fn event_to_command_highlight() {
        let event = make_event("ai.highlight", vec!["panel-1"]);
        let cmd = SquirrelAdapter::event_to_command(&event).unwrap();
        match cmd {
            AiInteractionCommand::Highlight { targets, .. } => {
                assert_eq!(targets, vec!["panel-1"]);
            }
            _ => panic!("expected Highlight"),
        }
    }

    #[test]
    fn event_to_command_navigate() {
        let event = make_event("ai.navigate", vec!["dashboard"]);
        let cmd = SquirrelAdapter::event_to_command(&event).unwrap();
        match cmd {
            AiInteractionCommand::Navigate { destination } => {
                assert_eq!(destination, "dashboard");
            }
            _ => panic!("expected Navigate"),
        }
    }

    #[test]
    fn event_to_command_unknown_returns_none() {
        let event = make_event("unknown.event", vec![]);
        assert!(SquirrelAdapter::event_to_command(&event).is_none());
    }

    #[test]
    fn event_to_command_focus_no_target_returns_none() {
        let event = make_event("ai.focus", vec![]);
        assert!(SquirrelAdapter::event_to_command(&event).is_none());
    }

    #[test]
    fn drain_commands_returns_and_clears() {
        let mut adapter = SquirrelAdapter::new_deferred();
        adapter
            .pending_commands
            .push(AiInteractionCommand::Deselect);
        let cmds = adapter.drain_commands();
        assert_eq!(cmds.len(), 1);
        assert!(adapter.drain_commands().is_empty());
    }

    #[test]
    fn poll_converts_events_to_commands() {
        let mut reg = petal_tongue_ipc::InteractionSubscriberRegistry::new();
        reg.subscribe_with_filter(
            "squirrel-ui-adapter",
            vec!["ai.focus".to_string()],
            None,
            None,
        );
        let event = InteractionEventNotification {
            event_type: "ai.focus".to_string(),
            targets: vec!["target-1".to_string()],
            timestamp: String::new(),
            perspective_id: None,
        };
        reg.broadcast(&event);

        let mut adapter = SquirrelAdapter::new_deferred();
        adapter.poll(&mut reg);
        let cmds = adapter.drain_commands();
        assert_eq!(cmds.len(), 1);
        assert!(matches!(&cmds[0], AiInteractionCommand::Focus { target } if target == "target-1"));
    }

    #[test]
    fn apply_command_focus_sets_perspective_focus() {
        let mut bridge = crate::interaction_bridge::EguiInteractionBridge::new();
        let cmd = AiInteractionCommand::Focus {
            target: "test-node".to_string(),
        };
        SquirrelAdapter::apply_command(&mut bridge, &cmd);
        assert!(bridge.focused_data_id().is_some());
    }

    #[test]
    fn apply_command_select_sets_selection() {
        let mut bridge = crate::interaction_bridge::EguiInteractionBridge::new();
        let cmd = AiInteractionCommand::Select {
            targets: vec!["a".to_string(), "b".to_string()],
        };
        SquirrelAdapter::apply_command(&mut bridge, &cmd);
        assert_eq!(bridge.selected_data_ids().len(), 2);
    }

    #[test]
    fn apply_command_deselect_clears() {
        let mut bridge = crate::interaction_bridge::EguiInteractionBridge::new();
        SquirrelAdapter::apply_command(
            &mut bridge,
            &AiInteractionCommand::Select {
                targets: vec!["x".to_string()],
            },
        );
        assert_eq!(bridge.selected_data_ids().len(), 1);

        SquirrelAdapter::apply_command(&mut bridge, &AiInteractionCommand::Deselect);
        assert!(bridge.selected_data_ids().is_empty());
        assert!(bridge.focused_data_id().is_none());
    }

    #[test]
    fn ai_command_serialization_roundtrip() {
        let cmds = vec![
            AiInteractionCommand::Focus {
                target: "n1".to_string(),
            },
            AiInteractionCommand::Select {
                targets: vec!["a".to_string()],
            },
            AiInteractionCommand::Deselect,
            AiInteractionCommand::Highlight {
                targets: vec!["p1".to_string()],
                duration_ms: 500,
            },
            AiInteractionCommand::Navigate {
                destination: "home".to_string(),
            },
        ];
        for cmd in &cmds {
            let json = serde_json::to_string(cmd).unwrap();
            let parsed: AiInteractionCommand = serde_json::from_str(&json).unwrap();
            assert_eq!(
                serde_json::to_string(&parsed).unwrap(),
                json,
                "roundtrip failed"
            );
        }
    }
}
