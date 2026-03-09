// SPDX-License-Identifier: AGPL-3.0-only
//! # 🚀 tarpc Types and Traits for petalTongue
//!
//! **HIGH-PERFORMANCE PRIMAL-TO-PRIMAL RPC**
//!
//! Provides shared types and service traits for tarpc-based communication.
//! This module defines the interface used by both clients and servers.
//!
//! ## Performance
//! - ~10-20 μs latency (vs 50-100 μs for JSON-RPC)
//! - ~100K requests/sec (vs 10K for JSON-RPC)
//! - Zero-copy binary serialization with bincode
//! - Type-safe at compile time
//!
//! ## Philosophy
//! - tarpc PRIMARY for primal-to-primal communication
//! - JSON-RPC SECONDARY for local IPC and debugging
//! - HTTPS OPTIONAL for external/browser access
//! - Protocol-agnostic architecture
//! - Zero unsafe blocks in this module
//! - Modern idiomatic Rust
//!
//! ## Safety
//! The `#[tarpc::service]` macro generates safe code using the tarpc framework.
//! All serialization is handled by serde with compile-time type checking.
//! No manual memory manipulation or unsafe operations are performed.
//! The generated client/server implementations use only safe Rust abstractions.

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// tarpc service trait for petalTongue operations
///
/// This trait defines the async RPC interface using tarpc.
/// Both client and server implementations use this trait.
///
/// # Protocol Priority
/// 1. **tarpc** (PRIMARY) - High-performance binary RPC for primal-to-primal
/// 2. **JSON-RPC** (SECONDARY) - Universal, debuggable, local IPC
/// 3. **HTTPS** (OPTIONAL) - External/browser access
///
/// # Semantic Naming Convention
/// All methods follow the `domain.operation` pattern per `SEMANTIC_METHOD_NAMING_STANDARD.md`:
/// - `discovery.*` - Service discovery operations
/// - `health.*` - Health monitoring operations
/// - `capabilities.*` - Capability queries
/// - `ui.*` - UI rendering operations
/// - `metrics.*` - Telemetry operations
///
/// # Design Philosophy
/// - Agnostic: No hardcoded endpoints or service names
/// - Capability-based: Discover by capability, not by name
/// - Self-aware: Services know what they can do, not what others are
/// - Runtime discovery: Zero compile-time knowledge of other primals
#[tarpc::service]
pub trait PetalTongueRpc {
    /// Get primal capabilities (semantic: capabilities.list)
    ///
    /// Returns the capabilities this primal provides.
    /// Examples: "visualization", "gpu-rendering", "graph-compute", "discovery"
    ///
    /// # Returns
    /// List of capability strings this primal offers
    async fn capabilities_list() -> Vec<String>;

    /// Discover services by capability (semantic: `discovery.find_capability`)
    ///
    /// Query for primals that provide a specific capability.
    ///
    /// # Arguments
    /// * `capability` - Required capability (e.g., "gpu-rendering", "visualization")
    ///
    /// # Returns
    /// List of primal endpoints that provide this capability
    async fn discovery_find_capability(capability: String) -> Vec<PrimalEndpoint>;

    /// Get health and status (semantic: health.check)
    ///
    /// Returns current health metrics for monitoring and diagnostics.
    ///
    /// # Returns
    /// Health status with uptime, version, and metrics
    async fn health_check() -> HealthStatus;

    /// Get version information (semantic: version.get)
    ///
    /// Returns version and protocol compatibility info.
    ///
    /// # Returns
    /// Version information including protocol support
    async fn version_get() -> VersionInfo;

    /// Get supported protocols (semantic: protocols.list)
    ///
    /// Returns list of communication protocols this primal supports.
    ///
    /// # Returns
    /// List of protocol info (tarpc, jsonrpc, https)
    async fn protocols_list() -> Vec<ProtocolInfo>;

    /// Render graph topology (semantic: `ui.render_graph`)
    ///
    /// Renders a graph topology visualization.
    /// Only available if primal has "visualization" capability.
    ///
    /// # Arguments
    /// * `request` - Graph rendering request with topology data
    ///
    /// # Returns
    /// Rendered image data (PNG bytes)
    async fn ui_render_graph(request: RenderRequest) -> RenderResponse;

