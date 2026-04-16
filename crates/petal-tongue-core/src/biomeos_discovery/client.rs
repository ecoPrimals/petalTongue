// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON-RPC client over Unix stream to biomeOS Neural API.

use super::types::JsonRpcRequest;
use super::types::JsonRpcResponse;

/// Simple JSON-RPC client for biomeOS
#[derive(Debug, Clone)]
pub struct BiomeOsClient {
    pub socket_path: String,
}

impl BiomeOsClient {
    pub async fn call(&self, request: &JsonRpcRequest) -> Result<JsonRpcResponse, std::io::Error> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::UnixStream;

        let mut stream = UnixStream::connect(&self.socket_path).await?;

        let request_json = serde_json::to_vec(request)?;
        stream.write_all(&request_json).await?;
        stream.write_all(b"\n").await?;

        let mut response_buf = Vec::new();
        stream.read_to_end(&mut response_buf).await?;

        serde_json::from_slice(&response_buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}
