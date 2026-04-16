// SPDX-License-Identifier: AGPL-3.0-or-later
//! Unit tests for Tufte constraints.

use super::*;
use crate::grammar::{GeometryType, GrammarExpr, ScaleType};
use crate::primitive::{Color, Primitive};
use crate::render_plan::{AxisMeta, PanelBounds, PanelMeta};
use crate::scene_graph::SceneGraph;

fn make_data_primitive() -> Primitive {
    Primitive::Point {
        x: 0.0,
        y: 0.0,
        radius: 5.0,
        fill: Some(Color::BLACK),
        stroke: None,
        data_id: Some("d1".to_string()),
    }
}

fn make_non_data_primitive() -> Primitive {
    Primitive::Point {
        x: 0.0,
        y: 0.0,
        radius: 5.0,
        fill: Some(Color::rgb(0.5, 0.5, 0.5)),
        stroke: None,
        data_id: None,
    }
}

#[test]
fn data_ink_ratio_three_data_one_non_data() {
    let primitives = vec![
        make_data_primitive(),
        make_data_primitive(),
        make_data_primitive(),
        make_non_data_primitive(),
    ];
    let expr = GrammarExpr::new("data", GeometryType::Point);
    let result = DataInkRatio.evaluate(&primitives, &expr, None);
    assert!((result.score - 0.75).abs() < 1e-10);
    assert!(result.passed);
}

#[test]
fn data_ink_ratio_empty_primitives() {
    let primitives: Vec<Primitive> = vec![];
    let expr = GrammarExpr::new("data", GeometryType::Point);
    let result = DataInkRatio.evaluate(&primitives, &expr, None);
    assert!((result.score - 1.0).abs() < 1e-10);
    assert!(result.passed);
}

#[test]
fn tufte_report_evaluate_all_computes_average() {
    let primitives = vec![
        make_data_primitive(),
        make_data_primitive(),
        make_non_data_primitive(),
    ];
    let expr = GrammarExpr::new("data", GeometryType::Point);
    let constraints = [
        TufteConstraintImpl::DataInkRatio,
        TufteConstraintImpl::DataDensity,
    ];
    let report = TufteReport::evaluate_all(&constraints, &primitives, &expr, None);
    assert_eq!(report.results.len(), 2);
}

#[test]
fn chartjunk_detection_all_data_primitives_passes() {
    let primitives = vec![
        make_data_primitive(),
        Primitive::Rect {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 5.0,
            fill: Some(Color::BLACK),
            stroke: None,
            corner_radius: 0.0,
            data_id: Some("r1".to_string()),
        },
    ];
    let expr = GrammarExpr::new("data", GeometryType::Bar);
    let result = ChartjunkDetection.evaluate(&primitives, &expr, None);
    assert!(result.passed);
    assert!(result.score >= 1.0 || (result.score - 1.0).abs() < 1e-10);
}

#[test]
fn lie_factor_basic_no_plan_passes() {
    let primitives = vec![make_data_primitive()];
    let expr = GrammarExpr::new("data", GeometryType::Point);
    let result = LieFactor.evaluate(&primitives, &expr, None);
    assert!(result.passed);
}

#[test]
fn lie_factor_detects_truncated_y_axis() {
    let scene = SceneGraph::new();
    let grammar = GrammarExpr::new("data", GeometryType::Bar);
    let plan = RenderPlan::new(scene, grammar).with_panel(
        PanelMeta::new("main", PanelBounds::new(0.0, 0.0, 800.0, 600.0)).with_axis(
            AxisMeta::new("y", ScaleType::Linear)
                .with_domain(50.0, 100.0)
                .with_range(0.0, 600.0),
        ),
    );
    let result = LieFactor.evaluate_plan(&plan);
    assert!(
        !result.passed,
        "Truncated Y axis should fail: {}",
        result.message
    );
    assert!(result.message.contains("truncated"));
}

