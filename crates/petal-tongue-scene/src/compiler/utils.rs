// SPDX-License-Identifier: AGPL-3.0-only
//! Compiler utilities: field extraction, JSON helpers, faceting.

use std::collections::BTreeMap;
use std::sync::Arc;

use serde_json::Value;

use crate::grammar::{GrammarExpr, VariableRole};
use crate::primitive::Primitive;

/// Get the x field name from variable bindings.
pub(crate) fn x_field(expr: &GrammarExpr) -> Option<&str> {
    expr.variables
        .iter()
        .find(|v| v.role == VariableRole::X)
        .map(|v| v.field.as_str())
}

/// Get the y field name from variable bindings.
pub(crate) fn y_field(expr: &GrammarExpr) -> Option<&str> {
    expr.variables
        .iter()
        .find(|v| v.role == VariableRole::Y)
        .map(|v| v.field.as_str())
}

/// Extract a numeric value from JSON for a given key.
pub(crate) fn get_number(obj: &serde_json::Map<String, Value>, key: &str) -> Option<f64> {
    obj.get(key).and_then(|v| match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.parse().ok(),
        _ => None,
    })
}

/// Partition data rows by a JSON field value, preserving insertion order.
/// Uses `Arc<str>` for facet keys to avoid cloning when the same key appears
/// in multiple rows (zero-copy sharing across groups).
pub(crate) fn partition_by_field(data: &[Value], field: &str) -> Vec<(Arc<str>, Vec<Value>)> {
    let mut groups: BTreeMap<Arc<str>, Vec<Value>> = BTreeMap::new();
    for row in data {
        let key: Arc<str> = row.as_object().and_then(|o| o.get(field)).map_or_else(
            || Arc::from("(none)"),
            |v| match v {
                Value::String(s) => Arc::from(s.as_str()),
                other => Arc::from(other.to_string()),
            },
        );
        groups.entry(key).or_default().push(row.clone());
    }
    groups.into_iter().collect()
}

/// Offset a primitive's position by (dx, dy).
pub(crate) fn offset_primitive(p: &mut Primitive, dx: f64, dy: f64) {
    match p {
        Primitive::Point { x, y, .. }
        | Primitive::Text { x, y, .. }
        | Primitive::Rect { x, y, .. } => {
            *x += dx;
            *y += dy;
        }
        Primitive::Arc { cx, cy, .. } => {
            *cx += dx;
            *cy += dy;
        }
        Primitive::Line { points, .. } | Primitive::Polygon { points, .. } => {
            for pt in points {
                pt[0] += dx;
                pt[1] += dy;
            }
        }
        Primitive::BezierPath {
            start, segments, ..
        } => {
            start[0] += dx;
            start[1] += dy;
            for seg in segments {
                seg.cp1[0] += dx;
                seg.cp1[1] += dy;
                seg.cp2[0] += dx;
                seg.cp2[1] += dy;
                seg.end[0] += dx;
                seg.end[1] += dy;
            }
        }
        Primitive::Mesh { .. } => {}
    }
}
