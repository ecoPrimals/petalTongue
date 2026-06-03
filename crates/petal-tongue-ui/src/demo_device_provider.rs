// SPDX-License-Identifier: AGPL-3.0-or-later
//! Demo Device Provider - Graceful Fallback
//!
//! Provides demo device/primal/niche data for:
//! - Development and testing
//! - Graceful fallback when biomeOS unavailable
//! - Demo/showcase mode
//!
//! # Fallback vs Mock
//!
//! This is a **fallback** provider, not a mock. It provides degraded but real
//! functionality when biomeOS is unavailable. Use `--features mock` to enable
//! (or when running tests). Production builds without the feature use empty panels.

use super::biomeos_integration::{Device, DeviceStatus, DeviceType, Health, NicheTemplate, Primal};
use petal_tongue_core::{PrimalInfo, TopologyEdge};
use petal_tongue_discovery::{DiscoveryResult, ProviderMetadata, VisualizationDataProvider};
use tracing::{info, warn};

/// Demo provider for graceful fallback when biomeOS unavailable
///
/// Provides realistic demo data that showcases the UI without requiring
/// actual device management infrastructure.
pub struct DemoDeviceProvider {
    devices: Vec<Device>,
    primals: Vec<Primal>,
    templates: Vec<NicheTemplate>,
}

impl DemoDeviceProvider {
    /// Create a new demo provider with showcase data
    #[must_use]
    pub fn new() -> Self {
        info!("📚 Creating demo device provider (fallback data)");

        Self {
            devices: Self::create_demo_devices(),
            primals: Self::create_demo_primals(),
            templates: Self::create_demo_templates(),
        }
    }

    /// Check if demo/showcase mode is requested
    #[must_use]
    pub fn is_demo_mode_requested() -> bool {
        std::env::var("SHOWCASE_MODE")
            .unwrap_or_else(|_| "false".to_owned())
            .to_lowercase()
            == "true"
    }

    /// Get demo devices
    #[must_use]
    pub fn get_devices(&self) -> Vec<Device> {
        self.devices.clone()
    }

    /// Get demo primals
    #[must_use]
    pub fn get_primals_extended(&self) -> Vec<Primal> {
        self.primals.clone()
    }

    /// Get demo niche templates
    #[must_use]
    pub fn get_niche_templates(&self) -> Vec<NicheTemplate> {
        self.templates.clone()
    }

    /// Create demo devices for showcase
    fn create_demo_devices() -> Vec<Device> {
        vec![
            Device {
                id: "gpu-0".to_owned(),
                name: "NVIDIA RTX 4090".to_owned(),
                device_type: DeviceType::GPU,
                status: DeviceStatus::Online,
                resource_usage: 0.45,
                assigned_to: Some("primal-compute".to_owned()),
                metadata: serde_json::json!({
                    "vram": "24GB",
                    "cuda_cores": 16384,
                    "power": "450W"
                }),
            },
            Device {
                id: "cpu-0".to_owned(),
                name: "AMD Ryzen 9 7950X".to_owned(),
                device_type: DeviceType::CPU,
                status: DeviceStatus::Online,
                resource_usage: 0.32,
                assigned_to: None,
                metadata: serde_json::json!({
                    "cores": 16,
                    "threads": 32,
                    "base_clock": "4.5GHz"
                }),
            },
            Device {
                id: "ssd-0".to_owned(),
                name: "Samsung 990 PRO 2TB".to_owned(),
                device_type: DeviceType::Storage,
                status: DeviceStatus::Online,
                resource_usage: 0.67,
                assigned_to: Some("primal-storage".to_owned()),
                metadata: serde_json::json!({
                    "capacity": "2TB",
                    "used": "1.34TB",
                    "read_speed": "7450MB/s"
                }),
            },
            Device {
                id: "net-0".to_owned(),
                name: "Intel X710 10GbE".to_owned(),
                device_type: DeviceType::Network,
                status: DeviceStatus::Online,
                resource_usage: 0.15,
                assigned_to: None,
                metadata: serde_json::json!({
                    "speed": "10Gbps",
                    "rx": "1.2Gbps",
                    "tx": "0.8Gbps"
                }),
            },
            Device {
                id: "mem-0".to_owned(),
                name: "DDR5-6000 64GB".to_owned(),
                device_type: DeviceType::Memory,
                status: DeviceStatus::Online,
                resource_usage: 0.58,
                assigned_to: None,
                metadata: serde_json::json!({
                    "capacity": "64GB",
                    "used": "37.1GB",
                    "speed": "6000MT/s"
                }),
            },
            Device {
                id: "gpu-1".to_owned(),
                name: "NVIDIA A100".to_owned(),
                device_type: DeviceType::GPU,
                status: DeviceStatus::Busy,
                resource_usage: 0.92,
                assigned_to: Some("primal-compute".to_owned()),
                metadata: serde_json::json!({
                    "vram": "80GB",
                    "tensor_cores": 6912,
                    "power": "400W"
                }),
            },
            Device {
                id: "ssd-1".to_owned(),
                name: "WD Black SN850X 4TB".to_owned(),
                device_type: DeviceType::Storage,
                status: DeviceStatus::Offline,
                resource_usage: 0.0,
                assigned_to: None,
                metadata: serde_json::json!({
                    "capacity": "4TB",
                    "status": "maintenance"
                }),
            },
        ]
    }

