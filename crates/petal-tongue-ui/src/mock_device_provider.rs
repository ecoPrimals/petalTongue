// SPDX-License-Identifier: AGPL-3.0-only
//! Mock Provider - Testing and Graceful Degradation
//!
//! Provides demo device/primal/niche data for:
//! - Development and testing
//! - Graceful fallback when biomeOS unavailable
//! - Demo/showcase mode
//!
//! # Mocks vs Production
//!
//! **IMPORTANT**: Mocks are ONLY for testing! This provider should never be
//! used in production unless explicitly requested (`SHOWCASE_MODE=true`) or as
//! a graceful fallback when the real provider is unavailable.

use super::biomeos_integration::{Device, DeviceStatus, DeviceType, Health, NicheTemplate, Primal};
use anyhow::Result;
use async_trait::async_trait;
use petal_tongue_core::{PrimalInfo, TopologyEdge};
use petal_tongue_discovery::{ProviderMetadata, VisualizationDataProvider};
use tracing::{info, warn};

/// Mock provider for testing and graceful degradation
///
/// Provides realistic demo data that showcases the UI without requiring
/// actual device management infrastructure.
pub struct MockDeviceProvider {
    devices: Vec<Device>,
    primals: Vec<Primal>,
    templates: Vec<NicheTemplate>,
}

impl MockDeviceProvider {
    /// Create a new mock provider with demo data
    #[must_use]
    pub fn new() -> Self {
        info!("📚 Creating mock device provider (demo data)");

        Self {
            devices: Self::create_demo_devices(),
            primals: Self::create_demo_primals(),
            templates: Self::create_demo_templates(),
        }
    }

