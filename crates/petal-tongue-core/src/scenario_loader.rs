// SPDX-License-Identifier: AGPL-3.0-only
//! Scenario JSON loader for healthSpring-style scenario format.
//!
//! Loads and parses scenario JSON from disk or string into structured data
//! compatible with the universal visualization system.

use crate::{DataBinding, ThresholdRange};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A loaded scenario from a Spring or external source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadedScenario {
    /// Scenario display name.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Version string (e.g., "1.0.0").
    #[serde(default)]
    pub version: String,
    /// Mode identifier (e.g., "demo", "live").
    #[serde(default)]
    pub mode: String,
    /// Optional domain hint (health, ecology, etc.).
    #[serde(default)]
    pub domain: Option<String>,
    /// Ecosystem containing primals.
    pub ecosystem: ScenarioEcosystem,
    /// Edges between nodes.
    #[serde(default)]
    pub edges: Vec<ScenarioEdge>,
}

/// Ecosystem container for scenario nodes (primals).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioEcosystem {
    /// List of primal nodes.
    pub primals: Vec<ScenarioNode>,
}

/// A single primal node in the scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioNode {
    /// Unique node identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Node type (e.g., "healthspring").
    #[serde(rename = "type", default)]
    pub node_type: String,
    /// Family hint for domain inference.
    #[serde(default)]
    pub family: String,
    /// Status string (e.g., "healthy").
    #[serde(default)]
    pub status: String,
    /// Health value 0-100.
    #[serde(default)]
    pub health: u8,
    /// Data channels for visualization.
    #[serde(default)]
    pub data_channels: Vec<DataBinding>,
    /// Clinical/threshold ranges.
    #[serde(default)]
    pub clinical_ranges: Vec<ThresholdRange>,
    /// Capability strings.
    #[serde(default)]
    pub capabilities: Vec<String>,
}

/// Edge between two nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioEdge {
    /// Source node id.
    pub from: String,
    /// Target node id.
    pub to: String,
    /// Edge type (e.g., "feeds", "renders").
    #[serde(default)]
    pub edge_type: String,
    /// Optional label.
    #[serde(default)]
    pub label: String,
}

