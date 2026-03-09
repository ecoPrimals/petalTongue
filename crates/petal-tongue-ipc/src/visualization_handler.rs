// SPDX-License-Identifier: AGPL-3.0-only
//! Handlers for visualization.render and visualization.render.stream IPC methods.
//!
//! These methods allow springs and other primals to push data for rendering
//! without compile-time coupling -- they discover petalTongue at runtime and
//! send `DataBinding` payloads via JSON-RPC.

use petal_tongue_core::{DataBinding, ThresholdRange};
use petal_tongue_scene::compiler::GrammarCompiler;
use petal_tongue_scene::grammar::GrammarExpr;
use petal_tongue_scene::modality::{ModalityCompiler, ModalityOutput, SvgCompiler};
use petal_tongue_scene::scene_graph::SceneGraph;
use petal_tongue_scene::tufte::{ChartjunkDetection, DataInkRatio, TufteConstraint, TufteReport};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::warn;

/// UI configuration preset pushed from springs alongside data.
///
/// Allows springs to drive petalTongue panel visibility, mode, and zoom
/// without compile-time coupling (healthSpring V9 SAME DAVE motor channel).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiConfig {
    /// Panel visibility (e.g., "`left_sidebar`", "`audio_panel`", "`trust_dashboard`")
    #[serde(default)]
    pub show_panels: HashMap<String, bool>,
    /// Initial mode (e.g., "clinical", "research", "monitoring")
    #[serde(default)]
    pub mode: Option<String>,
    /// Zoom preset (e.g., "fit", "1.0", "2.0")
    #[serde(default)]
    pub initial_zoom: Option<String>,
    /// Whether to show the awakening sequence
    #[serde(default)]
    pub awakening_enabled: Option<bool>,
    /// Theme override (e.g., "clinical-dark", "ecology-light")
    #[serde(default)]
    pub theme: Option<String>,
}

/// Request payload for visualization.render
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationRenderRequest {
    /// Unique identifier for this visualization session
    pub session_id: String,
    /// Human-readable title for the visualization
    pub title: String,
    /// Data bindings to render (the actual data + chart type)
    pub bindings: Vec<DataBinding>,
    /// Optional threshold ranges for status coloring
    #[serde(default)]
    pub thresholds: Vec<ThresholdRange>,
    /// Domain hint for theme selection (e.g., "health", "physics", "ecology")
    #[serde(default)]
    pub domain: Option<String>,
    /// Optional UI configuration preset from the pushing spring
    #[serde(default)]
    pub ui_config: Option<UiConfig>,
}

/// Response for visualization.render
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationRenderResponse {
    /// Session ID (echoed back for tracking)
    pub session_id: String,
    /// Number of bindings accepted
    pub bindings_accepted: usize,
    /// Current rendering status
    pub status: String,
}

/// Request payload for visualization.render.stream (incremental update)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamUpdateRequest {
    /// Session ID to update (must match an existing render session)
    pub session_id: String,
    /// Which binding to update (by id)
    pub binding_id: String,
    /// The update operation
    pub operation: StreamOperation,
}

/// Types of incremental stream updates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamOperation {
    /// Append new data points to a `TimeSeries` or Spectrum
    #[serde(rename = "append")]
    Append {
        /// X-axis values (timestamps for `TimeSeries`, frequencies for Spectrum)
        x_values: Vec<f64>,
        /// Y-axis values (measurements for `TimeSeries`, amplitudes for Spectrum)
        y_values: Vec<f64>,
    },
    /// Replace the current value of a Gauge
    #[serde(rename = "set_value")]
    SetValue {
        /// New gauge value
        value: f64,
    },
    /// Replace the full binding (for Heatmap, `FieldMap`, etc.)
    #[serde(rename = "replace")]
    Replace {
        /// Replacement binding
        binding: DataBinding,
    },
}

/// Response for visualization.render.stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamUpdateResponse {
    /// Session ID (echoed back)
    pub session_id: String,
    /// Binding ID that was updated
    pub binding_id: String,
    /// Whether the update was accepted
    pub accepted: bool,
}

/// Request payload for `visualization.render.grammar` (declarative scene engine path).
///
/// Springs send a `GrammarExpr` (data source, variable bindings, geometry type) plus
/// raw data. petalTongue compiles this through the scene engine with Tufte validation
/// and returns SVG (or another modality) output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarRenderRequest {
    /// Unique session identifier.
    pub session_id: String,
    /// Grammar expression describing the visualization.
    pub grammar: GrammarExpr,
    /// Raw data rows (each row is a JSON object with field values).
    pub data: Vec<serde_json::Value>,
    /// Requested output modality: "svg" (default), "audio", "description".
    #[serde(default = "default_modality")]
    pub modality: String,
    /// Whether to run Tufte constraint validation.
    #[serde(default = "default_true")]
    pub validate_tufte: bool,
    /// Domain hint (e.g. "health", "physics") for constraint tuning.
    #[serde(default)]
    pub domain: Option<String>,
}