    /// Query primal metrics (semantic: metrics.get)
    ///
    /// Returns current performance and operational metrics.
    ///
    /// # Returns
    /// Metrics including FPS, hang detection, resource usage
    async fn metrics_get() -> PrimalMetrics;
}

/// Primal endpoint information
///
/// Represents a discovered primal's connection details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalEndpoint {
    /// Unique primal identifier (UUID)
    pub primal_id: String,

    /// Human-readable primal name (optional)
    pub name: Option<String>,

    /// Endpoint URL (e.g., "<tarpc://hostname:9001>")
    pub endpoint: String,

    /// Capabilities this primal provides
    pub capabilities: Vec<String>,

    /// Primal type (e.g., "petalTongue", "Toadstool", "Songbird")
    pub primal_type: String,

    /// Protocol used (e.g., "tarpc", "jsonrpc", "https")
    pub protocol: String,

    /// Optional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Health status
///
/// Operational health and status information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Status string ("healthy", "degraded", "unhealthy")
    pub status: String,

    /// Primal version
    pub version: String,

    /// Uptime in seconds
    pub uptime_seconds: u64,

    /// Current capabilities available
    pub capabilities: Vec<String>,

    /// Optional health details
    #[serde(default)]
    pub details: HashMap<String, String>,
}

/// Version information
///
/// Version and compatibility details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// Primal version string (e.g., "1.2.0")
    pub version: String,

    /// tarpc protocol version
    pub tarpc_version: String,

    /// JSON-RPC protocol version
    pub jsonrpc_version: String,

    /// HTTPS API version (if enabled)
    pub https_version: Option<String>,

    /// Supported capabilities
    pub capabilities: Vec<String>,
}

/// Protocol information
///
/// Details about a supported communication protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolInfo {
    /// Protocol name ("tarpc", "jsonrpc", "https")
    pub name: String,

    /// Endpoint (e.g., "<tarpc://localhost:9001>", "<unix:///tmp/petaltongue.sock>")
    pub endpoint: String,

    /// Whether this protocol is currently enabled
    pub enabled: bool,

    /// Protocol priority (1 = primary, 2 = secondary, 3 = fallback)
    pub priority: u8,

    /// Optional additional info
    #[serde(default)]
    pub info: HashMap<String, String>,
}

/// Graph rendering request
///
/// Request to render a graph topology visualization or raw frame buffer.
/// Supports two modes:
/// 1. Graph topology rendering (topology field populated)
/// 2. Raw frame buffer rendering (data field populated, format="rgba8")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderRequest {
    /// Graph topology data (JSON or binary) - for graph rendering
    #[serde(default)]
    pub topology: Bytes,

    /// Raw pixel data - for frame buffer rendering (e.g., RGBA8)
    #[serde(default)]
    pub data: Bytes,

    /// Render width in pixels
    pub width: u32,

    /// Render height in pixels
    pub height: u32,

    /// Render format ("png", "svg", "jpg", "rgba8")
    /// - "rgba8": Raw 32-bit RGBA pixel data for frame buffer rendering
    /// - "png"/"svg"/"jpg": Graph topology rendering output formats
    pub format: String,

    /// Optional render settings
    #[serde(default)]
    pub settings: HashMap<String, String>,

    /// Optional metadata (capabilities, primal info, etc.)
    #[serde(default)]
    pub metadata: Option<HashMap<String, String>>,
}

/// Graph rendering response
///
/// Rendered visualization or frame buffer output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderResponse {
    /// Success flag
    pub success: bool,

    /// Rendered image data (bytes)
    /// - For graph rendering: PNG/SVG/JPG encoded data
    /// - For frame buffer: RGBA8 pixel data (optional, may be displayed remotely)
    #[serde(default)]
    pub data: Bytes,

    /// Output width in pixels
    pub width: u32,

    /// Output height in pixels
    pub height: u32,

    /// Error message if failed
    pub error: Option<String>,

    /// Render time in milliseconds
    pub render_time_ms: u64,
}

