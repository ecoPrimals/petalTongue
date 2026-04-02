// SPDX-License-Identifier: AGPL-3.0-or-later
//! Scenario-based visualization provider for benchTop demonstrations
//!
//! This provider loads data from JSON scenario files for rich, pre-defined
//! demonstrations of petalTongue capabilities.

use crate::errors::{DiscoveryError, DiscoveryResult};
use crate::traits::VisualizationDataProvider;
use async_trait::async_trait;
use petal_tongue_core::PrimalHealthStatus;
use petal_tongue_core::{PrimalInfo, Properties, PropertyValue, TopologyEdge};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Provider that loads data from a scenario JSON file
pub struct ScenarioVisualizationProvider {
    primals: Vec<PrimalInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScenarioPrimal {
    id: String,
    name: String,
    #[serde(rename = "type")]
    primal_type: String,
    family: String,
    status: String,
    #[serde(default)]
    capabilities: Vec<String>,
    #[serde(default)]
    position: Option<Position>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScenarioFile {
    ecosystem: Ecosystem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Ecosystem {
    #[serde(default)]
    primals: Vec<ScenarioPrimal>,
}

impl ScenarioVisualizationProvider {
    /// Create a new scenario provider from a JSON file
    ///
    /// # Errors
    /// Returns `DiscoveryError::FileReadError` if the file cannot be read,
    /// or `DiscoveryError::ScenarioParseError` if the JSON is invalid.
    pub fn from_file<P: AsRef<Path>>(path: P) -> DiscoveryResult<Self> {
        let path = path.as_ref();
        let contents =
            std::fs::read_to_string(path).map_err(|e| DiscoveryError::FileReadError {
                path: path.display().to_string(),
                source: e,
            })?;

        let scenario: ScenarioFile =
            serde_json::from_str(&contents).map_err(|e| DiscoveryError::ScenarioParseError {
                message: format!("Failed to parse scenario JSON: {} - {e}", path.display()),
            })?;

        #[expect(
            clippy::cast_sign_loss,
            reason = "Unix timestamp for current time is always non-negative"
        )]
        let now = chrono::Utc::now().timestamp() as u64;
        let primals: Vec<PrimalInfo> = scenario
            .ecosystem
            .primals
            .into_iter()
            .map(|p| {
                let health = match p.status.as_str() {
                    "healthy" => PrimalHealthStatus::Healthy,
                    "warning" => PrimalHealthStatus::Warning,
                    "critical" => PrimalHealthStatus::Critical,
                    _ => PrimalHealthStatus::Unknown,
                };

                let mut properties = Properties::new();
                properties.insert("family_id".to_string(), PropertyValue::String(p.family));

                if let Some(pos) = p.position {
                    properties.insert(
                        "position_x".to_string(),
                        PropertyValue::Number(f64::from(pos.x)),
                    );
                    properties.insert(
                        "position_y".to_string(),
                        PropertyValue::Number(f64::from(pos.y)),
                    );
                }

                let endpoint = format!("scenario://{}", p.id);
                PrimalInfo {
                    id: p.id.into(),
                    name: p.name,
                    primal_type: p.primal_type,
                    endpoint,
                    capabilities: p.capabilities,
                    health,
                    last_seen: now,
                    endpoints: None,
                    metadata: None,
                    properties,
                }
            })
            .collect();

        tracing::info!("📋 Loaded {} primals from scenario", primals.len());

        Ok(Self { primals })
    }
}

#[async_trait]
impl VisualizationDataProvider for ScenarioVisualizationProvider {
    async fn get_primals(&self) -> DiscoveryResult<Vec<PrimalInfo>> {
        Ok(self.primals.clone())
    }

    async fn get_topology(&self) -> DiscoveryResult<Vec<TopologyEdge>> {
        // Generate automatic topology based on primal types
        let mut edges = Vec::new();

        // Find nucleus-type primal if one exists (type-based, not name-based)
        if let Some(nucleus) = self.primals.iter().find(|p| p.primal_type == "nucleus") {
            // Connect nucleus to all other primals (star topology)
            for primal in &self.primals {
                if primal.id != nucleus.id {
                    edges.push(TopologyEdge {
                        from: nucleus.id.clone(),
                        to: primal.id.clone(),
                        edge_type: "coordination".to_string(),
                        label: Some("Neural API".to_string()),
                        capability: None,
                        metrics: None,
                    });
                }
            }
        } else {
            // Without a nucleus primal, create a mesh topology
            // Connect each primal to the next one in a ring
            for i in 0..self.primals.len() {
                let next = (i + 1) % self.primals.len();
                edges.push(TopologyEdge {
                    from: self.primals[i].id.clone(),
                    to: self.primals[next].id.clone(),
                    edge_type: "peer".to_string(),
                    label: Some("Peer connection".to_string()),
                    capability: None,
                    metrics: None,
                });
            }
        }

        Ok(edges)
    }

    async fn health_check(&self) -> DiscoveryResult<String> {
        Ok(format!(
            "Scenario provider with {} primals",
            self.primals.len()
        ))
    }

