// SPDX-License-Identifier: AGPL-3.0-or-later
//! BearDog Transport Security Profile (BTSP).
//!
//! Phase 1: insecure startup guard, family-scoped socket names, and visualization symlinks.
//! Phase 2: handshake enforcement via BearDog session delegation.

use petal_tongue_core::constants::APP_DIR_NAME;
use std::env;
use thiserror::Error;

/// Error returned when BTSP startup validation fails.
#[derive(Debug, Clone, Error)]
pub enum BtspGuardError {
    /// Both a production family ID and `BIOMEOS_INSECURE=1` are set.
    #[error(
        "BTSP guard violation: FAMILY_ID={family_id} and BIOMEOS_INSECURE=1 are mutually exclusive. \
         Remove BIOMEOS_INSECURE for production or unset FAMILY_ID for development."
    )]
    ConflictingPosture {
        /// The family identifier that was set.
        family_id: String,
    },
}

/// BTSP posture for socket naming and handshake policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BtspPosture {
    /// Development: no production family in env, empty/`"default"` family, or insecure dev mode.
    Development,
    /// Production: non-default family ID set (`FAMILY_ID` or `PETALTONGUE_FAMILY_ID`).
    Production {
        /// Family identifier used for socket scoping.
        family_id: String,
    },
}

fn raw_family_id_from_env() -> Option<String> {
    env::var("FAMILY_ID")
        .ok()
        .or_else(|| env::var("PETALTONGUE_FAMILY_ID").ok())
}

fn is_production_family_id(fid: Option<&String>) -> bool {
    fid.is_some_and(|s| {
        let t = s.trim();
        !t.is_empty() && !t.eq_ignore_ascii_case("default")
    })
}

/// Resolve posture from the environment when the insecure guard passes (no `FAMILY_ID` + insecure).
#[must_use]
fn posture_after_guard() -> BtspPosture {
    let fid = raw_family_id_from_env();
    if is_production_family_id(fid.as_ref()) {
        BtspPosture::Production {
            family_id: fid.unwrap_or_default(),
        }
    } else {
        BtspPosture::Development
    }
}

/// Validate the BTSP insecure guard at startup.
///
/// Per `BTSP_PROTOCOL_STANDARD.md`: production family + `BIOMEOS_INSECURE=1` must refuse to start.
pub fn validate_insecure_guard() -> Result<BtspPosture, BtspGuardError> {
    let fid = raw_family_id_from_env();
    let is_prod = is_production_family_id(fid.as_ref());

    let biomeos_insecure = env::var("BIOMEOS_INSECURE").ok();
    let is_insecure = biomeos_insecure
        .as_ref()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));

    if is_prod && is_insecure {
        return Err(BtspGuardError::ConflictingPosture {
            family_id: fid.unwrap_or_default(),
        });
    }

    Ok(posture_after_guard())
}

/// Best-effort posture for logging (matches [`validate_insecure_guard`] when env is consistent).
///
/// If the environment is conflicting, returns [`BtspPosture::Development`].
#[must_use]
pub fn current_btsp_posture() -> BtspPosture {
    validate_insecure_guard().unwrap_or(BtspPosture::Development)
}

/// Socket filename under the biomeOS directory.
#[must_use]
pub fn socket_filename(posture: &BtspPosture) -> String {
    match posture {
        BtspPosture::Development => format!("{APP_DIR_NAME}.sock"),
        BtspPosture::Production { family_id } => {
            format!("{APP_DIR_NAME}-{}.sock", sanitize_family_segment(family_id))
        }
    }
}

/// Domain capability symlink next to the socket (`visualization*.sock` → canonical socket).
#[must_use]
pub fn domain_symlink_filename(posture: &BtspPosture) -> String {
    match posture {
        BtspPosture::Development => "visualization.sock".to_string(),
        BtspPosture::Production { family_id } => {
            format!("visualization-{}.sock", sanitize_family_segment(family_id))
        }
    }
}

