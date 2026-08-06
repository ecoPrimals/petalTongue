// SPDX-License-Identifier: AGPL-3.0-or-later
//! Async negotiation functions (client-side and server-side).

use super::wire::{NegotiationError, ProtocolId, ProtocolRequest, ProtocolResponse};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tracing::{debug, info, warn};

/// Negotiation timeout for detecting whether the first line is a protocol header.
const NEGOTIATION_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(100);

/// Client-side negotiation: send supported protocols, receive server's choice.
///
/// # Errors
///
/// Returns `NegotiationError` if I/O fails or the server response is malformed.
pub async fn negotiate_client<T>(
    transport: &mut T,
    supported: &[ProtocolId],
) -> Result<ProtocolId, NegotiationError>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    let request = ProtocolRequest {
        supported: supported.to_vec(),
    };
    let wire = request.to_wire();

    debug!("G65 client sending: {:?}", wire.trim());
    transport
        .write_all(wire.as_bytes())
        .await
        .map_err(|e| NegotiationError::Io(e.to_string()))?;
    transport
        .flush()
        .await
        .map_err(|e| NegotiationError::Io(e.to_string()))?;

    let mut reader = BufReader::new(transport);
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .await
        .map_err(|e| NegotiationError::Io(e.to_string()))?;

    let response = ProtocolResponse::from_wire(&response_line)?;
    info!("G65 negotiated: {}", response.selected);
    Ok(response.selected)
}

/// Server-side negotiation: read client header, select best protocol, respond.
///
/// Returns `Some(protocol)` if negotiation succeeded, or `None` if the first
/// bytes were not a negotiation header (caller should assume JSON-RPC and replay
/// the already-read bytes).
///
/// # Design
///
/// Uses a 100 ms timeout on the first line read. If no data arrives or the line
/// is not a `PROTOCOLS:` header, returns `None` with the buffered bytes so the
/// caller can feed them to the JSON-RPC handler transparently.
///
/// # Errors
///
/// Returns `NegotiationError` only for I/O failures during the write-back.
pub async fn negotiate_server<T>(
    transport: &mut T,
    server_supported: &[ProtocolId],
) -> Result<NegotiationResult, NegotiationError>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    let mut reader = BufReader::new(transport);
    let mut first_line = String::new();

    let read_result = tokio::time::timeout(NEGOTIATION_TIMEOUT, reader.read_line(&mut first_line))
        .await;

    match read_result {
        Ok(Ok(0)) => {
            // EOF before any data — no negotiation.
            Ok(NegotiationResult::NoNegotiation {
                buffered: Vec::new(),
            })
        }
        Ok(Ok(_n)) => {
            if first_line.starts_with("PROTOCOLS: ") {
                let request = ProtocolRequest::from_wire(&first_line)
                    .map_err(|e| NegotiationError::Io(format!("parse: {e}")))?;

                let selected = select_protocol(&request.supported, server_supported);
                let response = ProtocolResponse { selected };
                let wire = response.to_wire();

                reader
                    .get_mut()
                    .write_all(wire.as_bytes())
                    .await
                    .map_err(|e| NegotiationError::Io(e.to_string()))?;
                reader
                    .get_mut()
                    .flush()
                    .await
                    .map_err(|e| NegotiationError::Io(e.to_string()))?;

                info!("G65 server selected: {selected}");
                Ok(NegotiationResult::Negotiated(selected))
            } else {
                // Not a protocol header — buffer for JSON-RPC replay.
                warn!("G65: no negotiation header, assuming JSON-RPC");
                Ok(NegotiationResult::NoNegotiation {
                    buffered: first_line.into_bytes(),
                })
            }
        }
        Ok(Err(e)) => {
            warn!("G65 read error: {e}");
            Ok(NegotiationResult::NoNegotiation {
                buffered: Vec::new(),
            })
        }
        Err(_elapsed) => {
            // Timeout — no negotiation header arrived.
            debug!("G65: timeout waiting for negotiation header, assuming JSON-RPC");
            Ok(NegotiationResult::NoNegotiation {
                buffered: Vec::new(),
            })
        }
    }
}

/// Select the best protocol both sides support.
///
/// Walks the client's preference list and returns the first protocol that
/// appears in the server's supported list. Falls back to JSON-RPC if no
/// common protocol exists.
#[must_use]
pub fn select_protocol(client_prefs: &[ProtocolId], server_supported: &[ProtocolId]) -> ProtocolId {
    for client_proto in client_prefs {
        if server_supported.contains(client_proto) {
            return *client_proto;
        }
    }
    ProtocolId::JsonRpc
}

/// Result of server-side protocol negotiation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NegotiationResult {
    /// A protocol was successfully negotiated.
    Negotiated(ProtocolId),
    /// No negotiation occurred — first bytes were not a protocol header.
    /// The `buffered` bytes should be prepended to the stream for JSON-RPC.
    NoNegotiation {
        /// Bytes already read from the stream (to be replayed).
        buffered: Vec<u8>,
    },
}