    /// Check if mock mode is requested
    #[must_use]
    pub fn is_mock_mode_requested() -> bool {
        std::env::var("SHOWCASE_MODE")
            .unwrap_or_else(|_| "false".to_string())
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
                id: "gpu-0".to_string(),
                name: "NVIDIA RTX 4090".to_string(),
                device_type: DeviceType::GPU,
                status: DeviceStatus::Online,
                resource_usage: 0.45,
                assigned_to: Some("primal-compute".to_string()),
                metadata: serde_json::json!({
                    "vram": "24GB",
                    "cuda_cores": 16384,
                    "power": "450W"
                }),
            },
            Device {
                id: "cpu-0".to_string(),
                name: "AMD Ryzen 9 7950X".to_string(),
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
                id: "ssd-0".to_string(),
                name: "Samsung 990 PRO 2TB".to_string(),
                device_type: DeviceType::Storage,
                status: DeviceStatus::Online,
                resource_usage: 0.67,
                assigned_to: Some("primal-storage".to_string()),
                metadata: serde_json::json!({
                    "capacity": "2TB",
                    "used": "1.34TB",
                    "read_speed": "7450MB/s"
                }),
            },
            Device {
                id: "net-0".to_string(),
                name: "Intel X710 10GbE".to_string(),
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
                id: "mem-0".to_string(),
                name: "DDR5-6000 64GB".to_string(),
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
                id: "gpu-1".to_string(),
                name: "NVIDIA A100".to_string(),
                device_type: DeviceType::GPU,
                status: DeviceStatus::Busy,
                resource_usage: 0.92,
                assigned_to: Some("primal-compute".to_string()),
                metadata: serde_json::json!({
                    "vram": "80GB",
                    "tensor_cores": 6912,
                    "power": "400W"
                }),
            },
            Device {
                id: "ssd-1".to_string(),
                name: "WD Black SN850X 4TB".to_string(),
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
                id: "primal-security".to_string(),
                name: "Security Primal".to_string(),
                capabilities: vec!["security".to_string(), "auth".to_string()],
                health: Health::Healthy,
                load: 0.23,
                assigned_devices: vec![],
                metadata: serde_json::json!({
                    "version": "1.0.0",
                    "uptime": "15d 3h"
                }),
            },
            Primal {
                id: "primal-discovery".to_string(),
                name: "Discovery Primal".to_string(),
                capabilities: vec!["discovery".to_string(), "registry".to_string()],
                health: Health::Healthy,
                load: 0.15,
                assigned_devices: vec![],
                metadata: serde_json::json!({
                    "version": "1.2.0",
                    "uptime": "15d 3h"
                }),
            },
            Primal {
                id: "primal-compute".to_string(),
                name: "Compute Primal".to_string(),
                capabilities: vec!["compute".to_string(), "gpu".to_string()],
                health: Health::Healthy,
                load: 0.68,
                assigned_devices: vec!["gpu-0".to_string(), "gpu-1".to_string()],
                metadata: serde_json::json!({
                    "version": "2.1.0",
                    "uptime": "15d 3h"
                }),
            },
            Primal {
                id: "primal-storage".to_string(),
                name: "Storage Primal".to_string(),
                capabilities: vec!["storage".to_string(), "cache".to_string()],
                health: Health::Healthy,
                load: 0.45,
                assigned_devices: vec!["ssd-0".to_string()],
                metadata: serde_json::json!({
                    "version": "1.5.0",
                    "uptime": "15d 3h"
                }),
            },
            Primal {
                id: "primal-ui".to_string(),
                name: "UI Primal".to_string(),
                capabilities: vec!["ui".to_string(), "render".to_string()],
                health: Health::Healthy,
                load: 0.12,
                assigned_devices: vec![],
                metadata: serde_json::json!({
                    "version": "1.3.0+",
                    "uptime": "0d 1h"
                }),
            },
            Primal {
                id: "primal-ai".to_string(),
                name: "AI Primal".to_string(),
                capabilities: vec!["ai".to_string(), "learning".to_string()],
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
                id: "nest".to_string(),
                name: "Nest".to_string(),
                description: "Complete ecoPrimals ecosystem with all core primals".to_string(),
                required_primals: vec![
                    "security".to_string(),
                    "discovery".to_string(),
                    "compute".to_string(),
                    "storage".to_string(),
                ],
                optional_primals: vec!["ai".to_string(), "ui".to_string()],
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
                id: "tower".to_string(),
                name: "Tower".to_string(),
                description: "High-performance compute tower for GPU-intensive tasks".to_string(),
                required_primals: vec!["compute".to_string(), "storage".to_string()],
                optional_primals: vec!["ai".to_string()],
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
                id: "node".to_string(),
                name: "Node".to_string(),
                description: "Minimal node for edge deployment".to_string(),
                required_primals: vec!["discovery".to_string()],
                optional_primals: vec!["security".to_string()],
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

impl Default for MockDeviceProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Implement `VisualizationDataProvider` for backward compatibility
#[async_trait]
impl VisualizationDataProvider for MockDeviceProvider {
    async fn get_primals(&self) -> Result<Vec<PrimalInfo>> {
        // Convert our Primal to core PrimalInfo
        Ok(self
            .primals
            .iter()
            .map(|p| PrimalInfo {
                id: p.id.clone().into(),
                name: p.name.clone(),
                primal_type: "mock".to_string(),
                endpoint: format!("mock://{}", p.name),
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
                #[expect(deprecated)]
                trust_level: None,
                #[expect(deprecated)]
                family_id: None,
            })
            .collect())
    }

    async fn get_topology(&self) -> Result<Vec<TopologyEdge>> {
        // Mock topology showing some connections
        Ok(vec![
            TopologyEdge {
                from: "primal-discovery".to_string().into(),
                to: "primal-security".to_string().into(),
                edge_type: "discovery".to_string(),
                label: Some("Discovers".to_string()),
                capability: Some("security".to_string()),
                metrics: None,
            },
            TopologyEdge {
                from: "primal-discovery".to_string().into(),
                to: "primal-compute".to_string().into(),
                edge_type: "discovery".to_string(),
                label: Some("Discovers".to_string()),
                capability: Some("compute".to_string()),
                metrics: None,
            },
            TopologyEdge {
                from: "primal-compute".to_string().into(),
                to: "primal-storage".to_string().into(),
                edge_type: "storage".to_string(),
                label: Some("Uses".to_string()),
                capability: Some("storage".to_string()),
                metrics: None,
            },
        ])
    }

    async fn health_check(&self) -> Result<String> {
        Ok("healthy (mock)".to_string())
    }

    fn get_metadata(&self) -> ProviderMetadata {
        warn!("⚠️ Using mock provider - NOT for production!");
        ProviderMetadata {
            name: "Mock Device Provider (DEMO ONLY)".to_string(),
            endpoint: "mock://demo".to_string(),
            protocol: "mock".to_string(),
            capabilities: vec![
                "device.discovery (mock)".to_string(),
                "niche.templates (mock)".to_string(),
            ],
        }
    }
}

#[cfg(all(test, feature = "mock"))]
mod tests {
    use super::*;

    #[test]
    fn test_mock_provider_devices() {
        let provider = MockDeviceProvider::new();
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
    fn test_mock_provider_primals() {
        let provider = MockDeviceProvider::new();
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
    fn test_mock_provider_templates() {
        let provider = MockDeviceProvider::new();
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
    async fn test_mock_provider_as_visualization_provider() {
        let provider = MockDeviceProvider::new();

        let primals = provider.get_primals().await.unwrap();
        assert!(!primals.is_empty(), "Should return primals");

        let topology = provider.get_topology().await.unwrap();
        assert!(!topology.is_empty(), "Should return topology");

        let health = provider.health_check().await.unwrap();
        assert!(health.contains("healthy"), "Should be healthy");
    }

    #[test]
    fn test_mock_mode_detection() {
        use petal_tongue_core::test_fixtures::env_test_helpers;

        env_test_helpers::with_env_var_removed("SHOWCASE_MODE", || {
            assert!(!MockDeviceProvider::is_mock_mode_requested());
        });

        env_test_helpers::with_env_var("SHOWCASE_MODE", "true", || {
            assert!(MockDeviceProvider::is_mock_mode_requested());
        });
    }
}
