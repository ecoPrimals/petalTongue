// SPDX-License-Identifier: AGPL-3.0-only
//! Protocol selection logic for inter-primal communication
//!
//! Implements ecosystem-standard protocol priority:
//! 1. tarpc (PRIMARY) - High-performance binary RPC
//! 2. JSON-RPC (SECONDARY) - Universal, debuggable
//! 3. HTTPS (FALLBACK) - External/browser access

use petal_tongue_ipc::{
    JsonRpcClient, JsonRpcClientError, TarpcClient, TarpcClientError, TarpcResult,
};
use tracing::{debug, info, warn};

/// HTTPS client for primal-to-primal communication
///
/// Uses reqwest with TLS. Falls back to HTTP when HTTPS is unavailable.
#[derive(Clone)]
pub struct HttpsClient {
    base_url: String,
    client: reqwest::Client,
}

impl HttpsClient {
    /// Try common health endpoint paths
    const HEALTH_PATHS: &[&str] = &["/health", "/api/v1/health"];
    /// Try common capabilities endpoint paths
    const CAPABILITIES_PATHS: &[&str] = &["/api/v1/capabilities", "/capabilities"];

    async fn fetch_json(&self, path: &str) -> TarpcResult<serde_json::Value> {
        let url = format!("{}{path}", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| TarpcClientError::Connection(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(TarpcClientError::Connection(format!(
                "HTTP {}: {}",
                resp.status(),
                url
            )));
        }
        resp.json()
            .await
            .map_err(|e| TarpcClientError::Serialization(e.to_string()))
    }

    async fn health(&self) -> TarpcResult<petal_tongue_ipc::HealthStatus> {
        for path in Self::HEALTH_PATHS {
            if let Ok(value) = self.fetch_json(path).await {
                let status = value
                    .get("status")
                    .and_then(|s| s.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let version = value
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let uptime_seconds = value
                    .get("uptime_seconds")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0);
                let capabilities = value
                    .get("capabilities")
                    .or_else(|| value.get("modalities_active"))
                    .and_then(|c| c.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();
                return Ok(petal_tongue_ipc::HealthStatus {
                    status,
                    version,
                    uptime_seconds,
                    capabilities,
                    details: std::collections::HashMap::new(),
                });
            }
        }
        Err(TarpcClientError::Connection(format!(
            "No health endpoint responded at {}",
            self.base_url
        )))
    }

    async fn get_capabilities(&self) -> TarpcResult<Vec<String>> {
        for path in Self::CAPABILITIES_PATHS {
            if let Ok(value) = self.fetch_json(path).await {
                let capabilities = value
                    .get("capabilities")
                    .and_then(|c| c.as_array())
                    .ok_or_else(|| {
                        TarpcClientError::Configuration(
                            "HTTPS capabilities response missing 'capabilities' array".to_string(),
                        )
                    })?;
                let strings: Vec<String> = capabilities
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
                return Ok(strings);
            }
        }
        Err(TarpcClientError::Configuration(
            "No capabilities endpoint responded".to_string(),
        ))
    }
}

/// Protocol priority for primal-to-primal communication
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Protocol {
    /// tarpc - PRIMARY (highest priority)
    Tarpc = 1,
    /// JSON-RPC - SECONDARY
    JsonRpc = 2,
    /// HTTPS - FALLBACK (lowest priority)
    Https = 3,
}

/// Detected protocol for an endpoint
#[derive(Debug, Clone)]
pub struct DetectedProtocol {
    /// Protocol type
    pub protocol: Protocol,
    /// Endpoint string
    pub endpoint: String,
}

/// Detect protocol from endpoint string
///
/// # Arguments
/// * `endpoint` - Endpoint URL (e.g., "<tarpc://localhost:9001>", "<unix:///tmp/service.sock>")
///
/// # Returns
/// Detected protocol or HTTPS as fallback
pub fn detect_protocol(endpoint: &str) -> Protocol {
    if endpoint.starts_with("tarpc://") {
        Protocol::Tarpc
    } else if endpoint.starts_with("unix://") || endpoint.starts_with("ipc://") {
        Protocol::JsonRpc // Unix sockets use JSON-RPC
    } else if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
        Protocol::Https
    } else {
        debug!(
            "Unknown protocol for endpoint '{}', defaulting to HTTPS",
            endpoint
        );
        Protocol::Https
    }
}

/// Try to connect using protocol priority
///
/// Attempts protocols in order:
/// 1. tarpc (if endpoint is tarpc://)
/// 2. JSON-RPC (if endpoint is unix:// or ipc://)
/// 3. HTTPS (if endpoint is http:// or https://)
///
/// # Arguments
/// * `endpoint` - Service endpoint
///
/// # Returns
/// Client connection or error
pub async fn connect_with_priority(endpoint: &str) -> TarpcResult<PrimalConnection> {
    let protocol = detect_protocol(endpoint);

    match protocol {
        Protocol::Tarpc => {
            info!("🚀 Using tarpc (PRIMARY) for {}", endpoint);
            let client = TarpcClient::new(endpoint)?;

            // Test connection
            match client.health().await {
                Ok(health) => {
                    info!("✅ tarpc connection established: {}", health.status);
                    Ok(PrimalConnection::Tarpc(client))
                }
                Err(e) => {
                    warn!("❌ tarpc connection failed: {}", e);
                    Err(e)
                }
            }
        }
        Protocol::JsonRpc => {
            info!("📝 Using JSON-RPC (SECONDARY) for {}", endpoint);
            let socket_path = parse_unix_socket_path(endpoint)?;
            let client = JsonRpcClient::new(&socket_path)
                .map_err(|e| TarpcClientError::Connection(e.to_string()))?;

            // Test connection via health check
            match client.health_check().await {
                Ok(_) => {
                    info!("✅ JSON-RPC connection established");
                    Ok(PrimalConnection::JsonRpc(client))
                }
                Err(e) => {
                    warn!("❌ JSON-RPC connection failed: {}", e);
                    Err(TarpcClientError::Connection(e.to_string()))
                }
            }
        }
        Protocol::Https => {
            info!("🌐 Using HTTPS (FALLBACK) for {}", endpoint);
            connect_https(endpoint).await
        }
    }
}

/// Connect via HTTPS/HTTP with graceful fallback
async fn connect_https(endpoint: &str) -> TarpcResult<PrimalConnection> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| TarpcClientError::Configuration(format!("reqwest client: {e}")))?;

    // Try HTTPS first, then fall back to HTTP
    let urls_to_try: Vec<String> = if endpoint.starts_with("https://") {
        vec![
            endpoint.to_string(),
            endpoint.replacen("https://", "http://", 1),
        ]
    } else {
        vec![endpoint.to_string()]
    };

    for base_url in urls_to_try {
        let scheme = if base_url.starts_with("https://") {
            "HTTPS"
        } else {
            "HTTP"
        };
        info!("🌐 Trying {scheme} for {}", base_url);

        let https_client = HttpsClient {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: client.clone(),
        };

        match https_client.health().await {
            Ok(_) => {
                info!("✅ {scheme} connection established");
                return Ok(PrimalConnection::Https(https_client));
            }
            Err(e) => {
                warn!("❌ {scheme} connection failed: {}", e);
            }
        }
    }

    Err(TarpcClientError::Connection(
        "HTTPS and HTTP fallback both failed".to_string(),
    ))
}

