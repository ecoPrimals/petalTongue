// SPDX-License-Identifier: AGPL-3.0-only
//! biomeOS Discovery Backend
//!
//! Implements capability-based discovery via biomeOS Neural API.
//! This is the primary discovery mechanism in production.

use crate::capability_discovery::{
    CapabilityQuery, DiscoveryBackend, DiscoveryError, PrimalEndpoint, PrimalEndpoints,
    PrimalHealth,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// biomeOS discovery backend
#[derive(Debug)]
pub struct BiomeOsBackend {
    /// JSON-RPC client for biomeOS Neural API
    client: BiomeOsClient,
}

/// Simple JSON-RPC client for biomeOS
#[derive(Debug)]
struct BiomeOsClient {
    socket_path: String,
}

impl BiomeOsBackend {
    /// Create a new biomeOS discovery backend
    pub fn new(socket_path: impl Into<String>) -> Self {
        Self {
            client: BiomeOsClient {
                socket_path: socket_path.into(),
            },
        }
    }

    /// Try to create from environment (`XDG_RUNTIME_DIR` or fallback)
    /// Create from environment with capability-based discovery
    ///
    /// # Socket Discovery Priority
    /// 1. `BIOMEOS_NEURAL_API_SOCKET` - explicit override (highest priority)
    /// 2. `$XDG_RUNTIME_DIR/biomeos/neural-api.sock` - XDG standard
    /// 3. `/tmp/biomeos-neural-api.sock` - legacy fallback
    ///
    /// # TRUE PRIMAL: Zero hardcoded paths in discovery logic
    pub fn from_env() -> Result<Self, DiscoveryError> {
        use crate::platform_dirs;

        // Priority 1: Explicit environment override
        if let Ok(socket_path) = std::env::var("BIOMEOS_NEURAL_API_SOCKET") {
            let path = std::path::PathBuf::from(&socket_path);
            if path.exists() {
                return Ok(Self::new(socket_path));
            }
            // Env var set but socket doesn't exist - warn and continue discovery
            tracing::debug!(
                "BIOMEOS_NEURAL_API_SOCKET={} but socket not found, trying discovery",
                socket_path
            );
        }

        // Priority 2: XDG runtime directory
        if let Ok(runtime_dir) = platform_dirs::runtime_dir() {
            let socket_path =
                runtime_dir.join(format!("{}.sock", crate::constants::biomeos_socket_name()));
            if socket_path.exists() {
                return Ok(Self::new(socket_path.to_string_lossy().to_string()));
            }
        }

        // Priority 3: Legacy /tmp fallback
        let fallback = crate::constants::biomeos_legacy_socket();
        if fallback.exists() {
            return Ok(Self::new(fallback.to_string_lossy().to_string()));
        }

        Err(DiscoveryError::BackendUnavailable(
            "biomeOS Neural API socket not found. Set BIOMEOS_NEURAL_API_SOCKET env var or start biomeOS.".to_string(),
        ))
    }
}

#[async_trait::async_trait]
impl DiscoveryBackend for BiomeOsBackend {
    async fn query(&self, query: &CapabilityQuery) -> Result<Vec<PrimalEndpoint>, DiscoveryError> {
        // Build JSON-RPC request for capability query
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "discovery.query_capability".to_string(),
            params: serde_json::json!({
                "domain": query.domain,
                "operation": query.operation,
                "version_req": query.version_req,
            }),
            id: 1,
        };

        // Call biomeOS via Unix socket
        let response: JsonRpcResponse = self
            .client
            .call(&request)
            .await
            .map_err(|e| DiscoveryError::CommunicationError(e.to_string()))?;

        // Parse response
        if let Some(error) = response.error {
            if error.message.contains("not found") {
                return Err(DiscoveryError::CapabilityNotFound {
                    domain: query.domain.clone(),
                });
            }
            return Err(DiscoveryError::CommunicationError(error.message));
        }

        let result = response.result.ok_or_else(|| {
            DiscoveryError::CommunicationError("No result in response".to_string())
        })?;

        // Parse primals from result
        let primals: Vec<BiomeOsPrimal> = serde_json::from_value(result)
            .map_err(|e| DiscoveryError::CommunicationError(format!("Parse error: {e}")))?;

        // Convert to PrimalEndpoint
        Ok(primals.into_iter().map(std::convert::Into::into).collect())
    }

    async fn subscribe(&self, _query: &CapabilityQuery) -> Result<(), DiscoveryError> {
        // TODO: Implement WebSocket subscription for real-time updates
        Ok(())
    }
}

impl BiomeOsClient {
    async fn call(&self, request: &JsonRpcRequest) -> Result<JsonRpcResponse, std::io::Error> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::UnixStream;

        // Connect to biomeOS socket
        let mut stream = UnixStream::connect(&self.socket_path).await?;

        // Send request
        let request_json = serde_json::to_vec(request)?;
        stream.write_all(&request_json).await?;
        stream.write_all(b"\n").await?;

        // Read response
        let mut response_buf = Vec::new();
        stream.read_to_end(&mut response_buf).await?;

