//! Songbird discovery client
//!
//! Queries Songbird primal for capability-based discovery of other primals.
//! This is the PRIMARY method for petalTongue to discover live ecosystem topology.
//!
//! MODERN IDIOMATIC RUST:
//! - Aggressive timeouts to prevent hanging
//! - Non-blocking operations throughout
//! - Proper error propagation

use anyhow::{Context, Result};
use petal_tongue_core::types::{PrimalHealthStatus, PrimalInfo};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
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
    /// Discover Songbird's Unix socket
    ///
    /// Searches in biomeOS-standard locations for songbird-<family_id>.sock
    pub fn discover(family_id: Option<&str>) -> Result<Self> {
        let family = family_id
            .map(String::from)
            .or_else(|| std::env::var("FAMILY_ID").ok())
            .unwrap_or_else(|| "nat0".to_string());

        let socket_name = format!("songbird-{}.sock", family);

        // Try XDG_RUNTIME_DIR first
        let search_paths = Self::get_search_paths();

        for base_path in search_paths {
            let socket_path = base_path.join(&socket_name);
            if socket_path.exists() {
                info!("🎵 Found Songbird at: {}", socket_path.display());
                return Ok(Self { socket_path });
            }
        }

        // Songbird not found
        warn!("⚠️ Songbird not found in standard locations");
        warn!("   Searched for: {}", socket_name);
        warn!("   Search paths:");
        for path in Self::get_search_paths() {
            warn!("     - {}", path.display());
        }

        Err(anyhow::anyhow!(
            "Songbird not found. Is it running? (looking for {})",
            socket_name
        ))
    }

    /// Get standard search paths for Unix sockets
    fn get_search_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Priority 1: XDG_RUNTIME_DIR
        if let Ok(xdg_runtime) = std::env::var("XDG_RUNTIME_DIR") {
            paths.push(PathBuf::from(xdg_runtime));
        }

        // Priority 2: /run/user/<uid>
        // SAFETY: getuid() is a safe FFI call that returns the effective user ID
        // without preconditions that could lead to undefined behavior.
        let uid = unsafe { libc::getuid() };
        paths.push(PathBuf::from(format!("/run/user/{}", uid)));

        // Priority 3: /tmp (development)
        paths.push(PathBuf::from("/tmp"));

        paths
    }

    /// Create client with explicit socket path (for testing)
    #[must_use]
    pub fn with_socket_path(socket_path: PathBuf) -> Self {
        Self { socket_path }
    }

    /// Get the socket path (for metadata/debugging)
    #[must_use]
    pub fn socket_path(&self) -> &PathBuf {
        &self.socket_path
    }

    /// Discover primals by capability
    ///
    /// Queries Songbird for all registered primals with the given capability.
    /// Returns list of primals that can provide this capability.
    ///
    /// # Example Capabilities
    /// - "visualization" - primals that provide UI/visualization
    /// - "encryption" - primals that provide encryption (BearDog)
    /// - "storage" - primals that provide persistent storage (NestGate)
    /// - "compute" - primals that provide execution (ToadStool)
    /// - "ai" - primals that provide AI inference (Squirrel)
    pub async fn discover_by_capability(&self, capability: &str) -> Result<Vec<PrimalInfo>> {
        debug!("🔍 Querying Songbird for capability: {}", capability);

        let request = json!({
            "jsonrpc": "2.0",
            "method": "discover_by_capability",
            "params": {
                "capability": capability
            },
            "id": 1
        });

        let result = self.send_request(request).await?;

        // Parse primals from response
        let primals = result
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Expected array of primals"))?;

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
    pub async fn get_all_primals(&self) -> Result<Vec<PrimalInfo>> {
        debug!("🔍 Querying Songbird for all registered primals");

        let request = json!({
            "jsonrpc": "2.0",
            "method": "get_all_primals",
            "params": {},
            "id": 1
        });

        let result = self.send_request(request).await?;

        // Parse primals from response
        let primals = result
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Expected array of primals"))?;

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

    /// Health check - verify Songbird is responding
    pub async fn health_check(&self) -> Result<String> {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "health_check",
            "params": {},
            "id": 1
        });

        let result = self.send_request(request).await?;

        Ok(result["status"].as_str().unwrap_or("unknown").to_string())
    }

    /// Send JSON-RPC request to Songbird
    ///
    /// Uses aggressive timeouts to prevent hanging on unresponsive Songbird.
    async fn send_request(&self, request: Value) -> Result<Value> {
        // CRITICAL: Wrap socket connect in timeout
        let connect_timeout = Duration::from_millis(200);

        let stream =
            match tokio::time::timeout(connect_timeout, UnixStream::connect(&self.socket_path))
                .await
            {
                Ok(Ok(stream)) => stream,
                Ok(Err(e)) => {
                    return Err(e).context(format!(
                        "Failed to connect to Songbird at {}",
                        self.socket_path.display()
                    ));
                }
                Err(_) => {
                    return Err(anyhow::anyhow!(
                        "Connection timeout to Songbird at {}",
                        self.socket_path.display()
                    ));
                }
            };

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        // CRITICAL: Wrap write operations in timeout
        let write_timeout = Duration::from_millis(100);

        let request_json = serde_json::to_string(&request)?;

        match tokio::time::timeout(write_timeout, async {
            writer.write_all(request_json.as_bytes()).await?;
            writer.write_all(b"\n").await?;
            writer.flush().await?;
            Ok::<(), std::io::Error>(())
        })
        .await
        {
            Ok(Ok(())) => {}
            Ok(Err(e)) => return Err(e.into()),
            Err(_) => return Err(anyhow::anyhow!("Write timeout to Songbird")),
        }

        // CRITICAL: Wrap read operation in timeout
        let read_timeout = Duration::from_millis(500);

        let mut line = String::new();
        match tokio::time::timeout(read_timeout, reader.read_line(&mut line)).await {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => return Err(e.into()),
            Err(_) => return Err(anyhow::anyhow!("Read timeout from Songbird")),
        }

        // Parse response
        let response: JsonRpcResponse = serde_json::from_str(&line)?;

        if let Some(error) = response.error {
            anyhow::bail!(
                "Songbird returned error: {} (code: {})",
                error.message,
                error.code
            );
        }

        response
            .result
            .ok_or_else(|| anyhow::anyhow!("No result in Songbird response"))
    }

    /// Parse a primal from Songbird's JSON response
    fn parse_primal(&self, value: &Value) -> Result<PrimalInfo> {
        let id = value["id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'id' field"))?
            .to_string();

        let name = value["name"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'name' field"))?
            .to_string();

        let primal_type = value["primal_type"]
            .as_str()
            .or_else(|| value["type"].as_str())
            .unwrap_or("unknown")
            .to_string();

        let endpoint = value["endpoint"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'endpoint' field"))?
            .to_string();

        let capabilities: Vec<String> = value["capabilities"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(String::from)
                    .collect()
            })
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
                .unwrap()
                .as_secs()
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
    fn test_parse_primal() {
        let client = SongbirdClient::with_socket_path(PathBuf::from("/tmp/test.sock"));

        let json = json!({
            "id": "beardog-123",
            "name": "beardog",
            "primal_type": "beardog",
            "endpoint": "unix:///run/user/1000/beardog-nat0.sock",
            "capabilities": ["encryption", "identity"],
            "health": "healthy",
            "last_seen": 1234567890
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
}
