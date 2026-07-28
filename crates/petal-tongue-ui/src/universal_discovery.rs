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
use petal_tongue_discovery::{MdnsVisualizationProvider, VisualizationDataProvider};
use petal_tongue_ipc::JsonRpcClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{debug, info, warn};

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
                protocol: "auto".to_owned(), // Auto-detect
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
    async fn discover_via_config(&self, capability: &str) -> Result<Vec<DiscoveredService>> {
        debug!("Checking config file for capability: {capability}");

        let mut services = Vec::new();

        for path in discovery_config_paths() {
            if !path.is_file() {
                continue;
            }

            let content = match tokio::fs::read_to_string(&path).await {
                Ok(content) => content,
                Err(e) => {
                    warn!("Failed to read discovery config {}: {e}", path.display());
                    continue;
                }
            };

            match parse_config_services(&content) {
                Ok(entries) => {
                    info!(
                        "Loaded {} service(s) from {}",
                        entries.len(),
                        path.display()
                    );
                    for entry in entries {
                        if entry.capability == capability {
                            services.push(DiscoveredService {
                                id: entry.name.clone(),
                                capabilities: vec![entry.capability.clone()],
                                endpoint: entry.endpoint.clone(),
                                protocol: entry
                                    .protocol
                                    .clone()
                                    .unwrap_or_else(|| "auto".to_owned()),
                                metadata: HashMap::new(),
                            });
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to parse discovery config {}: {e}", path.display());
                }
            }
        }

        Ok(services)
    }

    /// Discover via Unix socket probing (AGNOSTIC)
    ///
    /// Probes DH-1 socket search dirs + common locations WITHOUT assuming names:
    /// - `$BIOMEOS_SOCKET_DIR`/*.sock
    /// - `$XDG_RUNTIME_DIR/biomeos`/*.sock
    /// - /var/run/*.sock
    /// - ~/.local/share/*/sockets/*
    async fn discover_via_unix_socket(&self, capability: &str) -> Result<Vec<DiscoveredService>> {
        debug!("Probing Unix sockets for capability: {}", capability);

        let socket_paths = petal_tongue_core::constants::socket_search_dirs();

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
    async fn discover_via_mdns(&self, capability: &str) -> Result<Vec<DiscoveredService>> {
        debug!("Querying mDNS for capability: {}", capability);

        let service_type = mdns_service_type_for_capability(capability);

        match MdnsVisualizationProvider::discover_for_service(&service_type).await {
            Ok(providers) => {
                let services = providers
                    .into_iter()
                    .map(|provider| {
                        let metadata = provider.get_metadata();
                        let mut capabilities = metadata.capabilities;
                        if !capabilities
                            .iter()
                            .any(|cap| capability_matches(cap, capability))
                        {
                            capabilities.push(capability.to_owned());
                        }
                        DiscoveredService {
                            id: format!("mdns-{}", metadata.name),
                            capabilities,
                            endpoint: metadata.endpoint,
                            protocol: metadata.protocol,
                            metadata: HashMap::new(),
                        }
                    })
                    .collect();
                Ok(services)
            }
            Err(e) => {
                warn!("mDNS discovery failed for capability '{capability}': {e}");
                Ok(Vec::new())
            }
        }
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
    async fn query_unix_socket(
        &self,
        endpoint: &str,
        capability: &str,
    ) -> Result<Vec<DiscoveredService>> {
        debug!("Querying Unix socket: {}", endpoint);

        let socket_path = endpoint.strip_prefix("unix://").unwrap_or(endpoint);
        if socket_path.is_empty() {
            return Ok(Vec::new());
        }

        let client = match JsonRpcClient::with_timeout(socket_path, Duration::from_millis(500)) {
            Ok(client) => client,
            Err(e) => {
                debug!("Invalid Unix socket path {socket_path}: {e}");
                return Ok(Vec::new());
            }
        };

        let response = match client
            .call("capabilities.list", serde_json::json!({}))
            .await
        {
            Ok(result) => result,
            Err(e) => {
                debug!("capabilities.list failed on {socket_path}: {e}");
                return Ok(Vec::new());
            }
        };

        let capabilities = extract_capabilities_from_rpc_result(&response);
        if !service_matches_capability(&capabilities, capability) {
            return Ok(Vec::new());
        }

        let id = response
            .get("primal")
            .or_else(|| response.get("name"))
            .and_then(serde_json::Value::as_str)
            .map_or_else(
                || {
                    Path::new(socket_path)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unix-socket")
                        .to_owned()
                },
                str::to_owned,
            );

        Ok(vec![DiscoveredService {
            id: format!("uds-{id}"),
            capabilities,
            endpoint: endpoint.to_owned(),
            protocol: "json-rpc".to_owned(),
            metadata: HashMap::new(),
        }])
    }
}

/// Operator-configured discovery entries from `discovery.toml`.
#[derive(Debug, Deserialize)]
struct DiscoveryConfigFile {
    #[serde(default)]
    services: Vec<ConfigServiceEntry>,
}

#[derive(Debug, Deserialize)]
struct ConfigServiceEntry {
    name: String,
    capability: String,
    endpoint: String,
    protocol: Option<String>,
}

/// Candidate paths for operator discovery configuration.
fn discovery_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(config_home) = petal_tongue_core::platform_dirs::config_dir() {
        paths.push(config_home.join("petaltongue/discovery.toml"));
    }

    paths.push(PathBuf::from("config/discovery.toml"));
    paths
}

fn parse_config_services(
    content: &str,
) -> std::result::Result<Vec<ConfigServiceEntry>, toml::de::Error> {
    let config: DiscoveryConfigFile = toml::from_str(content)?;
    Ok(config.services)
}

/// Map a capability name to an mDNS DNS-SD service type.
fn mdns_service_type_for_capability(capability: &str) -> String {
    match capability {
        "visualization" | "visualization-provider" => {
            "_visualization-provider._tcp.local".to_owned()
        }
        _ => format!("_{capability}._tcp.local"),
    }
}

/// Whether a service advertises the requested capability.
fn service_matches_capability(advertised: &[String], requested: &str) -> bool {
    !advertised.is_empty()
        && advertised
            .iter()
            .any(|cap| capability_matches(cap, requested))
}

fn capability_matches(advertised: &str, requested: &str) -> bool {
    advertised == requested
        || advertised.contains(requested)
        || requested.contains(advertised)
        || advertised.replace('.', "-") == requested
        || advertised.replace('-', ".") == requested
}

/// Extract capability strings from a `capabilities.list` JSON-RPC result.
fn extract_capabilities_from_rpc_result(result: &serde_json::Value) -> Vec<String> {
    if let Some(array) = result
        .get("capabilities")
        .and_then(serde_json::Value::as_array)
    {
        return array
            .iter()
            .filter_map(serde_json::Value::as_str)
            .map(str::to_owned)
            .collect();
    }

    if let Some(array) = result.get("methods").and_then(serde_json::Value::as_array) {
        return array
            .iter()
            .filter_map(serde_json::Value::as_str)
            .map(str::to_owned)
            .collect();
    }

    if let Some(array) = result.as_array() {
        return array
            .iter()
            .filter_map(serde_json::Value::as_str)
            .map(str::to_owned)
            .collect();
    }

    Vec::new()
}

impl Default for UniversalDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "universal_discovery_tests.rs"]
mod tests;