    fn get_metadata(&self) -> crate::traits::ProviderMetadata {
        crate::traits::ProviderMetadata {
            name: "Scenario Provider".to_string(),
            endpoint: "scenario://local".to_string(),
            protocol: "scenario".to_string(),
            capabilities: vec!["visualization".to_string(), "topology".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scenario_provider_creation() {
        let json = r#"{
            "ecosystem": {
                "primals": [
                    {
                        "id": "test-1",
                        "name": "TEST",
                        "type": "test",
                        "family": "nat0",
                        "status": "healthy",
                        "capabilities": ["test"],
                        "position": { "x": 100.0, "y": 200.0 }
                    }
                ]
            }
        }"#;

        let temp_file = std::env::temp_dir().join("test_scenario.json");
        std::fs::write(&temp_file, json).unwrap();

        let provider = ScenarioVisualizationProvider::from_file(&temp_file).unwrap();
        let primals = provider.get_primals().await.unwrap();

        assert_eq!(primals.len(), 1);
        assert_eq!(primals[0].name, "TEST");

        std::fs::remove_file(&temp_file).ok();
    }

    #[tokio::test]
    async fn test_scenario_topology_with_nucleus() {
        let json = r#"{
            "ecosystem": {
                "primals": [
                    {"id": "n1", "name": "NUCLEUS", "type": "nucleus", "family": "nat0", "status": "healthy"},
                    {"id": "p1", "name": "Primal1", "type": "compute", "family": "nat0", "status": "healthy"}
                ]
            }
        }"#;
        let temp_file = std::env::temp_dir().join("test_scenario_nucleus.json");
        std::fs::write(&temp_file, json).unwrap();
        let provider = ScenarioVisualizationProvider::from_file(&temp_file).unwrap();
        let topology = provider.get_topology().await.unwrap();
        assert_eq!(topology.len(), 1);
        assert_eq!(topology[0].edge_type, "coordination");
        std::fs::remove_file(&temp_file).ok();
    }

    #[tokio::test]
    async fn test_scenario_topology_mesh_without_nucleus() {
        let json = r#"{
            "ecosystem": {
                "primals": [
                    {"id": "a", "name": "A", "type": "t", "family": "nat0", "status": "healthy"},
                    {"id": "b", "name": "B", "type": "t", "family": "nat0", "status": "warning"}
                ]
            }
        }"#;
        let temp_file = std::env::temp_dir().join("test_scenario_mesh.json");
        std::fs::write(&temp_file, json).unwrap();
        let provider = ScenarioVisualizationProvider::from_file(&temp_file).unwrap();
        let topology = provider.get_topology().await.unwrap();
        assert_eq!(topology.len(), 2);
        assert_eq!(topology[0].edge_type, "peer");
        std::fs::remove_file(&temp_file).ok();
    }

    #[tokio::test]
    async fn test_scenario_health_status_variants() {
        let json = r#"{
            "ecosystem": {
                "primals": [
                    {"id": "h", "name": "H", "type": "t", "family": "nat0", "status": "healthy"},
                    {"id": "w", "name": "W", "type": "t", "family": "nat0", "status": "warning"},
                    {"id": "c", "name": "C", "type": "t", "family": "nat0", "status": "critical"},
                    {"id": "u", "name": "U", "type": "t", "family": "nat0", "status": "unknown"}
                ]
            }
        }"#;
        let temp_file = std::env::temp_dir().join("test_scenario_health.json");
        std::fs::write(&temp_file, json).unwrap();
        let provider = ScenarioVisualizationProvider::from_file(&temp_file).unwrap();
        let primals = provider.get_primals().await.unwrap();
        assert_eq!(primals.len(), 4);
        assert!(matches!(primals[0].health, PrimalHealthStatus::Healthy));
        assert!(matches!(primals[1].health, PrimalHealthStatus::Warning));
        assert!(matches!(primals[2].health, PrimalHealthStatus::Critical));
        assert!(matches!(primals[3].health, PrimalHealthStatus::Unknown));
        std::fs::remove_file(&temp_file).ok();
    }

    #[tokio::test]
    async fn test_scenario_metadata() {
        let json = r#"{"ecosystem": {"primals": []}}"#;
        let temp_file = std::env::temp_dir().join("test_scenario_meta.json");
        std::fs::write(&temp_file, json).unwrap();
        let provider = ScenarioVisualizationProvider::from_file(&temp_file).unwrap();
        let meta = provider.get_metadata();
        assert_eq!(meta.name, "Scenario Provider");
        assert!(meta.endpoint.contains("scenario"));
        std::fs::remove_file(&temp_file).ok();
    }

    #[test]
    fn test_scenario_from_file_nonexistent() {
        let result = ScenarioVisualizationProvider::from_file("/nonexistent/scenario-12345.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_scenario_from_file_invalid_json() {
        let temp_file = std::env::temp_dir().join("test_scenario_invalid.json");
        std::fs::write(&temp_file, "invalid json {").unwrap();
        let result = ScenarioVisualizationProvider::from_file(&temp_file);
        std::fs::remove_file(&temp_file).ok();
        assert!(result.is_err());
    }
}
