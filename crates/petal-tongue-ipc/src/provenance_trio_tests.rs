// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn provenance_session_serialization() {
    let session = ProvenanceSession {
        session_id: "test-123".to_string(),
        vertex_id: Some("v-abc".to_string()),
        braid_id: Some("b-def".to_string()),
        spine_entry_id: None,
    };
    let json = serde_json::to_string(&session).expect("serialize");
    let restored: ProvenanceSession = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.session_id, "test-123");
    assert_eq!(restored.vertex_id.as_deref(), Some("v-abc"));
    assert!(restored.spine_entry_id.is_none());
}

#[test]
fn trio_availability_full() {
    let a = TrioAvailability {
        ephemeral_dag: true,
        attribution: true,
        permanent_ledger: true,
    };
    assert!(a.is_full());
    assert!(a.is_partial());
}

#[test]
fn trio_availability_partial() {
    let a = TrioAvailability {
        ephemeral_dag: true,
        attribution: false,
        permanent_ledger: false,
    };
    assert!(!a.is_full());
    assert!(a.is_partial());
}

#[test]
fn trio_availability_none() {
    let a = TrioAvailability {
        ephemeral_dag: false,
        attribution: false,
        permanent_ledger: false,
    };
    assert!(!a.is_full());
    assert!(!a.is_partial());
}

#[test]
fn discover_returns_none_without_sockets() {
    use petal_tongue_core::test_fixtures::env_test_helpers::with_env_vars;

    let tmp = tempfile::tempdir().expect("tempdir");
    let tmp_path = tmp.path().to_string_lossy().into_owned();

    with_env_vars(
        &[
            ("XDG_RUNTIME_DIR", Some(&tmp_path)),
            ("DAG_SESSION_SOCKET", None),
            ("DAG_VERTEX_CREATE_SOCKET", None),
            ("BRAID_CREATE_SOCKET", None),
            ("SPINE_CREATE_SOCKET", None),
        ],
        || {
            let client = ProvenanceTrioClient::discover();
            let availability = client.availability();
            assert!(!availability.is_full());
            assert!(!availability.is_partial());
        },
    );
}

#[test]
fn with_sockets_sets_availability() {
    let client = ProvenanceTrioClient::with_sockets(
        Some("/tmp/rhizo.sock".to_string()),
        None,
        Some("/tmp/loam.sock".to_string()),
    );
    let a = client.availability();
    assert!(a.ephemeral_dag);
    assert!(!a.attribution);
    assert!(a.permanent_ledger);
}

#[tokio::test]
async fn begin_session_without_trio_returns_empty() {
    let client = ProvenanceTrioClient::with_sockets(None, None, None);
    let session = client.begin_session("s1", "Test", None).await;
    assert_eq!(session.session_id, "s1");
    assert!(session.vertex_id.is_none());
    assert!(session.braid_id.is_none());
}

#[tokio::test]
async fn record_contribution_without_attribution_is_noop() {
    let client = ProvenanceTrioClient::with_sockets(None, None, None);
    let session = ProvenanceSession {
        session_id: "s1".to_string(),
        vertex_id: None,
        braid_id: None,
        spine_entry_id: None,
    };
    // Should not panic
    client
        .record_contribution(&session, "neuralspring", "data-1", "timeseries")
        .await;
}

#[tokio::test]
async fn archive_session_without_ledger_is_noop() {
    let client = ProvenanceTrioClient::with_sockets(None, None, None);
    let mut session = ProvenanceSession {
        session_id: "s1".to_string(),
        vertex_id: None,
        braid_id: None,
        spine_entry_id: None,
    };
    client.archive_session(&mut session, "svg", b"<svg/>").await;
    assert!(session.spine_entry_id.is_none());
}

#[test]
fn capability_matches_dag_domain() {
    assert!(capability_matches_socket_name(
        "dag.session",
        "dag-provider-nat0"
    ));
    assert!(capability_matches_socket_name(
        "dag.vertex.create",
        "dag-family1"
    ));
    assert!(!capability_matches_socket_name("dag.session", "songbird"));
}

#[test]
fn capability_matches_braid_domain() {
    assert!(capability_matches_socket_name(
        "braid.create",
        "braid-provider-nat0"
    ));
    assert!(capability_matches_socket_name(
        "contribution.record",
        "braid-family1"
    ));
    assert!(!capability_matches_socket_name("braid.create", "loamspine"));
}

#[test]
fn capability_matches_spine_domain() {
    assert!(capability_matches_socket_name(
        "spine.create",
        "spine-provider-nat0"
    ));
    assert!(capability_matches_socket_name(
        "entry.append",
        "spine-family1"
    ));
    assert!(!capability_matches_socket_name(
        "spine.create",
        "rhizocrypt"
    ));
}

#[test]
fn blake3_hash_deterministic() {
    let h1 = blake3_hash(b"test data");
    let h2 = blake3_hash(b"test data");
    assert_eq!(h1, h2);
}

#[test]
fn blake3_hash_different_for_different_data() {
    let h1 = blake3_hash(b"data A");
    let h2 = blake3_hash(b"data B");
    assert_ne!(h1, h2);
}

#[test]
fn provenance_session_default_values() {
    let session = ProvenanceSession {
        session_id: String::new(),
        vertex_id: None,
        braid_id: None,
        spine_entry_id: None,
    };
    assert!(session.vertex_id.is_none());
    assert!(session.braid_id.is_none());
    assert!(session.spine_entry_id.is_none());
}

