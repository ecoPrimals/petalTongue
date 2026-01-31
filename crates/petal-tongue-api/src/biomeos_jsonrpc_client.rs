//! BiomeOS API Client (JSON-RPC over Unix Sockets)
//!
//! TRUE PRIMAL architecture: Uses JSON-RPC 2.0 over Unix sockets,
//! not HTTP/REST. Connects to BiomeOS via the Neural API provider.
//!
//! # Migration from HTTP
//!
//! This module replaces the HTTP-based BiomeOSClient with a proper
//! JSON-RPC implementation per PRIMAL_IPC_PROTOCOL.md.
//!
//! # Standards Compliance
//!
//! - Protocol: JSON-RPC 2.0 over Unix sockets
//! - Transport: tokio::net::UnixStream
//! - Methods: Semantic naming (neural_api.*)
//! - Discovery: Capability-based, no hardcoding

use petal_tongue_core::{PrimalInfo, TopologyEdge};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use tokio::net::UnixStream;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, info, warn};

/// BiomeOS JSON-RPC client (TRUE PRIMAL architecture)
pub struct BiomeOSJsonRpcClient {
    /// Socket path (e.g., /run/user/1000/biomeos-neural-api.sock)
    socket_path: PathBuf,
    /// Request ID counter
    request_id: std::sync::atomic::AtomicU64,
}

impl BiomeOSJsonRpcClient {
    /// Create a new JSON-RPC client
    ///
    /// # Socket Path Discovery
    ///
    /// 1. Environment variable: `BIOMEOS_SOCKET`
    /// 2. XDG runtime: `/run/user/<uid>/biomeos-neural-api.sock`
    /// 3. Fallback: `/tmp/biomeos-neural-api.sock`
    pub fn new() -> Result<Self> {
        let socket_path = Self::discover_socket_path()?;
        
        Ok(Self {
            socket_path,
            request_id: std::sync::atomic::AtomicU64::new(1),
        })
    }
    
    /// Create client with explicit socket path
    pub fn with_socket_path(socket_path: impl Into<PathBuf>) -> Self {
        Self {
            socket_path: socket_path.into(),
            request_id: std::sync::atomic::AtomicU64::new(1),
        }
    }
    
    /// Discover BiomeOS socket path
    fn discover_socket_path() -> Result<PathBuf> {
        // 1. Check environment variable
        if let Ok(path) = std::env::var("BIOMEOS_SOCKET") {
            return Ok(PathBuf::from(path));
        }
        
        // 2. Check XDG runtime directory
        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            let path = PathBuf::from(runtime_dir).join("biomeos-neural-api.sock");
            if path.exists() {
                return Ok(path);
            }
        }
        
        // 3. Try user runtime directory
        if let Ok(uid) = std::env::var("UID") {
            let path = PathBuf::from(format!("/run/user/{}/biomeos-neural-api.sock", uid));
            if path.exists() {
                return Ok(path);
            }
        }
        
        // 4. Fallback to /tmp
        let fallback = PathBuf::from("/tmp/biomeos-neural-api.sock");
        if fallback.exists() {
            return Ok(fallback);
        }
        
        // No socket found - return default and let connection fail with helpful error
        Ok(PathBuf::from("/tmp/biomeos-neural-api.sock"))
    }
    
    /// Check if BiomeOS is available
    pub async fn is_available(&self) -> bool {
        match tokio::time::timeout(
            std::time::Duration::from_millis(100),
            UnixStream::connect(&self.socket_path)
        ).await {
            Ok(Ok(_)) => true,
            _ => false,
        }
    }
    
    /// Health check (semantic: neural_api.health)
    pub async fn health_check(&self) -> Result<bool> {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "neural_api.health",
            "params": {},
            "id": self.next_request_id(),
        });
        
        match self.send_request(&request).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    /// Discover primals (semantic: neural_api.get_primals)
    pub async fn discover_primals(&self) -> Result<Vec<PrimalInfo>> {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "neural_api.get_primals",
            "params": {},
            "id": self.next_request_id(),
        });
        
        let result = self.send_request(&request).await
            .context("Failed to discover primals from BiomeOS")?;
        
        let primals: Vec<PrimalInfo> = serde_json::from_value(result)
            .context("Failed to parse primals from BiomeOS response")?;
        
        info!("✅ Discovered {} primals via JSON-RPC", primals.len());
        
        Ok(primals)
    }
    
    /// Get topology (semantic: neural_api.get_topology)
    pub async fn get_topology(&self) -> Result<Vec<TopologyEdge>> {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "neural_api.get_topology",
            "params": {},
            "id": self.next_request_id(),
        });
        
        let result = self.send_request(&request).await
            .context("Failed to get topology from BiomeOS")?;
        
        let edges: Vec<TopologyEdge> = serde_json::from_value(result)
            .context("Failed to parse topology from BiomeOS response")?;
        
        debug!("✅ Retrieved {} topology edges via JSON-RPC", edges.len());
        
        Ok(edges)
    }
    
    /// Send a JSON-RPC request
    async fn send_request(&self, request: &Value) -> Result<Value> {
        // Connect to BiomeOS
        let mut stream = UnixStream::connect(&self.socket_path).await
            .with_context(|| {
                format!(
                    "Failed to connect to BiomeOS at {}\n\
                    \n\
                    Troubleshooting:\n\
                    - Ensure BiomeOS nucleus is running\n\
                    - Check BIOMEOS_SOCKET environment variable\n\
                    - Verify socket permissions\n\
                    - Check XDG_RUNTIME_DIR is set correctly",
                    self.socket_path.display()
                )
            })?;
        
        // Send request (line-delimited JSON-RPC)
        let request_str = serde_json::to_string(request)?;
        stream.write_all(format!("{}\n", request_str).as_bytes()).await?;
        stream.flush().await?;
        
        // Read response
        let (reader, _) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut response_line = String::new();
        
        reader.read_line(&mut response_line).await
            .context("Failed to read response from BiomeOS")?;
        
        // Parse response
        let response: Value = serde_json::from_str(&response_line)
            .context("Failed to parse JSON-RPC response")?;
        
        // Check for JSON-RPC error
        if let Some(error) = response.get("error") {
            anyhow::bail!(
                "BiomeOS returned JSON-RPC error: {}\n\
                \n\
                This indicates BiomeOS received the request but encountered an error.",
                error
            );
        }
        
        // Extract result
        response.get("result")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No result field in JSON-RPC response"))
    }
    
    /// Get next request ID
    fn next_request_id(&self) -> u64 {
        self.request_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_socket_path_discovery() {
        // Should not panic, even if socket doesn't exist
        let _client = BiomeOSJsonRpcClient::new();
    }
    
    #[tokio::test]
    async fn test_biomeos_unavailable() {
        // Use non-existent socket
        let client = BiomeOSJsonRpcClient::with_socket_path("/tmp/nonexistent-biomeos.sock");
        
        // Should return false, not panic
        let available = client.is_available().await;
        assert!(!available);
    }
}
