// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn test_detect_protocol() {
    assert_eq!(detect_protocol("tarpc://localhost:9001"), Protocol::Tarpc);
    assert_eq!(
        detect_protocol("unix:///tmp/service.sock"),
        Protocol::JsonRpc
    );
    assert_eq!(detect_protocol("ipc:///tmp/service"), Protocol::JsonRpc);
    assert_eq!(detect_protocol("http://localhost:8080"), Protocol::Https);
    assert_eq!(detect_protocol("https://api.example.com"), Protocol::Https);
}

#[test]
fn test_detect_protocol_unknown_fallback() {
    // Unknown protocols default to HTTPS fallback
    assert_eq!(detect_protocol("unknown://foo"), Protocol::Https);
    assert_eq!(detect_protocol(""), Protocol::Https);
    assert_eq!(detect_protocol("ftp://example.com"), Protocol::Https);
}

#[test]
fn test_protocol_priority() {
    assert!(Protocol::Tarpc < Protocol::JsonRpc);
    assert!(Protocol::JsonRpc < Protocol::Https);
}

#[test]
fn test_protocol_ord() {
    assert!(Protocol::Tarpc < Protocol::Https);
    assert!(Protocol::Tarpc <= Protocol::Tarpc);
    assert!(Protocol::Https >= Protocol::JsonRpc);
}

#[test]
fn test_detected_protocol() {
    let d = DetectedProtocol {
        protocol: Protocol::Tarpc,
        endpoint: "tarpc://localhost:9001".to_string(),
    };
    assert_eq!(d.protocol, Protocol::Tarpc);
    assert_eq!(d.endpoint, "tarpc://localhost:9001");
}

#[test]
fn test_parse_unix_socket_path() {
    let path = super::connect::parse_unix_socket_path("unix:///tmp/service.sock").unwrap();
    assert_eq!(path.to_string_lossy(), "/tmp/service.sock");

    let path = super::connect::parse_unix_socket_path("ipc:///var/run/app.sock").unwrap();
    assert_eq!(path.to_string_lossy(), "/var/run/app.sock");
}

#[test]
fn test_parse_unix_socket_path_invalid() {
    assert!(super::connect::parse_unix_socket_path("tarpc://localhost:9001").is_err());
    assert!(super::connect::parse_unix_socket_path("http://localhost").is_err());
    assert!(super::connect::parse_unix_socket_path("invalid").is_err());
}

#[test]
fn test_parse_unix_socket_path_empty_after_prefix() {
    let path = super::connect::parse_unix_socket_path("unix://").expect("empty path is valid");
    assert_eq!(path.to_string_lossy(), "");
}

#[test]
fn test_detect_protocol_trailing_slash() {
    assert_eq!(detect_protocol("https://api.example.com/"), Protocol::Https);
    assert_eq!(detect_protocol("http://localhost:8080/"), Protocol::Https);
}

#[test]
fn test_detected_protocol_all_variants() {
    let tarpc = DetectedProtocol {
        protocol: Protocol::Tarpc,
        endpoint: "tarpc://host:1".to_string(),
    };
    assert_eq!(tarpc.protocol, Protocol::Tarpc);

    let jsonrpc = DetectedProtocol {
        protocol: Protocol::JsonRpc,
        endpoint: "unix:///tmp/sock".to_string(),
    };
    assert_eq!(jsonrpc.protocol, Protocol::JsonRpc);

    let https = DetectedProtocol {
        protocol: Protocol::Https,
        endpoint: "https://example.com".to_string(),
    };
    assert_eq!(https.protocol, Protocol::Https);
}

#[test]
fn test_protocol_equality() {
    assert_eq!(Protocol::Tarpc, Protocol::Tarpc);
    assert_ne!(Protocol::Tarpc, Protocol::Https);
}

