// SPDX-License-Identifier: AGPL-3.0-only
//! Capability-Based Discovery System
//!
//! TRUE PRIMAL principle: Primals have self-knowledge only.
//! They discover other primals by capability at runtime, never by name.
//!
//! This module provides the foundation for capability-based primal discovery,
//! eliminating all hardcoded primal names from the codebase.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// A capability that a primal can provide
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Capability {
    /// Capability domain (e.g., "crypto", "display", "storage")
    pub domain: String,
    /// Specific operation (e.g., "encrypt", "render", "store")
    pub operation: Option<String>,
    /// Version compatibility (semver)
    pub version: String,
}

impl Capability {
    /// Create a new capability
    pub fn new(domain: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            operation: None,
            version: "1.0.0".to_string(),
        }
    }

    /// Set specific operation
    #[must_use]
    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        self.operation = Some(operation.into());
        self
    }

    /// Set version requirement
    #[must_use]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Check if this capability matches a query
    #[must_use]
    pub fn matches(&self, query: &CapabilityQuery) -> bool {
        // Domain must match
        if self.domain != query.domain {
            return false;
        }

        // If query specifies operation, it must match
        if let Some(ref query_op) = query.operation
            && self.operation.as_ref() != Some(query_op)
        {
            return false;
        }

        // TODO: Add version compatibility check (semver)
        true
    }
}

/// A query for discovering primals by capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityQuery {
    /// Required capability domain
    pub domain: String,
    /// Optional specific operation
    pub operation: Option<String>,
    /// Optional version requirement
    pub version_req: Option<String>,
}

impl CapabilityQuery {
    /// Create a new capability query
    pub fn new(domain: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            operation: None,
            version_req: None,
        }
    }

    /// Query for specific operation
    #[must_use]
    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        self.operation = Some(operation.into());
        self
    }
}

/// Information about a discovered primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalEndpoint {
    /// Primal identifier (UUID or name - for internal routing only)
    pub id: String,
    /// Capabilities this primal provides
    pub capabilities: Vec<Capability>,
    /// Communication endpoints
    pub endpoints: PrimalEndpoints,
    /// Health status
    pub health: PrimalHealth,
}

/// Communication endpoints for a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalEndpoints {
    /// tarpc endpoint (primary, high-performance)
    pub tarpc: Option<String>,
    /// JSON-RPC endpoint (fallback, universal)
    pub jsonrpc: Option<String>,
    /// HTTPS endpoint (external, if applicable)
    pub https: Option<String>,
}

/// Health status of a primal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimalHealth {
    /// Primal is fully operational and responding normally
    Healthy,
    /// Primal is operational but experiencing issues (e.g., high latency, partial functionality)
    Degraded,
    /// Primal is not responding or has failed
    Unavailable,
}

/// Capability-based discovery service
///
/// This is the interface that primals use to discover each other.
/// Implementation delegates to biomeOS or other discovery providers.
pub struct CapabilityDiscovery {
    /// Cache of discovered primals
    cache: Arc<RwLock<HashMap<String, PrimalEndpoint>>>,
    /// Discovery backend (biomeOS, mDNS, etc.)
    backend: Box<dyn DiscoveryBackend>,
}

/// Discovery backend trait
///
/// Different implementations: biomeOS (primary), mDNS (fallback), static config
#[async_trait::async_trait]
pub trait DiscoveryBackend: Send + Sync {
    /// Query for primals providing a capability
    async fn query(&self, query: &CapabilityQuery) -> Result<Vec<PrimalEndpoint>, DiscoveryError>;

    /// Subscribe to capability changes (real-time updates)
    async fn subscribe(&self, query: &CapabilityQuery) -> Result<(), DiscoveryError>;
}

/// Discovery errors
#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    /// No primal was found that provides the requested capability
    #[error("No primal found providing capability: {domain}")]
    CapabilityNotFound {
        /// The capability domain that was not found
        domain: String,
    },

    /// The discovery backend (biomeOS, mDNS, etc.) is not available
    #[error("Discovery backend unavailable: {0}")]
    BackendUnavailable(String),

    /// Communication with the discovery backend failed
    #[error("Communication error: {0}")]
    CommunicationError(String),

    /// Multiple primals provide the same capability (requires disambiguation)
    #[error("Multiple primals provide capability (ambiguous): {domain}")]
    AmbiguousCapability {
        /// The capability domain with multiple providers
        domain: String,
    },
}

impl CapabilityDiscovery {
    /// Create a new capability discovery service
    #[must_use]
    pub fn new(backend: Box<dyn DiscoveryBackend>) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            backend,
        }
    }

    /// Discover a primal providing a capability
    ///
    /// Returns the FIRST healthy primal found.
    /// For load balancing, use `discover_all()`.
    pub async fn discover_one(
        &self,
        query: &CapabilityQuery,
    ) -> Result<PrimalEndpoint, DiscoveryError> {
        // Check cache first
        let cache_key = format!("{}:{:?}", query.domain, query.operation);
        {
            let cache = self.cache.read().await;
            if let Some(endpoint) = cache.get(&cache_key)
                && endpoint.health == PrimalHealth::Healthy
            {
                return Ok(endpoint.clone());
            }
        }

        // Query backend
        let results = self.backend.query(query).await?;

        // Filter for healthy primals
        let healthy: Vec<_> = results
            .into_iter()
            .filter(|p| p.health == PrimalHealth::Healthy)
            .collect();

        if healthy.is_empty() {
            return Err(DiscoveryError::CapabilityNotFound {
                domain: query.domain.clone(),
            });
        }

        // Cache and return first healthy result
        let endpoint = healthy[0].clone();
        self.cache.write().await.insert(cache_key, endpoint.clone());
        Ok(endpoint)
    }

    /// Discover all primals providing a capability
    pub async fn discover_all(
        &self,
        query: &CapabilityQuery,
    ) -> Result<Vec<PrimalEndpoint>, DiscoveryError> {
        let results = self.backend.query(query).await?;

        if results.is_empty() {
            return Err(DiscoveryError::CapabilityNotFound {
                domain: query.domain.clone(),
            });
        }

        Ok(results)
    }

    /// Clear discovery cache (force refresh)
    pub async fn invalidate_cache(&self) {
        self.cache.write().await.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_matching() {
        let cap = Capability::new("crypto").with_operation("encrypt");

        // Exact match
        let query = CapabilityQuery::new("crypto").with_operation("encrypt");
        assert!(cap.matches(&query));

        // Domain-only match
        let query = CapabilityQuery::new("crypto");
        assert!(cap.matches(&query));

        // Wrong domain
        let query = CapabilityQuery::new("display");
        assert!(!cap.matches(&query));

        // Wrong operation
        let query = CapabilityQuery::new("crypto").with_operation("decrypt");
        assert!(!cap.matches(&query));
    }

    #[test]
    fn test_capability_query_builder() {
        let query = CapabilityQuery::new("display").with_operation("render");
        assert_eq!(query.domain, "display");
        assert_eq!(query.operation, Some("render".to_string()));
    }
}
