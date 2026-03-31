// SPDX-License-Identifier: AGPL-3.0-or-later
//! Property-based tests for core data types: BoundingBox, DataObjectId, FilterExpr.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use petal_tongue_core::interaction::{BoundingBox, DataObjectId, FilterExpr};
use proptest::prelude::*;
use serde_json::Value;

/// Strategy for simple JSON values (used in FilterExpr and DataObjectId).
fn json_value_strategy() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(|n| serde_json::json!(n)),
        "[a-zA-Z0-9_-]{0,64}".prop_map(Value::String),
    ];
    leaf.prop_recursive(2, 4, 8, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..3).prop_map(Value::Array),
            prop::collection::hash_map("[a-z][a-z0-9_]{0,12}", inner, 0..3)
                .prop_map(|m| Value::Object(m.into_iter().collect())),
        ]
    })
}

/// Strategy for FilterExpr with limited depth.
fn filter_expr_strategy() -> impl Strategy<Value = FilterExpr> {
    let leaf = prop_oneof![
        ("[a-z][a-z0-9_]{0,16}", json_value_strategy())
            .prop_map(|(field, value)| FilterExpr::Eq { field, value }),
        (
            "[a-z][a-z0-9_]{0,16}",
            json_value_strategy(),
            json_value_strategy(),
        )
            .prop_map(|(field, min, max)| FilterExpr::Range { field, min, max }),
        ("[a-z][a-z0-9_]{0,16}", "[a-zA-Z0-9 ]{0,32}")
            .prop_map(|(field, substring)| FilterExpr::Contains { field, substring }),
    ];
    leaf.prop_recursive(
        2,
        4,
        8,
        |inner: proptest::strategy::BoxedStrategy<FilterExpr>| {
            prop_oneof![
                prop::collection::vec(inner.clone(), 1..3).prop_map(FilterExpr::And),
                prop::collection::vec(inner.clone(), 1..3).prop_map(FilterExpr::Or),
                inner.prop_map(|e| FilterExpr::Not(Box::new(e))),
            ]
        },
    )
}

proptest! {
    #[test]
    fn prop_bounding_box_normalization(x1 in -1e6f64..1e6f64, y1 in -1e6f64..1e6f64, x2 in -1e6f64..1e6f64, y2 in -1e6f64..1e6f64) {
        let bbox = BoundingBox::from_corners(x1, y1, x2, y2);
        prop_assert!(bbox.x_min <= bbox.x_max);
        prop_assert!(bbox.y_min <= bbox.y_max);
    }

    #[test]
    fn prop_bounding_box_contains(x1 in -1e6f64..1e6f64, y1 in -1e6f64..1e6f64, x2 in -1e6f64..1e6f64, y2 in -1e6f64..1e6f64) {
        let bbox = BoundingBox::from_corners(x1, y1, x2, y2);
        // Midpoint is always inside the normalized box
        let x = f64::midpoint(bbox.x_min, bbox.x_max);
        let y = f64::midpoint(bbox.y_min, bbox.y_max);
        prop_assert!(bbox.contains(x, y));
    }

    #[test]
    fn prop_bounding_box_dimensions_non_negative(x1 in -1e6f64..1e6f64, y1 in -1e6f64..1e6f64, x2 in -1e6f64..1e6f64, y2 in -1e6f64..1e6f64) {
        let bbox = BoundingBox::from_corners(x1, y1, x2, y2);
        prop_assert!(bbox.width() >= 0.0);
        prop_assert!(bbox.height() >= 0.0);
    }

    #[test]
    fn prop_data_object_id_display_contains_source(source in "[a-zA-Z0-9._-]{1,64}", row_key in json_value_strategy()) {
        let id = DataObjectId::new(source.clone(), row_key);
        let display = format!("{id}");
        prop_assert!(display.contains(&source), "display must contain source");
    }

    #[test]
    fn prop_filter_expr_serialization_roundtrip(expr in filter_expr_strategy()) {
        let json = serde_json::to_string(&expr).expect("serialize");
        let parsed: FilterExpr = serde_json::from_str(&json).expect("deserialize");
        let json2 = serde_json::to_string(&parsed).expect("serialize again");
        let _parsed2: FilterExpr = serde_json::from_str(&json2).expect("deserialize again");
        prop_assert!(!json.is_empty());
        prop_assert!(!json2.is_empty());
    }
}
