// SPDX-License-Identifier: AGPL-3.0-or-later
//! BTSP Phase 3: encrypted frame I/O using ChaCha20-Poly1305 AEAD.
//!
//! After a successful `btsp.negotiate` that selects `chacha20-poly1305`,
//! both sides derive directional session keys via HKDF-SHA256 and switch
//! to length-prefixed encrypted frames:
//!
//! ```text
//! [4B BE length][12B random nonce][ciphertext + 16B Poly1305 tag]
//! ```
//!
//! Each frame carries exactly one JSON-RPC object (UTF-8, no trailing
//! newline required). Random 12-byte nonces per frame (CSPRNG).

use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use hkdf::Hkdf;
use sha2::Sha256;
use zeroize::{Zeroize, ZeroizeOnDrop};

use super::error::BtspHandshakeError;
use super::framing::MAX_FRAME_SIZE;
use crate::json_rpc::{JsonRpcRequest, JsonRpcResponse, error_codes};
use crate::unix_socket_rpc_handlers::RpcHandlers;

const NONCE_SIZE: usize = 12;
const TAG_SIZE: usize = 16;
const KEY_SIZE: usize = 32;

/// Directional session keys for Phase 3 encrypted communication.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SessionKeys {
    /// Key used for encrypting outbound frames.
    pub encrypt_key: [u8; KEY_SIZE],
    /// Key used for decrypting inbound frames.
    pub decrypt_key: [u8; KEY_SIZE],
}

impl SessionKeys {
    /// Derive directional session keys from the handshake material.
    ///
    /// Per ecosystem standard (W81):
    /// - `salt = client_nonce || server_nonce`
    /// - `ikm  = handshake_key` (session_key from `btsp.session.verify`)
    /// - `info = "btsp-session-v1-c2s"` → client-to-server key
    /// - `info = "btsp-session-v1-s2c"` → server-to-client key
    pub fn derive(
        handshake_key: &[u8],
        client_nonce: &[u8],
        server_nonce: &[u8],
        is_server: bool,
    ) -> Result<Self, BtspHandshakeError> {
        let mut salt = Vec::with_capacity(client_nonce.len() + server_nonce.len());
        salt.extend_from_slice(client_nonce);
        salt.extend_from_slice(server_nonce);

        let hk = Hkdf::<Sha256>::new(Some(&salt), handshake_key);

        let mut c2s_key = [0u8; KEY_SIZE];
        let mut s2c_key = [0u8; KEY_SIZE];

        hk.expand(b"btsp-session-v1-c2s", &mut c2s_key)
            .map_err(|_| BtspHandshakeError::KeyDerivationFailed {
                context: "HKDF expand c2s",
            })?;
        hk.expand(b"btsp-session-v1-s2c", &mut s2c_key)
            .map_err(|_| BtspHandshakeError::KeyDerivationFailed {
                context: "HKDF expand s2c",
            })?;

        if is_server {
            Ok(Self {
                encrypt_key: s2c_key,
                decrypt_key: c2s_key,
            })
        } else {
            Ok(Self {
                encrypt_key: c2s_key,
                decrypt_key: s2c_key,
            })
        }
    }
}

/// Phase 3 encrypted session using ChaCha20-Poly1305 with random nonces.
pub struct Phase3Session {
    encrypt_cipher: ChaCha20Poly1305,
    decrypt_cipher: ChaCha20Poly1305,
}

impl Phase3Session {
    /// Create a new Phase 3 session from derived keys.
    pub fn new(keys: &SessionKeys) -> Result<Self, BtspHandshakeError> {
        let encrypt_cipher = ChaCha20Poly1305::new_from_slice(&keys.encrypt_key).map_err(|_| {
            BtspHandshakeError::KeyDerivationFailed {
                context: "encrypt cipher init",
            }
        })?;
        let decrypt_cipher = ChaCha20Poly1305::new_from_slice(&keys.decrypt_key).map_err(|_| {
            BtspHandshakeError::KeyDerivationFailed {
                context: "decrypt cipher init",
            }
        })?;
        Ok(Self {
            encrypt_cipher,
            decrypt_cipher,
        })
    }

