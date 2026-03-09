// SPDX-License-Identifier: AGPL-3.0-only
//! Node validation rules: required parameters and type-specific checks.

use crate::graph_builder::{NodeType, VisualGraph};

use super::types::{ValidationIssue, ValidationResult};

/// Validate all nodes in the graph.
pub(super) fn validate_nodes(graph: &VisualGraph, result: &mut ValidationResult) {
    for node in &graph.nodes {
        let node_id = &node.id;
        // Check for missing required parameters
        if !node.has_all_required_parameters() {
            let missing = node.missing_parameters();
            result.add_issue(
                ValidationIssue::node_error(
                    node_id.clone(),
                    format!(
                        "Node '{}' missing required parameters: {}",
                        node_id,
                        missing.join(", ")
                    ),
                )
                .with_suggestion(
                    "Fill in all required parameters in the Property Panel".to_string(),
                ),
            );
        }

        // Type-specific validation
        match node.node_type {
            NodeType::PrimalStart => {
                // Should have primal_name and family_id
                if node.get_parameter("primal_name").is_none() {
                    result.add_issue(ValidationIssue::node_error(
                        node_id.clone(),
                        "PrimalStart node requires 'primal_name' parameter".to_string(),
                    ));
                }
            }
            NodeType::Verification => {
                // Should have primal_name
                if node.get_parameter("primal_name").is_none() {
                    result.add_issue(ValidationIssue::node_error(
                        node_id.clone(),
                        "Verification node requires 'primal_name' parameter".to_string(),
                    ));
                }
            }
            NodeType::WaitFor => {
                // Should have condition and timeout
                if node.get_parameter("condition").is_none() {
                    result.add_issue(ValidationIssue::node_error(
                        node_id.clone(),
                        "WaitFor node requires 'condition' parameter".to_string(),
                    ));
                }
            }
            NodeType::Conditional => {
                // Should have condition
                if node.get_parameter("condition").is_none() {
                    result.add_issue(ValidationIssue::node_error(
                        node_id.clone(),
                        "Conditional node requires 'condition' parameter".to_string(),
                    ));
                }
            }
        }
    }
}