impl LoadedScenario {
    /// Load a scenario from a JSON file.
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let scenario: Self = serde_json::from_str(&contents)?;
        Ok(scenario)
    }

    /// Load from a JSON string.
    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        let scenario: Self = serde_json::from_str(json)?;
        Ok(scenario)
    }

    /// Extract all `DataBindings` from all nodes.
    #[must_use]
    pub fn all_bindings(&self) -> Vec<&DataBinding> {
        self.ecosystem
            .primals
            .iter()
            .flat_map(|p| p.data_channels.iter())
            .collect()
    }

    /// Extract all `ThresholdRanges` from all nodes.
    #[must_use]
    pub fn all_thresholds(&self) -> Vec<&ThresholdRange> {
        self.ecosystem
            .primals
            .iter()
            .flat_map(|p| p.clinical_ranges.iter())
            .collect()
    }

    /// Infer the domain from the scenario metadata or node families.
    ///
    /// Uses capability-based matching via well-known suffix conventions rather than
    /// hardcoding specific primal names. Any spring family is parsed by stripping
    /// the "spring"/"Spring" suffix to derive a domain hint.
    #[must_use]
    pub fn inferred_domain(&self) -> &str {
        if let Some(ref d) = self.domain {
            return d;
        }
        for node in &self.ecosystem.primals {
            if let Some(domain) = Self::domain_from_family(&node.family) {
                return domain;
            }
        }
        "measurement"
    }

    /// Derive a domain hint from a spring family name.
    ///
    /// Strips the "spring"/"Spring" suffix and maps the prefix to a domain.
    /// This is agnostic — any new spring family automatically resolves if it
    /// follows the naming convention and is registered in the domain map.
    fn domain_from_family(family: &str) -> Option<&'static str> {
        let prefix = family
            .strip_suffix("spring")
            .or_else(|| family.strip_suffix("Spring"))?;
        let prefix_lower = prefix.to_ascii_lowercase();
        match prefix_lower.as_str() {
            "health" => Some("health"),
            "wet" => Some("ecology"),
            "hot" => Some("physics"),
            "air" => Some("agriculture"),
            "ground" => Some("measurement"),
            "neural" => Some("neural"),
            "ludo" => Some("game"),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    const MINIMAL_SCENARIO: &str = r#"{
        "name": "Test Scenario",
        "description": "A test scenario",
        "ecosystem": {
            "primals": [
                {
                    "id": "p1",
                    "name": "Primal 1",
                    "type": "healthspring",
                    "family": "healthSpring",
                    "data_channels": [],
                    "clinical_ranges": []
                }
            ]
        }
    }"#;

    #[test]
    fn from_json_minimal() {
        let scenario = LoadedScenario::from_json(MINIMAL_SCENARIO).expect("parse");
        assert_eq!(scenario.name, "Test Scenario");
        assert_eq!(scenario.description, "A test scenario");
        assert_eq!(scenario.ecosystem.primals.len(), 1);
        assert_eq!(scenario.ecosystem.primals[0].id, "p1");
        assert_eq!(scenario.ecosystem.primals[0].family, "healthSpring");
    }

    #[test]
    fn from_json_with_defaults() {
        let json = r#"{
            "name": "Minimal",
            "description": "Desc",
            "ecosystem": {"primals": []}
        }"#;
        let scenario = LoadedScenario::from_json(json).expect("parse");
        assert!(scenario.version.is_empty());
        assert!(scenario.mode.is_empty());
        assert!(scenario.domain.is_none());
        assert!(scenario.edges.is_empty());
    }

    #[test]
    fn from_json_with_data_channels() {
        let json = r#"{
            "name": "With Channels",
            "description": "Desc",
            "ecosystem": {
                "primals": [{
                    "id": "p1",
                    "name": "P1",
                    "data_channels": [{
                        "channel_type": "timeseries",
                        "id": "ts1",
                        "label": "Glucose",
                        "x_label": "Time",
                        "y_label": "mg/dL",
                        "unit": "mg/dL",
                        "x_values": [0.0, 1.0],
                        "y_values": [90.0, 95.0]
                    }],
                    "clinical_ranges": [{
                        "label": "Normal",
                        "min": 70.0,
                        "max": 100.0,
                        "status": "normal"
                    }]
                }]
            }
        }"#;
        let scenario = LoadedScenario::from_json(json).expect("parse");
        let bindings = scenario.all_bindings();
        assert_eq!(bindings.len(), 1);
        let thresholds = scenario.all_thresholds();
        assert_eq!(thresholds.len(), 1);
        assert_eq!(thresholds[0].label, "Normal");
    }

    #[test]
    fn inferred_domain_from_metadata() {
        let json = r#"{
            "name": "D",
            "description": "D",
            "domain": "health",
            "ecosystem": {"primals": []}
        }"#;
        let scenario = LoadedScenario::from_json(json).expect("parse");
        assert_eq!(scenario.inferred_domain(), "health");
    }

    #[test]
    fn inferred_domain_from_family_healthspring() {
        let json = r#"{
            "name": "D",
            "description": "D",
            "ecosystem": {
                "primals": [{"id": "p1", "name": "P1", "family": "healthspring"}]
            }
        }"#;
        let scenario = LoadedScenario::from_json(json).expect("parse");
        assert_eq!(scenario.inferred_domain(), "health");
    }

    #[test]
    fn inferred_domain_from_family_wetspring() {
        let json = r#"{
            "name": "D",
            "description": "D",
            "ecosystem": {
                "primals": [{"id": "p1", "name": "P1", "family": "wetSpring"}]
            }
        }"#;
        let scenario = LoadedScenario::from_json(json).expect("parse");
        assert_eq!(scenario.inferred_domain(), "ecology");
    }

    #[test]
    fn inferred_domain_from_family_hotspring() {
        let json = r#"{
            "name": "D",
            "description": "D",
            "ecosystem": {
                "primals": [{"id": "p1", "name": "P1", "family": "hotSpring"}]
            }
        }"#;
        let scenario = LoadedScenario::from_json(json).expect("parse");
        assert_eq!(scenario.inferred_domain(), "physics");
    }

    #[test]
    fn inferred_domain_from_family_airspring() {
        let json = r#"{
            "name": "D",
            "description": "D",
            "ecosystem": {
                "primals": [{"id": "p1", "name": "P1", "family": "airSpring"}]
            }
        }"#;
        let scenario = LoadedScenario::from_json(json).expect("parse");
        assert_eq!(scenario.inferred_domain(), "agriculture");
    }

    #[test]
    fn inferred_domain_from_family_groundspring() {
        let json = r#"{
            "name": "D",
            "description": "D",
            "ecosystem": {
                "primals": [{"id": "p1", "name": "P1", "family": "groundSpring"}]
            }
        }"#;
        let scenario = LoadedScenario::from_json(json).expect("parse");
        assert_eq!(scenario.inferred_domain(), "measurement");
    }

    #[test]
    fn inferred_domain_from_family_neuralspring() {
        let json = r#"{
            "name": "D",
            "description": "D",
            "ecosystem": {
                "primals": [{"id": "p1", "name": "P1", "family": "neuralSpring"}]
            }
        }"#;
        let scenario = LoadedScenario::from_json(json).expect("parse");
        assert_eq!(scenario.inferred_domain(), "neural");
    }

    #[test]
    fn inferred_domain_from_family_ludospring() {
        let json = r#"{
            "name": "D",
            "description": "D",
            "ecosystem": {
                "primals": [{"id": "p1", "name": "P1", "family": "ludoSpring"}]
            }
        }"#;
        let scenario = LoadedScenario::from_json(json).expect("parse");
        assert_eq!(scenario.inferred_domain(), "game");
    }

    #[test]
    fn inferred_domain_fallback_measurement() {
        let json = r#"{
            "name": "D",
            "description": "D",
            "ecosystem": {
                "primals": [{"id": "p1", "name": "P1", "family": "unknown"}]
            }
        }"#;
        let scenario = LoadedScenario::from_json(json).expect("parse");
        assert_eq!(scenario.inferred_domain(), "measurement");
    }

    #[test]
    fn from_file_nonexistent_fails() {
        let result = LoadedScenario::from_file(Path::new("/nonexistent/path/scenario.json"));
        assert!(result.is_err());
    }

    #[test]
    fn from_json_invalid_fails() {
        let result = LoadedScenario::from_json("{invalid}");
        assert!(result.is_err());
    }

    #[test]
    fn scenario_node_defaults() {
        let json = r#"{
            "name": "D",
            "description": "D",
            "ecosystem": {
                "primals": [{"id": "p1", "name": "P1"}]
            }
        }"#;
        let scenario = LoadedScenario::from_json(json).expect("parse");
        let node = &scenario.ecosystem.primals[0];
        assert!(node.node_type.is_empty());
        assert!(node.family.is_empty());
        assert!(node.status.is_empty());
        assert_eq!(node.health, 0);
        assert!(node.data_channels.is_empty());
        assert!(node.clinical_ranges.is_empty());
        assert!(node.capabilities.is_empty());
    }

    #[test]
    fn scenario_edge_parsing() {
        let json = r#"{
            "name": "D",
            "description": "D",
            "ecosystem": {"primals": []},
            "edges": [
                {"from": "a", "to": "b", "edge_type": "feeds", "label": "feeds"}
            ]
        }"#;
        let scenario = LoadedScenario::from_json(json).expect("parse");
        assert_eq!(scenario.edges.len(), 1);
        assert_eq!(scenario.edges[0].from, "a");
        assert_eq!(scenario.edges[0].to, "b");
        assert_eq!(scenario.edges[0].edge_type, "feeds");
    }

    #[test]
    fn from_file_valid_temp() {
        let temp = std::env::temp_dir().join("petal-scenario-from-file-test.json");
        let json = r#"{
            "name": "File Test",
            "description": "Loaded from file",
            "ecosystem": {"primals": [{"id": "f1", "name": "File Node"}]}
        }"#;
        std::fs::write(&temp, json).expect("write temp file");
        let scenario = LoadedScenario::from_file(&temp).expect("load from file");
        assert_eq!(scenario.name, "File Test");
        assert_eq!(scenario.ecosystem.primals[0].id, "f1");
        let _ = std::fs::remove_file(&temp);
    }

    #[test]
    fn scenario_node_construction() {
        let node = ScenarioNode {
            id: "n1".to_string(),
            name: "Node 1".to_string(),
            node_type: "healthspring".to_string(),
            family: "healthSpring".to_string(),
            status: "healthy".to_string(),
            health: 95,
            data_channels: vec![],
            clinical_ranges: vec![],
            capabilities: vec!["viz".to_string()],
        };
        assert_eq!(node.id, "n1");
        assert_eq!(node.health, 95);
        assert_eq!(node.capabilities.len(), 1);
    }

    #[test]
    fn scenario_edge_construction() {
        let edge = ScenarioEdge {
            from: "src".to_string(),
            to: "dst".to_string(),
            edge_type: "renders".to_string(),
            label: "renders".to_string(),
        };
        assert_eq!(edge.from, "src");
        assert_eq!(edge.to, "dst");
    }

    #[test]
    fn scenario_ecosystem_construction() {
        let eco = ScenarioEcosystem {
            primals: vec![ScenarioNode {
                id: "p1".to_string(),
                name: "P1".to_string(),
                node_type: String::new(),
                family: String::new(),
                status: String::new(),
                health: 0,
                data_channels: vec![],
                clinical_ranges: vec![],
                capabilities: vec![],
            }],
        };
        assert_eq!(eco.primals.len(), 1);
    }

    #[test]
    fn loaded_scenario_serde_roundtrip() {
        let scenario = LoadedScenario::from_json(MINIMAL_SCENARIO).expect("parse");
        let json = serde_json::to_string(&scenario).expect("serialize");
        let decoded = LoadedScenario::from_json(&json).expect("deserialize");
        assert_eq!(decoded.name, scenario.name);
        assert_eq!(
            decoded.ecosystem.primals.len(),
            scenario.ecosystem.primals.len()
        );
    }
}
