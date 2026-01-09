//! Unix socket discovery provider
//!
//! Discovers primals via Unix domain sockets by scanning /tmp for .sock files
//! and querying their capabilities via JSON-RPC.

use crate::{DiscoveryError, DiscoveryProvider, ProviderMetadata, VisualizationCapability};
use anyhow::Result;
use async_trait::async_trait;
use petal_tongue_core::{PrimalHealthStatus, PrimalInfo};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, info, warn};

/// Unix socket discovery provider
///
/// Discovers primals by scanning for Unix socket files and querying their capabilities.
pub struct UnixSocketProvider {
    /// Directories to search for Unix sockets
    search_paths: Vec<PathBuf>,
}

impl UnixSocketProvider {
    /// Create a new Unix socket provider
    pub fn new() -> Self {
        Self {
            search_paths: vec![
                PathBuf::from("/tmp"),
                PathBuf::from("/var/run/ecoPrimals"),
            ],
        }
    }
    
    /// Discover all primals via Unix sockets
    pub async fn discover(&self) -> Result<Vec<PrimalInfo>> {
        let mut primals = Vec::new();
        
        for search_path in &self.search_paths {
            if !search_path.exists() {
                debug!("Search path does not exist: {}", search_path.display());
                continue;
            }
            
            let entries = match std::fs::read_dir(search_path) {
                Ok(entries) => entries,
                Err(e) => {
                    warn!("Failed to read directory {}: {}", search_path.display(), e);
                    continue;
                }
            };
            
            for entry in entries.flatten() {
                let path = entry.path();
                
                // Look for .sock files (Unix domain sockets)
                if path.extension().and_then(|s| s.to_str()) == Some("sock") {
                    match self.probe_socket(&path).await {
                        Ok(info) => {
                            info!("🔍 Discovered primal via Unix socket: {} ({})", 
                                info.name, path.display());
                            primals.push(info);
                        }
                        Err(e) => {
                            debug!("Failed to probe socket {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }
        
        Ok(primals)
    }
    
    /// Probe a Unix socket for primal information
    async fn probe_socket(&self, path: &Path) -> Result<PrimalInfo> {
        // Connect to the Unix socket
        let stream = UnixStream::connect(path).await?;
        
        // Send get_capabilities request
        let request = json!({
            "jsonrpc": "2.0",
            "method": "get_capabilities",
            "params": {},
            "id": 1
        });
        
        let response = self.send_request(stream, request).await?;
        
        // Parse response into PrimalInfo
        self.parse_capabilities_response(path, response)
    }
    
    /// Send a JSON-RPC request and receive response
    async fn send_request(&self, stream: UnixStream, request: Value) -> Result<Value> {
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        
        // Send request
        let request_json = serde_json::to_string(&request)?;
        writer.write_all(request_json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
        
        // Read response
        let mut line = String::new();
        reader.read_line(&mut line).await?;
        
        // Parse response
        let response: JsonRpcResponse = serde_json::from_str(&line)?;
        
        if let Some(error) = response.error {
            anyhow::bail!("JSON-RPC error: {} (code: {})", error.message, error.code);
        }
        
        response.result.ok_or_else(|| anyhow::anyhow!("No result in response"))
    }
    
    /// Parse capabilities response into PrimalInfo
    fn parse_capabilities_response(&self, path: &Path, result: Value) -> Result<PrimalInfo> {
        let capabilities: Vec<String> = result["capabilities"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_str())
            .map(String::from)
            .collect();
        
        let node_id = result["node_id"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();
        
        let version = result["version"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();
        
        // Derive primal type from socket name or capabilities
        let primal_type = self.infer_primal_type(path, &capabilities);
        
        // Derive name from socket name
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        Ok(PrimalInfo::new(
            node_id.clone(),
            name,
            primal_type,
            format!("unix://{}", path.display()),
            capabilities,
            PrimalHealthStatus::Healthy,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        ))
    }
    
    /// Infer primal type from socket path and capabilities
    fn infer_primal_type(&self, path: &Path, capabilities: &[String]) -> String {
        // Try to infer from socket name
        let socket_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        
        if socket_name.contains("beardog") {
            return "beardog".to_string();
        } else if socket_name.contains("songbird") {
            return "songbird".to_string();
        } else if socket_name.contains("petaltongue") || socket_name.contains("petal-tongue") {
            return "petaltongue".to_string();
        } else if socket_name.contains("biomeos") {
            return "biomeos".to_string();
        }
        
        // Try to infer from capabilities
        if capabilities.iter().any(|c| c.contains("ui.") || c.contains("visualization.")) {
            "petaltongue".to_string()
        } else if capabilities.iter().any(|c| c.contains("security") || c.contains("encryption")) {
            "beardog".to_string()
        } else if capabilities.iter().any(|c| c.contains("discovery") || c.contains("p2p")) {
            "songbird".to_string()
        } else {
            "unknown".to_string()
        }
    }
}

impl Default for UnixSocketProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// JSON-RPC response for deserialization
#[derive(Debug, Deserialize, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    id: Value,
}

#[derive(Debug, Deserialize, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unix_socket_provider_creation() {
        let provider = UnixSocketProvider::new();
        assert_eq!(provider.search_paths.len(), 2);
        assert!(provider.search_paths[0].ends_with("tmp"));
    }

    #[test]
    fn test_infer_primal_type_from_socket_name() {
        let provider = UnixSocketProvider::new();
        
        let path = PathBuf::from("/tmp/beardog-node-alpha.sock");
        let primal_type = provider.infer_primal_type(&path, &[]);
        assert_eq!(primal_type, "beardog");
        
        let path = PathBuf::from("/tmp/songbird-node-alpha.sock");
        let primal_type = provider.infer_primal_type(&path, &[]);
        assert_eq!(primal_type, "songbird");
        
        let path = PathBuf::from("/tmp/petaltongue-node-alpha.sock");
        let primal_type = provider.infer_primal_type(&path, &[]);
        assert_eq!(primal_type, "petaltongue");
    }

    #[test]
    fn test_infer_primal_type_from_capabilities() {
        let provider = UnixSocketProvider::new();
        let path = PathBuf::from("/tmp/unknown.sock");
        
        let capabilities = vec!["ui.desktop-interface".to_string()];
        let primal_type = provider.infer_primal_type(&path, &capabilities);
        assert_eq!(primal_type, "petaltongue");
        
        let capabilities = vec!["security".to_string(), "encryption".to_string()];
        let primal_type = provider.infer_primal_type(&path, &capabilities);
        assert_eq!(primal_type, "beardog");
        
        let capabilities = vec!["discovery".to_string(), "p2p".to_string()];
        let primal_type = provider.infer_primal_type(&path, &capabilities);
        assert_eq!(primal_type, "songbird");
    }
}

