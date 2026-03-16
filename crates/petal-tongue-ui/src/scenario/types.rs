// SPDX-License-Identifier: AGPL-3.0-only
//! Core scenario types for benchTop demonstrations
//!
//! This module defines the main Scenario structure and related types
//! for loading and managing UI demonstration scenarios.

use serde::{Deserialize, Serialize};

use petal_tongue_core::TopologyEdge;

use crate::scenario::config::UiConfig;
use crate::scenario::ecosystem::Ecosystem;
use crate::scenario::sensory::SensoryConfig;

/// Complete scenario definition for benchTop demonstrations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    /// Human-readable name for the scenario
    pub name: String,
    /// Description of what this scenario demonstrates
    pub description: String,
    /// Semantic version of the scenario format (e.g., "2.0.0")
    pub version: String,
    /// Scenario mode (e.g., "doom-showcase", "live-ecosystem")
    pub mode: String,
    /// UI configuration (panels, theme, layout)
    #[serde(default)]
    pub ui_config: UiConfig,
    /// Ecosystem definition (primals, families)
    #[serde(default)]
    pub ecosystem: Ecosystem,
    /// Neural API integration settings
    #[serde(default)]
    pub neural_api: NeuralApiConfig,
    /// Sensory capability configuration (v2.2.0)
    #[serde(default)]
    pub sensory_config: SensoryConfig,
    /// Topology edges between primals (optional, used by healthSpring clinical scenarios)
    #[serde(default)]
    pub edges: Vec<TopologyEdge>,
}

impl Scenario {
    /// Get number of primals in scenario
    #[must_use]
    pub const fn primal_count(&self) -> usize {
        self.ecosystem.primals.len()
    }

    /// Get number of edges in scenario
    #[must_use]
    pub const fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Validate scenario configuration
    pub fn validate(&self) -> crate::error::Result<()> {
        // Check required fields
        if self.name.trim().is_empty() {
            return Err(crate::scenario_error::ScenarioError::MissingField {
                field: "name".to_string(),
                suggestion: Some("Add a non-empty 'name' field".to_string()),
            }
            .into());
        }

        if self.mode.trim().is_empty() {
            return Err(crate::scenario_error::ScenarioError::MissingField {
                field: "mode".to_string(),
                suggestion: Some("e.g., 'doom-showcase', 'live-ecosystem'".to_string()),
            }
            .into());
        }

        if self.version.trim().is_empty() {
            return Err(crate::scenario_error::ScenarioError::MissingField {
                field: "version".to_string(),
                suggestion: Some("e.g., '2.0.0'".to_string()),
            }
            .into());
        }

        // Validate version format (should be semver-like)
        let version_parts: Vec<&str> = self.version.split('.').collect();
        if version_parts.len() < 2 {
            tracing::warn!(
                "⚠️  Scenario version '{}' doesn't follow semver (e.g., '2.0.0')",
                self.version
            );
        }

        // Validate UI config
        self.ui_config.validate()?;

        // Validate sensory config
        self.sensory_config.validate()?;

        // Validate that we have at least one primal (unless it's a tutorial)
        if self.ecosystem.primals.is_empty() && self.mode != "tutorial" {
            tracing::warn!("⚠️  Scenario has no primals (mode: {})", self.mode);
        }

        Ok(())
    }
}

/// Neural API configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NeuralApiConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub learning_rate: f32,
    #[serde(default)]
    pub optimization_cycles: usize,
}

/// Proprioception data in scenario
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScenarioProprioception {
    #[serde(default)]
    pub self_awareness: SelfAwareness,
    #[serde(default)]
    pub motor: Motor,
    #[serde(default)]
    pub sensory: Sensory,
}

/// Self-awareness capabilities
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SelfAwareness {
    #[serde(default)]
    pub knows_about: usize,
    #[serde(default)]
    pub can_coordinate: bool,
    #[serde(default)]
    pub has_security: bool,
    #[serde(default)]
    pub has_discovery: bool,
    #[serde(default)]
    pub has_compute: bool,
}

/// Motor capabilities - actions this primal can perform
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Motor {
    /// Can deploy niches and resources
    #[serde(default)]
    pub can_deploy: bool,
    /// Can execute computational graphs
    #[serde(default)]
    pub can_execute_graphs: bool,
    /// Can coordinate other primals in the ecosystem
    #[serde(default)]
    pub can_coordinate_primals: bool,
}

