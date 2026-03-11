// SPDX-License-Identifier: AGPL-3.0-only
//! Proptest for Tufte constraints.

use super::*;
use crate::grammar::{GeometryType, GrammarExpr};
use crate::primitive::{Color, Primitive};
use proptest::prelude::*;

fn point_strategy() -> impl Strategy<Value = Primitive> {
    (
        0.0f64..800.0,
        0.0f64..600.0,
        0.1f64..50.0,
        prop_oneof![
            Just(None),
            Just(Some("d1".to_string())),
            Just(Some("id".to_string())),
        ],
    )
        .prop_map(|(x, y, radius, data_id)| Primitive::Point {
            x,
            y,
            radius,
            fill: Some(Color::BLACK),
            stroke: None,
            data_id,
        })
}

proptest! {
    /// Constraint scores and overall_score are always in [0.0, 1.0].
    #[test]
    fn prop_tufte_scores_in_range(primitives in proptest::collection::vec(point_strategy(), 0..20)) {
        let expr = GrammarExpr::new("data", GeometryType::Point);
        let constraints: Vec<&dyn TufteConstraint> = vec![
            &DataInkRatio,
            &LieFactor,
            &ChartjunkDetection,
            &ColorAccessibility,
            &DataDensity,
            &SmallestEffectiveDifference,
            &SmallMultiplesPreference,
        ];
        let report = TufteReport::evaluate_all(&constraints, &primitives, &expr, None);
        prop_assert!(
            (0.0..=1.0).contains(&report.overall_score),
            "overall_score {} not in [0,1]",
            report.overall_score
        );
        for (name, result) in &report.results {
            prop_assert!(
                (0.0..=1.0).contains(&result.score),
                "constraint {} score {} not in [0,1]",
                name,
                result.score
            );
        }
    }
}
