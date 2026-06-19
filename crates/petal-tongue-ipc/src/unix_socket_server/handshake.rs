// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::unix_socket_connection;
use crate::unix_socket_rpc_handlers::RpcHandlers;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Outcome of UDS BTSP handshake classification.
///
/// Distinguishes plain JSON-RPC (filesystem-authenticated, should be served)
/// from actual handshake failures (should be rejected).
pub(super) enum UdsHandshakeOutcome {
    /// BTSP handshake completed successfully.
    Authenticated(crate::btsp::HandshakeResult),
    /// Plain JSON-RPC on UDS — no BTSP handshake attempted.
    /// UDS is filesystem-authenticated; serve via method gate.
    PlainJsonRpc,
    /// Connection should be rejected (EOF, peek error, or failed handshake).
    Reject,
}

/// riboCipher signal prefix: `[0xEC, 0x01]`.
///
/// cellMembrane probes prepend this 2-byte prefix before JSON-RPC on UDS.
/// Per Wave 113 guideStone amendment, all primals MUST accept and strip
/// this prefix rather than rejecting the connection.
pub(super) const RIBOCIPHER_PREFIX: [u8; 2] = [0xEC, 0x01];

/// Peek the UDS `BufReader` for a riboCipher `[0xEC, 0x01]` prefix and
/// consume it if present. This runs before BTSP classification so the
/// remaining bytes start with `{` (JSON-RPC) or a BTSP framing byte.
pub(super) async fn strip_ribocipher_prefix<R: tokio::io::AsyncBufRead + Unpin>(reader: &mut R) {
    use tokio::io::AsyncBufReadExt;

    match reader.fill_buf().await {
        Ok(buf) if buf.len() >= 2 && buf[..2] == RIBOCIPHER_PREFIX => {
            debug!("riboCipher prefix detected on UDS — stripping [0xEC, 0x01]");
            reader.consume(2);
        }
        _ => {}
    }
}

/// Classify a peek buffer as BTSP JSON-line protocol announcement.
///
/// Whitespace-tolerant: leading ASCII whitespace is ignored before
/// checking for `{` (sweetGrass `detect_protocol` pattern).
///
/// Three wire formats (after whitespace trim):
/// - Non-`{` first byte → length-prefixed BTSP framing
/// - `{` + contains `"protocol"` → BTSP JSON-line announcement
/// - `{` + no `"protocol"` → plain JSON-RPC (biomeOS composition)
pub(super) fn is_btsp_json_announcement(buf: &[u8]) -> bool {
    let trimmed = buf
        .iter()
        .position(|b| !b.is_ascii_whitespace())
        .map_or(&[][..], |pos| &buf[pos..]);
    trimmed.first() == Some(&b'{')
        && trimmed
            .windows(b"\"protocol\"".len())
            .any(|w| w == b"\"protocol\"")
}

/// Handle a UDS connection with optional BTSP handshake.
///
/// Uses `BufReader::fill_buf()` to non-destructively peek initial bytes.
/// After a successful BTSP handshake with a non-null cipher, the first
/// post-handshake message is expected to be `btsp.negotiate` with a
/// `client_nonce`. If negotiate succeeds, the connection transitions to
/// encrypted frame I/O (Phase 3). Otherwise it falls back to NDJSON.
pub(super) async fn handle_uds_with_btsp(
    handlers: &RpcHandlers,
    stream: tokio::net::UnixStream,
    btsp_config: Option<Arc<crate::btsp::BtspHandshakeConfig>>,
) -> Result<(), unix_socket_connection::ConnectionError> {
    let (reader, mut writer) = stream.into_split();
    let mut buf_reader = tokio::io::BufReader::new(reader);

    strip_ribocipher_prefix(&mut buf_reader).await;

    let ctx = crate::method_gate::CallerContext::unix();

    if let Some(ref cfg) = btsp_config {
        let outcome = run_uds_handshake(&mut buf_reader, &mut writer, cfg).await?;

        match outcome {
            UdsHandshakeOutcome::Reject => {
                warn!(
                    family_id = %cfg.family_id,
                    "BTSP Phase 2: rejecting UDS connection (handshake failed or EOF)"
                );
                return Ok(());
            }
            UdsHandshakeOutcome::PlainJsonRpc => {
                debug!(
                    family_id = %cfg.family_id,
                    "BTSP Phase 2: UDS plain JSON-RPC — serving via filesystem auth"
                );
                unix_socket_connection::handle_connection_split(handlers, buf_reader, writer, &ctx)
                    .await?;
                return Ok(());
            }
            UdsHandshakeOutcome::Authenticated(ref hs) => {
                if is_phase3_cipher(&hs.cipher) {
                    if let Some(ref session_key) = hs.session_key {
                        return try_phase3_upgrade_split(
                            handlers,
                            buf_reader,
                            writer,
                            &hs.cipher,
                            session_key,
                            &ctx,
                        )
                        .await;
                    }
                    info!(
                        "Phase 3: cipher={} but no session_key from verify, staying plaintext",
                        hs.cipher
                    );
                }

                unix_socket_connection::handle_connection_split(handlers, buf_reader, writer, &ctx)
                    .await?;
            }
        }
    } else {
        unix_socket_connection::handle_connection_split(handlers, buf_reader, writer, &ctx).await?;
    }
    Ok(())
}

