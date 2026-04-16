// SPDX-License-Identifier: AGPL-3.0-or-later
//! Server-side BTSP handshake (delegates crypto to BearDog).

use super::client::provider_call;
use super::framing::{read_frame, write_frame};
use super::types::BtspHandshakeConfig;

/// Extract a string field from a JSON value, returning empty string if absent.
fn json_str(val: &serde_json::Value, key: &str) -> String {
    val.get(key)
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned()
}

/// Read `ClientHello`, create a BearDog session, and send `ServerHello`.
///
/// Returns `(session_id, server_ephemeral_pub, client_ephemeral_pub, challenge)`.
async fn exchange_hello<R, W>(
    reader: &mut R,
    writer: &mut W,
    config: &BtspHandshakeConfig,
) -> Result<(String, String, String, String), String>
where
    R: tokio::io::AsyncReadExt + Unpin,
    W: tokio::io::AsyncWriteExt + Unpin,
{
    let client_hello_bytes = read_frame(reader)
        .await
        .map_err(|e| format!("read ClientHello: {e}"))?;
    let client_hello: serde_json::Value = serde_json::from_slice(&client_hello_bytes)
        .map_err(|e| format!("parse ClientHello: {e}"))?;

    let client_ephemeral_pub = json_str(&client_hello, "client_ephemeral_pub");
    let challenge = format!("{:032x}", rand_u128());

    let create_result = provider_call(
        &config.provider_socket,
        "btsp.session.create",
        serde_json::json!({
            "family_seed_ref": "env:FAMILY_SEED",
            "client_ephemeral_pub": client_ephemeral_pub,
            "challenge": challenge,
        }),
    )
    .await?;

    let session_id = json_str(&create_result, "session_id");
    let server_ephemeral_pub = json_str(&create_result, "server_ephemeral_pub");

    let hello = serde_json::json!({
        "session_id": session_id,
        "server_ephemeral_pub": server_ephemeral_pub,
        "challenge": challenge,
    });
    let hello_bytes = serde_json::to_vec(&hello).map_err(|e| e.to_string())?;
    write_frame(writer, &hello_bytes)
        .await
        .map_err(|e| format!("write ServerHello: {e}"))?;

    Ok((
        session_id,
        server_ephemeral_pub,
        client_ephemeral_pub,
        challenge,
    ))
}

/// Read `ChallengeResponse`, verify via BearDog, and reject on failure.
async fn verify_challenge<R, W>(
    reader: &mut R,
    writer: &mut W,
    config: &BtspHandshakeConfig,
    session_id: &str,
    server_ephemeral_pub: &str,
    client_ephemeral_pub: &str,
    challenge: &str,
) -> Result<String, String>
where
    R: tokio::io::AsyncReadExt + Unpin,
    W: tokio::io::AsyncWriteExt + Unpin,
{
    let cr_bytes = read_frame(reader)
        .await
        .map_err(|e| format!("read ChallengeResponse: {e}"))?;
    let cr: serde_json::Value =
        serde_json::from_slice(&cr_bytes).map_err(|e| format!("parse ChallengeResponse: {e}"))?;

    let client_response = json_str(&cr, "response");
    let preferred_cipher = cr
        .get("preferred_cipher")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("null")
        .to_owned();

    let verify = provider_call(
        &config.provider_socket,
        "btsp.session.verify",
        serde_json::json!({
            "session_id": session_id,
            "client_response": client_response,
            "client_ephemeral_pub": client_ephemeral_pub,
            "server_ephemeral_pub": server_ephemeral_pub,
            "challenge": challenge,
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
        return Err(format!("BTSP verify failed: {reason}"));
    }

    Ok(preferred_cipher)
}

/// Perform the server-side BTSP handshake on a connection, delegating
/// crypto to BearDog via `btsp.session.create`, `btsp.session.verify`,
/// and `btsp.negotiate`.
///
/// After a successful handshake, the same stream is used for plain
/// newline-delimited JSON-RPC (null cipher — Phase 3 will add encryption).
pub async fn perform_server_handshake<S>(
    stream: &mut S,
    config: &BtspHandshakeConfig,
) -> Result<String, String>
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
) -> Result<String, String>
where
    R: tokio::io::AsyncReadExt + Unpin,
    W: tokio::io::AsyncWriteExt + Unpin,
{
    let (session_id, server_pub, client_pub, challenge) =
        exchange_hello(reader, writer, config).await?;

    let preferred_cipher = verify_challenge(
        reader,
        writer,
        config,
        &session_id,
        &server_pub,
        &client_pub,
        &challenge,
    )
    .await?;

    let _negotiate = provider_call(
        &config.provider_socket,
        "btsp.negotiate",
        serde_json::json!({
            "session_id": session_id,
            "preferred_cipher": preferred_cipher,
            "bond_type": "Covalent",
        }),
    )
    .await;

    let complete = serde_json::json!({
        "status": "complete",
        "session_id": session_id,
        "cipher": "null",
    });
    let complete_bytes = serde_json::to_vec(&complete).map_err(|e| e.to_string())?;
    write_frame(writer, &complete_bytes)
        .await
        .map_err(|e| format!("write Complete: {e}"))?;

    tracing::info!(session_id = %session_id, "BTSP handshake complete (null cipher)");
    Ok(session_id)
}

/// Simple PRNG for challenge nonces (not cryptographically strong —
/// BearDog provides the real crypto via session.create).
fn rand_u128() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    t ^ (std::process::id() as u128) ^ 0x5555_5555_5555_5555_5555_5555_5555_5555
}
