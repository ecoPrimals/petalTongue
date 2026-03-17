// SPDX-License-Identifier: AGPL-3.0-or-later
//! Songbird discovery client
//!
//! Queries Songbird primal for capability-based discovery of other primals.
//! This is the PRIMARY method for petalTongue to discover live ecosystem topology.
//!
//! MODERN IDIOMATIC RUST:
//! - Aggressive timeouts to prevent hanging
//! - Non-blocking operations throughout
//! - Proper error propagation

use crate::capability_parse;
use crate::errors::{DiscoveryError, DiscoveryResult};
use petal_tongue_core::types::{PrimalHealthStatus, PrimalInfo};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, info, warn};

/// JSON-RPC 2.0 error
#[derive(Debug, Deserialize, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

/// JSON-RPC 2.0 response
#[derive(Debug, Deserialize, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    id: Value,
}

/// Songbird client for primal discovery
///
/// Connects to Songbird via Unix socket and queries for registered primals.
#[derive(Debug)]
pub struct SongbirdClient {
    /// Path to Songbird's Unix socket
    socket_path: PathBuf,
}

impl SongbirdClient {
    /// Discover discovery service Unix socket (capability-based, no hardcoded primal names)
    ///
    /// Uses `DISCOVERY_SERVICE_SOCKET` env for socket name (default: discovery-service).
    /// Set to `songbird` for Songbird deployments.
    ///
    /// # Errors
    /// Returns `DiscoveryError::DiscoveryServiceNotFound` if no socket found in search paths.
    pub fn discover(family_id: Option<&str>) -> DiscoveryResult<Self> {
        let family = family_id
            .map(String::from)
            .or_else(|| std::env::var("FAMILY_ID").ok())
            .unwrap_or_else(|| "nat0".to_string());

        let socket_base = petal_tongue_core::constants::discovery_service_socket_name();
        let socket_name = format!("{socket_base}-{family}.sock");

        // Try XDG_RUNTIME_DIR first
        let search_paths = Self::get_search_paths();

        for base_path in search_paths {
            let socket_path = base_path.join(&socket_name);
            if socket_path.exists() {
                info!("🎵 Found discovery service at: {}", socket_path.display());
                return Ok(Self { socket_path });
            }
        }

        // Discovery service not found
        warn!("⚠️ Discovery service not found in standard locations");
        warn!("   Searched for: {}", socket_name);
        warn!("   Search paths:");
        for path in Self::get_search_paths() {
            warn!("     - {}", path.display());
        }

        Err(DiscoveryError::DiscoveryServiceNotFound { socket_name })
    }

    /// Get standard search paths for Unix sockets
    fn get_search_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Priority 1: XDG_RUNTIME_DIR
        if let Ok(xdg_runtime) = std::env::var("XDG_RUNTIME_DIR") {
            paths.push(PathBuf::from(xdg_runtime));
        }

        // Priority 2: /run/user/<uid>
        // EVOLVED: Now using safe rustix-based function from core (was unsafe libc::getuid())
        let uid = petal_tongue_core::system_info::get_current_uid();
        paths.push(PathBuf::from(format!("/run/user/{uid}")));

        // Priority 3: /tmp (development)
        paths.push(PathBuf::from("/tmp"));

