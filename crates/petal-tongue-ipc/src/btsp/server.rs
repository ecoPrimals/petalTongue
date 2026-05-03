// SPDX-License-Identifier: AGPL-3.0-or-later
//! Server-side BTSP handshake (delegates crypto to the security provider).

use super::client::provider_call;
use super::error::BtspHandshakeError;
use super::framing::{read_frame, write_frame};
use super::types::{BtspHandshakeConfig, HandshakeResult};

/// Extract a string field from a JSON value, returning empty string if absent.
fn json_str(val: &serde_json::Value, key: &str) -> String {
    val.get(key)
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned()
}

/// Read `ClientHello`, create a provider session, and send `ServerHello`.
///
/// Returns `(session_token, server_ephemeral_pub, client_ephemeral_pub, challenge)`.
async fn exchange_hello<R, W>(
    reader: &mut R,
    writer: &mut W,
    config: &BtspHandshakeConfig,
) -> Result<(String, String, String, String), BtspHandshakeError>
where
    R: tokio::io::AsyncReadExt + Unpin,
    W: tokio::io::AsyncWriteExt + Unpin,
{
    let client_hello_bytes = read_frame(reader)
        .await
        .map_err(|e| BtspHandshakeError::Io {
            context: "read ClientHello",
            source: e,
        })?;
    let client_hello: serde_json::Value =
        serde_json::from_slice(&client_hello_bytes).map_err(|e| BtspHandshakeError::Json {
            context: "parse ClientHello",
            source: e,
        })?;

    let client_ephemeral_pub = json_str(&client_hello, "client_ephemeral_pub");

    let family_seed = config
        .load_family_seed()
        .unwrap_or_else(|| config.family_id.clone());

    let create_result = provider_call(
        &config.provider_socket,
        "btsp.session.create",
        serde_json::json!({
            "family_seed": family_seed,
            "client_ephemeral_pub": client_ephemeral_pub,
        }),
    )
    .await?;

    let session_token = create_result
        .get("session_token")
        .or_else(|| create_result.get("session_id"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    let server_ephemeral_pub = json_str(&create_result, "server_ephemeral_pub");
    let challenge = json_str(&create_result, "challenge");

    let hello = serde_json::json!({
        "version": 1,
        "server_ephemeral_pub": server_ephemeral_pub,
        "challenge": challenge,
    });
    let hello_bytes = serde_json::to_vec(&hello).map_err(|e| BtspHandshakeError::Json {
        context: "serialize ServerHello",
        source: e,
    })?;
    write_frame(writer, &hello_bytes)
        .await
        .map_err(|e| BtspHandshakeError::Io {
            context: "write ServerHello",
            source: e,
        })?;

    Ok((
        session_token,
        server_ephemeral_pub,
        client_ephemeral_pub,
        challenge,
    ))
}

/// Verify result: preferred cipher + optional session key for Phase 3.
struct VerifyResult {
    preferred_cipher: String,
    session_key: Option<Vec<u8>>,
}

/// Read `ChallengeResponse`, verify via the security provider, and reject on failure.
async fn verify_challenge<R, W>(
    reader: &mut R,
    writer: &mut W,
    config: &BtspHandshakeConfig,
    session_token: &str,
    _server_ephemeral_pub: &str,
    client_ephemeral_pub: &str,
    _challenge: &str,
) -> Result<VerifyResult, BtspHandshakeError>
where
    R: tokio::io::AsyncReadExt + Unpin,
    W: tokio::io::AsyncWriteExt + Unpin,
{
    let cr_bytes = read_frame(reader)
        .await
        .map_err(|e| BtspHandshakeError::Io {
            context: "read ChallengeResponse",
            source: e,
        })?;
    let cr: serde_json::Value =
        serde_json::from_slice(&cr_bytes).map_err(|e| BtspHandshakeError::Json {
            context: "parse ChallengeResponse",
            source: e,
        })?;

    let response = json_str(&cr, "response");
    let preferred_cipher = cr
        .get("preferred_cipher")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("null")
        .to_owned();

    let verify = provider_call(
        &config.provider_socket,
        "btsp.session.verify",
        serde_json::json!({
            "session_token": session_token,
            "response": response,
            "client_ephemeral_pub": client_ephemeral_pub,
            "preferred_cipher": preferred_cipher,
        }),
    )
    .await?;

    if !verify
        .get("verified")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        let reason = json_str(&verify, "reason");
        let err = serde_json::json!({"error": "handshake_failed", "reason": reason});
        let _ = write_frame(writer, &serde_json::to_vec(&err).unwrap_or_default()).await;
        return Err(BtspHandshakeError::VerifyFailed { reason });
    }

    use base64::Engine;
    let session_key = verify
        .get("session_key")
        .and_then(serde_json::Value::as_str)
        .and_then(|s| base64::engine::general_purpose::STANDARD.decode(s).ok());

    Ok(VerifyResult {
        preferred_cipher,
        session_key,
    })
}

/// Perform the server-side BTSP handshake on a connection, delegating
/// crypto to the security provider via `btsp.session.create`, `btsp.session.verify`,
/// and `btsp.negotiate`.
///
/// Returns a [`HandshakeResult`] containing the negotiated cipher and
/// session key material for Phase 3 transport switch.
pub async fn perform_server_handshake<S>(
    stream: &mut S,
    config: &BtspHandshakeConfig,
) -> Result<HandshakeResult, BtspHandshakeError>
where
    S: tokio::io::AsyncReadExt + tokio::io::AsyncWriteExt + Unpin,
{
    let (reader, writer) = tokio::io::split(stream);
    tokio::pin!(reader);
    tokio::pin!(writer);
    perform_server_handshake_split(&mut reader, &mut writer, config).await
}

/// Split-stream variant of [`perform_server_handshake`] for use with
/// pre-split reader/writer pairs (e.g. after UDS first-byte peek).
pub async fn perform_server_handshake_split<R, W>(
    reader: &mut R,
    writer: &mut W,
    config: &BtspHandshakeConfig,
) -> Result<HandshakeResult, BtspHandshakeError>
where
    R: tokio::io::AsyncReadExt + Unpin,
    W: tokio::io::AsyncWriteExt + Unpin,
{
    let (session_token, server_pub, client_pub, challenge) =
        exchange_hello(reader, writer, config).await?;

    let verify = verify_challenge(
        reader,
        writer,
        config,
        &session_token,
        &server_pub,
        &client_pub,
        &challenge,
    )
    .await?;

    let negotiate_result = provider_call(
        &config.provider_socket,
        "btsp.negotiate",
        serde_json::json!({
            "session_id": session_token,
            "preferred_cipher": verify.preferred_cipher,
            "bond_type": "Covalent",
        }),
    )
    .await;

    if let Err(ref e) = negotiate_result {
        tracing::debug!(error = %e, "BTSP negotiate best-effort failed (non-fatal)");
    }

    let cipher = negotiate_result
        .ok()
        .and_then(|v| {
            v.get("cipher")
                .and_then(serde_json::Value::as_str)
                .map(String::from)
        })
        .unwrap_or_else(|| "null".to_owned());

    let complete = serde_json::json!({
        "status": "ok",
        "session_id": session_token,
        "cipher": cipher,
    });
    let complete_bytes = serde_json::to_vec(&complete).map_err(|e| BtspHandshakeError::Json {
        context: "serialize HandshakeComplete",
        source: e,
    })?;
    write_frame(writer, &complete_bytes)
        .await
        .map_err(|e| BtspHandshakeError::Io {
            context: "write HandshakeComplete",
            source: e,
        })?;

    tracing::info!(
        session_token = %session_token,
        cipher = %cipher,
        has_session_key = verify.session_key.is_some(),
        "BTSP handshake complete"
    );
    Ok(HandshakeResult {
        session_token,
        cipher,
        session_key: verify.session_key,
    })
}