    /// Create demo primals for showcase (generic names - zero sovereignty violations)
    fn create_demo_primals() -> Vec<Primal> {
        vec![
            Primal {
                id: "primal-security".to_owned(),
                name: "Security Primal".to_owned(),
                capabilities: vec!["security".to_owned(), "auth".to_owned()],
                health: Health::Healthy,
                load: 0.23,
                assigned_devices: vec![],
                metadata: serde_json::json!({
                    "version": "1.0.0",
                    "uptime": "15d 3h"
                }),
            },
            Primal {
                id: "primal-discovery".to_owned(),
                name: "Discovery Primal".to_owned(),
                capabilities: vec!["discovery".to_owned(), "registry".to_owned()],
                health: Health::Healthy,
                load: 0.15,
                assigned_devices: vec![],
                metadata: serde_json::json!({
                    "version": "1.2.0",
                    "uptime": "15d 3h"
                }),
            },
            Primal {
                id: "primal-compute".to_owned(),
                name: "Compute Primal".to_owned(),
                capabilities: vec!["compute".to_owned(), "gpu".to_owned()],
                health: Health::Healthy,
                load: 0.68,
                assigned_devices: vec!["gpu-0".to_owned(), "gpu-1".to_owned()],
                metadata: serde_json::json!({
                    "version": "2.1.0",
                    "uptime": "15d 3h"
                }),
            },
            Primal {
                id: "primal-storage".to_owned(),
                name: "Storage Primal".to_owned(),
                capabilities: vec!["storage".to_owned(), "cache".to_owned()],
                health: Health::Healthy,
                load: 0.45,
                assigned_devices: vec!["ssd-0".to_owned()],
                metadata: serde_json::json!({
                    "version": "1.6.6",
                    "uptime": "15d 3h"
                }),
            },
            Primal {
                id: "primal-ui".to_owned(),
                name: "UI Primal".to_owned(),
                capabilities: vec!["ui".to_owned(), "render".to_owned()],
                health: Health::Healthy,
                load: 0.12,
                assigned_devices: vec![],
                metadata: serde_json::json!({
                    "version": "1.3.0+",
                    "uptime": "0d 1h"
                }),
            },
            Primal {
                id: "primal-ai".to_owned(),
                name: "AI Primal".to_owned(),
                capabilities: vec!["ai".to_owned(), "learning".to_owned()],
                health: Health::Degraded,
                load: 0.87,
                assigned_devices: vec![],
                metadata: serde_json::json!({
                    "version": "0.9.0",
                    "uptime": "2d 5h",
                    "warning": "High load"
                }),
            },
        ]
    }

    /// Create demo niche templates
    fn create_demo_templates() -> Vec<NicheTemplate> {
        vec![
            NicheTemplate {
                id: "nest".to_owned(),
                name: "Nest".to_owned(),
                description: "Complete ecoPrimals ecosystem with all core primals".to_owned(),
                required_primals: vec![
                    "security".to_owned(),
                    "discovery".to_owned(),
                    "compute".to_owned(),
                    "storage".to_owned(),
                ],
                optional_primals: vec!["ai".to_owned(), "ui".to_owned()],
                metadata: serde_json::json!({
                    "icon": "🏠",
                    "complexity": "high",
                    "recommended_devices": {
                        "gpu": 1,
                        "cpu": 2,
                        "storage": 1,
                        "memory": "32GB+"
                    }
                }),
            },
            NicheTemplate {
                id: "tower".to_owned(),
                name: "Tower".to_owned(),
                description: "High-performance compute tower for GPU-intensive tasks".to_owned(),
                required_primals: vec!["compute".to_owned(), "storage".to_owned()],
                optional_primals: vec!["ai".to_owned()],
                metadata: serde_json::json!({
                    "icon": "🗼",
                    "complexity": "medium",
                    "recommended_devices": {
                        "gpu": "2+",
                        "storage": 1
                    }
                }),
            },
            NicheTemplate {
                id: "node".to_owned(),
                name: "Node".to_owned(),
                description: "Minimal node for edge deployment".to_owned(),
                required_primals: vec!["discovery".to_owned()],
                optional_primals: vec!["security".to_owned()],
                metadata: serde_json::json!({
                    "icon": "📡",
                    "complexity": "low",
                    "recommended_devices": {
                        "cpu": 1,
                        "memory": "8GB+"
                    }
                }),
            },
        ]
    }
}

