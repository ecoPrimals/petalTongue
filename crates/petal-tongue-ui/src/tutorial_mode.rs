// SPDX-License-Identifier: AGPL-3.0-or-later
//! Tutorial Mode - Demonstration & Graceful Degradation
//!
//! Provides standalone demonstration data and scenarios when:
//! - `SHOWCASE_MODE=true` (explicit tutorial/demo mode)
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

        let scenario_name =
            std::env::var("SANDBOX_SCENARIO").unwrap_or_else(|_| "simple".to_string());

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

    /// Create a disabled tutorial mode (for scenario mode)
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            scenario_name: "scenario".to_string(),
        }
    }

    /// Check if tutorial mode is explicitly enabled
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
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
        #[cfg(any(test, feature = "mock"))]
        {
            use crate::sandbox_provider::{get_default_scenario, load_sandbox_scenario};

            info!("📦 Loading tutorial scenario: {}", self.scenario_name);
            info!("🌸 Seamless transition from awakening to tutorial experience");

            // Try to load requested scenario, then simple.json, else use minimal example
            let scenario = match load_sandbox_scenario(&self.scenario_name) {
                Ok(s) => {
                    info!("✅ Loaded tutorial scenario: {}", s.name);
                    info!("📝 Description: {}", s.description);
                    s
                }
                Err(e) => {
                    warn!("Failed to load scenario '{}': {}", self.scenario_name, e);
                    match get_default_scenario() {
                        Ok(s) => {
                            info!("Using default scenario: {}", s.name);
                            s
                        }
                        Err(e2) => {
                            warn!("Sandbox scenarios unavailable: {}", e2);
                            info!("Using minimal example (no sandbox files found)");
                            let mut graph = graph
                                .write()
                                .unwrap_or_else(std::sync::PoisonError::into_inner);
                            *graph = GraphEngine::new();
                            Self::populate_minimal_example(&mut graph);
                            graph.set_layout(layout);
                            graph.layout(100);
                            info!("✅ Tutorial data loaded (minimal example)");
                            return;
                        }
                    }
                }
            };

            // Update graph with scenario data
            let mut graph = graph
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
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
                    from: edge.from_id.into(),
                    to: edge.to_id.into(),
                    edge_type: edge.edge_type,
                    label: None,
                    capability: None,
                    metrics: None,
                });
            }

            // Apply layout
            graph.set_layout(layout);
            graph.layout(100);

            info!("✅ Tutorial data loaded successfully");
            info!("🎓 Tutorial mode active - explore the sandbox!");
        }
        #[cfg(not(any(test, feature = "mock")))]
        {
            warn!("📚 Tutorial mode: mock feature disabled - using minimal example");
            info!("💡 Start with --features mock for sandbox scenarios");
            let mut graph = graph
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            *graph = GraphEngine::new();
            Self::populate_minimal_example(&mut graph);
            graph.set_layout(layout);
            graph.layout(100);
            drop(graph);
            info!("✅ Tutorial data loaded (minimal example)");
        }
    }

    /// Create fallback scenario when discovery fails
    ///
    /// This is a graceful degradation - when no real primals are found,
    /// we provide a minimal working example so the user can still perceive
    /// how petalTongue works.
    pub fn create_fallback_scenario(graph: Arc<RwLock<GraphEngine>>, layout: LayoutAlgorithm) {
        warn!("🎭 GRACEFUL FALLBACK: No primals discovered");
        info!("💡 Loading minimal tutorial scenario so you can explore petalTongue");
        info!("📚 To perceive your real primals, ensure they're running and discoverable");

        let mut graph = graph
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *graph = GraphEngine::new();

        // Create minimal example with 3 primals
        Self::populate_minimal_example(&mut graph);

        // Apply layout
        graph.set_layout(layout);
        graph.layout(100);
        drop(graph);

        info!("✅ Fallback scenario ready");
    }

    /// Populate a minimal working example (3 primals, 2 connections)
    ///
    /// Reflects the real ecoPrimals architecture: Unix socket IPC,
    /// capability-based discovery, and semantic method naming.
    fn populate_minimal_example(graph: &mut GraphEngine) {
        info!("Creating minimal tutorial example...");

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let socket_dir = petal_tongue_core::system_info::get_user_runtime_dir()
            .to_string_lossy()
            .into_owned();

        graph.add_node(PrimalInfo {
            id: "petaltongue-tutorial".into(),
            name: petal_tongue_core::constants::PRIMAL_NAME.to_string(),
            primal_type: "Visualization".to_string(),
            endpoint: format!("unix://{socket_dir}/biomeos/petaltongue.sock"),
            capabilities: vec![
                "ui.render".to_string(),
                "ui.visualization".to_string(),
                "ui.audio".to_string(),
                "ipc.unix-socket".to_string(),
                "ipc.json-rpc".to_string(),
            ],
            health: PrimalHealthStatus::Healthy,
            last_seen: now,
            endpoints: None,
            metadata: None,
            properties: {
                let mut props = Properties::new();
                props.insert(
                    "tutorial_note".to_string(),
                    petal_tongue_core::PropertyValue::String(
                        "Tutorial data. Start real primals to see actual topology.".to_string(),
                    ),
                );
                props
            },
        });

        graph.add_node(PrimalInfo {
            id: "security-example".into(),
            name: "Security Primal".to_string(),
            primal_type: "Security".to_string(),
            endpoint: "discovered-at-runtime".to_string(),
            capabilities: vec![
                "security.auth".to_string(),
                "security.signing".to_string(),
                "security.encryption".to_string(),
                "security.identity".to_string(),
            ],
            health: PrimalHealthStatus::Healthy,
            last_seen: now,
            endpoints: None,
            metadata: None,
            properties: Properties::new(),
        });

        graph.add_node(PrimalInfo {
            id: "discovery-example".into(),
            name: "Discovery Primal".to_string(),
            primal_type: "Discovery".to_string(),
            endpoint: "discovered-at-runtime".to_string(),
            capabilities: vec![
                "discovery.primals".to_string(),
                "discovery.services".to_string(),
                "orchestration.workflow".to_string(),
            ],
            health: PrimalHealthStatus::Healthy,
            last_seen: now,
            endpoints: None,
            metadata: None,
            properties: Properties::new(),
        });

        graph.add_edge(petal_tongue_core::TopologyEdge {
            from: "petaltongue-tutorial".into(),
            to: "discovery-example".into(),
            edge_type: "ipc.discovery".to_string(),
            label: Some("discovers".to_string()),
            capability: None,
            metrics: None,
        });

        graph.add_edge(petal_tongue_core::TopologyEdge {
            from: "discovery-example".into(),
            to: "security-example".into(),
            edge_type: "ipc.trust".to_string(),
            label: Some("authenticates".to_string()),
            capability: None,
            metrics: None,
        });

        info!("Minimal example created: 3 primals, 2 connections");
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
#[path = "tutorial_mode_tests.rs"]
mod tests;
