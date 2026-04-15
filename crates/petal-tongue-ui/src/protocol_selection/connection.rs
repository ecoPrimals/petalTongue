// SPDX-License-Identifier: AGPL-3.0-or-later
//! Protocol-agnostic connection wrapper and JSON-RPC error mapping.

use petal_tongue_ipc::{
    JsonRpcClient, JsonRpcClientError, TarpcClient, TarpcClientError, TarpcResult,
};

use super::https_client::HttpsClient;
use super::parse::{parse_capabilities_from_json, parse_health_from_json};

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
    ///
    /// # Errors
    ///
    /// Returns an error if the RPC call fails or the response cannot be parsed.
    pub async fn get_capabilities(&self) -> TarpcResult<Vec<String>> {
        match self {
            Self::Tarpc(client) => client.get_capabilities().await,
            Self::JsonRpc(client) => {
                let value = client
                    .get_capabilities()
                    .await
                    .map_err(jsonrpc_to_tarpc_error)?;
                parse_capabilities_from_json(&value)
            }
            Self::Https(client) => client.get_capabilities().await,
        }
    }

    /// Check health of remote primal
    ///
    /// # Errors
    ///
    /// Returns an error if the health check RPC call fails.
    pub async fn health(&self) -> TarpcResult<petal_tongue_ipc::HealthStatus> {
        match self {
            Self::Tarpc(client) => client.health().await,
            Self::JsonRpc(client) => {
                let value = client
                    .health_check()
                    .await
                    .map_err(jsonrpc_to_tarpc_error)?;
                Ok(parse_health_from_json(&value))
            }
            Self::Https(client) => client.health().await,
        }
    }
}

pub fn jsonrpc_to_tarpc_error(e: JsonRpcClientError) -> TarpcClientError {
    match e {
        JsonRpcClientError::Connection(s) => TarpcClientError::Connection(s),
        JsonRpcClientError::Timeout(s) => TarpcClientError::Timeout(s),
        JsonRpcClientError::RpcError { message, .. } => TarpcClientError::Rpc(message),
        JsonRpcClientError::Serialization(s) => TarpcClientError::Serialization(s),
        JsonRpcClientError::InvalidResponse(s) => TarpcClientError::Rpc(s),
        JsonRpcClientError::Io(e) => TarpcClientError::Connection(e.to_string()),
    }
}
