// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;
use crate::json_rpc::JsonRpcRequest;
use std::time::Duration;

#[test]
fn test_client_creation() {
    let client = JsonRpcClient::new("/tmp/test.sock").expect("valid path");
    assert_eq!(client.socket_path(), std::path::Path::new("/tmp/test.sock"));
    assert_eq!(client.timeout(), Duration::from_secs(5));
}

#[test]
fn test_client_creation_empty_path() {
    let result = JsonRpcClient::new("");
    assert!(result.is_err());
}

#[test]
fn test_with_timeout() {
    let client =
        JsonRpcClient::with_timeout("/tmp/test.sock", Duration::from_secs(10)).expect("valid path");
    assert_eq!(client.timeout(), Duration::from_secs(10));
}

#[test]
fn test_debug_impl() {
    let client = JsonRpcClient::new("/tmp/test.sock").expect("valid path");
    let debug_str = format!("{client:?}");
    assert!(debug_str.contains("JsonRpcClient"));
    assert!(debug_str.contains("/tmp/test.sock"));
}

#[tokio::test]
async fn test_call_nonexistent_socket() {
    let client =
        JsonRpcClient::new("/tmp/nonexistent-jsonrpc-test-12345.sock").expect("valid path");
    let result = client.call("health.check", serde_json::json!({})).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_notify_nonexistent_socket() {
    let client =
        JsonRpcClient::new("/tmp/nonexistent-jsonrpc-notify-12345.sock").expect("valid path");
    let result = client.notify("some.method", serde_json::json!({})).await;
    assert!(result.is_err());
}

#[test]
fn test_topology_data_structure() {
    let data = TopologyData {
        nodes: vec![serde_json::json!({"id": "n1"})],
        edges: vec![serde_json::json!({"from": "n1", "to": "n2"})],
    };
    let json = serde_json::to_value(&data).expect("serialize");
    assert_eq!(json["nodes"].as_array().expect("nodes").len(), 1);
    assert_eq!(json["edges"].as_array().expect("edges").len(), 1);
}

#[test]
fn test_topology_data_empty() {
    let data = TopologyData {
        nodes: vec![],
        edges: vec![],
    };
    let json = serde_json::to_value(&data).expect("serialize");
    assert!(json["nodes"].as_array().expect("nodes").is_empty());
}

#[test]
fn test_client_clone() {
    let client = JsonRpcClient::new("/tmp/clone-test.sock").expect("valid path");
    let cloned = client.clone();
    assert_eq!(client.socket_path(), cloned.socket_path());
}

#[test]
fn test_json_rpc_client_error_display() {
    let err = JsonRpcClientError::Connection("test".to_string());
    let s = format!("{err}");
    assert!(s.contains("Connection"));
    assert!(s.contains("test"));
}

#[test]
fn test_json_rpc_client_rpc_error() {
    let err = JsonRpcClientError::RpcError {
        code: -32601,
        message: "Method not found".to_string(),
        data: None,
    };
    let s = format!("{err}");
    assert!(s.contains("-32601"));
    assert!(s.contains("Method not found"));
}

#[test]
fn test_json_rpc_client_empty_path_error() {
    let result = JsonRpcClient::new("");
    assert!(result.is_err());
    if let Err(JsonRpcClientError::Connection(msg)) = result {
        assert!(msg.contains("empty"));
    } else {
        panic!("Expected Connection error");
    }
}

#[test]
fn test_json_rpc_request_format() {
    use crate::json_rpc::JsonRpcRequest;
    let req = JsonRpcRequest::new(
        "test.method",
        serde_json::json!({"a": 1}),
        serde_json::json!(42),
    );
    assert_eq!(req.jsonrpc, "2.0");
    assert_eq!(req.method, "test.method");
    assert_eq!(req.id, serde_json::json!(42));
}

#[test]
fn test_next_id_increments() {
    let client = JsonRpcClient::new("/tmp/test.sock").expect("valid path");
    let id1 = client.next_id();
    let id2 = client.next_id();
    assert_eq!(id2, id1 + 1);
}

#[test]
fn test_extract_result_no_result() {
    use crate::json_rpc::JsonRpcResponse;
    let client = JsonRpcClient::new("/tmp/test.sock").expect("valid path");
    let resp = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: None,
        error: None,
        id: serde_json::json!(1),
    };
    let result = client.extract_result(resp, 1);
    assert!(result.is_err());
    if let Err(JsonRpcClientError::InvalidResponse(msg)) = result {
        assert!(msg.contains("no result"));
    } else {
        panic!("Expected InvalidResponse");
    }
}

