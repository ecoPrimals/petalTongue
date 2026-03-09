// SPDX-License-Identifier: AGPL-3.0-only
//! Ecosystem and primal definition types
//!
//! Defines structures for primals in the ecosystem including their
//! positions, metrics, and proprioception data.

use petal_tongue_core::{DataBinding, ThresholdRange};
use serde::{Deserialize, Serialize};

use crate::scenario::types::ScenarioProprioception;

/// Ecosystem containing primals
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Ecosystem {
    #[serde(default)]
    pub primals: Vec<PrimalDefinition>,
}

/// Single primal definition in scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalDefinition {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub primal_type: String,
    pub family: String,
    pub status: String,
    pub health: u8,
    pub confidence: u8,
    pub position: Position,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub metrics: PrimalMetrics,
    #[serde(default)]
    pub proprioception: Option<ScenarioProprioception>,
    #[serde(default, rename = "data_bindings", alias = "data_channels")]
    pub data_bindings: Vec<DataBinding>,
    #[serde(default, rename = "threshold_ranges", alias = "clinical_ranges")]
    pub threshold_ranges: Vec<ThresholdRange>,
}

/// 2D position in scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

/// Primal metrics in scenario
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrimalMetrics {
    #[serde(default)]
    pub cpu_percent: f32,
    #[serde(default)]
    pub memory_mb: u64,
    #[serde(default)]
    pub uptime_seconds: u64,
    #[serde(default)]
    pub requests_per_second: u64,
    #[serde(default)]
    pub active_primals: usize,
    #[serde(default)]
    pub graphs_available: usize,
    #[serde(default)]
    pub active_executions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ecosystem_default_empty() {
        let eco = Ecosystem::default();
        assert!(eco.primals.is_empty());
    }

    #[test]
    fn ecosystem_with_primals() {
        let eco = Ecosystem {
            primals: vec![PrimalDefinition {
                id: "p1".to_string(),
                name: "Test".to_string(),
                primal_type: "test".to_string(),
                family: "fam".to_string(),
                status: "healthy".to_string(),
                health: 100,
                confidence: 90,
                position: Position { x: 1.0, y: 2.0 },
                capabilities: vec!["cap".to_string()],
                metrics: PrimalMetrics::default(),
                proprioception: None,
                data_bindings: vec![],
                threshold_ranges: vec![],
            }],
        };
        assert_eq!(eco.primals.len(), 1);
        assert_eq!(eco.primals[0].id, "p1");
        assert_eq!(eco.primals[0].position.x, 1.0);
    }

    #[test]
    fn primal_metrics_defaults() {
        let m = PrimalMetrics::default();
        assert_eq!(m.cpu_percent, 0.0);
        assert_eq!(m.memory_mb, 0);
        assert_eq!(m.uptime_seconds, 0);
    }

    #[test]
    fn position_serialization() {
        let p = Position { x: 10.5, y: 20.0 };
        let json = serde_json::to_string(&p).unwrap();
        let parsed: Position = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.x, 10.5);
        assert_eq!(parsed.y, 20.0);
    }
}
