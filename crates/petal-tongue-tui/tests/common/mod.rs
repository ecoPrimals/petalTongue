// SPDX-License-Identifier: AGPL-3.0-only
//! Common test utilities for petalTongue TUI tests

use chrono::Utc;
use petal_tongue_core::{PrimalHealthStatus, PrimalInfo, TopologyEdge};

/// Create a test PrimalInfo with minimal required fields
pub fn create_test_primal(name: &str, id: &str) -> PrimalInfo {
    PrimalInfo::new(
        id,
        name,
        "Test",
        format!("unix:///tmp/{}.sock", name),
        vec![],
        PrimalHealthStatus::Healthy,
        Utc::now().timestamp() as u64,
    )
}

/// Create a test PrimalInfo with capabilities
pub fn create_test_primal_with_caps(name: &str, id: &str, capabilities: Vec<String>) -> PrimalInfo {
    PrimalInfo::new(
        id,
        name,
        "Test",
        format!("unix:///tmp/{}.sock", name),
        capabilities,
        PrimalHealthStatus::Healthy,
        Utc::now().timestamp() as u64,
    )
}

/// Create a test PrimalInfo with specific health status
pub fn create_test_primal_with_health(
    name: &str,
    id: &str,
    health: PrimalHealthStatus,
) -> PrimalInfo {
    PrimalInfo::new(
        id,
        name,
        "Test",
        format!("unix:///tmp/{}.sock", name),
        vec![],
        health,
        Utc::now().timestamp() as u64,
    )
}

/// Create a test TopologyEdge
pub fn create_test_edge(from: &str, to: &str, edge_type: &str) -> TopologyEdge {
    TopologyEdge {
        from: from.to_string(),
        to: to.to_string(),
        edge_type: edge_type.to_string(),
        label: None,
        capability: None,
        metrics: None,
    }
}