#[test]
fn test_extract_result_success() {
    use crate::json_rpc::JsonRpcResponse;
    let client = JsonRpcClient::new("/tmp/test.sock").expect("valid path");
    let resp = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(serde_json::json!({"status": "ok"})),
        error: None,
        id: serde_json::json!(1),
    };
    let result = client.extract_result(resp, 1);
    assert!(result.is_ok());
    let val = result.expect("ok");
    assert_eq!(val["status"], "ok");
}

#[test]
fn test_json_rpc_client_rpc_error_with_data() {
    let err = JsonRpcClientError::RpcError {
        code: -32602,
        message: "Invalid params".to_string(),
        data: Some(serde_json::json!({"param": "id"})),
    };
    let s = format!("{err}");
    assert!(s.contains("-32602"));
    assert!(s.contains("Invalid params"));
}

#[tokio::test]
async fn test_batch_nonexistent_socket() {
    let client = JsonRpcClient::new("/tmp/nonexistent-batch-99999.sock").expect("valid path");
    let requests = vec![
        ("health.check", serde_json::json!({})),
        ("topology.get", serde_json::json!({})),
    ];
    let result = client.batch(requests).await;
    assert!(result.is_err());
}

#[test]
fn test_json_rpc_response_error_extraction() {
    use crate::json_rpc::JsonRpcResponse;
    let resp = JsonRpcResponse::error(
        serde_json::json!(1),
        crate::json_rpc::error_codes::METHOD_NOT_FOUND,
        "Method not found",
    );
    assert!(resp.result.is_none());
    assert!(resp.error.is_some());
}

#[test]
fn test_request_serialization_roundtrip() {
    use crate::json_rpc::JsonRpcRequest;
    let req = JsonRpcRequest::new(
        "topology.get",
        serde_json::json!({"filter": "nodes"}),
        serde_json::json!(42),
    );
    let json = serde_json::to_string(&req).expect("serialize");
    let parsed: JsonRpcRequest = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.method, "topology.get");
    assert_eq!(parsed.params["filter"], "nodes");
    assert_eq!(parsed.id, serde_json::json!(42));
}

#[test]
fn test_response_success_deserialization() {
    use crate::json_rpc::JsonRpcResponse;
    let json = r#"{"jsonrpc":"2.0","result":{"nodes":[],"edges":[]},"id":1}"#;
    let resp: JsonRpcResponse = serde_json::from_str(json).expect("deserialize");
    assert!(resp.result.is_some());
    assert!(resp.error.is_none());
    assert!(resp.result.as_ref().expect("result")["nodes"].is_array());
}

#[test]
fn test_response_error_deserialization() {
    use crate::json_rpc::JsonRpcResponse;
    let json = r#"{"jsonrpc":"2.0","error":{"code":-32601,"message":"Method not found"},"id":1}"#;
    let resp: JsonRpcResponse = serde_json::from_str(json).expect("deserialize");
    assert!(resp.result.is_none());
    let err = resp.error.as_ref().expect("error");
    assert_eq!(err.code, -32601);
    assert_eq!(err.message, "Method not found");
}

#[test]
fn test_method_name_formatting() {
    use crate::json_rpc::JsonRpcRequest;
    let req = JsonRpcRequest::new(
        "capability.list",
        serde_json::json!({}),
        serde_json::json!(1),
    );
    assert_eq!(req.method, "capability.list");
    let req2 = JsonRpcRequest::new("health.check", serde_json::json!({}), serde_json::json!(2));
    assert_eq!(req2.method, "health.check");
}

#[test]
fn test_request_includes_id() {
    use crate::json_rpc::JsonRpcRequest;
    let req = JsonRpcRequest::new("test.method", serde_json::json!({}), serde_json::json!(99));
    assert_eq!(req.id, serde_json::json!(99));
    let req_null = JsonRpcRequest::new(
        "notify.method",
        serde_json::json!({}),
        serde_json::Value::Null,
    );
    assert!(req_null.id.is_null());
}

#[test]
fn test_json_rpc_client_error_timeout() {
    let err = JsonRpcClientError::Timeout("Connection timeout".to_string());
    let s = format!("{err}");
    assert!(s.contains("Timeout"));
    assert!(s.contains("Connection timeout"));
}

