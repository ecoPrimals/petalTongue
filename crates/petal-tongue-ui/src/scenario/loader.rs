// SPDX-License-Identifier: AGPL-3.0-only
//! Scenario loader and validation logic
//!
//! Handles loading scenario JSON files and validating their contents.

use anyhow::{Context, Result};
use std::path::Path;

use crate::scenario::types::Scenario;

impl Scenario {
    /// Load scenario from JSON file with validation
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read scenario file: {}", path.display()))?;

        let scenario: Self = serde_json::from_str(&contents)
            .with_context(|| format!("Failed to parse scenario JSON: {}", path.display()))?;

        // ✅ Explicit validation
        scenario
            .validate()
            .with_context(|| format!("Scenario validation failed: {}", path.display()))?;

        tracing::info!(
            "📋 Loaded scenario: {} ({})",
            scenario.name,
            scenario.version
        );
        tracing::info!("   Mode: {}", scenario.mode);
        tracing::info!("   Primals: {}", scenario.ecosystem.primals.len());

        Ok(scenario)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn workspace_scenario_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .expect("workspace root")
            .join("sandbox/scenarios")
            .join(name)
    }

    #[test]
    fn load_from_path() {
        let path = workspace_scenario_path("paint-simple.json");
        let scenario = Scenario::load(&path).unwrap();
        assert_eq!(scenario.name, "Paint Test - Ultra Simple");
        assert_eq!(scenario.mode, "paint-canvas");
    }

    #[test]
    fn load_path_as_ref() {
        let path = workspace_scenario_path("paint-simple.json");
        let scenario = Scenario::load(path.as_path()).unwrap();
        assert!(!scenario.name.is_empty());
    }

    #[test]
    fn load_nonexistent_fails() {
        let result = Scenario::load("/nonexistent/path/scenario.json");
        assert!(result.is_err());
    }
}
