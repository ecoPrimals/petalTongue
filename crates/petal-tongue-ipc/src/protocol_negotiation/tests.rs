// SPDX-License-Identifier: AGPL-3.0-or-later
//! Tests for G65 protocol negotiation.

use super::negotiate::{negotiate_client, negotiate_server, select_protocol, NegotiationResult};
use super::wire::{NegotiationError, ProtocolId, ProtocolRequest, ProtocolResponse};

// ─── Wire format unit tests ──────────────────────────────────────────

#[test]
fn protocol_id_wire_names() {
    assert_eq!(ProtocolId::JsonRpc.wire_name(), "jsonrpc");
    assert_eq!(ProtocolId::Tarpc.wire_name(), "tarpc");
}

#[test]
fn protocol_id_from_wire_name_valid() {
    assert_eq!(ProtocolId::from_wire_name("jsonrpc"), Some(ProtocolId::JsonRpc));
    assert_eq!(ProtocolId::from_wire_name("json-rpc"), Some(ProtocolId::JsonRpc));
    assert_eq!(ProtocolId::from_wire_name("json_rpc"), Some(ProtocolId::JsonRpc));
    assert_eq!(ProtocolId::from_wire_name("JSONRPC"), Some(ProtocolId::JsonRpc));
    assert_eq!(ProtocolId::from_wire_name("tarpc"), Some(ProtocolId::Tarpc));
    assert_eq!(ProtocolId::from_wire_name("TARPC"), Some(ProtocolId::Tarpc));
}

#[test]
fn protocol_id_from_wire_name_invalid() {
    assert_eq!(ProtocolId::from_wire_name("grpc"), None);
    assert_eq!(ProtocolId::from_wire_name(""), None);
    assert_eq!(ProtocolId::from_wire_name("unknown"), None);
}

#[test]
fn protocol_id_default_is_jsonrpc() {
    assert_eq!(ProtocolId::default(), ProtocolId::JsonRpc);
}

#[test]
fn protocol_id_display() {
    assert_eq!(format!("{}", ProtocolId::JsonRpc), "jsonrpc");
    assert_eq!(format!("{}", ProtocolId::Tarpc), "tarpc");
}

#[test]
fn protocol_id_supported_includes_both() {
    let supported = ProtocolId::supported();
    assert!(supported.contains(&ProtocolId::JsonRpc));
    assert!(supported.contains(&ProtocolId::Tarpc));
}

#[test]
fn request_to_wire_single() {
    let req = ProtocolRequest {
        supported: vec![ProtocolId::JsonRpc],
    };
    assert_eq!(req.to_wire(), "PROTOCOLS: jsonrpc\n");
}

#[test]
fn request_to_wire_multi() {
    let req = ProtocolRequest {
        supported: vec![ProtocolId::Tarpc, ProtocolId::JsonRpc],
    };
    assert_eq!(req.to_wire(), "PROTOCOLS: tarpc,jsonrpc\n");
}

#[test]
fn request_from_wire_valid() {
    let req = ProtocolRequest::from_wire("PROTOCOLS: tarpc,jsonrpc\n").unwrap();
    assert_eq!(req.supported, vec![ProtocolId::Tarpc, ProtocolId::JsonRpc]);
}

#[test]
fn request_from_wire_single() {
    let req = ProtocolRequest::from_wire("PROTOCOLS: jsonrpc\n").unwrap();
    assert_eq!(req.supported, vec![ProtocolId::JsonRpc]);
}

#[test]
fn request_from_wire_ignores_unknown() {
    let req = ProtocolRequest::from_wire("PROTOCOLS: grpc,tarpc\n").unwrap();
    assert_eq!(req.supported, vec![ProtocolId::Tarpc]);
}

#[test]
fn request_from_wire_invalid_prefix() {
    let err = ProtocolRequest::from_wire("HELLO: tarpc\n").unwrap_err();
    assert_eq!(err, NegotiationError::InvalidPrefix);
}

#[test]
fn request_from_wire_no_valid_protocols() {
    let err = ProtocolRequest::from_wire("PROTOCOLS: grpc,thrift\n").unwrap_err();
    assert_eq!(err, NegotiationError::NoValidProtocols);
}

#[test]
fn response_to_wire() {
    let resp = ProtocolResponse {
        selected: ProtocolId::Tarpc,
    };
    assert_eq!(resp.to_wire(), "PROTOCOL: tarpc\n");
}

#[test]
fn response_from_wire_valid() {
    let resp = ProtocolResponse::from_wire("PROTOCOL: tarpc\n").unwrap();
    assert_eq!(resp.selected, ProtocolId::Tarpc);
}

#[test]
fn response_from_wire_jsonrpc() {
    let resp = ProtocolResponse::from_wire("PROTOCOL: jsonrpc\n").unwrap();
    assert_eq!(resp.selected, ProtocolId::JsonRpc);
}

#[test]
fn response_from_wire_invalid_prefix() {
    let err = ProtocolResponse::from_wire("STATUS: ok\n").unwrap_err();
    assert_eq!(err, NegotiationError::InvalidPrefix);
}

#[test]
fn response_from_wire_unknown_protocol() {
    let err = ProtocolResponse::from_wire("PROTOCOL: grpc\n").unwrap_err();
    assert_eq!(err, NegotiationError::UnknownProtocol);
}

