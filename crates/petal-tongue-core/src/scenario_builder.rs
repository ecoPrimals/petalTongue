// SPDX-License-Identifier: AGPL-3.0-or-later
//! `ScenarioBuilder` trait for springs and primals to produce visualization data.
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
    #[must_use]
    pub const fn new(metadata: ScenarioMetadata) -> Self {
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
    ///
    /// # Errors
    ///
    /// Returns an error if the scene cannot be serialized to JSON.
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
    fn id(&self) -> &'static str;

    /// Human-readable name.
    fn name(&self) -> &'static str;

    /// Domain this builder belongs to (for theme selection).
    fn domain(&self) -> &'static str;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scenario_metadata_construction() {
        let meta = ScenarioMetadata {
            title: "Test Scenario".to_string(),
            description: "A test".to_string(),
            version: "1.0.0".to_string(),
            domain: "health".to_string(),
        };
        assert_eq!(meta.title, "Test Scenario");
        assert_eq!(meta.domain, "health");
    }

    #[test]
    fn scenario_metadata_default() {
        let meta = ScenarioMetadata::default();
        assert!(meta.title.is_empty());
        assert!(meta.description.is_empty());
        assert!(meta.version.is_empty());
        assert!(meta.domain.is_empty());
    }

    #[test]
    fn visualization_scene_new() {
        let meta = ScenarioMetadata {
            title: "Scene".to_string(),
            ..Default::default()
        };
        let scene = VisualizationScene::new(meta);
        assert_eq!(scene.metadata.title, "Scene");
        assert!(scene.bindings.is_empty());
        assert!(scene.thresholds.is_empty());
    }

    #[test]
    fn visualization_scene_with_binding() {
        let meta = ScenarioMetadata::default();
        let binding = DataBinding::TimeSeries {
            id: "ts1".to_string(),
            label: "Metric".to_string(),
            x_label: "Time".to_string(),
            y_label: "Value".to_string(),
            unit: "mg/dL".to_string(),
            x_values: vec![0.0, 1.0],
            y_values: vec![100.0, 110.0],
        };
        let scene = VisualizationScene::new(meta).with_binding(binding);
        assert_eq!(scene.bindings.len(), 1);
    }

    #[test]
    fn visualization_scene_with_threshold() {
        let meta = ScenarioMetadata::default();
        let threshold = ThresholdRange {
            label: "Normal".to_string(),
            min: 70.0,
            max: 100.0,
            status: "normal".to_string(),
        };
        let scene = VisualizationScene::new(meta).with_threshold(threshold);
        assert_eq!(scene.thresholds.len(), 1);
    }

    #[test]
    fn visualization_scene_to_json() {
        let meta = ScenarioMetadata {
            title: "JSON Test".to_string(),
            description: "Desc".to_string(),
            version: "2.0.0".to_string(),
            domain: "physics".to_string(),
        };
        let binding = DataBinding::Gauge {
            id: "g1".to_string(),
            label: "Temp".to_string(),
            value: 50.0,
            min: 0.0,
            max: 100.0,
            unit: "°C".to_string(),
            normal_range: [20.0, 80.0],
            warning_range: [0.0, 100.0],
        };
        let scene = VisualizationScene::new(meta)
            .with_binding(binding)
            .with_threshold(ThresholdRange {
                label: "OK".to_string(),
                min: 0.0,
                max: 100.0,
                status: "ok".to_string(),
            });
        let json = scene.to_json().unwrap();
        assert!(json.contains("\"title\": \"JSON Test\""));
        assert!(json.contains("\"domain\": \"physics\""));
        assert!(json.contains("\"gauge\""));
        assert!(json.contains("\"bindings\""));
        assert!(json.contains("\"thresholds\""));
    }
}