/// Primal metrics
///
/// Performance and operational metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalMetrics {
    /// Frames per second (for UI primals)
    pub fps: Option<f32>,

    /// Time since last frame in seconds
    pub time_since_last_frame: Option<f32>,

    /// Is primal hanging (no frames for >5s)
    pub is_hanging: bool,

    /// Total frames rendered
    pub total_frames: u64,

    /// CPU usage percentage (0-100)
    pub cpu_usage: Option<f32>,

    /// Memory usage in bytes
    pub memory_usage: Option<u64>,

    /// Uptime in seconds
    pub uptime_seconds: u64,

    /// Custom metrics
    #[serde(default)]
    pub custom: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primal_endpoint_serialization() {
        let endpoint = PrimalEndpoint {
            primal_id: "test-123".to_string(),
            name: Some("Test Primal".to_string()),
            endpoint: "tarpc://localhost:9001".to_string(),
            capabilities: vec!["visualization".to_string()],
            primal_type: "petalTongue".to_string(),
            protocol: "tarpc".to_string(),
            metadata: HashMap::new(),
        };

        let json = serde_json::to_string(&endpoint).unwrap();
        let deserialized: PrimalEndpoint = serde_json::from_str(&json).unwrap();

        assert_eq!(endpoint.primal_id, deserialized.primal_id);
        assert_eq!(endpoint.primal_type, deserialized.primal_type);
    }

    #[test]
    fn test_health_status() {
        let health = HealthStatus {
            status: "healthy".to_string(),
            version: "1.2.0".to_string(),
            uptime_seconds: 3600,
            capabilities: vec!["visualization".to_string(), "graph-compute".to_string()],
            details: HashMap::new(),
        };

        assert_eq!(health.status, "healthy");
        assert_eq!(health.capabilities.len(), 2);
    }

    #[test]
    fn test_protocol_info_priority() {
        let tarpc = ProtocolInfo {
            name: "tarpc".to_string(),
            endpoint: "tarpc://localhost:9001".to_string(),
            enabled: true,
            priority: 1, // PRIMARY
            info: HashMap::new(),
        };

        let jsonrpc = ProtocolInfo {
            name: "jsonrpc".to_string(),
            endpoint: "unix:///tmp/petaltongue.sock".to_string(),
            enabled: true,
            priority: 2, // SECONDARY
            info: HashMap::new(),
        };

        assert!(tarpc.priority < jsonrpc.priority);
        assert_eq!(tarpc.name, "tarpc");
    }

    #[test]
    fn test_render_request() {
        let request = RenderRequest {
            topology: Bytes::from_static(&[1, 2, 3, 4]),
            data: Bytes::new(), // Empty for topology rendering
            width: 1920,
            height: 1080,
            format: "png".to_string(),
            settings: HashMap::new(),
            metadata: None,
        };

        assert_eq!(request.width, 1920);
        assert_eq!(request.format, "png");
    }

    #[test]
    fn test_primal_metrics() {
        let metrics = PrimalMetrics {
            fps: Some(60.0),
            time_since_last_frame: Some(0.016),
            is_hanging: false,
            total_frames: 1000,
            cpu_usage: Some(25.5),
            memory_usage: Some(104_857_600),
            uptime_seconds: 3600,
            custom: HashMap::new(),
        };

        assert_eq!(metrics.fps, Some(60.0));
        assert!(!metrics.is_hanging);
        assert_eq!(metrics.total_frames, 1000);
    }

    #[test]
    fn test_version_info() {
        let version = VersionInfo {
            version: "1.2.0".to_string(),
            tarpc_version: "0.34".to_string(),
            jsonrpc_version: "2.0".to_string(),
            https_version: None, // Not enabled
            capabilities: vec!["visualization".to_string()],
        };

        assert_eq!(version.version, "1.2.0");
        assert!(version.https_version.is_none());
        assert!(!version.capabilities.is_empty());
    }
}
