// SPDX-License-Identifier: AGPL-3.0-only
//! VisualizationDataProvider trait implementation for BiomeOSProvider.
//!
//! Provides backward compatibility with the discovery crate's provider interface.

use async_trait::async_trait;
use petal_tongue_core::{PrimalInfo, TopologyEdge};
use petal_tongue_discovery::{ProviderMetadata, VisualizationDataProvider};
use tokio::time::timeout;
use tracing::debug;

use super::provider::BiomeOSProvider;
use super::types::Health;

const HEALTH_CHECK_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

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
                    petal_tongue_core::system_info::get_current_uid(),
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
                #[expect(deprecated)]
                trust_level: None,
                #[expect(deprecated)]
                family_id: None,
            })
            .collect())
    }

    async fn get_topology(&self) -> anyhow::Result<Vec<TopologyEdge>> {
        // Default: empty topology (override in concrete providers)
        Ok(Vec::new())
    }

    async fn health_check(&self) -> anyhow::Result<String> {
        debug!("Health check for device management provider");

        let result = timeout(HEALTH_CHECK_TIMEOUT, self.health_check_jsonrpc()).await;

        match result {
            Ok(Ok(status)) => Ok(status),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(anyhow::anyhow!(
                "Health check timed out after {} seconds",
                HEALTH_CHECK_TIMEOUT.as_secs()
            )),
        }
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