/// Parse Unix socket path from endpoint URL
///
/// Supports: unix:///path/to/sock, ipc:///path/to/sock
fn parse_unix_socket_path(endpoint: &str) -> TarpcResult<std::path::PathBuf> {
    let path_str = endpoint
        .strip_prefix("unix://")
        .or_else(|| endpoint.strip_prefix("ipc://"))
        .ok_or_else(|| {
            TarpcClientError::Configuration(format!(
                "Invalid Unix socket endpoint (expected unix:// or ipc://): {endpoint}"
            ))
        })?;
    Ok(std::path::PathBuf::from(path_str))
}

/// Connection to remote primal (protocol-agnostic wrapper)
#[derive(Clone)]
pub enum PrimalConnection {
    /// tarpc connection (PRIMARY)
    Tarpc(TarpcClient),
    /// JSON-RPC connection (SECONDARY)
    JsonRpc(JsonRpcClient),
    /// HTTPS connection (FALLBACK)
    Https(HttpsClient),
}

impl PrimalConnection {
    /// Get capabilities from remote primal
    pub async fn get_capabilities(&self) -> TarpcResult<Vec<String>> {
        match self {
            PrimalConnection::Tarpc(client) => client.get_capabilities().await,
            PrimalConnection::JsonRpc(client) => {
                let value = client
                    .get_capabilities()
                    .await
                    .map_err(jsonrpc_to_tarpc_error)?;
                let capabilities = value
                    .get("capabilities")
                    .and_then(|c| c.as_array())
                    .ok_or_else(|| {
                        TarpcClientError::Configuration(
                            "JSON-RPC capabilities response missing 'capabilities' array"
                                .to_string(),
                        )
                    })?;
                let strings: Vec<String> = capabilities
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
                Ok(strings)
            }
            PrimalConnection::Https(client) => client.get_capabilities().await,
        }
    }