#[test]
fn trio_availability_serialization() {
    let a = TrioAvailability {
        ephemeral_dag: true,
        attribution: false,
        permanent_ledger: true,
    };
    let json = serde_json::to_string(&a).expect("serialize");
    let restored: TrioAvailability = serde_json::from_str(&json).expect("deserialize");
    assert!(restored.ephemeral_dag);
    assert!(!restored.attribution);
}

#[tokio::test]
async fn record_contribution_without_braid_id_is_noop() {
    let client =
        ProvenanceTrioClient::with_sockets(None, Some("/tmp/nonexistent.sock".to_string()), None);
    let session = ProvenanceSession {
        session_id: "s1".to_string(),
        vertex_id: None,
        braid_id: None,
        spine_entry_id: None,
    };
    client
        .record_contribution(&session, "neuralspring", "data-1", "timeseries")
        .await;
}

#[test]
fn capability_matches_unknown_capability() {
    assert!(!capability_matches_socket_name("unknown.cap", "dag-sock"));
    assert!(!capability_matches_socket_name("foo.bar", "braid-sock"));
}

#[test]
fn provenance_session_with_all_ids() {
    let session = ProvenanceSession {
        session_id: "full-session".to_string(),
        vertex_id: Some("v1".to_string()),
        braid_id: Some("b1".to_string()),
        spine_entry_id: Some("e1".to_string()),
    };
    let json = serde_json::to_string(&session).expect("serialize");
    let restored: ProvenanceSession = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.vertex_id.as_deref(), Some("v1"));
    assert_eq!(restored.braid_id.as_deref(), Some("b1"));
    assert_eq!(restored.spine_entry_id.as_deref(), Some("e1"));
}

#[test]
fn trio_availability_attribution_only() {
    let a = TrioAvailability {
        ephemeral_dag: false,
        attribution: true,
        permanent_ledger: false,
    };
    assert!(!a.is_full());
    assert!(a.is_partial());
}

#[test]
fn trio_availability_ledger_only() {
    let a = TrioAvailability {
        ephemeral_dag: false,
        attribution: false,
        permanent_ledger: true,
    };
    assert!(!a.is_full());
    assert!(a.is_partial());
}

#[test]
fn trio_availability_ephemeral_and_ledger() {
    let a = TrioAvailability {
        ephemeral_dag: true,
        attribution: false,
        permanent_ledger: true,
    };
    assert!(!a.is_full());
    assert!(a.is_partial());
}

#[test]
fn trio_availability_debug() {
    let a = TrioAvailability {
        ephemeral_dag: true,
        attribution: true,
        permanent_ledger: true,
    };
    let dbg = format!("{a:?}");
    assert!(dbg.contains("ephemeral_dag"));
    assert!(dbg.contains("attribution"));
}

#[test]
fn provenance_session_debug() {
    let session = ProvenanceSession {
        session_id: "dbg".to_string(),
        vertex_id: None,
        braid_id: None,
        spine_entry_id: None,
    };
    let dbg = format!("{session:?}");
    assert!(dbg.contains("dbg"));
}

#[test]
fn capability_matches_dag_socket_names() {
    assert!(capability_matches_socket_name("dag.session", "dag"));
    assert!(capability_matches_socket_name(
        "dag.vertex.create",
        "dag-alt"
    ));
}

#[test]
fn capability_matches_braid_socket_names() {
    assert!(capability_matches_socket_name(
        "contribution.record",
        "braid-record"
    ));
    assert!(capability_matches_socket_name("braid.create", "braid"));
}

#[test]
fn capability_matches_spine_socket_names() {
    assert!(capability_matches_socket_name(
        "entry.append",
        "spine-append"
    ));
    assert!(capability_matches_socket_name("spine.create", "spine"));
}

#[test]
fn blake3_hash_empty() {
    let h = blake3_hash(b"");
    let _ = h;
}

#[test]
fn blake3_hash_long_data() {
    let data = vec![0u8; 10000];
    let h = blake3_hash(&data);
    let h2 = blake3_hash(&data);
    assert_eq!(h, h2);
}

#[test]
fn with_sockets_all_none() {
    let client = ProvenanceTrioClient::with_sockets(None, None, None);
    let a = client.availability();
    assert!(!a.is_full());
    assert!(!a.is_partial());
}

#[test]
fn with_sockets_all_some() {
    let client = ProvenanceTrioClient::with_sockets(
        Some("/tmp/e.sock".to_string()),
        Some("/tmp/a.sock".to_string()),
        Some("/tmp/l.sock".to_string()),
    );
    let a = client.availability();
    assert!(a.is_full());
    assert!(a.is_partial());
}

#[tokio::test]
async fn begin_session_with_domain() {
    let client = ProvenanceTrioClient::with_sockets(None, None, None);
    let session = client
        .begin_session("s2", "Viz with Domain", Some("ecology"))
        .await;
    assert_eq!(session.session_id, "s2");
}

#[tokio::test]
async fn begin_session_domain_none() {
    let client = ProvenanceTrioClient::with_sockets(None, None, None);
    let session = client.begin_session("s3", "Title Only", None).await;
    assert_eq!(session.session_id, "s3");
}
