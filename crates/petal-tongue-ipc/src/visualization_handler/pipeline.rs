// SPDX-License-Identifier: AGPL-3.0-or-later
//! Multi-stage pipeline DAG support for spring visualization workflows.
//!
//! Springs like neuralSpring define composition pipelines as DAGs where each
//! stage produces data that feeds the next. petalTongue renders stages
//! progressively as they complete.

use petal_tongue_core::DataBinding;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A stage in a visualization pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStage {
    /// Unique stage identifier.
    pub id: String,
    /// Human-readable stage label.
    pub label: String,
    /// IDs of stages this stage depends on.
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Stage status.
    #[serde(default)]
    pub status: StageStatus,
    /// Data bindings produced by this stage (populated when complete).
    #[serde(default)]
    pub bindings: Vec<DataBinding>,
}

/// Pipeline stage execution status.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StageStatus {
    /// Waiting for dependencies.
    #[default]
    Pending,
    /// Currently executing.
    Running,
    /// Completed successfully.
    Complete,
    /// Failed.
    Failed,
    /// Skipped (dependency failed).
    Skipped,
}

/// Request for `visualization.pipeline.submit`: submit a pipeline DAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineSubmitRequest {
    /// Unique pipeline session ID.
    pub pipeline_id: String,
    /// Human-readable pipeline title.
    pub title: String,
    /// Pipeline stages (topological order preferred but not required).
    pub stages: Vec<PipelineStage>,
    /// Domain hint for theming.
    #[serde(default)]
    pub domain: Option<String>,
}

/// Response for `visualization.pipeline.submit`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineSubmitResponse {
    /// Pipeline ID (echoed back).
    pub pipeline_id: String,
    /// Number of stages accepted.
    pub stage_count: usize,
    /// Topological order of stage IDs.
    pub execution_order: Vec<String>,
}

/// Request for `visualization.pipeline.update`: update a stage's status/bindings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStageUpdateRequest {
    /// Pipeline ID.
    pub pipeline_id: String,
    /// Stage ID to update.
    pub stage_id: String,
    /// New status.
    pub status: StageStatus,
    /// Bindings produced (for Complete stages).
    #[serde(default)]
    pub bindings: Vec<DataBinding>,
}

/// Response for `visualization.pipeline.update`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStageUpdateResponse {
    /// Pipeline ID (echoed back).
    pub pipeline_id: String,
    /// Stage ID (echoed back).
    pub stage_id: String,
    /// Whether the update was accepted.
    pub accepted: bool,
    /// Overall pipeline progress (0.0 to 1.0).
    pub progress: f64,
}

/// Manages active pipeline sessions.
#[derive(Debug, Default)]
pub struct PipelineRegistry {
    pipelines: HashMap<String, PipelineSession>,
}

#[derive(Debug)]
struct PipelineSession {
    title: String,
    stages: Vec<PipelineStage>,
    execution_order: Vec<String>,
    domain: Option<String>,
}

impl PipelineRegistry {
    /// Create a new empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Submit a new pipeline DAG.
    pub fn submit(&mut self, req: PipelineSubmitRequest) -> PipelineSubmitResponse {
        let execution_order = topological_sort(&req.stages);
        let stage_count = req.stages.len();
        self.pipelines.insert(
            req.pipeline_id.clone(),
            PipelineSession {
                title: req.title,
                stages: req.stages,
                execution_order: execution_order.clone(),
                domain: req.domain,
            },
        );
        PipelineSubmitResponse {
            pipeline_id: req.pipeline_id,
            stage_count,
            execution_order,
        }
    }

    /// Update a stage's status and bindings.
    pub fn update_stage(&mut self, req: PipelineStageUpdateRequest) -> PipelineStageUpdateResponse {
        let Some(pipeline) = self.pipelines.get_mut(&req.pipeline_id) else {
            return PipelineStageUpdateResponse {
                pipeline_id: req.pipeline_id,
                stage_id: req.stage_id,
                accepted: false,
                progress: 0.0,
            };
        };
        let Some(stage) = pipeline.stages.iter_mut().find(|s| s.id == req.stage_id) else {
            return PipelineStageUpdateResponse {
                pipeline_id: req.pipeline_id,
                stage_id: req.stage_id,
                accepted: false,
                progress: Self::compute_progress(&pipeline.stages),
            };
        };
        stage.status = req.status;
        stage.bindings = req.bindings;
        let progress = Self::compute_progress(&pipeline.stages);
        PipelineStageUpdateResponse {
            pipeline_id: req.pipeline_id,
            stage_id: req.stage_id,
            accepted: true,
            progress,
        }
    }

    /// Get all completed bindings for a pipeline (for progressive dashboard rendering).
    #[must_use]
    pub fn completed_bindings(&self, pipeline_id: &str) -> Vec<&DataBinding> {
        self.pipelines.get(pipeline_id).map_or_else(Vec::new, |p| {
            p.stages
                .iter()
                .filter(|s| s.status == StageStatus::Complete)
                .flat_map(|s| s.bindings.iter())
                .collect()
        })
    }

    /// Pipeline domain hint.
    #[must_use]
    pub fn domain(&self, pipeline_id: &str) -> Option<&str> {
        self.pipelines
            .get(pipeline_id)
            .and_then(|p| p.domain.as_deref())
    }

    /// Human-readable pipeline title (for dashboard chrome).
    #[must_use]
    pub fn title(&self, pipeline_id: &str) -> Option<&str> {
        self.pipelines.get(pipeline_id).map(|p| p.title.as_str())
    }

