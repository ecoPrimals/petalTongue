// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for biomeOS topology format compatibility
//!
//! Verifies that petalTongue can correctly parse and handle topology data
//! in the biomeOS format (Unix sockets, metadata, connection metrics).

use petal_tongue_core::{PrimalInfo, TopologyEdge, property::PropertyValue};

#[test]
fn test_parse_biomeos_primal_format() {
    // biomeOS format with Unix socket and metadata
    let json = r#"{
        "id": "beardog-node-alpha",
        "name": "BearDog Alpha",
        "type": "beardog",
        "endpoint": "",
        "capabilities": ["security", "encryption", "identity"],
        "health": "Healthy",
        "last_seen": 1704758400,
        "endpoints": {
            "unix_socket": "/tmp/beardog-node-alpha.sock",
            "http": null
        },
        "metadata": {
            "version": "v0.15.2",
            "family_id": "nat0",
            "node_id": "node-alpha"
        }
    }"#;

    let primal: PrimalInfo = serde_json::from_str(json).expect("Failed to parse biomeOS format");

    assert_eq!(primal.id.as_str(), "beardog-node-alpha");
    assert_eq!(primal.name, "BearDog Alpha");
    assert_eq!(primal.primal_type, "beardog");
    assert_eq!(primal.capabilities.len(), 3);
    assert!(primal.capabilities.contains(&"security".to_string()));

    // Check endpoints
    assert!(primal.endpoints.is_some());
    let endpoints = primal.endpoints.unwrap();
    assert_eq!(
        endpoints.unix_socket,
        Some("/tmp/beardog-node-alpha.sock".to_string())
    );
    assert_eq!(endpoints.http, None);

    // Check metadata
    assert!(primal.metadata.is_some());
    let metadata = primal.metadata.unwrap();
    assert_eq!(metadata.version, Some("v0.15.2".to_string()));
    assert_eq!(metadata.family_id, Some("nat0".to_string()));
    assert_eq!(metadata.node_id, Some("node-alpha".to_string()));
}

#[test]
fn test_parse_biomeos_connection_format() {
    // biomeOS connection format with metrics
    let json = r#"{
        "from": "songbird-node-alpha",
        "to": "beardog-node-alpha",
        "type": "capability_invocation",
        "capability": "encryption",
        "metrics": {
            "request_count": 42,
            "avg_latency_ms": 2.3
        }
    }"#;

    let edge: TopologyEdge =
        serde_json::from_str(json).expect("Failed to parse biomeOS connection format");

    assert_eq!(edge.from.as_str(), "songbird-node-alpha");
    assert_eq!(edge.to.as_str(), "beardog-node-alpha");
    assert_eq!(edge.edge_type, "capability_invocation");
    assert_eq!(edge.capability, Some("encryption".to_string()));

    // Check metrics
    assert!(edge.metrics.is_some());
    let metrics = edge.metrics.unwrap();
    assert_eq!(metrics.request_count, Some(42));
    assert_eq!(metrics.avg_latency_ms, Some(2.3));
}

#[test]
fn test_parse_biomeos_primal_list() {
    // biomeOS returns a list of primals (not TopologyGraph format)
    let json = r#"[
            {
                "id": "beardog-node-alpha",
                "name": "BearDog",
                "type": "beardog",
                "endpoint": "",
                "capabilities": ["security", "encryption"],
                "health": "Healthy",
                "last_seen": 1704758400,
                "endpoints": {
                    "unix_socket": "/tmp/beardog-node-alpha.sock"
                },
                "metadata": {
                    "version": "v0.15.2",
                    "family_id": "nat0"
                }
            },
            {
                "id": "songbird-node-alpha",
                "name": "Songbird",
                "type": "songbird",
                "endpoint": "",
                "capabilities": ["discovery", "p2p"],
                "health": "Healthy",
                "last_seen": 1704758400,
                "endpoints": {
                    "unix_socket": "/tmp/songbird-node-alpha.sock"
                },
                "metadata": {
                    "version": "v3.19.0"
                }
            }
        ]"#;

    let primals: Vec<PrimalInfo> =
        serde_json::from_str(json).expect("Failed to parse biomeOS primal list");

    assert_eq!(primals.len(), 2);

    // Verify first primal
    let beardog = &primals[0];
    assert_eq!(beardog.id, "beardog-node-alpha");
    assert!(beardog.endpoints.is_some());

    // Verify second primal
    let songbird = &primals[1];
    assert_eq!(songbird.id, "songbird-node-alpha");
    assert!(songbird.endpoints.is_some());
}

