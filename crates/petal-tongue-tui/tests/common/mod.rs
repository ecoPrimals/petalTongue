// SPDX-License-Identifier: AGPL-3.0-or-later
//! Common test utilities for petalTongue TUI tests

use chrono::Utc;
use petal_tongue_core::{PrimalHealthStatus, PrimalInfo, TopologyEdge};

/// Create a test `PrimalInfo` with minimal required fields
#[expect(
    clippy::cast_sign_loss,
    reason = "Unix timestamp for current time is always non-negative"
)]
pub fn create_test_primal(name: &str, id: &str) -> PrimalInfo {
    PrimalInfo::new(
        id,
        name,
        "Test",
        format!("unix:///tmp/{name}.sock"),
        vec![],
        PrimalHealthStatus::Healthy,
        Utc::now().timestamp() as u64,
    )
}

/// Create a test `PrimalInfo` with capabilities
#[allow(dead_code)] // per-binary conditional: some test executables use this, others don't
#[expect(
    clippy::cast_sign_loss,
    reason = "Unix timestamp for current time is always non-negative"
)]
pub fn create_test_primal_with_caps(name: &str, id: &str, capabilities: Vec<String>) -> PrimalInfo {
    PrimalInfo::new(
        id,
        name,
        "Test",
        format!("unix:///tmp/{name}.sock"),
        capabilities,
        PrimalHealthStatus::Healthy,
        Utc::now().timestamp() as u64,
    )
}

/// Create a test `PrimalInfo` with specific health status
#[allow(dead_code)] // per-binary conditional: some test executables use this, others don't
#[expect(
    clippy::cast_sign_loss,
    reason = "Unix timestamp for current time is always non-negative"
)]
pub fn create_test_primal_with_health(
    name: &str,
    id: &str,
    health: PrimalHealthStatus,
) -> PrimalInfo {
    PrimalInfo::new(
        id,
        name,
        "Test",
        format!("unix:///tmp/{name}.sock"),
        vec![],
        health,
        Utc::now().timestamp() as u64,
    )
}

/// Create a test `TopologyEdge`
pub fn create_test_edge(from: &str, to: &str, edge_type: &str) -> TopologyEdge {
    TopologyEdge {
        from: from.into(),
        to: to.into(),
        edge_type: edge_type.to_string(),
        label: None,
        capability: None,
        metrics: None,
    }
}
