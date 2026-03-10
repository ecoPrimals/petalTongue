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

    /// Extract all DataBindings from all nodes.
    pub fn all_bindings(&self) -> Vec<&DataBinding> {
        self.ecosystem
            .primals
            .iter()
            .flat_map(|p| p.data_channels.iter())
            .collect()
    }

    /// Extract all ThresholdRanges from all nodes.
    pub fn all_thresholds(&self) -> Vec<&ThresholdRange> {
        self.ecosystem
            .primals
            .iter()
            .flat_map(|p| p.clinical_ranges.iter())
            .collect()
    }

    /// Infer the domain from the scenario metadata or node families.
    pub fn inferred_domain(&self) -> &str {
        if let Some(ref d) = self.domain {
            return d;
        }
        // Check node families for domain hints
        for node in &self.ecosystem.primals {
            match node.family.as_str() {
                "healthspring" | "healthSpring" => return "health",
                "wetspring" | "wetSpring" => return "ecology",
                "hotspring" | "hotSpring" => return "physics",
                "airspring" | "airSpring" => return "agriculture",
                "groundspring" | "groundSpring" => return "measurement",
                "neuralspring" | "neuralSpring" => return "neural",
                _ => {}
            }
        }
        "measurement"
    }
}
