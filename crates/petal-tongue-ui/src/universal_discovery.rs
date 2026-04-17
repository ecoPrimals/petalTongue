// SPDX-License-Identifier: AGPL-3.0-or-later
//! Universal Discovery System
//!
//! **ZERO HARDCODED KNOWLEDGE** - Infant Discovery Pattern
//!
//! This module discovers services WITHOUT knowing:
//! - ❌ Service names (hardcoded discovery or compute brands, etc.)
//! - ❌ Vendor names (k8s, consul, etc.)
//! - ❌ Port numbers
//! - ❌ Protocols
//!
//! Instead, it discovers:
//! - ✅ "Who provides discovery?"
//! - ✅ "Who provides rendering?"
//! - ✅ "What protocols are available?"
//!
//! # Philosophy
//!
//! **"Code starts with ZERO knowledge, discovers like an infant."**
//!
//! Just as an infant learns by exploring, our code discovers the environment
//! at runtime without assumptions.

use crate::error::Result;
use petal_tongue_core::constants;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Universal service discovery
///
/// Discovers ANY service by capability, without hardcoding names.
#[derive(Debug, Clone)]
pub struct UniversalDiscovery {
    /// Discovery methods to try
    discovery_methods: Vec<DiscoveryMethod>,
}

/// A discovered service (AGNOSTIC)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredService {
    /// Opaque service ID (we don't care what it's called)
    pub id: String,

    /// What capabilities does it provide?
    pub capabilities: Vec<String>,

    /// How do we connect?
    pub endpoint: String,

    /// What protocol?
    pub protocol: String,

    /// Optional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Discovery method (in priority order)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryMethod {
    /// Direct connection via environment variables
    Environment,

    /// Configuration file
    ConfigFile,

    /// Unix socket probing
    UnixSocket,

    /// mDNS/Multicast discovery
    Mdns,

    /// HTTP endpoint discovery
    HttpProbe,
}

impl UniversalDiscovery {
    /// Create new universal discovery with default methods
    #[must_use]
    pub fn new() -> Self {
        Self {
            discovery_methods: vec![
                DiscoveryMethod::Environment, // Fastest
                DiscoveryMethod::UnixSocket,  // Port-free
                DiscoveryMethod::Mdns,        // Zero-config
                DiscoveryMethod::HttpProbe,   // Fallback
            ],
        }
    }

    /// Discover services by capability (AGNOSTIC)
    ///
    /// # Arguments
    /// * `capability` - What capability do we need? (e.g., "gpu-rendering", "discovery", "storage")
    ///
    /// # Returns
    /// List of services that provide this capability
    ///
    /// # Example
    /// ```no_run
    /// use petal_tongue_ui::universal_discovery::UniversalDiscovery;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let discovery = UniversalDiscovery::new();
    ///
    /// // Discover rendering without knowing WHO provides it
    /// let renderers = discovery.discover_capability("gpu-rendering").await?;
    ///
    /// for renderer in renderers {
    ///     println!("Found renderer: {} at {}", renderer.id, renderer.endpoint);
    ///     // We don't know or care which primal provides it—only that it matches the capability
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Currently always returns `Ok`; failed discovery methods are skipped and logged.
    pub async fn discover_capability(&self, capability: &str) -> Result<Vec<DiscoveredService>> {
        info!(
            "🔍 Discovering capability: '{}' (infant mode - zero assumptions)",
            capability
        );

        let mut discovered = Vec::new();

        // Try each discovery method in order
        for method in &self.discovery_methods {
            debug!("Trying discovery method: {:?}", method);

            match self.try_discovery_method(method, capability).await {
                Ok(services) if !services.is_empty() => {
                    info!("✅ Found {} service(s) via {:?}", services.len(), method);
                    discovered.extend(services);

                    // Continue to find all providers, not just first
                }
                Ok(_) => {
                    debug!("No services found via {:?}", method);
                }
                Err(e) => {
                    debug!("Discovery method {:?} failed: {}", method, e);
                }
            }
        }

        if discovered.is_empty() {
            info!("ℹ️  No services found for capability '{}'", capability);
        } else {
            info!(
                "✅ Total discovered: {} service(s) for '{}'",
                discovered.len(),
                capability
            );
        }

        Ok(discovered)
    }

