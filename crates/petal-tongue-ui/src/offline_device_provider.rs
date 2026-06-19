// SPDX-License-Identifier: AGPL-3.0-or-later
//! Offline Device Provider — graceful degradation when biomeOS is unavailable
//!
//! Provides honest offline/degraded state when the ecosystem connection is lost.
//! With the `offline-demo` feature, last-known-good sample data may be served for
//! development and sandbox scenarios — always clearly labeled as offline/degraded.
//!
//! # Production behavior
//!
//! Without `offline-demo`, this module is not compiled. The UI shows empty panels
//! and an "ecosystem unavailable" indicator rather than fabricated live data.

use super::biomeos_integration::{Device, DeviceStatus, DeviceType, Health, NicheTemplate, Primal};
use petal_tongue_core::{PrimalInfo, TopologyEdge};
use petal_tongue_discovery::{DiscoveryResult, ProviderMetadata, VisualizationDataProvider};
use tracing::{info, warn};

/// Offline device provider — honest degraded state when biomeOS is unreachable
///
/// When sample data is enabled (via `offline-demo` / `test-fixtures`), serves
/// last-known-good reference data for UI development. Metadata and health checks
/// always indicate offline/degraded capability — never a live ecosystem connection.
pub struct OfflineDeviceProvider {
    devices: Vec<Device>,
    primals: Vec<Primal>,
    templates: Vec<NicheTemplate>,
}

impl OfflineDeviceProvider {
    /// Create a new offline device provider
    #[must_use]
    pub fn new() -> Self {
        info!("Creating offline device provider (ecosystem connection unavailable)");

        Self {
            devices: Self::offline_sample_devices(),
            primals: Self::offline_sample_primals(),
            templates: Self::offline_sample_templates(),
        }
    }

    /// Whether this provider is operating in an offline/degraded capacity
    #[must_use]
    pub const fn is_offline(&self) -> bool {
        true
    }

    /// Check if offline showcase mode is requested via environment
    #[must_use]
    pub fn is_offline_demo_requested() -> bool {
        std::env::var("SHOWCASE_MODE")
            .unwrap_or_else(|_| "false".to_owned())
            .to_lowercase()
            == "true"
    }

    /// Get cached device list (offline sample data when `offline-demo` is enabled)
    #[must_use]
    pub fn get_devices(&self) -> Vec<Device> {
        self.devices.clone()
    }

    /// Get cached primal list (offline sample data when `offline-demo` is enabled)
    #[must_use]
    pub fn get_primals_extended(&self) -> Vec<Primal> {
        self.primals.clone()
    }

    /// Get cached niche templates (offline sample data when `offline-demo` is enabled)
    #[must_use]
    pub fn get_niche_templates(&self) -> Vec<NicheTemplate> {
        self.templates.clone()
    }

    /// Offline sample devices for degraded UI development (`offline-demo` only)
    fn offline_sample_devices() -> Vec<Device> {
        vec![
            Device {
                id: "gpu-0".to_owned(),
                name: "NVIDIA RTX 4090 (offline sample)".to_owned(),
                device_type: DeviceType::GPU,
                status: DeviceStatus::Offline,
                resource_usage: 0.0,
                assigned_to: None,
                metadata: serde_json::json!({
                    "source": "offline-sample",
                    "note": "ecosystem unavailable"
                }),
            },
            Device {
                id: "cpu-0".to_owned(),
                name: "AMD Ryzen 9 7950X (offline sample)".to_owned(),
                device_type: DeviceType::CPU,
                status: DeviceStatus::Offline,
                resource_usage: 0.0,
                assigned_to: None,
                metadata: serde_json::json!({
                    "source": "offline-sample"
                }),
            },
        ]
    }

    /// Offline sample primals for degraded UI development (`offline-demo` only)
    fn offline_sample_primals() -> Vec<Primal> {
        vec![
            Primal {
                id: "primal-security".to_owned(),
                name: "Security Primal (offline sample)".to_owned(),
                capabilities: vec!["security".to_owned(), "auth".to_owned()],
                health: Health::Offline,
                load: 0.0,
                assigned_devices: vec![],
                metadata: serde_json::json!({
                    "source": "offline-sample"
                }),
            },
            Primal {
                id: "primal-discovery".to_owned(),
                name: "Discovery Primal (offline sample)".to_owned(),
                capabilities: vec!["discovery".to_owned(), "registry".to_owned()],
                health: Health::Offline,
                load: 0.0,
                assigned_devices: vec![],
                metadata: serde_json::json!({
                    "source": "offline-sample"
                }),
            },
        ]
    }

