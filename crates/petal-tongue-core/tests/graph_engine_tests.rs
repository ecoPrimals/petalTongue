//! Comprehensive tests for graph_engine module
//!
//! Tests verify the core graph structure, layout algorithms, and node positioning.

use petal_tongue_core::{
    GraphEngine, LayoutAlgorithm, Position, PrimalHealthStatus, PrimalInfo, TopologyEdge,
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
    assert_eq!(graph.node_count(), 0);
    assert_eq!(graph.edge_count(), 0);
}

#[test]
fn test_add_node() {
    let mut graph = GraphEngine::new();
    let primal = create_test_primal("test-1", "Test Primal");

    graph.add_node(primal.clone());
    assert_eq!(graph.node_count(), 1);
    assert!(graph.has_node(&primal.id));
}

#[test]
fn test_add_duplicate_node() {
    let mut graph = GraphEngine::new();
    let primal = create_test_primal("test-1", "Test Primal");

    graph.add_node(primal.clone());
    graph.add_node(primal.clone()); // Add again

    // Should still only have one node
    assert_eq!(graph.node_count(), 1);
}

#[test]
fn test_remove_node() {
    let mut graph = GraphEngine::new();
    let primal = create_test_primal("test-1", "Test Primal");

    graph.add_node(primal.clone());
    assert_eq!(graph.node_count(), 1);

    graph.remove_node(&primal.id);
    assert_eq!(graph.node_count(), 0);
    assert!(!graph.has_node(&primal.id));
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
    };

    graph.add_edge(edge);
    assert_eq!(graph.edge_count(), 1);
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
    // Graph should handle missing nodes gracefully
    assert_eq!(graph.edge_count(), 1);
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
    assert_eq!(graph.edge_count(), 1);

    graph.remove_edge(&edge.from, &edge.to);
    assert_eq!(graph.edge_count(), 0);
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
    assert_eq!(graph.node_count(), 0);
    assert_eq!(graph.edge_count(), 0);
}

#[test]
fn test_get_node() {
    let mut graph = GraphEngine::new();
    let primal = create_test_primal("test-1", "Test Primal");

    graph.add_node(primal.clone());

    let retrieved = graph.get_node(&primal.id);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, primal.id);
}

#[test]
fn test_get_nonexistent_node() {
    let graph = GraphEngine::new();
    let retrieved = graph.get_node("non-existent");
    assert!(retrieved.is_none());
}

#[test]
fn test_get_neighbors() {
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
    });
    graph.add_edge(TopologyEdge {
        from: primal1.id.clone(),
        to: primal3.id.clone(),
        edge_type: "connection".to_string(),
        label: None,
    });

    let neighbors = graph.get_neighbors(&primal1.id);
    assert_eq!(neighbors.len(), 2);
    assert!(neighbors.contains(&primal2.id));
    assert!(neighbors.contains(&primal3.id));
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
    });

    graph.apply_layout(LayoutAlgorithm::ForceDirected);

    // Verify nodes have positions after layout
    let pos1 = graph.get_position(&primal1.id);
    let pos2 = graph.get_position(&primal2.id);
    assert!(pos1.is_some(), "Node 1 should have position after layout");
    assert!(pos2.is_some(), "Node 2 should have position after layout");
}

#[test]
fn test_layout_algorithm_circular() {
    let mut graph = GraphEngine::new();
    for i in 0..5 {
        let primal = create_test_primal(&format!("primal-{}", i), &format!("Primal {}", i));
        graph.add_node(primal);
    }

    graph.apply_layout(LayoutAlgorithm::Circular);

    // Verify all nodes have positions
    for i in 0..5 {
        let pos = graph.get_position(&format!("primal-{}", i));
        assert!(pos.is_some(), "Node {} should have position", i);
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
    });
    graph.add_edge(TopologyEdge {
        from: primal1.id.clone(),
        to: primal3.id,
        edge_type: "parent".to_string(),
        label: None,
    });

    graph.apply_layout(LayoutAlgorithm::Hierarchical);

    // Verify positions exist
    assert!(graph.get_position(&primal1.id).is_some());
}

#[test]
fn test_set_custom_position() {
    let mut graph = GraphEngine::new();
    let primal = create_test_primal("test-1", "Test Primal");

    graph.add_node(primal.clone());

    let custom_pos = Position::new_2d(100.0, 200.0);
    graph.set_position(&primal.id, custom_pos);

    let retrieved_pos = graph.get_position(&primal.id);
    assert!(retrieved_pos.is_some());
    let pos = retrieved_pos.unwrap();
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

    let all_nodes = graph.get_all_nodes();
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

    let all_edges = graph.get_all_edges();
    assert_eq!(all_edges.len(), 1);
}

#[test]
fn test_update_node() {
    let mut graph = GraphEngine::new();
    let mut primal = create_test_primal("test-1", "Test Primal");

    graph.add_node(primal.clone());

    // Update the primal
    primal.health = PrimalHealthStatus::Degraded;
    graph.add_node(primal.clone()); // Adding again should update

    let retrieved = graph.get_node(&primal.id).unwrap();
    assert_eq!(retrieved.health, PrimalHealthStatus::Degraded);
}

#[test]
fn test_graph_serialization() {
    let mut graph = GraphEngine::new();
    let primal = create_test_primal("test-1", "Test Primal");
    graph.add_node(primal);

    // Test that graph can be serialized
    let serialized = serde_json::to_string(&graph);
    assert!(serialized.is_ok(), "Graph should be serializable");
}

#[test]
fn test_empty_graph_neighbors() {
    let graph = GraphEngine::new();
    let neighbors = graph.get_neighbors("non-existent");
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

    assert_eq!(graph.node_count(), 6);
    assert_eq!(graph.edge_count(), 5);

    let center_neighbors = graph.get_neighbors(&center.id);
    assert_eq!(center_neighbors.len(), 5);
}

// Helper function to create test primals
#[allow(deprecated)]
fn create_test_primal(id: &str, name: &str) -> PrimalInfo {
    PrimalInfo {
        id: id.to_string(),
        name: name.to_string(),
        primal_type: "test".to_string(),
        endpoint: format!("http://test-{}:8080", id),
        health: PrimalHealthStatus::Healthy,
        trust_level: None,
        family_id: None,
        capabilities: vec![],
        last_seen: 0,
        properties: Default::default(),
    }
}
