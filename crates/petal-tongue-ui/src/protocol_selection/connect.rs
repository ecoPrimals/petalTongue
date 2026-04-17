// SPDX-License-Identifier: AGPL-3.0-or-later
//! Connection orchestration: tarpc, JSON-RPC over Unix socket, HTTPS fallback.

use petal_tongue_ipc::{JsonRpcClient, TarpcClient, TarpcClientError, TarpcResult};
use tracing::{info, warn};

use super::connection::PrimalConnection;
use super::https_client::HttpsClient;
use super::protocol::{Protocol, detect_protocol, https_fallback_urls};

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
///
/// # Errors
///
/// Returns an error if connection fails for the detected protocol (tarpc health check, JSON-RPC health, or HTTPS health).
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

/// Connect via HTTP with graceful fallback (TLS delegated to Songbird)
async fn connect_https(endpoint: &str) -> TarpcResult<PrimalConnection> {
    let client =
        petal_tongue_ipc::LocalHttpClient::with_timeout(std::time::Duration::from_secs(10));

    for base_url in https_fallback_urls(endpoint) {
        let scheme = if base_url.starts_with("https://") {
            "HTTPS"
        } else {
            "HTTP"
        };
        info!("Trying {scheme} for {}", base_url);

        let https_client = HttpsClient {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: client.clone(),
        };

        match https_client.health().await {
            Ok(_) => {
                info!("{scheme} connection established");
                return Ok(PrimalConnection::Https(https_client));
            }
            Err(e) => {
                warn!("{scheme} connection failed: {}", e);
            }
        }
    }

    Err(TarpcClientError::Connection(
        "HTTPS and HTTP fallback both failed".to_string(),
    ))
}

/// Parse Unix socket path from endpoint URL
///
/// Supports: <unix:///path/to/sock>, <ipc:///path/to/sock>
pub fn parse_unix_socket_path(endpoint: &str) -> TarpcResult<std::path::PathBuf> {
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