    /// Offline sample niche templates for degraded UI development (`offline-demo` only)
    fn offline_sample_templates() -> Vec<NicheTemplate> {
        vec![NicheTemplate {
            id: "offline-placeholder".to_owned(),
            name: "Unavailable".to_owned(),
            description: "Ecosystem connection required to load niche templates".to_owned(),
            required_primals: vec![],
            optional_primals: vec![],
            metadata: serde_json::json!({
                "source": "offline-sample",
                "status": "degraded"
            }),
        }]
    }
}

impl Default for OfflineDeviceProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl VisualizationDataProvider for OfflineDeviceProvider {
    async fn get_primals(&self) -> DiscoveryResult<Vec<PrimalInfo>> {
        Ok(self
            .primals
            .iter()
            .map(|p| PrimalInfo {
                id: p.id.clone().into(),
                name: p.name.clone(),
                primal_type: "offline".to_owned(),
                endpoint: format!("offline://{}", p.id),
                capabilities: p.capabilities.clone(),
                health: match p.health {
                    Health::Healthy => petal_tongue_core::PrimalHealthStatus::Warning,
                    Health::Degraded => petal_tongue_core::PrimalHealthStatus::Warning,
                    Health::Offline => petal_tongue_core::PrimalHealthStatus::Critical,
                },
                last_seen: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                endpoints: None,
                metadata: None,
                properties: Default::default(),
            })
            .collect())
    }

    async fn get_topology(&self) -> DiscoveryResult<Vec<TopologyEdge>> {
        Ok(vec![])
    }

    async fn health_check(&self) -> DiscoveryResult<String> {
        Ok("degraded: no ecosystem connection — serving offline sample data".to_owned())
    }

    fn get_metadata(&self) -> ProviderMetadata {
        warn!("Using offline device provider (biomeOS unavailable)");
        ProviderMetadata {
            name: "Offline Device Provider (degraded)".to_owned(),
            endpoint: "offline://unavailable".to_owned(),
            protocol: "offline".to_owned(),
            capabilities: vec![
                "device.discovery (offline)".to_owned(),
                "niche.templates (offline)".to_owned(),
            ],
        }
    }
}

#[cfg(all(test, feature = "offline-demo"))]
mod tests {
    use super::*;

    #[test]
    fn test_offline_provider_devices() {
        let provider = OfflineDeviceProvider::new();
        assert!(provider.is_offline());
        let devices = provider.get_devices();

        assert!(!devices.is_empty(), "Should have offline sample devices");
        assert!(
            devices.iter().all(|d| d.status == DeviceStatus::Offline),
            "Offline sample devices should report offline status"
        );
    }

    #[test]
    fn test_offline_provider_primals() {
        let provider = OfflineDeviceProvider::new();
        let primals = provider.get_primals_extended();

        assert!(!primals.is_empty(), "Should have offline sample primals");
        assert!(
            primals.iter().all(|p| p.health == Health::Offline),
            "Offline sample primals should report offline health"
        );
    }

    #[test]
    fn test_offline_provider_templates() {
        let provider = OfflineDeviceProvider::new();
        let templates = provider.get_niche_templates();

        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].name, "Unavailable");
    }

    #[tokio::test]
    async fn test_offline_provider_as_visualization_provider() {
        let provider = OfflineDeviceProvider::new();

        let primals = provider.get_primals().await.unwrap();
        assert!(!primals.is_empty());

        let topology = provider.get_topology().await.unwrap();
        assert!(
            topology.is_empty(),
            "Offline provider returns empty topology"
        );

        let health = provider.health_check().await.unwrap();
        assert!(health.contains("degraded"));
    }

    #[test]
    fn test_offline_demo_detection() {
        use petal_tongue_core::test_fixtures::env_test_helpers;

        env_test_helpers::with_env_var_removed("SHOWCASE_MODE", || {
            assert!(!OfflineDeviceProvider::is_offline_demo_requested());
        });

        env_test_helpers::with_env_var("SHOWCASE_MODE", "true", || {
            assert!(OfflineDeviceProvider::is_offline_demo_requested());
        });
    }
}
