// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use crate::graph_editor::DependencyType;
use crate::graph_editor::edge::GraphEdge;
use crate::graph_editor::node::GraphNode;

#[test]
fn test_create_empty_graph() {
    let graph = Graph::new("test-1".to_string(), "Test Graph".to_string());
    assert_eq!(graph.id, "test-1");
    assert_eq!(graph.name, "Test Graph");
    assert_eq!(graph.nodes.len(), 0);
    assert_eq!(graph.edges.len(), 0);
}

#[test]
fn test_add_node() {
    let mut graph = Graph::new("test-1".to_string(), "Test Graph".to_string());
    let node = GraphNode::new("node-1".to_string(), "test-type".to_string());

    assert!(graph.add_node(node).is_ok());
    assert_eq!(graph.nodes.len(), 1);
}

#[test]
fn test_add_duplicate_node() {
    let mut graph = Graph::new("test-1".to_string(), "Test Graph".to_string());
    let node1 = GraphNode::new("node-1".to_string(), "test-type".to_string());
    let node2 = GraphNode::new("node-1".to_string(), "test-type".to_string());

    assert!(graph.add_node(node1).is_ok());
    assert!(graph.add_node(node2).is_err());
}

#[test]
fn test_remove_node() {
    let mut graph = Graph::new("test-1".to_string(), "Test Graph".to_string());
    let node = GraphNode::new("node-1".to_string(), "test-type".to_string());

    graph.add_node(node).unwrap();
    assert_eq!(graph.nodes.len(), 1);

    let affected = graph.remove_node("node-1").unwrap();
    assert_eq!(graph.nodes.len(), 0);
    assert_eq!(affected.len(), 0);
}

#[test]
fn test_add_edge() {
    let mut graph = Graph::new("test-1".to_string(), "Test Graph".to_string());
    let node1 = GraphNode::new("node-1".to_string(), "test-type".to_string());
    let node2 = GraphNode::new("node-2".to_string(), "test-type".to_string());

    graph.add_node(node1).unwrap();
    graph.add_node(node2).unwrap();

    let edge = GraphEdge::new(
        "edge-1".to_string(),
        "node-1".to_string(),
        "node-2".to_string(),
        DependencyType::Sequential,
    );

    assert!(graph.add_edge(edge).is_ok());
    assert_eq!(graph.edges.len(), 1);
}

#[test]
fn test_cycle_detection() {
    let mut graph = Graph::new("test-1".to_string(), "Test Graph".to_string());
    let node1 = GraphNode::new("node-1".to_string(), "test-type".to_string());
    let node2 = GraphNode::new("node-2".to_string(), "test-type".to_string());
    let node3 = GraphNode::new("node-3".to_string(), "test-type".to_string());

    graph.add_node(node1).unwrap();
    graph.add_node(node2).unwrap();
    graph.add_node(node3).unwrap();

    // Add edges: 1 → 2 → 3
    let edge1 = GraphEdge::new(
        "edge-1".to_string(),
        "node-1".to_string(),
        "node-2".to_string(),
        DependencyType::Sequential,
    );
    let edge2 = GraphEdge::new(
        "edge-2".to_string(),
        "node-2".to_string(),
        "node-3".to_string(),
        DependencyType::Sequential,
    );

    graph.add_edge(edge1).unwrap();
    graph.add_edge(edge2).unwrap();

    // Try to add edge 3 → 1 (would create cycle)
    let edge3 = GraphEdge::new(
        "edge-3".to_string(),
        "node-3".to_string(),
        "node-1".to_string(),
        DependencyType::Sequential,
    );

    assert!(graph.add_edge(edge3).is_err());
}

