// SPDX-License-Identifier: AGPL-3.0-only
//! tarpc client implementation

use std::net::SocketAddr;
use std::time::Duration;

use serde_json::Value;
use tracing::{debug, info, warn};

use crate::tarpc_types::{
    HealthStatus, PetalTongueRpcClient, PrimalEndpoint, PrimalMetrics, ProtocolInfo, RenderRequest,
    RenderResponse, VersionInfo,
};

use super::types::{TarpcClient, TarpcClientError, TarpcResult};

impl TarpcClient {
    /// Create new tarpc client from endpoint
    pub fn new(endpoint: &str) -> TarpcResult<Self> {
        debug!("Creating tarpc client for endpoint: {}", endpoint);

        let addr = Self::parse_endpoint(endpoint)?;

        Ok(Self {
            endpoint: endpoint.to_string(),
            addr,
            connection: std::sync::Arc::new(tokio::sync::RwLock::new(None)),
            timeout: Duration::from_secs(5),
        })
    }

    /// Set request timeout
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Get capabilities from remote primal
    pub async fn get_capabilities(&self) -> TarpcResult<Vec<String>> {
        debug!("Getting capabilities from {}", self.endpoint);
        let client = self.get_connection().await?;
        let ctx = tarpc::context::current();

        client
            .capabilities_list(ctx)
            .await
            .map_err(|e| TarpcClientError::Rpc(format!("capabilities_list failed: {e}")))
    }

    /// Discover primals by capability
    pub async fn discover_capability(&self, capability: &str) -> TarpcResult<Vec<PrimalEndpoint>> {
        debug!(
            "Discovering capability '{}' from {}",
            capability, self.endpoint
        );
        let client = self.get_connection().await?;
        let ctx = tarpc::context::current();

        client
            .discovery_find_capability(ctx, capability.to_string())
            .await
            .map_err(|e| TarpcClientError::Rpc(format!("discovery_find_capability failed: {e}")))
    }

    /// Get health status from remote primal
    pub async fn health(&self) -> TarpcResult<HealthStatus> {
        debug!("Checking health of {}", self.endpoint);
        let client = self.get_connection().await?;
        let ctx = tarpc::context::current();

        client
            .health_check(ctx)
            .await
            .map_err(|e| TarpcClientError::Rpc(format!("health check failed: {e}")))
    }

    /// Get version information from remote primal
    pub async fn version(&self) -> TarpcResult<VersionInfo> {
        debug!("Getting version from {}", self.endpoint);
        let client = self.get_connection().await?;
        let ctx = tarpc::context::current();

        client
            .version_get(ctx)
            .await
            .map_err(|e| TarpcClientError::Rpc(format!("version call failed: {e}")))
    }

    /// Get available protocols from remote primal
    pub async fn protocols(&self) -> TarpcResult<Vec<ProtocolInfo>> {
        debug!("Getting protocols from {}", self.endpoint);
        let client = self.get_connection().await?;
        let ctx = tarpc::context::current();

        client
            .protocols_list(ctx)
            .await
            .map_err(|e| TarpcClientError::Rpc(format!("protocols call failed: {e}")))
    }

    /// Render graph topology (requires "visualization" or "gpu-rendering" capability)
    pub async fn render_graph(&self, request: RenderRequest) -> TarpcResult<RenderResponse> {
        debug!("Rendering graph via {}", self.endpoint);
        let client = self.get_connection().await?;
        let ctx = tarpc::context::current();

        client
            .ui_render_graph(ctx, request)
            .await
            .map_err(|e| TarpcClientError::Rpc(format!("ui_render_graph failed: {e}")))
    }

    /// Get metrics from remote primal
    pub async fn get_metrics(&self) -> TarpcResult<PrimalMetrics> {
        debug!("Getting metrics from {}", self.endpoint);
        let client = self.get_connection().await?;
        let ctx = tarpc::context::current();

        client
            .metrics_get(ctx)
            .await
            .map_err(|e| TarpcClientError::Rpc(format!("metrics_get failed: {e}")))
    }