/// Sensory capabilities for input/output detection
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Sensory {
    /// Number of currently active socket connections for sensory input
    #[serde(default)]
    pub active_sockets: usize,
    /// Timestamp of the last sensory capability scan (ISO 8601 format)
    #[serde(default)]
    pub last_scan: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::config::UiConfig;
    use crate::scenario::ecosystem::{Ecosystem, Position, PrimalDefinition};
    use crate::scenario::sensory::SensoryConfig;

    fn minimal_scenario() -> Scenario {
        let mut sensory = SensoryConfig::default();
        sensory.complexity_hint = "auto".to_string();
        Scenario {
            name: "Test".to_string(),
            description: "Desc".to_string(),
            version: "2.0.0".to_string(),
            mode: "doom-showcase".to_string(),
            ui_config: UiConfig::default(),
            ecosystem: Ecosystem::default(),
            neural_api: NeuralApiConfig::default(),
            sensory_config: sensory,
            edges: vec![],
        }
    }

    #[test]
    fn scenario_primal_count() {
        let mut s = minimal_scenario();
        assert_eq!(s.primal_count(), 0);
        s.ecosystem.primals.push(PrimalDefinition {
            id: "1".to_string(),
            name: "P1".to_string(),
            primal_type: "t".to_string(),
            family: "f".to_string(),
            status: "ok".to_string(),
            health: 100,
            confidence: 100,
            position: Position { x: 0.0, y: 0.0 },
            capabilities: vec![],
            metrics: Default::default(),
            proprioception: None,
            data_bindings: vec![],
            threshold_ranges: vec![],
        });
        assert_eq!(s.primal_count(), 1);
    }

    #[test]
    fn scenario_edge_count() {
        let mut s = minimal_scenario();
        assert_eq!(s.edge_count(), 0);
        s.edges.push(TopologyEdge {
            from: petal_tongue_core::PrimalId::from("a"),
            to: petal_tongue_core::PrimalId::from("b"),
            edge_type: "test".to_string(),
            label: None,
            capability: None,
            metrics: None,
        });
        assert_eq!(s.edge_count(), 1);
    }

    #[test]
    fn scenario_validate_empty_name_fails() {
        let mut s = minimal_scenario();
        s.name = "".to_string();
        assert!(s.validate().is_err());
    }

    #[test]
    fn scenario_validate_empty_mode_fails() {
        let mut s = minimal_scenario();
        s.mode = "".to_string();
        assert!(s.validate().is_err());
    }

    #[test]
    fn scenario_validate_empty_version_fails() {
        let mut s = minimal_scenario();
        s.version = "".to_string();
        assert!(s.validate().is_err());
    }

    #[test]
    fn scenario_validate_tutorial_allows_empty_primals() {
        let mut s = minimal_scenario();
        s.mode = "tutorial".to_string();
        assert!(s.validate().is_ok());
    }

    #[test]
    fn scenario_validate_ok() {
        let mut s = minimal_scenario();
        s.ecosystem.primals.push(PrimalDefinition {
            id: "1".to_string(),
            name: "P1".to_string(),
            primal_type: "t".to_string(),
            family: "f".to_string(),
            status: "ok".to_string(),
            health: 100,
            confidence: 100,
            position: Position { x: 0.0, y: 0.0 },
            capabilities: vec![],
            metrics: Default::default(),
            proprioception: None,
            data_bindings: vec![],
            threshold_ranges: vec![],
        });
        assert!(s.validate().is_ok());
    }

    #[test]
    fn neural_api_config_default() {
        let c = NeuralApiConfig::default();
        assert!(!c.enabled);
        assert_eq!(c.learning_rate, 0.0);
        assert_eq!(c.optimization_cycles, 0);
    }

    #[test]
    fn scenario_proprioception_default() {
        let p = ScenarioProprioception::default();
        assert_eq!(p.self_awareness.knows_about, 0);
        assert!(!p.motor.can_deploy);
    }

    #[test]
    fn scenario_serialization_roundtrip() {
        let s = minimal_scenario();
        let json = serde_json::to_string(&s).unwrap();
        let parsed: Scenario = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, s.name);
        assert_eq!(parsed.version, s.version);
    }
}