impl Default for DemoDeviceProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Implement `VisualizationDataProvider` for backward compatibility
impl VisualizationDataProvider for DemoDeviceProvider {
    async fn get_primals(&self) -> DiscoveryResult<Vec<PrimalInfo>> {
        Ok(self
            .primals
            .iter()
            .map(|p| PrimalInfo {
                id: p.id.clone().into(),
                name: p.name.clone(),
                primal_type: "demo".to_owned(),
                endpoint: format!("demo://{}", p.name),
                capabilities: p.capabilities.clone(),
                health: match p.health {
                    Health::Healthy => petal_tongue_core::PrimalHealthStatus::Healthy,
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
        Ok(vec![
            TopologyEdge {
                from: "primal-discovery".to_owned().into(),
                to: "primal-security".to_owned().into(),
                edge_type: "discovery".to_owned(),
                label: Some("Discovers".to_owned()),
                capability: Some("security".to_owned()),
                metrics: None,
            },
            TopologyEdge {
                from: "primal-discovery".to_owned().into(),
                to: "primal-compute".to_owned().into(),
                edge_type: "discovery".to_owned(),
                label: Some("Discovers".to_owned()),
                capability: Some("compute".to_owned()),
                metrics: None,
            },
            TopologyEdge {
                from: "primal-compute".to_owned().into(),
                to: "primal-storage".to_owned().into(),
                edge_type: "storage".to_owned(),
                label: Some("Uses".to_owned()),
                capability: Some("storage".to_owned()),
                metrics: None,
            },
        ])
    }

    async fn health_check(&self) -> DiscoveryResult<String> {
        Ok("healthy (demo fallback)".to_owned())
    }

    fn get_metadata(&self) -> ProviderMetadata {
        warn!("⚠️ Using demo device provider (biomeOS unavailable)");
        ProviderMetadata {
            name: "Demo Device Provider (Fallback)".to_owned(),
            endpoint: "demo://fallback".to_owned(),
            protocol: "demo".to_owned(),
            capabilities: vec![
                "device.discovery (demo)".to_owned(),
                "niche.templates (demo)".to_owned(),
            ],
        }
    }
}

#[cfg(all(test, feature = "mock"))]
mod tests {
    use super::*;

    #[test]
    fn test_demo_provider_devices() {
        let provider = DemoDeviceProvider::new();
        let devices = provider.get_devices();

        assert!(!devices.is_empty(), "Should have demo devices");
        assert!(
            devices.iter().any(|d| d.device_type == DeviceType::GPU),
            "Should have GPU devices"
        );
        assert!(
            devices.iter().any(|d| d.status == DeviceStatus::Online),
            "Should have online devices"
        );
    }

    #[test]
    fn test_demo_provider_primals() {
        let provider = DemoDeviceProvider::new();
        let primals = provider.get_primals_extended();

        assert!(!primals.is_empty(), "Should have demo primals");
        assert!(
            primals.iter().any(|p| p.id == "primal-compute"),
            "Should have compute primal"
        );
        assert!(
            primals.iter().any(|p| p.health == Health::Healthy),
            "Should have healthy primals"
        );
    }

    #[test]
    fn test_demo_provider_templates() {
        let provider = DemoDeviceProvider::new();
        let templates = provider.get_niche_templates();

        assert_eq!(templates.len(), 3, "Should have 3 templates");
        assert!(
            templates.iter().any(|t| t.name == "Nest"),
            "Should have Nest template"
        );
        assert!(
            templates.iter().any(|t| t.name == "Tower"),
            "Should have Tower template"
        );
        assert!(
            templates.iter().any(|t| t.name == "Node"),
            "Should have Node template"
        );
    }

    #[tokio::test]
    async fn test_demo_provider_as_visualization_provider() {
        let provider = DemoDeviceProvider::new();

        let primals = provider.get_primals().await.unwrap();
        assert!(!primals.is_empty(), "Should return primals");

        let topology = provider.get_topology().await.unwrap();
        assert!(!topology.is_empty(), "Should return topology");

        let health = provider.health_check().await.unwrap();
        assert!(health.contains("healthy"), "Should be healthy");
    }

    #[test]
    fn test_demo_mode_detection() {
        use petal_tongue_core::test_fixtures::env_test_helpers;

        env_test_helpers::with_env_var_removed("SHOWCASE_MODE", || {
            assert!(!DemoDeviceProvider::is_demo_mode_requested());
        });

        env_test_helpers::with_env_var("SHOWCASE_MODE", "true", || {
            assert!(DemoDeviceProvider::is_demo_mode_requested());
        });
    }
}