        // Parse response
        serde_json::from_slice(&response_buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

/// JSON-RPC request
#[derive(Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Value,
    id: u64,
}

/// JSON-RPC response
#[derive(Deserialize)]
struct JsonRpcResponse {
    #[expect(dead_code)]
    jsonrpc: String,
    result: Option<Value>,
    error: Option<JsonRpcError>,
    #[expect(dead_code)]
    id: u64,
}

#[derive(Deserialize)]
struct JsonRpcError {
    message: String,
}

/// biomeOS primal format (from Neural API)
#[derive(Deserialize)]
struct BiomeOsPrimal {
    id: String,
    capabilities: Vec<String>,
    tarpc_endpoint: Option<String>,
    jsonrpc_endpoint: Option<String>,
    health: String,
}

impl From<BiomeOsPrimal> for PrimalEndpoint {
    fn from(p: BiomeOsPrimal) -> Self {
        use crate::capability_discovery::Capability;

        Self {
            id: p.id,
            capabilities: p
                .capabilities
                .into_iter()
                .map(|cap| {
                    // Parse capability string "domain.operation"
                    let parts: Vec<&str> = cap.split('.').collect();
                    if parts.len() == 2 {
                        Capability::new(parts[0]).with_operation(parts[1])
                    } else {
                        Capability::new(cap)
                    }
                })
                .collect(),
            endpoints: PrimalEndpoints {
                tarpc: p.tarpc_endpoint,
                jsonrpc: p.jsonrpc_endpoint,
                https: None,
            },
            health: match p.health.as_str() {
                "healthy" => PrimalHealth::Healthy,
                "degraded" => PrimalHealth::Degraded,
                _ => PrimalHealth::Unavailable,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::env_test_helpers;

    #[test]
    fn test_biomeos_backend_new() {
        let backend = BiomeOsBackend::new("/tmp/custom.sock");
        // Just verify it constructs - we can't call query without a real socket
        drop(backend);
    }

    #[test]
    fn test_biomeos_from_env_explicit_socket() {
        let temp = std::env::temp_dir().join("biomeos-test-socket");
        std::fs::create_dir_all(temp.parent().unwrap()).unwrap();
        std::fs::write(&temp, "").unwrap();

        env_test_helpers::with_env_var("BIOMEOS_NEURAL_API_SOCKET", temp.to_str().unwrap(), || {
            let backend = BiomeOsBackend::from_env().unwrap();
            drop(backend);
        });

        let _ = std::fs::remove_file(&temp);
    }

    #[test]
    fn test_biomeos_from_env_socket_not_found() {
        env_test_helpers::with_env_var(
            "BIOMEOS_NEURAL_API_SOCKET",
            "/nonexistent/path/neural-api.sock",
            || {
                let result = BiomeOsBackend::from_env();
                assert!(result.is_err());
                if let Err(DiscoveryError::BackendUnavailable(_)) = result {
                    // Expected
                } else {
                    panic!("Expected BackendUnavailable error");
                }
            },
        );
    }

    #[test]
    fn test_biomeos_primal_conversion_healthy() {
        let biomeos_primal = BiomeOsPrimal {
            id: "test-primal-1".to_string(),
            capabilities: vec!["crypto.encrypt".to_string(), "crypto.decrypt".to_string()],
            tarpc_endpoint: Some("tarpc://unix:/run/primal/test".to_string()),
            jsonrpc_endpoint: Some("/run/primal/test.sock".to_string()),
            health: "healthy".to_string(),
        };

        let endpoint: PrimalEndpoint = biomeos_primal.into();
        assert_eq!(endpoint.id, "test-primal-1");
        assert_eq!(endpoint.capabilities.len(), 2);
        assert_eq!(endpoint.health, PrimalHealth::Healthy);
    }

    #[test]
    fn test_biomeos_primal_conversion_degraded() {
        let biomeos_primal = BiomeOsPrimal {
            id: "degraded-primal".to_string(),
            capabilities: vec!["storage.cache".to_string()],
            tarpc_endpoint: None,
            jsonrpc_endpoint: Some("/run/degraded.sock".to_string()),
            health: "degraded".to_string(),
        };

        let endpoint: PrimalEndpoint = biomeos_primal.into();
        assert_eq!(endpoint.health, PrimalHealth::Degraded);
    }

    #[test]
    fn test_biomeos_primal_conversion_unavailable() {
        let biomeos_primal = BiomeOsPrimal {
            id: "unavail-primal".to_string(),
            capabilities: vec![],
            tarpc_endpoint: None,
            jsonrpc_endpoint: None,
            health: "unavailable".to_string(),
        };

        let endpoint: PrimalEndpoint = biomeos_primal.into();
        assert_eq!(endpoint.health, PrimalHealth::Unavailable);
    }

    #[test]
    fn test_biomeos_primal_conversion_unknown_health() {
        let biomeos_primal = BiomeOsPrimal {
            id: "unknown-primal".to_string(),
            capabilities: vec!["ui.render".to_string()],
            tarpc_endpoint: None,
            jsonrpc_endpoint: None,
            health: "unknown-status".to_string(),
        };

        let endpoint: PrimalEndpoint = biomeos_primal.into();
        assert_eq!(endpoint.health, PrimalHealth::Unavailable);
    }

    #[test]
    fn test_biomeos_primal_conversion_single_part_capability() {
        let biomeos_primal = BiomeOsPrimal {
            id: "legacy-primal".to_string(),
            capabilities: vec!["legacy".to_string()],
            tarpc_endpoint: None,
            jsonrpc_endpoint: None,
            health: "healthy".to_string(),
        };

        let endpoint: PrimalEndpoint = biomeos_primal.into();
        assert_eq!(endpoint.capabilities.len(), 1);
    }
}
