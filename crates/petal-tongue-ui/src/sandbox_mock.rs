// SPDX-License-Identifier: AGPL-3.0-only
//! Sandbox Mock Data Provider
//!
//! Loads mock data from sandbox/scenarios/ for demonstrations and testing

use petal_tongue_core::constants::{DEFAULT_SANDBOX_DISCOVERY_PORT, DEFAULT_SANDBOX_SECURITY_PORT};
use petal_tongue_core::{PrimalHealthStatus, PrimalInfo, Properties};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, warn};

/// Sandbox scenario for demonstrations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxScenario {
    /// Scenario name
    pub name: String,
    /// Scenario description
    pub description: String,
    /// Primals in this scenario
    pub primals: Vec<PrimalInfo>,
    /// Edges between primals
    #[serde(default)]
    pub edges: Vec<SandboxEdge>,
}

/// Edge in sandbox scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxEdge {
    /// Source primal ID
    pub from_id: String,
    /// Target primal ID
    pub to_id: String,
    /// Type of edge relationship
    pub edge_type: String,
}

/// Load sandbox scenario from file
pub fn load_sandbox_scenario(name: &str) -> Result<SandboxScenario, String> {
    // Find sandbox directory (relative to project root)
    let sandbox_path = find_sandbox_dir()?;
    let scenario_file = sandbox_path.join("scenarios").join(format!("{name}.json"));

    info!("📦 Loading sandbox scenario from: {:?}", scenario_file);

    if !scenario_file.exists() {
        return Err(format!(
            "Sandbox scenario not found: {}",
            scenario_file.display()
        ));
    }

    // Read and parse JSON
    let contents = std::fs::read_to_string(&scenario_file)
        .map_err(|e| format!("Failed to read scenario file: {e}"))?;

    let mut scenario: SandboxScenario = serde_json::from_str(&contents)
        .map_err(|e| format!("Failed to parse scenario JSON: {e}"))?;

    // Migrate deprecated fields (trust_level, family_id) to properties for adapter-based rendering
    for primal in &mut scenario.primals {
        primal.migrate_deprecated_fields();
    }

    info!(
        "✅ Loaded sandbox scenario '{}' with {} primals",
        scenario.name,
        scenario.primals.len()
    );

    Ok(scenario)
}

/// Find sandbox directory
fn find_sandbox_dir() -> Result<PathBuf, String> {
    // Try environment variable first
    if let Ok(sandbox) = std::env::var("PETALTONGUE_SANDBOX_DIR") {
        let path = PathBuf::from(sandbox);
        if path.exists() {
            return Ok(path);
        }
    }

    // Try relative to current directory
    let current_dir =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {e}"))?;

    // Try ./sandbox
    let sandbox = current_dir.join("sandbox");
    if sandbox.exists() {
        return Ok(sandbox);
    }

    // Try ../sandbox (if running from crates/)
    let sandbox = current_dir
        .parent()
        .ok_or("No parent directory")?
        .join("sandbox");
    if sandbox.exists() {
        return Ok(sandbox);
    }

    // Try ../../sandbox (if running from crates/petal-tongue-ui/)
    let sandbox = current_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or("No grandparent directory")?
        .join("sandbox");
    if sandbox.exists() {
        return Ok(sandbox);
    }

    Err(
        "Sandbox directory not found. Set PETALTONGUE_SANDBOX_DIR or run from project root."
            .to_string(),
    )
}

/// List available sandbox scenarios
#[must_use]
pub fn list_sandbox_scenarios() -> Vec<String> {
    let sandbox_dir = match find_sandbox_dir() {
        Ok(dir) => dir,
        Err(_) => return Vec::new(),
    };

    let scenarios_dir = sandbox_dir.join("scenarios");
    if !scenarios_dir.exists() {
        return Vec::new();
    }

    std::fs::read_dir(scenarios_dir)
        .ok()
        .map(|entries| {
            entries
                .filter_map(std::result::Result::ok)
                .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "json"))
                .filter_map(|entry| {
                    entry
                        .path()
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(String::from)
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Get default sandbox scenario (for showcase mode)
pub fn get_default_scenario() -> SandboxScenario {
    // Try to load simple.json
    if let Ok(scenario) = load_sandbox_scenario("simple") {
        return scenario;
    }

    // Fallback to hardcoded simple scenario
    warn!("⚠️  Using fallback scenario (sandbox/scenarios/simple.json not found)");

    SandboxScenario {
        name: "Fallback Simple".to_string(),
        description: "Basic 3-primal demonstration (fallback)".to_string(),
        primals: vec![
            PrimalInfo {
                id: "local".into(),
                name: "petalTongue (Local)".to_string(),
                primal_type: "Visualization".to_string(),
                endpoint: "self".to_string(),
                capabilities: vec!["visual".to_string(), "audio".to_string()],
                health: PrimalHealthStatus::Healthy,
                last_seen: chrono::Utc::now().timestamp() as u64,
                endpoints: None,
                metadata: None,
                properties: Properties::new(),
                #[expect(deprecated)]
                trust_level: None,
                #[expect(deprecated)]
                family_id: None,
            },
            PrimalInfo {
                id: "security".into(),
                name: "Security".to_string(),
                primal_type: "Security".to_string(),
                endpoint: std::env::var("PETALTONGUE_SANDBOX_SECURITY_ENDPOINT").unwrap_or_else(
                    |_| format!("http://localhost:{DEFAULT_SANDBOX_SECURITY_PORT}"),
                ),
                capabilities: vec!["authentication".to_string(), "encryption".to_string()],
                health: PrimalHealthStatus::Healthy,
                last_seen: chrono::Utc::now().timestamp() as u64,
                endpoints: None,
                metadata: None,
                properties: Properties::new(),
                #[expect(deprecated)]
                trust_level: None,
                #[expect(deprecated)]
                family_id: None,
            },
            PrimalInfo {
                id: "discovery".into(),
                name: "Discovery".to_string(),
                primal_type: "Orchestration".to_string(),
                endpoint: std::env::var("PETALTONGUE_SANDBOX_DISCOVERY_ENDPOINT").unwrap_or_else(
                    |_| format!("http://localhost:{DEFAULT_SANDBOX_DISCOVERY_PORT}"),
                ),
                capabilities: vec!["discovery".to_string(), "coordination".to_string()],
                health: PrimalHealthStatus::Healthy,
                last_seen: chrono::Utc::now().timestamp() as u64,
                endpoints: None,
                metadata: None,
                properties: Properties::new(),
                #[expect(deprecated)]
                trust_level: None,
                #[expect(deprecated)]
                family_id: None,
            },
        ],
        edges: vec![
            SandboxEdge {
                from_id: "local".to_string(),
                to_id: "security".to_string(),
                edge_type: "trust".to_string(),
            },
            SandboxEdge {
                from_id: "local".to_string(),
                to_id: "discovery".to_string(),
                edge_type: "discovery".to_string(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_scenarios() {
        let scenarios = list_sandbox_scenarios();
        // Should find at least the default scenarios
        assert!(!scenarios.is_empty() || find_sandbox_dir().is_err());
    }

    #[test]
    fn test_default_scenario() {
        let scenario = get_default_scenario();
        assert!(!scenario.primals.is_empty(), "Scenario should have primals");
        // First primal ID depends on which scenario loaded (simple.json vs fallback)
        assert!(
            scenario.primals[0].id.as_str().len() > 0,
            "First primal should have valid ID"
        );
    }
}