#[test]
fn test_jsonrpc_to_tarpc_error_mapping() {
    use petal_tongue_ipc::{JsonRpcClientError, TarpcClientError};

    let jsonrpc_err = JsonRpcClientError::Connection("conn".to_string());
    let tarpc_err = super::connection::jsonrpc_to_tarpc_error(jsonrpc_err);
    assert!(matches!(tarpc_err, TarpcClientError::Connection(_)));

    let jsonrpc_err = JsonRpcClientError::Timeout("timeout".to_string());
    let tarpc_err = super::connection::jsonrpc_to_tarpc_error(jsonrpc_err);
    assert!(matches!(tarpc_err, TarpcClientError::Timeout(_)));

    let jsonrpc_err = JsonRpcClientError::Serialization("ser".to_string());
    let tarpc_err = super::connection::jsonrpc_to_tarpc_error(jsonrpc_err);
    assert!(matches!(tarpc_err, TarpcClientError::Serialization(_)));
}

#[test]
fn test_jsonrpc_to_tarpc_error_rpc_error() {
    use petal_tongue_ipc::{JsonRpcClientError, TarpcClientError};

    let jsonrpc_err = JsonRpcClientError::RpcError {
        code: -32601,
        message: "Method not found".to_string(),
        data: None,
    };
    let tarpc_err = super::connection::jsonrpc_to_tarpc_error(jsonrpc_err);
    assert!(matches!(tarpc_err, TarpcClientError::Rpc(_)));
}

#[test]
fn test_jsonrpc_to_tarpc_error_invalid_response() {
    use petal_tongue_ipc::{JsonRpcClientError, TarpcClientError};

    let jsonrpc_err = JsonRpcClientError::InvalidResponse("bad json".to_string());
    let tarpc_err = super::connection::jsonrpc_to_tarpc_error(jsonrpc_err);
    assert!(matches!(tarpc_err, TarpcClientError::Rpc(_)));
}

#[test]
fn test_jsonrpc_to_tarpc_error_io() {
    use petal_tongue_ipc::{JsonRpcClientError, TarpcClientError};

    let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
    let jsonrpc_err = JsonRpcClientError::Io(io_err);
    let tarpc_err = super::connection::jsonrpc_to_tarpc_error(jsonrpc_err);
    assert!(matches!(tarpc_err, TarpcClientError::Connection(_)));
}

#[test]
fn test_jsonrpc_to_tarpc_error_rpc_error_code_ranges() {
    use petal_tongue_ipc::{JsonRpcClientError, TarpcClientError};

    // Parse error (-32700)
    let err = JsonRpcClientError::RpcError {
        code: -32700,
        message: "Parse error".to_string(),
        data: None,
    };
    let tarpc = super::connection::jsonrpc_to_tarpc_error(err);
    assert!(matches!(tarpc, TarpcClientError::Rpc(_)));

    // Invalid params (-32602)
    let err = JsonRpcClientError::RpcError {
        code: -32602,
        message: "Invalid params".to_string(),
        data: Some(serde_json::json!({"hint": "x"})),
    };
    let tarpc = super::connection::jsonrpc_to_tarpc_error(err);
    assert!(matches!(tarpc, TarpcClientError::Rpc(_)));

    // Server error range (-32000 to -32099)
    let err = JsonRpcClientError::RpcError {
        code: -32000,
        message: "Server error".to_string(),
        data: None,
    };
    let tarpc = super::connection::jsonrpc_to_tarpc_error(err);
    assert!(matches!(tarpc, TarpcClientError::Rpc(_)));
}

#[test]
fn test_protocol_debug() {
    assert!(format!("{:?}", Protocol::Tarpc).contains("Tarpc"));
    assert!(format!("{:?}", Protocol::JsonRpc).contains("JsonRpc"));
    assert!(format!("{:?}", Protocol::Https).contains("Https"));
}

