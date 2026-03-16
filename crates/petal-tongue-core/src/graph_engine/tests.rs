// SPDX-License-Identifier: AGPL-3.0-or-later
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
        let dist = node.position.x.hypot(node.position.y);
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

#[test]
fn test_hierarchical_layout() {
    let mut graph = GraphEngine::new();

    graph.add_node(create_test_primal("1", "Root"));
    graph.add_node(create_test_primal("2", "Child1"));
    graph.add_node(create_test_primal("3", "Child2"));
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

    graph.set_layout(LayoutAlgorithm::Hierarchical);
    graph.layout(1);

    let root = graph.get_node("1").unwrap();
    let child1 = graph.get_node("2").unwrap();
    let child2 = graph.get_node("3").unwrap();

    assert!((root.position.y - 0.0).abs() < f32::EPSILON);
    assert!((child1.position.y - 150.0).abs() < f32::EPSILON);
    assert!((child2.position.y - 150.0).abs() < f32::EPSILON);
}

#[test]
fn test_random_layout_deterministic() {
    let mut graph = GraphEngine::new();
    graph.add_node(create_test_primal("a", "A"));
    graph.add_node(create_test_primal("b", "B"));

    graph.set_layout(LayoutAlgorithm::Random);
    graph.layout(1);

    let a = graph.get_node("a").unwrap();

    let mut graph2 = GraphEngine::new();
    graph2.add_node(create_test_primal("a", "A"));
    graph2.add_node(create_test_primal("b", "B"));
    graph2.set_layout(LayoutAlgorithm::Random);
    graph2.layout(1);

    let a2 = graph2.get_node("a").unwrap();
    assert!((a.position.x - a2.position.x).abs() < f32::EPSILON);
    assert!((a.position.y - a2.position.y).abs() < f32::EPSILON);
}

#[test]
fn test_position_new_2d() {
    let p = Position::new_2d(10.0, 20.0);
    assert!((p.x - 10.0).abs() < f32::EPSILON);
    assert!((p.y - 20.0).abs() < f32::EPSILON);
    assert!(p.z.is_none());
}

#[test]
fn test_position_new_3d() {
    let p = Position::new_3d(1.0, 2.0, 3.0);
    assert!((p.x - 1.0).abs() < f32::EPSILON);
    assert!((p.y - 2.0).abs() < f32::EPSILON);
    assert!((p.z.unwrap() - 3.0).abs() < f32::EPSILON);
}

#[test]
fn test_position_to_3d() {
    let p2d = Position::new_2d(5.0, 10.0);
    let p3d = p2d.to_3d();
    assert!((p3d.z.unwrap() - 0.0).abs() < f32::EPSILON);

    let p3 = Position::new_3d(1.0, 2.0, 3.0);
    let p3conv = p3.to_3d();
    assert!((p3conv.z.unwrap() - 3.0).abs() < f32::EPSILON);
}

#[test]
fn test_position_distance_to() {
    let a = Position::new_2d(0.0, 0.0);
    let b = Position::new_2d(3.0, 4.0);
    assert!((a.distance_to(b) - 5.0).abs() < 0.001);
}

#[test]
fn test_position_distance_to_3d() {
    let a = Position::new_3d(0.0, 0.0, 0.0);
    let b = Position::new_3d(1.0, 0.0, 0.0);
    assert!((a.distance_to_3d(b) - 1.0).abs() < 0.001);
}

#[test]
fn test_node_new() {
    let info = create_test_primal("x", "X");
    let node = Node::new(info);
    assert!((node.position.x - 0.0).abs() < f32::EPSILON);
    assert!((node.position.y - 0.0).abs() < f32::EPSILON);
    assert!(node.position.z.is_none());
    assert_eq!(node.info.id.as_str(), "x");
}

#[test]
fn test_node_with_position() {
    let info = create_test_primal("y", "Y");
    let pos = Position::new_2d(100.0, 200.0);
    let node = Node::with_position(info, pos);
    assert!((node.position.x - 100.0).abs() < f32::EPSILON);
    assert!((node.position.y - 200.0).abs() < f32::EPSILON);
}

#[test]
fn test_layout_algorithm_equality() {
    assert_eq!(
        LayoutAlgorithm::ForceDirected,
        LayoutAlgorithm::ForceDirected
    );
    assert_ne!(LayoutAlgorithm::ForceDirected, LayoutAlgorithm::Circular);
}
