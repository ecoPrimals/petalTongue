//! biomeOS Integration - Visualization Data Provider
//!
//! Provides capability-based discovery and integration with biomeOS for device
//! and niche management UI.
//!
//! # TRUE PRIMAL Principles
//!
//! - **Zero Hardcoding**: Discovers biomeOS by capability, not by name
//! - **Graceful Degradation**: Falls back to mock data when biomeOS unavailable
//! - **Self-Knowledge**: Announces own capabilities to ecosystem
//! - **Runtime Discovery**: No compile-time dependencies on biomeOS
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │ petalTongue                                                 │
//! │  ├─ DevicePanel ────────────┐                              │
//! │  ├─ PrimalPanel ────────────┼─→ BiomeOSProvider           │
//! │  └─ NicheDesigner ──────────┘      │                       │
//! │                                     ↓                       │
//! │                              [Event Stream]                 │
//! │                                     ↓                       │
//! │                              UIEventHandler                 │
//! └─────────────────────────────────────────────────────────────┘
//!                                      ↓
//!                          (Unix Socket / WebSocket)
//!                                      ↓
//! ┌─────────────────────────────────────────────────────────────┐
//! │ biomeOS (Capability: "device.management")                   │
//! │  ├─ Device Discovery                                        │
//! │  ├─ Primal Registry                                         │
//! │  ├─ Niche Orchestration                                     │
//! │  └─ AI Suggestions                                          │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use anyhow::{Context, Result};
use async_trait::async_trait;
use petal_tongue_core::{PrimalInfo, TopologyEdge};
use petal_tongue_discovery::{ProviderMetadata, VisualizationDataProvider};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// biomeOS provider - discovers and integrates with biomeOS for device management
///
/// # Discovery
///
/// Discovers biomeOS by querying for the "device.management" capability.
/// Does NOT hardcode "biomeOS" - any primal with this capability will work!
pub struct BiomeOSProvider {
    /// Connection to biomeOS (or any primal with device.management capability)
    endpoint: String,
    /// Cached data for offline mode
    cache: Arc<RwLock<ProviderCache>>,
    /// Event stream subscription (for real-time updates)
    event_stream: Arc<RwLock<Option<EventStream>>>,
}

/// Cached data for graceful degradation
#[derive(Debug, Clone, Default)]
struct ProviderCache {
    devices: Vec<Device>,
    primals: Vec<Primal>,
    niche_templates: Vec<NicheTemplate>,
    last_update: Option<std::time::Instant>,
}

/// Event stream for real-time updates
struct EventStream {
    // TODO: Implement WebSocket connection
    // For Phase 1, we'll use a placeholder
}

/// Device representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub device_type: DeviceType,
    pub status: DeviceStatus,
    pub resource_usage: f64,         // 0.0-1.0
    pub assigned_to: Option<String>, // Primal ID if assigned
    pub metadata: serde_json::Value,
}

/// Device type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceType {
    GPU,
    CPU,
    Storage,
    Network,
    Memory,
    Other,
}

/// Device status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceStatus {
    Online,
    Offline,
    Busy,
    Error,
}

/// Primal representation (extended from PrimalInfo)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Primal {
    pub id: String,
    pub name: String,
    pub capabilities: Vec<String>,
    pub health: Health,
    pub load: f64,                     // 0.0-1.0
    pub assigned_devices: Vec<String>, // Device IDs
    pub metadata: serde_json::Value,
}

/// Primal health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Health {
    Healthy,
    Degraded,
    Offline,
}

/// Niche template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NicheTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub required_primals: Vec<String>, // Capability strings
    pub optional_primals: Vec<String>,
    pub metadata: serde_json::Value,
}

impl BiomeOSProvider {
    /// Discover biomeOS by capability (TRUE PRIMAL!)
    ///
    /// Queries all discovered primals for "device.management" capability.
    /// Returns `None` if no provider found (graceful degradation).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use petal_tongue_ui::biomeos_integration::BiomeOSProvider;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// if let Some(provider) = BiomeOSProvider::discover().await? {
    ///     let devices = provider.get_devices().await?;
    ///     println!("Discovered {} devices", devices.len());
    /// } else {
    ///     println!("No device management provider found - using mock data");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn discover() -> Result<Option<Self>> {
        info!("🔍 Discovering device management provider (capability-based)...");

        // Query all primals for "device.management" capability
        // This is TRUE PRIMAL - we don't hardcode "biomeOS"!
        let capability = "device.management";

        // TODO: Implement actual capability discovery
        // For Phase 1, we'll check environment variable as a hint
        if let Ok(endpoint) = std::env::var("DEVICE_MANAGEMENT_ENDPOINT") {
            info!("✅ Found device management provider at: {}", endpoint);

            let provider = Self {
                endpoint,
                cache: Arc::new(RwLock::new(ProviderCache::default())),
                event_stream: Arc::new(RwLock::new(None)),
            };

            // Test connection
            if provider.health_check().await.is_ok() {
                info!("✅ Device management provider healthy");
                return Ok(Some(provider));
            } else {
                warn!("⚠️ Device management provider found but unhealthy");
            }
        }