fn sanitize_family_segment(s: &str) -> String {
    s.trim()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// BTSP Phase 2 handshake policy.
#[derive(Debug, Clone)]
pub enum HandshakePolicy {
    /// Development: no handshake required.
    Open,
    /// Production: BearDog handshake enforced on all connections.
    EnforceBearDog {
        /// Family identifier.
        family_id: String,
    },
}

/// Map posture to handshake policy.
#[must_use]
pub fn handshake_policy(posture: &BtspPosture) -> HandshakePolicy {
    match posture {
        BtspPosture::Development => HandshakePolicy::Open,
        BtspPosture::Production { family_id } => HandshakePolicy::EnforceBearDog {
            family_id: family_id.clone(),
        },
    }
}

/// Log handshake policy on server startup.
pub fn log_handshake_policy(policy: &HandshakePolicy) {
    match policy {
        HandshakePolicy::Open => {
            tracing::debug!("BTSP Phase 2: development mode — no handshake required");
        }
        HandshakePolicy::EnforceBearDog { family_id } => {
            tracing::info!(
                family_id = %family_id,
                "BTSP Phase 2: BearDog handshake enforced on all connections"
            );
        }
    }
}

// ── BTSP Phase 2: Handshake Config ──────────────────────────────────────

/// Configuration for BTSP server-side handshake (Phase 2).
///
/// When present, every accepted connection must complete a BTSP handshake
/// via the BearDog security provider before JSON-RPC is served.
#[derive(Debug, Clone)]
pub struct BtspHandshakeConfig {
    /// Path to BearDog's UDS socket for `btsp.session.*` RPCs.
    pub provider_socket: std::path::PathBuf,
    /// Family identifier.
    pub family_id: String,
}

impl BtspHandshakeConfig {
    /// Resolve handshake config from the environment.
    ///
    /// Returns `Some` when `FAMILY_ID`/`PETALTONGUE_FAMILY_ID` is set to a
    /// production value (non-empty, not `"default"`).
    ///
    /// Provider socket resolution: `BTSP_PROVIDER_SOCKET` > `BEARDOG_SOCKET`
    /// > `$BIOMEOS_SOCKET_DIR/{provider}-{family_id}.sock` > `$XDG_RUNTIME_DIR/biomeos/beardog-{family_id}.sock`.
    #[must_use]
    pub fn from_env() -> Option<Self> {
        let fid = raw_family_id_from_env().filter(|s| is_production_family_id(Some(s)))?;

        let provider_socket = env::var("BTSP_PROVIDER_SOCKET")
            .or_else(|_| env::var("BEARDOG_SOCKET"))
            .ok()
            .map_or_else(
                || {
                    let provider =
                        env::var("BTSP_PROVIDER").unwrap_or_else(|_| "security".to_owned());
                    let socket_dir = env::var("BIOMEOS_SOCKET_DIR").unwrap_or_else(|_| {
                        let xdg = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| {
                            petal_tongue_core::constants::LEGACY_TMP_PREFIX.to_owned()
                        });
                        format!("{xdg}/biomeos")
                    });
                    std::path::PathBuf::from(format!(
                        "{socket_dir}/{provider}-{}.sock",
                        sanitize_family_segment(&fid)
                    ))
                },
                std::path::PathBuf::from,
            );

        Some(Self {
            provider_socket,
            family_id: fid,
        })
    }
}

// ── BTSP Phase 2: Wire Framing ──────────────────────────────────────────

/// Maximum BTSP frame size (16 MiB).
const MAX_FRAME_SIZE: u32 = 0x0100_0000;

/// Read a length-prefixed BTSP frame.
pub(crate) async fn read_frame<R: tokio::io::AsyncReadExt + Unpin>(
    reader: &mut R,
) -> Result<bytes::Bytes, std::io::Error> {
    let len = reader.read_u32().await?;
    if len > MAX_FRAME_SIZE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("BTSP frame too large: {len} > {MAX_FRAME_SIZE}"),
        ));
    }
    let mut buf = bytes::BytesMut::zeroed(len as usize);
    reader.read_exact(&mut buf).await?;
    Ok(buf.freeze())
}

/// Write a length-prefixed BTSP frame.
pub(crate) async fn write_frame<W: tokio::io::AsyncWriteExt + Unpin>(
    writer: &mut W,
    data: &[u8],
) -> Result<(), std::io::Error> {
    let len = u32::try_from(data.len()).map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "frame too large for u32")
    })?;
    writer.write_u32(len).await?;
    writer.write_all(data).await?;
    writer.flush().await
}

// ── BTSP Phase 2: Provider Client ───────────────────────────────────────

/// Call a BearDog `btsp.session.*` RPC via UDS.
async fn provider_call(
    socket: &std::path::Path,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let mut stream = tokio::net::UnixStream::connect(socket)
        .await
        .map_err(|e| format!("BTSP provider {}: {e}", socket.display()))?;

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });
    let mut line = serde_json::to_string(&request).map_err(|e| e.to_string())?;
    line.push('\n');
    stream
        .write_all(line.as_bytes())
        .await
        .map_err(|e| e.to_string())?;
    stream.flush().await.map_err(|e| e.to_string())?;

    let mut reader = BufReader::new(stream);
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .await
        .map_err(|e| e.to_string())?;

    let resp: serde_json::Value =
        serde_json::from_str(&response_line).map_err(|e| e.to_string())?;
    if let Some(err) = resp.get("error") {
        return Err(format!("BTSP provider error: {err}"));
    }
    resp.get("result")
        .cloned()
        .ok_or_else(|| "no result in provider response".to_owned())
}

