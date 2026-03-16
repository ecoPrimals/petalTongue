// SPDX-License-Identifier: AGPL-3.0-or-later
//! Semantic interaction intents.
//!
//! Device events are NOT interactions. A pointer activation at (423, 187) is a
//! physical signal. The *intent* is "select the object at this position."
//! Input adapters translate device events into these semantic intents,
//! which are modality-agnostic and shareable across perspectives.

use serde::{Deserialize, Serialize};

use super::target::{DataObjectId, InteractionTarget};

/// What the user wants to do, independent of input device.
///
/// A pointer activation, keyboard Enter, voice command "select that", and
/// Braille display routing key all produce the same variant when
/// they mean the same thing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum InteractionIntent {
    /// Select one or more data objects.
    Select {
        /// What to select (resolved by inverse pipeline).
        target: InteractionTarget,
        /// How to modify existing selection.
        mode: SelectionMode,
    },

    /// Request detail on a data object (tooltip, panel, speech).
    Inspect {
        /// What to inspect.
        target: InteractionTarget,
        /// How much detail.
        depth: InspectionDepth,
    },

    /// Move the viewport or listener position.
    Navigate {
        /// Which direction.
        direction: NavigationDirection,
        /// How far (0.0..1.0 normalized, or device-specific magnitude).
        magnitude: f64,
    },

    /// Modify a data object or view property.
    Manipulate {
        /// What to modify.
        target: InteractionTarget,
        /// What operation.
        operation: ManipulationOp,
    },

    /// Add an annotation to a data object.
    Annotate {
        /// What to annotate.
        target: InteractionTarget,
        /// The annotation payload.
        content: AnnotationContent,
    },

    /// Free-form command (search, filter, macro).
    Command {
        /// Verb (e.g. "search", "filter", "export").
        verb: String,
        /// Arguments as freeform JSON.
        arguments: serde_json::Value,
    },

    /// Move focus without committing (hover, tab-to).
    Focus {
        /// What to focus.
        target: InteractionTarget,
    },

    /// Dismiss the current view or selection.
    Dismiss,
}

/// How a selection modifies the existing set.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SelectionMode {
    /// Clear existing selection, select only this.
    #[default]
    Replace,
    /// Add to existing selection.
    Add,
    /// Remove from existing selection.
    Remove,
    /// Toggle membership in existing selection.
    Toggle,
}

/// How much detail to show when inspecting.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InspectionDepth {
    /// Brief summary (tooltip, audio label).
    #[default]
    Summary,
    /// Full detail panel.
    Detail,
    /// Raw data (JSON, CSV row).
    Raw,
}

/// Semantic navigation direction (not pixel deltas).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum NavigationDirection {
    /// Forward in the current traversal order.
    Forward,
    /// Backward in the current traversal order.
    Backward,
    /// Up in a hierarchy or spatial Y+.
    Up,
    /// Down in a hierarchy or spatial Y-.
    Down,
    /// Spatial left.
    Left,
    /// Spatial right.
    Right,
    /// Zoom in / move closer.
    In,
    /// Zoom out / move farther.
    Out,
    /// Jump to a specific data location.
    ToData {
        /// The data object to navigate to.
        target: DataObjectId,
    },
}

/// What kind of manipulation to perform.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ManipulationOp {
    /// Move by a delta in data-space coordinates.
    Move {
        /// [dx, dy, dz] offset in data-space units.
        delta: [f64; 3],
    },
    /// Resize by a factor.
    Resize {
        /// Multiplicative scale factor.
        factor: f64,
    },
    /// Reorder relative to another object.
    Reorder {
        /// Place the target before this object.
        before: DataObjectId,
    },
    /// Set a field to a value.
    SetValue {
        /// Which field to set.
        field: String,
        /// New value for the field.
        value: serde_json::Value,
    },
    /// Delete the target.
    Delete,
}

/// Annotation payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum AnnotationContent {
    /// Text note.
    Text(String),
    /// Structured label (key-value).
    Label {
        /// Label key.
        key: String,
        /// Label value.
        value: String,
    },
    /// Flag / marker with a category.
    Flag {
        /// Flag category name.
        category: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interaction::target::InteractionTarget;

    #[test]
    fn intent_serialization_roundtrip() {
        let intent = InteractionIntent::Select {
            target: InteractionTarget::Nothing,
            mode: SelectionMode::Replace,
        };
        let json = serde_json::to_string(&intent).expect("serialize");
        let back: InteractionIntent = serde_json::from_str(&json).expect("deserialize");
        match back {
            InteractionIntent::Select { mode, .. } => {
                assert_eq!(mode, SelectionMode::Replace);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn navigate_to_data() {
        let dir = NavigationDirection::ToData {
            target: DataObjectId::new("health.metrics", serde_json::json!(42)),
        };
        let json = serde_json::to_string(&dir).expect("serialize");
        assert!(json.contains("health.metrics"));
    }

    #[test]
    fn selection_mode_default_is_replace() {
        assert_eq!(SelectionMode::default(), SelectionMode::Replace);
    }
}
