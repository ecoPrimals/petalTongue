// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn test_tutorial_mode_creation() {
    let tutorial = TutorialMode::new();
    // Should not be enabled unless env var is set
    assert!(!tutorial.is_enabled());
    // Should have default scenario name
    assert_eq!(tutorial.scenario_name(), "simple");
}

#[test]
fn test_tutorial_mode_scenario_name() {
    let tutorial = TutorialMode::new();
    assert_eq!(tutorial.scenario_name(), "simple");
}

#[test]
fn test_should_fallback_no_providers() {
    // Should fallback when no providers found
    assert!(should_fallback(0));
}

#[test]
fn test_should_not_fallback_with_providers() {
    // Should not fallback when providers exist
    assert!(!should_fallback(1));
    assert!(!should_fallback(5));
    assert!(!should_fallback(100));
}

#[test]
fn test_minimal_example_structure() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    TutorialMode::create_fallback_scenario(Arc::clone(&graph), LayoutAlgorithm::ForceDirected);

    let graph = graph
        .read()
        .expect("SAFETY: Graph lock poisoned - indicates panic in graph thread");

    // Verify structure
    assert_eq!(graph.nodes().len(), 3, "Should have 3 tutorial primals");
    assert_eq!(graph.edges().len(), 2, "Should have 2 connections");
}

#[test]
fn test_minimal_example_node_ids() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    TutorialMode::create_fallback_scenario(Arc::clone(&graph), LayoutAlgorithm::ForceDirected);

    let graph = graph
        .read()
        .expect("SAFETY: Graph lock poisoned - indicates panic in graph thread");
    let nodes = graph.nodes();

    // Verify tutorial nodes have correct IDs
    let ids: Vec<&str> = nodes.iter().map(|n| n.info.id.as_str()).collect();
    assert!(ids.contains(&"petaltongue-tutorial"));
    assert!(ids.contains(&"security-example"));
    assert!(ids.contains(&"discovery-example"));
}

#[test]
fn test_minimal_example_node_health() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    TutorialMode::create_fallback_scenario(Arc::clone(&graph), LayoutAlgorithm::ForceDirected);

    let graph = graph
        .read()
        .expect("SAFETY: Graph lock poisoned - indicates panic in graph thread");

    // All tutorial nodes should be healthy
    for node in graph.nodes() {
        assert_eq!(
            node.info.health,
            PrimalHealthStatus::Healthy,
            "Tutorial node {} should be healthy",
            node.info.id
        );
    }
}

#[test]
fn test_minimal_example_self_awareness() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    TutorialMode::create_fallback_scenario(Arc::clone(&graph), LayoutAlgorithm::ForceDirected);

    let graph = graph
        .read()
        .expect("SAFETY: Graph lock poisoned - indicates panic in graph thread");

    let petal_node = graph
        .nodes()
        .iter()
        .find(|n| n.info.id == "petaltongue-tutorial");

    assert!(petal_node.is_some(), "Should include petalTongue itself");

    let petal_node = petal_node.unwrap();
    assert_eq!(petal_node.info.primal_type, "Visualization");
    assert_eq!(petal_node.info.name, "petalTongue");
    assert!(
        petal_node
            .info
            .capabilities
            .contains(&"ui.render".to_string()),
        "Should have capability taxonomy capabilities"
    );
    assert!(
        petal_node
            .info
            .capabilities
            .contains(&"ipc.json-rpc".to_string()),
        "Should advertise IPC capability"
    );
    assert!(
        petal_node.info.endpoint.starts_with("unix://"),
        "Endpoint should be a Unix socket path, got: {}",
        petal_node.info.endpoint
    );
}

#[test]
fn test_minimal_example_edges() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    TutorialMode::create_fallback_scenario(Arc::clone(&graph), LayoutAlgorithm::ForceDirected);

    let graph = graph
        .read()
        .expect("SAFETY: Graph lock poisoned - indicates panic in graph thread");
    let edges = graph.edges();

    assert_eq!(edges.len(), 2);

    let discovery_edge = edges
        .iter()
        .find(|e| e.from == "petaltongue-tutorial" && e.to == "discovery-example");
    assert!(
        discovery_edge.is_some(),
        "Should have petalTongue → discovery edge"
    );
    assert_eq!(
        discovery_edge.unwrap().edge_type,
        "ipc.discovery",
        "Edge should use semantic type"
    );

    let trust_edge = edges
        .iter()
        .find(|e| e.from == "discovery-example" && e.to == "security-example");
    assert!(
        trust_edge.is_some(),
        "Should have discovery → security edge"
    );
    assert_eq!(
        trust_edge.unwrap().edge_type,
        "ipc.trust",
        "Edge should use semantic type"
    );
}

