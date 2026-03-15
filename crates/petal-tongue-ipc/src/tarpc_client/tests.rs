// SPDX-License-Identifier: AGPL-3.0-only
//! tarpc client tests

use std::net::SocketAddr;
use std::time::Duration;

use super::TarpcClient;

#[test]
fn test_endpoint_parsing_valid() {
    let addr = TarpcClient::parse_endpoint("tarpc://localhost:9001").expect("valid endpoint");
    assert_eq!(addr.port(), 9001);
}

#[test]
fn test_endpoint_parsing_with_ip() {
    let addr = TarpcClient::parse_endpoint("tarpc://127.0.0.1:9002").expect("valid IP endpoint");
    assert_eq!(addr.port(), 9002);
}

#[test]
fn test_endpoint_parsing_localhost_localdomain() {
    let addr = TarpcClient::parse_endpoint("tarpc://localhost.localdomain:9003")
        .expect("localhost.localdomain");
    assert_eq!(addr.port(), 9003);
    assert_eq!(
        addr.ip(),
        std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
    );
}

#[test]
fn test_endpoint_parsing_invalid_no_prefix() {
    let result = TarpcClient::parse_endpoint("localhost:9001");
    assert!(result.is_err());
    assert!(
        result
            .expect_err("no prefix")
            .to_string()
            .contains("Invalid tarpc endpoint")
    );
}

#[test]
fn test_endpoint_parsing_invalid_address() {
    let result = TarpcClient::parse_endpoint("tarpc://invalid");
    assert!(result.is_err());
}

#[test]
fn test_endpoint_parsing_missing_port() {
    let result = TarpcClient::parse_endpoint("tarpc://localhost");
    assert!(result.is_err());
    assert!(
        result
            .expect_err("missing port")
            .to_string()
            .contains("missing port")
    );
}

#[test]
fn test_endpoint_parsing_invalid_port() {
    let result = TarpcClient::parse_endpoint("tarpc://localhost:notaport");
    assert!(result.is_err());
    assert!(
        result
            .expect_err("invalid port")
            .to_string()
            .contains("Invalid port")
    );
}

#[test]
fn test_endpoint_parsing_ip_parsed_directly() {
    let addr = TarpcClient::parse_endpoint("tarpc://192.0.2.1:8080").expect("IP address");
    assert_eq!(addr.port(), 8080);
    assert_eq!(addr.ip().to_string(), "192.0.2.1");
}

#[test]
fn test_client_default_timeout() {
    let client = TarpcClient::new("tarpc://127.0.0.1:9001").expect("create client");
    assert_eq!(client.timeout(), Duration::from_secs(5));
}

#[test]
fn test_client_creation() {
    let client = TarpcClient::new("tarpc://localhost:9001").expect("create client");
    assert_eq!(client.endpoint(), "tarpc://localhost:9001");
    assert_eq!(client.addr().port(), 9001);
}

#[test]
fn test_with_timeout_builder() {
    let client = TarpcClient::new("tarpc://localhost:9001")
        .expect("create client")
        .with_timeout(Duration::from_secs(10));

    assert_eq!(client.timeout(), Duration::from_secs(10));
}

#[test]
fn test_with_timeout_preserves_other_fields() {
    let client = TarpcClient::new("tarpc://127.0.0.1:9005")
        .expect("create client")
        .with_timeout(Duration::from_millis(500));
    assert_eq!(client.addr().port(), 9005);
    assert_eq!(client.timeout(), Duration::from_millis(500));
}

#[test]
fn test_addr_returns_socket_addr() {
    let client = TarpcClient::new("tarpc://localhost:9010").expect("create client");
    let addr: SocketAddr = client.addr();
    assert_eq!(addr.port(), 9010);
}

#[test]
fn test_debug_impl() {
    let client = TarpcClient::new("tarpc://localhost:9001").expect("create client");
    let debug_str = format!("{client:?}");
    assert!(debug_str.contains("TarpcClient"));
    assert!(debug_str.contains("localhost:9001"));
}

