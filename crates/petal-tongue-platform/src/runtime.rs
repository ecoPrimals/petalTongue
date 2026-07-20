// SPDX-License-Identifier: AGPL-3.0-or-later
//! Embedded runtime: owns the tokio executor, scene compilation, and IPC client.
//!
//! This is the core of `petal-tongue-platform`. The host application interacts
//! exclusively through [`EmbeddedRuntime`] (Rust) or the C-FFI layer (`ffi.rs`).

use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::RwLock;
use tracing::{info, warn};

use petal_tongue_core::platform_metrics::{self, PlatformMetrics};
use petal_tongue_core::scenario_builder::ScenarioBuilder;
use petal_tongue_core::scenarios::{
    AirSpringCropCoefficientScenario, AirSpringDroughtIndexScenario, AirSpringET0Scenario,
    AirSpringRichardsPDEScenario, GroundSpringAndersonLocalizationScenario,
    GroundSpringSeismicScenario, GroundSpringSensorDriftScenario,
    GroundSpringSpectralReconstructionScenario,
};
use petal_tongue_scene::modality::SvgCompiler;
use petal_tongue_scene::modality::WebGlCompiler;
use petal_tongue_scene::{
    DataBindingCompiler, GrammarCompiler, ModalityCompiler, ModalityOutput, SceneGraph,
};

use crate::config::{EmbedConfig, PlatformConfig};
use crate::lifecycle::{PlatformError, PlatformEvent, PlatformLifecycle, RuntimeState};

/// Callback type for platform events sent back to the host.
pub type EventCallback = Box<dyn Fn(PlatformEvent) + Send + Sync>;

/// The embedded runtime that drives petalTongue from within a host application.
///
/// Owns:
/// - A tokio multi-thread runtime (configurable worker count)
/// - The grammar/scene compilation pipeline
/// - SVG rendering via `SvgCompiler`
/// - An event callback for pushing results to the host
///
/// Does **not** own:
/// - The OS event loop (that belongs to the host)
/// - GPU surfaces (host passes raw handles if needed)
pub struct EmbeddedRuntime {
    state: RuntimeState,
    config: EmbedConfig,
    tokio_rt: Option<Runtime>,
    compiler: Arc<GrammarCompiler>,
    svg_compiler: Arc<SvgCompiler>,
    webgl_compiler: Arc<WebGlCompiler>,
    scene_cache: Arc<RwLock<Option<SceneGraph>>>,
    event_callback: Option<EventCallback>,
    builders: Vec<Box<dyn ScenarioBuilder>>,
    metrics: Box<dyn PlatformMetrics>,
}

impl EmbeddedRuntime {
    /// Create a new runtime in the [`RuntimeState::Created`] state.
    ///
    /// # Errors
    /// Returns [`PlatformError::Runtime`] if the tokio runtime cannot be built.
    pub fn new(config: EmbedConfig) -> Result<Self, PlatformError> {
        let tokio_rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .thread_name("pt-platform")
            .build()
            .map_err(|e| PlatformError::Runtime(format!("tokio init failed: {e}")))?;

        Ok(Self {
            state: RuntimeState::Created,
            config,
            tokio_rt: Some(tokio_rt),
            compiler: Arc::new(GrammarCompiler::new()),
            svg_compiler: Arc::new(SvgCompiler::new()),
            webgl_compiler: Arc::new(WebGlCompiler::new()),
            scene_cache: Arc::new(RwLock::new(None)),
            event_callback: None,
            builders: Self::builtin_builders(),
            metrics: platform_metrics::detect(),
        })
    }

    /// Register a callback for events emitted by the runtime.
    pub fn set_event_callback(&mut self, cb: EventCallback) {
        self.event_callback = Some(cb);
    }

    /// Start the runtime (connect transport, begin discovery).
    ///
    /// # Errors
    /// Returns [`PlatformError::InvalidState`] if not in `Created` or `Stopped` state.
    pub fn start(&mut self) -> Result<(), PlatformError> {
        match self.state {
            RuntimeState::Created | RuntimeState::Stopped => {}
            other => {
                return Err(PlatformError::InvalidState {
                    current: other,
                    attempted: "start".to_owned(),
                });
            }
        }

        info!(
            platform = %self.config.platform,
            "petalTongue platform runtime starting"
        );

        self.state = RuntimeState::Running;
        self.emit_event(PlatformEvent::StateChanged(RuntimeState::Running));
        Ok(())
    }

