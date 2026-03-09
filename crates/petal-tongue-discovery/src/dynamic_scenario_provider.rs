// SPDX-License-Identifier: AGPL-3.0-only
//! Dynamic scenario provider using schema-agnostic data structures
//!
//! This provider replaces the static, brittle scenario_provider.rs with
//! a fully dynamic implementation that can handle evolving JSON schemas
//! without recompilation.
//!
//! # Philosophy
//!
//! **Schemas evolve. Code should adapt.**
//!
//! Instead of hardcoding `PrimalDefinition` with specific fields,
//! we use `DynamicData` to capture ALL fields (known and unknown).

use crate::traits::{ProviderMetadata, VisualizationDataProvider};
use anyhow::{Context, Result};
use async_trait::async_trait;
use petal_tongue_core::{
    DynamicData, DynamicValue, PrimalHealthStatus, PrimalInfo, Properties, PropertyValue,
    TopologyEdge,
};
use std::path::Path;

/// Dynamic scenario provider (schema-agnostic)
pub struct DynamicScenarioProvider {
    /// Scenario data (fully dynamic)
    scenario: DynamicData,

    /// Extracted primals (for performance)
    primals: Vec<PrimalInfo>,

    /// Schema version string (if present)
    version: Option<String>,
}

impl DynamicScenarioProvider {
    /// Create from file (auto-detects and migrates schema)
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        // Load JSON content
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read scenario file: {}", path.display()))?;

        // Parse as DynamicData (flexible schema)
        let scenario = DynamicData::from_json_str(&content)?;

        // Extract schema version string (if present)
        let version = scenario.get_str("version").map(String::from);

        if let Some(ref v) = version {
            tracing::info!("📋 Scenario schema version: {}", v);
        }

        // Extract primals from dynamic data
        let primals = Self::extract_primals(&scenario)?;

        tracing::info!("📋 Loaded dynamic scenario: {} primals", primals.len());

        Ok(Self {
            scenario,
            primals,
            version,
        })
    }

    /// Extract primals from dynamic scenario data
    fn extract_primals(scenario: &DynamicData) -> Result<Vec<PrimalInfo>> {
        let ecosystem = scenario
            .get("ecosystem")
            .and_then(|v| v.as_object())
            .context("Missing 'ecosystem' field in scenario")?;

        let primals_array = ecosystem
            .get("primals")
            .and_then(|v| v.as_array())
            .context("Missing 'primals' array in ecosystem")?;

        let now = chrono::Utc::now().timestamp() as u64;
        let mut primals = Vec::new();

        for (idx, primal_value) in primals_array.iter().enumerate() {
            let primal_obj = primal_value
                .as_object()
                .with_context(|| format!("Primal {idx} is not an object"))?;

            // Required fields (graceful fallback if missing)
            let id = Self::get_string(primal_obj, "id").unwrap_or_else(|| format!("primal-{idx}"));
            let name = Self::get_string(primal_obj, "name")
                .unwrap_or_else(|| format!("Unknown Primal {idx}"));
            let primal_type =
                Self::get_string(primal_obj, "type").unwrap_or_else(|| "Unknown".to_string());

            // Health status (with fallback)
            let health = Self::get_string(primal_obj, "status")
                .and_then(|s| match s.as_str() {
                    "healthy" => Some(PrimalHealthStatus::Healthy),
                    "warning" => Some(PrimalHealthStatus::Warning),
                    "critical" => Some(PrimalHealthStatus::Critical),
                    _ => None,
                })
                .unwrap_or(PrimalHealthStatus::Unknown);

            // Convert ALL fields to Properties (dynamic!)
            let mut properties = Properties::new();
            for (key, value) in primal_obj {
                if let Some(prop_value) = Self::dynamic_to_property(value) {
                    properties.insert(key.clone(), prop_value);
                }
            }

            // Extract capabilities (if present)
            let capabilities = primal_obj
                .get("capabilities")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            // Extract family (if present)
            let family_id = Self::get_string(primal_obj, "family");

            primals.push(PrimalInfo {
                id: id.into(),
                name,
                primal_type,
                endpoint: "scenario://dynamic".to_string(),
                capabilities,
                health,
                last_seen: now,
                endpoints: None,
                metadata: None,
                properties,
                #[expect(deprecated)]
                trust_level: None,
                #[expect(deprecated)]
                family_id,
            });
        }

        Ok(primals)
    }

    /// Get string from dynamic object (helper)
    fn get_string(
        obj: &std::collections::HashMap<String, DynamicValue>,
        key: &str,
    ) -> Option<String> {
        obj.get(key)?.as_str().map(String::from)
    }

    /// Convert DynamicValue to PropertyValue
    fn dynamic_to_property(value: &DynamicValue) -> Option<PropertyValue> {
        match value {
            DynamicValue::String(s) => Some(PropertyValue::String(s.clone())),
            DynamicValue::Number(n) => Some(PropertyValue::Number(*n)),
            DynamicValue::Boolean(b) => Some(PropertyValue::Boolean(*b)),
            DynamicValue::Array(arr) => {
                let prop_arr: Vec<PropertyValue> =
                    arr.iter().filter_map(Self::dynamic_to_property).collect();
                Some(PropertyValue::Array(prop_arr))
            }
            DynamicValue::Object(obj) => {
                let prop_obj: std::collections::HashMap<String, PropertyValue> = obj
                    .iter()
                    .filter_map(|(k, v)| Self::dynamic_to_property(v).map(|pv| (k.clone(), pv)))
                    .collect();
                Some(PropertyValue::Object(prop_obj))
            }
            DynamicValue::Null => None,
        }
    }

    /// Get scenario name (if present)
    pub fn name(&self) -> Option<&str> {
        self.scenario.get_str("name")
    }

    /// Get scenario version string
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    /// Get arbitrary scenario field (for custom data)
    pub fn get_field(&self, key: &str) -> Option<&DynamicValue> {
        self.scenario.get(key)
    }
}

