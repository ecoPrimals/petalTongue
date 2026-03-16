// SPDX-License-Identifier: AGPL-3.0-or-later
//! HTTP-based visualization data provider
//!
//! ⚠️  **DEPRECATED AS PRIMARY PROTOCOL** ⚠️
//!
//! HTTP/REST is the **FALLBACK** protocol for external integrations only.
//! The PRIMARY protocol for ecoPrimals is **JSON-RPC 2.0 over Unix sockets**.
//!
//! # When to Use This Provider (Fallback)
//!
//! This provider is kept for backward compatibility and specific use cases:
//!
//! - **External web integrations**: Web dashboards, remote monitoring UIs
//! - **Remote access over network**: When primals are on different hosts
//! - **Legacy systems**: Systems without Unix socket support
//! - **CI/test environments**: When `BIOMEOS_URL=http://...` is simpler to configure
//!
//! Enable with: `cargo build --features legacy-http`
//!
//! # Why JSON-RPC First?
//!
//! - 100x faster (Unix sockets vs TCP/IP)
//! - Port-free architecture
//! - Secure by default (file permissions)
//! - Compatible with all primals (Songbird, `BearDog`, `ToadStool`, etc.)
//!
//! # Migration
//!
//! Replace:
//! ```bash
//! BIOMEOS_URL=http://localhost:3000
//! ```
//!
//! With:
//! ```bash
//! BIOMEOS_URL=unix:///run/user/$UID/biomeos-device-management.sock
//! ```

use crate::errors::{DiscoveryError, DiscoveryResult};
use crate::traits::{ProviderMetadata, VisualizationDataProvider};
use async_trait::async_trait;
use petal_tongue_core::{PrimalHealthStatus, PrimalInfo, TopologyEdge};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// HTTP-based visualization data provider
///
/// ⚠️  **DEPRECATED AS PRIMARY PROTOCOL**
///
/// This provider is for **external integrations only**.
/// Use `JsonRpcProvider` for TRUE PRIMAL inter-primal communication!
///
/// Works with any primal that exposes:
/// - `GET /api/v1/health`
/// - `GET /api/v1/primals`
/// - `GET /api/v1/topology` (optional)
#[deprecated(
    since = "1.4.0",
    note = "HTTP is FALLBACK protocol. Use JsonRpcProvider for TRUE PRIMAL architecture!"
)]
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

#[expect(deprecated)]
impl HttpVisualizationProvider {
    /// Create a new HTTP provider
    ///
    /// ⚠️  **DEPRECATED**: Use `JsonRpcProvider` for TRUE PRIMAL protocol!
    ///
    /// This is for external integrations only (web APIs, remote access).
    #[expect(deprecated)]
    pub fn new(endpoint: impl Into<String>) -> DiscoveryResult<Self> {
        let endpoint_str = endpoint.into();

        // Log deprecation warning
        tracing::warn!("⚠️  HttpVisualizationProvider is DEPRECATED as primary protocol!");
        tracing::warn!(
            "💡 Migrate to JSON-RPC: JsonRpcProvider::discover() or BIOMEOS_URL=unix://..."
        );
        tracing::info!("Using HTTP provider at: {}", endpoint_str);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30)) // Increased timeout
            .connect_timeout(Duration::from_secs(10)) // Separate connect timeout
            .pool_idle_timeout(Duration::from_secs(90)) // Keep connections alive longer
            .pool_max_idle_per_host(10) // More idle connections
            .tcp_keepalive(Duration::from_secs(60)) // TCP keep-alive
            .http2_keep_alive_interval(Some(Duration::from_secs(30))) // HTTP/2 keep-alive
            .http2_keep_alive_timeout(Duration::from_secs(10))
            .build()
            .map_err(DiscoveryError::HttpError)?;

        Ok(Self {
            endpoint: endpoint_str,
            client,
        })
    }
}

#[async_trait]
#[expect(deprecated)]
impl VisualizationDataProvider for HttpVisualizationProvider {
    async fn get_primals(&self) -> DiscoveryResult<Vec<PrimalInfo>> {
        let url = format!("{}/api/v1/primals", self.endpoint);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(DiscoveryError::HttpError)?;

        if !response.status().is_success() {
            return Err(DiscoveryError::ProviderHttpError {
                status: response.status().as_u16(),
                endpoint: Some(self.endpoint.clone()),
            });
        }

        let discovery: DiscoveryResponse =
            response
                .json()
                .await
                .map_err(|e| DiscoveryError::ParseError {
                    data_type: "primals".to_string(),
                    message: e.to_string(),
                })?;

        Ok(discovery
            .primals
            .into_iter()
            .map(std::convert::Into::into)
            .collect())
    }

