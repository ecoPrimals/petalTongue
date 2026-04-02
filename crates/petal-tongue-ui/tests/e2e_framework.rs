// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! End-to-End Test Framework
//!
//! Provides infrastructure for full-stack integration testing of petalTongue.
//! Tests the complete flow from `BiomeOS` API → UI rendering → User interaction.

use anyhow::Result;
use petal_tongue_api::BiomeOSClient;
use petal_tongue_core::{GraphEngine, PrimalHealthStatus, PrimalInfo, TopologyEdge};
use std::sync::{Arc, RwLock};

/// E2E test configuration
pub struct E2ETestConfig {
    /// Use mock `BiomeOS` (true) or real instance (false)
    pub use_mock: bool,
    /// `BiomeOS` URL (if not using mock)
    pub biomeos_url: Option<String>,
    /// Test timeout in seconds
    pub timeout_secs: u64,
    /// Enable detailed logging
    pub verbose: bool,
}

impl Default for E2ETestConfig {
    fn default() -> Self {
        Self {
            use_mock: true,
            biomeos_url: None,
            timeout_secs: 30,
            verbose: false,
        }
    }
}

/// E2E test scenario result
#[derive(Debug)]
pub struct E2ETestResult {
    /// Test name
    pub name: String,
    /// Success status
    pub success: bool,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Steps completed
    pub steps_completed: usize,
    /// Total steps
    pub total_steps: usize,
}

impl E2ETestResult {
    /// Check if test passed
    #[must_use]
    pub const fn passed(&self) -> bool {
        self.success
    }
}

/// E2E test runner
pub struct E2ETestRunner {
    config: E2ETestConfig,
    results: Vec<E2ETestResult>,
}

