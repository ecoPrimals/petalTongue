// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON-RPC method calls and response parsing for the discovery service.

use crate::capability_parse;
use crate::errors::{DiscoveryError, DiscoveryResult};
use petal_tongue_core::types::{PrimalHealthStatus, PrimalInfo};
use serde_json::{Value, json};
use tracing::{debug, info, warn};

use super::DiscoveryServiceClient;

impl DiscoveryServiceClient {
    /// Discover primals by capability (semantic: discovery.query)
    ///
    /// Queries the discovery service for all registered primals with the given capability.
    /// Returns list of primals that can provide this capability.
    ///
    /// # Semantic Method Name
    /// Calls `discovery.query` per `SEMANTIC_METHOD_NAMING_STANDARD.md`
    ///
    /// # Example Capabilities
    /// - "visualization" - primals that provide UI/visualization
    /// - "encryption" - primals that provide encryption (security/crypto provider)
    /// - "storage" - primals that provide persistent storage (storage/persistence provider)
    /// - "compute" - primals that provide execution (compute provider)
    /// - "ai" - primals that provide AI inference (AI/narration provider)
    ///
    /// # Errors
    /// Returns `DiscoveryError` on network/JSON-RPC errors or invalid response.
    pub async fn discover_by_capability(
        &self,
        capability: &str,
    ) -> DiscoveryResult<Vec<PrimalInfo>> {
        debug!(
            "🔍 Querying discovery service for capability: {}",
            capability
        );

        let request = json!({
            "jsonrpc": "2.0",
            "method": "discovery.query",  // Semantic naming
            "params": {
                "capability": capability
            },
            "id": 1
        });

        let result = self.send_request(request).await?;
        let primal_infos = self.parse_primal_array(&result)?;

        info!(
            "🔍 Discovery service found {} primals with capability '{}'",
            primal_infos.len(),
            capability
        );

        Ok(primal_infos)
    }

    /// Get all registered primals
    ///
    /// Returns the complete list of primals registered with the discovery service.
    /// Uses discovery.query("*") to get all registered primals.
    ///
    /// # Errors
    /// Returns `DiscoveryError` on network/JSON-RPC errors or invalid response.
    pub async fn get_all_primals(&self) -> DiscoveryResult<Vec<PrimalInfo>> {
        debug!("🔍 Querying discovery service for all registered primals");

        let request = json!({
            "jsonrpc": "2.0",
            "method": "discovery.query",
            "params": {
                "capability": "*"
            },
            "id": 1
        });

        let result = self.send_request(request).await?;
        let primal_infos = self.parse_primal_array(&result)?;

        info!(
            "🔍 Discovery service reports {} total registered primals",
            primal_infos.len()
        );

        Ok(primal_infos)
    }

    /// Parse a JSON-RPC result array into a list of `PrimalInfo`, logging parse failures.
    fn parse_primal_array(&self, result: &Value) -> DiscoveryResult<Vec<PrimalInfo>> {
        let primals = result
            .as_array()
            .ok_or_else(|| DiscoveryError::ExpectedArray {
                context: " of primals".to_string(),
            })?;

        Ok(primals
            .iter()
            .filter_map(|v| match self.parse_primal(v) {
                Ok(info) => Some(info),
                Err(e) => {
                    warn!("Failed to parse primal from discovery response: {e}");
                    None
                }
            })
            .collect())
    }

    /// Health check — verify the discovery service is responding (semantic: health.check)
    ///
    /// # Errors
    /// Returns `DiscoveryError` on connection or JSON-RPC errors.
    pub async fn health_check(&self) -> DiscoveryResult<String> {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "health.check",  // Semantic naming
            "params": {},
            "id": 1
        });

        let result = self.send_request(request).await?;

        Ok(result["status"].as_str().unwrap_or("unknown").to_string())
    }

    /// Parse a primal from the discovery service JSON response
    #[expect(
        clippy::unused_self,
        reason = "method for consistency with other parsers"
    )]
    pub(crate) fn parse_primal(&self, value: &Value) -> DiscoveryResult<PrimalInfo> {
        let id = value["id"]
            .as_str()
            .ok_or_else(|| DiscoveryError::MissingField {
                field: "id".to_string(),
                context: String::new(),
            })?
            .to_string();

        let name = value["name"]
            .as_str()
            .ok_or_else(|| DiscoveryError::MissingField {
                field: "name".to_string(),
                context: String::new(),
            })?
            .to_string();

        let primal_type = value["primal_type"]
            .as_str()
            .or_else(|| value["type"].as_str())
            .unwrap_or("unknown")
            .to_string();

        let endpoint = value["endpoint"]
            .as_str()
            .ok_or_else(|| DiscoveryError::MissingField {
                field: "endpoint".to_string(),
                context: String::new(),
            })?
            .to_string();

        let capabilities: Vec<String> = value["capabilities"]
            .as_array()
            .map(|v| capability_parse::parse_capabilities(v))
            .unwrap_or_default();

        let health = value["health"]
            .as_str()
            .and_then(|s| match s.to_lowercase().as_str() {
                "healthy" | "ok" => Some(PrimalHealthStatus::Healthy),
                "degraded" | "warning" => Some(PrimalHealthStatus::Warning),
                "unhealthy" | "error" | "critical" => Some(PrimalHealthStatus::Critical),
                _ => None,
            })
            .unwrap_or(PrimalHealthStatus::Healthy);

        let last_seen = value["last_seen"].as_u64().unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0) // Fallback to epoch if clock is broken
        });

        Ok(PrimalInfo::new(
            id,
            name,
            primal_type,
            endpoint,
            capabilities,
            health,
            last_seen,
        ))
    }
}