    /// Topological execution order of stage IDs (progressive rendering order).
    #[must_use]
    pub fn execution_order(&self, pipeline_id: &str) -> Option<&[String]> {
        self.pipelines
            .get(pipeline_id)
            .map(|p| p.execution_order.as_slice())
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "progress fraction 0..1, precision sufficient"
    )]
    fn compute_progress(stages: &[PipelineStage]) -> f64 {
        if stages.is_empty() {
            return 1.0;
        }
        let complete = stages
            .iter()
            .filter(|s| {
                matches!(
                    s.status,
                    StageStatus::Complete | StageStatus::Failed | StageStatus::Skipped
                )
            })
            .count();
        complete as f64 / stages.len() as f64
    }
}

/// Simple topological sort using Kahn's algorithm. Falls back to input order on cycles.
fn topological_sort(stages: &[PipelineStage]) -> Vec<String> {
    let ids: Vec<&str> = stages.iter().map(|s| s.id.as_str()).collect();
    let mut in_degree: HashMap<&str, usize> = ids.iter().map(|id| (*id, 0)).collect();
    let mut adj: HashMap<&str, Vec<&str>> = ids.iter().map(|id| (*id, Vec::new())).collect();

    for stage in stages {
        for dep in &stage.depends_on {
            if let Some(neighbors) = adj.get_mut(dep.as_str()) {
                neighbors.push(&stage.id);
            }
            if let Some(deg) = in_degree.get_mut(stage.id.as_str()) {
                *deg += 1;
            }
        }
    }

    let mut queue: std::collections::VecDeque<&str> = in_degree
        .iter()
        .filter(|(_, deg)| **deg == 0)
        .map(|(id, _)| *id)
        .collect();
    let mut result = Vec::with_capacity(stages.len());

    while let Some(id) = queue.pop_front() {
        result.push(id.to_string());
        if let Some(neighbors) = adj.get(id) {
            for &neighbor in neighbors {
                if let Some(deg) = in_degree.get_mut(neighbor) {
                    *deg = deg.saturating_sub(1);
                    if *deg == 0 {
                        queue.push_back(neighbor);
                    }
                }
            }
        }
    }

    if result.len() < stages.len() {
        stages.iter().map(|s| s.id.clone()).collect()
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_stage(id: &str, deps: &[&str]) -> PipelineStage {
        PipelineStage {
            id: id.to_string(),
            label: id.to_string(),
            depends_on: deps.iter().map(ToString::to_string).collect(),
            status: StageStatus::Pending,
            bindings: vec![],
        }
    }

    #[test]
    fn topological_sort_linear() {
        let stages = vec![
            make_stage("a", &[]),
            make_stage("b", &["a"]),
            make_stage("c", &["b"]),
        ];
        let order = topological_sort(&stages);
        assert_eq!(order, vec!["a", "b", "c"]);
    }

    #[test]
    fn topological_sort_diamond() {
        let stages = vec![
            make_stage("a", &[]),
            make_stage("b", &["a"]),
            make_stage("c", &["a"]),
            make_stage("d", &["b", "c"]),
        ];
        let order = topological_sort(&stages);
        let a_pos = order.iter().position(|s| s == "a").unwrap();
        let d_pos = order.iter().position(|s| s == "d").unwrap();
        assert!(a_pos < d_pos);
        assert_eq!(order.len(), 4);
    }

    #[test]
    fn pipeline_submit_and_update() {
        let mut reg = PipelineRegistry::new();
        let resp = reg.submit(PipelineSubmitRequest {
            pipeline_id: "p1".to_string(),
            title: "Test".to_string(),
            stages: vec![make_stage("a", &[]), make_stage("b", &["a"])],
            domain: Some("neural".to_string()),
        });
        assert_eq!(resp.stage_count, 2);
        assert_eq!(resp.execution_order, vec!["a", "b"]);

        let update = reg.update_stage(PipelineStageUpdateRequest {
            pipeline_id: "p1".to_string(),
            stage_id: "a".to_string(),
            status: StageStatus::Complete,
            bindings: vec![DataBinding::Gauge {
                id: "g1".to_string(),
                label: "G".to_string(),
                value: 1.0,
                min: 0.0,
                max: 2.0,
                unit: String::new(),
                normal_range: [0.0, 2.0],
                warning_range: [0.0, 0.0],
            }],
        });
        assert!(update.accepted);
        assert!((update.progress - 0.5).abs() < f64::EPSILON);

        let bindings = reg.completed_bindings("p1");
        assert_eq!(bindings.len(), 1);
        assert_eq!(reg.domain("p1"), Some("neural"));
        assert_eq!(reg.title("p1"), Some("Test"));
        let order = reg.execution_order("p1").expect("execution order");
        assert_eq!(order, ["a".to_string(), "b".to_string()].as_slice());
    }

    #[test]
    fn pipeline_update_nonexistent() {
        let mut reg = PipelineRegistry::new();
        let resp = reg.update_stage(PipelineStageUpdateRequest {
            pipeline_id: "missing".to_string(),
            stage_id: "a".to_string(),
            status: StageStatus::Complete,
            bindings: vec![],
        });
        assert!(!resp.accepted);
    }

    #[test]
    fn empty_pipeline_progress_is_one() {
        let mut reg = PipelineRegistry::new();
        reg.submit(PipelineSubmitRequest {
            pipeline_id: "empty".to_string(),
            title: "Empty".to_string(),
            stages: vec![],
            domain: None,
        });
        assert!(reg.completed_bindings("empty").is_empty());
    }
}