#[test]
fn test_detected_protocol_debug() {
    let d = DetectedProtocol {
        protocol: Protocol::Tarpc,
        endpoint: "tarpc://host:1".to_string(),
    };
    let s = format!("{:?}", d);
    assert!(s.contains("Tarpc"));
    assert!(s.contains("tarpc://host:1"));
}

#[test]
fn test_parse_unix_socket_path_ipc_empty() {
    let path = super::connect::parse_unix_socket_path("ipc://").expect("ipc empty path valid");
    assert_eq!(path.to_string_lossy(), "");
}

#[test]
fn test_parse_unix_socket_path_relative() {
    let path = super::connect::parse_unix_socket_path("unix://./relative.sock").unwrap();
    assert_eq!(path.to_string_lossy(), "./relative.sock");
}

#[test]
fn test_parse_unix_socket_path_malformed_no_scheme() {
    assert!(super::connect::parse_unix_socket_path("").is_err());
    assert!(super::connect::parse_unix_socket_path("/tmp/sock").is_err());
}

#[tokio::test]
async fn test_connect_with_priority_tarpc_unavailable() {
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        connect_with_priority("tarpc://127.0.0.1:19999"),
    )
    .await;
    assert!(result.is_ok(), "should not hang");
    let conn_result = result.unwrap();
    assert!(conn_result.is_err(), "no server on port 19999");
}

#[tokio::test]
async fn test_connect_with_priority_unix_unavailable() {
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        connect_with_priority("unix:///tmp/nonexistent-socket-xyz-12345.sock"),
    )
    .await;
    assert!(result.is_ok(), "should not hang");
    let conn_result = result.unwrap();
    assert!(conn_result.is_err(), "socket does not exist");
}

#[tokio::test]
async fn test_connect_with_priority_https_unavailable() {
    // Use localhost with no server - fails quickly (connection refused)
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        connect_with_priority("https://127.0.0.1:19998/"),
    )
    .await;
    assert!(result.is_ok(), "should not hang");
    let conn_result = result.unwrap();
    assert!(conn_result.is_err(), "no server on port 19998");
}

#[test]
fn test_detect_protocol_tarpc_variants() {
    assert_eq!(detect_protocol("tarpc://127.0.0.1:9001"), Protocol::Tarpc);
    assert_eq!(
        detect_protocol("tarpc://host.example.com:12345"),
        Protocol::Tarpc
    );
}

#[test]
fn test_detect_protocol_https_variants() {
    assert_eq!(detect_protocol("https://example.com/api"), Protocol::Https);
    assert_eq!(detect_protocol("http://localhost"), Protocol::Https);
}

#[test]
fn test_protocol_ord_consistency() {
    assert!(Protocol::Tarpc <= Protocol::Tarpc);
    assert!(Protocol::JsonRpc > Protocol::Tarpc);
    assert!(Protocol::Https > Protocol::JsonRpc);
}

#[test]
fn test_parse_health_from_json() {
    let v = serde_json::json!({
        "status": "ok",
        "version": "1.0",
        "uptime_seconds": 100,
        "capabilities": ["a", "b"]
    });
    let h = parse_health_from_json(&v);
    assert_eq!(h.status, "ok");
    assert_eq!(h.version, "1.0");
    assert_eq!(h.uptime_seconds, 100);
    assert_eq!(h.capabilities, vec!["a", "b"]);
}

#[test]
fn test_parse_capabilities_from_json() {
    let v = serde_json::json!({"capabilities": ["x", "y"]});
    let c = parse_capabilities_from_json(&v).unwrap();
    assert_eq!(c, vec!["x", "y"]);
}

#[test]
fn test_https_fallback_urls() {
    let urls = https_fallback_urls("https://example.com");
    assert_eq!(urls.len(), 2);
    assert_eq!(urls[0], "https://example.com");
    assert_eq!(urls[1], "http://example.com");
    let urls = https_fallback_urls("http://x");
    assert_eq!(urls.len(), 1);
    assert_eq!(urls[0], "http://x");
}