#[test]
fn test_json_rpc_client_error_serialization() {
    let err = JsonRpcClientError::Serialization("JSON parse failed".to_string());
    let s = format!("{err}");
    assert!(s.contains("Serialization"));
    assert!(s.contains("JSON parse failed"));
}

#[test]
fn test_json_rpc_client_error_invalid_response() {
    let err = JsonRpcClientError::InvalidResponse("Malformed JSON".to_string());
    let s = format!("{err}");
    assert!(s.contains("Invalid response"));
    assert!(s.contains("Malformed JSON"));
}

#[test]
fn test_json_rpc_client_error_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
    let err = JsonRpcClientError::Io(io_err);
    let s = format!("{err}");
    assert!(s.contains("refused") || s.contains("I/O"));
}

#[test]
fn test_batch_request_structure() {
    let requests = [
        ("health.check", serde_json::json!({})),
        ("topology.get", serde_json::json!({})),
    ];
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].0, "health.check");
    assert_eq!(requests[1].0, "topology.get");
}

#[test]
fn test_topology_data_serialization() {
    let data = TopologyData {
        nodes: vec![
            serde_json::json!({"id": "n1", "x": 0.0}),
            serde_json::json!({"id": "n2", "x": 1.0}),
        ],
        edges: vec![serde_json::json!({"from": "n1", "to": "n2"})],
    };
    let json = serde_json::to_value(&data).expect("serialize");
    assert_eq!(json["nodes"].as_array().expect("nodes").len(), 2);
    assert_eq!(json["edges"].as_array().expect("edges").len(), 1);
}

#[tokio::test]
async fn test_send_request_invalid_json_response() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("invalid-json.sock");
    let listener = tokio::net::UnixListener::bind(&sock).expect("bind");
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        let (mut reader, mut writer) = stream.into_split();
        let mut buf = [0u8; 1024];
        let _ = tokio::io::AsyncReadExt::read(&mut reader, &mut buf).await;
        writer.write_all(b"{not valid json\n").await.expect("write");
        writer.flush().await.expect("flush");
    });
    let client =
        JsonRpcClient::with_timeout(sock.as_os_str(), Duration::from_millis(500)).expect("client");
    let req = JsonRpcRequest::new("test.method", serde_json::json!({}), serde_json::json!(1));
    let result = client.send_request(&req).await;
    assert!(result.is_err());
    if let Err(JsonRpcClientError::InvalidResponse(msg)) = result {
        assert!(msg.contains("Invalid JSON") || msg.contains("invalid"));
    } else {
        panic!("Expected InvalidResponse");
    }
}

#[tokio::test]
async fn test_send_request_rpc_error_response() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("rpc-error.sock");
    let listener = tokio::net::UnixListener::bind(&sock).expect("bind");
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        let (mut reader, mut writer) = stream.into_split();
        let mut buf = [0u8; 1024];
        let _ = tokio::io::AsyncReadExt::read(&mut reader, &mut buf).await;
        let err_resp =
            r#"{"jsonrpc":"2.0","error":{"code":-32601,"message":"Method not found"},"id":1}"#;
        writer
            .write_all(format!("{err_resp}\n").as_bytes())
            .await
            .expect("write");
        writer.flush().await.expect("flush");
    });
    let client =
        JsonRpcClient::with_timeout(sock.as_os_str(), Duration::from_millis(500)).expect("client");
    let req = JsonRpcRequest::new("test.method", serde_json::json!({}), serde_json::json!(1));
    let result = client.send_request(&req).await;
    assert!(result.is_err());
    if let Err(JsonRpcClientError::RpcError { code, message, .. }) = result {
        assert_eq!(code, -32601);
        assert!(message.contains("Method not found"));
    } else {
        panic!("Expected RpcError");
    }
}

#[tokio::test]
async fn test_send_request_empty_response() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("empty.sock");
    let listener = tokio::net::UnixListener::bind(&sock).expect("bind");
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        let (mut reader, mut writer) = stream.into_split();
        let mut buf = [0u8; 1024];
        let _ = tokio::io::AsyncReadExt::read(&mut reader, &mut buf).await;
        writer.write_all(b"\n").await.expect("write");
        writer.flush().await.expect("flush");
    });
    let client =
        JsonRpcClient::with_timeout(sock.as_os_str(), Duration::from_millis(500)).expect("client");
    let req = JsonRpcRequest::new("test.method", serde_json::json!({}), serde_json::json!(1));
    let result = client.send_request(&req).await;
    assert!(result.is_err());
    assert!(result.is_err());
    let err_str = format!("{}", result.unwrap_err());
    assert!(
        err_str.contains("Empty") || err_str.contains("empty") || err_str.contains("Invalid"),
        "expected empty/invalid response error: {err_str}"
    );
}

