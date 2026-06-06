// SPDX-License-Identifier: AGPL-3.0-or-later
//! Graph data loading from scenario files and built-in demonstration topologies.

use crate::args::Args;
use crate::error::HeadlessError;
use petal_tongue_core::GraphEngine;
use std::sync::{Arc, RwLock};

/// Load graph data from scenario file, demo topology, or leave empty for headless export.
pub fn load_graph_data(graph: &Arc<RwLock<GraphEngine>>, args: &Args) -> Result<(), HeadlessError> {
    #[expect(
        clippy::option_if_let_else,
        reason = "three-way branch is clearer as if-let"
    )]
    if let Some(ref path) = args.scenario {
        load_scenario_file(graph, path)
    } else if args.demo {
        load_demo_topology(graph)
    } else {
        tracing::info!(
            "No data source specified — graph is empty. \
             Use --scenario <file> or --demo for sample data."
        );
        Ok(())
    }
}

/// Load graph from a scenario JSON file.
fn load_scenario_file(graph: &Arc<RwLock<GraphEngine>>, path: &str) -> Result<(), HeadlessError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| HeadlessError::ScenarioLoad(format!("read {path}: {e}")))?;
    let scenario: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| HeadlessError::ScenarioLoad(format!("parse {path}: {e}")))?;

    let mut g = graph.write()?;
    if let Some(nodes) = scenario.get("primals").and_then(|v| v.as_array()) {
        for node in nodes {
            if let (Some(id), Some(name), Some(domain)) = (
                node.get("id").and_then(|v| v.as_str()),
                node.get("name").and_then(|v| v.as_str()),
                node.get("domain").and_then(|v| v.as_str()),
            ) {
                let caps = node
                    .get("capabilities")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();
                g.add_node(petal_tongue_core::PrimalInfo::new(
                    id,
                    name,
                    domain,
                    node.get("endpoint")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unix://local"),
                    caps,
                    petal_tongue_core::PrimalHealthStatus::Healthy,
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                ));
            }
        }
    }
    if let Some(edges) = scenario.get("edges").and_then(|v| v.as_array()) {
        for edge in edges {
            if let (Some(from), Some(to)) = (
                edge.get("from").and_then(|v| v.as_str()),
                edge.get("to").and_then(|v| v.as_str()),
            ) {
                g.add_edge(petal_tongue_core::TopologyEdge {
                    from: from.into(),
                    to: to.into(),
                    edge_type: edge
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("connected")
                        .to_string(),
                    label: edge.get("label").and_then(|v| v.as_str()).map(String::from),
                    capability: None,
                    metrics: None,
                });
            }
        }
    }
    g.layout(10);
    let (nc, ec) = (g.nodes().len(), g.edges().len());
    drop(g);
    tracing::info!("📋 Scenario loaded from {path}: {nc} primals, {ec} edges");
    Ok(())
}

#[cfg(test)]
fn make_test_args(scenario: Option<&str>, demo: bool) -> Args {
    Args {
        mode: crate::args::OutputMode::Auto,
        output: None,
        width: 800,
        height: 600,
        scenario: scenario.map(String::from),
        demo,
    }
}

