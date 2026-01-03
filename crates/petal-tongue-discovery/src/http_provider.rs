//! HTTP-based visualization data provider
//!
//! Generic provider that works with any HTTP REST API following the
//! visualization data contract (not specific to biomeOS!)

use crate::traits::{ProviderMetadata, VisualizationDataProvider};
use async_trait::async_trait;
use petal_tongue_core::{PrimalHealthStatus, PrimalInfo, TopologyEdge};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// HTTP-based visualization data provider
///
/// Works with any primal that exposes:
/// - `GET /api/v1/health`
/// - `GET /api/v1/primals`
/// - `GET /api/v1/topology` (optional)
pub struct HttpVisualizationProvider {
    /// Endpoint URL
    endpoint: String,
    /// HTTP client
    client: reqwest::Client,
}

/// Response from primal discovery endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DiscoveryResponse {
    primals: Vec<DiscoveredPrimal>,
}

/// A discovered primal
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DiscoveredPrimal {
    id: String,
    name: String,
    primal_type: String,
    endpoint: String,
    capabilities: Vec<String>,
    health: String,
    last_seen: u64,
}

impl HttpVisualizationProvider {
    /// Create a new HTTP provider
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30)) // Increased timeout
                .connect_timeout(Duration::from_secs(10)) // Separate connect timeout
                .pool_idle_timeout(Duration::from_secs(90)) // Keep connections alive longer
                .pool_max_idle_per_host(10) // More idle connections
                .tcp_keepalive(Duration::from_secs(60)) // TCP keep-alive
                .http2_keep_alive_interval(Some(Duration::from_secs(30))) // HTTP/2 keep-alive
                .http2_keep_alive_timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to build HTTP client"),
        }
    }
}

#[async_trait]
impl VisualizationDataProvider for HttpVisualizationProvider {
    async fn get_primals(&self) -> anyhow::Result<Vec<PrimalInfo>> {
        let url = format!("{}/api/v1/primals", self.endpoint);
        
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Provider returned error status: {}",
                response.status()
            ));
        }

        let discovery: DiscoveryResponse = response.json().await?;
        
        Ok(discovery
            .primals
            .into_iter()
            .map(|p| p.into())
            .collect())
    }

    async fn get_topology(&self) -> anyhow::Result<Vec<TopologyEdge>> {
        let url = format!("{}/api/v1/topology", self.endpoint);
        
        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    // Try to parse as full topology response (with nodes and edges)
                    #[derive(serde::Deserialize)]
                    struct TopologyResponse {
                        edges: Vec<TopologyEdge>,
                        #[allow(dead_code)]
                        nodes: Option<serde_json::Value>, // Optional nodes field
                    }
                    
                    if let Ok(topology) = response.json::<TopologyResponse>().await {
                        Ok(topology.edges)
                    } else {
                        // Fallback to direct edge array
                        Ok(Vec::new())
                    }
                } else {
                    // Topology endpoint not available - that's okay
                    Ok(Vec::new())
                }
            }
            Err(_) => {
                // Topology endpoint not available - that's okay
                Ok(Vec::new())
            }
        }
    }

    async fn health_check(&self) -> anyhow::Result<String> {
        let url = format!("{}/api/v1/health", self.endpoint);
        
        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(format!("HTTP provider at {} is healthy", self.endpoint))
                } else {
                    Err(anyhow::anyhow!(
                        "Health check failed: {}",
                        response.status()
                    ))
                }
            }
            Err(e) => Err(anyhow::anyhow!("Health check failed: {}", e)),
        }
    }

    fn get_metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            name: "HTTP Provider".to_string(), // Generic name - not used for logic!
            endpoint: self.endpoint.clone(),
            protocol: "http".to_string(),
            capabilities: vec![
                "visualization.primal-provider".to_string(),
                "visualization.health-provider".to_string(),
            ],
        }
    }
}

/// Convert discovered primal to PrimalInfo
impl From<DiscoveredPrimal> for PrimalInfo {
    fn from(primal: DiscoveredPrimal) -> Self {
        Self {
            id: primal.id,
            name: primal.name,
            primal_type: primal.primal_type,
            endpoint: primal.endpoint,
            capabilities: primal.capabilities,
            health: match primal.health.to_lowercase().as_str() {
                "healthy" => PrimalHealthStatus::Healthy,
                "warning" => PrimalHealthStatus::Warning,
                "critical" => PrimalHealthStatus::Critical,
                _ => PrimalHealthStatus::Unknown,
            },
            last_seen: primal.last_seen,
            trust_level: None,  // Will be enriched from topology data
            family_id: None,    // Will be enriched from topology data
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = HttpVisualizationProvider::new("http://test:9000");
        let metadata = provider.get_metadata();
        assert_eq!(metadata.endpoint, "http://test:9000");
        assert_eq!(metadata.protocol, "http");
    }

    #[test]
    fn test_discovered_primal_conversion() {
        let discovered = DiscoveredPrimal {
            id: "test-1".to_string(),
            name: "Test Primal".to_string(),
            primal_type: "Test".to_string(),
            endpoint: "http://test:8000".to_string(),
            capabilities: vec!["test".to_string()],
            health: "healthy".to_string(),
            last_seen: 1234567890,
        };

        let primal_info: PrimalInfo = discovered.into();
        assert_eq!(primal_info.id, "test-1");
        assert_eq!(primal_info.health, PrimalHealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_health_check_with_invalid_endpoint() {
        let provider = HttpVisualizationProvider::new("http://nonexistent:99999");
        let health = provider.health_check().await;
        assert!(health.is_err(), "Invalid endpoint should fail health check");
    }
}

