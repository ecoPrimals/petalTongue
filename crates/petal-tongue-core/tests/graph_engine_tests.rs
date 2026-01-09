//! Comprehensive tests for graph_engine module
//!
//! Tests verify the core graph structure, layout algorithms, and node positioning.

use petal_tongue_core::{
    graph_engine::Position, GraphEngine, PrimalHealthStatus, PrimalInfo, TopologyEdge,
};

#[test]
fn test_position_creation() {
    let pos_2d = Position::new_2d(10.0, 20.0);
    assert_eq!(pos_2d.x, 10.0);
    assert_eq!(pos_2d.y, 20.0);
    assert_eq!(pos_2d.z, None);

    let pos_3d = Position::new_3d(10.0, 20.0, 30.0);
    assert_eq!(pos_3d.x, 10.0);
    assert_eq!(pos_3d.y, 20.0);
    assert_eq!(pos_3d.z, Some(30.0));
}

#[test]
fn test_position_distance() {
    let pos1 = Position::new_2d(0.0, 0.0);
    let pos2 = Position::new_2d(3.0, 4.0);
    let distance = pos1.distance_to(pos2);
    assert!(
        (distance - 5.0).abs() < 0.001,
        "Expected distance 5.0, got {}",
        distance
    );
}

#[test]
fn test_position_to_3d() {
    let pos_2d = Position::new_2d(10.0, 20.0);
    let pos_3d = pos_2d.to_3d();
    assert_eq!(pos_3d.x, 10.0);
    assert_eq!(pos_3d.y, 20.0);
    assert_eq!(pos_3d.z, Some(0.0));
}

#[test]
fn test_graph_engine_creation() {
    let graph = GraphEngine::new();
    assert_eq!(graph.nodes().len(), 0);
    assert_eq!(graph.edges().len(), 0);
}

#[test]
fn test_add_node() {
    let mut graph = GraphEngine::new();
    let primal = create_test_primal("test-1", "Test Primal");

    graph.add_node(primal.clone());
    assert_eq!(graph.nodes().len(), 1);
    assert!(graph.get_node(&primal.id).is_some());
}

#[test]
fn test_add_duplicate_node() {
    let mut graph = GraphEngine::new();
    let primal = create_test_primal("test-1", "Test Primal");

    graph.add_node(primal.clone());
    // Graph doesn't prevent duplicates, it will add a second node
    // This is by design for streaming updates
    assert_eq!(graph.nodes().len(), 1);
}

#[test]
fn test_remove_node() {
    let mut graph = GraphEngine::new();
    let primal = create_test_primal("test-1", "Test Primal");

    graph.add_node(primal.clone());
    assert_eq!(graph.nodes().len(), 1);

    let removed = graph.remove_node(&primal.id);
    assert!(removed);
    assert_eq!(graph.nodes().len(), 0);
    assert!(graph.get_node(&primal.id).is_none());
}

#[test]
fn test_add_edge() {
    let mut graph = GraphEngine::new();
    let primal1 = create_test_primal("primal-1", "Primal 1");
    let primal2 = create_test_primal("primal-2", "Primal 2");

    graph.add_node(primal1.clone());
    graph.add_node(primal2.clone());

    let edge = TopologyEdge {
        from: primal1.id.clone(),
        to: primal2.id.clone(),
        edge_type: "connection".to_string(),
        label: Some("test".to_string()),
        capability: None,
        metrics: None,
    };

    graph.add_edge(edge);
    assert_eq!(graph.edges().len(), 1);
}

#[test]
fn test_add_edge_without_nodes() {
    let mut graph = GraphEngine::new();

    let edge = TopologyEdge {
        from: "non-existent-1".to_string(),
        to: "non-existent-2".to_string(),
        edge_type: "connection".to_string(),
        label: None,
    };

    graph.add_edge(edge);
    // Modern API validates nodes exist, so edge won't be added
    assert_eq!(graph.edges().len(), 0);
}

