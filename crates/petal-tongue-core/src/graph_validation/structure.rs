// SPDX-License-Identifier: AGPL-3.0-only
//! Graph structure validation: cycles, reachability, and topological sort.

use crate::graph_builder::VisualGraph;
use std::collections::{HashMap, HashSet, VecDeque};

use super::types::{ValidationIssue, ValidationResult};

/// Check for cycles using DFS.
pub(super) fn check_cycles(graph: &VisualGraph, result: &mut ValidationResult) {
    // Build adjacency list
    let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();
    for node in &graph.nodes {
        adj_list.insert(node.id.clone(), Vec::new());
    }
    for edge in &graph.edges {
        adj_list
            .entry(edge.from.clone())
            .or_default()
            .push(edge.to.clone());
    }

    // DFS cycle detection
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();

    for node in &graph.nodes {
        let node_id = &node.id;
        if !visited.contains(node_id)
            && dfs_has_cycle(node_id, &adj_list, &mut visited, &mut rec_stack)
        {
            result.add_issue(
                ValidationIssue::error(format!(
                    "Graph contains a cycle involving node '{node_id}'"
                ))
                .with_suggestion(
                    "Remove edges to break the cycle - graphs must be acyclic (DAG)".to_string(),
                ),
            );
            return; // Report first cycle only
        }
    }
}

/// DFS helper for cycle detection.
fn dfs_has_cycle(
    node: &str,
    adj_list: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    rec_stack: &mut HashSet<String>,
) -> bool {
    visited.insert(node.to_string());
    rec_stack.insert(node.to_string());

    if let Some(neighbors) = adj_list.get(node) {
        for neighbor in neighbors {
            if !visited.contains(neighbor) {
                if dfs_has_cycle(neighbor, adj_list, visited, rec_stack) {
                    return true;
                }
            } else if rec_stack.contains(neighbor) {
                return true; // Cycle detected
            }
        }
    }

    rec_stack.remove(node);
    false
}

/// Check for unreachable nodes.
pub(super) fn check_unreachable_nodes(graph: &VisualGraph, result: &mut ValidationResult) {
    if graph.nodes.is_empty() {
        return;
    }

    // Find all nodes with no incoming edges (potential start nodes)
    let mut has_incoming = HashSet::new();
    for edge in &graph.edges {
        has_incoming.insert(edge.to.clone());
    }

    let start_nodes: Vec<_> = graph
        .nodes
        .iter()
        .map(|n| &n.id)
        .filter(|id| !has_incoming.contains(*id))
        .cloned()
        .collect();

    if start_nodes.is_empty() {
        result.add_issue(
            ValidationIssue::warning(
                "No start nodes found - all nodes have incoming edges".to_string(),
            )
            .with_suggestion(
                "Add a PrimalStart node or ensure at least one node has no dependencies"
                    .to_string(),
            ),
        );
        return;
    }

    // BFS to find all reachable nodes
    let mut reachable = HashSet::new();
    let mut queue = VecDeque::new();

    for start in &start_nodes {
        queue.push_back(start.clone());
        reachable.insert(start.clone());
    }

    // Build adjacency list
    let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();
    for edge in &graph.edges {
        adj_list
            .entry(edge.from.clone())
            .or_default()
            .push(edge.to.clone());
    }

    while let Some(node) = queue.pop_front() {
        if let Some(neighbors) = adj_list.get(&node) {
            for neighbor in neighbors {
                if !reachable.contains(neighbor) {
                    reachable.insert(neighbor.clone());
                    queue.push_back(neighbor.clone());
                }
            }
        }
    }

    // Report unreachable nodes
    for node in &graph.nodes {
        if !reachable.contains(&node.id) {
            result.add_issue(
                ValidationIssue::node_warning(
                    node.id.clone(),
                    format!("Node '{}' is unreachable", node.id),
                )
                .with_suggestion("Connect this node to the graph or remove it".to_string()),
            );
        }
    }
}

