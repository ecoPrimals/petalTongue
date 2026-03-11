// SPDX-License-Identifier: AGPL-3.0-only
//! Chaos Testing Framework
//!
//! Provides fault injection and resilience testing for petalTongue.
//! Tests system behavior under adverse conditions.

use petal_tongue_core::{GraphEngine, PrimalHealthStatus, PrimalInfo};
use std::sync::{Arc, RwLock};

/// Chaos test scenario
#[derive(Debug, Clone)]
pub enum ChaosScenario {
    /// Rapidly add and remove primals
    PrimalChurn { count: usize, iterations: usize },
    /// Simulate network partition
    NetworkPartition { duration_ms: u64 },
    /// Simulate memory pressure
    MemoryPressure { allocation_mb: usize },
    /// High update rate stress test
    HighUpdateRate {
        updates_per_sec: usize,
        duration_secs: u64,
    },
    /// Random health status changes
    RandomHealthChanges { changes: usize },
    /// Concurrent modification stress
    ConcurrentModification {
        threads: usize,
        ops_per_thread: usize,
    },
}

/// Chaos test result
#[derive(Debug)]
pub struct ChaosTestResult {
    /// Scenario name
    pub scenario: String,
    /// Did the system survive?
    pub survived: bool,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Error encountered (if any)
    pub error: Option<String>,
    /// Operations completed
    pub operations_completed: usize,
    /// Crashes or panics
    pub crashes: usize,
}

/// Chaos test runner
pub struct ChaosTestRunner {
    results: Vec<ChaosTestResult>,
}

impl Default for ChaosTestRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl ChaosTestRunner {
    /// Create a new chaos test runner
    #[must_use]
    pub const fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    /// Run all chaos scenarios
    pub fn run_all(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Starting chaos test suite");

        // Scenario 1: Primal churn
        self.test_primal_churn()?;

        // Scenario 2: High update rate
        self.test_high_update_rate()?;

        // Scenario 3: Random health changes
        self.test_random_health_changes()?;

        // Scenario 4: Concurrent modifications
        self.test_concurrent_modifications()?;

        tracing::info!("Chaos test suite completed");
        self.print_summary();

        Ok(())
    }

