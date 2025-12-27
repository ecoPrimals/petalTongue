//! Integration tests for petalTongue UI components
//!
//! Tests the state management, data source, and integration between modules.

use petal_tongue_core::{GraphEngine, PrimalHealthStatus, PrimalInfo, TopologyEdge};
use std::sync::{Arc, RwLock};

// Module path imports - adjust based on actual module structure
// These tests will validate the refactored modules

#[test]
fn test_graph_engine_integration() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));

    // Add a primal
    let primal = PrimalInfo {
        id: "test-primal-1".to_string(),
        name: "Test Primal".to_string(),
        primal_type: "compute".to_string(),
        endpoint: "http://test:8080".to_string(),
        health: PrimalHealthStatus::Healthy,
        capabilities: vec!["capability-1".to_string()],
        last_seen: 1234567890,
    };

    {
        let mut g = graph.write().unwrap();
        g.add_node(primal);
    }

    // Verify
    let g = graph.read().unwrap();
    assert_eq!(g.nodes().len(), 1);
    let node = g.get_node("test-primal-1").unwrap();
    assert_eq!(node.info.name, "Test Primal");
}

#[test]
fn test_graph_with_edges() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));

    // Add two primals
    let primal1 = PrimalInfo {
        id: "primal-a".to_string(),
        name: "Primal A".to_string(),
        primal_type: "compute".to_string(),
        endpoint: "http://a:8080".to_string(),
        health: PrimalHealthStatus::Healthy,
        capabilities: vec![],
        last_seen: 1234567890,
    };

    let primal2 = PrimalInfo {
        id: "primal-b".to_string(),
        name: "Primal B".to_string(),
        primal_type: "storage".to_string(),
        endpoint: "http://b:8080".to_string(),
        health: PrimalHealthStatus::Healthy,
        capabilities: vec![],
        last_seen: 1234567890,
    };

    let edge = TopologyEdge {
        from: "primal-a".to_string(),
        to: "primal-b".to_string(),
        edge_type: "capability".to_string(),
        label: Some("test-edge".to_string()),
    };

    {
        let mut g = graph.write().unwrap();
        g.add_node(primal1);
        g.add_node(primal2);
        g.add_edge(edge);
    }

    // Verify
    let g = graph.read().unwrap();
    assert_eq!(g.nodes().len(), 2);
    assert_eq!(g.edges().len(), 1);
}

#[test]
fn test_multiple_primals_different_health() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));

    let primals = vec![
        PrimalInfo {
            id: "healthy-1".to_string(),
            name: "Healthy Primal".to_string(),
            primal_type: "compute".to_string(),
            endpoint: "http://h1:8080".to_string(),
            health: PrimalHealthStatus::Healthy,
            capabilities: vec![],
            last_seen: 1234567890,
        },
        PrimalInfo {
            id: "warning-1".to_string(),
            name: "Warning Primal".to_string(),
            primal_type: "storage".to_string(),
            endpoint: "http://w1:8080".to_string(),
            health: PrimalHealthStatus::Warning,
            capabilities: vec![],
            last_seen: 1234567890,
        },
        PrimalInfo {
            id: "critical-1".to_string(),
            name: "Critical Primal".to_string(),
            primal_type: "network".to_string(),
            endpoint: "http://c1:8080".to_string(),
            health: PrimalHealthStatus::Critical,
            capabilities: vec![],
            last_seen: 1234567890,
        },
    ];

    {
        let mut g = graph.write().unwrap();
        for primal in primals {
            g.add_node(primal);
        }
    }

    // Verify
    let g = graph.read().unwrap();
    assert_eq!(g.nodes().len(), 3);

    let healthy = g.get_node("healthy-1").unwrap();
    assert!(matches!(healthy.info.health, PrimalHealthStatus::Healthy));

    let warning = g.get_node("warning-1").unwrap();
    assert!(matches!(warning.info.health, PrimalHealthStatus::Warning));

    let critical = g.get_node("critical-1").unwrap();
    assert!(matches!(critical.info.health, PrimalHealthStatus::Critical));
}

#[test]
fn test_graph_clear() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));

    // Add primals
    {
        let mut g = graph.write().unwrap();
        for i in 0..5 {
            let primal = PrimalInfo {
                id: format!("primal-{}", i),
                name: format!("Primal {}", i),
                primal_type: "compute".to_string(),
                endpoint: format!("http://p{}:8080", i),
                health: PrimalHealthStatus::Healthy,
                capabilities: vec![],
                last_seen: 1234567890,
            };
            g.add_node(primal);
        }
    }

    // Verify added
    {
        let g = graph.read().unwrap();
        assert_eq!(g.nodes().len(), 5);
    }

    // Clear
    {
        let mut g = graph.write().unwrap();
        g.clear();
    }

    // Verify cleared
    {
        let g = graph.read().unwrap();
        assert_eq!(g.nodes().len(), 0);
    }
}

