// SPDX-License-Identifier: AGPL-3.0-only
//! Ecosystem and primal definition types
//!
//! Defines structures for primals in the ecosystem including their
//! positions, metrics, and proprioception data.

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