#[test]
fn data_ink_ratio_high_decoration_low_score() {
    let primitives = vec![
        make_non_data_primitive(),
        make_non_data_primitive(),
        make_non_data_primitive(),
        make_data_primitive(),
    ];
    let expr = GrammarExpr::new("data", GeometryType::Point);
    let result = DataInkRatio.evaluate(&primitives, &expr, None);
    assert!(!result.passed);
    assert!((result.score - 0.25).abs() < 1e-10);
}

#[test]
fn data_ink_ratio_all_data_passes() {
    let primitives = vec![
        make_data_primitive(),
        make_data_primitive(),
        make_data_primitive(),
    ];
    let expr = GrammarExpr::new("data", GeometryType::Line);
    let result = DataInkRatio.evaluate(&primitives, &expr, None);
    assert!(result.passed);
    assert!((result.score - 1.0).abs() < 1e-10);
}

#[test]
fn data_ink_ratio_half_data_passes_threshold() {
    let primitives = vec![make_data_primitive(), make_non_data_primitive()];
    let expr = GrammarExpr::new("data", GeometryType::Point);
    let result = DataInkRatio.evaluate(&primitives, &expr, None);
    assert!(result.passed);
    assert!((result.score - 0.5).abs() < 1e-10);
}

#[test]
fn color_accessibility_high_contrast_passes() {
    let primitives = vec![Primitive::Point {
        x: 0.0,
        y: 0.0,
        radius: 5.0,
        fill: Some(Color::WHITE),
        stroke: None,
        data_id: Some("d1".to_string()),
    }];
    let expr = GrammarExpr::new("data", GeometryType::Point);
    let result = ColorAccessibility.evaluate(&primitives, &expr, None);
    assert!(
        result.passed,
        "White on dark bg should pass: {}",
        result.message
    );
}

#[test]
fn color_accessibility_low_contrast_fails() {
    let primitives = vec![Primitive::Point {
        x: 0.0,
        y: 0.0,
        radius: 5.0,
        fill: Some(Color::rgba(0.13, 0.13, 0.15, 1.0)),
        stroke: None,
        data_id: Some("d1".to_string()),
    }];
    let expr = GrammarExpr::new("data", GeometryType::Point);
    let result = ColorAccessibility.evaluate(&primitives, &expr, None);
    assert!(
        !result.passed,
        "Near-bg color should fail: {}",
        result.message
    );
}

#[test]
fn smallest_effective_difference_single_point_passes() {
    let primitives = vec![make_data_primitive()];
    let expr = GrammarExpr::new("data", GeometryType::Point);
    let result = SmallestEffectiveDifference.evaluate(&primitives, &expr, None);
    assert!(result.passed);
}

#[test]
fn smallest_effective_difference_overlapping_fails() {
    let primitives = vec![
        Primitive::Point {
            x: 100.0,
            y: 100.0,
            radius: 5.0,
            fill: Some(Color::BLACK),
            stroke: None,
            data_id: Some("a".into()),
        },
        Primitive::Point {
            x: 100.5,
            y: 100.5,
            radius: 5.0,
            fill: Some(Color::BLACK),
            stroke: None,
            data_id: Some("b".into()),
        },
    ];
    let expr = GrammarExpr::new("data", GeometryType::Point);
    let result = SmallestEffectiveDifference.evaluate(&primitives, &expr, None);
    assert!(
        !result.passed,
        "Overlapping points should fail: {}",
        result.message
    );
}

#[test]
fn small_multiples_preference_no_color_passes() {
    let expr = GrammarExpr::new("data", GeometryType::Point);
    let result = SmallMultiplesPreference.evaluate(&[], &expr, None);
    assert!(result.passed);
}

#[test]
fn chartjunk_detection_decorative_fill_fails() {
    let primitives = vec![Primitive::Rect {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 50.0,
        fill: Some(Color::rgb(0.9, 0.9, 0.9)),
        stroke: None,
        corner_radius: 0.0,
        data_id: None,
    }];
    let expr = GrammarExpr::new("data", GeometryType::Bar);
    let result = ChartjunkDetection.evaluate(&primitives, &expr, None);
    assert!(!result.passed);
    assert!(result.score < 1.0);
}

