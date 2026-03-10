// SPDX-License-Identifier: AGPL-3.0-only
//! Tests for scenario_loader module.
//!
//! Verifies parsing of healthSpring-style scenario JSON produced by dump_scenarios.

#![allow(clippy::float_cmp)]

use petal_tongue_core::{DataBinding, LoadedScenario};
use std::path::Path;

/// healthSpring-style scenario JSON matching dump_scenarios output.
const HEALTHSPRING_SCENARIO_JSON: &str = r#"{
    "name": "PK/PD Demo",
    "description": "Oral pharmacokinetics and gut microbiome visualization",
    "version": "1.0.0",
    "mode": "demo",
    "domain": "health",
    "ecosystem": {
        "primals": [
            {
                "id": "healthspring-1",
                "name": "HealthSpring Demo",
                "type": "healthspring",
                "family": "healthSpring",
                "status": "healthy",
                "health": 100,
                "data_channels": [
                    {
                        "channel_type": "timeseries",
                        "id": "pk_curve",
                        "label": "Oral PK Concentration",
                        "x_label": "Time (hr)",
                        "y_label": "Concentration (mg/L)",
                        "unit": "mg/L",
                        "x_values": [0.0, 1.0, 2.0, 3.0],
                        "y_values": [0.0, 0.1, 0.15, 0.12]
                    },
                    {
                        "channel_type": "gauge",
                        "id": "heart_rate",
                        "label": "Heart Rate",
                        "value": 72.0,
                        "min": 40.0,
                        "max": 140.0,
                        "unit": "bpm",
                        "normal_range": [60.0, 100.0],
                        "warning_range": [40.0, 60.0]
                    },
                    {
                        "channel_type": "bar",
                        "id": "gut_abundances",
                        "label": "Genus Relative Abundance",
                        "categories": ["Genus A", "Genus B", "Genus C"],
                        "values": [0.3, 0.25, 0.2],
                        "unit": "relative"
                    }
                ],
                "clinical_ranges": [
                    {"label": "Cmax therapeutic", "min": 0.05, "max": 0.3, "status": "normal"},
                    {"label": "Cmax high", "min": 0.3, "max": 0.5, "status": "warning"}
                ],
                "capabilities": ["timeseries", "gauge", "bar"]
            }
        ]
    },
    "edges": [
        {"from": "healthspring-1", "to": "viz-1", "edge_type": "feeds", "label": "data"},
        {"from": "viz-1", "to": "output", "edge_type": "renders", "label": ""}
    ]
}"#;

#[test]
fn test_basic_parsing_timeseries_gauge_bar() {
    let scenario = LoadedScenario::from_json(HEALTHSPRING_SCENARIO_JSON).expect("parse scenario");

    assert_eq!(scenario.name, "PK/PD Demo");
    assert_eq!(
        scenario.description,
        "Oral pharmacokinetics and gut microbiome visualization"
    );
    assert_eq!(scenario.version, "1.0.0");
    assert_eq!(scenario.mode, "demo");
    assert_eq!(scenario.domain.as_deref(), Some("health"));
    assert_eq!(scenario.ecosystem.primals.len(), 1);

    let node = &scenario.ecosystem.primals[0];
    assert_eq!(node.id, "healthspring-1");
    assert_eq!(node.name, "HealthSpring Demo");
    assert_eq!(node.node_type, "healthspring");
    assert_eq!(node.family, "healthSpring");
    assert_eq!(node.health, 100);
    assert_eq!(node.data_channels.len(), 3);

    // Timeseries
    match &node.data_channels[0] {
        DataBinding::TimeSeries {
            id,
            label,
            x_values,
            y_values,
            ..
        } => {
            assert_eq!(id, "pk_curve");
            assert_eq!(label, "Oral PK Concentration");
            assert_eq!(x_values.len(), 4);
            assert!((y_values[2] - 0.15).abs() < 1e-9);
        }
        _ => panic!("expected TimeSeries"),
    }

    // Gauge
    match &node.data_channels[1] {
        DataBinding::Gauge {
            id,
            value,
            normal_range,
            ..
        } => {
            assert_eq!(id, "heart_rate");
            assert!((*value - 72.0).abs() < 1e-9);
            assert!((normal_range[0] - 60.0).abs() < f64::EPSILON);
            assert!((normal_range[1] - 100.0).abs() < f64::EPSILON);
        }
        _ => panic!("expected Gauge"),
    }

    // Bar
    match &node.data_channels[2] {
        DataBinding::Bar {
            id,
            categories,
            values,
            ..
        } => {
            assert_eq!(id, "gut_abundances");
            assert_eq!(categories.len(), 3);
            assert!((values[0] - 0.3).abs() < 1e-9);
        }
        _ => panic!("expected Bar"),
    }
}