    /// Stop the runtime (disconnect transport, flush state).
    ///
    /// # Errors
    /// Returns [`PlatformError::InvalidState`] if not in a stoppable state.
    pub fn stop(&mut self) -> Result<(), PlatformError> {
        match self.state {
            RuntimeState::Running | RuntimeState::Paused => {}
            other => {
                return Err(PlatformError::InvalidState {
                    current: other,
                    attempted: "stop".to_owned(),
                });
            }
        }

        info!("petalTongue platform runtime stopping");
        self.state = RuntimeState::Stopped;
        self.emit_event(PlatformEvent::StateChanged(RuntimeState::Stopped));
        Ok(())
    }

    /// Render a named scenario/scene to SVG.
    ///
    /// # Arguments
    /// * `builder_id` — scenario builder identifier (e.g. `"airspring.et0"`)
    /// * `scene_name` — scene within that builder (e.g. `"daily_et0"`)
    ///
    /// # Errors
    /// Returns error if scenario is unknown or compilation fails.
    pub fn render_svg(&self, builder_id: &str, scene_name: &str) -> Result<String, PlatformError> {
        if self.state != RuntimeState::Running {
            return Err(PlatformError::InvalidState {
                current: self.state,
                attempted: "render_svg".to_owned(),
            });
        }

        let scene_graph = self.compile_scene(builder_id, scene_name)?;
        let output = self.svg_compiler.compile(&scene_graph);

        match output {
            ModalityOutput::Svg(bytes) => String::from_utf8(bytes.to_vec())
                .map_err(|e| PlatformError::Runtime(format!("SVG output is not valid UTF-8: {e}"))),
            _ => Err(PlatformError::Runtime(
                "SvgCompiler did not produce SVG output".to_owned(),
            )),
        }
    }

    /// Render a `DataBinding` directly to SVG (for host-provided data).
    ///
    /// # Errors
    /// Returns error if compilation fails.
    pub fn render_binding_svg(
        &self,
        binding_json: &str,
        domain: Option<&str>,
    ) -> Result<String, PlatformError> {
        if self.state != RuntimeState::Running {
            return Err(PlatformError::InvalidState {
                current: self.state,
                attempted: "render_binding_svg".to_owned(),
            });
        }

        let binding: petal_tongue_core::DataBinding = serde_json::from_str(binding_json)
            .map_err(|e| PlatformError::Serialization(format!("invalid DataBinding JSON: {e}")))?;

        let (expr, data) = DataBindingCompiler::compile(&binding, domain);
        let scene_graph = self.compiler.compile(&expr, &data);
        let output = self.svg_compiler.compile(&scene_graph);

        match output {
            ModalityOutput::Svg(bytes) => String::from_utf8(bytes.to_vec())
                .map_err(|e| PlatformError::Runtime(format!("SVG output is not valid UTF-8: {e}"))),
            _ => Err(PlatformError::Runtime(
                "SvgCompiler did not produce SVG output".to_owned(),
            )),
        }
    }

    /// Render a data binding to WebGL draw commands (JSON-serialized [`WebGlScene`]).
    ///
    /// # Errors
    /// Returns error if the runtime is not running or binding JSON is invalid.
    pub fn render_binding_webgl(
        &self,
        binding_json: &str,
        domain: Option<&str>,
    ) -> Result<String, PlatformError> {
        if self.state != RuntimeState::Running {
            return Err(PlatformError::InvalidState {
                current: self.state,
                attempted: "render_binding_webgl".to_owned(),
            });
        }

        let binding: petal_tongue_core::DataBinding = serde_json::from_str(binding_json)
            .map_err(|e| PlatformError::Serialization(format!("invalid DataBinding JSON: {e}")))?;

        let (expr, data) = DataBindingCompiler::compile(&binding, domain);
        let scene_graph = self.compiler.compile(&expr, &data);
        let output = self.webgl_compiler.compile(&scene_graph);

        match output {
            ModalityOutput::GpuCommands(bytes) => String::from_utf8(bytes.to_vec())
                .map_err(|e| PlatformError::Runtime(format!("WebGL output not UTF-8: {e}"))),
            _ => Err(PlatformError::Runtime(
                "WebGlCompiler did not produce GpuCommands output".to_owned(),
            )),
        }
    }

