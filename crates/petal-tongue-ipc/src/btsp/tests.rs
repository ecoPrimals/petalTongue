// SPDX-License-Identifier: AGPL-3.0-or-later

use petal_tongue_core::constants::APP_DIR_NAME;
use petal_tongue_core::test_fixtures::env_test_helpers;

use super::{
    BtspGuardError, BtspPosture, HandshakePolicy, current_btsp_posture, domain_symlink_filename,
    handshake_policy, socket_filename, validate_insecure_guard,
};

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

#[test]
fn config_from_env_checks_security_provider_socket() {
    env_test_helpers::with_env_vars(
        &[
            ("FAMILY_ID", Some("test-fam")),
            ("BTSP_PROVIDER_SOCKET", None),
            ("BEARDOG_SOCKET", None),
            ("SECURITY_PROVIDER_SOCKET", Some("/tmp/sec.sock")),
            ("CRYPTO_PROVIDER_SOCKET", None),
            ("SECURITY_SOCKET", None),
        ],
        || {
            let cfg = super::BtspHandshakeConfig::from_env().expect("should resolve config");
            assert_eq!(
                cfg.provider_socket,
                std::path::PathBuf::from("/tmp/sec.sock")
            );
        },
    );
}

#[test]
fn config_from_env_checks_crypto_provider_socket() {
    env_test_helpers::with_env_vars(
        &[
            ("FAMILY_ID", Some("test-fam")),
            ("BTSP_PROVIDER_SOCKET", None),
            ("BEARDOG_SOCKET", None),
            ("SECURITY_PROVIDER_SOCKET", None),
            ("CRYPTO_PROVIDER_SOCKET", Some("/tmp/crypto.sock")),
            ("SECURITY_SOCKET", None),
        ],
        || {
            let cfg = super::BtspHandshakeConfig::from_env().expect("should resolve config");
            assert_eq!(
                cfg.provider_socket,
                std::path::PathBuf::from("/tmp/crypto.sock")
            );
        },
    );
}

#[test]
fn config_from_env_checks_security_socket() {
    env_test_helpers::with_env_vars(
        &[
            ("FAMILY_ID", Some("test-fam")),
            ("BTSP_PROVIDER_SOCKET", None),
            ("BEARDOG_SOCKET", None),
            ("SECURITY_PROVIDER_SOCKET", None),
            ("CRYPTO_PROVIDER_SOCKET", None),
            ("SECURITY_SOCKET", Some("/tmp/security.sock")),
        ],
        || {
            let cfg = super::BtspHandshakeConfig::from_env().expect("should resolve config");
            assert_eq!(
                cfg.provider_socket,
                std::path::PathBuf::from("/tmp/security.sock")
            );
        },
    );
}

#[test]
fn load_family_seed_prefers_beardog_env() {
    env_test_helpers::with_env_vars(
        &[
            ("FAMILY_ID", Some("fam")),
            ("BEARDOG_FAMILY_SEED", Some("YmVhcmRvZy1zZWVk")),
            ("FAMILY_SEED", Some("ZmFtaWx5LXNlZWQ=")),
            ("BTSP_PROVIDER_SOCKET", Some("/tmp/test.sock")),
        ],
        || {
            let cfg = super::BtspHandshakeConfig::from_env().expect("should resolve config");
            assert_eq!(
                cfg.load_family_seed(),
                Some("YmVhcmRvZy1zZWVk".to_owned())
            );
        },
    );
}

#[test]
fn load_family_seed_falls_back_to_family_seed() {
    env_test_helpers::with_env_vars(
        &[
            ("FAMILY_ID", Some("fam")),
            ("BEARDOG_FAMILY_SEED", None),
            ("FAMILY_SEED", Some("ZmFtaWx5LXNlZWQ=")),
            ("BTSP_PROVIDER_SOCKET", Some("/tmp/test.sock")),
        ],
        || {
            let cfg = super::BtspHandshakeConfig::from_env().expect("should resolve config");
            assert_eq!(cfg.load_family_seed(), Some("ZmFtaWx5LXNlZWQ=".to_owned()));
        },
    );
}

#[test]
fn load_family_seed_none_when_unset() {
    env_test_helpers::with_env_vars(
        &[
            ("FAMILY_ID", Some("fam")),
            ("BEARDOG_FAMILY_SEED", None),
            ("FAMILY_SEED", None),
            ("BTSP_PROVIDER_SOCKET", Some("/tmp/test.sock")),
        ],
        || {
            let cfg = super::BtspHandshakeConfig::from_env().expect("should resolve config");
            assert_eq!(cfg.load_family_seed(), None);
        },
    );
}

#[test]
fn load_family_seed_passes_raw_hex_unchanged() {
    let raw_hex = "e06c1785c14c45983eab7f2d9a0b3c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b";
    env_test_helpers::with_env_vars(
        &[
            ("FAMILY_ID", Some("prod")),
            ("BEARDOG_FAMILY_SEED", None),
            ("FAMILY_SEED", Some(raw_hex)),
            ("BTSP_PROVIDER_SOCKET", Some("/tmp/test.sock")),
        ],
        || {
            let cfg = super::BtspHandshakeConfig::from_env().expect("should resolve config");
            assert_eq!(cfg.load_family_seed(), Some(raw_hex.to_owned()));
        },
    );
}

#[test]
fn load_family_seed_trims_whitespace() {
    env_test_helpers::with_env_vars(
        &[
            ("FAMILY_ID", Some("prod")),
            ("BEARDOG_FAMILY_SEED", None),
            ("FAMILY_SEED", Some("  a1b2c3d4e5f6  ")),
            ("BTSP_PROVIDER_SOCKET", Some("/tmp/test.sock")),
        ],
        || {
            let cfg = super::BtspHandshakeConfig::from_env().expect("should resolve config");
            assert_eq!(cfg.load_family_seed(), Some("a1b2c3d4e5f6".to_owned()));
        },
    );
}

#[test]
fn load_family_seed_empty_after_trim_returns_none() {
    env_test_helpers::with_env_vars(
        &[
            ("FAMILY_ID", Some("prod")),
            ("BEARDOG_FAMILY_SEED", None),
            ("FAMILY_SEED", Some("   ")),
            ("BTSP_PROVIDER_SOCKET", Some("/tmp/test.sock")),
        ],
        || {
            let cfg = super::BtspHandshakeConfig::from_env().expect("should resolve config");
            assert_eq!(cfg.load_family_seed(), None);
        },
    );
}
