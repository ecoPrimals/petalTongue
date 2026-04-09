// SPDX-License-Identifier: AGPL-3.0-or-later

use super::test_support::{PRIMAL_NAME, default_tarpc_endpoint};
use super::*;
use bytes::Bytes;
use std::collections::HashMap;

#[test]
fn test_primal_endpoint_serialization() {
    let endpoint = PrimalEndpoint {
        primal_id: "test-123".to_string(),
        name: Some("Test Primal".to_string()),
        endpoint: default_tarpc_endpoint(),
        capabilities: vec!["visualization".to_string()],
        primal_type: PRIMAL_NAME.to_string(),
        protocol: "tarpc".to_string(),
        metadata: HashMap::new(),
    };

    let json = serde_json::to_string(&endpoint).expect("serialize");
    let deserialized: PrimalEndpoint = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(endpoint.primal_id, deserialized.primal_id);
    assert_eq!(endpoint.primal_type, deserialized.primal_type);
}

#[test]
fn test_health_status() {
    let health = HealthStatus {
        status: "healthy".to_string(),
        version: "1.2.0".to_string(),
        uptime_seconds: 3600,
        capabilities: vec!["visualization".to_string(), "graph-compute".to_string()],
        details: HashMap::new(),
    };

    assert_eq!(health.status, "healthy");
    assert_eq!(health.capabilities.len(), 2);
}

#[test]
fn test_protocol_info_priority() {
    let tarpc = ProtocolInfo {
        name: "tarpc".to_string(),
        endpoint: default_tarpc_endpoint(),
        enabled: true,
        priority: 1, // PRIMARY
        info: HashMap::new(),
    };

    let jsonrpc = ProtocolInfo {
        name: "jsonrpc".to_string(),
        endpoint: "unix:///tmp/petaltongue.sock".to_string(),
        enabled: true,
        priority: 2, // SECONDARY
        info: HashMap::new(),
    };

    assert!(tarpc.priority < jsonrpc.priority);
    assert_eq!(tarpc.name, "tarpc");
}

#[test]
fn test_render_request() {
    let request = RenderRequest {
        topology: Bytes::from_static(&[1, 2, 3, 4]),
        data: Bytes::new(), // Empty for topology rendering
        width: 1920,
        height: 1080,
        format: "png".to_string(),
        settings: HashMap::new(),
        metadata: None,
    };

    assert_eq!(request.width, 1920);
    assert_eq!(request.format, "png");
}

#[test]
fn test_primal_metrics() {
    let metrics = PrimalMetrics {
        fps: Some(60.0),
        time_since_last_frame: Some(0.016),
        is_hanging: false,
        total_frames: 1000,
        cpu_usage: Some(25.5),
        memory_usage: Some(104_857_600),
        uptime_seconds: 3600,
        custom: HashMap::new(),
    };

    assert_eq!(metrics.fps, Some(60.0));
    assert!(!metrics.is_hanging);
    assert_eq!(metrics.total_frames, 1000);
}

#[test]
fn test_version_info() {
    let version = VersionInfo {
        version: "1.2.0".to_string(),
        tarpc_version: "0.34".to_string(),
        jsonrpc_version: "2.0".to_string(),
        https_version: None, // Not enabled
        capabilities: vec!["visualization".to_string()],
    };

    assert_eq!(version.version, "1.2.0");
    assert!(version.https_version.is_none());
    assert!(!version.capabilities.is_empty());
}

#[test]
fn test_primal_endpoint_serialization_roundtrip() {
    let endpoint = PrimalEndpoint {
        primal_id: "uuid-123".to_string(),
        name: Some(PRIMAL_NAME.to_string()),
        endpoint: default_tarpc_endpoint(),
        capabilities: vec!["visualization".to_string()],
        primal_type: PRIMAL_NAME.to_string(),
        protocol: "tarpc".to_string(),
        metadata: HashMap::new(),
    };
    let json = serde_json::to_string(&endpoint).expect("serialize");
    let restored: PrimalEndpoint = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.primal_id, endpoint.primal_id);
    assert_eq!(restored.name, endpoint.name);
}

#[test]
fn test_health_status_serialization_roundtrip() {
    let mut details = HashMap::new();
    details.insert("cpu".to_string(), "25%".to_string());
    let status = HealthStatus {
        status: "healthy".to_string(),
        version: "1.0.0".to_string(),
        uptime_seconds: 3600,
        capabilities: vec!["visualization".to_string()],
        details,
    };
    let json = serde_json::to_string(&status).expect("serialize");
    let restored: HealthStatus = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.status, "healthy");
    assert_eq!(restored.uptime_seconds, 3600);
}

#[test]
fn test_render_response_serialization() {
    let response = RenderResponse {
        success: true,
        data: Bytes::from_static(b"PNG\x89"),
        width: 800,
        height: 600,
        error: None,
        render_time_ms: 50,
    };
    let json = serde_json::to_value(&response).expect("serialize");
    assert_eq!(json["success"], true);
    assert_eq!(json["width"], 800);
    assert_eq!(json["height"], 600);
}

