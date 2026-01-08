//! E2E Tests for Tutorial Mode
//!
//! Tests the full tutorial mode workflow including:
//! - Explicit tutorial mode (SHOWCASE_MODE)
//! - Graceful fallback (no providers)
//! - Graph population
//! - Integration with app initialization

use petal_tongue_core::{GraphEngine, LayoutAlgorithm};
use petal_tongue_ui::tutorial_mode::{TutorialMode, should_fallback};
use std::sync::{Arc, RwLock};

#[test]
fn test_e2e_tutorial_mode_disabled_by_default() {
    // Tutorial mode should be OFF by default
    let tutorial = TutorialMode::new();
    assert!(
        !tutorial.is_enabled(),
        "Tutorial mode should be disabled without SHOWCASE_MODE"
    );
}

#[test]
fn test_e2e_fallback_with_no_providers() {
    // When no providers found, should recommend fallback
    assert!(
        should_fallback(0),
        "Should fallback when no providers available"
    );
}

#[test]
fn test_e2e_fallback_with_providers() {
    // When providers found, should NOT fallback
    assert!(
        !should_fallback(1),
        "Should not fallback when providers exist"
    );
}

#[test]
fn test_e2e_minimal_fallback_scenario_complete_flow() {
    // E2E test: Full fallback scenario creation and validation
    let graph = Arc::new(RwLock::new(GraphEngine::new()));

    // Create fallback scenario
    TutorialMode::create_fallback_scenario(Arc::clone(&graph), LayoutAlgorithm::ForceDirected);

    // Verify graph is populated and usable
    let graph = graph.read().unwrap();

    // Should have tutorial nodes
    assert_eq!(graph.nodes().len(), 3, "Should have 3 tutorial primals");
    assert_eq!(graph.edges().len(), 2, "Should have 2 connections");

    // Verify nodes are properly formed
    for node in graph.nodes() {
        assert!(!node.info.id.is_empty(), "Node ID should not be empty");
        assert!(!node.info.name.is_empty(), "Node name should not be empty");
        assert!(
            !node.info.primal_type.is_empty(),
            "Node type should not be empty"
        );
        assert!(
            !node.info.endpoint.is_empty(),
            "Node endpoint should not be empty"
        );
    }

    // Verify edges connect valid nodes
    let node_ids: Vec<&str> = graph.nodes().iter().map(|n| n.info.id.as_str()).collect();
    for edge in graph.edges() {
        assert!(
            node_ids.contains(&edge.from.as_str()),
            "Edge 'from' should reference valid node: {}",
            edge.from
        );
        assert!(
            node_ids.contains(&edge.to.as_str()),
            "Edge 'to' should reference valid node: {}",
            edge.to
        );
    }
}

#[test]
fn test_e2e_tutorial_mode_load_scenario() {
    // E2E test: Load tutorial scenario and verify it's usable
    let tutorial = TutorialMode::new();
    let graph = Arc::new(RwLock::new(GraphEngine::new()));

    // Load tutorial data
    tutorial.load_into_graph(Arc::clone(&graph), LayoutAlgorithm::ForceDirected);

    // Verify graph is populated
    let graph = graph.read().unwrap();
    assert!(!graph.nodes().is_empty(), "Tutorial should load nodes");

    // Note: Scenarios may or may not have edges depending on the scenario
    // The minimal scenario has edges, but we test that separately
}

#[test]
fn test_e2e_tutorial_scenario_with_all_layouts() {
    // E2E test: Verify tutorial works with all layout algorithms
    let tutorial = TutorialMode::new();

    let layouts = vec![
        LayoutAlgorithm::ForceDirected,
        LayoutAlgorithm::Hierarchical,
        LayoutAlgorithm::Circular,
    ];

    for layout in layouts {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        tutorial.load_into_graph(Arc::clone(&graph), layout);

        let graph = graph.read().unwrap();
        assert!(
            !graph.nodes().is_empty(),
            "Should load nodes with {:?} layout",
            layout
        );
    }
}