#[test]
fn test_primal_with_capabilities() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));

    let primal = PrimalInfo {
        id: "capable-primal".to_string(),
        name: "Capable Primal".to_string(),
        primal_type: "compute".to_string(),
        endpoint: "http://capable:8080".to_string(),
        health: PrimalHealthStatus::Healthy,
        capabilities: vec![
            "compute.execute".to_string(),
            "compute.schedule".to_string(),
            "storage.read".to_string(),
        ],
        last_seen: 1234567890,
    };

    {
        let mut g = graph.write().unwrap();
        g.add_node(primal);
    }

    let g = graph.read().unwrap();
    let node = g.get_node("capable-primal").unwrap();
    assert_eq!(node.info.capabilities.len(), 3);
    assert!(
        node.info
            .capabilities
            .contains(&"compute.execute".to_string())
    );
}

#[test]
fn test_edge_with_label() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));

    // Add primals
    {
        let mut g = graph.write().unwrap();
        g.add_node(PrimalInfo {
            id: "source".to_string(),
            name: "Source".to_string(),
            primal_type: "compute".to_string(),
            endpoint: "http://source:8080".to_string(),
            health: PrimalHealthStatus::Healthy,
            capabilities: vec![],
            last_seen: 1234567890,
        });

        g.add_node(PrimalInfo {
            id: "target".to_string(),
            name: "Target".to_string(),
            primal_type: "storage".to_string(),
            endpoint: "http://target:8080".to_string(),
            health: PrimalHealthStatus::Healthy,
            capabilities: vec![],
            last_seen: 1234567890,
        });

        g.add_edge(TopologyEdge {
            from: "source".to_string(),
            to: "target".to_string(),
            edge_type: "data_transfer".to_string(),
            label: Some("12.5ms latency".to_string()),
        });
    }

    let g = graph.read().unwrap();
    let edges = g.edges();
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].label, Some("12.5ms latency".to_string()));
}

#[test]
fn test_concurrent_access() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));

    // Add initial primal
    {
        let mut g = graph.write().unwrap();
        g.add_node(PrimalInfo {
            id: "initial".to_string(),
            name: "Initial".to_string(),
            primal_type: "compute".to_string(),
            endpoint: "http://initial:8080".to_string(),
            health: PrimalHealthStatus::Healthy,
            capabilities: vec![],
            last_seen: 1234567890,
        });
    }

    // Multiple readers
    let g1 = graph.clone();
    let g2 = graph.clone();

    let r1 = g1.read().unwrap();
    let r2 = g2.read().unwrap();

    assert_eq!(r1.nodes().len(), 1);
    assert_eq!(r2.nodes().len(), 1);

    drop(r1);
    drop(r2);

    // Writer after readers
    {
        let mut g = graph.write().unwrap();
        g.clear();
    }

    let g = graph.read().unwrap();
    assert_eq!(g.nodes().len(), 0);
}

#[test]
fn test_graph_stats() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));

    {
        let mut g = graph.write().unwrap();

        // Add 3 primals
        for i in 0..3 {
            g.add_node(PrimalInfo {
                id: format!("primal-{}", i),
                name: format!("Primal {}", i),
                primal_type: "compute".to_string(),
                endpoint: format!("http://p{}:8080", i),
                health: PrimalHealthStatus::Healthy,
                capabilities: vec![],
                last_seen: 1234567890,
            });
        }

        // Add edges to form a triangle
        g.add_edge(TopologyEdge {
            from: "primal-0".to_string(),
            to: "primal-1".to_string(),
            edge_type: "capability".to_string(),
            label: None,
        });

        g.add_edge(TopologyEdge {
            from: "primal-1".to_string(),
            to: "primal-2".to_string(),
            edge_type: "capability".to_string(),
            label: None,
        });

        g.add_edge(TopologyEdge {
            from: "primal-2".to_string(),
            to: "primal-0".to_string(),
            edge_type: "capability".to_string(),
            label: None,
        });
    }

    let g = graph.read().unwrap();
    let stats = g.stats();

    assert_eq!(stats.node_count, 3);
    assert_eq!(stats.edge_count, 3);
    assert!((stats.avg_degree - 2.0).abs() < f32::EPSILON); // 3 edges / 3 nodes * 2 = 2.0
}

#[test]
fn test_remove_node_removes_edges() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));

    {
        let mut g = graph.write().unwrap();

        // Add 3 primals
        for i in 0..3 {
            g.add_node(PrimalInfo {
                id: format!("primal-{}", i),
                name: format!("Primal {}", i),
                primal_type: "compute".to_string(),
                endpoint: format!("http://p{}:8080", i),
                health: PrimalHealthStatus::Healthy,
                capabilities: vec![],
                last_seen: 1234567890,
            });
        }

        // Add edges
        g.add_edge(TopologyEdge {
            from: "primal-0".to_string(),
            to: "primal-1".to_string(),
            edge_type: "capability".to_string(),
            label: None,
        });

        g.add_edge(TopologyEdge {
            from: "primal-1".to_string(),
            to: "primal-2".to_string(),
            edge_type: "capability".to_string(),
            label: None,
        });
    }

    // Verify initial state
    {
        let g = graph.read().unwrap();
        assert_eq!(g.nodes().len(), 3);
        assert_eq!(g.edges().len(), 2);
    }

    // Remove middle node
    {
        let mut g = graph.write().unwrap();
        g.remove_node("primal-1");
    }

    // Verify edges connected to removed node are also removed
    {
        let g = graph.read().unwrap();
        assert_eq!(g.nodes().len(), 2);
        assert_eq!(g.edges().len(), 0); // Both edges involved primal-1
    }
}