// ─── select_protocol logic ──────────────────────────────────────────

#[test]
fn select_protocol_client_prefers_tarpc_server_supports_both() {
    let selected = select_protocol(
        &[ProtocolId::Tarpc, ProtocolId::JsonRpc],
        &[ProtocolId::Tarpc, ProtocolId::JsonRpc],
    );
    assert_eq!(selected, ProtocolId::Tarpc);
}

#[test]
fn select_protocol_client_prefers_tarpc_server_only_jsonrpc() {
    let selected = select_protocol(
        &[ProtocolId::Tarpc, ProtocolId::JsonRpc],
        &[ProtocolId::JsonRpc],
    );
    assert_eq!(selected, ProtocolId::JsonRpc);
}

#[test]
fn select_protocol_no_common_falls_back_to_jsonrpc() {
    let selected = select_protocol(&[ProtocolId::Tarpc], &[ProtocolId::JsonRpc]);
    assert_eq!(selected, ProtocolId::JsonRpc);
}

#[test]
fn select_protocol_client_only_jsonrpc() {
    let selected = select_protocol(
        &[ProtocolId::JsonRpc],
        &[ProtocolId::Tarpc, ProtocolId::JsonRpc],
    );
    assert_eq!(selected, ProtocolId::JsonRpc);
}

#[test]
fn select_protocol_empty_client_falls_back() {
    let selected = select_protocol(&[], &[ProtocolId::Tarpc, ProtocolId::JsonRpc]);
    assert_eq!(selected, ProtocolId::JsonRpc);
}

// ─── Async duplex negotiation tests ──────────────────────────────────

#[tokio::test]
async fn negotiate_duplex_tarpc_preferred() {
    let (mut client_end, mut server_end) = tokio::io::duplex(4096);

    let server_supported = ProtocolId::supported();
    let server_task = tokio::spawn(async move {
        negotiate_server(&mut server_end, &server_supported).await
    });

    let client_task = tokio::spawn(async move {
        negotiate_client(&mut client_end, &[ProtocolId::Tarpc, ProtocolId::JsonRpc]).await
    });

    let client_result = client_task.await.unwrap().unwrap();
    assert_eq!(client_result, ProtocolId::Tarpc);

    let server_result = server_task.await.unwrap().unwrap();
    assert_eq!(server_result, NegotiationResult::Negotiated(ProtocolId::Tarpc));
}

#[tokio::test]
async fn negotiate_duplex_jsonrpc_only() {
    let (mut client_end, mut server_end) = tokio::io::duplex(4096);

    let server_supported = vec![ProtocolId::JsonRpc];
    let server_task = tokio::spawn(async move {
        negotiate_server(&mut server_end, &server_supported).await
    });

    let client_task = tokio::spawn(async move {
        negotiate_client(&mut client_end, &[ProtocolId::Tarpc, ProtocolId::JsonRpc]).await
    });

    let client_result = client_task.await.unwrap().unwrap();
    assert_eq!(client_result, ProtocolId::JsonRpc);

    let server_result = server_task.await.unwrap().unwrap();
    assert_eq!(server_result, NegotiationResult::Negotiated(ProtocolId::JsonRpc));
}

#[tokio::test]
async fn negotiate_server_non_protocol_line_returns_buffered() {
    let (mut client_end, mut server_end) = tokio::io::duplex(4096);

    tokio::spawn(async move {
        use tokio::io::AsyncWriteExt;
        client_end
            .write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"pt.health\"}\n")
            .await
            .unwrap();
        client_end.flush().await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(20)).await;

    let result = negotiate_server(&mut server_end, &ProtocolId::supported())
        .await
        .unwrap();

    match result {
        NegotiationResult::NoNegotiation { buffered } => {
            let s = String::from_utf8(buffered).unwrap();
            assert!(s.contains("jsonrpc"));
        }
        NegotiationResult::Negotiated(_) => panic!("expected no negotiation"),
    }
}

#[tokio::test]
async fn negotiate_server_timeout_no_data() {
    let (_client_end, mut server_end) = tokio::io::duplex(4096);

    let result = negotiate_server(&mut server_end, &ProtocolId::supported())
        .await
        .unwrap();

    assert!(matches!(result, NegotiationResult::NoNegotiation { .. }));
}

#[tokio::test]
async fn negotiate_server_eof_returns_no_negotiation() {
    let (client_end, mut server_end) = tokio::io::duplex(4096);
    drop(client_end);

    let result = negotiate_server(&mut server_end, &ProtocolId::supported())
        .await
        .unwrap();

    assert!(matches!(result, NegotiationResult::NoNegotiation { buffered } if buffered.is_empty()));
}

// ─── Error display ──────────────────────────────────────────────────

#[test]
fn negotiation_error_display() {
    assert_eq!(NegotiationError::InvalidPrefix.to_string(), "invalid negotiation prefix");
    assert_eq!(NegotiationError::NoValidProtocols.to_string(), "no valid protocols in request");
    assert_eq!(NegotiationError::UnknownProtocol.to_string(), "unknown protocol in response");
    assert_eq!(NegotiationError::Timeout.to_string(), "negotiation timed out");
    assert_eq!(
        NegotiationError::Io("broken pipe".to_owned()).to_string(),
        "negotiation I/O: broken pipe"
    );
}
