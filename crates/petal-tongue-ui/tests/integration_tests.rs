// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
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
        id: "test-primal-1".into(),
        name: "Test Primal".to_string(),
        primal_type: "compute".to_string(),
        endpoint: "http://test:8080".to_string(),
        health: PrimalHealthStatus::Healthy,
        properties: petal_tongue_core::Properties::new(),
        capabilities: vec!["capability-1".to_string()],
        last_seen: 1_234_567_890,
        endpoints: None,
        metadata: None,
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
        id: "primal-a".into(),
        name: "Primal A".to_string(),
        primal_type: "compute".to_string(),
        endpoint: "http://a:8080".to_string(),
        health: PrimalHealthStatus::Healthy,
        properties: petal_tongue_core::Properties::new(),
        capabilities: vec![],
        last_seen: 1_234_567_890,
        endpoints: None,
        metadata: None,
    };

    let primal2 = PrimalInfo {
        id: "primal-b".into(),
        name: "Primal B".to_string(),
        primal_type: "storage".to_string(),
        endpoint: "http://b:8080".to_string(),
        health: PrimalHealthStatus::Healthy,
        properties: petal_tongue_core::Properties::new(),
        capabilities: vec![],
        last_seen: 1_234_567_890,
        endpoints: None,
        metadata: None,
    };

    let edge = TopologyEdge {
        from: "primal-a".into(),
        to: "primal-b".into(),
        edge_type: "capability".to_string(),
        label: Some("test-edge".to_string()),
        capability: None,
        metrics: None,
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
            id: "healthy-1".into(),
            name: "Healthy Primal".to_string(),
            primal_type: "compute".to_string(),
            endpoint: "http://h1:8080".to_string(),
            health: PrimalHealthStatus::Healthy,
            properties: petal_tongue_core::Properties::new(),
            capabilities: vec![],
            last_seen: 1_234_567_890,
            endpoints: None,
            metadata: None,
        },
        PrimalInfo {
            id: "warning-1".into(),
            name: "Warning Primal".to_string(),
            primal_type: "storage".to_string(),
            endpoint: "http://w1:8080".to_string(),
            health: PrimalHealthStatus::Warning,
            properties: petal_tongue_core::Properties::new(),
            capabilities: vec![],
            last_seen: 1_234_567_890,
            endpoints: None,
            metadata: None,
        },
        PrimalInfo {
            id: "critical-1".into(),
            name: "Critical Primal".to_string(),
            primal_type: "network".to_string(),
            endpoint: "http://c1:8080".to_string(),
            health: PrimalHealthStatus::Critical,
            properties: petal_tongue_core::Properties::new(),
            capabilities: vec![],
            last_seen: 1_234_567_890,
            endpoints: None,
            metadata: None,
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
                id: format!("primal-{i}").into(),
                name: format!("Primal {i}"),
                primal_type: "compute".to_string(),
                endpoint: format!("http://p{i}:8080"),
                health: PrimalHealthStatus::Healthy,
                properties: petal_tongue_core::Properties::new(),
                capabilities: vec![],
                last_seen: 1_234_567_890,
                endpoints: None,
                metadata: None,
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
        id: "capable-primal".into(),
        name: "Capable Primal".to_string(),
        primal_type: "compute".to_string(),
        endpoint: "http://capable:8080".to_string(),
        health: PrimalHealthStatus::Healthy,
        properties: petal_tongue_core::Properties::new(),
        capabilities: vec![
            "compute.execute".to_string(),
            "compute.schedule".to_string(),
            "storage.read".to_string(),
        ],
        last_seen: 1_234_567_890,
        endpoints: None,
        metadata: None,
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
            id: "source".into(),
            name: "Source".to_string(),
            primal_type: "compute".to_string(),
            endpoint: "http://source:8080".to_string(),
            health: PrimalHealthStatus::Healthy,
            properties: petal_tongue_core::Properties::new(),
            capabilities: vec![],
            last_seen: 1_234_567_890,
            endpoints: None,
            metadata: None,
        });

        g.add_node(PrimalInfo {
            id: "target".into(),
            name: "Target".to_string(),
            primal_type: "storage".to_string(),
            endpoint: "http://target:8080".to_string(),
            health: PrimalHealthStatus::Healthy,
            properties: petal_tongue_core::Properties::new(),
            capabilities: vec![],
            last_seen: 1_234_567_890,
            endpoints: None,
            metadata: None,
        });

        g.add_edge(TopologyEdge {
            from: "source".into(),
            to: "target".into(),
            edge_type: "data_transfer".to_string(),
            label: Some("12.5ms latency".to_string()),
            capability: None,
            metrics: None,
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
            id: "initial".into(),
            name: "Initial".to_string(),
            primal_type: "compute".to_string(),
            endpoint: "http://initial:8080".to_string(),
            health: PrimalHealthStatus::Healthy,
            properties: petal_tongue_core::Properties::new(),
            capabilities: vec![],
            last_seen: 1_234_567_890,
            endpoints: None,
            metadata: None,
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
                id: format!("primal-{i}").into(),
                name: format!("Primal {i}"),
                primal_type: "compute".to_string(),
                endpoint: format!("http://p{i}:8080"),
                health: PrimalHealthStatus::Healthy,
                properties: petal_tongue_core::Properties::new(),
                capabilities: vec![],
                last_seen: 1_234_567_890,
                endpoints: None,
                metadata: None,
            });
        }

        // Add edges to form a triangle
        g.add_edge(TopologyEdge {
            from: "primal-0".into(),
            to: "primal-1".into(),
            edge_type: "capability".to_string(),
            label: None,
            capability: None,
            metrics: None,
        });

        g.add_edge(TopologyEdge {
            from: "primal-1".into(),
            to: "primal-2".into(),
            edge_type: "capability".to_string(),
            label: None,
            capability: None,
            metrics: None,
        });

        g.add_edge(TopologyEdge {
            from: "primal-2".into(),
            to: "primal-0".into(),
            edge_type: "capability".to_string(),
            label: None,
            capability: None,
            metrics: None,
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
                id: format!("primal-{i}").into(),
                name: format!("Primal {i}"),
                primal_type: "compute".to_string(),
                endpoint: format!("http://p{i}:8080"),
                health: PrimalHealthStatus::Healthy,
                properties: petal_tongue_core::Properties::new(),
                capabilities: vec![],
                last_seen: 1_234_567_890,
                endpoints: None,
                metadata: None,
            });
        }

        // Add edges
        g.add_edge(TopologyEdge {
            from: "primal-0".into(),
            to: "primal-1".into(),
            edge_type: "capability".to_string(),
            label: None,
            capability: None,
            metrics: None,
        });

        g.add_edge(TopologyEdge {
            from: "primal-1".into(),
            to: "primal-2".into(),
            edge_type: "capability".to_string(),
            label: None,
            capability: None,
            metrics: None,
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
