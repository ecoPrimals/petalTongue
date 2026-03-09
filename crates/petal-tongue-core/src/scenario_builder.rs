// SPDX-License-Identifier: AGPL-3.0-only
//! ScenarioBuilder trait for springs and primals to produce visualization data.
//!
//! Any data source (spring, primal, or external tool) can implement this trait
//! to provide structured data for petalTongue visualization.

use crate::{DataBinding, ThresholdRange};

/// Metadata about a visualization scenario.
#[derive(Debug, Clone, Default)]
pub struct ScenarioMetadata {
    /// Human-readable title.
    pub title: String,
    /// Description of what this scenario shows.
    pub description: String,
    /// Semantic version (e.g., "1.0.0").
    pub version: String,
    /// Domain hint for theme selection (e.g., "health", "physics", "ecology", "measurement").
    pub domain: String,
}

/// A complete visualization scene produced by a builder.
#[derive(Debug, Clone)]
pub struct VisualizationScene {
    /// Scenario metadata.
    pub metadata: ScenarioMetadata,
    /// Data bindings to render.
    pub bindings: Vec<DataBinding>,
    /// Threshold ranges for status coloring.
    pub thresholds: Vec<ThresholdRange>,
}

impl VisualizationScene {
    /// Create a new empty scene with metadata.
    pub fn new(metadata: ScenarioMetadata) -> Self {
        Self {
            metadata,
            bindings: vec![],
            thresholds: vec![],
        }
    }

    /// Add a data binding.
    #[must_use]
    pub fn with_binding(mut self, binding: DataBinding) -> Self {
        self.bindings.push(binding);
        self
    }

    /// Add a threshold range.
    #[must_use]
    pub fn with_threshold(mut self, threshold: ThresholdRange) -> Self {
        self.thresholds.push(threshold);
        self
    }

    /// Serialize to JSON (for file-based fallback).
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&serde_json::json!({
            "title": self.metadata.title,
            "description": self.metadata.description,
            "version": self.metadata.version,
            "domain": self.metadata.domain,
            "bindings": self.bindings,
            "thresholds": self.thresholds,
        }))
    }
}

/// Trait for producing visualization data.
///
/// Springs implement this to provide domain-specific visualizations.
/// Each builder can produce one or more scenes.
pub trait ScenarioBuilder: Send + Sync {
    /// Unique identifier for this builder (e.g., "healthspring.pkpd", "hotspring.plasma").
    fn id(&self) -> &str;

    /// Human-readable name.
    fn name(&self) -> &str;

    /// Domain this builder belongs to (for theme selection).
    fn domain(&self) -> &str;

    /// List available scenes this builder can produce.
    fn available_scenes(&self) -> Vec<String>;

    /// Build a specific scene by name.
    fn build_scene(&self, scene_name: &str) -> Option<VisualizationScene>;

    /// Build all available scenes.
    fn build_all(&self) -> Vec<VisualizationScene> {
        self.available_scenes()
            .iter()
            .filter_map(|name| self.build_scene(name))
            .collect()
    }
}
