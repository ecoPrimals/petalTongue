// SPDX-License-Identifier: AGPL-3.0-only
//! Edge validation rules: connectivity and structural checks.

use crate::graph_builder::VisualGraph;
use std::collections::HashSet;

use super::types::{ValidationIssue, ValidationResult};

/// Validate all edges in the graph.
pub(super) fn validate_edges(graph: &VisualGraph, result: &mut ValidationResult) {
    // Build node ID set for quick lookups
    let node_ids: HashSet<_> = graph.nodes.iter().map(|n| &n.id).collect();

    for (idx, edge) in graph.edges.iter().enumerate() {
        // Check that source node exists
        if !node_ids.contains(&edge.from) {
            result.add_issue(ValidationIssue::error(format!(
                "Edge {} references non-existent source node: {}",
                idx, edge.from
            )));
        }

        // Check that target node exists
        if !node_ids.contains(&edge.to) {
            result.add_issue(ValidationIssue::error(format!(
                "Edge {} references non-existent target node: {}",
                idx, edge.to
            )));
        }

        // Check for self-loops
        if edge.from == edge.to {
            result.add_issue(
                ValidationIssue::warning(format!("Node '{}' has a self-loop edge", edge.from))
                    .with_suggestion(
                        "Remove self-loop edges - they may cause infinite loops".to_string(),
                    ),
            );
        }
    }
}