// ── BTSP Phase 2: Server Handshake ──────────────────────────────────────

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
pub(crate) async fn perform_server_handshake<S>(
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
pub(crate) async fn perform_server_handshake_split<R, W>(
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

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::test_fixtures::env_test_helpers;

    #[test]
    fn validate_development_no_family_id() {
        env_test_helpers::with_env_vars(
            &[
                ("FAMILY_ID", None),
                ("PETALTONGUE_FAMILY_ID", None),
                ("BIOMEOS_INSECURE", None),
            ],
            || {
                assert_eq!(
                    validate_insecure_guard().expect("ok"),
                    BtspPosture::Development
                );
            },
        );
    }

    #[test]
    fn validate_development_family_default() {
        env_test_helpers::with_env_vars(
            &[("FAMILY_ID", Some("default")), ("BIOMEOS_INSECURE", None)],
            || {
                assert_eq!(
                    validate_insecure_guard().expect("ok"),
                    BtspPosture::Development
                );
            },
        );
    }

    #[test]
    fn validate_development_insecure_allowed() {
        env_test_helpers::with_env_vars(
            &[("FAMILY_ID", None), ("BIOMEOS_INSECURE", Some("1"))],
            || {
                assert_eq!(
                    validate_insecure_guard().expect("ok"),
                    BtspPosture::Development
                );
            },
        );
    }

    #[test]
    fn validate_production_family_scoped() {
        env_test_helpers::with_env_vars(
            &[("FAMILY_ID", Some("fam-a")), ("BIOMEOS_INSECURE", None)],
            || match validate_insecure_guard().expect("ok") {
                BtspPosture::Production { family_id } => {
                    assert_eq!(family_id, "fam-a");
                }
                BtspPosture::Development => panic!("expected Production"),
            },
        );
    }

    #[test]
    fn validate_conflicting_posture() {
        env_test_helpers::with_env_vars(
            &[("FAMILY_ID", Some("prod")), ("BIOMEOS_INSECURE", Some("1"))],
            || {
                let e = validate_insecure_guard().expect_err("conflict");
                match e {
                    BtspGuardError::ConflictingPosture { family_id } => {
                        assert_eq!(family_id, "prod");
                    }
                }
            },
        );
    }

    #[test]
    fn posture_development_when_family_id_unset() {
        env_test_helpers::with_env_var_removed("FAMILY_ID", || {
            assert_eq!(current_btsp_posture(), BtspPosture::Development);
        });
    }

    #[test]
    fn posture_production_when_family_id_set() {
        env_test_helpers::with_env_var("FAMILY_ID", "fam-1", || {
            assert_eq!(
                current_btsp_posture(),
                BtspPosture::Production {
                    family_id: "fam-1".to_string()
                }
            );
        });
    }

    #[test]
    fn posture_development_when_family_id_is_default() {
        env_test_helpers::with_env_var("FAMILY_ID", "default", || {
            assert_eq!(current_btsp_posture(), BtspPosture::Development);
        });
    }

    #[test]
    fn socket_filename_development_and_production() {
        assert_eq!(
            socket_filename(&BtspPosture::Development),
            format!("{APP_DIR_NAME}.sock")
        );
        let p = BtspPosture::Production {
            family_id: "acme".to_string(),
        };
        assert_eq!(socket_filename(&p), format!("{APP_DIR_NAME}-acme.sock"));
    }

    #[test]
    fn domain_symlink_matches_posture() {
        assert_eq!(
            domain_symlink_filename(&BtspPosture::Development),
            "visualization.sock"
        );
        assert_eq!(
            domain_symlink_filename(&BtspPosture::Production {
                family_id: "x".to_string()
            }),
            "visualization-x.sock"
        );
    }

    #[test]
    fn handshake_open_in_development() {
        let p = BtspPosture::Development;
        assert!(matches!(handshake_policy(&p), HandshakePolicy::Open));
    }

    #[test]
    fn handshake_enforced_in_production() {
        let p = BtspPosture::Production {
            family_id: "x".to_string(),
        };
        match handshake_policy(&p) {
            HandshakePolicy::EnforceBearDog { family_id } => assert_eq!(family_id, "x"),
            HandshakePolicy::Open => panic!("expected EnforceBearDog"),
        }
    }
}