#[tokio::test]
async fn test_call_success_via_mock_server() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("success.sock");
    let listener = tokio::net::UnixListener::bind(&sock).expect("bind");
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        let (reader, mut writer) = stream.into_split();
        let mut line = String::new();
        tokio::io::AsyncBufReadExt::read_line(&mut tokio::io::BufReader::new(reader), &mut line)
            .await
            .expect("read");
        let resp = r#"{"jsonrpc":"2.0","result":{"ok":true},"id":1}"#;
        writer
            .write_all(format!("{resp}\n").as_bytes())
            .await
            .expect("write");
        writer.flush().await.expect("flush");
    });
    let client =
        JsonRpcClient::with_timeout(sock.as_os_str(), Duration::from_millis(500)).expect("client");
    let result = client.call("test.method", serde_json::json!({})).await;
    assert!(result.is_ok());
    let val = result.expect("ok");
    assert_eq!(val["ok"], true);
}

#[tokio::test]
async fn test_connection_timeout_to_nonexistent() {
    let client = JsonRpcClient::with_timeout(
        "/tmp/nonexistent-timeout-test-99999.sock",
        Duration::from_millis(10),
    )
    .expect("client");
    let result = client.call("health.check", serde_json::json!({})).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_str = format!("{err}");
    assert!(
        err_str.contains("Timeout") || err_str.contains("Connection") || err_str.contains("Failed"),
        "expected timeout or connection error, got: {err_str}"
    );
}

#[test]
fn test_topology_data_deserialization_roundtrip() {
    let json = serde_json::json!({
        "nodes": [{"id": "n1", "x": 0.0}, {"id": "n2", "x": 1.0}],
        "edges": [{"from": "n1", "to": "n2"}]
    });
    let data: TopologyData = serde_json::from_value(json).expect("deserialize");
    let serialized = serde_json::to_value(&data).expect("serialize");
    assert_eq!(serialized["nodes"].as_array().unwrap().len(), 2);
    assert_eq!(serialized["edges"].as_array().unwrap().len(), 1);
}

#[test]
fn test_request_notification_null_id() {
    use crate::json_rpc::JsonRpcRequest;
    let req = JsonRpcRequest::new(
        "notify.method",
        serde_json::json!({}),
        serde_json::Value::Null,
    );
    let json = serde_json::to_string(&req).expect("serialize");
    assert!(json.contains("null"));
    assert!(req.id.is_null());
}

#[test]
fn test_primal_info_deserialization_from_discover_format() {
    let json = serde_json::json!([{
        "id": "p1",
        "name": "petaltongue",
        "primal_type": "petaltongue",
        "endpoint": "/primal/petaltongue",
        "capabilities": ["ui.render", "graph.topology"],
        "health": "Healthy",
        "last_seen": 1_234_567_890
    }]);
    let primals: Vec<petal_tongue_core::PrimalInfo> =
        serde_json::from_value(json).expect("deserialize");
    assert_eq!(primals.len(), 1);
    assert_eq!(primals[0].id.as_str(), "p1");
    assert_eq!(primals[0].name, "petaltongue");
    assert!(matches!(
        primals[0].health,
        petal_tongue_core::PrimalHealthStatus::Healthy
    ));
}

#[tokio::test]
async fn test_send_request_read_timeout() {
    // Server accepts connection, reads request, but never sends response
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("read-timeout.sock");
    let listener = tokio::net::UnixListener::bind(&sock).expect("bind");
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        let (mut reader, _writer) = stream.into_split();
        let mut buf = [0u8; 1024];
        let _ = tokio::io::AsyncReadExt::read(&mut reader, &mut buf).await;
        // Never send response - client will timeout on read
        tokio::time::sleep(Duration::from_secs(5)).await;
    });
    let client =
        JsonRpcClient::with_timeout(sock.as_os_str(), Duration::from_millis(100)).expect("client");
    let req = JsonRpcRequest::new("test.method", serde_json::json!({}), serde_json::json!(1));
    let result = client.send_request(&req).await;
    assert!(result.is_err());
    let err_str = format!("{}", result.unwrap_err());
    assert!(
        err_str.contains("Timeout") || err_str.contains("timeout"),
        "expected timeout error: {err_str}"
    );
}
