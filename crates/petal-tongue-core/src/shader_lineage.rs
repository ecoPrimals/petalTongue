// SPDX-License-Identifier: AGPL-3.0-only
//! Cross-spring shader lineage tracking and visualization.
//!
//! Tracks the evolution of shaders across the spring ecosystem:
//! - Which spring contributed which shader
//! - Delegation chains (spring -> barraCuda -> toadStool -> coralReef)
//! - Validation status per spring
//!
//! Used by petalTongue to render a lineage graph showing shader provenance.

use serde::{Deserialize, Serialize};

use crate::scenario_builder::{ScenarioBuilder, ScenarioMetadata, VisualizationScene};
use crate::{DataBinding, ThresholdRange};

/// Validation status of a shader at a specific point in the lineage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShaderValidationStatus {
    /// All validation checks passed.
    Passed,
    /// Some checks failed.
    Failed,
    /// Validation not yet run.
    Pending,
    /// Validation skipped (e.g., not applicable for this stage).
    Skipped,
}

/// A single node in the shader lineage graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderLineageNode {
    /// Unique identifier for this node.
    pub id: String,
    /// Human-readable label.
    pub label: String,
    /// Spring or system that owns this node.
    pub origin: String,
    /// Shader language/format (e.g., "WGSL", "SPIR-V", "GLSL").
    pub shader_format: String,
    /// Number of validation checks run.
    pub validation_checks: u32,
    /// Number of validation checks that passed.
    pub validation_passed: u32,
    /// Overall validation status.
    pub validation_status: ShaderValidationStatus,
}

/// A delegation edge in the shader lineage graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderDelegation {
    /// Source node ID.
    pub from: String,
    /// Target node ID.
    pub to: String,
    /// Delegation type (e.g., "compile", "optimize", "dispatch", "validate").
    pub delegation_type: String,
}

/// Complete shader lineage for a set of springs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShaderLineage {
    /// Nodes in the lineage graph.
    pub nodes: Vec<ShaderLineageNode>,
    /// Delegation edges.
    pub delegations: Vec<ShaderDelegation>,
}

impl ShaderLineage {
    /// Create an empty lineage.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node to the lineage.
    pub fn add_node(&mut self, node: ShaderLineageNode) {
        self.nodes.push(node);
    }

    /// Add a delegation edge.
    pub fn add_delegation(&mut self, delegation: ShaderDelegation) {
        self.delegations.push(delegation);
    }

    /// Get the validation pass rate across all nodes (0.0..=1.0).
    #[must_use]
    pub fn overall_pass_rate(&self) -> f64 {
        let total_checks: u32 = self.nodes.iter().map(|n| n.validation_checks).sum();
        let total_passed: u32 = self.nodes.iter().map(|n| n.validation_passed).sum();
        if total_checks == 0 {
            return 1.0;
        }
        f64::from(total_passed) / f64::from(total_checks)
    }

    /// Get all nodes from a specific spring origin.
    #[must_use]
    pub fn nodes_by_origin(&self, origin: &str) -> Vec<&ShaderLineageNode> {
        self.nodes.iter().filter(|n| n.origin == origin).collect()
    }

    /// Convert to `DataBinding` representations for visualization.
    #[must_use]
    pub fn to_bindings(&self) -> Vec<DataBinding> {
        let mut bindings = Vec::new();

        // Per-spring validation bar chart
        let mut origins: Vec<String> = self.nodes.iter().map(|n| n.origin.clone()).collect();
        origins.sort();
        origins.dedup();

        if !origins.is_empty() {
            let values: Vec<f64> = origins
                .iter()
                .map(|origin| {
                    let nodes = self.nodes_by_origin(origin);
                    let total: u32 = nodes.iter().map(|n| n.validation_checks).sum();
                    let passed: u32 = nodes.iter().map(|n| n.validation_passed).sum();
                    if total == 0 {
                        1.0
                    } else {
                        f64::from(passed) / f64::from(total)
                    }
                })
                .collect();
            bindings.push(DataBinding::Bar {
                id: "shader_validation_by_spring".to_string(),
                label: "Shader Validation Pass Rate".to_string(),
                categories: origins.clone(),
                values,
                unit: "ratio".to_string(),
            });
        }

        // Per-stage heatmap (origins × delegation types)
        let mut delegation_types: Vec<String> = self
            .delegations
            .iter()
            .map(|d| d.delegation_type.clone())
            .collect();
        delegation_types.sort();
        delegation_types.dedup();

        if !origins.is_empty() && !delegation_types.is_empty() {
            let values: Vec<f64> = origins
                .iter()
                .flat_map(|origin| {
                    delegation_types.iter().map(move |dt| {
                        let count = self
                            .delegations
                            .iter()
                            .filter(|d| {
                                d.delegation_type == *dt
                                    && self
                                        .nodes
                                        .iter()
                                        .any(|n| n.id == d.from && n.origin == *origin)
                            })
                            .count();
                        #[expect(
                            clippy::cast_precision_loss,
                            reason = "delegation count fits within f64 mantissa"
                        )]
                        {
                            count as f64
                        }
                    })
                })
                .collect();
            bindings.push(DataBinding::Heatmap {
                id: "shader_delegation_matrix".to_string(),
                label: "Delegation Matrix".to_string(),
                x_labels: delegation_types,
                y_labels: origins,
                values,
                unit: "delegations".to_string(),
            });
        }

        bindings
    }
}

