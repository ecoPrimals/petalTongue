// SPDX-License-Identifier: AGPL-3.0-or-later
//! BearDog Transport Security Profile (BTSP).
//!
//! Phase 1: insecure startup guard, family-scoped socket names, and visualization symlinks.
//! Phase 2: handshake policy stubs until BearDog enforces sessions.

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

/// BTSP Phase 2 handshake state (stub for BearDog integration).
#[derive(Debug, Clone)]
pub enum HandshakePolicy {
    /// Development: no handshake required.
    Open,
    /// Production: verification pending in BearDog.
    PendingBearDog {
        /// Family identifier for logging and future enforcement.
        family_id: String,
    },
}

/// Map posture to handshake policy (Phase 2 stub).
#[must_use]
pub fn handshake_policy(posture: &BtspPosture) -> HandshakePolicy {
    match posture {
        BtspPosture::Development => HandshakePolicy::Open,
        BtspPosture::Production { family_id } => HandshakePolicy::PendingBearDog {
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
        HandshakePolicy::PendingBearDog { family_id } => {
            tracing::warn!(
                family_id = %family_id,
                "BTSP Phase 2: production FAMILY_ID set but BearDog handshake not yet enforced. \
                 Connections accepted without verification (Phase 2 pending)."
            );
        }
    }
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
    fn handshake_pending_in_production() {
        let p = BtspPosture::Production {
            family_id: "x".to_string(),
        };
        match handshake_policy(&p) {
            HandshakePolicy::PendingBearDog { family_id } => assert_eq!(family_id, "x"),
            HandshakePolicy::Open => panic!("expected PendingBearDog"),
        }
    }
}
