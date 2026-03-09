// SPDX-License-Identifier: AGPL-3.0-only
//! Interaction results and state changes.
//!
//! When an [`InteractionIntent`](super::intent::InteractionIntent) is resolved,
//! the engine produces an [`InteractionResult`] describing what happened. This
//! result is broadcast to all local modalities and IPC subscribers.

use serde::{Deserialize, Serialize};
use std::time::Instant;

use super::intent::{AnnotationContent, InteractionIntent};
use super::perspective::PerspectiveId;
use super::target::{DataObjectId, DataSourceId, GrammarId};

/// The outcome of processing an interaction intent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionResult {
    /// The original intent that was processed.
    pub intent: InteractionIntent,
    /// Data objects that were resolved as targets.
    pub resolved_targets: Vec<DataObjectId>,
    /// What state changed as a result.
    pub state_changes: Vec<StateChange>,
    /// Which perspective initiated this interaction.
    pub perspective_id: PerspectiveId,
    /// Monotonic timestamp (milliseconds since engine start).
    pub timestamp_ms: u64,
}

/// A discrete change to interaction state.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum StateChange {
    /// Selection was modified.
    SelectionChanged {
        /// New complete selection set.
        selected: Vec<DataObjectId>,
    },

    /// A data filter was applied or removed.
    FilterChanged {
        /// Which variable the filter applies to.
        variable: String,
        /// New range, or `None` if filter was removed.
        range: Option<(serde_json::Value, serde_json::Value)>,
    },

    /// Viewport / camera changed.
    ViewChanged {
        /// New viewport center X.
        center_x: f64,
        /// New viewport center Y.
        center_y: f64,
        /// New zoom level.
        zoom: f64,
    },

    /// Data was mutated (field set, object created/deleted).
    DataMutated {
        /// Which data source was affected.
        source: DataSourceId,
        /// What mutation occurred.
        mutation: DataMutation,
    },

    /// An annotation was added.
    AnnotationAdded {
        /// Which data object was annotated.
        target: DataObjectId,
        /// The annotation content.
        content: AnnotationContent,
    },

    /// Focus changed (hover / tab-to).
    FocusChanged {
        /// New focused object, or `None` for no focus.
        focused: Option<DataObjectId>,
    },
}

/// A mutation to a data source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DataMutation {
    /// A field was set on a row.
    FieldSet {
        /// Which row was modified.
        row: DataObjectId,
        /// Which field was set.
        field: String,
        /// New field value.
        value: serde_json::Value,
    },
    /// A row was created.
    RowCreated {
        /// The new row's identity.
        row: DataObjectId,
    },
    /// A row was deleted.
    RowDeleted {
        /// The deleted row's identity.
        row: DataObjectId,
    },
    /// A row was moved/reordered.
    RowMoved {
        /// The moved row's identity.
        row: DataObjectId,
        /// Placed before this row.
        before: DataObjectId,
    },
}

/// An interaction event as transmitted over IPC.
///
/// This is the wire format for `visualization.interact` messages.
/// Uses millisecond timestamps (not `Instant`) for serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionEvent {
    /// Semantic event type.
    pub event_type: String,
    /// Resolved data-space targets.
    pub targets: Vec<DataObjectId>,
    /// Which perspective originated this event.
    pub perspective_id: PerspectiveId,
    /// Which grammar expression this relates to.
    pub grammar_id: GrammarId,
    /// ISO 8601 timestamp.
    pub timestamp: String,
}

impl InteractionEvent {
    /// Create from an `InteractionResult` for IPC transmission.
    pub fn from_result(result: &InteractionResult, grammar_id: GrammarId) -> Self {
        let event_type = match &result.intent {
            InteractionIntent::Select { .. } => "select",
            InteractionIntent::Inspect { .. } => "inspect",
            InteractionIntent::Navigate { .. } => "navigate",
            InteractionIntent::Manipulate { .. } => "manipulate",
            InteractionIntent::Annotate { .. } => "annotate",
            InteractionIntent::Command { .. } => "command",
            InteractionIntent::Focus { .. } => "focus",
            InteractionIntent::Dismiss => "dismiss",
        };

        Self {
            event_type: event_type.to_string(),
            targets: result.resolved_targets.clone(),
            perspective_id: result.perspective_id,
            grammar_id,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Tracks monotonic time for interaction timestamps.
pub struct InteractionClock {
    start: Instant,
}

impl InteractionClock {
    /// Create a new clock starting now.
    #[must_use]
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Milliseconds since this clock was created.
    #[must_use]
    pub fn elapsed_ms(&self) -> u64 {
        u64::try_from(self.start.elapsed().as_millis()).unwrap_or(u64::MAX)
    }
}

impl Default for InteractionClock {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interaction::target::InteractionTarget;

    #[test]
    fn interaction_event_from_result() {
        let result = InteractionResult {
            intent: InteractionIntent::Select {
                target: InteractionTarget::Nothing,
                mode: crate::interaction::intent::SelectionMode::Replace,
            },
            resolved_targets: vec![DataObjectId::new("test", serde_json::json!("row1"))],
            state_changes: vec![StateChange::SelectionChanged {
                selected: vec![DataObjectId::new("test", serde_json::json!("row1"))],
            }],
            perspective_id: 1,
            timestamp_ms: 12345,
        };

        let event = InteractionEvent::from_result(&result, "my_grammar".into());
        assert_eq!(event.event_type, "select");
        assert_eq!(event.targets.len(), 1);
        assert_eq!(event.grammar_id, "my_grammar");
    }

    #[test]
    fn interaction_clock_monotonic() {
        let clock = InteractionClock::new();
        let t1 = clock.elapsed_ms();
        std::thread::sleep(std::time::Duration::from_millis(5));
        let t2 = clock.elapsed_ms();
        assert!(t2 >= t1);
    }

    #[test]
    fn state_change_serialization() {
        let change = StateChange::SelectionChanged {
            selected: vec![DataObjectId::new("src", serde_json::json!(1))],
        };
        let json = serde_json::to_string(&change).expect("serialize");
        assert!(json.contains("SelectionChanged"));
    }
}