    /// Compile a scene graph from a named scenario builder and scene.
    ///
    /// # Errors
    /// Returns error if the builder or scene name is unknown.
    pub fn compile_scene(
        &self,
        builder_id: &str,
        scene_name: &str,
    ) -> Result<SceneGraph, PlatformError> {
        let builder = self
            .builders
            .iter()
            .find(|b| b.id() == builder_id)
            .ok_or_else(|| {
                PlatformError::Config(format!("unknown scenario builder: {builder_id}"))
            })?;

        let vis_scene = builder.build_scene(scene_name).ok_or_else(|| {
            PlatformError::Config(format!(
                "unknown scene '{scene_name}' in builder '{builder_id}'"
            ))
        })?;

        let domain = &vis_scene.metadata.domain;
        let domain_ref = if domain.is_empty() {
            None
        } else {
            Some(domain.as_str())
        };

        // Compile the first binding (primary visualization for the scene).
        // Multi-binding scenes compose at the host layer via multiple render calls.
        let binding = vis_scene.bindings.first().ok_or_else(|| {
            PlatformError::Config(format!(
                "scene '{scene_name}' in builder '{builder_id}' has no data bindings"
            ))
        })?;

        let (expr, data) = DataBindingCompiler::compile(binding, domain_ref);
        let scene_graph = self.compiler.compile(&expr, &data);
        Ok(scene_graph)
    }

    /// List all available scenario builders and their scenes.
    #[must_use]
    pub fn list_scenarios(&self) -> Vec<ScenarioInfo> {
        self.builders
            .iter()
            .map(|b| ScenarioInfo {
                id: b.id().to_owned(),
                name: b.name().to_owned(),
                domain: b.domain().to_owned(),
                scenes: b.available_scenes(),
            })
            .collect()
    }

    /// Process a JSON-RPC request string, returning the JSON response.
    ///
    /// This enables the host to use the same protocol as network clients.
    ///
    /// # Errors
    /// Returns error if the request is malformed or processing fails.
    #[allow(clippy::too_many_lines)]
    pub fn ipc_request(&mut self, json: &str) -> Result<String, PlatformError> {
        if self.state != RuntimeState::Running {
            return Err(PlatformError::InvalidState {
                current: self.state,
                attempted: "ipc_request".to_owned(),
            });
        }

        let request: serde_json::Value = serde_json::from_str(json)
            .map_err(|e| PlatformError::Serialization(format!("invalid JSON-RPC request: {e}")))?;

        let method = request
            .get("method")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");

        let response = match method {
            "health.check" => serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.get("id"),
                "result": { "status": "ok", "state": format!("{:?}", self.state) }
            }),
            "capabilities.list" => serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.get("id"),
                "result": {
                    "capabilities": [
                        "pt.render_svg",
                        "pt.render_binding",
                        "pt.render_webgl",
                        "pt.state",
                        "pt.scenarios",
                        "pt.metrics",
                        "health.check",
                        "capabilities.list"
                    ]
                }
            }),
            "pt.render_svg" => {
                let builder_id = request
                    .pointer("/params/builder_id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("airspring.et0");
                let scene_name = request
                    .pointer("/params/scene_name")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("daily_et0");

                match self.render_svg(builder_id, scene_name) {
                    Ok(svg) => serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": request.get("id"),
                        "result": { "svg": svg }
                    }),
                    Err(e) => Self::error_response(&request, -32000, &e.to_string()),
                }
            }
            "pt.render_binding" => {
                let binding_json = request
                    .pointer("/params/binding")
                    .map(std::string::ToString::to_string)
                    .unwrap_or_default();
                let domain = request
                    .pointer("/params/domain")
                    .and_then(serde_json::Value::as_str);

                match self.render_binding_svg(&binding_json, domain) {
                    Ok(svg) => serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": request.get("id"),
                        "result": { "svg": svg }
                    }),
                    Err(e) => Self::error_response(&request, -32000, &e.to_string()),
                }
            }
            "pt.render_webgl" => {
                let binding_json = request
                    .pointer("/params/binding")
                    .map(std::string::ToString::to_string)
                    .unwrap_or_default();
                let domain = request
                    .pointer("/params/domain")
                    .and_then(serde_json::Value::as_str);

                match self.render_binding_webgl(&binding_json, domain) {
                    Ok(webgl_json) => serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": request.get("id"),
                        "result": serde_json::from_str::<serde_json::Value>(&webgl_json).unwrap_or_default()
                    }),
                    Err(e) => Self::error_response(&request, -32000, &e.to_string()),
                }
            }
            "pt.state" => serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.get("id"),
                "result": { "state": format!("{:?}", self.state) }
            }),
            "pt.metrics" => {
                let snap = self.metrics.snapshot();
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": request.get("id"),
                    "result": {
                        "cpu_percent": snap.cpu_percent,
                        "memory_total": snap.memory_total,
                        "memory_used": snap.memory_used,
                        "memory_percent": snap.memory_percent(),
                        "cpu_count": snap.cpu_count,
                        "source": self.metrics.source_id()
                    }
                })
            }
            "pt.scenarios" => {
                let list = self.list_scenarios();
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": request.get("id"),
                    "result": { "scenarios": list }
                })
            }
            _ => Self::error_response(&request, -32601, &format!("unknown method: {method}")),
        };

        serde_json::to_string(&response)
            .map_err(|e| PlatformError::Serialization(format!("failed to serialize response: {e}")))
    }

    /// Current runtime state.
    #[must_use]
    pub const fn state(&self) -> RuntimeState {
        self.state
    }

    /// Reference to the embed configuration.
    #[must_use]
    pub const fn config(&self) -> &EmbedConfig {
        &self.config
    }

    fn emit_event(&self, event: PlatformEvent) {
        if let Some(ref cb) = self.event_callback {
            cb(event);
        }
    }

    fn error_response(request: &serde_json::Value, code: i64, message: &str) -> serde_json::Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": request.get("id"),
            "error": { "code": code, "message": message }
        })
    }

    fn builtin_builders() -> Vec<Box<dyn ScenarioBuilder>> {
        vec![
            Box::new(AirSpringET0Scenario),
            Box::new(AirSpringRichardsPDEScenario),
            Box::new(AirSpringCropCoefficientScenario),
            Box::new(AirSpringDroughtIndexScenario),
            Box::new(GroundSpringSeismicScenario),
            Box::new(GroundSpringAndersonLocalizationScenario),
            Box::new(GroundSpringSensorDriftScenario),
            Box::new(GroundSpringSpectralReconstructionScenario),
        ]
    }
}