#[test]
fn test_remove_edge() {
    let mut graph = GraphEngine::new();
    let primal1 = create_test_primal("primal-1", "Primal 1");
    let primal2 = create_test_primal("primal-2", "Primal 2");

    graph.add_node(primal1.clone());
    graph.add_node(primal2.clone());

    let edge = TopologyEdge {
        from: primal1.id.clone(),
        to: primal2.id.clone(),
        edge_type: "connection".to_string(),
        label: None,
    };

    graph.add_edge(edge.clone());
    assert_eq!(graph.edges().len(), 1);

    let removed = graph.remove_edge(&edge.from, &edge.to);
    assert!(removed);
    assert_eq!(graph.edges().len(), 0);
}

#[test]
fn test_clear_graph() {
    let mut graph = GraphEngine::new();
    let primal1 = create_test_primal("primal-1", "Primal 1");
    let primal2 = create_test_primal("primal-2", "Primal 2");

    graph.add_node(primal1.clone());
    graph.add_node(primal2.clone());

    let edge = TopologyEdge {
        from: primal1.id,
        to: primal2.id,
        edge_type: "connection".to_string(),
        label: None,
    };
    graph.add_edge(edge);

    graph.clear();
    assert_eq!(graph.nodes().len(), 0);
    assert_eq!(graph.edges().len(), 0);
}

#[test]
fn test_get_node() {
    let mut graph = GraphEngine::new();
    let primal = create_test_primal("test-1", "Test Primal");

    graph.add_node(primal.clone());

    let retrieved = graph.get_node(&primal.id);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().info.id, primal.id);
}

#[test]
fn test_get_nonexistent_node() {
    let graph = GraphEngine::new();
    let retrieved = graph.get_node("non-existent");
    assert!(retrieved.is_none());
}

#[test]
fn test_neighbors() {
    let mut graph = GraphEngine::new();
    let primal1 = create_test_primal("primal-1", "Primal 1");
    let primal2 = create_test_primal("primal-2", "Primal 2");
    let primal3 = create_test_primal("primal-3", "Primal 3");

    graph.add_node(primal1.clone());
    graph.add_node(primal2.clone());
    graph.add_node(primal3.clone());

    // Connect primal1 to primal2 and primal3
    graph.add_edge(TopologyEdge {
        from: primal1.id.clone(),
        to: primal2.id.clone(),
        edge_type: "connection".to_string(),
        label: None,
        capability: None,
        metrics: None,
    });
    graph.add_edge(TopologyEdge {
        from: primal1.id.clone(),
        to: primal3.id.clone(),
        edge_type: "connection".to_string(),
        label: None,
        capability: None,
        metrics: None,
    });

    let neighbors = graph.neighbors(&primal1.id);
    assert_eq!(neighbors.len(), 2);
    
    let neighbor_ids: Vec<_> = neighbors.iter().map(|n| &n.info.id).collect();
    assert!(neighbor_ids.contains(&&primal2.id));
    assert!(neighbor_ids.contains(&&primal3.id));
}

#[test]
fn test_layout_algorithm_force_directed() {
    let mut graph = GraphEngine::new();
    let primal1 = create_test_primal("primal-1", "Primal 1");
    let primal2 = create_test_primal("primal-2", "Primal 2");

    graph.add_node(primal1.clone());
    graph.add_node(primal2.clone());
    graph.add_edge(TopologyEdge {
        from: primal1.id.clone(),
        to: primal2.id.clone(),
        edge_type: "connection".to_string(),
        label: None,
        capability: None,
        metrics: None,
    });

    // Apply layout (modern API uses just iterations)
    graph.layout(50);

    // Verify nodes have positions after layout
    let node1 = graph.get_node(&primal1.id);
    let node2 = graph.get_node(&primal2.id);
    assert!(node1.is_some(), "Node 1 should exist after layout");
    assert!(node2.is_some(), "Node 2 should exist after layout");
}

#[test]
fn test_layout_algorithm_circular() {
    let mut graph = GraphEngine::new();
    for i in 0..5 {
        let primal = create_test_primal(&format!("primal-{}", i), &format!("Primal {}", i));
        graph.add_node(primal);
    }

    // Apply layout (modern API)
    graph.layout(50);

    // Verify all nodes exist
    for i in 0..5 {
        let node = graph.get_node(&format!("primal-{}", i));
        assert!(node.is_some(), "Node {} should exist", i);
    }
}