fn default_modality() -> String {
    "svg".into()
}
const fn default_true() -> bool {
    true
}

/// Response for `visualization.render.grammar`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarRenderResponse {
    /// Session ID (echoed back).
    pub session_id: String,
    /// The compiled output (SVG string, audio params JSON, or description text).
    pub output: serde_json::Value,
    /// Output modality that was used.
    pub modality: String,
    /// Number of scene graph nodes.
    pub scene_nodes: usize,
    /// Total rendering primitives.
    pub total_primitives: usize,
    /// Tufte constraint report (if validation was requested).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tufte_report: Option<serde_json::Value>,
}

/// Manages active visualization sessions from springs/primals
pub struct VisualizationState {
    sessions: HashMap<String, RenderSession>,
    grammar_scenes: HashMap<String, SceneGraph>,
}

/// A single visualization session with its bindings and metadata
pub struct RenderSession {
    /// Human-readable title for the visualization
    pub title: String,
    /// Data bindings to render
    pub bindings: Vec<DataBinding>,
    /// Optional threshold ranges for status coloring
    pub thresholds: Vec<ThresholdRange>,
    /// Domain hint for theme selection
    pub domain: Option<String>,
    /// UI configuration preset from the pushing spring
    pub ui_config: Option<UiConfig>,
    /// Last update timestamp
    pub updated_at: std::time::Instant,
}

impl VisualizationState {
    /// Create a new empty visualization state
    #[must_use]
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            grammar_scenes: HashMap::new(),
        }
    }

    /// Handle a visualization.render request, creating or replacing a session
    pub fn handle_render(
        &mut self,
        req: VisualizationRenderRequest,
    ) -> VisualizationRenderResponse {
        let count = req.bindings.len();
        self.sessions.insert(
            req.session_id.clone(),
            RenderSession {
                title: req.title,
                bindings: req.bindings,
                thresholds: req.thresholds,
                domain: req.domain,
                ui_config: req.ui_config,
                updated_at: std::time::Instant::now(),
            },
        );
        VisualizationRenderResponse {
            session_id: req.session_id,
            bindings_accepted: count,
            status: "rendering".to_string(),
        }
    }

    /// Handle a visualization.render.stream incremental update
    pub fn handle_stream_update(&mut self, req: StreamUpdateRequest) -> StreamUpdateResponse {
        let accepted = if let Some(session) = self.sessions.get_mut(&req.session_id) {
            if let Some(binding) = session
                .bindings
                .iter_mut()
                .find(|b| binding_id(b) == req.binding_id)
            {
                apply_operation(binding, &req.operation);
                session.updated_at = std::time::Instant::now();
                true
            } else {
                false
            }
        } else {
            false
        };
        StreamUpdateResponse {
            session_id: req.session_id,
            binding_id: req.binding_id,
            accepted,
        }
    }

    /// Get all active sessions (for rendering)
    #[must_use]
    pub fn sessions(&self) -> &HashMap<String, RenderSession> {
        &self.sessions
    }

    /// Get bindings for a specific session
    #[must_use]
    pub fn session_bindings(&self, session_id: &str) -> Option<&[DataBinding]> {
        self.sessions.get(session_id).map(|s| s.bindings.as_slice())
    }

    /// Get all bindings across all sessions
    #[must_use]
    pub fn all_bindings(&self) -> Vec<&DataBinding> {
        self.sessions
            .values()
            .flat_map(|s| s.bindings.iter())
            .collect()
    }

    /// Handle `visualization.render.grammar`: compile a grammar expression through
    /// the scene engine with Tufte validation and return the requested modality output.
    pub fn handle_grammar_render(&mut self, req: GrammarRenderRequest) -> GrammarRenderResponse {
        let compiler = GrammarCompiler;
        let scene = compiler.compile(&req.grammar, &req.data);

        let tufte_report = if req.validate_tufte {
            let primitives: Vec<_> = scene
                .flatten()
                .into_iter()
                .map(|(_, p)| p.clone())
                .collect();
            let data_ink = DataInkRatio;
            let constraints: Vec<&dyn TufteConstraint> = vec![&data_ink];
            let report = TufteReport::evaluate_all(&constraints, &primitives, &req.grammar);
            serde_json::to_value(&report).ok()
        } else {
            None
        };

        let node_count = scene.node_count();
        let prim_count = scene.total_primitives();

        let (output, modality) = compile_modality(&scene, &req.modality);

        // Store scene metadata in a grammar session for future streaming/updates
        self.grammar_scenes.insert(req.session_id.clone(), scene);

        GrammarRenderResponse {
            session_id: req.session_id,
            output,
            modality,
            scene_nodes: node_count,
            total_primitives: prim_count,
            tufte_report,
        }
    }

    /// Get a compiled scene graph by session ID (for downstream rendering).
    #[must_use]
    pub fn grammar_scene(&self, session_id: &str) -> Option<&SceneGraph> {
        self.grammar_scenes.get(session_id)
    }

    /// Handle `visualization.validate`: validate grammar + data against Tufte constraints.
    pub fn handle_validate(&self, req: ValidateRequest) -> ValidateResponse {
        use petal_tongue_scene::compiler::GrammarCompiler;

        let compiler = GrammarCompiler;
        let scene = compiler.compile(&req.grammar, &req.data);
        let primitives: Vec<_> = scene
            .flatten()
            .into_iter()
            .map(|(_, p)| p.clone())
            .collect();

        let data_ink = DataInkRatio;
        let chartjunk = ChartjunkDetection;
        let constraints: Vec<&dyn TufteConstraint> = vec![&data_ink, &chartjunk];
        let report = TufteReport::evaluate_all(&constraints, &primitives, &req.grammar);

        let constraints_out: Vec<ConstraintResult> = report
            .results
            .iter()
            .map(|(name, r)| ConstraintResult {
                name: name.clone(),
                score: r.score,
                passed: r.passed,
                details: r.message.clone(),
            })
            .collect();

        ValidateResponse {
            score: report.overall_score,
            passed: report.overall_score >= 0.5,
            constraints: constraints_out,
        }
    }

    /// Handle `visualization.export`: export a session to the requested format.
    pub fn handle_export(&self, req: ExportRequest) -> ExportResponse {
        if let Some(scene) = self.grammar_scenes.get(&req.session_id) {
            let (output, format) = compile_modality(scene, &req.format);
            ExportResponse {
                session_id: req.session_id,
                format,
                content: match output {
                    serde_json::Value::String(s) => s,
                    other => other.to_string(),
                },
            }
        } else {
            ExportResponse {
                session_id: req.session_id,
                format: req.format,
                content: String::new(),
            }
        }
    }

    /// Handle `visualization.dismiss`: remove a session.
    pub fn handle_dismiss(&mut self, req: DismissRequest) -> DismissResponse {
        let removed_session = self.sessions.remove(&req.session_id).is_some();
        let removed_scene = self.grammar_scenes.remove(&req.session_id).is_some();
        DismissResponse {
            session_id: req.session_id,
            dismissed: removed_session || removed_scene,
        }
    }
}

