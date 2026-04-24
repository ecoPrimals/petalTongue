// SPDX-License-Identifier: AGPL-3.0-or-later
//! BTSP types: posture, guard error, socket naming, handshake policy, provider config.

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

pub(super) fn raw_family_id_from_env() -> Option<String> {
    env::var("FAMILY_ID")
        .ok()
        .or_else(|| env::var("PETALTONGUE_FAMILY_ID").ok())
}

pub(super) fn is_production_family_id(fid: Option<&String>) -> bool {
    fid.is_some_and(|s| {
        let t = s.trim();
        !t.is_empty() && !t.eq_ignore_ascii_case("default")
    })
}

/// Resolve posture from the environment when the insecure guard passes (no `FAMILY_ID` + insecure).
#[must_use]
pub(super) fn posture_after_guard() -> BtspPosture {
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

pub(super) fn sanitize_family_segment(s: &str) -> String {
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
    /// Provider socket resolution:
    /// `BTSP_PROVIDER_SOCKET` > `BEARDOG_SOCKET` > `SECURITY_PROVIDER_SOCKET` >
    /// `CRYPTO_PROVIDER_SOCKET` > `SECURITY_SOCKET` >
    /// `$BIOMEOS_SOCKET_DIR/{provider}-{family_id}.sock`.
    #[must_use]
    pub fn from_env() -> Option<Self> {
        let fid = raw_family_id_from_env().filter(|s| is_production_family_id(Some(s)))?;

        let provider_socket = std::env::var("BTSP_PROVIDER_SOCKET")
            .or_else(|_| std::env::var("BEARDOG_SOCKET"))
            .or_else(|_| std::env::var("SECURITY_PROVIDER_SOCKET"))
            .or_else(|_| std::env::var("CRYPTO_PROVIDER_SOCKET"))
            .or_else(|_| std::env::var("SECURITY_SOCKET"))
            .ok()
            .map_or_else(
                || {
                    let provider =
                        std::env::var("BTSP_PROVIDER").unwrap_or_else(|_| "security".to_owned());
                    let socket_dir = std::env::var("BIOMEOS_SOCKET_DIR").unwrap_or_else(|_| {
                        let xdg = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| {
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

    /// Load the raw family seed string from environment.
    ///
    /// Per SOURDOUGH standard: the value is passed to BearDog as-is
    /// (trimmed). Do NOT hex-decode or base64-encode — BearDog handles
    /// encoding internally.
    ///
    /// Resolution: `BEARDOG_FAMILY_SEED` > `FAMILY_SEED`.
    /// Returns `None` if neither is set.
    #[must_use]
    pub fn load_family_seed(&self) -> Option<String> {
        std::env::var("BEARDOG_FAMILY_SEED")
            .or_else(|_| std::env::var("FAMILY_SEED"))
            .ok()
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
    }
}
