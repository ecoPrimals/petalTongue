// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON-line (newline-delimited) BTSP handshake relay.
//!
//! primalSpring sends ClientHello as `{"protocol":"btsp",...}\n` — a JSON line,
//! not a length-prefixed frame. When the accept loop detects this framing, the
//! entire 4-step handshake must use JSON-line framing (not length-prefixed).
//!
//! Wire protocol (from PRIMALSPRING_PHASE45C_BTSP_DEFAULT_UPSTREAM_HANDOFF):
//!   1. Read ClientHello line → extract `client_ephemeral_pub`
//!   2. Call BearDog `btsp.session.create` → get session_token, server_ephemeral_pub, challenge
//!   3. Write ServerHello line
//!   4. Read ChallengeResponse line → extract response, preferred_cipher
//!   5. Call BearDog `btsp.session.verify`
//!   6. Call BearDog `btsp.negotiate` (best-effort, non-fatal — Phase 3 cipher selection)
//!   7. Write HandshakeComplete line (includes negotiated cipher or `"null"`)

use super::client::provider_call;
use super::error::BtspHandshakeError;
use super::types::BtspHandshakeConfig;

/// Read one newline-delimited JSON line.
async fn read_json_line<R>(reader: &mut R) -> Result<serde_json::Value, BtspHandshakeError>
where
    R: tokio::io::AsyncBufReadExt + Unpin,
{
    let mut buf: Vec<u8> = Vec::with_capacity(4096);
    reader
        .read_until(b'\n', &mut buf)
        .await
        .map_err(|e| BtspHandshakeError::Io {
            context: "read JSON line",
            source: e,
        })?;
    if buf.is_empty() {
        return Err(BtspHandshakeError::UnexpectedEof {
            expected: "JSON line",
        });
    }
    let trimmed = if buf.last() == Some(&b'\n') {
        &buf[..buf.len() - 1]
    } else {
        &buf
    };
    serde_json::from_slice(trimmed).map_err(|e| BtspHandshakeError::Json {
        context: "parse JSON line",
        source: e,
    })
}

/// Write a JSON value as a newline-terminated line.
async fn write_json_line<W>(
    writer: &mut W,
    val: &serde_json::Value,
) -> Result<(), BtspHandshakeError>
where
    W: tokio::io::AsyncWriteExt + Unpin,
{
    let mut buf = serde_json::to_vec(val).map_err(|e| BtspHandshakeError::Json {
        context: "serialize JSON line",
        source: e,
    })?;
    buf.push(b'\n');
    writer
        .write_all(&buf)
        .await
        .map_err(|e| BtspHandshakeError::Io {
            context: "write JSON line",
            source: e,
        })?;
    writer.flush().await.map_err(|e| BtspHandshakeError::Io {
        context: "flush JSON line",
        source: e,
    })
}

/// Extract a string field, falling back to an alternate key.
fn json_str_or(val: &serde_json::Value, key: &str, alt: &str) -> String {
    val.get(key)
        .or_else(|| val.get(alt))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned()
}

/// Exchange hello with BearDog: read ClientHello, create session, write ServerHello.
async fn exchange_hello_json_line<R, W>(
    reader: &mut R,
    writer: &mut W,
    config: &BtspHandshakeConfig,
) -> Result<(String, String), BtspHandshakeError>
where
    R: tokio::io::AsyncBufReadExt + Unpin,
    W: tokio::io::AsyncWriteExt + Unpin,
{
    let client_hello = read_json_line(reader).await?;
    let client_ephemeral_pub = client_hello
        .get("client_ephemeral_pub")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();

    tracing::debug!(
        client_ephemeral_pub_len = client_ephemeral_pub.len(),
        "BTSP JSON-line: parsed ClientHello"
    );

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

    let session_token = json_str_or(&create_result, "session_token", "session_id");
    let server_ephemeral_pub = create_result
        .get("server_ephemeral_pub")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    let challenge = create_result
        .get("challenge")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();

    tracing::debug!(session_token = %session_token, "BTSP JSON-line: session created via BearDog");

    let server_hello = serde_json::json!({
        "version": 1,
        "server_ephemeral_pub": server_ephemeral_pub,
        "challenge": challenge,
    });
    write_json_line(writer, &server_hello).await?;

    Ok((session_token, client_ephemeral_pub))
}

