// SPDX-License-Identifier: AGPL-3.0-only
//! VisualizationDataProvider trait implementation for BiomeOSProvider.
//!
//! Provides backward compatibility with the discovery crate's provider interface.

use async_trait::async_trait;
use petal_tongue_core::{PrimalInfo, TopologyEdge};
use petal_tongue_discovery::{ProviderMetadata, VisualizationDataProvider};
use tracing::debug;

use super::provider::BiomeOSProvider;
use super::types::Health;

#[async_trait]
impl VisualizationDataProvider for BiomeOSProvider {
    async fn get_primals(&self) -> anyhow::Result<Vec<PrimalInfo>> {
        // Convert our extended Primal to core PrimalInfo
        let primals = self.get_primals_extended().await?;

        Ok(primals
            .into_iter()
            .map(|p| PrimalInfo {
                id: p.id.clone().into(),
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
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
                endpoints: None,
                metadata: None,
                properties: Default::default(),
                #[allow(deprecated)]
                trust_level: None,
                #[allow(deprecated)]
                family_id: None,
            })
            .collect())
    }

    async fn get_topology(&self) -> anyhow::Result<Vec<TopologyEdge>> {
        // For Phase 1, return empty topology
        // Phase 2 will implement actual topology discovery
        Ok(Vec::new())
    }

    async fn health_check(&self) -> anyhow::Result<String> {
        debug!("Health check for device management provider");

        // TODO: Implement actual health check
        // For Phase 1, always return healthy
        Ok("healthy".to_string())
    }

    fn get_metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            name: "Device Management Provider".to_string(),
            endpoint: self.endpoint().to_string(),
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
