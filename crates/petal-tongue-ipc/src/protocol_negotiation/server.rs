// SPDX-License-Identifier: AGPL-3.0-or-later
//! G65 negotiating server: single-socket listener with protocol dispatch.
//!
//! Binds `petaltongue.negotiate.sock` and for each connection:
//! 1. Attempts protocol negotiation (100 ms timeout)
//! 2. If tarpc selected → connection is ready for binary framing
//! 3. If JSON-RPC (or no negotiation) → connection is ready for newline-delimited JSON

use super::negotiate::{negotiate_server, NegotiationResult};
use super::wire::ProtocolId;
use crate::socket_path;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info};

/// G65 negotiating server errors.
#[derive(Debug, thiserror::Error)]
pub enum NegotiateServerError {
    /// Socket path resolution failed.
    #[error("socket path: {0}")]
    SocketPath(#[from] crate::socket_path_error::SocketPathError),
    /// Bind failed.
    #[error("bind {path}: {source}")]
    Bind {
        /// Socket path that failed to bind.
        path: String,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// I/O error during accept loop.
    #[error("accept: {0}")]
    Accept(#[from] std::io::Error),
}

/// G65 negotiating server.
///
/// Listens on `petaltongue.negotiate.sock` and dispatches each connection
/// to either tarpc or JSON-RPC based on the negotiation header.
pub struct NegotiateServer {
    socket_path: PathBuf,
}

impl NegotiateServer {
    /// Create from the default negotiated socket path.
    pub fn from_default_path() -> Result<Self, NegotiateServerError> {
        let socket_path = socket_path::get_petaltongue_negotiate_socket_path()?;
        Ok(Self { socket_path })
    }

    /// The socket path this server will bind to.
    #[must_use]
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    /// Run the accept loop. Each connection is spawned into a task that
    /// negotiates protocol and logs the result.
    ///
    /// Full dispatch (routing to tarpc codec or JSON-RPC handler) is wired
    /// in a future phase once the negotiation socket replaces the dual-socket.
    pub async fn serve(self) -> Result<(), NegotiateServerError> {
        if let Err(e) = std::fs::remove_file(&self.socket_path)
            && e.kind() != std::io::ErrorKind::NotFound
        {
            return Err(NegotiateServerError::Bind {
                path: self.socket_path.display().to_string(),
                source: e,
            });
        }

        if let Some(parent) = self.socket_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let listener =
            tokio::net::UnixListener::bind(&self.socket_path).map_err(|e| {
                NegotiateServerError::Bind {
                    path: self.socket_path.display().to_string(),
                    source: e,
                }
            })?;

        info!(
            "G65 negotiate server: {} (single-socket Phase 3)",
            self.socket_path.display()
        );

        loop {
            let (mut stream, _addr) = listener.accept().await?;
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
        let _ = std::fs::remove_file(&self.socket_path);
    }
}
