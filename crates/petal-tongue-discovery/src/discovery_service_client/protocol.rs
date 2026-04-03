// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON-RPC wire protocol: request/response framing and socket I/O.

use crate::errors::{DiscoveryError, DiscoveryResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

use super::DiscoveryServiceClient;

/// JSON-RPC 2.0 error
#[derive(Debug, Deserialize, Serialize)]
pub(super) struct JsonRpcError {
    pub(super) code: i32,
    pub(super) message: String,
}

/// JSON-RPC 2.0 response
#[derive(Debug, Deserialize, Serialize)]
pub(super) struct JsonRpcResponse {
    pub(super) jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) error: Option<JsonRpcError>,
    pub(super) id: Value,
}

impl DiscoveryServiceClient {
    /// Send JSON-RPC request to the discovery service
    ///
    /// Uses aggressive timeouts to prevent hanging on an unresponsive peer.
    pub(super) async fn send_request(&self, request: Value) -> DiscoveryResult<Value> {
        use petal_tongue_core::constants::discovery_timeouts;
        let connect_timeout = discovery_timeouts::DISCOVERY_SERVICE_CONNECT_TIMEOUT;

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

        let write_timeout = discovery_timeouts::DISCOVERY_SERVICE_WRITE_TIMEOUT;

        let mut request_bytes = serde_json::to_vec(&request).map_err(DiscoveryError::Json)?;
        request_bytes.push(b'\n');

        match tokio::time::timeout(write_timeout, async {
            writer.write_all(&request_bytes).await?;
            writer.flush().await?;
            Ok::<(), std::io::Error>(())
        })
        .await
        {
            Ok(Ok(())) => {}
            Ok(Err(e)) => return Err(DiscoveryError::Io(e)),
            Err(_) => {
                return Err(DiscoveryError::WriteTimeout {
                    endpoint: "discovery service".to_string(),
                });
            }
        }

        let read_timeout = discovery_timeouts::DISCOVERY_SERVICE_READ_TIMEOUT;

        let mut line = String::new();
        match tokio::time::timeout(read_timeout, reader.read_line(&mut line)).await {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => return Err(DiscoveryError::Io(e)),
            Err(_) => {
                return Err(DiscoveryError::ReadTimeout {
                    endpoint: "discovery service".to_string(),
                });
            }
        }

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
                context: " (discovery service)".to_string(),
            })
    }
}