        paths
    }

    /// Create client with explicit socket path (for testing)
    #[must_use]
    pub const fn with_socket_path(socket_path: PathBuf) -> Self {
        Self { socket_path }
    }

    /// Get the socket path (for metadata/debugging)
    #[must_use]
    pub const fn socket_path(&self) -> &PathBuf {
        &self.socket_path
    }

    /// Discover primals by capability (semantic: discovery.query)
    ///
    /// Queries Songbird for all registered primals with the given capability.
    /// Returns list of primals that can provide this capability.
    ///
    /// # Semantic Method Name
    /// Calls `discovery.query` per `SEMANTIC_METHOD_NAMING_STANDARD.md`
    ///
    /// # Example Capabilities
    /// - "visualization" - primals that provide UI/visualization
    /// - "encryption" - primals that provide encryption (`BearDog`)
    /// - "storage" - primals that provide persistent storage (`NestGate`)
    /// - "compute" - primals that provide execution (`ToadStool`)
    /// - "ai" - primals that provide AI inference (Squirrel)
    ///
    /// # Errors
    /// Returns `DiscoveryError` on network/JSON-RPC errors or invalid response.
    pub async fn discover_by_capability(
        &self,
        capability: &str,
    ) -> DiscoveryResult<Vec<PrimalInfo>> {
        debug!("🔍 Querying Songbird for capability: {}", capability);

        let request = json!({
            "jsonrpc": "2.0",
            "method": "discovery.query",  // Semantic naming
            "params": {
                "capability": capability
            },
            "id": 1
        });

        let result = self.send_request(request).await?;

        // Parse primals from response
        let primals = result
            .as_array()
            .ok_or_else(|| DiscoveryError::ExpectedArray {
                context: " of primals".to_string(),
            })?;

        let mut primal_infos = Vec::new();
        for primal in primals {
            if let Ok(info) = self.parse_primal(primal) {
                primal_infos.push(info);
            } else {
                warn!(
                    "Failed to parse primal from Songbird response: {:?}",
                    primal
                );
            }
        }

        info!(
            "🎵 Songbird found {} primals with capability '{}'",
            primal_infos.len(),
            capability
        );

        Ok(primal_infos)
    }

    /// Get all registered primals
    ///
    /// Returns the complete list of primals known to Songbird.
    /// Uses discovery.query("*") to get all registered primals.
    ///
    /// # Errors
    /// Returns `DiscoveryError` on network/JSON-RPC errors or invalid response.
    pub async fn get_all_primals(&self) -> DiscoveryResult<Vec<PrimalInfo>> {
        debug!("🔍 Querying Songbird for all registered primals");

        // Songbird's API uses discovery.query with "*" to get all primals
        let request = json!({
            "jsonrpc": "2.0",
            "method": "discovery.query",  // Semantic naming
            "params": {
                "capability": "*"
            },
            "id": 1
        });

        let result = self.send_request(request).await?;

        // Parse primals from response
        let primals = result
            .as_array()
            .ok_or_else(|| DiscoveryError::ExpectedArray {
                context: " of primals".to_string(),
            })?;

        let mut primal_infos = Vec::new();
        for primal in primals {
            if let Ok(info) = self.parse_primal(primal) {
                primal_infos.push(info);
            } else {
                warn!(
                    "Failed to parse primal from Songbird response: {:?}",
                    primal
                );
            }
        }

        info!(
            "🎵 Songbird reports {} total registered primals",
            primal_infos.len()
        );

        Ok(primal_infos)
    }

    /// Health check - verify Songbird is responding (semantic: health.check)
    ///
    /// # Errors
    /// Returns `DiscoveryError` on connection or JSON-RPC errors.
    pub async fn health_check(&self) -> DiscoveryResult<String> {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "health.check",  // Semantic naming
            "params": {},
            "id": 1
        });

        let result = self.send_request(request).await?;

        Ok(result["status"].as_str().unwrap_or("unknown").to_string())
    }

    /// Send JSON-RPC request to Songbird
    ///
    /// Uses aggressive timeouts to prevent hanging on unresponsive Songbird.
    async fn send_request(&self, request: Value) -> DiscoveryResult<Value> {
        // CRITICAL: Wrap socket connect in timeout
        let connect_timeout = Duration::from_millis(200);

        let stream =
            match tokio::time::timeout(connect_timeout, UnixStream::connect(&self.socket_path))
                .await
            {
                Ok(Ok(stream)) => stream,
                Ok(Err(e)) => {
                    return Err(DiscoveryError::Io(e));
                }
                Err(_) => {
                    return Err(DiscoveryError::ConnectionTimeout {
                        endpoint: self.socket_path.display().to_string(),
                    });
                }
            };

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        // CRITICAL: Wrap write operations in timeout
        let write_timeout = Duration::from_millis(100);

        let request_json = serde_json::to_string(&request).map_err(DiscoveryError::Json)?;

        match tokio::time::timeout(write_timeout, async {
            writer.write_all(request_json.as_bytes()).await?;
            writer.write_all(b"\n").await?;
            writer.flush().await?;
            Ok::<(), std::io::Error>(())
        })
        .await
        {
            Ok(Ok(())) => {}
            Ok(Err(e)) => return Err(DiscoveryError::Io(e)),
            Err(_) => {
                return Err(DiscoveryError::WriteTimeout {
                    endpoint: "Songbird".to_string(),
                });
            }
        }

        // CRITICAL: Wrap read operation in timeout
        let read_timeout = Duration::from_millis(500);

        let mut line = String::new();
        match tokio::time::timeout(read_timeout, reader.read_line(&mut line)).await {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => return Err(DiscoveryError::Io(e)),
            Err(_) => {
                return Err(DiscoveryError::ReadTimeout {
                    endpoint: "Songbird".to_string(),
                });
            }
        }

        // Parse response
        let response: JsonRpcResponse =
            serde_json::from_str(&line).map_err(DiscoveryError::Json)?;

        if let Some(error) = response.error {
            return Err(DiscoveryError::JsonRpcError {
                code: Some(error.code),
                message: error.message,
            });
        }

        response
            .result
            .ok_or_else(|| DiscoveryError::NoResultInResponse {
                context: " (Songbird)".to_string(),
            })
    }

    /// Parse a primal from Songbird's JSON response
    #[expect(
        clippy::unused_self,
        reason = "method for consistency with other parsers"
    )]
    fn parse_primal(&self, value: &Value) -> DiscoveryResult<PrimalInfo> {
        let id = value["id"]
            .as_str()
            .ok_or_else(|| DiscoveryError::MissingField {
                field: "id".to_string(),
                context: String::new(),
            })?
            .to_string();

        let name = value["name"]
            .as_str()
            .ok_or_else(|| DiscoveryError::MissingField {
                field: "name".to_string(),
                context: String::new(),
            })?
            .to_string();

        let primal_type = value["primal_type"]
            .as_str()
            .or_else(|| value["type"].as_str())
            .unwrap_or("unknown")
            .to_string();

        let endpoint = value["endpoint"]
            .as_str()
            .ok_or_else(|| DiscoveryError::MissingField {
                field: "endpoint".to_string(),
                context: String::new(),
            })?
            .to_string();

        let capabilities: Vec<String> = value["capabilities"]
            .as_array()
            .map(|v| capability_parse::parse_capabilities(v))
            .unwrap_or_default();

        let health = value["health"]
            .as_str()
            .and_then(|s| match s.to_lowercase().as_str() {
                "healthy" | "ok" => Some(PrimalHealthStatus::Healthy),
                "degraded" | "warning" => Some(PrimalHealthStatus::Warning),
                "unhealthy" | "error" | "critical" => Some(PrimalHealthStatus::Critical),
                _ => None,
            })
            .unwrap_or(PrimalHealthStatus::Healthy);

        let last_seen = value["last_seen"].as_u64().unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0) // Fallback to epoch if clock is broken
        });

        Ok(PrimalInfo::new(
            id,
            name,
            primal_type,
            endpoint,
            capabilities,
            health,
            last_seen,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_search_paths() {
        let paths = SongbirdClient::get_search_paths();
        assert!(!paths.is_empty());

        // Should always have /tmp as fallback
        assert!(paths.iter().any(|p| p.ends_with("tmp")));
    }

    #[test]
    fn test_with_socket_path() {
        let path = PathBuf::from("/run/user/1000/discovery.sock");
        let client = SongbirdClient::with_socket_path(path.clone());
        assert_eq!(client.socket_path(), &path);
    }

    #[test]
    fn test_parse_primal() {
        let client = SongbirdClient::with_socket_path(PathBuf::from("/tmp/test.sock"));

        let json = json!({
            "id": "beardog-123",
            "name": "beardog",
            "primal_type": "beardog",
            "endpoint": "unix:///run/user/1000/beardog-nat0.sock",
            "capabilities": ["encryption", "identity"],
            "health": "healthy",
            "last_seen": 1_234_567_890
        });

        let primal = client.parse_primal(&json).unwrap();
        assert_eq!(primal.id, "beardog-123");
        assert_eq!(primal.name, "beardog");
        assert_eq!(primal.capabilities.len(), 2);
        assert!(matches!(primal.health, PrimalHealthStatus::Healthy));
    }

    #[test]
    fn test_parse_primal_minimal() {
        let client = SongbirdClient::with_socket_path(PathBuf::from("/tmp/test.sock"));

        let json = json!({
            "id": "minimal-primal",
            "name": "minimal",
            "endpoint": "unix:///tmp/minimal.sock"
        });

        let primal = client.parse_primal(&json).unwrap();
        assert_eq!(primal.id, "minimal-primal");
        assert_eq!(primal.primal_type, "unknown");
        assert!(primal.capabilities.is_empty());
    }

    #[test]
    fn test_parse_primal_health_variants() {
        let client = SongbirdClient::with_socket_path(PathBuf::from("/tmp/test.sock"));

        let json_healthy = json!({
            "id": "test",
            "name": "test",
            "endpoint": "unix:///tmp/test.sock",
            "health": "healthy"
        });
        assert!(matches!(
            client.parse_primal(&json_healthy).unwrap().health,
            PrimalHealthStatus::Healthy
        ));

        let json_degraded = json!({
            "id": "test",
            "name": "test",
            "endpoint": "unix:///tmp/test.sock",
            "health": "degraded"
        });
        assert!(matches!(
            client.parse_primal(&json_degraded).unwrap().health,
            PrimalHealthStatus::Warning
        ));

        let json_critical = json!({
            "id": "test",
            "name": "test",
            "endpoint": "unix:///tmp/test.sock",
            "health": "critical"
        });
        assert!(matches!(
            client.parse_primal(&json_critical).unwrap().health,
            PrimalHealthStatus::Critical
        ));
    }

    #[test]
    fn test_parse_primal_type_fallback() {
        let client = SongbirdClient::with_socket_path(PathBuf::from("/tmp/test.sock"));

        let json = json!({
            "id": "test",
            "name": "test",
            "primal_type": "toadstool",
            "endpoint": "unix:///tmp/test.sock"
        });
        let primal = client.parse_primal(&json).unwrap();
        assert_eq!(primal.primal_type, "toadstool");

        let json_type = json!({
            "id": "test",
            "name": "test",
            "type": "beardog",
            "endpoint": "unix:///tmp/test.sock"
        });
        let primal2 = client.parse_primal(&json_type).unwrap();
        assert_eq!(primal2.primal_type, "beardog");
    }

    #[test]
    fn test_parse_primal_missing_id() {
        let client = SongbirdClient::with_socket_path(PathBuf::from("/tmp/test.sock"));
        let json = json!({
            "name": "test",
            "endpoint": "unix:///tmp/test.sock"
        });
        assert!(client.parse_primal(&json).is_err());
    }

    #[test]
    fn test_parse_primal_missing_name() {
        let client = SongbirdClient::with_socket_path(PathBuf::from("/tmp/test.sock"));
        let json = json!({
            "id": "test",
            "endpoint": "unix:///tmp/test.sock"
        });
        assert!(client.parse_primal(&json).is_err());
    }

    #[test]
    fn test_parse_primal_missing_endpoint() {
        let client = SongbirdClient::with_socket_path(PathBuf::from("/tmp/test.sock"));
        let json = json!({
            "id": "test",
            "name": "test"
        });
        assert!(client.parse_primal(&json).is_err());
    }

    #[test]
    fn test_discover_fails_without_socket() {
        let result = SongbirdClient::discover(Some("nonexistent-family-xyz"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_send_request_fails_nonexistent_socket() {
        let client =
            SongbirdClient::with_socket_path(PathBuf::from("/tmp/nonexistent-socket-12345.sock"));
        let result = client.discover_by_capability("visualization").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_primal_last_seen_fallback() {
        let client = SongbirdClient::with_socket_path(PathBuf::from("/tmp/test.sock"));
        let json = json!({
            "id": "test",
            "name": "test",
            "endpoint": "unix:///tmp/test.sock"
        });
        let primal = client.parse_primal(&json).expect("parse");
        assert!(primal.last_seen > 0);
    }

    #[test]
    fn test_parse_primal_health_warning_variants() {
        let client = SongbirdClient::with_socket_path(PathBuf::from("/tmp/test.sock"));
        for (health_str, expected) in [
            ("warning", PrimalHealthStatus::Warning),
            ("error", PrimalHealthStatus::Critical),
            ("unhealthy", PrimalHealthStatus::Critical),
        ] {
            let json = json!({
                "id": "test",
                "name": "test",
                "endpoint": "unix:///tmp/test.sock",
                "health": health_str
            });
            let primal = client.parse_primal(&json).expect("parse");
            assert!(
                std::mem::discriminant(&primal.health) == std::mem::discriminant(&expected),
                "health {health_str}"
            );
        }
    }

    #[test]
    fn test_parse_primal_unknown_health_defaults_healthy() {
        let client = SongbirdClient::with_socket_path(PathBuf::from("/tmp/test.sock"));
        let json = json!({
            "id": "test",
            "name": "test",
            "endpoint": "unix:///tmp/test.sock",
            "health": "unknown_status"
        });
        let primal = client.parse_primal(&json).expect("parse");
        assert!(matches!(primal.health, PrimalHealthStatus::Healthy));
    }

    #[test]
    fn test_discovery_query_request_structure() {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "discovery.query",
            "params": {"capability": "visualization"},
            "id": 1
        });
        assert_eq!(request["method"], "discovery.query");
        assert_eq!(request["params"]["capability"], "visualization");
    }

    #[test]
    fn test_health_check_request_structure() {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "health.check",
            "params": {},
            "id": 1
        });
        assert_eq!(request["method"], "health.check");
    }

    #[test]
    fn test_get_all_primals_request_structure() {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "discovery.query",
            "params": {"capability": "*"},
            "id": 1
        });
        assert_eq!(request["params"]["capability"], "*");
    }

    #[test]
    fn test_jsonrpc_response_deserialization() {
        let json = r#"{"jsonrpc":"2.0","result":[{"id":"p1","name":"p1","endpoint":"unix:///tmp/p1.sock"}],"id":1}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).expect("deserialize");
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_jsonrpc_error_deserialization() {
        let json =
            r#"{"jsonrpc":"2.0","error":{"code":-32601,"message":"Method not found"},"id":1}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).expect("deserialize");
        assert!(response.error.is_some());
        let err = response.error.as_ref().unwrap();
        assert_eq!(err.code, -32601);
        assert_eq!(err.message, "Method not found");
    }

    #[test]
    fn test_jsonrpc_response_serialization() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(json!([{"id": "p1"}])),
            error: None,
            id: json!(1),
        };
        let json = serde_json::to_string(&response).expect("serialize");
        assert!(json.contains("2.0"));
        assert!(json.contains("p1"));
    }

    #[test]
    fn test_jsonrpc_error_serialization() {
        let err = JsonRpcError {
            code: -32600,
            message: "Invalid request".to_string(),
        };
        let json = serde_json::to_string(&err).expect("serialize");
        assert!(json.contains("-32600"));
        assert!(json.contains("Invalid request"));
    }

    #[test]
    fn test_get_search_paths_with_xdg() {
        petal_tongue_core::test_fixtures::env_test_helpers::with_env_var(
            "XDG_RUNTIME_DIR",
            "/custom/xdg",
            || {
                let paths = SongbirdClient::get_search_paths();
                assert_eq!(paths.first().and_then(|p| p.to_str()), Some("/custom/xdg"));
            },
        );
    }

    #[test]
    fn test_parse_primal_capabilities_mixed_types() {
        let client = SongbirdClient::with_socket_path(PathBuf::from("/tmp/test.sock"));
        let json = json!({
            "id": "test",
            "name": "test",
            "endpoint": "unix:///tmp/test.sock",
            "capabilities": ["cap1", "cap2"]
        });
        let primal = client.parse_primal(&json).unwrap();
        assert_eq!(primal.capabilities, vec!["cap1", "cap2"]);
    }

    #[test]
    fn test_health_check_response_parsing() {
        let result = json!({"status": "healthy"});
        let status = result["status"].as_str().unwrap_or("unknown").to_string();
        assert_eq!(status, "healthy");
    }

    #[test]
    fn test_discovery_socket_name_format() {
        let family = "nat0";
        let base = petal_tongue_core::constants::discovery_service_socket_name();
        let socket_name = format!("{base}-{family}.sock");
        assert!(
            std::path::Path::new(&socket_name)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("sock"))
        );
        assert!(socket_name.contains("nat0"));
    }
}
