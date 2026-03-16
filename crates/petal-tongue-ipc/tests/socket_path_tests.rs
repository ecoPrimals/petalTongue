// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Integration tests for socket_path.

use petal_tongue_ipc::socket_path;
use std::path::Path;

#[test]
fn test_socket_exists_true_for_socket_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("test.sock");
    std::fs::write(&sock, "").expect("write");
    assert!(socket_path::socket_exists(&sock));
}

#[test]
fn test_socket_exists_false_for_nonexistent() {
    assert!(!socket_path::socket_exists(Path::new(
        "/nonexistent/path/to/socket.sock"
    )));
}

#[test]
fn test_socket_exists_false_for_directory() {
    let dir = tempfile::tempdir().expect("tempdir");
    assert!(!socket_path::socket_exists(dir.path()));
}

#[test]
fn test_get_petaltongue_socket_path_with_custom_parent() {
    let dir = tempfile::tempdir().expect("tempdir");
    let custom_sock = dir.path().join("nested").join("custom.sock");
    let sock_str = custom_sock.to_string_lossy();
    let result = petal_tongue_core::test_fixtures::env_test_helpers::with_env_var(
        "PETALTONGUE_SOCKET",
        sock_str.as_ref(),
        socket_path::get_petaltongue_socket_path,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), custom_sock);
    assert!(custom_sock.parent().unwrap().exists());
}
