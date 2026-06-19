// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

use std::sync::Arc;

use crate::data_service::DataService;

#[test]
fn test_bind_address_parse() {
    use petal_tongue_core::constants::{DEFAULT_HEADLESS_PORT, DEFAULT_LOOPBACK_HOST};
    let bind = format!("{DEFAULT_LOOPBACK_HOST}:{DEFAULT_HEADLESS_PORT}");
    let addr: std::net::SocketAddr = bind.parse().expect("valid bind");
    assert_eq!(addr.port(), DEFAULT_HEADLESS_PORT);
}

#[test]
fn test_invalid_bind_address() {
    let result: Result<std::net::SocketAddr, _> = "not-an-address".parse();
    assert!(result.is_err());
}

#[test]
fn test_bind_address_default_format() {
    use petal_tongue_core::constants::{DEFAULT_BIND_HOST, DEFAULT_WEB_PORT};
    let bind = format!("{DEFAULT_BIND_HOST}:{DEFAULT_WEB_PORT}");
    let addr: std::net::SocketAddr = bind.parse().expect("valid default bind");
    assert_eq!(addr.port(), DEFAULT_WEB_PORT);
    assert!(addr.ip().is_unspecified());
}

#[test]
fn test_bind_address_loopback_with_port() {
    use petal_tongue_core::constants::{DEFAULT_HEADLESS_PORT, DEFAULT_LOOPBACK_HOST};
    let bind = format!("{DEFAULT_LOOPBACK_HOST}:{DEFAULT_HEADLESS_PORT}");
    let addr: std::net::SocketAddr = bind.parse().expect("valid loopback");
    assert_eq!(addr.port(), DEFAULT_HEADLESS_PORT);
    assert!(addr.ip().is_loopback());
}
#[tokio::test]
async fn test_run_invalid_bind_address() {
    let data_service = Arc::new(DataService::new());
    let result = run(test_config("not-a-valid-address"), data_service).await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("parse") || err_msg.contains("bind"));
}

#[tokio::test]
async fn test_run_empty_bind_address() {
    let data_service = Arc::new(DataService::new());
    let result = run(test_config(""), data_service).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_run_invalid_port() {
    let data_service = Arc::new(DataService::new());
    let result = run(test_config("127.0.0.1:999999"), data_service).await;
    assert!(result.is_err());
}
#[tokio::test]
async fn test_run_invalid_docroot_rejects() {
    let data_service = Arc::new(DataService::new());
    let mut cfg = test_config("127.0.0.1:0");
    cfg.docroot = Some("/nonexistent/docroot/path".to_string());
    let result = run(cfg, data_service).await;
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("docroot"),
        "error should mention docroot: {msg}"
    );
}
#[tokio::test]
async fn test_run_content_direct_invalid_docroot_rejects() {
    let mut cfg = test_config("127.0.0.1:0");
    cfg.backend = "content-direct";
    cfg.docroot = Some("/nonexistent/content/dir".to_owned());
    let ds = Arc::new(DataService::new());
    let result = super::run(cfg, ds).await;
    assert!(
        result.is_err(),
        "content-direct with invalid docroot should fail"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Web config") || err_msg.contains("docroot"),
        "error should mention docroot: {err_msg}"
    );
}