/// Perform the full JSON-line BTSP handshake relay on a split stream.
///
/// The ClientHello line has already been peeked (still in the `BufReader`
/// buffer) — this function consumes it via `read_line`, then continues the
/// relay using JSON-line framing throughout.
pub async fn relay_json_line_handshake_split<R, W>(
    reader: &mut R,
    writer: &mut W,
    config: &BtspHandshakeConfig,
) -> Result<String, BtspHandshakeError>
where
    R: tokio::io::AsyncBufReadExt + Unpin,
    W: tokio::io::AsyncWriteExt + Unpin,
{
    let (session_token, client_ephemeral_pub) =
        exchange_hello_json_line(reader, writer, config).await?;

    let challenge_response = read_json_line(reader).await?;
    let response = challenge_response
        .get("response")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("")
        .to_owned();
    let preferred_cipher = challenge_response
        .get("preferred_cipher")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("null")
        .to_owned();

    let verify_result = provider_call(
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

    if !verify_result
        .get("verified")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        let reason = verify_result
            .get("reason")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown")
            .to_owned();
        let err_msg = serde_json::json!({"error": "handshake_failed", "reason": reason});
        let _ = write_json_line(writer, &err_msg).await;
        return Err(BtspHandshakeError::VerifyFailed { reason });
    }

    let negotiate_result = provider_call(
        &config.provider_socket,
        "btsp.negotiate",
        serde_json::json!({
            "session_id": session_token,
            "preferred_cipher": preferred_cipher,
            "bond_type": "Covalent",
        }),
    )
    .await;

    if let Err(ref e) = negotiate_result {
        tracing::debug!(error = %e, "BTSP JSON-line: negotiate best-effort failed (non-fatal)");
    }

    let session_id = json_str_or(&verify_result, "session_id", "session_token");
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
        "session_id": if session_id.is_empty() { &session_token } else { &session_id },
        "cipher": cipher,
    });
    write_json_line(writer, &complete).await?;

    tracing::info!(
        session_token = %session_token,
        "BTSP JSON-line handshake complete (cipher={cipher})"
    );
    Ok(session_token)
}

/// Perform JSON-line BTSP handshake on a full stream (TCP path).
///
/// Wraps the stream in a `BufReader` for line-based reading, then delegates
/// to [`relay_json_line_handshake_split`].
pub async fn relay_json_line_handshake<S>(
    stream: &mut S,
    config: &BtspHandshakeConfig,
) -> Result<String, BtspHandshakeError>
where
    S: tokio::io::AsyncReadExt + tokio::io::AsyncWriteExt + Unpin,
{
    let (reader, writer) = tokio::io::split(stream);
    let mut buf_reader = tokio::io::BufReader::new(reader);
    tokio::pin!(writer);
    relay_json_line_handshake_split(&mut buf_reader, &mut writer, config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_str_or_prefers_primary_key() {
        let val = serde_json::json!({"session_token": "tok1", "session_id": "id1"});
        assert_eq!(json_str_or(&val, "session_token", "session_id"), "tok1");
    }

    #[test]
    fn json_str_or_falls_back_to_alt() {
        let val = serde_json::json!({"session_id": "id1"});
        assert_eq!(json_str_or(&val, "session_token", "session_id"), "id1");
    }

    #[test]
    fn json_str_or_returns_empty_when_both_absent() {
        let val = serde_json::json!({"other": "x"});
        assert_eq!(json_str_or(&val, "session_token", "session_id"), "");
    }
}