impl Default for VisualizationState {
    fn default() -> Self {
        Self::new()
    }
}

// -----------------------------------------------------------------------------
// visualization.validate
// -----------------------------------------------------------------------------

/// Request for `visualization.validate`: validate grammar + data against Tufte constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateRequest {
    /// Grammar expression to validate.
    pub grammar: petal_tongue_scene::grammar::GrammarExpr,
    /// Raw data rows for the grammar.
    pub data: Vec<serde_json::Value>,
}

/// Response for `visualization.validate`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateResponse {
    /// Overall Tufte score (0.0 to 1.0).
    pub score: f64,
    /// Whether the visualization passed validation.
    pub passed: bool,
    /// Per-constraint results.
    pub constraints: Vec<ConstraintResult>,
}

/// Result of evaluating a single Tufte constraint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintResult {
    /// Constraint name (e.g. "DataInkRatio", "ChartjunkDetection").
    pub name: String,
    /// Numeric score (0.0 to 1.0).
    pub score: f64,
    /// Whether the constraint passed.
    pub passed: bool,
    /// Human-readable details.
    pub details: String,
}

// -----------------------------------------------------------------------------
// visualization.export
// -----------------------------------------------------------------------------

/// Request for `visualization.export`: export a session to a format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    /// Session ID to export.
    pub session_id: String,
    /// Output format: "svg", "json", "description".
    pub format: String,
}

/// Response for `visualization.export`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResponse {
    /// Session ID (echoed back).
    pub session_id: String,
    /// Format that was used.
    pub format: String,
    /// Exported content (SVG string, JSON, or description text).
    pub content: String,
}

// -----------------------------------------------------------------------------
// visualization.dismiss
// -----------------------------------------------------------------------------

/// Request for `visualization.dismiss`: remove a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DismissRequest {
    /// Session ID to dismiss.
    pub session_id: String,
}