/// Builds a representative shader lineage for demonstration/testing.
///
/// Uses generic origin labels ("spring-a", "spring-b", "compiler", "dispatcher")
/// rather than hardcoded primal names. In production, origins come from runtime
/// discovery and are never hardcoded.
#[cfg(any(test, feature = "test-fixtures"))]
pub fn build_demo_lineage() -> ShaderLineage {
    let mut lineage = ShaderLineage::new();

    lineage.add_node(ShaderLineageNode {
        id: "source-plasma".to_string(),
        label: "Plasma Density".to_string(),
        origin: "spring-a".to_string(),
        shader_format: "WGSL".to_string(),
        validation_checks: 85,
        validation_passed: 85,
        validation_status: ShaderValidationStatus::Passed,
    });

    lineage.add_node(ShaderLineageNode {
        id: "source-df64".to_string(),
        label: "DF64 Arithmetic".to_string(),
        origin: "spring-a".to_string(),
        shader_format: "WGSL".to_string(),
        validation_checks: 120,
        validation_passed: 118,
        validation_status: ShaderValidationStatus::Failed,
    });

    lineage.add_node(ShaderLineageNode {
        id: "source-seismic".to_string(),
        label: "Seismic Propagation".to_string(),
        origin: "spring-b".to_string(),
        shader_format: "WGSL".to_string(),
        validation_checks: 48,
        validation_passed: 48,
        validation_status: ShaderValidationStatus::Passed,
    });

    lineage.add_node(ShaderLineageNode {
        id: "compiler-stage".to_string(),
        label: "Shader Compiler".to_string(),
        origin: "compiler".to_string(),
        shader_format: "SPIR-V".to_string(),
        validation_checks: 200,
        validation_passed: 195,
        validation_status: ShaderValidationStatus::Failed,
    });

    lineage.add_node(ShaderLineageNode {
        id: "dispatch-stage".to_string(),
        label: "GPU Dispatch".to_string(),
        origin: "dispatcher".to_string(),
        shader_format: "SPIR-V".to_string(),
        validation_checks: 30,
        validation_passed: 30,
        validation_status: ShaderValidationStatus::Passed,
    });

    lineage.add_delegation(ShaderDelegation {
        from: "source-plasma".to_string(),
        to: "compiler-stage".to_string(),
        delegation_type: "compile".to_string(),
    });
    lineage.add_delegation(ShaderDelegation {
        from: "source-df64".to_string(),
        to: "compiler-stage".to_string(),
        delegation_type: "compile".to_string(),
    });
    lineage.add_delegation(ShaderDelegation {
        from: "source-seismic".to_string(),
        to: "compiler-stage".to_string(),
        delegation_type: "compile".to_string(),
    });
    lineage.add_delegation(ShaderDelegation {
        from: "compiler-stage".to_string(),
        to: "dispatch-stage".to_string(),
        delegation_type: "dispatch".to_string(),
    });

    lineage
}

/// Scenario builder for shader lineage visualization.
pub struct ShaderLineageScenario {
    lineage: ShaderLineage,
}

impl ShaderLineageScenario {
    /// Create with a specific lineage.
    #[must_use]
    pub const fn new(lineage: ShaderLineage) -> Self {
        Self { lineage }
    }

    /// Create with the built-in demo lineage (test/fixture use only).
    #[cfg(any(test, feature = "test-fixtures"))]
    #[must_use]
    pub fn demo() -> Self {
        Self::new(build_demo_lineage())
    }
}

