//! Core scenario types for benchTop demonstrations
//!
//! This module defines the main Scenario structure and related types
//! for loading and managing UI demonstration scenarios.

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::scenario::config::{
    AnimationConfig, CustomPanelConfig, FeatureFlags, PanelVisibility, PerformanceConfig, UiConfig,
};
use crate::scenario::ecosystem::{Ecosystem, PrimalDefinition};
use crate::scenario::sensory::{CapabilityRequirements, SensoryConfig};

/// Complete scenario definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub name: String,
    pub description: String,
    pub version: String,
    pub mode: String,
    #[serde(default)]
    pub ui_config: UiConfig,
    #[serde(default)]
    pub ecosystem: Ecosystem,
    #[serde(default)]
    pub neural_api: NeuralApiConfig,
    /// Sensory capability configuration (v2.2.0)
    #[serde(default)]
    pub sensory_config: SensoryConfig,
}

impl Scenario {
    /// Get number of primals in scenario
    pub fn primal_count(&self) -> usize {
        self.ecosystem.primals.len()
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

/// Motor capabilities
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Motor {
    #[serde(default)]
    pub can_deploy: bool,
    #[serde(default)]
    pub can_execute_graphs: bool,
    #[serde(default)]
    pub can_coordinate_primals: bool,
}

/// Sensory capabilities
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Sensory {
    #[serde(default)]
    pub active_sockets: usize,
    #[serde(default)]
    pub last_scan: Option<String>,
}
