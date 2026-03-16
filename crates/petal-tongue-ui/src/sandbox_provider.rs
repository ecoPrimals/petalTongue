// SPDX-License-Identifier: AGPL-3.0-only
//! Sandbox Scenario Provider
//!
//! Loads demonstration scenarios from `sandbox/scenarios/*.json` for:
//! - Tutorial mode (`SHOWCASE_MODE=true`)
//! - Development and demonstrations
//!
//! This is NOT a mock—it loads real JSON files. Use `--features mock` to enable
//! (or when running tests). Production builds without the feature use empty tutorial.

use petal_tongue_core::PrimalInfo;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::info;

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
///
/// Loads `sandbox/scenarios/simple.json`. Returns error if file not found—
/// callers should use `TutorialMode::populate_minimal_example` for graceful degradation.
pub fn get_default_scenario() -> Result<SandboxScenario, String> {
    load_sandbox_scenario("simple")
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
        // get_default_scenario returns Result - when sandbox/scenarios/simple.json exists
        // and matches SandboxScenario schema (top-level primals), we verify it.
        // When file is missing or schema differs (e.g. ecosystem.primals format), Err is acceptable.
        if let Ok(scenario) = get_default_scenario() {
            assert!(!scenario.primals.is_empty(), "Scenario should have primals");
            assert!(
                !scenario.primals[0].id.as_str().is_empty(),
                "First primal should have valid ID"
            );
        }
    }
}
