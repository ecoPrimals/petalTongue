// SPDX-License-Identifier: AGPL-3.0-or-later
//! Unit tests for mDNS provider and packet parsing.

use super::*;
use crate::traits::ProviderMetadata;
use std::net::SocketAddr;

#[test]
fn test_build_mdns_query() {
    let query = build_mdns_query(SERVICE_NAME);

    assert!(query.len() > 12, "Query packet too short");
    assert_eq!(&query[0..2], &[0x00, 0x00], "Transaction ID should be 0");
    assert_eq!(&query[4..6], &[0x00, 0x01], "Should have 1 question");
}

#[test]
fn test_build_mdns_query_different_service() {
    let query = build_mdns_query("_http._tcp.local");
    assert!(query.len() > 12);
    assert_eq!(&query[4..6], &[0x00, 0x01]);
    assert!(query.len() > 20);
}

#[test]
fn test_build_mdns_query_question_format() {
    let query = build_mdns_query("a.b");
    assert!(query.len() >= 12 + 4 + 4);
    assert_eq!(query[12], 1);
    assert_eq!(query[13], b'a');
    assert_eq!(query[14], 1);
    assert_eq!(query[15], b'b');
}

#[test]
fn test_parse_mdns_response_empty() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 5353));
    let result = parse_mdns_response(&[], addr);
    if let Err(e) = result {
        let msg = e.to_string();
        assert!(
            msg.contains("short") || msg.contains("header"),
            "got: {msg}"
        );
    } else {
        panic!("expected parse to fail for empty input");
    }
}

#[test]
fn test_parse_mdns_response_too_short() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 5353));
    let data = [0u8; 11];
    let result = parse_mdns_response(&data, addr);
    assert!(result.is_err());
}

#[test]
fn test_parse_mdns_response_not_response() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 5353));
    let mut data = vec![0x00, 0x00, 0x00, 0x00];
    data.extend_from_slice(&[0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    let result = parse_mdns_response(&data, addr);
    if let Err(e) = result {
        assert!(e.to_string().contains("Not a DNS response"));
    } else {
        panic!("expected parse to fail for non-response");
    }
}

#[test]
fn test_parse_mdns_response_no_srv_record() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let mut data = Vec::new();
    data.extend_from_slice(&[0x00, 0x00]);
    data.extend_from_slice(&[0x84, 0x00]);
    data.extend_from_slice(&[0x00, 0x01]);
    data.extend_from_slice(&[0x00, 0x01]);
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    data.extend_from_slice(&[22]);
    data.extend_from_slice(b"_visualization-provider");
    data.extend_from_slice(&[4]);
    data.extend_from_slice(b"_tcp");
    data.extend_from_slice(&[5]);
    data.extend_from_slice(b"local");
    data.extend_from_slice(&[0]);
    data.extend_from_slice(&[0x00, 0x0C]);
    data.extend_from_slice(&[0x00, 0x01]);
    data.extend_from_slice(&[22]);
    data.extend_from_slice(b"_visualization-provider");
    data.extend_from_slice(&[4]);
    data.extend_from_slice(b"_tcp");
    data.extend_from_slice(&[5]);
    data.extend_from_slice(b"local");
    data.extend_from_slice(&[0]);
    data.extend_from_slice(&[0x00, 0x0C, 0x00, 0x01]);
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x3C]);
    data.extend_from_slice(&[0x00, 0x02]);
    data.extend_from_slice(&[0xC0, 0x0C]);

    let result = parse_mdns_response(&data, addr);
    assert!(
        result.is_err(),
        "Parse should fail when response has no SRV record with port"
    );
}

#[test]
fn test_mdns_provider_new() {
    let metadata = ProviderMetadata {
        name: "Test Provider".to_string(),
        endpoint: "http://192.0.2.1:8080".to_string(),
        protocol: "http".to_string(),
        capabilities: vec!["viz".to_string()],
    };
    let provider = MdnsVisualizationProvider::new("http://192.0.2.1:8080".to_string(), metadata);
    assert_eq!(provider.get_metadata().name, "Test Provider");
    assert_eq!(provider.get_metadata().endpoint, "http://192.0.2.1:8080");
}

