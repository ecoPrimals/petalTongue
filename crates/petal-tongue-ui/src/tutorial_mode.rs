//! Tutorial Mode - Demonstration & Graceful Degradation
//!
//! Provides standalone demonstration data and scenarios when:
//! - SHOWCASE_MODE=true (explicit tutorial/demo mode)
//! - No real primals are discovered (graceful fallback)
//! - Testing/development without full ecosystem
//!
//! This is NOT a mock in production - it's a tutorial system that enables
//! petalTongue to function independently for learning and demonstration.
//!
//! # Architecture Philosophy
//!
//! TRUE PRIMAL PRINCIPLE: "Graceful degradation, not silent mocking"
//! - Tutorial mode is EXPLICIT (environment variable)
//! - Fallback is LOGGED (user knows what's happening)
//! - Production discovers real primals (no hardcoding)
//!
//! # Usage
//!
//! ```bash
//! # Explicit tutorial mode
//! SHOWCASE_MODE=true cargo run
//!
//! # Specific scenario
//! SHOWCASE_MODE=true SANDBOX_SCENARIO=complex cargo run
//! ```

use petal_tongue_core::{GraphEngine, LayoutAlgorithm, PrimalHealthStatus, PrimalInfo, Properties};
use std::sync::{Arc, RwLock};
use tracing::{info, warn};

/// Tutorial mode manager - handles demonstration scenarios and graceful fallback
pub struct TutorialMode {
    /// Whether tutorial mode is explicitly enabled
    enabled: bool,
    /// Current scenario name
    scenario_name: String,
}

impl Default for TutorialMode {
    fn default() -> Self {
        Self::new()
    }
}

impl TutorialMode {
    /// Create new tutorial mode manager
    #[must_use]
    pub fn new() -> Self {
        // Check for explicit tutorial mode
        let enabled = std::env::var("SHOWCASE_MODE")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(false);

        let scenario_name = std::env::var("SANDBOX_SCENARIO")
            .unwrap_or_else(|_| "simple".to_string());

        if enabled {
            info!("🎭 TUTORIAL MODE ENABLED");
            info!("📚 Scenario: {}", scenario_name);
            info!("💡 This is demonstration data, not production primals");
        }

        Self {
            enabled,
            scenario_name,
        }
    }

    /// Check if tutorial mode is explicitly enabled
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get current scenario name
    #[must_use]
    pub fn scenario_name(&self) -> &str {
        &self.scenario_name
    }

    /// Load tutorial data into graph
    ///
    /// This populates the graph with demonstration scenarios for:
    /// - Learning how petalTongue works
    /// - Testing without full ecosystem
    /// - Showcasing capabilities
    pub fn load_into_graph(&self, graph: Arc<RwLock<GraphEngine>>, layout: LayoutAlgorithm) {
        use crate::sandbox_mock::{get_default_scenario, load_sandbox_scenario};

        info!("📦 Loading tutorial scenario: {}", self.scenario_name);
        info!("🌸 Seamless transition from awakening to tutorial experience");

        // Try to load requested scenario
        let scenario = match load_sandbox_scenario(&self.scenario_name) {
            Ok(s) => {
                info!("✅ Loaded tutorial scenario: {}", s.name);
                info!("📝 Description: {}", s.description);
                s
            }
            Err(e) => {
                warn!("Failed to load scenario '{}': {}", self.scenario_name, e);
                info!("Using default tutorial scenario");
                get_default_scenario()
            }
        };

        // Update graph with scenario data
        let mut graph = graph.write().expect("graph lock poisoned");
        *graph = GraphEngine::new();

        info!(
            "📊 Populating graph: {} primals, {} edges",
            scenario.primals.len(),
            scenario.edges.len()
        );

        // Add primals from scenario
        for primal in scenario.primals {
            graph.add_node(primal);
        }

        // Add edges from scenario
        for edge in scenario.edges {
            use petal_tongue_core::TopologyEdge;
            graph.add_edge(TopologyEdge {
                from: edge.from_id,
                to: edge.to_id,
                edge_type: edge.edge_type,
                label: None,
            });
        }

        // Apply layout
        graph.set_layout(layout);
        graph.layout(100);

        info!("✅ Tutorial data loaded successfully");
        info!("🎓 Tutorial mode active - explore the sandbox!");
    }