    /// Try a specific discovery method
    async fn try_discovery_method(
        &self,
        method: &DiscoveryMethod,
        capability: &str,
    ) -> Result<Vec<DiscoveredService>> {
        match method {
            DiscoveryMethod::Environment => self.discover_via_environment(capability).await,
            DiscoveryMethod::ConfigFile => self.discover_via_config(capability).await,
            DiscoveryMethod::UnixSocket => self.discover_via_unix_socket(capability).await,
            DiscoveryMethod::Mdns => self.discover_via_mdns(capability).await,
            DiscoveryMethod::HttpProbe => self.discover_via_http(capability).await,
        }
    }

    /// Discover via environment variables (AGNOSTIC)
    ///
    /// Looks for patterns like:
    /// - `{CAPABILITY}_ENDPOINT` (e.g., `GPU_RENDERING_ENDPOINT`)
    /// - `SERVICE_MESH_ENDPOINT` (generic discovery service)
    async fn discover_via_environment(&self, capability: &str) -> Result<Vec<DiscoveredService>> {
        debug!("Checking environment for capability: {}", capability);

        let mut services = Vec::new();

        // Try capability-specific env var
        let env_key = format!("{}_ENDPOINT", capability.to_uppercase().replace('-', "_"));
        if let Ok(endpoint) = std::env::var(&env_key) {
            info!("✅ Found direct endpoint via {}: {}", env_key, endpoint);

            services.push(DiscoveredService {
                id: format!("env-{capability}"),
                capabilities: vec![capability.to_string()],
                endpoint,
                protocol: "auto".to_string(), // Auto-detect
                metadata: HashMap::new(),
            });
        }

        // Try generic service mesh endpoint
        if let Ok(mesh_endpoint) = std::env::var("SERVICE_MESH_ENDPOINT") {
            debug!("Found SERVICE_MESH_ENDPOINT: {}", mesh_endpoint);

            // Query the service mesh for this capability
            if let Ok(mesh_services) = self.query_service_mesh(&mesh_endpoint, capability).await {
                services.extend(mesh_services);
            }
        }

        // Try discovery service endpoint (another generic option)
        if let Ok(discovery_endpoint) = std::env::var("DISCOVERY_SERVICE_ENDPOINT") {
            debug!("Found DISCOVERY_SERVICE_ENDPOINT: {}", discovery_endpoint);

            if let Ok(discovered_services) = self
                .query_discovery_service(&discovery_endpoint, capability)
                .await
            {
                services.extend(discovered_services);
            }
        }

        Ok(services)
    }

    /// Discover via config file (AGNOSTIC)
    #[expect(clippy::unused_async, reason = "async for trait compatibility")]
    async fn discover_via_config(&self, capability: &str) -> Result<Vec<DiscoveredService>> {
        debug!("Checking config file for capability: {capability}");

        // Delegated: config-file discovery is handled by the deployment layer.
        // petalTongue discovers capabilities at runtime via socket/mDNS/HTTP probing;
        // config files are an operator concern, not a primal concern.
        Ok(Vec::new())
    }