#[test]
fn test_multicast_constants() {
    assert_eq!(MDNS_MULTICAST_ADDR, Ipv4Addr::new(224, 0, 0, 251));
    assert_eq!(MDNS_PORT, 5353);
    assert_eq!(SERVICE_NAME, "_visualization-provider._tcp.local");
}

#[tokio::test]
async fn test_discover_timeout() {
    let result = MdnsVisualizationProvider::discover().await;
    assert!(result.is_ok());
    let providers = result.unwrap();
    tracing::info!("Discovery found {} providers", providers.len());
}

#[test]
fn test_build_mdns_query_ptr_type_class() {
    let query = build_mdns_query("_svc._tcp.local");
    let q_start = 12;
    let name_len = "_svc._tcp.local"
        .split('.')
        .map(|l| 1 + l.len())
        .sum::<usize>()
        + 1;
    let type_offset = q_start + name_len;
    assert_eq!(&query[type_offset..type_offset + 2], &[0x00, 0x0C]);
    assert_eq!(&query[type_offset + 2..type_offset + 4], &[0x00, 0x01]);
}

#[test]
fn test_build_mdns_query_long_label() {
    let query = build_mdns_query("a.b.c.d.e.local");
    assert!(query.len() > 20);
    assert_eq!(query[12], 1);
    assert_eq!(query[13], b'a');
}

#[test]
fn test_mdns_provider_get_metadata() {
    let metadata = ProviderMetadata {
        name: "mDNS Test".to_string(),
        endpoint: "http://10.0.0.1:9000".to_string(),
        protocol: "http".to_string(),
        capabilities: vec!["viz".to_string()],
    };
    let provider =
        MdnsVisualizationProvider::new("http://10.0.0.1:9000".to_string(), metadata.clone());
    assert_eq!(provider.get_metadata().name, metadata.name);
    assert_eq!(provider.get_metadata().capabilities, metadata.capabilities);
}

#[test]
fn test_build_mdns_query_empty_service_name() {
    let query = build_mdns_query("");
    assert!(query.len() >= 12);
    assert_eq!(&query[4..6], &[0x00, 0x01]);
}

#[tokio::test]
async fn test_mdns_provider_get_primals_via_http() {
    let mock_server = wiremock::MockServer::start().await;
    let primals_json = serde_json::json!({
        "primals": [{
            "id": "mdns-primal",
            "name": "mDNS Primal",
            "primal_type": "test",
            "endpoint": "http://test:8080",
            "capabilities": ["viz"],
            "health": "Healthy",
            "last_seen": 12345
        }]
    });
    wiremock::Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/api/v1/primals/discovered"))
        .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(&primals_json))
        .mount(&mock_server)
        .await;

    let uri = mock_server.uri();
    let metadata = ProviderMetadata {
        name: "mDNS Test".to_string(),
        endpoint: uri.clone(),
        protocol: "http".to_string(),
        capabilities: vec![],
    };
    let provider = MdnsVisualizationProvider::new(uri, metadata);
    let primals = provider.get_primals().await.unwrap();
    assert_eq!(primals.len(), 1);
    assert_eq!(primals[0].id, "mdns-primal");
}

#[tokio::test]
async fn test_mdns_provider_get_topology_via_http() {
    let mock_server = wiremock::MockServer::start().await;
    let topology_json = serde_json::json!({
        "edges": [{
            "from": "a",
            "to": "b",
            "edge_type": "peer",
            "label": null,
            "capability": null,
            "metrics": null
        }]
    });
    wiremock::Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/api/v1/topology"))
        .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(&topology_json))
        .mount(&mock_server)
        .await;

    let uri = mock_server.uri();
    let metadata = ProviderMetadata {
        name: "mDNS Topology".to_string(),
        endpoint: uri.clone(),
        protocol: "http".to_string(),
        capabilities: vec![],
    };
    let provider = MdnsVisualizationProvider::new(uri, metadata);
    let edges = provider.get_topology().await.unwrap();
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].from, "a");
    assert_eq!(edges[0].to, "b");
}