    /// Encrypt plaintext into `[12B random nonce][ciphertext + 16B tag]`.
    pub fn encrypt_frame(&self, plaintext: &[u8]) -> Result<Vec<u8>, BtspHandshakeError> {
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        rand::Rng::fill(&mut rand::thread_rng(), &mut nonce_bytes);

        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = self.encrypt_cipher.encrypt(nonce, plaintext).map_err(|_| {
            BtspHandshakeError::Phase3Crypto {
                context: "encrypt frame",
            }
        })?;

        let mut frame = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        frame.extend_from_slice(&nonce_bytes);
        frame.extend_from_slice(&ciphertext);
        Ok(frame)
    }

    /// Decrypt `[12B nonce][ciphertext + 16B tag]` → plaintext.
    pub fn decrypt_frame(&self, frame: &[u8]) -> Result<Vec<u8>, BtspHandshakeError> {
        if frame.len() < NONCE_SIZE + TAG_SIZE {
            return Err(BtspHandshakeError::Phase3Crypto {
                context: "frame too short for nonce + tag",
            });
        }
        let nonce = Nonce::from_slice(&frame[..NONCE_SIZE]);
        let ciphertext = &frame[NONCE_SIZE..];

        self.decrypt_cipher.decrypt(nonce, ciphertext).map_err(|_| {
            BtspHandshakeError::Phase3Crypto {
                context: "decrypt frame (authentication failed)",
            }
        })
    }
}

/// Read one encrypted frame: `[4B BE length][payload]` → decrypt → plaintext.
pub async fn read_encrypted_frame<R: tokio::io::AsyncReadExt + Unpin>(
    reader: &mut R,
    session: &Phase3Session,
) -> Result<Vec<u8>, BtspHandshakeError> {
    let len = reader
        .read_u32()
        .await
        .map_err(|e| BtspHandshakeError::Io {
            context: "read encrypted frame length",
            source: e,
        })?;
    if len > MAX_FRAME_SIZE {
        return Err(BtspHandshakeError::Io {
            context: "encrypted frame too large",
            source: std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("frame {len} > max {MAX_FRAME_SIZE}"),
            ),
        });
    }
    let mut buf = vec![0u8; len as usize];
    reader
        .read_exact(&mut buf)
        .await
        .map_err(|e| BtspHandshakeError::Io {
            context: "read encrypted frame payload",
            source: e,
        })?;
    session.decrypt_frame(&buf)
}

/// Encrypt plaintext → write `[4B BE length][payload]`.
pub async fn write_encrypted_frame<W: tokio::io::AsyncWriteExt + Unpin>(
    writer: &mut W,
    session: &Phase3Session,
    plaintext: &[u8],
) -> Result<(), BtspHandshakeError> {
    let payload = session.encrypt_frame(plaintext)?;
    let len = u32::try_from(payload.len()).map_err(|_| BtspHandshakeError::Io {
        context: "encrypted frame too large for u32",
        source: std::io::Error::new(std::io::ErrorKind::InvalidData, "frame too large"),
    })?;
    writer
        .write_u32(len)
        .await
        .map_err(|e| BtspHandshakeError::Io {
            context: "write encrypted frame length",
            source: e,
        })?;
    writer
        .write_all(&payload)
        .await
        .map_err(|e| BtspHandshakeError::Io {
            context: "write encrypted frame payload",
            source: e,
        })?;
    writer.flush().await.map_err(|e| BtspHandshakeError::Io {
        context: "flush encrypted frame",
        source: e,
    })?;
    Ok(())
}