#[async_trait]
impl VisualizationDataProvider for DynamicScenarioProvider {
    async fn get_primals(&self) -> Result<Vec<PrimalInfo>> {
        Ok(self.primals.clone())
    }

    async fn get_topology(&self) -> Result<Vec<TopologyEdge>> {
        // Auto-generate topology (NUCLEUS-centric or ring mesh)
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
        } else if self.primals.len() > 1 {
            // No NUCLEUS: Create ring mesh
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
        let name = self.name().unwrap_or("Dynamic Scenario");
        Ok(format!(
            "Dynamic scenario '{}' with {} primals",
            name,
            self.primals.len()
        ))
    }

    fn get_metadata(&self) -> ProviderMetadata {
        let name = self.name().unwrap_or("Dynamic Scenario");
        ProviderMetadata {
            name: format!("Dynamic Scenario: {name}"),
            endpoint: "scenario://dynamic".to_string(),
            protocol: "dynamic".to_string(),
            capabilities: vec!["visualization".to_string(), "topology".to_string()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_scenario_minimal() {
        let json = r#"{
            "name": "Minimal Test",
            "version": "1.0.0",
            "ecosystem": {
                "primals": [
                    {
                        "id": "test-1",
                        "name": "TEST",
                        "type": "test",
                        "status": "healthy"
                    }
                ]
            }
        }"#;

        let temp_file = std::env::temp_dir().join("test_dynamic_minimal.json");
        std::fs::write(&temp_file, json).unwrap();

        let provider = DynamicScenarioProvider::from_file(&temp_file).unwrap();
        assert_eq!(provider.name(), Some("Minimal Test"));

        let version = provider
            .scenario
            .version
            .as_ref()
            .expect("version should be parsed");
        assert_eq!(version.to_string(), "1.0.0");

        assert_eq!(provider.primals.len(), 1);
        assert_eq!(provider.primals[0].name, "TEST");

        std::fs::remove_file(&temp_file).ok();
    }

    #[test]
    fn test_dynamic_scenario_unknown_fields() {
        // Test that unknown fields are captured
        let json = r#"{
            "name": "Unknown Fields Test",
            "ecosystem": {
                "primals": [
                    {
                        "id": "test-1",
                        "name": "TEST",
                        "type": "test",
                        "status": "healthy",
                        "custom_field": "custom_value",
                        "tier": 3
                    }
                ]
            }
        }"#;

        let temp_file = std::env::temp_dir().join("test_dynamic_unknown.json");
        std::fs::write(&temp_file, json).unwrap();

        let provider = DynamicScenarioProvider::from_file(&temp_file).unwrap();
        let primal = &provider.primals[0];

        // Unknown fields should be in properties
        assert!(primal.properties.get("custom_field").is_some());
        assert!(primal.properties.get("tier").is_some());

        std::fs::remove_file(&temp_file).ok();
    }

    #[tokio::test]
    async fn test_dynamic_provider_interface() {
        let json = r#"{
            "name": "Interface Test",
            "ecosystem": {
                "primals": [
                    {
                        "id": "primal-1",
                        "name": "Primal1",
                        "type": "test",
                        "status": "healthy"
                    },
                    {
                        "id": "primal-2",
                        "name": "Primal2",
                        "type": "test",
                        "status": "healthy"
                    }
                ]
            }
        }"#;

        let temp_file = std::env::temp_dir().join("test_dynamic_interface.json");
        std::fs::write(&temp_file, json).unwrap();

        let provider = DynamicScenarioProvider::from_file(&temp_file).unwrap();

        // Test VisualizationDataProvider interface
        let primals = provider.get_primals().await.unwrap();
        assert_eq!(primals.len(), 2);

        let topology = provider.get_topology().await.unwrap();
        assert!(!topology.is_empty()); // Should have ring mesh

        let health = provider.health_check().await.unwrap();
        assert!(health.contains("2 primals"));

        let metadata = provider.get_metadata();
        assert!(metadata.name.contains("Interface Test"));

        std::fs::remove_file(&temp_file).ok();
    }
}