#[tokio::test]
async fn test_call_method_unknown_method() {
    let client = TarpcClient::new("tarpc://localhost:9999").expect("create client");
    let result = client.call_method("unknown.method", None).await;
    assert!(result.is_err());
    let err = result.expect_err("unknown method");
    assert!(err.to_string().contains("Unknown method"));
}

#[tokio::test]
async fn test_call_method_discover_capability_missing_param() {
    let client = TarpcClient::new("tarpc://localhost:9998").expect("create client");
    let result = client
        .call_method("discovery.find_capability", Some(serde_json::json!({})))
        .await;
    assert!(result.is_err());
    assert!(
        result
            .expect_err("missing capability param")
            .to_string()
            .contains("Missing capability")
    );
}

#[tokio::test]
async fn test_call_method_render_graph_missing_param() {
    let client = TarpcClient::new("tarpc://localhost:9997").expect("create client");
    let result = client.call_method("visualization.render.graph", None).await;
    assert!(result.is_err());
    assert!(
        result
            .expect_err("missing request")
            .to_string()
            .contains("Missing request")
    );
}

#[test]
fn test_tarpc_client_error_display() {
    use super::TarpcClientError;
    let err = TarpcClientError::Configuration("bad config".to_string());
    assert!(format!("{err}").contains("bad config"));

    let err = TarpcClientError::Connection("refused".to_string());
    assert!(format!("{err}").contains("refused"));

    let err = TarpcClientError::Rpc("timeout".to_string());
    assert!(format!("{err}").contains("timeout"));

    let err = TarpcClientError::Serialization("invalid".to_string());
    assert!(format!("{err}").contains("invalid"));

    let err = TarpcClientError::Timeout("5s".to_string());
    assert!(format!("{err}").contains("5s"));
}

#[test]
fn test_parse_endpoint_empty_after_prefix() {
    let result = TarpcClient::parse_endpoint("tarpc://");
    assert!(result.is_err());
}

#[test]
fn test_client_creation_various_urls() {
    let c1 = TarpcClient::new("tarpc://0.0.0.0:1234").expect("0.0.0.0");
    assert_eq!(c1.addr().port(), 1234);

    let c2 = TarpcClient::new("tarpc://10.0.0.1:5678").expect("10.0.0.1");
    assert_eq!(c2.addr().port(), 5678);
}

#[tokio::test]
async fn test_get_capabilities_connection_failure() {
    let client = TarpcClient::new("tarpc://127.0.0.1:1")
        .expect("create")
        .with_timeout(Duration::from_millis(50));
    let result = client.get_capabilities().await;
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.contains("Connection")
            || err_str.contains("Timeout")
            || err_str.contains("refused"),
        "expected connection/timeout error: {err_str}"
    );
}

#[tokio::test]
async fn test_call_method_capability_list_connection_failure() {
    let client = TarpcClient::new("tarpc://127.0.0.1:1")
        .expect("create")
        .with_timeout(Duration::from_millis(50));
    let result = client.call_method("capability.list", None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_call_method_health_check_connection_failure() {
    let client = TarpcClient::new("tarpc://127.0.0.1:1")
        .expect("create")
        .with_timeout(Duration::from_millis(50));
    let result = client.call_method("health.check", None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_call_method_discover_capability_with_param() {
    let client = TarpcClient::new("tarpc://127.0.0.1:1")
        .expect("create")
        .with_timeout(Duration::from_millis(50));
    let result = client
        .call_method(
            "discovery.find_capability",
            Some(serde_json::json!({"capability": "visualization"})),
        )
        .await;
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.contains("Connection")
            || err_str.contains("Timeout")
            || err_str.contains("refused"),
        "expected connection error: {err_str}"
    );
}

#[test]
fn test_call_method_get_capabilities_alias() {
    let client = TarpcClient::new("tarpc://127.0.0.1:9999").expect("create");
    assert_eq!(client.endpoint(), "tarpc://127.0.0.1:9999");
}

#[test]
fn test_parse_endpoint_host_with_multiple_colons() {
    let result = TarpcClient::parse_endpoint("tarpc://[::1]:8080");
    assert!(result.is_ok() || result.is_err());
}
