// SPDX-License-Identifier: AGPL-3.0-or-later
//! `BiomeOS` API Client
//!
//! Connects to BiomeOS/Songbird for live primal discovery and health monitoring.

use crate::biomeos_error::BiomeOsClientError;
use petal_tongue_core::{PrimalHealthStatus, PrimalInfo, Properties, TopologyEdge};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// `BiomeOS` API client for live primal discovery
pub struct BiomeOSClient {
    /// `BiomeOS` API base URL
    base_url: String,
    /// HTTP client
    client: reqwest::Client,
    /// Enable fixture mode for development (deterministic data when biomeOS unavailable).
    fixture_mode: bool,
}

/// Response from `BiomeOS` discovery API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResponse {
    /// List of discovered primals
    pub primals: Vec<DiscoveredPrimal>,
}

/// Response from `BiomeOS` topology API (new format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyResponse {
    /// Topology nodes (enriched with trust levels, family, etc.)
    #[serde(default)]
    pub nodes: Vec<TopologyNode>,
    /// Topology edges (connections between primals)
    pub edges: Vec<TopologyEdge>,
    /// Mode indicator (mock, live, etc.)
    #[serde(default)]
    pub mode: String,
}

/// Topology node (enriched node data from biomeOS)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyNode {
    /// Node ID
    pub id: String,
    /// Node name
    #[serde(default)]
    pub name: String,
    /// Node type
    #[serde(default, rename = "type")]
    pub node_type: String,
    /// Health status
    #[serde(default)]
    pub status: String,
    /// Trust level (0-3)
    #[serde(default)]
    pub trust_level: Option<u8>,
    /// Family ID (genetic lineage)
    #[serde(default)]
    pub family_id: Option<String>,
    /// Capabilities
    #[serde(default)]
    pub capabilities: Vec<String>,
}

/// A discovered primal from `BiomeOS`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPrimal {
    /// Primal ID
    pub id: String,
    /// Primal name
    pub name: String,
    /// Primal type
    pub primal_type: String,
    /// Endpoint URL
    pub endpoint: String,
    /// Capabilities
    pub capabilities: Vec<String>,
    /// Health status
    pub health: String,
    /// Last seen timestamp (Unix)
    pub last_seen: u64,
}

