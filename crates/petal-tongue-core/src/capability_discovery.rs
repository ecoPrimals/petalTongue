// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::manual_async_fn)] // explicit `impl Future + Send` on `DiscoveryBackend`
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

/// Parse "major.minor.patch" into (major, minor, patch). Handles partial versions.
pub(crate) fn parse_version(s: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = s.split('.').collect();
    let major = parts.first()?.trim().parse().ok()?;
    let minor = parts
        .get(1)
        .and_then(|p| p.trim().parse().ok())
        .unwrap_or(0);
    let patch = parts
        .get(2)
        .and_then(|p| p.trim().parse().ok())
        .unwrap_or(0);
    Some((major, minor, patch))
}

/// Check if capability version satisfies the required version.
/// Compatible if major versions match and capability >= required.
pub(crate) fn version_compatible(cap_version: &str, required: &str) -> bool {
    let Some((cap_maj, cap_min, cap_patch)) = parse_version(cap_version) else {
        return false;
    };
    let Some((req_maj, req_min, req_patch)) = parse_version(required) else {
        return true; // Unparseable requirement: allow
    };
    if cap_maj != req_maj {
        return false;
    }
    (cap_min, cap_patch) >= (req_min, req_patch)
}

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

        // Version compatibility: major must match, capability version must be >= required
        if let Some(ref version_req) = query.version_req {
            return version_compatible(&self.version, version_req);
        }

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

    /// Query for minimum version (semver-compatible)
    #[must_use]
    pub fn with_version(mut self, version_req: impl Into<String>) -> Self {
        self.version_req = Some(version_req.into());
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
pub struct CapabilityDiscovery<B: DiscoveryBackend> {
    /// Cache of discovered primals
    cache: Arc<RwLock<HashMap<String, PrimalEndpoint>>>,
    /// Discovery backend (biomeOS, mDNS, etc.)
    backend: B,
}

/// Discovery backend trait
///
/// Different implementations: biomeOS (primary), mDNS (fallback), static config
pub trait DiscoveryBackend: Send + Sync {
    /// Query for primals providing a capability
    fn query(
        &self,
        query: &CapabilityQuery,
    ) -> impl std::future::Future<Output = Result<Vec<PrimalEndpoint>, DiscoveryError>> + Send;

    /// Subscribe to capability changes (real-time updates)
    fn subscribe(
        &self,
        query: &CapabilityQuery,
    ) -> impl std::future::Future<Output = Result<(), DiscoveryError>> + Send;
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

impl<B: DiscoveryBackend> CapabilityDiscovery<B> {
    /// Create a new capability discovery service
    #[must_use]
    pub fn new(backend: B) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            backend,
        }
    }

    /// Discover a primal providing a capability
    ///
    /// Returns the FIRST healthy primal found.
    /// For load balancing, use `discover_all()`.
    ///
    /// # Errors
    ///
    /// Returns an error if the discovery backend fails to query, or if no healthy
    /// primal provides the requested capability.
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
    ///
    /// # Errors
    ///
    /// Returns an error if the discovery backend fails to query, or if no primal
    /// provides the requested capability.
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
    fn test_version_compatible_same_major_higher_minor() {
        assert!(version_compatible("1.2.0", "1.1.0"));
        assert!(version_compatible("1.3.5", "1.2.0"));
    }

    #[test]
    fn test_version_compatible_different_major() {
        assert!(!version_compatible("1.0.0", "2.0.0"));
        assert!(!version_compatible("2.0.0", "1.0.0"));
    }

    #[test]
    fn test_version_compatible_same_version() {
        assert!(version_compatible("1.2.3", "1.2.3"));
    }

    #[test]
    fn test_version_compatible_lower_minor_fails() {
        assert!(!version_compatible("1.1.0", "1.2.0"));
    }

    #[test]
    fn test_parse_version() {
        assert_eq!(parse_version("1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_version("1.0"), Some((1, 0, 0)));
        assert_eq!(parse_version("2"), Some((2, 0, 0)));
        assert_eq!(parse_version("invalid"), None);
    }

    /// Mock discovery backend for testing
    struct MockDiscoveryBackend {
        results: Vec<PrimalEndpoint>,
    }

    impl DiscoveryBackend for MockDiscoveryBackend {
        fn query(
            &self,
            _query: &CapabilityQuery,
        ) -> impl std::future::Future<Output = Result<Vec<PrimalEndpoint>, DiscoveryError>> + Send
        {
            let results = self.results.clone();
            async move { Ok(results) }
        }

        fn subscribe(
            &self,
            _query: &CapabilityQuery,
        ) -> impl std::future::Future<Output = Result<(), DiscoveryError>> + Send {
            async { Ok(()) }
        }
    }

    #[tokio::test]
    async fn test_discover_one_with_mock_backend() {
        let endpoint = PrimalEndpoint {
            id: "test-primal".to_string(),
            capabilities: vec![Capability::new("display")],
            endpoints: PrimalEndpoints {
                tarpc: Some("tarpc://test".to_string()),
                jsonrpc: None,
                https: None,
            },
            health: PrimalHealth::Healthy,
        };
        let backend = MockDiscoveryBackend {
            results: vec![endpoint],
        };
        let discovery = CapabilityDiscovery::new(backend);
        let query = CapabilityQuery::new("display");

        let result = discovery.discover_one(&query).await.unwrap();
        assert_eq!(result.id, "test-primal");
        assert_eq!(result.health, PrimalHealth::Healthy);
    }

    #[tokio::test]
    async fn test_discover_one_empty_returns_error() {
        let backend = MockDiscoveryBackend { results: vec![] };
        let discovery = CapabilityDiscovery::new(backend);
        let query = CapabilityQuery::new("nonexistent");

        let result = discovery.discover_one(&query).await;
        assert!(matches!(
            result,
            Err(DiscoveryError::CapabilityNotFound { domain }) if domain == "nonexistent"
        ));
    }

    #[tokio::test]
    async fn test_discover_all_with_mock_backend() {
        let endpoints = vec![
            PrimalEndpoint {
                id: "primal-1".to_string(),
                capabilities: vec![Capability::new("storage")],
                endpoints: PrimalEndpoints {
                    tarpc: None,
                    jsonrpc: None,
                    https: None,
                },
                health: PrimalHealth::Healthy,
            },
            PrimalEndpoint {
                id: "primal-2".to_string(),
                capabilities: vec![Capability::new("storage")],
                endpoints: PrimalEndpoints {
                    tarpc: None,
                    jsonrpc: None,
                    https: None,
                },
                health: PrimalHealth::Degraded,
            },
        ];
        let backend = MockDiscoveryBackend { results: endpoints };
        let discovery = CapabilityDiscovery::new(backend);
        let query = CapabilityQuery::new("storage");

        let result = discovery.discover_all(&query).await.unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_discover_all_empty_returns_error() {
        let backend = MockDiscoveryBackend { results: vec![] };
        let discovery = CapabilityDiscovery::new(backend);
        let query = CapabilityQuery::new("empty");

        let result = discovery.discover_all(&query).await;
        assert!(matches!(
            result,
            Err(DiscoveryError::CapabilityNotFound { domain }) if domain == "empty"
        ));
    }

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

    #[test]
    fn test_capability_version_matching() {
        let cap = Capability::new("crypto")
            .with_operation("encrypt")
            .with_version("1.2.3");

        // Same version
        let query = CapabilityQuery::new("crypto")
            .with_operation("encrypt")
            .with_version("1.2.3");
        assert!(cap.matches(&query));

        // Lower required version
        let query = CapabilityQuery::new("crypto")
            .with_operation("encrypt")
            .with_version("1.0.0");
        assert!(cap.matches(&query));

        // Higher required version
        let query = CapabilityQuery::new("crypto")
            .with_operation("encrypt")
            .with_version("1.3.0");
        assert!(!cap.matches(&query));

        // Different major version
        let query = CapabilityQuery::new("crypto")
            .with_operation("encrypt")
            .with_version("2.0.0");
        assert!(!cap.matches(&query));
    }
}