/// Info about an available scenario builder (for JSON serialization).
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScenarioInfo {
    /// Unique builder identifier (e.g. `"airspring.et0"`).
    pub id: String,
    /// Human-readable builder name.
    pub name: String,
    /// Domain for palette selection (e.g. `"agriculture"`).
    pub domain: String,
    /// Available scene names within this builder.
    pub scenes: Vec<String>,
}

impl PlatformLifecycle for EmbeddedRuntime {
    fn on_create(&mut self, config: EmbedConfig) -> Result<(), PlatformError> {
        self.config = config;
        info!(platform = %self.config.platform, "lifecycle: on_create");
        Ok(())
    }

    fn on_start(&mut self) -> Result<(), PlatformError> {
        self.start()
    }

    fn on_resume(&mut self) -> Result<(), PlatformError> {
        if self.state == RuntimeState::Paused {
            self.state = RuntimeState::Running;
            self.emit_event(PlatformEvent::StateChanged(RuntimeState::Running));
        }
        Ok(())
    }

    fn on_pause(&mut self) -> Result<(), PlatformError> {
        if self.state == RuntimeState::Running {
            self.state = RuntimeState::Paused;
            self.emit_event(PlatformEvent::StateChanged(RuntimeState::Paused));
        }
        Ok(())
    }

    fn on_stop(&mut self) -> Result<(), PlatformError> {
        self.stop()
    }

    fn on_destroy(&mut self) -> Result<(), PlatformError> {
        info!("lifecycle: on_destroy");
        self.state = RuntimeState::Stopped;
        if let Some(rt) = self.tokio_rt.take() {
            rt.shutdown_background();
        }
        Ok(())
    }

    fn on_low_memory(&mut self) {
        warn!("lifecycle: low memory — dropping scene cache");
        if let Some(ref rt) = self.tokio_rt {
            let cache = Arc::clone(&self.scene_cache);
            rt.spawn(async move {
                let mut guard = cache.write().await;
                *guard = None;
            });
        }
    }

    fn on_configuration_changed(&mut self, config: PlatformConfig) {
        info!(
            width = config.width,
            height = config.height,
            density = config.density,
            dark_mode = config.dark_mode,
            "lifecycle: configuration changed"
        );
        self.config.display = config;
    }

    fn state(&self) -> RuntimeState {
        self.state
    }
}