    /// Check health of remote primal
    pub async fn health(&self) -> TarpcResult<petal_tongue_ipc::HealthStatus> {
        match self {
            PrimalConnection::Tarpc(client) => client.health().await,
            PrimalConnection::JsonRpc(client) => {
                let value = client
                    .health_check()
                    .await
                    .map_err(jsonrpc_to_tarpc_error)?;
                let status = value
                    .get("status")
                    .and_then(|s| s.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let version = value
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let uptime_seconds = value
                    .get("uptime_seconds")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0);
                let capabilities = value
                    .get("capabilities")
                    .or_else(|| value.get("modalities_active"))
                    .and_then(|c| c.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();
                Ok(petal_tongue_ipc::HealthStatus {
                    status,
                    version,
                    uptime_seconds,
                    capabilities,
                    details: std::collections::HashMap::new(),
                })
            }
            PrimalConnection::Https(client) => client.health().await,
        }
    }
}

fn jsonrpc_to_tarpc_error(e: JsonRpcClientError) -> TarpcClientError {
    match e {
        JsonRpcClientError::Connection(s) => TarpcClientError::Connection(s),
        JsonRpcClientError::Timeout(s) => TarpcClientError::Timeout(s),
        JsonRpcClientError::RpcError { message, .. } => TarpcClientError::Rpc(message),
        JsonRpcClientError::Serialization(s) => TarpcClientError::Serialization(s),
        JsonRpcClientError::InvalidResponse(s) => TarpcClientError::Rpc(s),
        JsonRpcClientError::Io(e) => TarpcClientError::Connection(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_protocol() {
        assert_eq!(detect_protocol("tarpc://localhost:9001"), Protocol::Tarpc);
        assert_eq!(
            detect_protocol("unix:///tmp/service.sock"),
            Protocol::JsonRpc
        );
        assert_eq!(detect_protocol("ipc:///tmp/service"), Protocol::JsonRpc);
        assert_eq!(detect_protocol("http://localhost:8080"), Protocol::Https);
        assert_eq!(detect_protocol("https://api.example.com"), Protocol::Https);
    }

    #[test]
    fn test_detect_protocol_unknown_fallback() {
        // Unknown protocols default to HTTPS fallback
        assert_eq!(detect_protocol("unknown://foo"), Protocol::Https);
        assert_eq!(detect_protocol(""), Protocol::Https);
        assert_eq!(detect_protocol("ftp://example.com"), Protocol::Https);
    }

    #[test]
    fn test_protocol_priority() {
        assert!(Protocol::Tarpc < Protocol::JsonRpc);
        assert!(Protocol::JsonRpc < Protocol::Https);
    }

    #[test]
    fn test_protocol_ord() {
        assert!(Protocol::Tarpc < Protocol::Https);
        assert!(Protocol::Tarpc <= Protocol::Tarpc);
        assert!(Protocol::Https >= Protocol::JsonRpc);
    }

    #[test]
    fn test_detected_protocol() {
        let d = DetectedProtocol {
            protocol: Protocol::Tarpc,
            endpoint: "tarpc://localhost:9001".to_string(),
        };
        assert_eq!(d.protocol, Protocol::Tarpc);
        assert_eq!(d.endpoint, "tarpc://localhost:9001");
    }

    #[test]
    fn test_parse_unix_socket_path() {
        let path = super::parse_unix_socket_path("unix:///tmp/service.sock").unwrap();
        assert_eq!(path.to_string_lossy(), "/tmp/service.sock");

        let path = super::parse_unix_socket_path("ipc:///var/run/app.sock").unwrap();
        assert_eq!(path.to_string_lossy(), "/var/run/app.sock");
    }

    #[test]
    fn test_parse_unix_socket_path_invalid() {
        assert!(super::parse_unix_socket_path("tarpc://localhost:9001").is_err());
        assert!(super::parse_unix_socket_path("http://localhost").is_err());
        assert!(super::parse_unix_socket_path("invalid").is_err());
    }

    #[test]
    fn test_parse_unix_socket_path_empty_after_prefix() {
        let path = super::parse_unix_socket_path("unix://").expect("empty path is valid");
        assert_eq!(path.to_string_lossy(), "");
    }

    #[test]
    fn test_detect_protocol_trailing_slash() {
        assert_eq!(detect_protocol("https://api.example.com/"), Protocol::Https);
        assert_eq!(detect_protocol("http://localhost:8080/"), Protocol::Https);
    }

    #[test]
    fn test_detected_protocol_all_variants() {
        let tarpc = DetectedProtocol {
            protocol: Protocol::Tarpc,
            endpoint: "tarpc://host:1".to_string(),
        };
        assert_eq!(tarpc.protocol, Protocol::Tarpc);

        let jsonrpc = DetectedProtocol {
            protocol: Protocol::JsonRpc,
            endpoint: "unix:///tmp/sock".to_string(),
        };
        assert_eq!(jsonrpc.protocol, Protocol::JsonRpc);

        let https = DetectedProtocol {
            protocol: Protocol::Https,
            endpoint: "https://example.com".to_string(),
        };
        assert_eq!(https.protocol, Protocol::Https);
    }

    #[test]
    fn test_protocol_equality() {
        assert_eq!(Protocol::Tarpc, Protocol::Tarpc);
        assert_ne!(Protocol::Tarpc, Protocol::Https);
    }

    #[test]
    fn test_jsonrpc_to_tarpc_error_mapping() {
        use petal_tongue_ipc::{JsonRpcClientError, TarpcClientError};

        let jsonrpc_err = JsonRpcClientError::Connection("conn".to_string());
        let tarpc_err = super::jsonrpc_to_tarpc_error(jsonrpc_err);
        assert!(matches!(tarpc_err, TarpcClientError::Connection(_)));

        let jsonrpc_err = JsonRpcClientError::Timeout("timeout".to_string());
        let tarpc_err = super::jsonrpc_to_tarpc_error(jsonrpc_err);
        assert!(matches!(tarpc_err, TarpcClientError::Timeout(_)));

        let jsonrpc_err = JsonRpcClientError::Serialization("ser".to_string());
        let tarpc_err = super::jsonrpc_to_tarpc_error(jsonrpc_err);
        assert!(matches!(tarpc_err, TarpcClientError::Serialization(_)));
    }

    #[test]
    fn test_jsonrpc_to_tarpc_error_rpc_error() {
        use petal_tongue_ipc::{JsonRpcClientError, TarpcClientError};

        let jsonrpc_err = JsonRpcClientError::RpcError {
            code: -32601,
            message: "Method not found".to_string(),
            data: None,
        };
        let tarpc_err = super::jsonrpc_to_tarpc_error(jsonrpc_err);
        assert!(matches!(tarpc_err, TarpcClientError::Rpc(_)));
    }

    #[test]
    fn test_jsonrpc_to_tarpc_error_invalid_response() {
        use petal_tongue_ipc::{JsonRpcClientError, TarpcClientError};

        let jsonrpc_err = JsonRpcClientError::InvalidResponse("bad json".to_string());
        let tarpc_err = super::jsonrpc_to_tarpc_error(jsonrpc_err);
        assert!(matches!(tarpc_err, TarpcClientError::Rpc(_)));
    }

    #[test]
    fn test_jsonrpc_to_tarpc_error_io() {
        use petal_tongue_ipc::{JsonRpcClientError, TarpcClientError};

        let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
        let jsonrpc_err = JsonRpcClientError::Io(io_err);
        let tarpc_err = super::jsonrpc_to_tarpc_error(jsonrpc_err);
        assert!(matches!(tarpc_err, TarpcClientError::Connection(_)));
    }
}
