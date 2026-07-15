// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON-RPC client for biomeOS Neural API via `TransportEndpoint`.
//!
//! Uses the ecosystem `TransportEndpoint` abstraction for platform-agnostic
//! local IPC (UDS on Unix, Named Pipes on Windows, TCP everywhere).

use super::types::JsonRpcRequest;
use super::types::JsonRpcResponse;
use crate::transport::{connect_transport, TransportEndpoint};

/// Simple JSON-RPC client for biomeOS Neural API.
///
/// Connects via [`TransportEndpoint`] (platform-agnostic: UDS on Unix,
/// Named Pipe on Windows, or TCP).
#[derive(Debug, Clone)]
pub struct BiomeOsClient {
    /// Transport endpoint for the Neural API socket.
    pub endpoint: TransportEndpoint,
}

impl BiomeOsClient {
    /// Create a client from a socket path (legacy convenience).
    ///
    /// Wraps the path as a [`TransportEndpoint::Uds`] which auto-adapts
    /// to Named Pipe on Windows.
    #[must_use]
    pub fn from_socket_path(socket_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            endpoint: TransportEndpoint::uds(socket_path),
        }
    }

    /// Send a JSON-RPC request and receive the response.
    ///
    /// # Errors
    ///
    /// Returns I/O error on connection failure or invalid response JSON.
    pub async fn call(&self, request: &JsonRpcRequest) -> Result<JsonRpcResponse, std::io::Error> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let mut stream = connect_transport(&self.endpoint)
            .await
            .map_err(std::io::Error::from)?;

        let request_json = serde_json::to_vec(request)?;
        stream.write_all(&request_json).await?;
        stream.write_all(b"\n").await?;

        let mut response_buf = Vec::new();
        stream.read_to_end(&mut response_buf).await?;

        serde_json::from_slice(&response_buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}