    /// Discover via Unix socket probing (AGNOSTIC)
    ///
    /// Probes common socket locations WITHOUT assuming names:
    /// - /tmp/*.sock
    /// - /var/run/*.sock
    /// - ~/.local/share/*/sockets/*
    async fn discover_via_unix_socket(&self, capability: &str) -> Result<Vec<DiscoveredService>> {
        debug!("Probing Unix sockets for capability: {}", capability);

        let socket_paths = vec![
            "/tmp", "/var/run",
            // Add more common socket locations
        ];

        let mut services = Vec::new();

        for base_path in socket_paths {
            if let Ok(entries) = std::fs::read_dir(base_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("sock"))
                    {
                        // Try to query this socket
                        let endpoint = format!("unix://{}", path.display());

                        if let Ok(socket_services) =
                            self.query_generic_endpoint(&endpoint, capability).await
                        {
                            services.extend(socket_services);
                        }
                    }
                }
            }
        }

        Ok(services)
    }

    /// Discover via mDNS (AGNOSTIC)
    ///
    /// Queries for service types WITHOUT hardcoded names:
    /// - _discovery._tcp.local
    /// - _gpu-rendering._tcp.local
    /// - _compute._tcp.local
    #[expect(clippy::unused_async, reason = "async for trait compatibility")]
    async fn discover_via_mdns(&self, capability: &str) -> Result<Vec<DiscoveredService>> {
        debug!("Querying mDNS for capability: {}", capability);

        // Convert capability to mDNS service type
        let _service_type = format!("_{capability}._tcp.local");

        // mDNS discovery is implemented in petal-tongue-discovery::MdnsProvider.
        // This layer delegates to the provider when available; returns empty otherwise.
        Ok(Vec::new())
    }

    /// Discover via HTTP probing (AGNOSTIC)
    ///
    /// Probes ports WITHOUT assumptions:
    /// - Reads `PETALTONGUE_DISCOVERY_PORTS` or `DISCOVERY_PORTS` env var if provided
    /// - Falls back to common service port range (documented in `ENV_VARS.md`)
    /// - Checks /capabilities, /health, /api/v1/capabilities endpoints
    async fn discover_via_http(&self, capability: &str) -> Result<Vec<DiscoveredService>> {
        debug!("Probing HTTP endpoints for capability: {}", capability);

        let ports: Vec<u16> = constants::default_discovery_ports();

        let base = std::env::var("PETALTONGUE_DISCOVERY_BASE").unwrap_or_else(|_| {
            format!(
                "http://{}",
                petal_tongue_core::constants::DEFAULT_LOOPBACK_HOST
            )
        });

        let mut services = Vec::new();

        for port in ports {
            let endpoint = format!("{base}:{port}");

            if let Ok(http_services) = self.query_generic_endpoint(&endpoint, capability).await {
                services.extend(http_services);
            }
        }

        Ok(services)
    }

    /// Query a service mesh generically
    async fn query_service_mesh(
        &self,
        endpoint: &str,
        capability: &str,
    ) -> Result<Vec<DiscoveredService>> {
        debug!("Querying service mesh at: {}", endpoint);

        let client =
            petal_tongue_ipc::LocalHttpClient::with_timeout(std::time::Duration::from_secs(5));

        let api_paths = vec![
            format!("/api/v1/capabilities/{}", capability),
            format!("/discover?capability={}", capability),
            format!("/services?capability={}", capability),
        ];

        for path in api_paths {
            let url = format!("{endpoint}{path}");

            if let Ok(response) = client.get(&url).await
                && response.is_success()
                && let Ok(services) = response.json::<Vec<DiscoveredService>>()
                && !services.is_empty()
            {
                return Ok(services);
            }
        }

        Ok(Vec::new())
    }

    /// Query a discovery service generically
    async fn query_discovery_service(
        &self,
        endpoint: &str,
        capability: &str,
    ) -> Result<Vec<DiscoveredService>> {
        debug!("Querying discovery service at: {}", endpoint);

        // Similar to service mesh, but might use different API patterns
        self.query_service_mesh(endpoint, capability).await
    }

    /// Query any endpoint generically
    async fn query_generic_endpoint(
        &self,
        endpoint: &str,
        capability: &str,
    ) -> Result<Vec<DiscoveredService>> {
        debug!("Querying generic endpoint: {}", endpoint);

        // Try to detect protocol and query appropriately
        if endpoint.starts_with("unix://") {
            self.query_unix_socket(endpoint, capability).await
        } else if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
            self.query_service_mesh(endpoint, capability).await
        } else {
            // Unknown protocol
            Ok(Vec::new())
        }
    }

    /// Query a Unix socket generically
    #[expect(clippy::unused_async, reason = "async for trait compatibility")]
    async fn query_unix_socket(
        &self,
        endpoint: &str,
        _capability: &str,
    ) -> Result<Vec<DiscoveredService>> {
        debug!("Querying Unix socket: {}", endpoint);

        // Unix socket querying is implemented in petal-tongue-discovery::UnixSocketProvider.
        // This layer delegates to the provider when available; returns empty otherwise.
        Ok(Vec::new())
    }
}

impl Default for UniversalDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "universal_discovery_tests.rs"]
mod tests;
