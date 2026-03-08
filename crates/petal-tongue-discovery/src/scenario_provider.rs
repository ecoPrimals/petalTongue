// SPDX-License-Identifier: AGPL-3.0-only
//! Scenario-based visualization provider for benchTop demonstrations
//!
//! This provider loads data from JSON scenario files for rich, pre-defined
//! demonstrations of petalTongue capabilities.

use crate::traits::VisualizationDataProvider;
use anyhow::{Context, Result};
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
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read scenario file: {}", path.display()))?;

        let scenario: ScenarioFile = serde_json::from_str(&contents)
            .with_context(|| format!("Failed to parse scenario JSON: {}", path.display()))?;

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
                properties.insert(
                    "family_id".to_string(),
                    PropertyValue::String(p.family.clone()),
                );

                if let Some(pos) = p.position {
                    properties.insert(
                        "position_x".to_string(),
                        PropertyValue::Number(pos.x as f64),
                    );
                    properties.insert(
                        "position_y".to_string(),
                        PropertyValue::Number(pos.y as f64),
                    );
                }

                PrimalInfo {
                    id: p.id.clone(),
                    name: p.name.clone(),
                    primal_type: p.primal_type.clone(),
                    endpoint: format!("scenario://{}", p.id),
                    capabilities: p.capabilities.clone(),
                    health,
                    last_seen: now,
                    endpoints: None,
                    metadata: None,
                    properties,
                    #[expect(deprecated)]
                    trust_level: None,
                    #[expect(deprecated)]
                    family_id: Some(p.family),
                }
            })
            .collect();

        tracing::info!("📋 Loaded {} primals from scenario", primals.len());

        Ok(Self { primals })
    }
}

#[async_trait]
impl VisualizationDataProvider for ScenarioVisualizationProvider {
    async fn get_primals(&self) -> Result<Vec<PrimalInfo>> {
        Ok(self.primals.clone())
    }

    async fn get_topology(&self) -> Result<Vec<TopologyEdge>> {
        // Generate automatic topology based on primal types
        let mut edges = Vec::new();

        // Find NUCLEUS if it exists
        if let Some(nucleus) = self.primals.iter().find(|p| p.name == "NUCLEUS") {
            // Connect NUCLEUS to all other primals
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
            // Without NUCLEUS, create a mesh topology
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

    async fn health_check(&self) -> Result<String> {
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
}