    async fn get_topology(&self) -> DiscoveryResult<Vec<TopologyEdge>> {
        let url = format!("{}/api/v1/topology", self.endpoint);

        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    // Try to parse as full topology response (with nodes and edges)
                    #[derive(serde::Deserialize)]
                    struct TopologyResponse {
                        edges: Vec<TopologyEdge>,
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

    async fn health_check(&self) -> DiscoveryResult<String> {
        let url = format!("{}/api/v1/health", self.endpoint);

        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(format!("HTTP provider at {} is healthy", self.endpoint))
                } else {
                    Err(DiscoveryError::ProviderHttpError {
                        status: response.status().as_u16(),
                        endpoint: Some(self.endpoint.clone()),
                    })
                }
            }
            Err(e) => Err(DiscoveryError::HealthCheckFailed {
                name: "HTTP Provider".to_string(),
                endpoint: self.endpoint.clone(),
                source: Box::new(e),
            }),
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

/// Convert discovered primal to `PrimalInfo`
impl From<DiscoveredPrimal> for PrimalInfo {
    fn from(primal: DiscoveredPrimal) -> Self {
        Self {
            id: primal.id.into(),
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
            endpoints: None,
            metadata: None,
            properties: petal_tongue_core::Properties::new(), // Start with empty properties
            #[expect(deprecated)]
            trust_level: None, // Will be enriched from topology data
            #[expect(deprecated)]
            family_id: None, // Will be enriched from topology data
        }
    }
}

#[cfg(test)]
#[expect(deprecated)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = HttpVisualizationProvider::new("http://test:9000").unwrap();
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
            last_seen: 1_234_567_890,
        };

        let primal_info: PrimalInfo = discovered.into();
        assert_eq!(primal_info.id, "test-1");
        assert_eq!(primal_info.health, PrimalHealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_health_check_with_invalid_endpoint() {
        let provider = HttpVisualizationProvider::new("http://nonexistent:99999").unwrap();
        let health = provider.health_check().await;
        assert!(health.is_err(), "Invalid endpoint should fail health check");
    }

    #[test]
    fn test_provider_metadata_capabilities() {
        let provider = HttpVisualizationProvider::new("http://localhost:3000").unwrap();
        let metadata = provider.get_metadata();
        assert_eq!(metadata.name, "HTTP Provider");
        assert!(
            metadata
                .capabilities
                .contains(&"visualization.primal-provider".to_string())
        );
        assert!(
            metadata
                .capabilities
                .contains(&"visualization.health-provider".to_string())
        );
    }

    #[test]
    fn test_discovered_primal_health_variants() {
        for (health, expected) in [
            ("healthy", PrimalHealthStatus::Healthy),
            ("warning", PrimalHealthStatus::Warning),
            ("critical", PrimalHealthStatus::Critical),
        ] {
            let discovered = DiscoveredPrimal {
                id: "test".to_string(),
                name: "Test".to_string(),
                primal_type: "Test".to_string(),
                endpoint: "http://test:8000".to_string(),
                capabilities: vec![],
                health: health.to_string(),
                last_seen: 0,
            };
            let primal: PrimalInfo = discovered.into();
            assert!(
                std::mem::discriminant(&primal.health) == std::mem::discriminant(&expected),
                "health {health}"
            );
        }
    }

    #[test]
    fn test_discovered_primal_unknown_health() {
        let discovered = DiscoveredPrimal {
            id: "test".to_string(),
            name: "Test".to_string(),
            primal_type: "Test".to_string(),
            endpoint: "http://test:8000".to_string(),
            capabilities: vec!["cap".to_string()],
            health: "invalid".to_string(),
            last_seen: 123,
        };
        let primal: PrimalInfo = discovered.into();
        assert!(matches!(primal.health, PrimalHealthStatus::Unknown));
        assert_eq!(primal.capabilities.len(), 1);
        assert_eq!(primal.last_seen, 123);
    }

    #[test]
    fn test_provider_creation_with_trailing_slash() {
        let provider = HttpVisualizationProvider::new("http://test:9000/").unwrap();
        let metadata = provider.get_metadata();
        assert_eq!(metadata.endpoint, "http://test:9000/");
    }
}