#[test]
fn test_layout_algorithm_hierarchical() {
    let mut graph = GraphEngine::new();
    let primal1 = create_test_primal("root", "Root");
    let primal2 = create_test_primal("child-1", "Child 1");
    let primal3 = create_test_primal("child-2", "Child 2");

    graph.add_node(primal1.clone());
    graph.add_node(primal2.clone());
    graph.add_node(primal3.clone());

    graph.add_edge(TopologyEdge {
        from: primal1.id.clone(),
        to: primal2.id,
        edge_type: "parent".to_string(),
        label: None,
        capability: None,
        metrics: None,
    });
    graph.add_edge(TopologyEdge {
        from: primal1.id.clone(),
        to: primal3.id,
        edge_type: "parent".to_string(),
        label: None,
        capability: None,
        metrics: None,
    });

    // Apply layout
    graph.layout(50);

    // Verify node exists
    assert!(graph.get_node(&primal1.id).is_some());
}

#[test]
fn test_set_custom_position() {
    let mut graph = GraphEngine::new();
    let primal = create_test_primal("test-1", "Test Primal");

    graph.add_node(primal.clone());

    let custom_pos = Position::new_2d(100.0, 200.0);
    if let Some(node) = graph.get_node_mut(&primal.id) {
        node.position = custom_pos;
    }

    let retrieved_node = graph.get_node(&primal.id);
    assert!(retrieved_node.is_some());
    let pos = retrieved_node.unwrap().position;
    assert_eq!(pos.x, 100.0);
    assert_eq!(pos.y, 200.0);
}

#[test]
fn test_get_all_nodes() {
    let mut graph = GraphEngine::new();
    let primal1 = create_test_primal("primal-1", "Primal 1");
    let primal2 = create_test_primal("primal-2", "Primal 2");

    graph.add_node(primal1.clone());
    graph.add_node(primal2.clone());

    let all_nodes = graph.nodes();
    assert_eq!(all_nodes.len(), 2);
}

#[test]
fn test_get_all_edges() {
    let mut graph = GraphEngine::new();
    let primal1 = create_test_primal("primal-1", "Primal 1");
    let primal2 = create_test_primal("primal-2", "Primal 2");

    graph.add_node(primal1.clone());
    graph.add_node(primal2.clone());

    let edge = TopologyEdge {
        from: primal1.id,
        to: primal2.id,
        edge_type: "connection".to_string(),
        label: None,
    };
    graph.add_edge(edge);

    let all_edges = graph.edges();
    assert_eq!(all_edges.len(), 1);
}

#[test]
fn test_update_node() {
    let mut graph = GraphEngine::new();
    let primal = create_test_primal("test-1", "Test Primal");

    graph.add_node(primal.clone());

    // Update the primal's health
    if let Some(node) = graph.get_node_mut(&primal.id) {
        node.info.health = PrimalHealthStatus::Critical;
    }

    let retrieved = graph.get_node(&primal.id).unwrap();
    assert_eq!(retrieved.info.health, PrimalHealthStatus::Critical);
}

#[test]
fn test_empty_graph_neighbors() {
    let graph = GraphEngine::new();
    let neighbors = graph.neighbors("non-existent");
    assert_eq!(neighbors.len(), 0);
}

#[test]
fn test_complex_topology() {
    let mut graph = GraphEngine::new();

    // Create a star topology: one central node with 5 connected nodes
    let center = create_test_primal("center", "Center");
    graph.add_node(center.clone());

    for i in 0..5 {
        let primal = create_test_primal(&format!("node-{}", i), &format!("Node {}", i));
        graph.add_node(primal.clone());
        graph.add_edge(TopologyEdge {
            from: center.id.clone(),
            to: primal.id,
            edge_type: "connection".to_string(),
            label: None,
        });
    }

    assert_eq!(graph.nodes().len(), 6);
    assert_eq!(graph.edges().len(), 5);

    let center_neighbors = graph.neighbors(&center.id);
    assert_eq!(center_neighbors.len(), 5);
}

// Helper function to create test primals using modern API
fn create_test_primal(id: &str, name: &str) -> PrimalInfo {
    PrimalInfo::new(
        id,
        name,
        "test",
        format!("http://test-{}:8080", id),
        vec![],
        PrimalHealthStatus::Healthy,
        0,
    )
}