/// Built-in demonstration topology (opt-in via `--demo` or `SHOWCASE_MODE`).
fn load_demo_topology(graph: &Arc<RwLock<GraphEngine>>) -> Result<(), HeadlessError> {
    use petal_tongue_core::constants;
    use petal_tongue_core::{PrimalHealthStatus, PrimalInfo, TopologyEdge};

    tracing::info!("📚 Loading demonstration topology (--demo)");

    let mut g = graph.write()?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let health_id = std::env::var("PETALTONGUE_HEADLESS_DEMO_HEALTH_ID")
        .unwrap_or_else(|_| "health-monitor-1".to_owned());
    let health_name = std::env::var("PETALTONGUE_HEADLESS_DEMO_HEALTH_NAME")
        .unwrap_or_else(|_| "Health Monitor".to_owned());

    let primals = vec![
        PrimalInfo::new(
            "petaltongue-headless",
            "petalTongue Headless",
            "Visualization",
            constants::default_headless_url(),
            vec!["visualization".to_owned(), "export".to_owned()],
            PrimalHealthStatus::Healthy,
            now,
        ),
        PrimalInfo::new(
            health_id.as_str(),
            health_name.as_str(),
            "Health Monitoring",
            constants::default_web_url(),
            vec!["health".to_owned(), "monitoring".to_owned()],
            PrimalHealthStatus::Healthy,
            now,
        ),
        PrimalInfo::new(
            "encryption-demo-1",
            "Encryption Primal",
            "Encrypted Communication",
            constants::default_sandbox_security_url(),
            vec!["encryption".to_owned(), "messaging".to_owned()],
            PrimalHealthStatus::Warning,
            now,
        ),
    ];

    for primal in primals {
        g.add_node(primal);
    }

    g.add_edge(TopologyEdge {
        from: health_id.into(),
        to: "petaltongue-headless".into(),
        edge_type: "monitors".to_owned(),
        label: Some("Health Monitoring".to_owned()),
        capability: None,
        metrics: None,
    });
    g.add_edge(TopologyEdge {
        from: "encryption-demo-1".into(),
        to: "petaltongue-headless".into(),
        edge_type: "sends_data".to_owned(),
        label: Some("Encrypted Messages".to_owned()),
        capability: None,
        metrics: None,
    });

    g.layout(10);
    let (nc, ec) = (g.nodes().len(), g.edges().len());
    drop(g);
    tracing::info!("📊 Loaded: {nc} primals, {ec} connections");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_graph() -> Arc<RwLock<GraphEngine>> {
        Arc::new(RwLock::new(GraphEngine::new()))
    }

    #[test]
    fn load_graph_data_no_source_succeeds() {
        let graph = new_graph();
        let args = make_test_args(None, false);
        assert!(load_graph_data(&graph, &args).is_ok());
        assert_eq!(graph.read().unwrap().nodes().len(), 0);
    }

    #[test]
    fn load_graph_data_demo_populates_graph() {
        let graph = new_graph();
        let args = make_test_args(None, true);
        assert!(load_graph_data(&graph, &args).is_ok());
        let g = graph.read().unwrap();
        assert!(g.nodes().len() >= 3, "demo should load 3+ primals");
        assert!(g.edges().len() >= 2, "demo should load 2+ edges");
    }

    #[test]
    fn load_scenario_valid_json() {
        let scenario = serde_json::json!({
            "primals": [
                {
                    "id": "test-1",
                    "name": "TestPrimal",
                    "domain": "testing",
                    "capabilities": ["test.run"]
                },
                {
                    "id": "test-2",
                    "name": "TestPrimal2",
                    "domain": "testing"
                }
            ],
            "edges": [
                { "from": "test-1", "to": "test-2", "type": "depends_on", "label": "dep" }
            ]
        });
        let temp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), scenario.to_string()).unwrap();

        let graph = new_graph();
        let path = temp.path().to_str().unwrap();
        load_scenario_file(&graph, path).unwrap();

        let g = graph.read().unwrap();
        assert_eq!(g.nodes().len(), 2);
        assert_eq!(g.edges().len(), 1);
    }

    #[test]
    fn load_scenario_nonexistent_file() {
        let graph = new_graph();
        let result = load_scenario_file(&graph, "/nonexistent/scenario.json");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, HeadlessError::ScenarioLoad(_)),
            "expected ScenarioLoad, got {err:?}"
        );
    }

    #[test]
    fn load_scenario_invalid_json() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), "not json {{{").unwrap();

        let graph = new_graph();
        let result = load_scenario_file(&graph, temp.path().to_str().unwrap());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            HeadlessError::ScenarioLoad(_)
        ));
    }

    #[test]
    fn load_scenario_empty_arrays() {
        let scenario = serde_json::json!({ "primals": [], "edges": [] });
        let temp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), scenario.to_string()).unwrap();

        let graph = new_graph();
        load_scenario_file(&graph, temp.path().to_str().unwrap()).unwrap();
        assert_eq!(graph.read().unwrap().nodes().len(), 0);
    }

    #[test]
    fn load_scenario_missing_required_fields_skips_node() {
        let scenario = serde_json::json!({
            "primals": [
                { "id": "only-id" },
                { "id": "valid", "name": "Valid", "domain": "test" }
            ]
        });
        let temp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), scenario.to_string()).unwrap();

        let graph = new_graph();
        load_scenario_file(&graph, temp.path().to_str().unwrap()).unwrap();
        let g = graph.read().unwrap();
        assert_eq!(
            g.nodes().len(),
            1,
            "node with missing name/domain should be skipped"
        );
    }

    #[test]
    fn load_scenario_edges_missing_from_to_skipped() {
        let scenario = serde_json::json!({
            "primals": [
                { "id": "a", "name": "A", "domain": "test" },
                { "id": "b", "name": "B", "domain": "test" }
            ],
            "edges": [
                { "type": "orphan" },
                { "from": "a", "to": "b" }
            ]
        });
        let temp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), scenario.to_string()).unwrap();

        let graph = new_graph();
        load_scenario_file(&graph, temp.path().to_str().unwrap()).unwrap();
        let g = graph.read().unwrap();
        assert_eq!(
            g.edges().len(),
            1,
            "edge without from/to should be skipped, valid edge should remain"
        );
    }

    #[test]
    fn headless_error_display() {
        let err = HeadlessError::ScenarioLoad("bad file".into());
        assert!(err.to_string().contains("bad file"));
        let err = HeadlessError::LockPoisoned("poisoned".into());
        assert!(err.to_string().contains("poisoned"));
    }
}