#[test]
fn test_load_into_graph_with_layout() {
    let tutorial = TutorialMode::new();
    let graph = Arc::new(RwLock::new(GraphEngine::new()));

    // Test with different layouts
    for layout in &[
        LayoutAlgorithm::ForceDirected,
        LayoutAlgorithm::Hierarchical,
        LayoutAlgorithm::Circular,
    ] {
        tutorial.load_into_graph(Arc::clone(&graph), *layout);

        let graph_read = graph
            .read()
            .expect("SAFETY: Graph lock poisoned - indicates panic in graph thread");
        assert!(
            !graph_read.nodes().is_empty(),
            "Should load nodes with {:?}",
            layout
        );
        drop(graph_read); // Release lock for next iteration
    }
}

#[test]
fn test_minimal_example_security_capabilities() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    TutorialMode::create_fallback_scenario(Arc::clone(&graph), LayoutAlgorithm::ForceDirected);

    let graph = graph
        .read()
        .expect("SAFETY: Graph lock poisoned - indicates panic in graph thread");

    let security = graph
        .nodes()
        .iter()
        .find(|n| n.info.id == "security-example")
        .expect("Security Primal should exist");

    assert_eq!(security.info.name, "Security Primal");
    assert!(
        security
            .info
            .capabilities
            .contains(&"security.auth".to_string())
    );
    assert!(
        security
            .info
            .capabilities
            .contains(&"security.encryption".to_string())
    );
    assert_eq!(security.info.endpoint, "discovered-at-runtime");
}

#[test]
fn test_minimal_example_discovery_capabilities() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    TutorialMode::create_fallback_scenario(Arc::clone(&graph), LayoutAlgorithm::ForceDirected);

    let graph = graph
        .read()
        .expect("SAFETY: Graph lock poisoned - indicates panic in graph thread");

    let discovery = graph
        .nodes()
        .iter()
        .find(|n| n.info.id == "discovery-example")
        .expect("Discovery Primal should exist");

    assert_eq!(discovery.info.name, "Discovery Primal");
    assert!(
        discovery
            .info
            .capabilities
            .contains(&"discovery.primals".to_string())
    );
    assert!(
        discovery
            .info
            .capabilities
            .contains(&"orchestration.workflow".to_string())
    );
    assert_eq!(discovery.info.endpoint, "discovered-at-runtime");
}

#[test]
fn test_tutorial_properties() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    TutorialMode::create_fallback_scenario(Arc::clone(&graph), LayoutAlgorithm::ForceDirected);

    let graph = graph
        .read()
        .expect("SAFETY: Graph lock poisoned - indicates panic in graph thread");

    // petalTongue node should have tutorial_note property
    let petal_node = graph
        .nodes()
        .iter()
        .find(|n| n.info.id == "petaltongue-tutorial")
        .unwrap();

    assert!(
        petal_node.info.properties.contains_key("tutorial_note"),
        "Should have tutorial_note property"
    );
}

#[test]
fn test_tutorial_mode_disabled() {
    let tutorial = TutorialMode::disabled();
    assert!(!tutorial.is_enabled());
    assert_eq!(tutorial.scenario_name(), "scenario");
}

#[test]
fn test_should_fallback_edge_cases() {
    assert!(should_fallback(0));
    assert!(!should_fallback(1));
    assert!(!should_fallback(usize::MAX));
}

#[test]
fn test_fallback_creates_valid_graph() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    TutorialMode::create_fallback_scenario(Arc::clone(&graph), LayoutAlgorithm::ForceDirected);

    let graph = graph
        .read()
        .expect("SAFETY: Graph lock poisoned - indicates panic in graph thread");

    // Verify graph is valid and usable
    assert!(!graph.nodes().is_empty());
    assert!(!graph.edges().is_empty());

    // All nodes should have valid timestamps
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    for node in graph.nodes() {
        assert!(
            node.info.last_seen <= now,
            "Node {} has invalid future timestamp",
            node.info.id
        );
        assert!(
            node.info.last_seen > now - 60,
            "Node {} timestamp should be recent",
            node.info.id
        );
    }
}

#[test]
fn test_tutorial_mode_default() {
    let t = TutorialMode::default();
    assert_eq!(t.scenario_name(), "simple");
}

#[test]
fn test_minimal_example_edge_labels() {
    let graph = Arc::new(RwLock::new(GraphEngine::new()));
    TutorialMode::create_fallback_scenario(Arc::clone(&graph), LayoutAlgorithm::ForceDirected);

    let graph = graph
        .read()
        .expect("SAFETY: Graph lock poisoned - indicates panic in graph thread");
    let edges = graph.edges();

    let disc_edge = edges
        .iter()
        .find(|e| e.from == "petaltongue-tutorial" && e.to == "discovery-example")
        .unwrap();
    assert_eq!(disc_edge.label.as_deref(), Some("discovers"));

    let trust_edge = edges
        .iter()
        .find(|e| e.from == "discovery-example" && e.to == "security-example")
        .unwrap();
    assert_eq!(trust_edge.label.as_deref(), Some("authenticates"));
}
