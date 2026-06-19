// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::unix_socket_connection;
use crate::unix_socket_rpc_handlers::RpcHandlers;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use super::handshake::{is_btsp_json_announcement, is_phase3_cipher, try_phase3_upgrade_split};

/// Handle a TCP connection with optional BTSP handshake.
///
/// Peeks up to 64 bytes to classify the wire format (same three-way
/// classification as UDS: length-prefixed BTSP, JSON-line BTSP
/// announcement, or plain JSON-RPC). Leading ASCII whitespace is
/// skipped before classification (sweetGrass tolerance pattern).
///
/// After a successful BTSP handshake with a non-null cipher, attempts
/// Phase 3 transport switch to encrypted frame I/O.
///
/// For JSON-line BTSP: the stream is split and wrapped in a BufReader
/// **before** the handshake, and the same BufReader is carried through
/// to Phase 3 negotiate + encrypted framing. This prevents byte loss
/// from a transient BufReader that prefetches beyond the handshake
/// (barraCuda Sprint 51b / coralReef Iter 90 pattern).
pub(super) async fn handle_tcp_with_btsp(
    handlers: &RpcHandlers,
    stream: tokio::net::TcpStream,
    btsp_config: Option<Arc<crate::btsp::BtspHandshakeConfig>>,
    peer_addr: std::net::SocketAddr,
) -> Result<(), unix_socket_connection::ConnectionError> {
    if let Some(ref cfg) = btsp_config {
        let mut peek_buf = [0u8; 64];
        let n = stream.peek(&mut peek_buf).await?;
        if n == 0 {
            debug!("BTSP: TCP connection sent EOF before any data, rejecting");
            return Ok(());
        }
        let peeked = &peek_buf[..n];

        let first_non_ws = peeked.iter().find(|b| !b.is_ascii_whitespace());
        let is_json_start = first_non_ws == Some(&b'{');

        let is_json_line = if !is_json_start {
            false
        } else if is_btsp_json_announcement(peeked) {
            debug!("BTSP: JSON-line protocol announcement on TCP, routing to JSON-line handshake");
            true
        } else {
            warn!(
                "BTSP Phase 2: rejecting unauthenticated TCP connection — \
                 plain JSON-RPC without handshake (PT-09 enforcement)"
            );
            return Ok(());
        };

        let ctx = crate::method_gate::CallerContext::tcp(peer_addr);
        if is_json_line {
            return run_tcp_json_line_btsp(handlers, stream, cfg, &ctx).await;
        }
        return run_tcp_length_prefixed_btsp(handlers, stream, cfg, &ctx).await;
    }
    let ctx = crate::method_gate::CallerContext::tcp(peer_addr);
    unix_socket_connection::handle_connection(handlers, stream, &ctx).await?;
    Ok(())
}

/// TCP JSON-line BTSP: split first, BufReader survives through Phase 3.
async fn run_tcp_json_line_btsp(
    handlers: &RpcHandlers,
    stream: tokio::net::TcpStream,
    cfg: &crate::btsp::BtspHandshakeConfig,
    ctx: &crate::method_gate::CallerContext,
) -> Result<(), unix_socket_connection::ConnectionError> {
    let (reader, writer) = tokio::io::split(stream);
    let mut buf_reader = tokio::io::BufReader::new(reader);
    let mut pin_writer = writer;

    let handshake_result =
        match crate::btsp::relay_json_line_handshake_split(&mut buf_reader, &mut pin_writer, cfg)
            .await
        {
            Ok(result) => {
                debug!(
                    "BTSP authenticated on TCP (JSON-line): session={}",
                    result.session_token
                );
                result
            }
            Err(e) => {
                error!("BTSP JSON-line handshake failed on TCP, rejecting connection: {e}");
                return Ok(());
            }
        };

    if is_phase3_cipher(&handshake_result.cipher) {
        if let Some(ref session_key) = handshake_result.session_key {
            return try_phase3_upgrade_split(
                handlers,
                buf_reader,
                pin_writer,
                &handshake_result.cipher,
                session_key,
                ctx,
            )
            .await;
        }
        info!(
            "Phase 3: cipher={} but no session_key from verify, staying plaintext",
            handshake_result.cipher
        );
    }

    unix_socket_connection::handle_connection_split(handlers, buf_reader, pin_writer, ctx).await
}

/// TCP length-prefixed BTSP: handshake reads raw frames, no BufReader.
async fn run_tcp_length_prefixed_btsp(
    handlers: &RpcHandlers,
    mut stream: tokio::net::TcpStream,
    cfg: &crate::btsp::BtspHandshakeConfig,
    ctx: &crate::method_gate::CallerContext,
) -> Result<(), unix_socket_connection::ConnectionError> {
    let handshake_result = match crate::btsp::perform_server_handshake(&mut stream, cfg).await {
        Ok(result) => {
            debug!(
                "BTSP authenticated on TCP (length-prefixed): session={}",
                result.session_token
            );
            result
        }
        Err(e) => {
            error!("BTSP handshake failed on TCP, rejecting connection: {e}");
            return Ok(());
        }
    };

    if is_phase3_cipher(&handshake_result.cipher) {
        if let Some(ref session_key) = handshake_result.session_key {
            let (reader, writer) = tokio::io::split(stream);
            let buf_reader = tokio::io::BufReader::new(reader);
            return try_phase3_upgrade_split(
                handlers,
                buf_reader,
                writer,
                &handshake_result.cipher,
                session_key,
                ctx,
            )
            .await;
        }
        info!(
            "Phase 3: cipher={} but no session_key from verify, staying plaintext",
            handshake_result.cipher
        );
    }

    unix_socket_connection::handle_connection(handlers, stream, ctx).await
}