#[test]
fn test_topological_sort() {
    let mut graph = Graph::new("test-1".to_string(), "Test Graph".to_string());
    let node1 = GraphNode::new("node-1".to_string(), "test-type".to_string());
    let node2 = GraphNode::new("node-2".to_string(), "test-type".to_string());
    let node3 = GraphNode::new("node-3".to_string(), "test-type".to_string());

    graph.add_node(node1).unwrap();
    graph.add_node(node2).unwrap();
    graph.add_node(node3).unwrap();

    // Add edges: 1 → 2 → 3
    let edge1 = GraphEdge::new(
        "edge-1".to_string(),
        "node-1".to_string(),
        "node-2".to_string(),
        DependencyType::Sequential,
    );
    let edge2 = GraphEdge::new(
        "edge-2".to_string(),
        "node-2".to_string(),
        "node-3".to_string(),
        DependencyType::Sequential,
    );

    graph.add_edge(edge1).unwrap();
    graph.add_edge(edge2).unwrap();

    let sorted = graph.topological_sort().unwrap();
    assert_eq!(sorted.len(), 3);

    // node-1 should come before node-2, node-2 before node-3
    let pos1 = sorted.iter().position(|id| id == "node-1").unwrap();
    let pos2 = sorted.iter().position(|id| id == "node-2").unwrap();
    let pos3 = sorted.iter().position(|id| id == "node-3").unwrap();

    assert!(pos1 < pos2);
    assert!(pos2 < pos3);
}

#[test]
fn test_graph_stats() {
    let mut graph = Graph::new("test-1".to_string(), "Test Graph".to_string());
    let node1 = GraphNode::new("node-1".to_string(), "test-type".to_string());
    let node2 = GraphNode::new("node-2".to_string(), "test-type".to_string());

    graph.add_node(node1).unwrap();
    graph.add_node(node2).unwrap();

    let edge = GraphEdge::new(
        "edge-1".to_string(),
        "node-1".to_string(),
        "node-2".to_string(),
        DependencyType::Sequential,
    );
    graph.add_edge(edge).unwrap();

    let stats = graph.stats();
    assert_eq!(stats.node_count, 2);
    assert_eq!(stats.edge_count, 1);
    assert!(!stats.has_cycles);
}

#[test]
fn test_remove_node_with_edges() {
    let mut graph = Graph::new("test-1".to_string(), "Test Graph".to_string());
    let node1 = GraphNode::new("node-1".to_string(), "test-type".to_string());
    let node2 = GraphNode::new("node-2".to_string(), "test-type".to_string());
    graph.add_node(node1).unwrap();
    graph.add_node(node2).unwrap();
    graph
        .add_edge(GraphEdge::new(
            "edge-1".to_string(),
            "node-1".to_string(),
            "node-2".to_string(),
            DependencyType::Sequential,
        ))
        .unwrap();

    let affected = graph.remove_node("node-1").unwrap();
    assert_eq!(graph.nodes.len(), 1);
    assert_eq!(affected.len(), 1);
    assert_eq!(affected[0], "edge-1");
}

#[test]
fn test_remove_node_not_found() {
    let mut graph = Graph::new("test-1".to_string(), "Test Graph".to_string());
    assert!(graph.remove_node("nonexistent").is_err());
}

#[test]
fn test_modify_node_not_found() {
    let mut graph = Graph::new("test-1".to_string(), "Test Graph".to_string());
    let node = GraphNode::new("node-1".to_string(), "test-type".to_string());
    let updated = GraphNode::new("node-1".to_string(), "test-type".to_string());
    assert!(graph.modify_node("nonexistent", updated).is_err());
    graph.add_node(node).unwrap();
    let updated2 = GraphNode::new("node-1".to_string(), "test-type".to_string());
    assert!(graph.modify_node("nonexistent", updated2).is_err());
}

#[test]
fn test_modify_node_id_change() {
    let mut graph = Graph::new("test-1".to_string(), "Test Graph".to_string());
    let node = GraphNode::new("node-1".to_string(), "test-type".to_string());
    graph.add_node(node).unwrap();
    let updated = GraphNode::new("node-2".to_string(), "test-type".to_string());
    assert!(graph.modify_node("node-1", updated).is_err());
}

#[test]
fn test_add_edge_source_not_found() {
    let mut graph = Graph::new("test-1".to_string(), "Test Graph".to_string());
    let node = GraphNode::new("node-2".to_string(), "test-type".to_string());
    graph.add_node(node).unwrap();
    let edge = GraphEdge::new(
        "edge-1".to_string(),
        "node-1".to_string(),
        "node-2".to_string(),
        DependencyType::Sequential,
    );
    assert!(graph.add_edge(edge).is_err());
}