impl BiomeOSClient {
    /// Create a new `BiomeOS` client
    pub fn new(base_url: impl Into<String>) -> Self {
        // Build HTTP client with robust configuration
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30)) // Increased timeout
            .connect_timeout(Duration::from_secs(10)) // Separate connect timeout
            .pool_idle_timeout(Duration::from_secs(90)) // Keep connections alive longer
            .pool_max_idle_per_host(10) // More idle connections
            .tcp_keepalive(Duration::from_secs(60)) // TCP keep-alive
            .http2_keep_alive_interval(Some(Duration::from_secs(30))) // HTTP/2 keep-alive
            .http2_keep_alive_timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|e| {
                tracing::warn!(
                    "Failed to build HTTP client with custom config: {}. Using default client.",
                    e
                );
                reqwest::Client::new()
            });

        Self {
            base_url: base_url.into(),
            client,
            fixture_mode: false,
        }
    }

    /// Enable fixture mode (deterministic data when biomeOS unavailable).
    /// Requires `test-fixtures` feature; production builds return
    /// [`BiomeOsClientError::FixtureModeUnavailable`] at runtime.
    #[must_use]
    pub const fn with_fixture_mode(mut self, enabled: bool) -> Self {
        self.fixture_mode = enabled;
        self
    }

    /// Check if `BiomeOS` API is available
    ///
    /// # Errors
    /// Returns `BiomeOsClientError` on network failure or when fixture mode is enabled without the feature.
    pub async fn health_check(&self) -> Result<bool, BiomeOsClientError> {
        #[cfg(any(test, feature = "test-fixtures"))]
        if self.fixture_mode {
            return Ok(true);
        }
        #[cfg(not(any(test, feature = "test-fixtures")))]
        if self.fixture_mode {
            return Err(BiomeOsClientError::FixtureModeUnavailable);
        }

        let url = format!("{}/api/v1/health", self.base_url);
        Ok(self
            .client
            .get(&url)
            .send()
            .await
            .is_ok_and(|response| response.status().is_success()))
    }

    /// Discover primals from `BiomeOS`/Songbird
    ///
    /// **PRODUCTION MODE**: Returns error if API fails (no fixture fallback)
    /// **FIXTURE MODE**: Set `fixture_mode` for deterministic test data
    ///
    /// # Errors
    /// Returns `BiomeOsClientError` on network failure, non-success status, or JSON parse error.
    pub async fn discover_primals(&self) -> Result<Vec<PrimalInfo>, BiomeOsClientError> {
        #[cfg(any(test, feature = "test-fixtures"))]
        if self.fixture_mode {
            tracing::debug!("fixture mode — returning deterministic primals");
            return Ok(self.mock_discover_primals());
        }
        #[cfg(not(any(test, feature = "test-fixtures")))]
        if self.fixture_mode {
            return Err(BiomeOsClientError::FixtureModeUnavailable);
        }

        // Query BiomeOS discovery endpoint
        let url = format!("{}/api/v1/primals", self.base_url);

        let response =
            self.client
                .get(&url)
                .send()
                .await
                .map_err(|e| BiomeOsClientError::Network {
                    url: url.clone(),
                    source: e,
                })?;

        if !response.status().is_success() {
            return Err(BiomeOsClientError::ServerError {
                status: response.status().as_u16(),
                url,
            });
        }

        let discovery = response.json::<DiscoveryResponse>().await.map_err(|e| {
            BiomeOsClientError::Parse(format!(
                "{e}\n\n\
                This may indicate:\n\
                - API format mismatch\n\
                - biomeOS API is not fully implemented\n\
                - Response is not valid JSON\n\
                \n\
                Expected format: {{\"primals\": [...]}}"
            ))
        })?;

        tracing::info!(
            "✅ Successfully discovered {} primals from biomeOS",
            discovery.primals.len()
        );

        Ok(discovery.primals.into_iter().map(|p| p.into()).collect())
    }

    /// Get topology edges (connections between primals)
    ///
    /// **PRODUCTION MODE**: Returns error if API fails (no fixture fallback)
    /// **FIXTURE MODE**: Set `fixture_mode` for deterministic topology data
    ///
    /// Supports biomeOS's topology format with nodes + edges.
    ///
    /// # Errors
    /// Returns `BiomeOsClientError` on network failure, non-success status, or JSON parse error.
    pub async fn get_topology(&self) -> Result<Vec<TopologyEdge>, BiomeOsClientError> {
        #[cfg(any(test, feature = "test-fixtures"))]
        if self.fixture_mode {
            tracing::debug!("fixture mode — returning deterministic topology");
            return Ok(self.mock_topology());
        }
        #[cfg(not(any(test, feature = "test-fixtures")))]
        if self.fixture_mode {
            return Err(BiomeOsClientError::FixtureModeUnavailable);
        }

        let url = format!("{}/api/v1/topology", self.base_url);

        let response =
            self.client
                .get(&url)
                .send()
                .await
                .map_err(|e| BiomeOsClientError::Network {
                    url: url.clone(),
                    source: e,
                })?;

        if !response.status().is_success() {
            return Err(BiomeOsClientError::ServerError {
                status: response.status().as_u16(),
                url,
            });
        }

        // Try new format first (nodes + edges + mode)
        let topology = response.json::<TopologyResponse>().await.map_err(|e| {
            BiomeOsClientError::Parse(format!(
                "{e}\n\n\
                Expected format: {{\"nodes\": [...], \"edges\": [...], \"mode\": \"...\"}}"
            ))
        })?;

        tracing::debug!(
            "✅ Successfully retrieved topology: {} nodes, {} edges (mode: {})",
            topology.nodes.len(),
            topology.edges.len(),
            topology.mode
        );

        // Returns edges for topology graph construction.
        // Enriched node data (trust, family) is available via topology.nodes when needed.
        Ok(topology.edges)
    }

    /// Fixture primal discovery (test/dev only).
    /// Gated behind `test-fixtures` feature. Production builds return error when `fixture_mode` is set.
    #[cfg(any(test, feature = "test-fixtures"))]
    fn mock_discover_primals(&self) -> Vec<PrimalInfo> {
        let now = chrono::Utc::now().timestamp().cast_unsigned();
        vec![
            PrimalInfo {
                id: "primal-alpha".into(),
                name: "Security Primal".to_string(),
                primal_type: "Security".to_string(),
                endpoint: "http://mock-primal-alpha:8001".to_string(),
                capabilities: vec![
                    "authentication".to_string(),
                    "authorization".to_string(),
                    "encryption".to_string(),
                ],
                health: PrimalHealthStatus::Healthy,
                last_seen: now,
                endpoints: None,
                metadata: None,
                properties: Properties::new(), // Generic properties
            },
            PrimalInfo {
                id: "primal-beta".into(),
                name: "Compute Primal".to_string(),
                primal_type: "Compute".to_string(),
                endpoint: "http://mock-primal-beta:8002".to_string(),
                capabilities: vec![
                    "container_runtime".to_string(),
                    "workload_execution".to_string(),
                ],
                health: PrimalHealthStatus::Warning,
                last_seen: now,
                endpoints: None,
                metadata: None,
                properties: Properties::new(),
            },
            PrimalInfo {
                id: "primal-gamma".into(),
                name: "Discovery Primal".to_string(),
                primal_type: "Discovery".to_string(),
                endpoint: "http://mock-primal-gamma:8003".to_string(),
                capabilities: vec![
                    "service_discovery".to_string(),
                    "capability_matching".to_string(),
                ],
                health: PrimalHealthStatus::Healthy,
                last_seen: now,
                endpoints: None,
                metadata: None,
                properties: Properties::new(),
            },
            PrimalInfo {
                id: "primal-delta".into(),
                name: "Storage Primal".to_string(),
                primal_type: "Storage".to_string(),
                endpoint: "http://mock-primal-delta:8004".to_string(),
                capabilities: vec![
                    "permanent_storage".to_string(),
                    "content_addressing".to_string(),
                    "attribution".to_string(),
                ],
                health: PrimalHealthStatus::Healthy,
                last_seen: now,
                endpoints: None,
                metadata: None,
                properties: Properties::new(),
            },
            PrimalInfo {
                id: "primal-epsilon".into(),
                name: "AI Primal".to_string(),
                primal_type: "AI".to_string(),
                endpoint: "http://mock-primal-epsilon:8005".to_string(),
                capabilities: vec!["intent_parsing".to_string(), "task_planning".to_string()],
                health: PrimalHealthStatus::Critical,
                last_seen: now,
                endpoints: None,
                metadata: None,
                properties: Properties::new(),
            },
        ]
    }

    /// Mock topology (TEST/DEV ONLY - never in production)
    /// Gated behind test-fixtures feature.
    #[cfg(any(test, feature = "test-fixtures"))]
    fn mock_topology(&self) -> Vec<TopologyEdge> {
        vec![
            TopologyEdge {
                from: "primal-alpha".into(),
                to: "primal-beta".into(),
                edge_type: "authenticates".to_string(),
                capability: None,
                metrics: None,
                label: Some("Auth Flow".to_string()),
            },
            TopologyEdge {
                from: "primal-gamma".into(),
                to: "primal-alpha".into(),
                edge_type: "discovers".to_string(),
                capability: None,
                metrics: None,
                label: None,
            },
            TopologyEdge {
                from: "primal-beta".into(),
                to: "primal-delta".into(),
                edge_type: "stores_to".to_string(),
                capability: None,
                metrics: None,
                label: Some("Data Flow".to_string()),
            },
            TopologyEdge {
                from: "primal-epsilon".into(),
                to: "primal-gamma".into(),
                edge_type: "queries".to_string(),
                capability: None,
                metrics: None,
                label: None,
            },
            TopologyEdge {
                from: "primal-epsilon".into(),
                to: "primal-beta".into(),
                edge_type: "orchestrates".to_string(),
                capability: None,
                metrics: None,
                label: Some("Task Execution".to_string()),
            },
        ]
    }
}