    /// Create fallback scenario when discovery fails
    ///
    /// This is a graceful degradation - when no real primals are found,
    /// we provide a minimal working example so the user can still see
    /// how petalTongue works.
    pub fn create_fallback_scenario(graph: Arc<RwLock<GraphEngine>>, layout: LayoutAlgorithm) {
        warn!("🎭 GRACEFUL FALLBACK: No primals discovered");
        info!("💡 Loading minimal tutorial scenario so you can explore petalTongue");
        info!("📚 To see your real primals, ensure they're running and discoverable");

        let mut graph = graph.write().expect("graph lock poisoned");
        *graph = GraphEngine::new();

        // Create minimal example with 3 primals
        Self::populate_minimal_example(&mut graph);

        // Apply layout
        graph.set_layout(layout);
        graph.layout(100);

        info!("✅ Fallback scenario ready");
    }

    /// Populate a minimal working example (3 primals, 2 connections)
    ///
    /// This shows the basic concepts without overwhelming the user.
    fn populate_minimal_example(graph: &mut GraphEngine) {
        info!("📦 Creating minimal tutorial example...");

        // Add petalTongue (self-awareness!)
        graph.add_node(PrimalInfo {
            id: "petaltongue-tutorial".to_string(),
            name: "petalTongue (You Are Here)".to_string(),
            primal_type: "Visualization".to_string(),
            endpoint: "http://localhost:3030".to_string(),
            capabilities: vec![
                "visual_2d".to_string(),
                "audio_sonification".to_string(),
                "multi_modal_representation".to_string(),
            ],
            health: PrimalHealthStatus::Healthy,
            last_seen: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            properties: {
                let mut props = Properties::new();
                props.insert(
                    "tutorial_note".to_string(),
                    petal_tongue_core::PropertyValue::String(
                        "This is tutorial data. Start real primals to see actual topology."
                            .to_string(),
                    ),
                );
                props
            },
            #[allow(deprecated)]
            trust_level: None,
            #[allow(deprecated)]
            family_id: None,
        });

        // Add example Security primal
        graph.add_node(PrimalInfo {
            id: "beardog-tutorial".to_string(),
            name: "BearDog (Security)".to_string(),
            primal_type: "Security".to_string(),
            endpoint: "http://localhost:8001".to_string(),
            capabilities: vec![
                "authentication".to_string(),
                "authorization".to_string(),
                "encryption".to_string(),
            ],
            health: PrimalHealthStatus::Healthy,
            last_seen: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            properties: Properties::new(),
            #[allow(deprecated)]
            trust_level: None,
            #[allow(deprecated)]
            family_id: None,
        });

        // Add example Discovery primal
        graph.add_node(PrimalInfo {
            id: "songbird-tutorial".to_string(),
            name: "Songbird (Discovery)".to_string(),
            primal_type: "Discovery".to_string(),
            endpoint: "http://localhost:8003".to_string(),
            capabilities: vec![
                "service_discovery".to_string(),
                "capability_matching".to_string(),
            ],
            health: PrimalHealthStatus::Healthy,
            last_seen: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            properties: Properties::new(),
            #[allow(deprecated)]
            trust_level: None,
            #[allow(deprecated)]
            family_id: None,
        });

        // Add connections
        graph.add_edge(petal_tongue_core::TopologyEdge {
            from: "petaltongue-tutorial".to_string(),
            to: "songbird-tutorial".to_string(),
            edge_type: "discovers_via".to_string(),
            label: Some("Discovery".to_string()),
        });

        graph.add_edge(petal_tongue_core::TopologyEdge {
            from: "songbird-tutorial".to_string(),
            to: "beardog-tutorial".to_string(),
            edge_type: "finds".to_string(),
            label: Some("Service Discovery".to_string()),
        });

        info!("✅ Minimal example created: 3 primals, 2 connections");
    }
}