#[test]
fn test_edge_parsing() {
    let scenario = LoadedScenario::from_json(HEALTHSPRING_SCENARIO_JSON).expect("parse scenario");

    assert_eq!(scenario.edges.len(), 2);

    assert_eq!(scenario.edges[0].from, "healthspring-1");
    assert_eq!(scenario.edges[0].to, "viz-1");
    assert_eq!(scenario.edges[0].edge_type, "feeds");
    assert_eq!(scenario.edges[0].label, "data");

    assert_eq!(scenario.edges[1].from, "viz-1");
    assert_eq!(scenario.edges[1].to, "output");
    assert_eq!(scenario.edges[1].edge_type, "renders");
    assert_eq!(scenario.edges[1].label, "");
}

#[test]
fn test_all_bindings() {
    let scenario = LoadedScenario::from_json(HEALTHSPRING_SCENARIO_JSON).expect("parse scenario");

    let bindings = scenario.all_bindings();
    assert_eq!(bindings.len(), 3, "expected 3 data channels from 1 node");

    let ids: Vec<&str> = bindings
        .iter()
        .map(|b| match b {
            DataBinding::TimeSeries { id, .. }
            | DataBinding::Gauge { id, .. }
            | DataBinding::Bar { id, .. } => id.as_str(),
            _ => "other",
        })
        .collect();
    assert_eq!(ids, ["pk_curve", "heart_rate", "gut_abundances"]);
}

#[test]
fn test_all_thresholds() {
    let scenario = LoadedScenario::from_json(HEALTHSPRING_SCENARIO_JSON).expect("parse scenario");

    let thresholds = scenario.all_thresholds();
    assert_eq!(thresholds.len(), 2, "expected 2 clinical ranges");

    assert_eq!(thresholds[0].label, "Cmax therapeutic");
    assert!((thresholds[0].min - 0.05).abs() < f64::EPSILON);
    assert_eq!(thresholds[0].status, "normal");

    assert_eq!(thresholds[1].label, "Cmax high");
    assert_eq!(thresholds[1].status, "warning");
}

#[test]
fn test_inferred_domain_from_explicit() {
    let scenario = LoadedScenario::from_json(HEALTHSPRING_SCENARIO_JSON).expect("parse scenario");
    assert_eq!(scenario.inferred_domain(), "health");
}

#[test]
fn test_inferred_domain_from_family() {
    let json_no_domain = r#"{
        "name": "Test",
        "description": "Test",
        "ecosystem": {
            "primals": [{
                "id": "n1",
                "name": "Node",
                "family": "wetSpring",
                "data_channels": [],
                "clinical_ranges": []
            }]
        }
    }"#;
    let scenario = LoadedScenario::from_json(json_no_domain).expect("parse");
    assert_eq!(scenario.inferred_domain(), "ecology");
}

#[test]
fn test_inferred_domain_fallback() {
    let json_no_hint = r#"{
        "name": "Test",
        "description": "Test",
        "ecosystem": {
            "primals": [{
                "id": "n1",
                "name": "Node",
                "family": "unknown",
                "data_channels": [],
                "clinical_ranges": []
            }]
        }
    }"#;
    let scenario = LoadedScenario::from_json(json_no_hint).expect("parse");
    assert_eq!(scenario.inferred_domain(), "measurement");
}

#[test]
fn test_from_json_invalid_returns_error() {
    let result = LoadedScenario::from_json("{ invalid json }");
    assert!(result.is_err());

    let result = LoadedScenario::from_json("null");
    assert!(result.is_err());

    let result = LoadedScenario::from_json(r#"{"name": "x"}"#);
    assert!(result.is_err()); // missing required fields
}

#[test]
fn test_from_file() {
    let temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    std::fs::write(temp_file.path(), HEALTHSPRING_SCENARIO_JSON).expect("write temp file");

    let scenario = LoadedScenario::from_file(temp_file.path()).expect("load from file");
    assert_eq!(scenario.name, "PK/PD Demo");
    assert_eq!(scenario.ecosystem.primals.len(), 1);
}

#[test]
fn test_from_file_nonexistent() {
    let result = LoadedScenario::from_file(Path::new("/nonexistent/path/scenario.json"));
    assert!(result.is_err());
}