/// Run the BTSP handshake on a UDS connection, classifying and dispatching.
async fn run_uds_handshake(
    buf_reader: &mut tokio::io::BufReader<tokio::net::unix::OwnedReadHalf>,
    writer: &mut tokio::net::unix::OwnedWriteHalf,
    cfg: &crate::btsp::BtspHandshakeConfig,
) -> Result<UdsHandshakeOutcome, unix_socket_connection::ConnectionError> {
    use tokio::io::AsyncBufReadExt;

    enum BtspRoute {
        LengthPrefixed,
        JsonLine,
        PlainJsonRpc,
    }

    let route = match buf_reader.fill_buf().await {
        Ok(buf) if !buf.is_empty() => {
            let first_non_ws = buf.iter().find(|b| !b.is_ascii_whitespace());
            if first_non_ws != Some(&b'{') {
                BtspRoute::LengthPrefixed
            } else if is_btsp_json_announcement(buf) {
                debug!("BTSP: JSON-line announcement on UDS, routing to JSON-line handshake");
                BtspRoute::JsonLine
            } else {
                debug!("BTSP: plain JSON-RPC on UDS, serving via filesystem auth");
                BtspRoute::PlainJsonRpc
            }
        }
        Ok(_) => {
            debug!("BTSP: UDS connection sent EOF before any data");
            return Ok(UdsHandshakeOutcome::Reject);
        }
        Err(e) => {
            error!("BTSP: UDS peek failed: {e}");
            return Ok(UdsHandshakeOutcome::Reject);
        }
    };

    match route {
        BtspRoute::LengthPrefixed => {
            match crate::btsp::perform_server_handshake_split(buf_reader, writer, cfg).await {
                Ok(result) => {
                    debug!(
                        "BTSP authenticated on UDS (length-prefixed): session={}",
                        result.session_token
                    );
                    Ok(UdsHandshakeOutcome::Authenticated(result))
                }
                Err(e) => {
                    error!("BTSP handshake failed on UDS, rejecting connection: {e}");
                    Ok(UdsHandshakeOutcome::Reject)
                }
            }
        }
        BtspRoute::JsonLine => {
            match crate::btsp::relay_json_line_handshake_split(buf_reader, writer, cfg).await {
                Ok(result) => {
                    debug!(
                        "BTSP authenticated on UDS (JSON-line): session={}",
                        result.session_token
                    );
                    Ok(UdsHandshakeOutcome::Authenticated(result))
                }
                Err(e) => {
                    error!("BTSP JSON-line handshake failed on UDS, rejecting: {e}");
                    Ok(UdsHandshakeOutcome::Reject)
                }
            }
        }
        BtspRoute::PlainJsonRpc => Ok(UdsHandshakeOutcome::PlainJsonRpc),
    }
}

/// Whether the cipher requires Phase 3 encrypted transport.
pub(super) fn is_phase3_cipher(cipher: &str) -> bool {
    cipher == "chacha20-poly1305" || cipher == "chacha20_poly1305"
}

/// Attempt Phase 3 negotiate and encrypted stream on a split connection.
///
/// Reads the first post-handshake line. If it's `btsp.negotiate` with
/// a `client_nonce`, derives session keys and enters the encrypted
/// frame loop. Otherwise, processes the line as normal JSON-RPC and
/// falls back to the plaintext NDJSON loop.
pub(super) async fn try_phase3_upgrade_split<R, W>(
    handlers: &RpcHandlers,
    mut reader: R,
    mut writer: W,
    cipher_hint: &str,
    session_key: &[u8],
    ctx: &crate::method_gate::CallerContext,
) -> Result<(), unix_socket_connection::ConnectionError>
where
    R: tokio::io::AsyncBufRead + tokio::io::AsyncRead + Unpin,
    W: tokio::io::AsyncWrite + Unpin,
{
    let negotiate_result = crate::btsp::phase3::try_phase3_negotiate(
        &mut reader,
        &mut writer,
        handlers,
        cipher_hint,
        ctx,
    )
    .await?;

    if let Some(neg) = negotiate_result {
        let keys = crate::btsp::phase3::SessionKeys::derive(
            session_key,
            &neg.client_nonce,
            &neg.server_nonce,
            true,
        )
        .map_err(|e| {
            error!("Phase 3 key derivation failed: {e}");
            unix_socket_connection::ConnectionError::Io(std::io::Error::other(format!(
                "Phase 3 key derivation: {e}"
            )))
        })?;

        let session = crate::btsp::phase3::Phase3Session::new(&keys).map_err(|e| {
            error!("Phase 3 session init failed: {e}");
            unix_socket_connection::ConnectionError::Io(std::io::Error::other(format!(
                "Phase 3 session init: {e}"
            )))
        })?;

        info!("Phase 3: encrypted frame I/O active (ChaCha20-Poly1305)");
        crate::btsp::phase3::handle_encrypted_stream(
            handlers,
            &mut reader,
            &mut writer,
            &session,
            ctx,
        )
        .await
    } else {
        debug!("Phase 3: negotiate not completed, continuing with plaintext NDJSON");
        unix_socket_connection::handle_connection_split(handlers, reader, writer, ctx).await
    }
}