/// Check if we should use tutorial mode as a fallback
///
/// Returns true if:
/// - Discovery returned no providers
/// - User hasn't explicitly disabled fallback
#[must_use]
pub fn should_fallback(providers_found: usize) -> bool {
    if providers_found > 0 {
        return false;
    }

    // Check if user explicitly disabled fallback
    let disable_fallback = std::env::var("PETALTONGUE_NO_FALLBACK")
        .ok()
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(false);

    if disable_fallback {
        warn!("Fallback disabled by PETALTONGUE_NO_FALLBACK=true");
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
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
        TutorialMode::create_fallback_scenario(
            Arc::clone(&graph),
            LayoutAlgorithm::ForceDirected,
        );

        let graph = graph.read().unwrap();
        
        // Verify structure
        assert_eq!(graph.nodes().len(), 3, "Should have 3 tutorial primals");
        assert_eq!(graph.edges().len(), 2, "Should have 2 connections");
    }

    #[test]
    fn test_minimal_example_node_ids() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        TutorialMode::create_fallback_scenario(
            Arc::clone(&graph),
            LayoutAlgorithm::ForceDirected,
        );

        let graph = graph.read().unwrap();
        let nodes = graph.nodes();
        
        // Verify tutorial nodes have correct IDs
        let ids: Vec<&str> = nodes.iter().map(|n| n.info.id.as_str()).collect();
        assert!(ids.contains(&"petaltongue-tutorial"));
        assert!(ids.contains(&"beardog-tutorial"));
        assert!(ids.contains(&"songbird-tutorial"));
    }

    #[test]
    fn test_minimal_example_node_health() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        TutorialMode::create_fallback_scenario(
            Arc::clone(&graph),
            LayoutAlgorithm::ForceDirected,
        );

        let graph = graph.read().unwrap();
        
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
        TutorialMode::create_fallback_scenario(
            Arc::clone(&graph),
            LayoutAlgorithm::ForceDirected,
        );

        let graph = graph.read().unwrap();
        
        // Find petalTongue node (self-awareness)
        let petal_node = graph
            .nodes()
            .iter()
            .find(|n| n.info.id == "petaltongue-tutorial");
        
        assert!(petal_node.is_some(), "Should include petalTongue itself");
        
        let petal_node = petal_node.unwrap();
        assert_eq!(petal_node.info.primal_type, "Visualization");
        assert!(petal_node.info.capabilities.contains(&"visual_2d".to_string()));
    }

    #[test]
    fn test_minimal_example_edges() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        TutorialMode::create_fallback_scenario(
            Arc::clone(&graph),
            LayoutAlgorithm::ForceDirected,
        );

        let graph = graph.read().unwrap();
        let edges = graph.edges();
        
        // Verify edge structure
        assert_eq!(edges.len(), 2);
        
        // Should connect petalTongue → songbird → beardog
        let has_petal_to_songbird = edges
            .iter()
            .any(|e| e.from == "petaltongue-tutorial" && e.to == "songbird-tutorial");
        assert!(
            has_petal_to_songbird,
            "Should have petalTongue → songbird edge"
        );
        
        let has_songbird_to_beardog = edges
            .iter()
            .any(|e| e.from == "songbird-tutorial" && e.to == "beardog-tutorial");
        assert!(
            has_songbird_to_beardog,
            "Should have songbird → beardog edge"
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
            
            let graph_read = graph.read().unwrap();
            assert!(!graph_read.nodes().is_empty(), "Should load nodes with {:?}", layout);
            drop(graph_read); // Release lock for next iteration
        }
    }

    #[test]
    fn test_tutorial_properties() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        TutorialMode::create_fallback_scenario(
            Arc::clone(&graph),
            LayoutAlgorithm::ForceDirected,
        );

        let graph = graph.read().unwrap();
        
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
    fn test_fallback_creates_valid_graph() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        TutorialMode::create_fallback_scenario(
            Arc::clone(&graph),
            LayoutAlgorithm::ForceDirected,
        );

        let graph = graph.read().unwrap();
        
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
}