impl ScenarioBuilder for ShaderLineageScenario {
    fn id(&self) -> &'static str {
        "petaltongue.shader_lineage"
    }

    fn name(&self) -> &'static str {
        "Cross-Spring Shader Lineage"
    }

    fn domain(&self) -> &'static str {
        "physics"
    }

    fn available_scenes(&self) -> Vec<String> {
        vec![
            "validation_summary".to_string(),
            "delegation_matrix".to_string(),
        ]
    }

    fn build_scene(&self, scene_name: &str) -> Option<VisualizationScene> {
        let meta = ScenarioMetadata {
            title: format!("Shader Lineage: {scene_name}"),
            description: "Cross-spring shader evolution pipeline".to_string(),
            version: "1.0.0".to_string(),
            domain: "physics".to_string(),
        };

        let bindings = self.lineage.to_bindings();

        match scene_name {
            "validation_summary" => {
                let binding = bindings
                    .into_iter()
                    .find(|b| matches!(b, DataBinding::Bar { id, .. } if id == "shader_validation_by_spring"))?;
                let scene = VisualizationScene::new(meta)
                    .with_binding(binding)
                    .with_threshold(ThresholdRange {
                        label: "Passing".to_string(),
                        min: 0.95,
                        max: 1.0,
                        status: "normal".to_string(),
                    })
                    .with_threshold(ThresholdRange {
                        label: "Degraded".to_string(),
                        min: 0.8,
                        max: 0.95,
                        status: "warning".to_string(),
                    });
                Some(scene)
            }
            "delegation_matrix" => {
                let binding = bindings
                    .into_iter()
                    .find(|b| matches!(b, DataBinding::Heatmap { id, .. } if id == "shader_delegation_matrix"))?;
                Some(VisualizationScene::new(meta).with_binding(binding))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_lineage() {
        let lineage = ShaderLineage::new();
        assert!(lineage.nodes.is_empty());
        assert!(lineage.delegations.is_empty());
        assert!((lineage.overall_pass_rate() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn demo_lineage_has_nodes_and_delegations() {
        let lineage = build_demo_lineage();
        assert_eq!(lineage.nodes.len(), 5);
        assert_eq!(lineage.delegations.len(), 4);
    }

    #[test]
    fn overall_pass_rate_calculation() {
        let lineage = build_demo_lineage();
        let rate = lineage.overall_pass_rate();
        assert!(
            rate > 0.9,
            "demo lineage should have >90% pass rate: {rate}"
        );
        assert!(rate < 1.0, "demo lineage has some failures: {rate}");
    }

    #[test]
    fn nodes_by_origin() {
        let lineage = build_demo_lineage();
        let origin_a = lineage.nodes_by_origin("spring-a");
        assert_eq!(origin_a.len(), 2);
        let origin_b = lineage.nodes_by_origin("spring-b");
        assert_eq!(origin_b.len(), 1);
    }

    #[test]
    fn to_bindings_produces_bar_and_heatmap() {
        let lineage = build_demo_lineage();
        let bindings = lineage.to_bindings();
        assert_eq!(bindings.len(), 2);
        assert!(
            bindings
                .iter()
                .any(|b| matches!(b, DataBinding::Bar { .. }))
        );
        assert!(
            bindings
                .iter()
                .any(|b| matches!(b, DataBinding::Heatmap { .. }))
        );
    }

    #[test]
    fn validation_bar_chart_categories() {
        let lineage = build_demo_lineage();
        let bindings = lineage.to_bindings();
        let bar = bindings
            .iter()
            .find(|b| matches!(b, DataBinding::Bar { .. }))
            .unwrap();
        match bar {
            DataBinding::Bar {
                categories, values, ..
            } => {
                assert!(categories.contains(&"spring-a".to_string()));
                assert!(categories.contains(&"spring-b".to_string()));
                assert!(categories.contains(&"compiler".to_string()));
                assert!(categories.contains(&"dispatcher".to_string()));
                assert_eq!(categories.len(), values.len());
            }
            _ => panic!("expected Bar"),
        }
    }

    #[test]
    fn scenario_builder_metadata() {
        let builder = ShaderLineageScenario::demo();
        assert_eq!(builder.id(), "petaltongue.shader_lineage");
        assert_eq!(builder.domain(), "physics");
        assert_eq!(builder.available_scenes().len(), 2);
    }

    #[test]
    fn scenario_builder_validation_summary() {
        let builder = ShaderLineageScenario::demo();
        let scene = builder.build_scene("validation_summary").unwrap();
        assert_eq!(scene.bindings.len(), 1);
        assert_eq!(scene.thresholds.len(), 2);
        assert!(matches!(&scene.bindings[0], DataBinding::Bar { .. }));
    }

    #[test]
    fn scenario_builder_delegation_matrix() {
        let builder = ShaderLineageScenario::demo();
        let scene = builder.build_scene("delegation_matrix").unwrap();
        assert_eq!(scene.bindings.len(), 1);
        assert!(matches!(&scene.bindings[0], DataBinding::Heatmap { .. }));
    }

    #[test]
    fn scenario_builder_unknown_scene() {
        let builder = ShaderLineageScenario::demo();
        assert!(builder.build_scene("nonexistent").is_none());
    }

    #[test]
    fn scenario_builder_build_all() {
        let builder = ShaderLineageScenario::demo();
        let scenes = builder.build_all();
        assert_eq!(scenes.len(), 2);
    }

    #[test]
    fn shader_validation_status_serde() {
        let statuses = [
            ShaderValidationStatus::Passed,
            ShaderValidationStatus::Failed,
            ShaderValidationStatus::Pending,
            ShaderValidationStatus::Skipped,
        ];
        for status in &statuses {
            let json = serde_json::to_string(status).unwrap();
            let parsed: ShaderValidationStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(*status, parsed);
        }
    }

    #[test]
    fn shader_lineage_node_serde() {
        let node = ShaderLineageNode {
            id: "test".to_string(),
            label: "Test".to_string(),
            origin: "spring".to_string(),
            shader_format: "WGSL".to_string(),
            validation_checks: 10,
            validation_passed: 9,
            validation_status: ShaderValidationStatus::Failed,
        };
        let json = serde_json::to_string(&node).unwrap();
        let parsed: ShaderLineageNode = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "test");
        assert_eq!(parsed.validation_passed, 9);
    }
}
