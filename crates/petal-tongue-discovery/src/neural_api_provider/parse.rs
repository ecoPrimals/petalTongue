// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON value parsing for Neural API responses.

use crate::capability_parse;
use crate::errors::{DiscoveryError, DiscoveryResult};
use petal_tongue_core::{PrimalHealthStatus, PrimalInfo, TopologyEdge};
use serde_json::Value;

/// Parse a single primal object from Neural API `primal.list` results into [`PrimalInfo`].
#[expect(
    clippy::unnecessary_wraps,
    reason = "Ok wrapper for struct literal in Result chain"
)]
pub(super) fn parse_primal(primal: &Value) -> DiscoveryResult<PrimalInfo> {
    Ok(PrimalInfo {
        id: primal["id"].as_str().unwrap_or("unknown").to_owned().into(),
        name: primal["primal_type"]
            .as_str()
            .unwrap_or("unknown")
            .to_owned(),
        primal_type: primal["primal_type"]
            .as_str()
            .unwrap_or("unknown")
            .to_owned(),
        endpoint: primal["socket_path"].as_str().unwrap_or("").to_owned(),
        capabilities: primal["capabilities"]
            .as_array()
            .map(|v| capability_parse::parse_capabilities(v))
            .unwrap_or_default(),
        health: match primal["health"].as_str() {
            Some("healthy") => PrimalHealthStatus::Healthy,
            _ => PrimalHealthStatus::Unknown,
        },
        last_seen: 0, // Neural API doesn't provide this yet
        endpoints: None,
        metadata: None,
        properties: std::collections::HashMap::default(),
    })
}

/// Parse `neural_api.get_topology` result into [`TopologyEdge`] list.
pub(super) fn parse_topology_edges(result: &Value) -> DiscoveryResult<Vec<TopologyEdge>> {
    let connections =
        result["connections"]
            .as_array()
            .ok_or_else(|| DiscoveryError::ExpectedArray {
                context: " of connections".to_owned(),
            })?;

    let mut edges = Vec::new();
    for conn in connections {
        edges.push(TopologyEdge {
            from: conn["from"].as_str().unwrap_or("").to_owned().into(),
            to: conn["to"].as_str().unwrap_or("").to_owned().into(),
            edge_type: conn["connection_type"]
                .as_str()
                .unwrap_or("unknown")
                .to_owned(),
            capability: None,
            label: None,
            metrics: None,
        });
    }

    Ok(edges)
}