impl Drop for EmbeddedRuntime {
    fn drop(&mut self) {
        if let Some(rt) = self.tokio_rt.take() {
            rt.shutdown_background();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Platform;

    fn test_runtime() -> EmbeddedRuntime {
        let config = EmbedConfig::new(Platform::Desktop);
        let mut rt = EmbeddedRuntime::new(config).expect("runtime should create");
        rt.start().expect("runtime should start");
        rt
    }

    #[test]
    fn health_check_returns_ok() -> Result<(), PlatformError> {
        let mut rt = test_runtime();
        let resp =
            rt.ipc_request(r#"{"jsonrpc":"2.0","id":1,"method":"health.check","params":{}}"#)?;
        let v: serde_json::Value = serde_json::from_str(&resp).expect("valid JSON");
        assert_eq!(v["result"]["status"], "ok");
        assert_eq!(v["id"], 1);
        Ok(())
    }

    #[test]
    fn capabilities_list_returns_methods() -> Result<(), PlatformError> {
        let mut rt = test_runtime();
        let resp =
            rt.ipc_request(r#"{"jsonrpc":"2.0","id":2,"method":"capabilities.list","params":{}}"#)?;
        let v: serde_json::Value = serde_json::from_str(&resp).expect("valid JSON");
        let caps = v["result"]["capabilities"].as_array().expect("array");
        assert!(caps.iter().any(|c| c == "health.check"));
        assert!(caps.iter().any(|c| c == "pt.render_svg"));
        Ok(())
    }

    #[test]
    fn unknown_method_returns_error() -> Result<(), PlatformError> {
        let mut rt = test_runtime();
        let resp =
            rt.ipc_request(r#"{"jsonrpc":"2.0","id":3,"method":"no.such.method","params":{}}"#)?;
        let v: serde_json::Value = serde_json::from_str(&resp).expect("valid JSON");
        assert_eq!(v["error"]["code"], -32601);
        Ok(())
    }

    #[test]
    fn state_method_returns_running() -> Result<(), PlatformError> {
        let mut rt = test_runtime();
        let resp = rt.ipc_request(r#"{"jsonrpc":"2.0","id":4,"method":"pt.state","params":{}}"#)?;
        let v: serde_json::Value = serde_json::from_str(&resp).expect("valid JSON");
        assert_eq!(v["result"]["state"], "Running");
        Ok(())
    }

    #[test]
    fn ipc_request_while_stopped_fails() {
        let config = EmbedConfig::new(Platform::Desktop);
        let mut rt = EmbeddedRuntime::new(config).expect("runtime should create");
        let result =
            rt.ipc_request(r#"{"jsonrpc":"2.0","id":1,"method":"health.check","params":{}}"#);
        assert!(result.is_err());
    }

    #[test]
    fn metrics_returns_snapshot() -> Result<(), PlatformError> {
        let mut rt = test_runtime();
        let resp =
            rt.ipc_request(r#"{"jsonrpc":"2.0","id":5,"method":"pt.metrics","params":{}}"#)?;
        let v: serde_json::Value = serde_json::from_str(&resp).expect("valid JSON");
        assert_eq!(v["id"], 5);
        assert!(v["result"]["cpu_count"].as_u64().unwrap_or(0) >= 1);
        assert!(v["result"]["source"].is_string());
        assert!(v["result"]["memory_percent"].is_number());
        Ok(())
    }

    #[test]
    fn render_webgl_returns_scene_data() -> Result<(), PlatformError> {
        let mut rt = test_runtime();
        let binding = serde_json::json!({
            "channel_type": "scatter",
            "id": "test-scatter",
            "label": "Test",
            "x": [1.0, 3.0],
            "y": [2.0, 4.0],
            "unit": "mm"
        });
        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 10,
            "method": "pt.render_webgl",
            "params": { "binding": binding }
        });
        let resp = rt.ipc_request(&serde_json::to_string(&req).unwrap())?;
        let v: serde_json::Value = serde_json::from_str(&resp).expect("valid JSON");
        assert_eq!(v["id"], 10);
        assert!(
            v.get("error").is_none(),
            "unexpected error: {}",
            serde_json::to_string_pretty(&v).unwrap()
        );
        assert!(
            v["result"]["vertices"].is_array(),
            "response: {}",
            serde_json::to_string_pretty(&v).unwrap()
        );
        assert!(v["result"]["indices"].is_array());
        assert!(v["result"]["draw_calls"].is_array());
        assert!(v["result"]["view_projection"].is_array());
        Ok(())
    }

    #[test]
    fn capabilities_includes_render_webgl() -> Result<(), PlatformError> {
        let mut rt = test_runtime();
        let resp =
            rt.ipc_request(r#"{"jsonrpc":"2.0","id":11,"method":"capabilities.list","params":{}}"#)?;
        let v: serde_json::Value = serde_json::from_str(&resp).expect("valid JSON");
        let caps = v["result"]["capabilities"].as_array().expect("array");
        assert!(caps.iter().any(|c| c == "pt.render_webgl"));
        Ok(())
    }
}