#[test]
fn test_e2e_fallback_graph_is_interactive() {
    // E2E test: Verify fallback graph supports interactive operations
    let graph = Arc::new(RwLock::new(GraphEngine::new()));

    TutorialMode::create_fallback_scenario(Arc::clone(&graph), LayoutAlgorithm::ForceDirected);

    // Test graph operations
    {
        let graph = graph.read().unwrap();

        // Can query nodes
        let nodes = graph.nodes();
        assert!(!nodes.is_empty());

        // Can query edges
        let edges = graph.edges();
        assert!(!edges.is_empty());

        // Can find specific node
        let petal_node = nodes.iter().find(|n| n.info.id.contains("petaltongue"));
        assert!(petal_node.is_some(), "Should find petalTongue node");
    }

    // Can modify graph (not read-only)
    {
        let mut graph = graph.write().unwrap();
        let initial_count = graph.nodes().len();

        // Should be able to apply layout
        graph.layout(10);

        // Node count shouldn't change from layout
        assert_eq!(
            graph.nodes().len(),
            initial_count,
            "Layout should not add/remove nodes"
        );
    }
}

#[test]
fn test_e2e_tutorial_self_awareness() {
    // E2E test: Tutorial includes petalTongue itself (self-awareness)
    let graph = Arc::new(RwLock::new(GraphEngine::new()));

    TutorialMode::create_fallback_scenario(Arc::clone(&graph), LayoutAlgorithm::ForceDirected);

    let graph = graph.read().unwrap();

    // Should include petalTongue itself
    let self_node = graph
        .nodes()
        .iter()
        .find(|n| n.info.id == "petaltongue-tutorial");

    assert!(
        self_node.is_some(),
        "Tutorial should include petalTongue itself"
    );

    let self_node = self_node.unwrap();
    assert_eq!(self_node.info.primal_type, "Visualization");
    assert!(
        self_node
            .info
            .capabilities
            .contains(&"visual_2d".to_string())
    );
}

#[test]
fn test_e2e_tutorial_graph_connectivity() {
    // E2E test: Tutorial graph should be connected (no isolated nodes)
    let graph = Arc::new(RwLock::new(GraphEngine::new()));

    TutorialMode::create_fallback_scenario(Arc::clone(&graph), LayoutAlgorithm::ForceDirected);

    let graph = graph.read().unwrap();
    let nodes = graph.nodes();
    let edges = graph.edges();

    // Build adjacency information
    let mut connected_nodes = std::collections::HashSet::new();
    for edge in edges {
        connected_nodes.insert(edge.from.clone());
        connected_nodes.insert(edge.to.clone());
    }

    // All nodes should be connected (no isolated nodes)
    for node in nodes {
        assert!(
            connected_nodes.contains(&node.info.id),
            "Node {} should be connected to the graph",
            node.info.id
        );
    }
}

#[test]
fn test_e2e_tutorial_timestamps_are_recent() {
    // E2E test: Tutorial nodes should have recent timestamps
    let graph = Arc::new(RwLock::new(GraphEngine::new()));

    TutorialMode::create_fallback_scenario(Arc::clone(&graph), LayoutAlgorithm::ForceDirected);

    let graph = graph.read().unwrap();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // All tutorial nodes should have recent timestamps
    for node in graph.nodes() {
        let age = now.saturating_sub(node.info.last_seen);
        assert!(
            age < 60,
            "Node {} timestamp should be recent (age: {}s)",
            node.info.id,
            age
        );
    }
}

#[test]
fn test_e2e_tutorial_node_health() {
    // E2E test: All tutorial nodes should be healthy
    let graph = Arc::new(RwLock::new(GraphEngine::new()));

    TutorialMode::create_fallback_scenario(Arc::clone(&graph), LayoutAlgorithm::ForceDirected);

    let graph = graph.read().unwrap();

    for node in graph.nodes() {
        assert_eq!(
            node.info.health,
            petal_tongue_core::PrimalHealthStatus::Healthy,
            "Tutorial node {} should be healthy",
            node.info.id
        );
    }
}

#[test]
fn test_e2e_tutorial_provides_working_example() {
    // E2E test: Tutorial provides a minimal but complete example
    let graph = Arc::new(RwLock::new(GraphEngine::new()));

    TutorialMode::create_fallback_scenario(Arc::clone(&graph), LayoutAlgorithm::ForceDirected);

    let graph = graph.read().unwrap();

    // Should have enough nodes to demonstrate concepts (3 minimum)
    assert!(
        graph.nodes().len() >= 3,
        "Tutorial should provide at least 3 primals"
    );

    // Should have connections (not just isolated nodes)
    assert!(
        graph.edges().len() >= 2,
        "Tutorial should provide at least 2 connections"
    );

    // Should include diverse primal types
    let primal_types: Vec<&str> = graph
        .nodes()
        .iter()
        .map(|n| n.info.primal_type.as_str())
        .collect();

    let unique_types: std::collections::HashSet<_> = primal_types.into_iter().collect();
    assert!(
        unique_types.len() >= 2,
        "Tutorial should show diverse primal types"
    );
}
