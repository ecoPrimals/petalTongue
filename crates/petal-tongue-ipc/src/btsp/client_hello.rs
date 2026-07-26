// SPDX-License-Identifier: AGPL-3.0-or-later
//! BTSP client-side handshake — ClientHello flow for connecting to BTSP-enforcing primals.
//!
//! When petalTongue connects to another primal (e.g., bearDog for content signing),
//! it must complete the BTSP 4-step handshake if that primal enforces BTSP strict mode.
//!
//! # Protocol Steps
//!
//! 1. Client → Server: `ClientHello { protocol: "btsp", version: 1, client_ephemeral_pub }`
//! 2. Server → Client: `ServerHello { version: 1, server_ephemeral_pub, challenge }`
//! 3. Client → Server: `ChallengeResponse { response: HMAC-SHA256(family_seed, challenge), preferred_cipher }`
//! 4. Server → Client: `HandshakeComplete { status: "ok", session_id, cipher }`

use super::error::BtspHandshakeError;
use super::framing::{read_frame, write_frame};
use super::types::HandshakeResult;

/// Configuration for a BTSP client handshake.
#[derive(Debug, Clone)]
pub struct BtspClientConfig {
    /// Family seed (hex or raw bytes) for HMAC challenge-response.
    pub family_seed: Vec<u8>,
    /// Preferred cipher for Phase 3 negotiation.
    pub preferred_cipher: String,
}

impl BtspClientConfig {
    /// Resolve client config from environment variables.
    ///
    /// Reads `BTSP_FAMILY_SEED` or `FAMILY_SEED` for the shared secret.
    /// Returns `None` if no family seed is available (development mode).
    #[must_use]
    pub fn from_env() -> Option<Self> {
        let seed_hex = std::env::var(petal_tongue_core::constants::BTSP_FAMILY_SEED)
            .or_else(|_| std::env::var(petal_tongue_core::constants::FAMILY_SEED))
            .ok()
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())?;

        Some(Self {
            family_seed: seed_hex.into_bytes(),
            preferred_cipher: "chacha20-poly1305".to_owned(),
        })
    }
}

/// Perform the BTSP client-side handshake on an established connection.
///
/// The connection must already be open to the target primal. This function
/// performs the 4-step BTSP handshake and returns the session details.
///
/// # Errors
///
/// Returns `BtspHandshakeError` if any handshake step fails (I/O, protocol,
/// or verification rejection).
pub async fn perform_client_handshake<S>(
    stream: &mut S,
    config: &BtspClientConfig,
) -> Result<HandshakeResult, BtspHandshakeError>
where
    S: tokio::io::AsyncReadExt + tokio::io::AsyncWriteExt + Unpin,
{
    let (reader, writer) = tokio::io::split(stream);
    tokio::pin!(reader);
    tokio::pin!(writer);
    perform_client_handshake_split(&mut reader, &mut writer, config).await
}

/// Split-stream variant of [`perform_client_handshake`].
pub async fn perform_client_handshake_split<R, W>(
    reader: &mut R,
    writer: &mut W,
    config: &BtspClientConfig,
) -> Result<HandshakeResult, BtspHandshakeError>
where
    R: tokio::io::AsyncReadExt + Unpin,
    W: tokio::io::AsyncWriteExt + Unpin,
{
    let client_ephemeral_pub = generate_ephemeral_pub();

    send_client_hello(writer, &client_ephemeral_pub).await?;
    let (challenge, session_id) = receive_server_hello(reader).await?;
    send_challenge_response(writer, config, &challenge).await?;
    receive_handshake_complete(reader, &session_id).await
}

/// Step 1: Send ClientHello frame.
async fn send_client_hello<W: tokio::io::AsyncWriteExt + Unpin>(
    writer: &mut W,
    client_ephemeral_pub: &str,
) -> Result<(), BtspHandshakeError> {
    let client_hello = serde_json::json!({
        "protocol": "btsp",
        "version": 1,
        "client_ephemeral_pub": client_ephemeral_pub,
    });
    let hello_bytes = serde_json::to_vec(&client_hello).map_err(|e| BtspHandshakeError::Json {
        context: "serialize ClientHello",
        source: e,
    })?;
    write_frame(writer, &hello_bytes)
        .await
        .map_err(|e| BtspHandshakeError::Io {
            context: "write ClientHello",
            source: e,
        })
}