#[test]
fn test_primal_endpoint_display_and_metadata() {
    let mut metadata = HashMap::new();
    metadata.insert("version".to_string(), "1.0".to_string());
    let endpoint = PrimalEndpoint {
        primal_id: "ep-1".to_string(),
        name: None,
        endpoint: default_tarpc_endpoint(),
        capabilities: vec!["viz".to_string(), "compute".to_string()],
        primal_type: PRIMAL_NAME.to_string(),
        protocol: "tarpc".to_string(),
        metadata: metadata.clone(),
    };
    let json = serde_json::to_string(&endpoint).expect("serialize");
    let restored: PrimalEndpoint = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.metadata.get("version"), Some(&"1.0".to_string()));
}

#[test]
fn test_health_status_with_details() {
    let mut details = HashMap::new();
    details.insert("cpu".to_string(), "25%".to_string());
    details.insert("memory".to_string(), "512MB".to_string());
    let status = HealthStatus {
        status: "degraded".to_string(),
        version: "1.0.0".to_string(),
        uptime_seconds: 7200,
        capabilities: vec![],
        details,
    };
    assert_eq!(status.status, "degraded");
    assert_eq!(status.uptime_seconds, 7200);
}

#[test]
fn test_version_info_with_https() {
    let version = VersionInfo {
        version: "2.0.0".to_string(),
        tarpc_version: "0.34".to_string(),
        jsonrpc_version: "2.0".to_string(),
        https_version: Some("1.0".to_string()),
        capabilities: vec!["visualization".to_string(), "https".to_string()],
    };
    let json = serde_json::to_string(&version).expect("serialize");
    let restored: VersionInfo = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.https_version, Some("1.0".to_string()));
}

#[test]
fn test_protocol_info_with_info_map() {
    let mut info = HashMap::new();
    info.insert("latency_us".to_string(), "15".to_string());
    let proto = ProtocolInfo {
        name: "tarpc".to_string(),
        endpoint: default_tarpc_endpoint(),
        enabled: true,
        priority: 1,
        info,
    };
    let json = serde_json::to_string(&proto).expect("serialize");
    let restored: ProtocolInfo = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.info.get("latency_us"), Some(&"15".to_string()));
}

#[test]
fn test_render_request_with_metadata() {
    let mut metadata = HashMap::new();
    metadata.insert("source".to_string(), "topology".to_string());
    let request = RenderRequest {
        topology: Bytes::new(),
        data: Bytes::from(vec![0u8; 4]),
        width: 100,
        height: 100,
        format: "rgba8".to_string(),
        settings: HashMap::new(),
        metadata: Some(metadata),
    };
    assert_eq!(request.format, "rgba8");
    assert_eq!(request.data.len(), 4);
}

#[test]
fn test_render_response_with_error() {
    let response = RenderResponse {
        success: false,
        data: Bytes::new(),
        width: 0,
        height: 0,
        error: Some("Render failed".to_string()),
        render_time_ms: 0,
    };
    assert!(!response.success);
    assert_eq!(response.error.as_deref(), Some("Render failed"));
}

#[test]
fn test_primal_metrics_hanging() {
    let metrics = PrimalMetrics {
        fps: None,
        time_since_last_frame: Some(6.0),
        is_hanging: true,
        total_frames: 100,
        cpu_usage: None,
        memory_usage: None,
        uptime_seconds: 60,
        custom: HashMap::new(),
    };
    assert!(metrics.is_hanging);
    assert_eq!(metrics.total_frames, 100);
}

#[test]
fn test_primal_metrics_custom() {
    let mut custom = HashMap::new();
    custom.insert("gpu_usage".to_string(), "45%".to_string());
    let metrics = PrimalMetrics {
        fps: Some(60.0),
        time_since_last_frame: None,
        is_hanging: false,
        total_frames: 0,
        cpu_usage: None,
        memory_usage: None,
        uptime_seconds: 0,
        custom,
    };
    assert_eq!(metrics.custom.get("gpu_usage"), Some(&"45%".to_string()));
}

#[test]
fn test_bincode_roundtrip_primal_endpoint() {
    let endpoint = PrimalEndpoint {
        primal_id: "bincode-test".to_string(),
        name: Some("Test".to_string()),
        endpoint: default_tarpc_endpoint(),
        capabilities: vec!["viz".to_string()],
        primal_type: PRIMAL_NAME.to_string(),
        protocol: "tarpc".to_string(),
        metadata: HashMap::new(),
    };
    let encoded = bincode::serialize(&endpoint).expect("bincode serialize");
    let decoded: PrimalEndpoint = bincode::deserialize(&encoded).expect("bincode deserialize");
    assert_eq!(decoded.primal_id, endpoint.primal_id);
}