    /// Test: Primal churn (rapid add/remove)
    #[allow(clippy::unnecessary_wraps)]
    fn test_primal_churn(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();
        let scenario_name = "primal_churn";
        let count = 50;
        let iterations = 10;

        let result = std::panic::catch_unwind(|| {
            let mut operations = 0;
            let mut graph = GraphEngine::new();

            for iteration in 0..iterations {
                // Add primals
                for i in 0..count {
                    graph.add_node(PrimalInfo {
                        id: format!("churn_{iteration}_{i}").into(),
                        name: format!("Churn Primal {i}"),
                        primal_type: "ChurnTest".to_string(),
                        endpoint: format!("http://churn:808{i}"),
                        capabilities: vec![],
                        health: PrimalHealthStatus::Healthy,
                        endpoints: None,
                        metadata: None,
                        properties: petal_tongue_core::Properties::new(),
                        #[expect(deprecated)]
                        trust_level: None,
                        #[expect(deprecated)]
                        family_id: None,
                        last_seen: 0,
                    });
                    operations += 1;
                }

                // Verify
                assert!(
                    graph.nodes().len() >= count,
                    "Should have at least {count} nodes"
                );

                // Clear (simulating removal)
                graph.clear();
                operations += 1;

                // Verify empty
                assert_eq!(graph.nodes().len(), 0, "Graph should be empty after clear");
            }

            operations
        });

        #[expect(
            clippy::cast_possible_truncation,
            reason = "test duration in ms fits u64"
        )]
        let duration_ms = start.elapsed().as_millis() as u64;
        let (survived, error, final_ops) = match result {
            Ok(ops) => (true, None, ops),
            Err(e) => (false, Some(format!("Panic: {e:?}")), 0),
        };

        self.results.push(ChaosTestResult {
            scenario: scenario_name.to_string(),
            survived,
            duration_ms,
            error,
            operations_completed: final_ops,
            crashes: usize::from(!survived),
        });

        Ok(())
    }

    /// Test: High update rate
    #[allow(clippy::unnecessary_wraps)]
    fn test_high_update_rate(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();
        let scenario_name = "high_update_rate";
        let updates_per_sec = 1000;
        let duration_secs = 1;

        let result = std::panic::catch_unwind(|| {
            let mut operations = 0;
            let graph = Arc::new(RwLock::new(GraphEngine::new()));

            // Add initial node
            {
                let mut g = graph.write().unwrap();
                g.add_node(PrimalInfo {
                    id: "stress_test".into(),
                    name: "Stress Test Node".to_string(),
                    primal_type: "StressTest".to_string(),
                    endpoint: "http://stress:8080".to_string(),
                    capabilities: vec![],
                    health: PrimalHealthStatus::Healthy,
                    endpoints: None,
                    metadata: None,
                    properties: petal_tongue_core::Properties::new(),
                    #[expect(deprecated)]
                    trust_level: None,
                    #[expect(deprecated)]
                    family_id: None,
                    last_seen: 0,
                });
            }

            let target_updates = updates_per_sec * duration_secs;
            for _ in 0..target_updates {
                let mut g = graph.write().unwrap();
                if let Some(node) = g.get_node_mut("stress_test") {
                    // Toggle health status
                    node.info.health = match node.info.health {
                        PrimalHealthStatus::Healthy => PrimalHealthStatus::Warning,
                        _ => PrimalHealthStatus::Healthy,
                    };
                    operations += 1;
                }
            }

            operations
        });

        #[expect(
            clippy::cast_possible_truncation,
            reason = "test duration in ms fits u64"
        )]
        let duration_ms = start.elapsed().as_millis() as u64;
        let (survived, error, final_ops) = match result {
            Ok(ops) => (true, None, ops),
            Err(e) => (false, Some(format!("Panic: {e:?}")), 0),
        };

        self.results.push(ChaosTestResult {
            scenario: scenario_name.to_string(),
            survived,
            duration_ms,
            error,
            operations_completed: final_ops,
            crashes: usize::from(!survived),
        });

        Ok(())
    }

    /// Test: Random health changes
    #[allow(clippy::unnecessary_wraps)]
    fn test_random_health_changes(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();
        let scenario_name = "random_health_changes";
        let changes = 500;

        let result = std::panic::catch_unwind(|| {
            let mut operations = 0;
            let mut graph = GraphEngine::new();

            // Add test nodes
            for i in 0..10 {
                graph.add_node(PrimalInfo {
                    id: format!("health_test_{i}").into(),
                    name: format!("Health Test {i}"),
                    primal_type: "HealthTest".to_string(),
                    endpoint: format!("http://health:808{i}"),
                    capabilities: vec![],
                    health: PrimalHealthStatus::Healthy,
                    endpoints: None,
                    metadata: None,
                    properties: petal_tongue_core::Properties::new(),
                    #[expect(deprecated)]
                    trust_level: None,
                    #[expect(deprecated)]
                    family_id: None,
                    last_seen: 0,
                });
            }

            // Random health changes
            for i in 0..changes {
                let node_id = format!("health_test_{}", i % 10);
                if let Some(node) = graph.get_node_mut(&node_id) {
                    node.info.health = match i % 3 {
                        0 => PrimalHealthStatus::Healthy,
                        1 => PrimalHealthStatus::Warning,
                        _ => PrimalHealthStatus::Critical,
                    };
                    operations += 1;
                }
            }

            operations
        });

        #[expect(
            clippy::cast_possible_truncation,
            reason = "test duration in ms fits u64"
        )]
        let duration_ms = start.elapsed().as_millis() as u64;
        let (survived, error, final_ops) = match result {
            Ok(ops) => (true, None, ops),
            Err(e) => (false, Some(format!("Panic: {e:?}")), 0),
        };

        self.results.push(ChaosTestResult {
            scenario: scenario_name.to_string(),
            survived,
            duration_ms,
            error,
            operations_completed: final_ops,
            crashes: usize::from(!survived),
        });

        Ok(())
    }

    /// Test: Concurrent modifications
    #[allow(clippy::unnecessary_wraps)]
    fn test_concurrent_modifications(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();
        let scenario_name = "concurrent_modifications";
        let threads = 4;
        let ops_per_thread = 100;
        let total_ops = threads * ops_per_thread;

        let result = std::panic::catch_unwind(|| {
            let graph = Arc::new(RwLock::new(GraphEngine::new()));

            // Add initial nodes
            {
                let mut g = graph.write().unwrap();
                for i in 0..threads {
                    g.add_node(PrimalInfo {
                        id: format!("concurrent_{i}").into(),
                        name: format!("Concurrent {i}"),
                        primal_type: "ConcurrentTest".to_string(),
                        endpoint: format!("http://concurrent:808{i}"),
                        capabilities: vec![],
                        health: PrimalHealthStatus::Healthy,
                        endpoints: None,
                        metadata: None,
                        properties: petal_tongue_core::Properties::new(),
                        #[expect(deprecated)]
                        trust_level: None,
                        #[expect(deprecated)]
                        family_id: None,
                        last_seen: 0,
                    });
                }
            }

            // Spawn threads for concurrent access
            let handles: Vec<_> = (0..threads)
                .map(|thread_id| {
                    let graph_clone = Arc::clone(&graph);
                    std::thread::spawn(move || {
                        for _ in 0..ops_per_thread {
                            let mut g = graph_clone.write().unwrap();
                            let node_id = format!("concurrent_{thread_id}");
                            if let Some(node) = g.get_node_mut(&node_id) {
                                node.info.health = PrimalHealthStatus::Warning;
                            }
                        }
                    })
                })
                .collect();

            // Wait for all threads
            for handle in handles {
                handle.join().unwrap();
            }

            total_ops
        });

        #[expect(
            clippy::cast_possible_truncation,
            reason = "test duration in ms fits u64"
        )]
        let duration_ms = start.elapsed().as_millis() as u64;
        let (survived, error, final_ops) = match result {
            Ok(ops) => (true, None, ops),
            Err(e) => (false, Some(format!("Panic: {e:?}")), 0),
        };

        self.results.push(ChaosTestResult {
            scenario: scenario_name.to_string(),
            survived,
            duration_ms,
            error,
            operations_completed: final_ops,
            crashes: usize::from(!survived),
        });

        Ok(())
    }

    /// Print chaos test summary
    fn print_summary(&self) {
        let survived = self.results.iter().filter(|r| r.survived).count();
        let crashed = self.results.len() - survived;
        let total_ops: usize = self.results.iter().map(|r| r.operations_completed).sum();
        let total_duration: u64 = self.results.iter().map(|r| r.duration_ms).sum();

        tracing::info!("\n=== Chaos Test Summary ===");
        tracing::info!("Total Scenarios: {}", self.results.len());
        tracing::info!("Survived: {survived}");
        tracing::info!("Crashed: {crashed}");
        tracing::info!("Total Operations: {total_ops}");
        tracing::info!("Total Duration: {total_duration}ms");
        tracing::info!("==========================\n");

        for result in &self.results {
            let status = if result.survived {
                "✅ SURVIVED"
            } else {
                "❌ CRASHED"
            };
            tracing::info!(
                "{status} {} ({} ops): {}ms",
                result.scenario,
                result.operations_completed,
                result.duration_ms
            );
            if let Some(ref error) = result.error {
                tracing::error!("  Error: {error}");
            }
        }
    }

    /// Get test results
    #[must_use]
    pub fn results(&self) -> &[ChaosTestResult] {
        &self.results
    }

    /// Check if all scenarios survived
    #[must_use]
    pub fn all_survived(&self) -> bool {
        self.results.iter().all(|r| r.survived)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_chaos_runner_creation() {
        let runner = ChaosTestRunner::new();
        assert_eq!(runner.results.len(), 0);
    }

    #[tokio::test]
    async fn test_chaos_scenarios() {
        let mut runner = ChaosTestRunner::new();

        // Run all chaos tests
        let result = runner.run_all();
        assert!(result.is_ok());

        // Check that tests ran
        assert!(!runner.results.is_empty());

        // All scenarios should survive (no panics/crashes)
        assert!(runner.all_survived(), "All chaos scenarios should survive");
    }
}