impl E2ETestRunner {
    /// Create a new E2E test runner
    #[must_use]
    pub const fn new(config: E2ETestConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
        }
    }

    /// Run all E2E tests
    pub async fn run_all(&mut self) -> Result<()> {
        tracing::info!("Starting E2E test suite");

        // Test 1: Basic discovery cycle
        self.test_discovery_cycle().await?;

        // Test 2: Graph topology rendering
        self.test_topology_rendering().await?;

        // Test 3: Health status updates
        self.test_health_updates().await?;

        // Test 4: Edge creation and removal
        self.test_edge_lifecycle().await?;

        // Test 5: Capability detection
        self.test_capability_detection().await?;

        tracing::info!("E2E test suite completed");
        self.print_summary();

        Ok(())
    }

    /// Test 1: Full discovery cycle
    async fn test_discovery_cycle(&mut self) -> Result<()> {
        let start = std::time::Instant::now();
        let test_name = "discovery_cycle";
        let total_steps = 4;
        let mut steps_completed = 0;

        let result: Result<()> = async {
            // Step 1: Create BiomeOS client
            let client =
                BiomeOSClient::new(petal_tongue_core::test_fixtures::endpoints::MOCK_BIOMEOS)
                    .with_fixture_mode(self.config.use_mock);
            steps_completed += 1;

            // Step 2: Discover primals
            let primals = client.discover_primals().await?;
            if primals.is_empty() {
                return Err(anyhow::anyhow!("No primals discovered"));
            }
            steps_completed += 1;

            // Step 3: Create graph and add primals
            let mut graph = GraphEngine::new();
            for primal in primals {
                graph.add_node(primal);
            }
            steps_completed += 1;

            // Step 4: Verify graph state
            if graph.nodes().is_empty() {
                return Err(anyhow::anyhow!("Graph has no nodes after discovery"));
            }
            steps_completed += 1;

            Ok(())
        }
        .await;

        #[expect(
            clippy::cast_possible_truncation,
            reason = "test duration in ms fits u64"
        )]
        let duration_ms = start.elapsed().as_millis() as u64;

        self.results.push(E2ETestResult {
            name: test_name.to_string(),
            success: result.is_ok(),
            duration_ms,
            error: match &result {
                Err(e) => Some(e.to_string()),
                Ok(()) => None,
            },
            steps_completed,
            total_steps,
        });

        result
    }

    /// Test 2: Graph topology rendering
    async fn test_topology_rendering(&mut self) -> Result<()> {
        let start = std::time::Instant::now();
        let test_name = "topology_rendering";
        let total_steps = 3;
        let mut steps_completed = 0;

        let result: Result<()> = async {
            // Step 1: Create graph with test data
            let mut graph = GraphEngine::new();
            graph.add_node(PrimalInfo {
                id: "test1".into(),
                name: "Test Primal 1".to_string(),
                primal_type: "TestType".to_string(),
                endpoint: "http://test:8080".to_string(),
                capabilities: vec!["test".to_string()],
                health: PrimalHealthStatus::Healthy,
                properties: petal_tongue_core::Properties::new(),
                last_seen: 0,
                endpoints: None,
                metadata: None,
            });
            steps_completed += 1;

            // Step 2: Add edge
            graph.add_edge(TopologyEdge {
                from: "test1".into(),
                to: "test1".into(),
                edge_type: "self".to_string(),
                label: None,
                capability: None,
                metrics: None,
            });
            steps_completed += 1;

            // Step 3: Verify topology
            if graph.nodes().len() != 1 || graph.edges().len() != 1 {
                return Err(anyhow::anyhow!("Topology not correctly constructed"));
            }
            steps_completed += 1;

            Ok(())
        }
        .await;

        #[expect(
            clippy::cast_possible_truncation,
            reason = "test duration in ms fits u64"
        )]
        let duration_ms = start.elapsed().as_millis() as u64;

        self.results.push(E2ETestResult {
            name: test_name.to_string(),
            success: result.is_ok(),
            duration_ms,
            error: match &result {
                Err(e) => Some(e.to_string()),
                Ok(()) => None,
            },
            steps_completed,
            total_steps,
        });

        result
    }

    /// Test 3: Health status updates
    async fn test_health_updates(&mut self) -> Result<()> {
        let start = std::time::Instant::now();
        let test_name = "health_updates";
        let total_steps = 3;
        let mut steps_completed = 0;

        let result: Result<()> = async {
            // Step 1: Create graph with primal
            let graph = Arc::new(RwLock::new(GraphEngine::new()));
            {
                let mut g = graph.write().unwrap();
                g.add_node(PrimalInfo {
                    id: "test1".into(),
                    name: "Test".to_string(),
                    primal_type: "Test".to_string(),
                    endpoint: "http://test:8080".to_string(),
                    capabilities: vec![],
                    health: PrimalHealthStatus::Healthy,
                    properties: petal_tongue_core::Properties::new(),
                    last_seen: 0,
                    endpoints: None,
                    metadata: None,
                });
            }
            steps_completed += 1;

            // Step 2: Update health status
            {
                let mut g = graph.write().unwrap();
                if let Some(node) = g.get_node_mut("test1") {
                    node.info.health = PrimalHealthStatus::Warning;
                }
            }
            steps_completed += 1;

            // Step 3: Verify update
            {
                let g = graph.read().unwrap();
                if let Some(node) = g.get_node("test1")
                    && node.info.health != PrimalHealthStatus::Warning
                {
                    return Err(anyhow::anyhow!("Health status not updated"));
                }
            }
            steps_completed += 1;

            Ok(())
        }
        .await;

        #[expect(
            clippy::cast_possible_truncation,
            reason = "test duration in ms fits u64"
        )]
        let duration_ms = start.elapsed().as_millis() as u64;

        self.results.push(E2ETestResult {
            name: test_name.to_string(),
            success: result.is_ok(),
            duration_ms,
            error: match &result {
                Err(e) => Some(e.to_string()),
                Ok(()) => None,
            },
            steps_completed,
            total_steps,
        });

        result
    }

    /// Test 4: Edge lifecycle
    async fn test_edge_lifecycle(&mut self) -> Result<()> {
        let start = std::time::Instant::now();
        let test_name = "edge_lifecycle";
        let total_steps = 4;
        let mut steps_completed = 0;

        let result: Result<()> = async {
            // Step 1: Create graph with nodes
            let mut graph = GraphEngine::new();
            graph.add_node(PrimalInfo {
                id: "node1".into(),
                name: "Node 1".to_string(),
                primal_type: "Test".to_string(),
                endpoint: "http://test:8080".to_string(),
                capabilities: vec![],
                health: PrimalHealthStatus::Healthy,
                properties: petal_tongue_core::Properties::new(),
                last_seen: 0,
                endpoints: None,
                metadata: None,
            });
            graph.add_node(PrimalInfo {
                id: "node2".into(),
                name: "Node 2".to_string(),
                primal_type: "Test".to_string(),
                endpoint: "http://test:8081".to_string(),
                capabilities: vec![],
                health: PrimalHealthStatus::Healthy,
                properties: petal_tongue_core::Properties::new(),
                last_seen: 0,
                endpoints: None,
                metadata: None,
            });
            steps_completed += 1;

            // Step 2: Add edge
            graph.add_edge(TopologyEdge {
                from: "node1".into(),
                to: "node2".into(),
                edge_type: "test".to_string(),
                label: Some("test edge".to_string()),
                capability: None,
                metrics: None,
            });
            if graph.edges().len() != 1 {
                return Err(anyhow::anyhow!("Edge not added"));
            }
            steps_completed += 1;

            // Step 3: Verify edge exists
            let edges: Vec<_> = graph
                .edges()
                .iter()
                .filter(|e| e.from == "node1" && e.to == "node2")
                .collect();
            if edges.is_empty() {
                return Err(anyhow::anyhow!("Edge not found"));
            }
            steps_completed += 1;

            // Step 4: Clear and verify
            graph.clear();
            if !graph.edges().is_empty() || !graph.nodes().is_empty() {
                return Err(anyhow::anyhow!("Graph not cleared"));
            }
            steps_completed += 1;

            Ok(())
        }
        .await;

        #[expect(
            clippy::cast_possible_truncation,
            reason = "test duration in ms fits u64"
        )]
        let duration_ms = start.elapsed().as_millis() as u64;

        self.results.push(E2ETestResult {
            name: test_name.to_string(),
            success: result.is_ok(),
            duration_ms,
            error: match &result {
                Err(e) => Some(e.to_string()),
                Ok(()) => None,
            },
            steps_completed,
            total_steps,
        });

        result
    }

    /// Test 5: Capability detection
    async fn test_capability_detection(&mut self) -> Result<()> {
        let start = std::time::Instant::now();
        let test_name = "capability_detection";
        let total_steps = 2;
        let mut steps_completed = 0;

        let result: Result<()> = async {
            // Step 1: Create capability detector
            let detector = petal_tongue_core::CapabilityDetector::default();
            steps_completed += 1;

            // Step 2: Verify capabilities detected
            let caps = detector.get_all();
            if caps.is_empty() {
                return Err(anyhow::anyhow!("No capabilities detected"));
            }
            steps_completed += 1;

            Ok(())
        }
        .await;

        #[expect(
            clippy::cast_possible_truncation,
            reason = "test duration in ms fits u64"
        )]
        let duration_ms = start.elapsed().as_millis() as u64;

        self.results.push(E2ETestResult {
            name: test_name.to_string(),
            success: result.is_ok(),
            duration_ms,
            error: match &result {
                Err(e) => Some(e.to_string()),
                Ok(()) => None,
            },
            steps_completed,
            total_steps,
        });

        result
    }

    /// Print test summary
    fn print_summary(&self) {
        let passed = self.results.iter().filter(|r| r.success).count();
        let failed = self.results.len() - passed;
        let total_duration: u64 = self.results.iter().map(|r| r.duration_ms).sum();

        tracing::info!("\n=== E2E Test Summary ===");
        tracing::info!("Total: {}", self.results.len());
        tracing::info!("Passed: {passed}");
        tracing::info!("Failed: {failed}");
        tracing::info!("Duration: {total_duration}ms");
        tracing::info!("========================\n");

        for result in &self.results {
            let status = if result.success {
                "✅ PASS"
            } else {
                "❌ FAIL"
            };
            tracing::info!(
                "{status} {} ({}/{}): {}ms",
                result.name,
                result.steps_completed,
                result.total_steps,
                result.duration_ms
            );
            if let Some(ref error) = result.error {
                tracing::error!("  Error: {error}");
            }
        }
    }

    /// Get test results
    #[must_use]
    pub fn results(&self) -> &[E2ETestResult] {
        &self.results
    }

    /// Check if all tests passed
    #[must_use]
    pub fn all_passed(&self) -> bool {
        self.results.iter().all(|r| r.success)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_e2e_runner_creation() {
        let config = E2ETestConfig::default();
        let runner = E2ETestRunner::new(config);
        assert_eq!(runner.results.len(), 0);
    }

    #[cfg(feature = "mock")]
    #[tokio::test]
    async fn test_e2e_runner_with_mock() {
        let config = E2ETestConfig {
            use_mock: true,
            ..Default::default()
        };
        let mut runner = E2ETestRunner::new(config);

        // Run all tests
        let result = runner.run_all().await;
        assert!(result.is_ok());

        // Check results
        assert!(!runner.results.is_empty());
        let passed = runner.results.iter().filter(|r| r.passed()).count();
        assert!(passed > 0, "At least some tests should pass");
    }
}