/// Validate execution order (topological sort). Used internally during validation.
pub(super) fn validate_execution_order(graph: &VisualGraph, _result: &mut ValidationResult) {
    // Build in-degree map
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    for node in &graph.nodes {
        in_degree.insert(node.id.clone(), 0);
    }
    for edge in &graph.edges {
        *in_degree.entry(edge.to.clone()).or_insert(0) += 1;
    }

    // Build adjacency list
    let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();
    for edge in &graph.edges {
        adj_list
            .entry(edge.from.clone())
            .or_default()
            .push(edge.to.clone());
    }

    // Kahn's algorithm for topological sort
    let mut queue = VecDeque::new();
    for (node_id, &degree) in &in_degree {
        if degree == 0 {
            queue.push_back(node_id.clone());
        }
    }

    let mut _sorted_count = 0;
    while let Some(node) = queue.pop_front() {
        _sorted_count += 1;

        if let Some(neighbors) = adj_list.get(&node) {
            for neighbor in neighbors {
                if let Some(degree) = in_degree.get_mut(neighbor) {
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }
    }

    // If we couldn't sort all nodes, there's a cycle (already caught above)
}

/// Compute execution order (topological sort). Returns `None` if graph has cycles.
#[must_use]
pub(super) fn get_execution_order(graph: &VisualGraph) -> Option<Vec<String>> {
    // Build in-degree map
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    for node in &graph.nodes {
        in_degree.insert(node.id.clone(), 0);
    }
    for edge in &graph.edges {
        *in_degree.entry(edge.to.clone()).or_insert(0) += 1;
    }

    // Build adjacency list
    let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();
    for edge in &graph.edges {
        adj_list
            .entry(edge.from.clone())
            .or_default()
            .push(edge.to.clone());
    }

    // Kahn's algorithm
    let mut queue = VecDeque::new();
    for (node_id, &degree) in &in_degree {
        if degree == 0 {
            queue.push_back(node_id.clone());
        }
    }

    let mut order = Vec::new();
    while let Some(node) = queue.pop_front() {
        order.push(node.clone());

        if let Some(neighbors) = adj_list.get(&node) {
            for neighbor in neighbors {
                if let Some(degree) = in_degree.get_mut(neighbor) {
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(neighbor.clone());
                    }
                }
            }
        }
    }

    if order.len() == graph.nodes.len() {
        Some(order)
    } else {
        None // Cycle detected
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_builder::{GraphEdge, GraphNode, NodeType, Vec2, VisualGraph};

    #[test]
    fn test_get_execution_order_linear() {
        let mut graph = VisualGraph::new("g".to_string());
        let mut n1 = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        n1.set_parameter("primal_name".to_string(), "a".to_string());
        n1.set_parameter("family_id".to_string(), "f1".to_string());
        let id1 = n1.id.clone();
        let mut n2 = GraphNode::new(NodeType::Verification, Vec2::zero());
        n2.set_parameter("primal_name".to_string(), "b".to_string());
        n2.set_parameter("timeout".to_string(), "30".to_string());
        let id2 = n2.id.clone();
        graph.add_node(n1);
        graph.add_node(n2);
        graph
            .add_edge(GraphEdge::dependency(id1.clone(), id2.clone()))
            .unwrap();
        let order = get_execution_order(&graph).unwrap();
        assert_eq!(order.len(), 2);
        assert_eq!(order[0], id1);
        assert_eq!(order[1], id2);
    }

    #[test]
    fn test_get_execution_order_cycle_returns_none() {
        let mut graph = VisualGraph::new("g".to_string());
        let mut n1 = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        n1.set_parameter("primal_name".to_string(), "a".to_string());
        n1.set_parameter("family_id".to_string(), "f1".to_string());
        let id1 = n1.id.clone();
        let mut n2 = GraphNode::new(NodeType::Verification, Vec2::zero());
        n2.set_parameter("primal_name".to_string(), "b".to_string());
        n2.set_parameter("timeout".to_string(), "30".to_string());
        let id2 = n2.id.clone();
        graph.add_node(n1);
        graph.add_node(n2);
        graph
            .add_edge(GraphEdge::dependency(id1.clone(), id2.clone()))
            .unwrap();
        graph.add_edge(GraphEdge::dependency(id2, id1)).unwrap();
        let order = get_execution_order(&graph);
        assert!(order.is_none());
    }

    #[test]
    fn test_check_cycles_detects_cycle() {
        let mut graph = VisualGraph::new("g".to_string());
        let mut n1 = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        n1.set_parameter("primal_name".to_string(), "a".to_string());
        n1.set_parameter("family_id".to_string(), "f1".to_string());
        let id1 = n1.id.clone();
        let mut n2 = GraphNode::new(NodeType::Verification, Vec2::zero());
        n2.set_parameter("primal_name".to_string(), "b".to_string());
        n2.set_parameter("timeout".to_string(), "30".to_string());
        let id2 = n2.id.clone();
        graph.add_node(n1);
        graph.add_node(n2);
        graph
            .add_edge(GraphEdge::dependency(id1.clone(), id2.clone()))
            .unwrap();
        graph.add_edge(GraphEdge::dependency(id2, id1)).unwrap();
        let mut result = crate::graph_validation::types::ValidationResult::new();
        check_cycles(&graph, &mut result);
        assert!(result.has_errors());
        assert!(result.errors().iter().any(|e| e.message.contains("cycle")));
    }

    #[test]
    fn test_check_unreachable_nodes() {
        let mut graph = VisualGraph::new("g".to_string());
        let mut n1 = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        n1.set_parameter("primal_name".to_string(), "a".to_string());
        n1.set_parameter("family_id".to_string(), "f1".to_string());
        let id1 = n1.id.clone();
        let mut n2 = GraphNode::new(NodeType::Verification, Vec2::zero());
        n2.set_parameter("primal_name".to_string(), "b".to_string());
        n2.set_parameter("timeout".to_string(), "30".to_string());
        let id2 = n2.id.clone();
        let mut n3 = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        n3.set_parameter("primal_name".to_string(), "orphan".to_string());
        n3.set_parameter("family_id".to_string(), "f1".to_string());
        let id3 = n3.id.clone();
        graph.add_node(n1);
        graph.add_node(n2);
        graph.add_node(n3);
        graph
            .add_edge(GraphEdge::dependency(id1, id2))
            .expect("add edge");
        graph.edges.push(GraphEdge::dependency(id3.clone(), id3));
        let mut result = crate::graph_validation::types::ValidationResult::new();
        check_unreachable_nodes(&graph, &mut result);
        assert!(result.has_warnings());
    }

    #[test]
    fn test_check_unreachable_nodes_empty_graph_returns_early() {
        let graph = VisualGraph::new("g".to_string());
        let mut result = crate::graph_validation::types::ValidationResult::new();
        check_unreachable_nodes(&graph, &mut result);
        assert!(!result.has_errors());
        assert!(!result.has_warnings());
    }

    #[test]
    fn test_check_unreachable_nodes_all_have_incoming() {
        let mut graph = VisualGraph::new("g".to_string());
        let mut n1 = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        n1.set_parameter("primal_name".to_string(), "a".to_string());
        n1.set_parameter("family_id".to_string(), "f1".to_string());
        let id1 = n1.id.clone();
        let mut n2 = GraphNode::new(NodeType::Verification, Vec2::zero());
        n2.set_parameter("primal_name".to_string(), "b".to_string());
        n2.set_parameter("timeout".to_string(), "30".to_string());
        let id2 = n2.id.clone();
        graph.add_node(n1);
        graph.add_node(n2);
        graph
            .add_edge(GraphEdge::dependency(id1.clone(), id2.clone()))
            .expect("add edge");
        graph
            .add_edge(GraphEdge::dependency(id2, id1))
            .expect("add edge - cycle");
        let mut result = crate::graph_validation::types::ValidationResult::new();
        check_unreachable_nodes(&graph, &mut result);
        assert!(result.has_warnings(), "cycle means no start nodes");
    }

    #[test]
    fn test_get_execution_order_single_node() {
        let mut graph = VisualGraph::new("g".to_string());
        let mut n = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        n.set_parameter("primal_name".to_string(), "x".to_string());
        n.set_parameter("family_id".to_string(), "f1".to_string());
        let id = n.id.clone();
        graph.add_node(n);
        let order = get_execution_order(&graph).expect("single node has no cycle");
        assert_eq!(order.len(), 1);
        assert_eq!(order[0], id);
    }

    #[test]
    fn test_get_execution_order_empty_graph() {
        let graph = VisualGraph::new("g".to_string());
        let order = get_execution_order(&graph);
        assert_eq!(order, Some(vec![]));
    }

    #[test]
    fn test_get_execution_order_diamond() {
        let mut graph = VisualGraph::new("g".to_string());
        let mut n1 = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        n1.set_parameter("primal_name".to_string(), "a".to_string());
        n1.set_parameter("family_id".to_string(), "f1".to_string());
        let id1 = n1.id.clone();
        let mut n2 = GraphNode::new(NodeType::Verification, Vec2::zero());
        n2.set_parameter("primal_name".to_string(), "b".to_string());
        n2.set_parameter("timeout".to_string(), "30".to_string());
        let id2 = n2.id.clone();
        let mut n3 = GraphNode::new(NodeType::Verification, Vec2::zero());
        n3.set_parameter("primal_name".to_string(), "c".to_string());
        n3.set_parameter("timeout".to_string(), "30".to_string());
        let id3 = n3.id.clone();
        let mut n4 = GraphNode::new(NodeType::Verification, Vec2::zero());
        n4.set_parameter("primal_name".to_string(), "d".to_string());
        n4.set_parameter("timeout".to_string(), "30".to_string());
        let id4 = n4.id.clone();
        graph.add_node(n1);
        graph.add_node(n2);
        graph.add_node(n3);
        graph.add_node(n4);
        graph
            .add_edge(GraphEdge::dependency(id1.clone(), id2.clone()))
            .expect("edge");
        graph
            .add_edge(GraphEdge::dependency(id1.clone(), id3.clone()))
            .expect("edge");
        graph
            .add_edge(GraphEdge::dependency(id2, id4.clone()))
            .expect("edge");
        graph
            .add_edge(GraphEdge::dependency(id3, id4))
            .expect("edge");
        let order = get_execution_order(&graph).expect("diamond is acyclic");
        assert_eq!(order.len(), 4);
        assert_eq!(order[0], id1);
    }

    #[test]
    fn test_check_cycles_no_cycle() {
        let mut graph = VisualGraph::new("g".to_string());
        let mut n1 = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        n1.set_parameter("primal_name".to_string(), "a".to_string());
        n1.set_parameter("family_id".to_string(), "f1".to_string());
        let id1 = n1.id.clone();
        let mut n2 = GraphNode::new(NodeType::Verification, Vec2::zero());
        n2.set_parameter("primal_name".to_string(), "b".to_string());
        n2.set_parameter("timeout".to_string(), "30".to_string());
        let id2 = n2.id.clone();
        graph.add_node(n1);
        graph.add_node(n2);
        graph
            .add_edge(GraphEdge::dependency(id1, id2))
            .expect("edge");
        let mut result = crate::graph_validation::types::ValidationResult::new();
        check_cycles(&graph, &mut result);
        assert!(!result.has_errors());
    }

    #[test]
    fn test_validate_execution_order_invoked() {
        let mut graph = VisualGraph::new("g".to_string());
        let mut n = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        n.set_parameter("primal_name".to_string(), "x".to_string());
        n.set_parameter("family_id".to_string(), "f1".to_string());
        graph.add_node(n);
        let mut result = crate::graph_validation::types::ValidationResult::new();
        validate_execution_order(&graph, &mut result);
        assert!(result.is_valid());
    }
}