#[tokio::test]
async fn test_mdns_provider_health_check_success() {
    let mock_server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/api/v1/health"))
        .respond_with(wiremock::ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let uri = mock_server.uri();
    let metadata = ProviderMetadata {
        name: "mDNS Health".to_string(),
        endpoint: uri.clone(),
        protocol: "http".to_string(),
        capabilities: vec![],
    };
    let provider = MdnsVisualizationProvider::new(uri, metadata);
    let health = provider.health_check().await.unwrap();
    assert!(health.contains("healthy"));
}

#[tokio::test]
async fn test_mdns_provider_http_error_status() {
    let mock_server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/api/v1/primals/discovered"))
        .respond_with(wiremock::ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    let uri = mock_server.uri();
    let metadata = ProviderMetadata {
        name: "mDNS Error".to_string(),
        endpoint: uri.clone(),
        protocol: "http".to_string(),
        capabilities: vec![],
    };
    let provider = MdnsVisualizationProvider::new(uri, metadata);
    let result = provider.get_primals().await;
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("500") || e.to_string().contains("status"));
    }
}

#[test]
fn test_build_mdns_query_single_label() {
    let query = build_mdns_query("local");
    assert!(query.len() > 12);
    assert_eq!(query[12], 5);
    assert_eq!(&query[13..18], b"local");
}

#[test]
fn test_build_mdns_query_dns_header_structure() {
    let query = build_mdns_query(SERVICE_NAME);
    assert!(query.len() >= 12 + 5);
    assert_eq!(&query[2..4], &[0x00, 0x00]);
    assert_eq!(&query[6..8], &[0x00, 0x00]);
    assert_eq!(&query[8..10], &[0x00, 0x00]);
    assert_eq!(&query[10..12], &[0x00, 0x00]);
}

#[test]
fn test_parse_mdns_response_ipv6_fallback() {
    let addr = SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 1], 5353));
    let data = [0u8; 12];
    let result = parse_mdns_response(&data, addr);
    assert!(result.is_err());
}

#[test]
fn test_mdns_provider_default_capabilities() {
    let metadata = ProviderMetadata {
        name: "Test".to_string(),
        endpoint: "http://127.0.0.1:8080".to_string(),
        protocol: "http".to_string(),
        capabilities: vec!["custom.cap".to_string()],
    };
    let provider = MdnsVisualizationProvider::new("http://127.0.0.1:8080".to_string(), metadata);
    let meta = provider.get_metadata();
    assert!(meta.capabilities.contains(&"custom.cap".to_string()));
}

#[test]
fn test_build_mdns_query_question_section_min_length() {
    let query = build_mdns_query("x");
    assert!(query.len() >= 12 + 1 + 1 + 4);
}

#[test]
fn test_parse_mdns_response_header_parse() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 5353));
    let mut data = vec![0u8; 12];
    data[2] = 0x80;
    data[3] = 0x00;
    data[4] = 0x00;
    data[5] = 0x01;
    data[6] = 0x00;
    data[7] = 0x00;
    let result = parse_mdns_response(&data, addr);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_mdns_provider_health_check_failure() {
    let mock_server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/api/v1/health"))
        .respond_with(wiremock::ResponseTemplate::new(503))
        .mount(&mock_server)
        .await;

    let uri = mock_server.uri();
    let metadata = ProviderMetadata {
        name: "mDNS Unhealthy".to_string(),
        endpoint: uri.clone(),
        protocol: "http".to_string(),
        capabilities: vec![],
    };
    let provider = MdnsVisualizationProvider::new(uri, metadata);
    let result = provider.health_check().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_mdns_provider_get_topology_empty() {
    let mock_server = wiremock::MockServer::start().await;
    let topology_json = serde_json::json!({ "edges": [] });
    wiremock::Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/api/v1/topology"))
        .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(&topology_json))
        .mount(&mock_server)
        .await;

    let uri = mock_server.uri();
    let metadata = ProviderMetadata {
        name: "mDNS Empty".to_string(),
        endpoint: uri.clone(),
        protocol: "http".to_string(),
        capabilities: vec![],
    };
    let provider = MdnsVisualizationProvider::new(uri, metadata);
    let edges = provider.get_topology().await.unwrap();
    assert!(edges.is_empty());
}

#[test]
fn test_build_mdns_query_ptr_type() {
    let query = build_mdns_query("_test._tcp.local");
    let q_start = 12;
    let name_len: usize = "_test._tcp.local"
        .split('.')
        .map(|l| 1 + l.len())
        .sum::<usize>()
        + 1;
    let type_offset = q_start + name_len;
    assert_eq!(&query[type_offset..type_offset + 2], &[0x00, 0x0C]);
}
