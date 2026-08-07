// SPDX-License-Identifier: AGPL-3.0-or-later
//! G65 negotiating server: single-socket listener with protocol dispatch.
//!
//! Binds via G66 transport abstraction and for each connection:
//! 1. Attempts protocol negotiation (100 ms timeout)
//! 2. If tarpc selected — connection is ready for binary framing
//! 3. If JSON-RPC (or no negotiation) — connection is ready for newline-delimited JSON

use super::negotiate::{negotiate_server, NegotiationResult};
use super::wire::ProtocolId;
use crate::socket_path;
use petal_tongue_core::transport::{TransportEndpoint, bind_transport};
use tracing::{debug, error, info};

/// G65 negotiating server errors.
#[derive(Debug, thiserror::Error)]
pub enum NegotiateServerError {
    /// Socket path resolution failed.
    #[error("socket path: {0}")]
    SocketPath(#[from] crate::socket_path_error::SocketPathError),
    /// Bind failed.
    #[error("bind {endpoint}: {detail}")]
    Bind {
        /// Endpoint description that failed to bind.
        endpoint: String,
        /// Error detail.
        detail: String,
    },
    /// I/O error during accept loop.
    #[error("accept: {0}")]
    Accept(#[from] std::io::Error),
}

/// G65 negotiating server — transport-agnostic.
///
/// Listens on the configured transport endpoint and dispatches each connection
/// to either tarpc or JSON-RPC based on the negotiation header.
pub struct NegotiateServer {
    endpoint: TransportEndpoint,
}

impl NegotiateServer {
    /// Create from the default negotiated socket path.
    pub fn from_default_path() -> Result<Self, NegotiateServerError> {
        let socket_path = socket_path::get_petaltongue_negotiate_socket_path()?;
        Ok(Self {
            endpoint: TransportEndpoint::uds(socket_path),
        })
    }

    /// The endpoint this server will bind to.
    #[must_use]
    pub fn endpoint(&self) -> &TransportEndpoint {
        &self.endpoint
    }

    /// Run the accept loop. Each connection is spawned into a task that
    /// negotiates protocol and logs the result.
    ///
    /// Full dispatch (routing to tarpc codec or JSON-RPC handler) is wired
    /// in a future phase once the negotiation socket replaces the dual-socket.
    pub async fn serve(self) -> Result<(), NegotiateServerError> {
        let listener = bind_transport(&self.endpoint).await.map_err(|e| {
            NegotiateServerError::Bind {
                endpoint: self.endpoint.to_string(),
                detail: e.to_string(),
            }
        })?;

        info!(
            "G65 negotiate server: {} (single-socket Phase 3)",
            listener.local_addr_display()
        );

        loop {
            let mut stream = listener.accept().await?;
            debug!("G65: connection accepted");

            tokio::spawn(async move {
                let server_supported = ProtocolId::supported();
                match negotiate_server(&mut stream, &server_supported).await {
                    Ok(NegotiationResult::Negotiated(ProtocolId::Tarpc)) => {
                        info!("G65: tarpc negotiated — binary framing ready");
                    }
                    Ok(NegotiationResult::Negotiated(ProtocolId::JsonRpc)) => {
                        info!("G65: jsonrpc negotiated explicitly");
                    }
                    Ok(NegotiationResult::NoNegotiation { buffered }) => {
                        debug!(
                            "G65: no negotiation header ({} bytes buffered), defaulting to JSON-RPC",
                            buffered.len()
                        );
                    }
                    Err(e) => {
                        error!("G65 negotiation error: {e}");
                    }
                }
            });
        }
    }
}

impl Drop for NegotiateServer {
    fn drop(&mut self) {
        if let TransportEndpoint::Uds { path } = &self.endpoint {
            let _ = std::fs::remove_file(path);
        }
    }
}