/// Encrypted frame JSON-RPC dispatch loop (Phase 3 transport).
///
/// Replaces `handle_connection_split` after a successful Phase 3 negotiate.
/// Each inbound frame is decrypted, parsed as JSON-RPC, dispatched, and
/// the response is encrypted and written back.
pub async fn handle_encrypted_stream<R, W>(
    handler: &RpcHandlers,
    reader: &mut R,
    writer: &mut W,
    session: &Phase3Session,
    ctx: &crate::method_gate::CallerContext,
) -> Result<(), crate::unix_socket_connection::ConnectionError>
where
    R: tokio::io::AsyncReadExt + Unpin,
    W: tokio::io::AsyncWriteExt + Unpin,
{
    loop {
        let plaintext = match read_encrypted_frame(reader, session).await {
            Ok(pt) => pt,
            Err(BtspHandshakeError::Io { source, .. })
                if source.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                tracing::debug!("Phase 3: client disconnected (EOF)");
                break;
            }
            Err(e) => {
                tracing::error!("Phase 3 read error: {e}");
                break;
            }
        };

        let response = match serde_json::from_slice::<JsonRpcRequest>(&plaintext) {
            Ok(request) => {
                tracing::debug!("Phase 3: method={}, id={}", request.method, request.id);
                let per_req = ctx.clone().with_token_from_params(&request.params);
                handler.handle_request(request, &per_req).await
            }
            Err(e) => {
                tracing::error!("Phase 3: parse error: {e}");
                JsonRpcResponse::error(
                    serde_json::json!(null),
                    error_codes::PARSE_ERROR,
                    format!("Parse error: {e}"),
                )
            }
        };

        let response_bytes = serde_json::to_vec(&response)
            .map_err(crate::unix_socket_connection::ConnectionError::Serialize)?;

        if let Err(e) = write_encrypted_frame(writer, session, &response_bytes).await {
            tracing::error!("Phase 3 write error: {e}");
            break;
        }
    }

    Ok(())
}

/// Result of a Phase 3 negotiate exchange.
pub(crate) struct NegotiateResult {
    /// Client nonce (raw bytes, decoded from base64).
    pub client_nonce: Vec<u8>,
    /// Server nonce (raw bytes).
    pub server_nonce: Vec<u8>,
}