/// Convert `DiscoveredPrimal` to `PrimalInfo`
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
            properties: Properties::new(), // Start with empty properties
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_fixture_mode() {
        let client = BiomeOSClient::new("http://test-mock:9000").with_fixture_mode(true);

        let primals = client.discover_primals().await.unwrap();
        assert_eq!(primals.len(), 5);
        assert_eq!(primals[0].id, "primal-alpha");

        let topology = client.get_topology().await.unwrap();
        assert_eq!(topology.len(), 5);

        // Health check should always succeed in mock mode
        let health = client.health_check().await.unwrap();
        assert!(health);
    }

    #[tokio::test]
    async fn test_health_check_failure() {
        let client = BiomeOSClient::new("http://nonexistent:99999");

        let health = client.health_check().await.unwrap();
        assert!(!health);
    }

    #[tokio::test]
    async fn test_convert_discovered_primal() {
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

    #[test]
    fn test_discovery_response_serialization() {
        let response = DiscoveryResponse {
            primals: vec![DiscoveredPrimal {
                id: "p1".to_string(),
                name: "Primal 1".to_string(),
                primal_type: "Compute".to_string(),
                endpoint: "http://localhost:8000".to_string(),
                capabilities: vec!["compute".to_string()],
                health: "healthy".to_string(),
                last_seen: 0,
            }],
        };
        let json = serde_json::to_string(&response).unwrap();
        let parsed: DiscoveryResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.primals.len(), 1);
        assert_eq!(parsed.primals[0].id, "p1");
    }

    #[test]
    fn test_topology_response_serialization() {
        let response = TopologyResponse {
            nodes: vec![TopologyNode {
                id: "n1".to_string(),
                name: "Node 1".to_string(),
                node_type: "Compute".to_string(),
                status: "healthy".to_string(),
                trust_level: Some(3),
                family_id: Some("fam-1".to_string()),
                capabilities: vec!["compute".to_string()],
            }],
            edges: vec![TopologyEdge {
                from: "n1".into(),
                to: "n2".into(),
                edge_type: "conn".to_string(),
                label: None,
                capability: None,
                metrics: None,
            }],
            mode: "live".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        let parsed: TopologyResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.nodes.len(), 1);
        assert_eq!(parsed.edges.len(), 1);
    }

    #[test]
    fn test_discovered_primal_health_mapping() {
        for (health_str, expected) in [
            ("healthy", PrimalHealthStatus::Healthy),
            ("warning", PrimalHealthStatus::Warning),
            ("critical", PrimalHealthStatus::Critical),
            ("unknown", PrimalHealthStatus::Unknown),
        ] {
            let discovered = DiscoveredPrimal {
                id: "x".to_string(),
                name: "X".to_string(),
                primal_type: "T".to_string(),
                endpoint: "http://x".to_string(),
                capabilities: vec![],
                health: health_str.to_string(),
                last_seen: 0,
            };
            let info: PrimalInfo = discovered.into();
            assert_eq!(info.health, expected);
        }
    }

    #[tokio::test]
    async fn test_discover_primals_success_via_wiremock() {
        let mock_server = MockServer::start().await;

        let discovery_json = serde_json::json!({
            "primals": [
                {
                    "id": "p1",
                    "name": "Primal 1",
                    "primal_type": "Compute",
                    "endpoint": "http://localhost:8000",
                    "capabilities": ["compute"],
                    "health": "healthy",
                    "last_seen": 12345
                }
            ]
        });

        Mock::given(method("GET"))
            .and(path("/api/v1/primals"))
            .respond_with(ResponseTemplate::new(200).set_body_json(discovery_json))
            .mount(&mock_server)
            .await;

        let client = BiomeOSClient::new(mock_server.uri()).with_fixture_mode(false);
        let primals = client.discover_primals().await.expect("discover_primals");
        assert_eq!(primals.len(), 1);
        assert_eq!(primals[0].id, "p1");
        assert_eq!(primals[0].name, "Primal 1");
    }

    #[tokio::test]
    async fn test_discover_primals_server_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v1/primals"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let client = BiomeOSClient::new(mock_server.uri()).with_fixture_mode(false);
        let err = client.discover_primals().await.expect_err("should fail");
        assert!(matches!(
            err,
            BiomeOsClientError::ServerError { status: 500, .. }
        ));
    }

    #[tokio::test]
    async fn test_discover_primals_parse_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v1/primals"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not valid json"))
            .mount(&mock_server)
            .await;

        let client = BiomeOSClient::new(mock_server.uri()).with_fixture_mode(false);
        let err = client.discover_primals().await.expect_err("should fail");
        assert!(matches!(err, BiomeOsClientError::Parse(_)));
    }

    #[tokio::test]
    async fn test_get_topology_success_via_wiremock() {
        let mock_server = MockServer::start().await;

        let topology_json = serde_json::json!({
            "nodes": [{"id": "n1", "name": "Node 1"}],
            "edges": [
                {"from": "n1", "to": "n2", "edge_type": "conn"}
            ],
            "mode": "live"
        });

        Mock::given(method("GET"))
            .and(path("/api/v1/topology"))
            .respond_with(ResponseTemplate::new(200).set_body_json(topology_json))
            .mount(&mock_server)
            .await;

        let client = BiomeOSClient::new(mock_server.uri()).with_fixture_mode(false);
        let edges = client.get_topology().await.expect("get_topology");
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].from.as_str(), "n1");
        assert_eq!(edges[0].to.as_str(), "n2");
    }

    #[tokio::test]
    async fn test_get_topology_server_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v1/topology"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        let client = BiomeOSClient::new(mock_server.uri()).with_fixture_mode(false);
        let err = client.get_topology().await.expect_err("should fail");
        assert!(matches!(
            err,
            BiomeOsClientError::ServerError { status: 404, .. }
        ));
    }

    #[tokio::test]
    async fn test_get_topology_parse_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v1/topology"))
            .respond_with(ResponseTemplate::new(200).set_body_string("{invalid"))
            .mount(&mock_server)
            .await;

        let client = BiomeOSClient::new(mock_server.uri()).with_fixture_mode(false);
        let err = client.get_topology().await.expect_err("should fail");
        assert!(matches!(err, BiomeOsClientError::Parse(_)));
    }

    #[tokio::test]
    async fn test_health_check_success_via_wiremock() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v1/health"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let client = BiomeOSClient::new(mock_server.uri()).with_fixture_mode(false);
        let healthy = client.health_check().await.expect("health_check");
        assert!(healthy);
    }

    #[tokio::test]
    async fn test_health_check_non_success_status() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v1/health"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&mock_server)
            .await;

        let client = BiomeOSClient::new(mock_server.uri()).with_fixture_mode(false);
        let healthy = client.health_check().await.expect("health_check");
        assert!(!healthy);
    }
}