/// Response for `visualization.dismiss`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DismissResponse {
    /// Session ID (echoed back).
    pub session_id: String,
    /// Whether the session was removed.
    pub dismissed: bool,
}

// -----------------------------------------------------------------------------
// visualization.interact.apply
// -----------------------------------------------------------------------------

/// Request for `visualization.interact.apply`: apply an interaction intent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionApplyRequest {
    /// Intent: "select", "focus", "inspect", "filter", "navigate".
    pub intent: String,
    /// Target identifiers to apply the intent to.
    pub targets: Vec<String>,
    /// Optional grammar ID for scoped interactions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grammar_id: Option<String>,
}

/// Response for `visualization.interact.apply`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionApplyResponse {
    /// Whether the interaction was accepted.
    pub accepted: bool,
    /// Number of targets resolved.
    pub targets_resolved: usize,
}

// -----------------------------------------------------------------------------
// visualization.interact.perspectives
// -----------------------------------------------------------------------------

/// A visualization perspective (view configuration).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Perspective {
    /// Perspective identifier.
    pub id: String,
    /// Modalities active in this perspective.
    pub modalities: Vec<String>,
    /// Current selection.
    pub selection: Vec<String>,
    /// Sync mode (e.g. "shared_selection").
    pub sync_mode: String,
}

/// Compile a scene graph to the requested output modality.
fn compile_modality(scene: &SceneGraph, modality: &str) -> (serde_json::Value, String) {
    use petal_tongue_scene::modality::{AudioCompiler, DescriptionCompiler};

    match modality {
        "svg" => {
            let compiler = SvgCompiler;
            match compiler.compile(scene) {
                ModalityOutput::Svg(s) => (serde_json::Value::String(s), "svg".into()),
                _ => (serde_json::Value::Null, "error".into()),
            }
        }
        "audio" => {
            let compiler = AudioCompiler;
            match compiler.compile(scene) {
                ModalityOutput::AudioParams(params) => {
                    let v = serde_json::to_value(&params).unwrap_or(serde_json::Value::Null);
                    (v, "audio".into())
                }
                _ => (serde_json::Value::Null, "error".into()),
            }
        }
        "description" | "accessibility" => {
            let compiler = DescriptionCompiler;
            match compiler.compile(scene) {
                ModalityOutput::Description(s) => {
                    (serde_json::Value::String(s), "description".into())
                }
                _ => (serde_json::Value::Null, "error".into()),
            }
        }
        other => {
            warn!("Unknown modality '{other}', falling back to SVG");
            let compiler = SvgCompiler;
            match compiler.compile(scene) {
                ModalityOutput::Svg(s) => (serde_json::Value::String(s), "svg".into()),
                _ => (serde_json::Value::Null, "error".into()),
            }
        }
    }
}

/// Poll-based registry for interaction event subscribers.
///
/// Springs call `interaction.subscribe` to register, then `interaction.poll`
/// to drain queued interaction events.
#[derive(Default)]
pub struct InteractionSubscriberRegistry {
    subscribers: HashMap<String, InteractionSubscriber>,
}

struct InteractionSubscriber {
    queue: Vec<InteractionEventNotification>,
    #[expect(dead_code)]
    subscribed_at: std::time::Instant,
    event_filter: Vec<String>,
    callback_method: Option<String>,
    #[expect(dead_code)]
    grammar_id: Option<String>,
}

/// A queued interaction event ready for IPC delivery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionEventNotification {
    /// Semantic event type (e.g. "select", "inspect", "navigate").
    pub event_type: String,
    /// Resolved data-space target identifiers.
    pub targets: Vec<String>,
    /// ISO 8601 timestamp of the event.
    pub timestamp: String,
    /// Perspective that originated the event, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub perspective_id: Option<u64>,
}

impl InteractionSubscriberRegistry {
    /// Create a new empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a subscriber. Returns `true` if newly registered.
    pub fn subscribe(&mut self, subscriber_id: &str) -> bool {
        self.subscribe_with_filter(subscriber_id, Vec::new(), None, None)
    }

    /// Register a subscriber with event type filter, callback method, and grammar filter.
    ///
    /// The callback model (healthSpring V12 pattern): when `callback_method` is set,
    /// events are queued for callback delivery rather than poll-only.
    pub fn subscribe_with_filter(
        &mut self,
        subscriber_id: &str,
        event_filter: Vec<String>,
        callback_method: Option<String>,
        grammar_id: Option<String>,
    ) -> bool {
        use std::collections::hash_map::Entry;
        match self.subscribers.entry(subscriber_id.to_string()) {
            Entry::Occupied(_) => false,
            Entry::Vacant(e) => {
                e.insert(InteractionSubscriber {
                    queue: Vec::new(),
                    subscribed_at: std::time::Instant::now(),
                    event_filter,
                    callback_method,
                    grammar_id,
                });
                true
            }
        }
    }