/// Try to handle the first post-handshake message as a `btsp.negotiate`.
///
/// If the first line is `btsp.negotiate`, extracts `client_nonce`, generates
/// `server_nonce`, writes the negotiate response, and returns the result.
///
/// If the first line is a regular JSON-RPC request, processes it normally
/// and returns `None` (caller should continue in NDJSON mode).
///
/// Returns `Some(NegotiateResult)` on successful negotiate, `None` otherwise.
pub(crate) async fn try_phase3_negotiate<R, W>(
    reader: &mut R,
    writer: &mut W,
    handler: &RpcHandlers,
    cipher_hint: &str,
    ctx: &crate::method_gate::CallerContext,
) -> Result<Option<NegotiateResult>, crate::unix_socket_connection::ConnectionError>
where
    R: tokio::io::AsyncBufRead + Unpin,
    W: tokio::io::AsyncWrite + Unpin,
{
    use base64::Engine;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

    if cipher_hint == "null" || cipher_hint.is_empty() {
        return Ok(None);
    }

    let mut line_buf: Vec<u8> = Vec::with_capacity(4096);
    let bytes_read = reader.read_until(b'\n', &mut line_buf).await?;
    if bytes_read == 0 {
        return Ok(None);
    }

    let request = match serde_json::from_slice::<JsonRpcRequest>(&line_buf) {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Phase 3 negotiate: parse error on first line: {e}");
            let err_response = JsonRpcResponse::error(
                serde_json::json!(null),
                error_codes::PARSE_ERROR,
                format!("Parse error: {e}"),
            );
            let mut buf = serde_json::to_vec(&err_response)?;
            buf.push(b'\n');
            writer.write_all(&buf).await?;
            writer.flush().await?;
            return Ok(None);
        }
    };

    if request.method != "btsp.negotiate" {
        tracing::debug!(
            "Phase 3: first post-handshake message is {} (not btsp.negotiate), staying plaintext",
            request.method
        );
        let per_req = ctx.clone().with_token_from_params(&request.params);
        let response = handler.handle_request(request, &per_req).await;
        let mut buf = serde_json::to_vec(&response)?;
        buf.push(b'\n');
        writer.write_all(&buf).await?;
        writer.flush().await?;
        return Ok(None);
    }

    let params = &request.params;

    let client_nonce_b64 = params
        .get("client_nonce")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    let client_nonce = base64::engine::general_purpose::STANDARD
        .decode(client_nonce_b64)
        .unwrap_or_default();

    if client_nonce.is_empty() {
        tracing::warn!("Phase 3: btsp.negotiate missing client_nonce, staying plaintext");
        let err_response = JsonRpcResponse::error(
            request.id,
            error_codes::INVALID_PARAMS,
            "btsp.negotiate requires client_nonce",
        );
        let mut buf = serde_json::to_vec(&err_response)?;
        buf.push(b'\n');
        writer.write_all(&buf).await?;
        writer.flush().await?;
        return Ok(None);
    }

    let mut server_nonce = vec![0u8; 32];
    rand::Rng::fill(&mut rand::thread_rng(), server_nonce.as_mut_slice());
    let server_nonce_b64 = base64::engine::general_purpose::STANDARD.encode(&server_nonce);

    let negotiate_response = JsonRpcResponse {
        jsonrpc: std::borrow::Cow::Borrowed("2.0"),
        id: request.id,
        result: Some(serde_json::json!({
            "cipher": cipher_hint,
            "server_nonce": server_nonce_b64,
        })),
        error: None,
    };
    let mut buf = serde_json::to_vec(&negotiate_response)?;
    buf.push(b'\n');
    writer.write_all(&buf).await?;
    writer.flush().await?;

    tracing::info!(
        cipher = %cipher_hint,
        "Phase 3: btsp.negotiate succeeded, transitioning to encrypted framing"
    );

    Ok(Some(NegotiateResult {
        client_nonce,
        server_nonce,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_keys_derive_server_perspective() {
        let handshake_key = b"test-handshake-key-32-bytes-long";
        let client_nonce = b"client-nonce-32-bytes-long-xxxxx";
        let server_nonce = b"server-nonce-32-bytes-long-xxxxx";

        let server_keys =
            SessionKeys::derive(handshake_key, client_nonce, server_nonce, true).unwrap();
        let client_keys =
            SessionKeys::derive(handshake_key, client_nonce, server_nonce, false).unwrap();

        assert_eq!(server_keys.encrypt_key, client_keys.decrypt_key);
        assert_eq!(server_keys.decrypt_key, client_keys.encrypt_key);
        assert_ne!(server_keys.encrypt_key, server_keys.decrypt_key);
    }

    #[test]
    fn session_keys_different_nonces_produce_different_keys() {
        let handshake_key = b"test-handshake-key-32-bytes-long";
        let nonce_a = b"nonce-a-32-bytes-long-xxxxxxxxxxx";
        let nonce_b = b"nonce-b-32-bytes-long-xxxxxxxxxxx";

        let keys_1 = SessionKeys::derive(handshake_key, nonce_a, nonce_b, true).unwrap();
        let keys_2 = SessionKeys::derive(handshake_key, nonce_b, nonce_a, true).unwrap();

        assert_ne!(keys_1.encrypt_key, keys_2.encrypt_key);
    }

    #[test]
    fn phase3_encrypt_decrypt_roundtrip() {
        let handshake_key = b"test-handshake-key-32-bytes-long";
        let client_nonce = b"client-nonce-32-bytes-long-xxxxx";
        let server_nonce = b"server-nonce-32-bytes-long-xxxxx";

        let server_keys =
            SessionKeys::derive(handshake_key, client_nonce, server_nonce, true).unwrap();
        let client_keys =
            SessionKeys::derive(handshake_key, client_nonce, server_nonce, false).unwrap();
        let server_session = Phase3Session::new(&server_keys).unwrap();
        let client_session = Phase3Session::new(&client_keys).unwrap();

        let plaintext = b"hello, encrypted world!";
        let encrypted = server_session.encrypt_frame(plaintext).unwrap();

        assert_ne!(encrypted, plaintext);
        assert!(encrypted.len() >= NONCE_SIZE + TAG_SIZE + plaintext.len());

        let decrypted = client_session.decrypt_frame(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn phase3_nonce_uniqueness() {
        let keys =
            SessionKeys::derive(b"key-32-bytes-long-xxxxxxxxxxxxxx", b"cn", b"sn", true).unwrap();
        let session = Phase3Session::new(&keys).unwrap();

        let frame1 = session.encrypt_frame(b"msg1").unwrap();
        let frame2 = session.encrypt_frame(b"msg1").unwrap();

        assert_ne!(frame1[..NONCE_SIZE], frame2[..NONCE_SIZE]);
    }

    #[test]
    fn phase3_tamper_detection() {
        let server_keys =
            SessionKeys::derive(b"key-32-bytes-long-xxxxxxxxxxxxxx", b"cn", b"sn", true).unwrap();
        let client_keys =
            SessionKeys::derive(b"key-32-bytes-long-xxxxxxxxxxxxxx", b"cn", b"sn", false).unwrap();
        let server_session = Phase3Session::new(&server_keys).unwrap();
        let client_session = Phase3Session::new(&client_keys).unwrap();

        let mut frame = server_session.encrypt_frame(b"sensitive data").unwrap();
        frame[NONCE_SIZE + 2] ^= 0xFF;

        assert!(client_session.decrypt_frame(&frame).is_err());
    }

    #[test]
    fn phase3_short_frame_rejected() {
        let keys =
            SessionKeys::derive(b"key-32-bytes-long-xxxxxxxxxxxxxx", b"cn", b"sn", true).unwrap();
        let session = Phase3Session::new(&keys).unwrap();

        assert!(session.decrypt_frame(&[0u8; 10]).is_err());
    }

    #[test]
    fn phase3_wrong_key_rejected() {
        let keys_a =
            SessionKeys::derive(b"key-a-32-bytes-long-xxxxxxxxxxxx", b"cn", b"sn", true).unwrap();
        let keys_b =
            SessionKeys::derive(b"key-b-32-bytes-long-xxxxxxxxxxxx", b"cn", b"sn", false).unwrap();

        let session_a = Phase3Session::new(&keys_a).unwrap();
        let session_b = Phase3Session::new(&keys_b).unwrap();

        let frame = session_a.encrypt_frame(b"secret").unwrap();
        assert!(session_b.decrypt_frame(&frame).is_err());
    }

    #[test]
    fn phase3_cross_direction_roundtrip() {
        let handshake_key = b"test-handshake-key-32-bytes-long";
        let client_nonce = b"client-nonce-32-bytes-long-xxxxx";
        let server_nonce = b"server-nonce-32-bytes-long-xxxxx";

        let server_keys =
            SessionKeys::derive(handshake_key, client_nonce, server_nonce, true).unwrap();
        let client_keys =
            SessionKeys::derive(handshake_key, client_nonce, server_nonce, false).unwrap();

        let server_session = Phase3Session::new(&server_keys).unwrap();
        let client_session = Phase3Session::new(&client_keys).unwrap();

        let server_msg = server_session.encrypt_frame(b"from server").unwrap();
        let client_msg = client_session.encrypt_frame(b"from client").unwrap();

        assert_eq!(
            client_session.decrypt_frame(&server_msg).unwrap(),
            b"from server"
        );
        assert_eq!(
            server_session.decrypt_frame(&client_msg).unwrap(),
            b"from client"
        );
    }

    #[tokio::test]
    async fn phase3_encrypted_frame_io_roundtrip() {
        let server_keys =
            SessionKeys::derive(b"key-32-bytes-long-xxxxxxxxxxxxxx", b"cn", b"sn", true).unwrap();
        let client_keys =
            SessionKeys::derive(b"key-32-bytes-long-xxxxxxxxxxxxxx", b"cn", b"sn", false).unwrap();
        let server_session = Phase3Session::new(&server_keys).unwrap();
        let client_session = Phase3Session::new(&client_keys).unwrap();

        let (mut client_reader, mut server_writer) = tokio::io::duplex(8192);

        let plaintext = b"test JSON-RPC message";
        write_encrypted_frame(&mut server_writer, &server_session, plaintext)
            .await
            .unwrap();

        let decrypted = read_encrypted_frame(&mut client_reader, &client_session)
            .await
            .unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[tokio::test]
    async fn phase3_multi_message_stream() {
        let server_keys =
            SessionKeys::derive(b"key-32-bytes-long-xxxxxxxxxxxxxx", b"cn", b"sn", true).unwrap();
        let client_keys =
            SessionKeys::derive(b"key-32-bytes-long-xxxxxxxxxxxxxx", b"cn", b"sn", false).unwrap();
        let server_session = Phase3Session::new(&server_keys).unwrap();
        let client_session = Phase3Session::new(&client_keys).unwrap();

        let (mut reader, mut writer) = tokio::io::duplex(65536);

        let messages: Vec<&[u8]> = vec![b"msg1", b"msg2", b"msg3"];
        for msg in &messages {
            write_encrypted_frame(&mut writer, &server_session, msg)
                .await
                .unwrap();
        }

        for expected in &messages {
            let decrypted = read_encrypted_frame(&mut reader, &client_session)
                .await
                .unwrap();
            assert_eq!(decrypted, *expected);
        }
    }
}