    /// Call method with dynamic params (for adapter integration)
    pub async fn call_method(&self, method: &str, params: Option<Value>) -> TarpcResult<Value> {
        debug!("Calling method: {} with params: {:?}", method, params);

        match method {
            "get_capabilities" => {
                let result = self.get_capabilities().await?;
                serde_json::to_value(result).map_err(|e| {
                    TarpcClientError::Serialization(format!("Failed to serialize: {e}"))
                })
            }
            "discover_capability" => {
                let capability = params
                    .as_ref()
                    .and_then(|v| v.get("capability"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        TarpcClientError::Configuration("Missing capability parameter".to_string())
                    })?
                    .to_string();

                let result = self.discover_capability(&capability).await?;
                serde_json::to_value(result).map_err(|e| {
                    TarpcClientError::Serialization(format!("Failed to serialize: {e}"))
                })
            }
            "health" => {
                let result = self.health().await?;
                serde_json::to_value(result).map_err(|e| {
                    TarpcClientError::Serialization(format!("Failed to serialize: {e}"))
                })
            }
            "version" => {
                let result = self.version().await?;
                serde_json::to_value(result).map_err(|e| {
                    TarpcClientError::Serialization(format!("Failed to serialize: {e}"))
                })
            }
            "protocols" => {
                let result = self.protocols().await?;
                serde_json::to_value(result).map_err(|e| {
                    TarpcClientError::Serialization(format!("Failed to serialize: {e}"))
                })
            }
            "render_graph" => {
                let request: RenderRequest = serde_json::from_value(params.ok_or_else(|| {
                    TarpcClientError::Configuration("Missing request parameter".to_string())
                })?)
                .map_err(|e| TarpcClientError::Serialization(format!("Invalid request: {e}")))?;

                let result = self.render_graph(request).await?;
                serde_json::to_value(result).map_err(|e| {
                    TarpcClientError::Serialization(format!("Failed to serialize: {e}"))
                })
            }
            "get_metrics" => {
                let result = self.get_metrics().await?;
                serde_json::to_value(result).map_err(|e| {
                    TarpcClientError::Serialization(format!("Failed to serialize: {e}"))
                })
            }
            _ => Err(TarpcClientError::Configuration(format!(
                "Unknown method: {method}"
            ))),
        }
    }

    async fn get_connection(&self) -> TarpcResult<PetalTongueRpcClient> {
        {
            let conn = self.connection.read().await;
            if let Some(ref client) = *conn {
                return Ok(client.clone());
            }
        }

        let mut conn = self.connection.write().await;

        if let Some(ref client) = *conn {
            return Ok(client.clone());
        }

        info!("🔌 Establishing tarpc connection to {}", self.addr);
        let client = self.connect().await?;
        *conn = Some(client.clone());

        Ok(client)
    }

    async fn connect(&self) -> TarpcResult<PetalTongueRpcClient> {
        debug!("Connecting to tarpc server at {}", self.addr);

        let stream = tokio::time::timeout(self.timeout, tokio::net::TcpStream::connect(self.addr))
            .await
            .map_err(|_| TarpcClientError::Timeout(format!("Connection timeout to {}", self.addr)))?
            .map_err(|e| {
                TarpcClientError::Connection(format!("Failed to connect to {}: {}", self.addr, e))
            })?;

        debug!("✅ TCP connection established to {}", self.addr);

        let transport = tarpc::serde_transport::new(
            tokio_util::codec::LengthDelimitedCodec::builder()
                .max_frame_length(16 * 1024 * 1024)
                .new_framed(stream),
            tokio_serde::formats::Bincode::default(),
        );

        let client = PetalTongueRpcClient::new(Default::default(), transport).spawn();

        info!("🚀 tarpc client ready for {}", self.endpoint);

        Ok(client)
    }

    /// Parse endpoint string to SocketAddr with DNS resolution
    #[doc(hidden)]
    pub fn parse_endpoint(endpoint: &str) -> TarpcResult<SocketAddr> {
        let addr_str = endpoint.strip_prefix("tarpc://").ok_or_else(|| {
            TarpcClientError::Configuration(format!(
                "Invalid tarpc endpoint (expected tarpc://host:port): {endpoint}"
            ))
        })?;

        if let Ok(addr) = addr_str.parse::<SocketAddr>() {
            debug!("✅ Parsed tarpc endpoint as IP address: {}", addr);
            return Ok(addr);
        }

        let (host, port) = addr_str.rsplit_once(':').ok_or_else(|| {
            TarpcClientError::Configuration(format!(
                "Invalid tarpc endpoint (missing port): {addr_str}"
            ))
        })?;

        let port: u16 = port
            .parse()
            .map_err(|e| TarpcClientError::Configuration(format!("Invalid port '{port}': {e}")))?;

        let ip = match host {
            "localhost" | "localhost.localdomain" => {
                debug!("🔍 Resolved localhost to 127.0.0.1");
                std::net::Ipv4Addr::LOCALHOST
            }
            _ => host.parse().map_err(|e| {
                warn!("Cannot resolve hostname '{}': {}. Use IP address or set up DNS/hosts entry.", host, e);
                TarpcClientError::Configuration(format!(
                    "Invalid hostname or IP '{host}': {e}. tarpc requires IP addresses or 'localhost'. Use env vars like GPU_RENDERER_ENDPOINT to configure."
                ))
            })?,
        };

        let addr = SocketAddr::new(std::net::IpAddr::V4(ip), port);
        debug!("✅ Resolved tarpc endpoint: {} → {}", addr_str, addr);
        Ok(addr)
    }

    #[doc(hidden)]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    #[doc(hidden)]
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    #[doc(hidden)]
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}

impl std::fmt::Debug for TarpcClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TarpcClient")
            .field("endpoint", &self.endpoint)
            .field("addr", &self.addr)
            .field("timeout", &self.timeout)
            .field("connection", &"<connection>")
            .finish()
    }
}
