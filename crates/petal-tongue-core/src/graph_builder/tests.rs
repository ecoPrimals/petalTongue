// SPDX-License-Identifier: AGPL-3.0-only
//! Graph builder tests

use super::types::{GraphEdge, GraphNode, NodeType, Vec2, VisualGraph};

#[test]
fn test_graph_creation() {
    let graph = VisualGraph::new("test-graph".to_string());
    assert_eq!(graph.name, "test-graph");
    assert!(graph.nodes.is_empty());
    assert!(graph.edges.is_empty());
}

#[test]
fn test_add_node() {
    let mut graph = VisualGraph::new("test".to_string());
    let node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
    let node_id = node.id.clone();
    graph.add_node(node);

    assert_eq!(graph.nodes.len(), 1);
    assert!(graph.get_node(&node_id).is_some());
}

#[test]
fn test_remove_node() {
    let mut graph = VisualGraph::new("test".to_string());
    let node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
    let node_id = node.id.clone();
    graph.add_node(node);

    graph.remove_node(&node_id);
    assert!(graph.nodes.is_empty());
}

#[test]
fn test_add_edge() {
    let mut graph = VisualGraph::new("test".to_string());
    let node1 = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
    let node2 = GraphNode::new(NodeType::Verification, Vec2::new(100.0, 0.0));
    let id1 = node1.id.clone();
    let id2 = node2.id.clone();

    graph.add_node(node1);
    graph.add_node(node2);

    let edge = GraphEdge::dependency(id1, id2);
    assert!(graph.add_edge(edge).is_ok());
    assert_eq!(graph.edges.len(), 1);
}

#[test]
fn test_add_edge_invalid_node() {
    let mut graph = VisualGraph::new("test".to_string());
    let node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
    let id = node.id.clone();
    graph.add_node(node);

    let edge = GraphEdge::dependency(id, "nonexistent".to_string());
    assert!(graph.add_edge(edge).is_err());
}

#[test]
fn test_cycle_detection() {
    let mut graph = VisualGraph::new("test".to_string());

    // Create three nodes
    let node1 = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
    let node2 = GraphNode::new(NodeType::Verification, Vec2::new(100.0, 0.0));
    let node3 = GraphNode::new(NodeType::WaitFor, Vec2::new(200.0, 0.0));
    let id1 = node1.id.clone();
    let id2 = node2.id.clone();
    let id3 = node3.id.clone();

    graph.add_node(node1);
    graph.add_node(node2);
    graph.add_node(node3);

    // Create a cycle: 1 -> 2 -> 3 -> 1
    graph
        .add_edge(GraphEdge::dependency(id1.clone(), id2.clone()))
        .unwrap();
    graph
        .add_edge(GraphEdge::dependency(id2, id3.clone()))
        .unwrap();
    graph.add_edge(GraphEdge::dependency(id3, id1)).unwrap();

    assert!(graph.has_cycle());
}

#[test]
fn test_no_cycle() {
    let mut graph = VisualGraph::new("test".to_string());

    let node1 = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
    let node2 = GraphNode::new(NodeType::Verification, Vec2::new(100.0, 0.0));
    let node3 = GraphNode::new(NodeType::WaitFor, Vec2::new(200.0, 0.0));
    let id1 = node1.id.clone();
    let id2 = node2.id.clone();
    let id3 = node3.id.clone();

    graph.add_node(node1);
    graph.add_node(node2);
    graph.add_node(node3);

    // Linear: 1 -> 2 -> 3
    graph
        .add_edge(GraphEdge::dependency(id1, id2.clone()))
        .unwrap();
    graph.add_edge(GraphEdge::dependency(id2, id3)).unwrap();

    assert!(!graph.has_cycle());
}

#[test]
fn test_entry_and_terminal_nodes() {
    let mut graph = VisualGraph::new("test".to_string());

    let node1 = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
    let node2 = GraphNode::new(NodeType::Verification, Vec2::new(100.0, 0.0));
    let node3 = GraphNode::new(NodeType::WaitFor, Vec2::new(200.0, 0.0));
    let id1 = node1.id.clone();
    let id2 = node2.id.clone();
    let id3 = node3.id.clone();

    graph.add_node(node1);
    graph.add_node(node2);
    graph.add_node(node3);

    // 1 -> 2 -> 3
    graph
        .add_edge(GraphEdge::dependency(id1.clone(), id2.clone()))
        .unwrap();
    graph
        .add_edge(GraphEdge::dependency(id2, id3.clone()))
        .unwrap();

    let entry = graph.get_entry_nodes();
    let terminal = graph.get_terminal_nodes();

    assert_eq!(entry.len(), 1);
    assert_eq!(entry[0].id, id1);

    assert_eq!(terminal.len(), 1);
    assert_eq!(terminal[0].id, id3);
}

#[test]
fn test_node_required_parameters() {
    let mut node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
    assert!(!node.has_all_required_parameters());

    node.set_parameter("primal_name".to_string(), "beardog".to_string());
    assert!(!node.has_all_required_parameters());

    node.set_parameter("family_id".to_string(), "nat0".to_string());
    assert!(node.has_all_required_parameters());
}

#[test]
fn test_vec2_snap() {
    let pos = Vec2::new(47.0, 53.0);
    let snapped = pos.snap(50.0);
    assert_eq!(snapped, Vec2::new(50.0, 50.0));

    let pos2 = Vec2::new(23.0, 27.0);
    let snapped2 = pos2.snap(50.0);
    assert_eq!(snapped2, Vec2::new(0.0, 50.0));
}