#[test]
fn data_density_with_data_passes() {
    let primitives = vec![make_data_primitive()];
    let expr = GrammarExpr::new("data", GeometryType::Point);
    let result = DataDensity.evaluate(&primitives, &expr, None);
    assert!(result.passed);
    assert!((result.score - 1.0).abs() < 1e-10);
}

#[test]
fn data_density_no_data_fails() {
    let primitives: Vec<Primitive> = vec![];
    let expr = GrammarExpr::new("data", GeometryType::Point);
    let result = DataDensity.evaluate(&primitives, &expr, None);
    assert!(!result.passed);
    assert!((result.score - 0.0).abs() < 1e-10);
}

#[test]
fn constraint_severity_variants() {
    let info = ConstraintSeverity::Info;
    let warning = ConstraintSeverity::Warning;
    let error = ConstraintSeverity::Error;
    assert!(matches!(info, ConstraintSeverity::Info));
    assert!(matches!(warning, ConstraintSeverity::Warning));
    assert!(matches!(error, ConstraintSeverity::Error));
}

#[test]
fn constraint_severity_serde() {
    let severities = [
        ConstraintSeverity::Info,
        ConstraintSeverity::Warning,
        ConstraintSeverity::Error,
    ];
    for sev in &severities {
        let json = serde_json::to_string(sev).unwrap();
        let restored: ConstraintSeverity = serde_json::from_str(&json).unwrap();
        assert_eq!(*sev, restored);
    }
}

#[test]
fn constraint_result_construction() {
    let result = ConstraintResult {
        passed: true,
        score: 0.9,
        message: "test".to_string(),
    };
    assert!(result.passed);
    assert!((result.score - 0.9).abs() < 1e-10);
    assert_eq!(result.message, "test");
}

#[test]
fn constraint_result_serde() {
    let result = ConstraintResult {
        passed: false,
        score: 0.5,
        message: "failed".to_string(),
    };
    let json = serde_json::to_string(&result).unwrap();
    let restored: ConstraintResult = serde_json::from_str(&json).unwrap();
    assert_eq!(result.passed, restored.passed);
    assert!((result.score - restored.score).abs() < 1e-10);
    assert_eq!(result.message, restored.message);
}

#[test]
fn tufte_report_construction() {
    let report = TufteReport {
        overall_score: 0.85,
        results: vec![(
            "DataInkRatio".to_string(),
            ConstraintResult {
                passed: true,
                score: 0.9,
                message: "ok".to_string(),
            },
        )],
        corrections_applied: vec![],
    };
    assert!((report.overall_score - 0.85).abs() < 1e-10);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.results[0].0, "DataInkRatio");
    assert!(report.corrections_applied.is_empty());
}

#[test]
fn tufte_report_serde() {
    let report = TufteReport {
        overall_score: 0.7,
        results: vec![(
            "Test".to_string(),
            ConstraintResult {
                passed: true,
                score: 0.7,
                message: "msg".to_string(),
            },
        )],
        corrections_applied: vec!["fix1".to_string()],
    };
    let json = serde_json::to_string(&report).unwrap();
    let restored: TufteReport = serde_json::from_str(&json).unwrap();
    assert!((report.overall_score - restored.overall_score).abs() < 1e-10);
    assert_eq!(report.results.len(), restored.results.len());
    assert_eq!(
        report.corrections_applied.len(),
        restored.corrections_applied.len()
    );
}

#[test]
fn tufte_constraint_severity_and_auto_correctable() {
    assert_eq!(DataInkRatio.severity(), ConstraintSeverity::Warning);
    assert!(!DataInkRatio.auto_correctable());
    assert_eq!(DataDensity.severity(), ConstraintSeverity::Info);
    assert_eq!(ChartjunkDetection.severity(), ConstraintSeverity::Warning);
    assert_eq!(LieFactor.severity(), ConstraintSeverity::Warning);
}
