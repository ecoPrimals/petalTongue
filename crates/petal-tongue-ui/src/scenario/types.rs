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
    pub fn primal_count(&self) -> usize {
        self.ecosystem.primals.len()
    }

    /// Get number of edges in scenario
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Validate scenario configuration
    pub fn validate(&self) -> anyhow::Result<()> {
        // Check required fields
        if self.name.trim().is_empty() {
            anyhow::bail!("Scenario name cannot be empty");
        }

        if self.mode.trim().is_empty() {
            anyhow::bail!(
                "Scenario mode cannot be empty (e.g., 'doom-showcase', 'live-ecosystem')"
            );
        }

        if self.version.trim().is_empty() {
            anyhow::bail!("Scenario version cannot be empty");
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