#[test]
fn test_add_edge_target_not_found() {
    let mut graph = Graph::new("test-1".to_string(), "Test Graph".to_string());
    let node = GraphNode::new("node-1".to_string(), "test-type".to_string());
    graph.add_node(node).unwrap();
    let edge = GraphEdge::new(
        "edge-1".to_string(),
        "node-1".to_string(),
        "node-2".to_string(),
        DependencyType::Sequential,
    );
    assert!(graph.add_edge(edge).is_err());
}

#[test]
fn test_add_edge_duplicate() {
    let mut graph = Graph::new("test-1".to_string(), "Test Graph".to_string());
    let node1 = GraphNode::new("node-1".to_string(), "test-type".to_string());
    let node2 = GraphNode::new("node-2".to_string(), "test-type".to_string());
    graph.add_node(node1).unwrap();
    graph.add_node(node2).unwrap();
    let edge = GraphEdge::new(
        "edge-1".to_string(),
        "node-1".to_string(),
        "node-2".to_string(),
        DependencyType::Sequential,
    );
    graph.add_edge(edge).unwrap();
    let edge2 = GraphEdge::new(
        "edge-2".to_string(),
        "node-1".to_string(),
        "node-2".to_string(),
        DependencyType::Sequential,
    );
    assert!(graph.add_edge(edge2).is_err());
}

#[test]
fn test_remove_edge_not_found() {
    let mut graph = Graph::new("test-1".to_string(), "Test Graph".to_string());
    assert!(graph.remove_edge("nonexistent").is_err());
}

#[test]
fn test_get_incoming_outgoing_edges() {
    let mut graph = Graph::new("test-1".to_string(), "Test Graph".to_string());
    let node1 = GraphNode::new("node-1".to_string(), "test-type".to_string());
    let node2 = GraphNode::new("node-2".to_string(), "test-type".to_string());
    let node3 = GraphNode::new("node-3".to_string(), "test-type".to_string());
    graph.add_node(node1).unwrap();
    graph.add_node(node2).unwrap();
    graph.add_node(node3).unwrap();
    graph
        .add_edge(GraphEdge::new(
            "e1".to_string(),
            "node-1".to_string(),
            "node-2".to_string(),
            DependencyType::Sequential,
        ))
        .unwrap();
    graph
        .add_edge(GraphEdge::new(
            "e2".to_string(),
            "node-3".to_string(),
            "node-2".to_string(),
            DependencyType::Sequential,
        ))
        .unwrap();

    let incoming = graph.get_incoming_edges("node-2");
    assert_eq!(incoming.len(), 2);
    let outgoing = graph.get_outgoing_edges("node-1");
    assert_eq!(outgoing.len(), 1);
}

#[test]
fn test_topological_sort_cycle_error() {
    let mut graph = Graph::new("test-1".to_string(), "Test Graph".to_string());
    let node1 = GraphNode::new("node-1".to_string(), "test-type".to_string());
    let node2 = GraphNode::new("node-2".to_string(), "test-type".to_string());
    graph.add_node(node1).unwrap();
    graph.add_node(node2).unwrap();
    graph
        .add_edge(GraphEdge::new(
            "e1".to_string(),
            "node-1".to_string(),
            "node-2".to_string(),
            DependencyType::Sequential,
        ))
        .unwrap();
    graph.edges.push(GraphEdge::new(
        "e2".to_string(),
        "node-2".to_string(),
        "node-1".to_string(),
        DependencyType::Sequential,
    ));

    assert!(graph.topological_sort().is_err());
}

#[test]
fn test_graph_stats_with_cycles() {
    let mut graph = Graph::new("test-1".to_string(), "Test Graph".to_string());
    let node1 = GraphNode::new("node-1".to_string(), "test-type".to_string());
    let node2 = GraphNode::new("node-2".to_string(), "test-type".to_string());
    graph.add_node(node1).unwrap();
    graph.add_node(node2).unwrap();
    graph
        .add_edge(GraphEdge::new(
            "e1".to_string(),
            "node-1".to_string(),
            "node-2".to_string(),
            DependencyType::Sequential,
        ))
        .unwrap();
    graph.edges.push(GraphEdge::new(
        "e2".to_string(),
        "node-2".to_string(),
        "node-1".to_string(),
        DependencyType::Sequential,
    ));

    let stats = graph.stats();
    assert!(stats.has_cycles);
}
