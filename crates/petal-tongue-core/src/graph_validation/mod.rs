// SPDX-License-Identifier: AGPL-3.0-only
//! Graph Validation - Ensure graph correctness before execution
//!
//! TRUE PRIMAL: Zero hardcoding, capability-based validation.

mod edge_rules;
mod node_rules;
mod structure;
mod types;

#[cfg(test)]
mod tests;

pub use types::{ValidationIssue, ValidationResult, ValidationSeverity};

use crate::graph_builder::VisualGraph;

use edge_rules::validate_edges;
use node_rules::validate_nodes;
use structure::{check_cycles, check_unreachable_nodes, validate_execution_order};

/// Graph validator
pub struct GraphValidator;

impl GraphValidator {
    /// Validate a graph
    #[must_use]
    pub fn validate(graph: &VisualGraph) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Check for empty graph
        if graph.nodes.is_empty() {
            result.add_issue(ValidationIssue::warning(
                "Graph is empty - add nodes to execute".to_string(),
            ));
            return result;
        }

        // Validate nodes
        validate_nodes(graph, &mut result);

        // Validate edges
        validate_edges(graph, &mut result);

        // Check for cycles (DAG requirement)
        check_cycles(graph, &mut result);

        // Check for unreachable nodes
        check_unreachable_nodes(graph, &mut result);

        // Validate execution order
        validate_execution_order(graph, &mut result);

        result
    }

    /// Get execution order (topological sort)
    /// Returns None if graph has cycles
    #[must_use]
    pub fn get_execution_order(graph: &VisualGraph) -> Option<Vec<String>> {
        structure::get_execution_order(graph)
    }
}
