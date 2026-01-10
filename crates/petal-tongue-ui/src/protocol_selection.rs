//! Protocol selection logic for inter-primal communication
//!
//! Implements ecosystem-standard protocol priority:
//! 1. tarpc (PRIMARY) - High-performance binary RPC
//! 2. JSON-RPC (SECONDARY) - Universal, debuggable
//! 3. HTTPS (FALLBACK) - External/browser access

use petal_tongue_ipc::{TarpcClient, TarpcResult};
use tracing::{debug, info, warn};

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
/// * `endpoint` - Endpoint URL (e.g., "tarpc://localhost:9001", "unix:///tmp/service.sock")
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
            // TODO: Implement JSON-RPC client connection
            // For now, return error to force fallback
            Err(petal_tongue_ipc::TarpcClientError::Configuration(
                "JSON-RPC client not yet implemented for primal-to-primal".to_string(),
            ))
        }
        Protocol::Https => {
            info!("🌐 Using HTTPS (FALLBACK) for {}", endpoint);
            // TODO: Implement HTTPS client connection
            Err(petal_tongue_ipc::TarpcClientError::Configuration(
                "HTTPS client not yet implemented for primal-to-primal".to_string(),
            ))
        }
    }
}

/// Connection to remote primal (protocol-agnostic wrapper)
#[derive(Clone)]
pub enum PrimalConnection {
    /// tarpc connection (PRIMARY)
    Tarpc(TarpcClient),
    /// JSON-RPC connection (SECONDARY)
    JsonRpc(/* TODO */),
    /// HTTPS connection (FALLBACK)
    Https(/* TODO */),
}

impl PrimalConnection {
    /// Get capabilities from remote primal
    pub async fn get_capabilities(&self) -> TarpcResult<Vec<String>> {
        match self {
            PrimalConnection::Tarpc(client) => client.get_capabilities().await,
            PrimalConnection::JsonRpc(/* client */) => {
                Err(petal_tongue_ipc::TarpcClientError::Configuration(
                    "JSON-RPC not yet implemented".to_string(),
                ))
            }
            PrimalConnection::Https(/* client */) => {
                Err(petal_tongue_ipc::TarpcClientError::Configuration(
                    "HTTPS not yet implemented".to_string(),
                ))
            }
        }
    }

    /// Check health of remote primal
    pub async fn health(&self) -> TarpcResult<petal_tongue_ipc::HealthStatus> {
        match self {
            PrimalConnection::Tarpc(client) => client.health().await,
            PrimalConnection::JsonRpc(/* client */) => {
                Err(petal_tongue_ipc::TarpcClientError::Configuration(
                    "JSON-RPC not yet implemented".to_string(),
                ))
            }
            PrimalConnection::Https(/* client */) => {
                Err(petal_tongue_ipc::TarpcClientError::Configuration(
                    "HTTPS not yet implemented".to_string(),
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_protocol() {
        assert_eq!(detect_protocol("tarpc://localhost:9001"), Protocol::Tarpc);
        assert_eq!(detect_protocol("unix:///tmp/service.sock"), Protocol::JsonRpc);
        assert_eq!(detect_protocol("ipc:///tmp/service"), Protocol::JsonRpc);
        assert_eq!(detect_protocol("http://localhost:8080"), Protocol::Https);
        assert_eq!(detect_protocol("https://api.example.com"), Protocol::Https);
    }

    #[test]
    fn test_protocol_priority() {
        assert!(Protocol::Tarpc < Protocol::JsonRpc);
        assert!(Protocol::JsonRpc < Protocol::Https);
    }
}

