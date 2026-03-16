// SPDX-License-Identifier: AGPL-3.0-or-later
//! Tests for biomeOS integration.

use petal_tongue_discovery::VisualizationDataProvider;

use super::events::BiomeOSEvent;
use super::provider::BiomeOSProvider;
use super::types::{Device, DeviceStatus, DeviceType, Health, NicheTemplate};

#[tokio::test]
async fn test_biomeos_provider_discovery_none() {
    use petal_tongue_core::test_fixtures::env_test_helpers;

    env_test_helpers::with_env_var_removed_async("DEVICE_MANAGEMENT_ENDPOINT", || async {
        let provider = BiomeOSProvider::discover().await.unwrap();
        assert!(
            provider.is_none(),
            "Should return None when no provider found"
        );
    })
    .await;
}

#[tokio::test]
async fn test_biomeos_provider_metadata() {
    let provider = BiomeOSProvider::new_for_test("unix:///tmp/test.sock");

    let metadata = provider.get_metadata();
    assert_eq!(metadata.name, "Device Management Provider");
    assert!(
        metadata
            .capabilities
            .contains(&"device.discovery".to_string())
    );
}

#[tokio::test]
async fn test_biomeos_provider_empty_cache() {
    let provider = BiomeOSProvider::new_for_test("unix:///tmp/nonexistent-petaltongue-test.sock");

    // With no live socket, these should either return empty or a graceful error
    match provider.get_devices().await {
        Ok(devices) => assert!(
            devices.is_empty(),
            "Empty cache should return empty devices"
        ),
        Err(_) => {} // Connection failure is expected without a live socket
    }

    match provider.get_primals_extended().await {
        Ok(primals) => {
            assert!(
                primals.is_empty(),
                "Empty cache should return empty primals"
            );
        }
        Err(_) => {} // Connection failure is expected without a live socket
    }
}

#[test]
fn test_biomeos_event_serialization() {
    let event = BiomeOSEvent::DeviceAdded {
        device: Device {
            id: "dev1".to_string(),
            name: "Test Device".to_string(),
            device_type: DeviceType::GPU,
            status: DeviceStatus::Online,
            resource_usage: 0.5,
            assigned_to: None,
            metadata: serde_json::json!({}),
        },
    };
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("DeviceAdded"));
    assert!(json.contains("dev1"));
}

#[test]
fn test_device_type_variants() {
    assert_ne!(DeviceType::GPU, DeviceType::CPU);
    assert_ne!(DeviceType::Storage, DeviceType::Network);
}

#[test]
fn test_device_status_variants() {
    assert_ne!(DeviceStatus::Online, DeviceStatus::Offline);
    assert_ne!(DeviceStatus::Busy, DeviceStatus::Error);
}

#[test]
fn test_health_variants() {
    assert_ne!(Health::Healthy, Health::Degraded);
    assert_ne!(Health::Degraded, Health::Offline);
}

#[test]
fn test_niche_template_structure() {
    let template = NicheTemplate {
        id: "niche1".to_string(),
        name: "Test Niche".to_string(),
        description: "A test niche".to_string(),
        required_primals: vec!["discovery.service".to_string()],
        optional_primals: vec!["compute.service".to_string()],
        metadata: serde_json::json!({}),
    };
    assert_eq!(template.id, "niche1");
    assert_eq!(template.required_primals.len(), 1);
    assert_eq!(template.optional_primals.len(), 1);
}
