// SPDX-License-Identifier: AGPL-3.0-only
//! Data-space interaction targets.
//!
//! All interaction resolution converges to [`DataObjectId`] values that are
//! perspective-invariant: the same data row has the same identity regardless
//! of which modality resolved it.

use serde::{Deserialize, Serialize};

/// Identifies a data source (capability string, not a primal name).
pub type DataSourceId = String;

/// Identifies a rendered primitive within a single frame.
pub type PrimitiveId = u64;

/// Identifies a grammar expression by ID.
pub type GrammarId = String;

/// Perspective-invariant reference to a data row.
///
/// Two users looking at the same data from different modalities (GUI, TUI,
/// audio) will resolve the same `DataObjectId` when they interact with
/// the same underlying data point.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DataObjectId {
    /// Which data source this row comes from (capability string).
    pub source: DataSourceId,
    /// Row key within that source (freeform JSON for composite keys).
    pub row_key: serde_json::Value,
}

impl DataObjectId {
    /// Create a new data object ID from a source and row key.
    pub fn new(source: impl Into<DataSourceId>, row_key: serde_json::Value) -> Self {
        Self {
            source: source.into(),
            row_key,
        }
    }
}

impl std::fmt::Display for DataObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.source, self.row_key)
    }
}

/// What the user is interacting with, before full data resolution.
///
/// The inverse pipeline resolves an `InteractionTarget` to zero or more
/// [`DataObjectId`] values. Resolution is modality-specific (pixel coords
/// for GUI, cursor position for TUI, time offset for audio, entity name
/// for voice commands).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum InteractionTarget {
    /// A single data row, already resolved.
    DataRow {
        /// The resolved data object.
        data_id: DataObjectId,
    },

    /// A range on a named variable (e.g., time between 14:20 and 14:25).
    DataRange {
        /// Variable name (e.g. "timestamp", "cpu_pct").
        variable: String,
        /// Range minimum.
        min: serde_json::Value,
        /// Range maximum.
        max: serde_json::Value,
    },

    /// A set of rows matching a predicate.
    DataSet {
        /// Filter expression selecting the rows.
        predicate: FilterExpr,
    },

    /// A spatial region in rendering space (pre-resolution).
    Region {
        /// Bounding box in normalized coordinates.
        bounds: BoundingBox,
    },

    /// A specific rendered primitive (pre-resolution to data).
    Primitive {
        /// ID of the rendered primitive.
        primitive_id: PrimitiveId,
    },

    /// No target (click on empty space, dismiss).
    Nothing,
}

/// Axis-aligned bounding box in normalized coordinates.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoundingBox {
    /// Minimum x.
    pub x_min: f64,
    /// Minimum y.
    pub y_min: f64,
    /// Maximum x.
    pub x_max: f64,
    /// Maximum y.
    pub y_max: f64,
}

impl BoundingBox {
    /// Create from two corners.
    #[must_use]
    pub fn from_corners(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self {
            x_min: x1.min(x2),
            y_min: y1.min(y2),
            x_max: x1.max(x2),
            y_max: y1.max(y2),
        }
    }

    /// Check if a point is inside the box.
    #[must_use]
    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x_min && x <= self.x_max && y >= self.y_min && y <= self.y_max
    }

    /// Width of the box.
    #[must_use]
    pub fn width(&self) -> f64 {
        self.x_max - self.x_min
    }

    /// Height of the box.
    #[must_use]
    pub fn height(&self) -> f64 {
        self.y_max - self.y_min
    }
}

/// A simple predicate for filtering data rows.
///
/// Intentionally minimal -- complex query languages belong in the data
/// layer, not the interaction layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum FilterExpr {
    /// Field equals a value.
    Eq {
        /// Field name to compare.
        field: String,
        /// Expected value.
        value: serde_json::Value,
    },
    /// Field is in a range [min, max].
    Range {
        /// Field name to compare.
        field: String,
        /// Range minimum (inclusive).
        min: serde_json::Value,
        /// Range maximum (inclusive).
        max: serde_json::Value,
    },
    /// Field matches a string pattern.
    Contains {
        /// Field name to search.
        field: String,
        /// Substring to match.
        substring: String,
    },
    /// Logical AND of multiple filters.
    And(Vec<FilterExpr>),
    /// Logical OR of multiple filters.
    Or(Vec<FilterExpr>),
    /// Logical NOT.
    Not(Box<FilterExpr>),
}

/// A row of data values keyed by column name.
pub type DataRow = indexmap::IndexMap<String, serde_json::Value>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_object_id_display() {
        let id = DataObjectId::new("health.metrics", serde_json::json!({"primal_id": "alpha"}));
        let display = format!("{id}");
        assert!(display.contains("health.metrics"));
        assert!(display.contains("alpha"));
    }

    #[test]
    fn bounding_box_contains() {
        let bbox = BoundingBox::from_corners(0.0, 0.0, 1.0, 1.0);
        assert!(bbox.contains(0.5, 0.5));
        assert!(!bbox.contains(1.5, 0.5));
    }

    #[test]
    fn bounding_box_from_corners_normalizes() {
        let bbox = BoundingBox::from_corners(1.0, 1.0, 0.0, 0.0);
        assert!((bbox.x_min - 0.0).abs() < f64::EPSILON);
        assert!((bbox.x_max - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn filter_expr_and_composition() {
        let filter = FilterExpr::And(vec![
            FilterExpr::Eq {
                field: "status".into(),
                value: serde_json::json!("healthy"),
            },
            FilterExpr::Range {
                field: "cpu".into(),
                min: serde_json::json!(0),
                max: serde_json::json!(80),
            },
        ]);
        // Ensure it serializes cleanly
        let json = serde_json::to_string(&filter).expect("serialize");
        assert!(json.contains("healthy"));
    }
}
