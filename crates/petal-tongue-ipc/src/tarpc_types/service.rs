// SPDX-License-Identifier: AGPL-3.0-only
//! tarpc service trait for petalTongue RPC operations.

use super::{
    discovery::PrimalEndpoint,
    health::{HealthStatus, ProtocolInfo, VersionInfo},
    metrics::PrimalMetrics,
    render::{RenderRequest, RenderResponse},
};

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

    /// Render graph topology (semantic: `ui.render.graph`)
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