/// Step 2: Read ServerHello, extract challenge and session ID.
async fn receive_server_hello<R: tokio::io::AsyncReadExt + Unpin>(
    reader: &mut R,
) -> Result<(String, String), BtspHandshakeError> {
    let server_hello_bytes = read_frame(reader)
        .await
        .map_err(|e| BtspHandshakeError::Io {
            context: "read ServerHello",
            source: e,
        })?;
    let server_hello: serde_json::Value =
        serde_json::from_slice(&server_hello_bytes).map_err(|e| BtspHandshakeError::Json {
            context: "parse ServerHello",
            source: e,
        })?;

    if server_hello.get("error").is_some() {
        let reason = server_hello
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_owned();
        return Err(BtspHandshakeError::VerifyFailed { reason });
    }

    let challenge = server_hello
        .get("challenge")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_owned();

    let session_id = server_hello
        .get("session_id")
        .or_else(|| server_hello.get("session_token"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_owned();

    Ok((challenge, session_id))
}

/// Step 3: Compute HMAC and send ChallengeResponse.
async fn send_challenge_response<W: tokio::io::AsyncWriteExt + Unpin>(
    writer: &mut W,
    config: &BtspClientConfig,
    challenge: &str,
) -> Result<(), BtspHandshakeError> {
    let response = compute_challenge_response(&config.family_seed, challenge);

    let challenge_response = serde_json::json!({
        "response": response,
        "preferred_cipher": config.preferred_cipher,
    });
    let cr_bytes =
        serde_json::to_vec(&challenge_response).map_err(|e| BtspHandshakeError::Json {
            context: "serialize ChallengeResponse",
            source: e,
        })?;
    write_frame(writer, &cr_bytes)
        .await
        .map_err(|e| BtspHandshakeError::Io {
            context: "write ChallengeResponse",
            source: e,
        })
}

/// Step 4: Read HandshakeComplete or rejection.
async fn receive_handshake_complete<R: tokio::io::AsyncReadExt + Unpin>(
    reader: &mut R,
    session_id: &str,
) -> Result<HandshakeResult, BtspHandshakeError> {
    let complete_bytes = read_frame(reader)
        .await
        .map_err(|e| BtspHandshakeError::Io {
            context: "read HandshakeComplete",
            source: e,
        })?;
    let complete: serde_json::Value =
        serde_json::from_slice(&complete_bytes).map_err(|e| BtspHandshakeError::Json {
            context: "parse HandshakeComplete",
            source: e,
        })?;

    if complete.get("error").is_some() {
        let reason = complete
            .get("reason")
            .or_else(|| complete.get("error"))
            .and_then(|v| v.as_str())
            .unwrap_or("handshake rejected")
            .to_owned();
        return Err(BtspHandshakeError::VerifyFailed { reason });
    }

    let cipher = complete
        .get("cipher")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("null")
        .to_owned();
    let final_session_id = complete
        .get("session_id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(session_id)
        .to_owned();

    tracing::info!(
        session_id = %final_session_id,
        cipher = %cipher,
        "BTSP client handshake complete"
    );

    Ok(HandshakeResult {
        session_token: final_session_id,
        cipher,
        session_key: None,
    })
}

/// Generate a random ephemeral public key (hex-encoded 32 bytes).
fn generate_ephemeral_pub() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let seed = blake3::hash(&timestamp.to_le_bytes());
    bytes_to_hex(seed.as_bytes())
}

/// Compute HMAC-SHA256(family_seed, challenge) for the challenge-response step.
fn compute_challenge_response(family_seed: &[u8], challenge: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;

    let key = if family_seed.is_empty() {
        &[0u8; 32] as &[u8]
    } else {
        family_seed
    };
    // HMAC-SHA256 accepts any key length — new_from_slice is infallible here.
    #[allow(clippy::expect_used)]
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC-SHA256 accepts any key length");
    mac.update(challenge.as_bytes());
    let result = mac.finalize();
    bytes_to_hex(&result.into_bytes())
}

/// Encode bytes as lowercase hex string.
fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(s, "{b:02x}");
    }
    s
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test code")]

    use super::*;
    use tokio::io::duplex;

    #[test]
    fn compute_challenge_response_deterministic() {
        let seed = b"test-family-seed";
        let challenge = "random-challenge-string";
        let r1 = compute_challenge_response(seed, challenge);
        let r2 = compute_challenge_response(seed, challenge);
        assert_eq!(r1, r2);
        assert_eq!(r1.len(), 64);
    }

    #[test]
    fn compute_challenge_response_different_seeds() {
        let challenge = "same-challenge";
        let r1 = compute_challenge_response(b"seed-a", challenge);
        let r2 = compute_challenge_response(b"seed-b", challenge);
        assert_ne!(r1, r2);
    }

    #[test]
    fn generate_ephemeral_pub_is_hex() {
        let pub_key = generate_ephemeral_pub();
        assert_eq!(pub_key.len(), 64);
        assert!(pub_key.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[tokio::test]
    async fn client_handshake_against_mock_server() {
        let (mut client_stream, mut server_stream) = duplex(8192);

        let config = BtspClientConfig {
            family_seed: b"test-seed".to_vec(),
            preferred_cipher: "chacha20-poly1305".to_owned(),
        };

        let server_task = tokio::spawn(async move {
            use super::super::framing::{read_frame, write_frame};

            let hello_bytes = read_frame(&mut server_stream).await.unwrap();
            let hello: serde_json::Value = serde_json::from_slice(&hello_bytes).unwrap();
            assert_eq!(hello["protocol"], "btsp");
            assert_eq!(hello["version"], 1);
            assert!(hello["client_ephemeral_pub"].is_string());

            let server_hello = serde_json::json!({
                "version": 1,
                "server_ephemeral_pub": "deadbeef".repeat(4),
                "challenge": "test-challenge-123",
                "session_id": "session-abc",
            });
            write_frame(&mut server_stream, &serde_json::to_vec(&server_hello).unwrap())
                .await
                .unwrap();

            let cr_bytes = read_frame(&mut server_stream).await.unwrap();
            let cr: serde_json::Value = serde_json::from_slice(&cr_bytes).unwrap();
            assert!(cr["response"].is_string());
            assert_eq!(cr["preferred_cipher"], "chacha20-poly1305");

            let expected_response =
                compute_challenge_response(b"test-seed", "test-challenge-123");
            assert_eq!(cr["response"].as_str().unwrap(), expected_response);

            let complete = serde_json::json!({
                "status": "ok",
                "session_id": "session-abc",
                "cipher": "chacha20-poly1305",
            });
            write_frame(&mut server_stream, &serde_json::to_vec(&complete).unwrap())
                .await
                .unwrap();
        });

        let result = perform_client_handshake(&mut client_stream, &config)
            .await
            .unwrap();

        assert_eq!(result.session_token, "session-abc");
        assert_eq!(result.cipher, "chacha20-poly1305");

        server_task.await.unwrap();
    }

    #[tokio::test]
    async fn client_handshake_server_rejection() {
        let (mut client_stream, mut server_stream) = duplex(8192);

        let config = BtspClientConfig {
            family_seed: b"bad-seed".to_vec(),
            preferred_cipher: "null".to_owned(),
        };

        let server_task = tokio::spawn(async move {
            use super::super::framing::{read_frame, write_frame};

            let _hello = read_frame(&mut server_stream).await.unwrap();

            let server_hello = serde_json::json!({
                "version": 1,
                "server_ephemeral_pub": "abcd1234".repeat(4),
                "challenge": "challenge-xyz",
                "session_id": "sess-1",
            });
            write_frame(&mut server_stream, &serde_json::to_vec(&server_hello).unwrap())
                .await
                .unwrap();

            let _cr = read_frame(&mut server_stream).await.unwrap();

            let reject = serde_json::json!({
                "error": "handshake_failed",
                "reason": "invalid HMAC",
            });
            write_frame(&mut server_stream, &serde_json::to_vec(&reject).unwrap())
                .await
                .unwrap();
        });

        let result = perform_client_handshake(&mut client_stream, &config).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("invalid HMAC"));

        server_task.await.unwrap();
    }
}
