// SPDX-License-Identifier: AGPL-3.0-only
//! Graph engine tests

use super::*;
use crate::PrimalId;
use crate::test_fixtures::primals;
use crate::types::TopologyEdge;

fn create_test_primal(id: &str, name: &str) -> crate::types::PrimalInfo {
    let mut primal = primals::test_primal(id);
    primal.name = name.to_string();
    primal
}

#[test]
fn test_graph_creation() {
    let graph = GraphEngine::new();
    assert_eq!(graph.nodes().len(), 0);
    assert_eq!(graph.edges().len(), 0);
}

#[test]
fn test_add_nodes() {
    let mut graph = GraphEngine::new();

    graph.add_node(create_test_primal("1", "Node 1"));
    graph.add_node(create_test_primal("2", "Node 2"));

    assert_eq!(graph.nodes().len(), 2);
    assert!(graph.get_node("1").is_some());
    assert!(graph.get_node("2").is_some());
}

#[test]
fn test_add_edges() {
    let mut graph = GraphEngine::new();

    graph.add_node(create_test_primal("1", "Node 1"));
    graph.add_node(create_test_primal("2", "Node 2"));

    graph.add_edge(TopologyEdge {
        from: PrimalId::from("1"),
        to: PrimalId::from("2"),
        edge_type: "test".to_string(),
        label: None,
        capability: None,
        metrics: None,
    });

    assert_eq!(graph.edges().len(), 1);
}

#[test]
fn test_remove_node() {
    let mut graph = GraphEngine::new();

    graph.add_node(create_test_primal("1", "Node 1"));
    graph.add_node(create_test_primal("2", "Node 2"));
    graph.add_edge(TopologyEdge {
        from: PrimalId::from("1"),
        to: PrimalId::from("2"),
        edge_type: "test".to_string(),
        label: None,
        capability: None,
        metrics: None,
    });

    assert!(graph.remove_node("1"));
    assert_eq!(graph.nodes().len(), 1);
    assert_eq!(graph.edges().len(), 0); // Edge should be removed too
}

#[test]
fn test_neighbors() {
    let mut graph = GraphEngine::new();

    graph.add_node(create_test_primal("1", "Node 1"));
    graph.add_node(create_test_primal("2", "Node 2"));
    graph.add_node(create_test_primal("3", "Node 3"));

    graph.add_edge(TopologyEdge {
        from: PrimalId::from("1"),
        to: PrimalId::from("2"),
        edge_type: "test".to_string(),
        label: None,
        capability: None,
        metrics: None,
    });
    graph.add_edge(TopologyEdge {
        from: PrimalId::from("1"),
        to: PrimalId::from("3"),
        edge_type: "test".to_string(),
        label: None,
        capability: None,
        metrics: None,
    });

    let neighbors = graph.neighbors("1");
    assert_eq!(neighbors.len(), 2);
}

#[test]
fn test_force_directed_layout() {
    let mut graph = GraphEngine::new();

    graph.add_node(create_test_primal("1", "Node 1"));
    graph.add_node(create_test_primal("2", "Node 2"));
    graph.add_node(create_test_primal("3", "Node 3"));
    graph.add_edge(TopologyEdge {
        from: PrimalId::from("1"),
        to: PrimalId::from("2"),
        edge_type: "test".to_string(),
        label: None,
        capability: None,
        metrics: None,
    });
    graph.add_edge(TopologyEdge {
        from: PrimalId::from("2"),
        to: PrimalId::from("3"),
        edge_type: "test".to_string(),
        label: None,
        capability: None,
        metrics: None,
    });

    graph.set_layout(LayoutAlgorithm::ForceDirected);
    graph.layout(50); // More iterations

    // Nodes should have moved from initial (0, 0) positions
    // With 3 nodes and repulsive forces, they should spread out
    let node1 = graph.get_node("1").unwrap();
    let node2 = graph.get_node("2").unwrap();
    let node3 = graph.get_node("3").unwrap();

    // At least one node should have moved significantly
    let total_movement = node1.position.x.abs()
        + node1.position.y.abs()
        + node2.position.x.abs()
        + node2.position.y.abs()
        + node3.position.x.abs()
        + node3.position.y.abs();

    assert!(
        total_movement > 10.0,
        "Nodes should have spread out, total movement: {total_movement}"
    );
}

#[test]
fn test_circular_layout() {
    let mut graph = GraphEngine::new();

    for i in 0..5 {
        graph.add_node(create_test_primal(&i.to_string(), &format!("Node {i}")));
    }

    graph.set_layout(LayoutAlgorithm::Circular);
    graph.layout(1);

    // All nodes should be roughly the same distance from origin
    let radius = 300.0;
    for node in graph.nodes() {
        let dist = (node.position.x * node.position.x + node.position.y * node.position.y).sqrt();
        assert!((dist - radius).abs() < 0.1);
    }
}

#[test]
fn test_graph_stats() {
    let mut graph = GraphEngine::new();

    graph.add_node(create_test_primal("1", "Node 1"));
    graph.add_node(create_test_primal("2", "Node 2"));
    graph.add_node(create_test_primal("3", "Node 3"));

    graph.add_edge(TopologyEdge {
        from: PrimalId::from("1"),
        to: PrimalId::from("2"),
        edge_type: "test".to_string(),
        label: None,
        capability: None,
        metrics: None,
    });
    graph.add_edge(TopologyEdge {
        from: PrimalId::from("2"),
        to: PrimalId::from("3"),
        edge_type: "test".to_string(),
        label: None,
        capability: None,
        metrics: None,
    });

    let stats = graph.stats();
    assert_eq!(stats.node_count, 3);
    assert_eq!(stats.edge_count, 2);
    assert!((stats.avg_degree - 1.33).abs() < 0.01);
}
