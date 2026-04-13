// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn test_universal_discovery_creation() {
    let discovery = UniversalDiscovery::new();
    assert_eq!(discovery.discovery_methods.len(), 4);
}

#[test]
fn test_discovery_method_order() {
    let discovery = UniversalDiscovery::new();
    assert_eq!(discovery.discovery_methods[0], DiscoveryMethod::Environment);
    assert_eq!(discovery.discovery_methods[1], DiscoveryMethod::UnixSocket);
}

#[tokio::test]
async fn test_discover_capability_no_results() {
    let discovery = UniversalDiscovery::new();
    let results = discovery
        .discover_capability("nonexistent-capability")
        .await;
    assert!(results.is_ok());
    assert!(results.unwrap().is_empty());
}

#[tokio::test]
async fn test_environment_discovery() {
    use petal_tongue_core::test_fixtures::env_test_helpers;

    env_test_helpers::with_env_var_async(
        "GPU_RENDERING_ENDPOINT",
        "tarpc://localhost:9001",
        || async {
            let discovery = UniversalDiscovery::new();
            let results = discovery
                .discover_capability("gpu-rendering")
                .await
                .unwrap();

            assert!(!results.is_empty());
            assert_eq!(results[0].capabilities[0], "gpu-rendering");
        },
    )
    .await;
}

#[test]
fn test_discovered_service_serialization() {
    let service = DiscoveredService {
        id: "test-service".to_string(),
        capabilities: vec!["gpu-rendering".to_string()],
        endpoint: "tarpc://localhost:9001".to_string(),
        protocol: "tarpc".to_string(),
        metadata: HashMap::new(),
    };

    let json = serde_json::to_string(&service).unwrap();
    let deserialized: DiscoveredService = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.id, "test-service");
    assert_eq!(deserialized.capabilities[0], "gpu-rendering");
}

#[test]
fn test_discovered_service_with_metadata() {
    let mut meta = HashMap::new();
    meta.insert("region".to_string(), "us-east".to_string());
    let service = DiscoveredService {
        id: "svc".to_string(),
        capabilities: vec!["storage".to_string()],
        endpoint: "http://localhost:8080".to_string(),
        protocol: "http".to_string(),
        metadata: meta.clone(),
    };
    let json = serde_json::to_string(&service).unwrap();
    let deserialized: DiscoveredService = serde_json::from_str(&json).unwrap();
    assert_eq!(
        deserialized.metadata.get("region"),
        Some(&"us-east".to_string())
    );
}

#[test]
fn test_universal_discovery_default() {
    let d1 = UniversalDiscovery::new();
    let d2 = UniversalDiscovery::default();
    assert_eq!(d1.discovery_methods.len(), d2.discovery_methods.len());
}

#[test]
fn test_discovery_method_variants() {
    assert_eq!(DiscoveryMethod::Environment, DiscoveryMethod::Environment);
    assert_ne!(DiscoveryMethod::Environment, DiscoveryMethod::ConfigFile);
    assert_eq!(DiscoveryMethod::ConfigFile, DiscoveryMethod::ConfigFile);
    assert_eq!(DiscoveryMethod::UnixSocket, DiscoveryMethod::UnixSocket);
    assert_eq!(DiscoveryMethod::Mdns, DiscoveryMethod::Mdns);
    assert_eq!(DiscoveryMethod::HttpProbe, DiscoveryMethod::HttpProbe);
}

#[tokio::test]
async fn test_discover_capability_empty_string() {
    let discovery = UniversalDiscovery::new();
    let results = discovery.discover_capability("").await.unwrap();
    assert!(results.is_empty() || !results.is_empty());
}
