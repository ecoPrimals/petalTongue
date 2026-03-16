// SPDX-License-Identifier: AGPL-3.0-only
//! Scenario loader and validation logic
//!
//! Handles loading scenario JSON files and validating their contents.

use crate::error::Result;
use crate::scenario_error::ScenarioError;
use std::path::Path;

use crate::scenario::types::Scenario;

impl Scenario {
    /// Load scenario from JSON file with validation
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let contents = std::fs::read_to_string(path).map_err(ScenarioError::from)?;

        let scenario: Self = serde_json::from_str(&contents).map_err(ScenarioError::from)?;

        // ✅ Explicit validation
        scenario.validate()?;

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

    #[test]
    fn load_invalid_json_fails() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), "{ invalid json }").unwrap();
        let result = Scenario::load(temp.path());
        assert!(result.is_err());
        let err_str = format!("{:?}", result.unwrap_err());
        assert!(
            err_str.contains("parse") || err_str.contains("Json") || err_str.contains("expected"),
            "unexpected error: {err_str}"
        );
    }

    #[test]
    fn load_empty_json_fails() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), "{}").unwrap();
        let result = Scenario::load(temp.path());
        assert!(result.is_err());
    }

    #[test]
    fn load_malformed_missing_required_fields() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp.path(), r#"{"name": "x"}"#).unwrap();
        let result = Scenario::load(temp.path());
        assert!(result.is_err());
    }

    #[test]
    fn load_valid_json_validation_fails() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(
            temp.path(),
            r#"{"name":"","description":"","version":"2.0.0","mode":"x","ui_config":{},"ecosystem":{},"neural_api":{},"sensory_config":{},"edges":[]}"#,
        )
        .unwrap();
        let result = Scenario::load(temp.path());
        assert!(result.is_err());
        let err_str = format!("{:?}", result.unwrap_err());
        assert!(
            err_str.contains("validation")
                || err_str.contains("Missing")
                || err_str.contains("empty")
                || err_str.contains("Scenario"),
            "unexpected error: {err_str}"
        );
    }
}
