// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed error types for petal-tongue-ui
//!
//! Replaces anyhow with domain-specific thiserror types for better
//! error handling and user-facing messages.

use std::io;
use thiserror::Error;

/// Top-level UI error type covering all domains
#[derive(Debug, Error)]
pub enum UiError {
    // ----- Display -----
    #[error(transparent)]
    Display(#[from] DisplayError),

    // ----- Audio -----
    #[error(transparent)]
    Audio(#[from] AudioError),

    // ----- Sensors -----
    #[error(transparent)]
    Sensor(#[from] SensorError),

    // ----- BiomeOS Integration -----
    #[error(transparent)]
    BiomeOsIntegration(#[from] BiomeOsIntegrationError),

    // ----- Graph Editor -----
    #[error(transparent)]
    GraphEditor(#[from] GraphEditorError),

    // ----- Scenario -----
    #[error(transparent)]
    Scenario(#[from] crate::scenario_error::ScenarioError),

    // ----- Backend -----
    #[error(transparent)]
    Backend(#[from] BackendError),

    // ----- I/O -----
    #[error(transparent)]
    Io(#[from] io::Error),

    // ----- JSON -----
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    /// Symphonia audio decoding error
    #[error("Audio decode error: {0}")]
    Symphonia(String),

    // ----- Generic -----
    #[error("{0}")]
    Generic(String),
}

impl From<petal_tongue_discovery::DiscoveryError> for UiError {
    fn from(e: petal_tongue_discovery::DiscoveryError) -> Self {
        Self::Generic(e.to_string())
    }
}

impl From<UiError> for petal_tongue_discovery::DiscoveryError {
    fn from(e: UiError) -> Self {
        match e {
            UiError::Io(io) => Self::Io(io),
            UiError::Json(j) => Self::Json(j),
            _ => Self::Integration(e.to_string()),
        }
    }
}

impl From<UiError> for petal_tongue_discovery::errors::HealthCheckSource {
    fn from(e: UiError) -> Self {
        match e {
            UiError::Io(io) => Self::Io(io),
            UiError::Json(j) => Self::Json(j),
            _ => Self::Upstream(e.to_string()),
        }
    }
}

impl From<petal_tongue_core::InstanceError> for UiError {
    fn from(e: petal_tongue_core::InstanceError) -> Self {
        Self::Generic(e.to_string())
    }
}

/// Display-related errors (renderer, backends, manager)
#[derive(Debug, Error)]
pub enum DisplayError {
    #[error("Failed to create pixmap for texture")]
    PixmapTextureCreation,

    #[error("Failed to create pixmap")]
    PixmapCreation,

    #[error("No display backends available")]
    NoBackendsAvailable,

    #[error("Failed to initialize any display backend")]
    InitFailed,

    #[error("No active display backend")]
    NoActiveBackend,

    #[error("No active backend to fallback from")]
    NoActiveBackendToFallback,

    #[error("No fallback backend available")]
    NoFallbackBackend,

    #[error("Invalid buffer size: expected {expected}, got {actual}")]
    InvalidBufferSize { expected: usize, actual: usize },

    #[error("Invalid buffer size: expected {expected} bytes ({width}x{height}x4), got {actual}")]
    InvalidBufferSizeDetailed {
        expected: usize,
        width: u32,
        height: u32,
        actual: usize,
    },

    #[error("Failed to open /dev/fb0: {0}")]
    FramebufferOpen(#[source] io::Error),

    #[error("Framebuffer /dev/fb0 not available")]
    FramebufferNotAvailable,

    #[error("Failed to seek framebuffer: {0}")]
    FramebufferSeek(#[source] io::Error),

    #[error("Failed to write to framebuffer: {0}")]
    FramebufferWrite(#[source] io::Error),

    #[error("Framebuffer device not initialized")]
    FramebufferNotInitialized,

    #[error("No external display server detected")]
    NoExternalDisplayServer,

    #[error("Invalid framebuffer width '{value}': {detail}")]
    InvalidFramebufferWidth { value: String, detail: String },

    #[error("Invalid framebuffer height '{value}': {detail}")]
    InvalidFramebufferHeight { value: String, detail: String },

    #[error("Unexpected sysfs format: '{0}'")]
    UnexpectedSysfsFormat(String),

    #[error("Cannot read framebuffer sysfs at {path}: {source}")]
    FramebufferSysfsRead { path: String, source: io::Error },

    #[error("{message} in {path}")]
    FramebufferParse { message: String, path: String },

    #[error("Failed to connect to biomeOS at {path}: {detail}")]
    BiomeOsConnect { path: String, detail: String },

    #[error("Failed to read response from biomeOS: {0}")]
    BiomeOsReadResponse(String),

    #[error("Failed to parse JSON-RPC response: {0}")]
    BiomeOsParseJsonRpc(String),

    #[error("biomeOS returned error: {0}")]
    BiomeOsError(String),

    #[error("No result field in JSON-RPC response")]
    BiomeOsNoResult,

    #[error("Failed to parse display capabilities: {0}")]
    ParseDisplayCapabilities(String),

    #[error("Failed to parse window response: {0}")]
    ParseWindowResponse(String),

    #[error("No window created yet")]
    NoWindowCreated,

    #[error("No displays available")]
    NoDisplaysAvailable,

    #[error("No displays available from display backend")]
    NoDisplaysFromBackend,

    #[error("Failed to create biomeOS discovery backend: {0}")]
    BiomeOsDiscoveryBackend(String),

    #[error("No discovery system available")]
    NoDiscoverySystem,

    #[error("Failed to discover display capability: {0}")]
    DisplayDiscoveryFailed(String),

    #[error("Display provider doesn't expose tarpc endpoint")]
    NoTarpcEndpoint,

    #[error("Failed to create tarpc client: {0}")]
    TarpcClientCreation(String),

    #[error("Not connected to display provider")]
    NotConnectedToDisplay,

    #[error("Failed to query capabilities: {0}")]
    QueryCapabilitiesFailed(String),

    #[error("Failed to create window: {0}")]
    CreateWindowFailed(String),

    #[error("Failed to commit frame: {0}")]
    CommitFrameFailed(String),

    #[error("Failed to broadcast message")]
    StreamBroadcastFailed,
}

/// Audio-related errors
#[derive(Debug, Error)]
pub enum AudioError {
    #[error("No audio backends available")]
    NoBackendsAvailable,

    #[error("No audio backend could be initialized")]
    NoBackendInitialized,

    #[error("No active backend")]
    NoActiveBackend,

    #[error("No fallback backend")]
    NoFallbackBackend,

    #[error("Beep task panicked: {0}")]
    BeepTaskPanicked(String),

    #[error("Tokio runtime creation failed: {0}")]
    TokioRuntimeCreation(String),

    #[error("AudioManager init failed: {0}")]
    AudioManagerInitFailed(String),

    #[error("No audio track found")]
    NoAudioTrack,

    #[error("No audio devices found in /dev/snd/")]
    NoAudioDevices,

    #[error("Socket audio backend is not available on this platform")]
    SocketBackendUnavailable,

    #[error("Socket audio backend not initialized: no connection")]
    SocketBackendNotInitialized,

    #[error("Socket audio backend connection failed: {0}")]
    SocketConnectionFailed(String),

    #[error("Direct audio device not available: {0}")]
    DirectDeviceUnavailable(String),

    #[error("Socket audio task failed: {0}")]
    SocketTaskFailed(String),
}

/// Sensor-related errors (screen, keyboard, mouse, audio)
#[derive(Debug, Error)]
pub enum SensorError {
    #[error("Crossterm I/O error: {0}")]
    Crossterm(String),

    #[error("Invalid framebuffer width '{value}': {detail}")]
    InvalidFramebufferWidth { value: String, detail: String },

    #[error("Invalid framebuffer height '{value}': {detail}")]
    InvalidFramebufferHeight { value: String, detail: String },

    #[error("Unexpected sysfs format: '{0}'")]
    UnexpectedSysfsFormat(String),

    #[error("Cannot read framebuffer sysfs at {path}: {source}")]
    FramebufferSysfsRead { path: String, source: io::Error },

    #[error("{message} in {path}")]
    FramebufferParse { message: String, path: String },

    #[error("{0}")]
    Other(String),
}

/// `BiomeOS` integration errors (provider, events)
#[derive(Debug, Error)]
pub enum BiomeOsIntegrationError {
    #[error("Failed to parse devices response: {0}")]
    ParseDevicesResponse(String),

    #[error("Failed to parse primals response: {0}")]
    ParsePrimalsResponse(String),

    #[error("Failed to parse niche templates: {0}")]
    ParseNicheTemplates(String),

    #[error("No niche_id in response")]
    NoNicheId,

    #[error("Failed to parse niche_id: {0}")]
    ParseNicheId(String),

    #[error("Failed to connect to device management provider: {0}")]
    ConnectToProvider(String),

    #[error("Failed to read response: {0}")]
    ReadResponse(String),

    #[error("Failed to parse JSON-RPC response: {0}")]
    ParseJsonRpcResponse(String),

    #[error("JSON-RPC error: {0}")]
    JsonRpcError(String),

    #[error("No result in JSON-RPC response")]
    NoJsonRpcResult,
}

/// Graph editor errors (streaming, `rpc_methods`, validation, graph)
#[derive(Debug, Error)]
pub enum GraphEditorError {
    #[error("Node with id '{0}' already exists")]
    NodeAlreadyExists(String),

    #[error("Node with id '{0}' not found")]
    NodeNotFound(String),

    #[error("Cannot change node ID (from '{from}' to '{to}')")]
    NodeIdChange { from: String, to: String },

    #[error("Source node '{0}' not found")]
    SourceNodeNotFound(String),

    #[error("Target node '{0}' not found")]
    TargetNodeNotFound(String),

    #[error("Edge from '{from}' to '{to}' would create a cycle")]
    EdgeWouldCreateCycle { from: String, to: String },

    #[error("Edge from '{from}' to '{to}' already exists")]
    EdgeAlreadyExists { from: String, to: String },

    #[error("Edge with id '{0}' not found")]
    EdgeNotFound(String),

    #[error("Graph contains cycles, cannot compute topological sort")]
    GraphContainsCycles,

    #[error("Node ID cannot be empty")]
    EmptyNodeId,

    #[error("Node type cannot be empty")]
    EmptyNodeType,

    #[error("Node position must be finite")]
    NonFiniteNodePosition,

    #[error("Edge '{edge}' references non-existent source node '{node}'")]
    EdgeReferencesMissingSource { edge: String, node: String },

    #[error("Edge '{edge}' references non-existent target node '{node}'")]
    EdgeReferencesMissingTarget { edge: String, node: String },

    #[error("Graph validation failed: contains cycles")]
    ValidationCycles,

    #[error("Invalid node '{0}'")]
    InvalidNode(String),

    #[error("Graph not found")]
    GraphNotFound,

    #[error("Template not found")]
    TemplateNotFound,

    #[error("Node not found")]
    RpcNodeNotFound,

    #[error("Failed to remove node: {0}")]
    RemoveNodeFailed(String),

    #[error("Failed to broadcast message")]
    StreamBroadcastFailed,
}

/// Backend errors (display backends, eframe)
#[derive(Debug, Error)]
pub enum BackendError {
    #[error("eframe backend not available (compile with --features ui-eframe)")]
    EframeNotAvailable,

    #[error("Display backend not available (use capability-discovered backend with biomeOS)")]
    DisplayBackendNotAvailable,

    #[error("Display backend requires biomeOS: {0}")]
    DisplayBackendRequiresBiomeOs(String),

    #[error("eframe::run_native failed: {0}")]
    EframeRunFailed(String),

    #[error("Graph lock poisoned: {0}")]
    GraphLockPoisoned(String),
}

/// Result type alias for UI operations
pub type Result<T> = std::result::Result<T, UiError>;

impl From<symphonia::core::errors::Error> for UiError {
    fn from(e: symphonia::core::errors::Error) -> Self {
        Self::Symphonia(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_error_display_generic() {
        let e = UiError::Generic("test message".to_string());
        let s = format!("{e}");
        assert!(s.contains("test message"));
    }

    #[test]
    fn ui_error_display_symphonia() {
        let e = UiError::Symphonia("decode failed".to_string());
        let s = format!("{e}");
        assert!(s.contains("decode"));
        assert!(s.contains("decode failed"));
    }

    #[test]
    fn display_error_display() {
        let e = DisplayError::PixmapTextureCreation;
        let s = format!("{e}");
        assert!(s.contains("pixmap"));
    }

    #[test]
    fn display_error_invalid_buffer_size() {
        let e = DisplayError::InvalidBufferSize {
            expected: 100,
            actual: 50,
        };
        let s = format!("{e}");
        assert!(s.contains("100"));
        assert!(s.contains("50"));
    }

    #[test]
    fn audio_error_display() {
        let e = AudioError::NoBackendsAvailable;
        let s = format!("{e}");
        assert!(s.contains("audio") || s.contains("backend"));
    }

    #[test]
    fn graph_editor_error_display() {
        let e = GraphEditorError::NodeNotFound("n1".to_string());
        let s = format!("{e}");
        assert!(s.contains("n1"));
    }

    #[test]
    fn backend_error_display() {
        let e = BackendError::EframeNotAvailable;
        let s = format!("{e}");
        assert!(s.contains("eframe"));
    }

    #[test]
    fn ui_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let e: UiError = io_err.into();
        let s = format!("{e}");
        assert!(!s.is_empty());
    }

    #[test]
    fn ui_error_debug() {
        let e = UiError::Generic("debug".to_string());
        let s = format!("{:?}", e);
        assert!(s.contains("Generic"));
    }
}