#[test]
fn test_migrate_biomeos_metadata_to_properties() {
    let json = r#"{
        "id": "beardog-node-alpha",
        "name": "BearDog",
        "type": "beardog",
        "endpoint": "",
        "capabilities": ["security"],
        "health": "Healthy",
        "last_seen": 1704758400,
        "metadata": {
            "version": "v0.15.2",
            "family_id": "nat0",
            "node_id": "node-alpha"
        }
    }"#;

    let mut primal: PrimalInfo = serde_json::from_str(json).unwrap();

    // `PrimalInfoWire` deserialization already copies metadata into properties
    assert!(primal.properties.contains_key("version"));
    assert!(primal.properties.contains_key("family_id"));
    assert!(primal.properties.contains_key("node_id"));

    // `migrate_metadata_to_properties` is idempotent for the serde path
    primal.migrate_metadata_to_properties();

    // Properties still contain metadata
    assert!(primal.properties.contains_key("version"));
    assert!(primal.properties.contains_key("family_id"));
    assert!(primal.properties.contains_key("node_id"));

    assert_eq!(
        primal.properties.get("version"),
        Some(&PropertyValue::String("v0.15.2".to_string()))
    );
    assert_eq!(
        primal.properties.get("family_id"),
        Some(&PropertyValue::String("nat0".to_string()))
    );
}

#[test]
fn test_endpoint_migration_from_unix_socket() {
    let json = r#"{
        "id": "beardog-node-alpha",
        "name": "BearDog",
        "type": "beardog",
        "endpoint": "",
        "capabilities": ["security"],
        "health": "Healthy",
        "last_seen": 1704758400,
        "endpoints": {
            "unix_socket": "/tmp/beardog-node-alpha.sock"
        }
    }"#;

    let mut primal: PrimalInfo = serde_json::from_str(json).unwrap();

    // `PrimalInfoWire` deserialization already fills endpoint from `unix_socket` when empty
    assert_eq!(primal.endpoint, "unix:///tmp/beardog-node-alpha.sock");

    primal.migrate_metadata_to_properties();

    assert_eq!(primal.endpoint, "unix:///tmp/beardog-node-alpha.sock");
}

#[test]
fn test_endpoint_migration_prefers_unix_socket() {
    let json = r#"{
        "id": "beardog-node-alpha",
        "name": "BearDog",
        "type": "beardog",
        "endpoint": "",
        "capabilities": ["security"],
        "health": "Healthy",
        "last_seen": 1704758400,
        "endpoints": {
            "unix_socket": "/tmp/beardog-node-alpha.sock",
            "http": "http://localhost:8080"
        }
    }"#;

    let mut primal: PrimalInfo = serde_json::from_str(json).unwrap();
    primal.migrate_metadata_to_properties();

    // Should prefer Unix socket over HTTP for local primals (serde + migrate agree)
    assert_eq!(primal.endpoint, "unix:///tmp/beardog-node-alpha.sock");
}

#[test]
fn test_backward_compatibility_with_old_format() {
    // Old petalTongue format (no endpoints or metadata)
    let json = r#"{
        "id": "legacy-primal",
        "name": "Legacy Primal",
        "primal_type": "compute",
        "endpoint": "http://localhost:8080",
        "capabilities": ["compute"],
        "health": "Healthy",
        "last_seen": 1704758400
    }"#;

    let primal: PrimalInfo = serde_json::from_str(json).expect("Failed to parse legacy format");

    assert_eq!(primal.id, "legacy-primal");
    assert_eq!(primal.endpoint, "http://localhost:8080");
    assert!(primal.endpoints.is_none());
    assert!(primal.metadata.is_none());
}

#[test]
fn test_connection_metrics_optional() {
    // Connection without metrics (should still parse)
    let json = r#"{
        "from": "primal-a",
        "to": "primal-b",
        "type": "connection"
    }"#;

    let edge: TopologyEdge =
        serde_json::from_str(json).expect("Failed to parse minimal connection");

    assert_eq!(edge.from.as_str(), "primal-a");
    assert_eq!(edge.to.as_str(), "primal-b");
    assert_eq!(edge.edge_type, "connection");
    assert!(edge.metrics.is_none());
    assert!(edge.capability.is_none());
}
