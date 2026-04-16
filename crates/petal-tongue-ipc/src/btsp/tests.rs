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