    /// Get the callback method for a subscriber (if callback-based).
    pub fn callback_method(&self, subscriber_id: &str) -> Option<&str> {
        self.subscribers
            .get(subscriber_id)
            .and_then(|s| s.callback_method.as_deref())
    }

    /// Get all subscribers that have callback methods and pending events.
    pub fn pending_callbacks(&self) -> Vec<(&str, &str, &[InteractionEventNotification])> {
        self.subscribers
            .iter()
            .filter_map(|(id, sub)| {
                sub.callback_method
                    .as_deref()
                    .filter(|_| !sub.queue.is_empty())
                    .map(|method| (id.as_str(), method, sub.queue.as_slice()))
            })
            .collect()
    }

    /// Remove a subscriber. Returns `true` if the subscriber existed.
    pub fn unsubscribe(&mut self, subscriber_id: &str) -> bool {
        self.subscribers.remove(subscriber_id).is_some()
    }

    /// Push an event to all active subscribers, respecting event type filters.
    pub fn broadcast(&mut self, event: InteractionEventNotification) {
        for sub in self.subscribers.values_mut() {
            let passes_event_filter =
                sub.event_filter.is_empty() || sub.event_filter.contains(&event.event_type);
            if passes_event_filter {
                sub.queue.push(event.clone());
            }
        }
    }

    /// Drain queued events for a subscriber, returning them.
    pub fn poll(&mut self, subscriber_id: &str) -> Vec<InteractionEventNotification> {
        self.subscribers
            .get_mut(subscriber_id)
            .map(|sub| std::mem::take(&mut sub.queue))
            .unwrap_or_default()
    }

    /// Number of active subscribers.
    #[must_use]
    pub fn subscriber_count(&self) -> usize {
        self.subscribers.len()
    }

    /// Apply an interaction intent and broadcast to subscribers.
    pub fn apply_interaction(&mut self, req: &InteractionApplyRequest) -> InteractionApplyResponse {
        let event = InteractionEventNotification {
            event_type: req.intent.clone(),
            targets: req.targets.clone(),
            timestamp: {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default();
                format!("{}Z", now.as_secs())
            },
            perspective_id: None,
        };
        self.broadcast(event);
        InteractionApplyResponse {
            accepted: true,
            targets_resolved: req.targets.len(),
        }
    }

    /// Return available perspectives for this visualization.
    #[must_use]
    pub fn perspectives(&self) -> Vec<Perspective> {
        vec![Perspective {
            id: "default_egui".to_string(),
            modalities: vec!["gui".to_string()],
            selection: Vec::new(),
            sync_mode: "shared_selection".to_string(),
        }]
    }
}

/// Extract the `id` field from any `DataBinding` variant
#[must_use]
pub fn binding_id(binding: &DataBinding) -> &str {
    match binding {
        DataBinding::TimeSeries { id, .. }
        | DataBinding::Distribution { id, .. }
        | DataBinding::Bar { id, .. }
        | DataBinding::Gauge { id, .. }
        | DataBinding::Heatmap { id, .. }
        | DataBinding::Scatter3D { id, .. }
        | DataBinding::FieldMap { id, .. }
        | DataBinding::Spectrum { id, .. } => id,
    }
}

/// Apply a stream operation to a binding in place
pub fn apply_operation(binding: &mut DataBinding, operation: &StreamOperation) {
    match (binding, operation) {
        (
            DataBinding::TimeSeries {
                x_values, y_values, ..
            },
            StreamOperation::Append {
                x_values: new_x,
                y_values: new_y,
            },
        ) => {
            x_values.extend(new_x.iter().copied());
            y_values.extend(new_y.iter().copied());
        }
        (
            DataBinding::Spectrum {
                frequencies,
                amplitudes,
                ..
            },
            StreamOperation::Append {
                x_values: new_freq,
                y_values: new_amp,
            },
        ) => {
            frequencies.extend(new_freq.iter().copied());
            amplitudes.extend(new_amp.iter().copied());
        }
        (DataBinding::Gauge { value, .. }, StreamOperation::SetValue { value: new_val }) => {
            *value = *new_val;
        }
        (b, StreamOperation::Replace { binding: new_b }) => {
            *b = new_b.clone();
        }
        _ => {
            warn!("Mismatched stream operation: operation not applicable to this binding type");
        }
    }
}