        debug!(
            "No device management provider found for capability: {}",
            capability
        );
        Ok(None)
    }

    /// Get list of discovered devices
    pub async fn get_devices(&self) -> Result<Vec<Device>> {
        debug!("Fetching devices from provider");

        // TODO: Implement actual JSON-RPC call
        // For Phase 1, return cached or empty
        let cache = self.cache.read().await;
        Ok(cache.devices.clone())
    }

    /// Get list of discovered primals
    pub async fn get_primals_extended(&self) -> Result<Vec<Primal>> {
        debug!("Fetching primals from provider");

        // TODO: Implement actual JSON-RPC call
        // For Phase 1, return cached or empty
        let cache = self.cache.read().await;
        Ok(cache.primals.clone())
    }

    /// Get niche templates
    pub async fn get_niche_templates(&self) -> Result<Vec<NicheTemplate>> {
        debug!("Fetching niche templates from provider");

        // TODO: Implement actual JSON-RPC call
        // For Phase 1, return cached or empty
        let cache = self.cache.read().await;
        Ok(cache.niche_templates.clone())
    }

    /// Assign device to primal
    pub async fn assign_device(&self, device_id: &str, primal_id: &str) -> Result<()> {
        info!("Assigning device {} to primal {}", device_id, primal_id);

        // TODO: Implement actual JSON-RPC call
        // For Phase 1, just log
        debug!("Assignment would be sent to: {}", self.endpoint);

        Ok(())
    }

    /// Deploy niche
    pub async fn deploy_niche(&self, niche: &NicheTemplate) -> Result<String> {
        info!("Deploying niche: {}", niche.name);

        // TODO: Implement actual JSON-RPC call
        // For Phase 1, return mock niche ID
        let niche_id = format!("niche-{}", uuid::Uuid::new_v4());
        debug!("Deployed niche ID: {}", niche_id);

        Ok(niche_id)
    }

    /// Subscribe to real-time events
    pub async fn subscribe_events(&self) -> Result<()> {
        info!("Subscribing to real-time events from provider");

        // TODO: Implement WebSocket subscription
        // For Phase 1, placeholder
        debug!("Event subscription would connect to: {}", self.endpoint);

        Ok(())
    }
}

/// Implement VisualizationDataProvider for backward compatibility
#[async_trait]
impl VisualizationDataProvider for BiomeOSProvider {
    async fn get_primals(&self) -> Result<Vec<PrimalInfo>> {
        // Convert our extended Primal to core PrimalInfo
        let primals = self.get_primals_extended().await?;

        Ok(primals
            .into_iter()
            .map(|p| PrimalInfo {
                id: p.id.clone(),
                name: p.name.clone(),
                primal_type: "device-managed".to_string(),
                endpoint: format!(
                    "unix:///run/user/{}/{}.sock",
                    users::get_current_uid(),
                    p.name
                ),
                capabilities: p.capabilities.clone(),
                health: match p.health {
                    Health::Healthy => petal_tongue_core::PrimalHealthStatus::Healthy,
                    Health::Degraded => petal_tongue_core::PrimalHealthStatus::Warning,
                    Health::Offline => petal_tongue_core::PrimalHealthStatus::Critical,
                },
                last_seen: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                endpoints: None,
                metadata: None,
                properties: Default::default(),
                trust_level: None,
                family_id: None,
            })
            .collect())
    }

    async fn get_topology(&self) -> Result<Vec<TopologyEdge>> {
        // For Phase 1, return empty topology
        // Phase 2 will implement actual topology discovery
        Ok(Vec::new())
    }

    async fn health_check(&self) -> Result<String> {
        debug!("Health check for device management provider");

        // TODO: Implement actual health check
        // For Phase 1, always return healthy
        Ok("healthy".to_string())
    }

    fn get_metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            name: "Device Management Provider".to_string(),
            endpoint: self.endpoint.clone(),
            protocol: "json-rpc+websocket".to_string(),
            capabilities: vec![
                "device.discovery".to_string(),
                "device.assignment".to_string(),
                "niche.deployment".to_string(),
                "real-time.events".to_string(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_biomeos_provider_discovery_none() {
        // Without DEVICE_MANAGEMENT_ENDPOINT, should return None
        unsafe {
            // SAFETY: Test isolation - we control environment in tests
            std::env::remove_var("DEVICE_MANAGEMENT_ENDPOINT");
        }

        let provider = BiomeOSProvider::discover().await.unwrap();
        assert!(
            provider.is_none(),
            "Should return None when no provider found"
        );
    }

    #[tokio::test]
    async fn test_biomeos_provider_metadata() {
        let provider = BiomeOSProvider {
            endpoint: "unix:///tmp/test.sock".to_string(),
            cache: Arc::new(RwLock::new(ProviderCache::default())),
            event_stream: Arc::new(RwLock::new(None)),
        };

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
        let provider = BiomeOSProvider {
            endpoint: "unix:///tmp/test.sock".to_string(),
            cache: Arc::new(RwLock::new(ProviderCache::default())),
            event_stream: Arc::new(RwLock::new(None)),
        };

        let devices = provider.get_devices().await.unwrap();
        assert!(
            devices.is_empty(),
            "Empty cache should return empty devices"
        );

        let primals = provider.get_primals_extended().await.unwrap();
        assert!(
            primals.is_empty(),
            "Empty cache should return empty primals"
        );
    }
}
