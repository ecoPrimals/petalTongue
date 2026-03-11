// SPDX-License-Identifier: AGPL-3.0-only
//! biomeOS provider - capability-based discovery and integration.
//!
//! Discovers device management by capability, not by name (TRUE PRIMAL).

use anyhow::Result;
use petal_tongue_core::constants;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::events::{BiomeOSEvent, EventStream};
use super::types::{Device, NicheTemplate, Primal};

/// Cached data for graceful degradation
#[derive(Debug, Clone, Default)]
pub(crate) struct ProviderCache {
    /// Cached device list
    pub devices: Vec<Device>,
    /// Cached primal list
    pub primals: Vec<Primal>,
    /// Cached niche templates
    pub niche_templates: Vec<NicheTemplate>,
    /// Last successful update timestamp
    pub last_update: Option<std::time::Instant>,
}

/// biomeOS provider - discovers and integrates with biomeOS for device management
///
/// # Discovery
///
/// Discovers biomeOS by querying for the "device.management" capability.
/// Does NOT hardcode "biomeOS" - any primal with this capability will work!
pub struct BiomeOSProvider {
    /// Connection to biomeOS (or any primal with device.management capability)
    endpoint: String,
    /// Cached data for offline mode
    cache: Arc<RwLock<ProviderCache>>,
    /// Event stream subscription (for real-time updates)
    event_stream: Arc<RwLock<Option<EventStream>>>,
}

impl BiomeOSProvider {
    /// Discover biomeOS by capability (TRUE PRIMAL!)
    ///
    /// Queries all discovered primals for "device.management" capability.
    /// Returns `None` if no provider found (graceful degradation).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use petal_tongue_ui::biomeos_integration::BiomeOSProvider;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// if let Some(provider) = BiomeOSProvider::discover().await? {
    ///     let devices = provider.get_devices().await?;
    ///     println!("Discovered {} devices", devices.len());
    /// } else {
    ///     println!("No device management provider found - using mock data");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn discover() -> Result<Option<Self>> {
        info!("🔍 Discovering device management provider (capability-based)...");

        let capability = "device.management";

        // Try capability discovery via biomeOS Neural API
        use petal_tongue_core::{
            biomeos_discovery::BiomeOsBackend,
            capability_discovery::{
                CapabilityQuery, DiscoveryBackend, PrimalEndpoint, PrimalHealth,
            },
        };
        if let Ok(backend) = BiomeOsBackend::from_env() {
            let query = CapabilityQuery::new("device").with_operation("management");
            match backend.query(&query).await {
                Ok(endpoints) => {
                    let endpoints: Vec<PrimalEndpoint> = endpoints;
                    if let Some(ep) = endpoints
                        .into_iter()
                        .find(|e| e.health != PrimalHealth::Unavailable)
                    {
                        let endpoint = ep
                            .endpoints
                            .jsonrpc
                            .or_else(|| {
                                ep.endpoints.tarpc.as_ref().map(|t: &String| {
                                    t.strip_prefix("tarpc://unix:")
                                        .map_or_else(|| t.clone(), |s: &str| s.to_string())
                                })
                            })
                            .unwrap_or_else(|| ep.id.clone());
                        info!("✅ Found device management provider at: {}", endpoint);
                        let provider = Self {
                            endpoint,
                            cache: Arc::new(RwLock::new(ProviderCache::default())),
                            event_stream: Arc::new(RwLock::new(None)),
                        };
                        if provider.health_check().await.is_ok() {
                            info!("✅ Device management provider healthy");
                            return Ok(Some(provider));
                        }
                        warn!("⚠️ Device management provider found but unhealthy");
                    }
                }
                Err(e) => {
                    debug!("Capability query failed: {}", e);
                }
            }
        }

        // Fallback: environment variable hint
        if let Ok(endpoint) = std::env::var("DEVICE_MANAGEMENT_ENDPOINT") {
            info!("✅ Found device management provider at: {}", endpoint);
            let provider = Self {
                endpoint,
                cache: Arc::new(RwLock::new(ProviderCache::default())),
                event_stream: Arc::new(RwLock::new(None)),
            };
            if provider.health_check().await.is_ok() {
                info!("✅ Device management provider healthy");
                return Ok(Some(provider));
            }
            warn!("⚠️ Device management provider found but unhealthy");
        }

        debug!(
            "No device management provider found for capability: {}",
            capability
        );
        Ok(None)
    }

    /// Get list of discovered devices
    pub async fn get_devices(&self) -> Result<Vec<Device>> {
        debug!("Fetching devices from device management provider");

        // JSON-RPC call via capability-discovered endpoint
        let response = self
            .call_jsonrpc("devices.list", serde_json::json!({}))
            .await?;

        // Parse device list
        let devices: Vec<Device> = serde_json::from_value(response)
            .map_err(|e| anyhow::anyhow!("Failed to parse devices response: {e}"))?;

        // Update cache for offline fallback
        let mut cache = self.cache.write().await;
        cache.devices = devices.clone();
        cache.last_update = Some(std::time::Instant::now());

        debug!("✅ Fetched {} devices", devices.len());
        Ok(devices)
    }

    /// Get list of discovered primals
    pub async fn get_primals_extended(&self) -> Result<Vec<Primal>> {
        debug!("Fetching primals from device management provider");

        // JSON-RPC call via capability-discovered endpoint
        let response = self
            .call_jsonrpc("primals.list_extended", serde_json::json!({}))
            .await?;

        // Parse primal list
        let primals: Vec<Primal> = serde_json::from_value(response)
            .map_err(|e| anyhow::anyhow!("Failed to parse primals response: {e}"))?;

        // Update cache for offline fallback
        let mut cache = self.cache.write().await;
        cache.primals = primals.clone();
        cache.last_update = Some(std::time::Instant::now());

        debug!("✅ Fetched {} primals", primals.len());
        Ok(primals)
    }

    /// Get niche templates
    pub async fn get_niche_templates(&self) -> Result<Vec<NicheTemplate>> {
        debug!("Fetching niche templates from device management provider");

        // JSON-RPC call via capability-discovered endpoint
        let response = self
            .call_jsonrpc("niches.list_templates", serde_json::json!({}))
            .await?;

        // Parse template list
        let templates: Vec<NicheTemplate> = serde_json::from_value(response)
            .map_err(|e| anyhow::anyhow!("Failed to parse niche templates: {e}"))?;

        // Update cache for offline fallback
        let mut cache = self.cache.write().await;
        cache.niche_templates = templates.clone();
        cache.last_update = Some(std::time::Instant::now());

        debug!("✅ Fetched {} niche templates", templates.len());
        Ok(templates)
    }

    /// Assign device to primal
    pub async fn assign_device(&self, device_id: &str, primal_id: &str) -> Result<()> {
        info!("Assigning device {} to primal {}", device_id, primal_id);

        // JSON-RPC call via capability-discovered endpoint
        let params = serde_json::json!({
            "device_id": device_id,
            "primal_id": primal_id,
        });

        self.call_jsonrpc("devices.assign", params).await?;

        info!("✅ Device assigned successfully");
        Ok(())
    }

    /// Deploy niche
    pub async fn deploy_niche(&self, niche: &NicheTemplate) -> Result<String> {
        info!("Deploying niche: {}", niche.name);

        // JSON-RPC call via capability-discovered endpoint
        let params = serde_json::json!({
            "name": niche.name,
            "description": niche.description,
            "required_primals": niche.required_primals,
            "optional_primals": niche.optional_primals,
            "metadata": niche.metadata,
        });

        let response = self.call_jsonrpc("niches.deploy", params).await?;

        // Extract niche ID from response
        let niche_id: String = serde_json::from_value(
            response
                .get("niche_id")
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("No niche_id in response"))?,
        )
        .map_err(|e| anyhow::anyhow!("Failed to parse niche_id: {e}"))?;

        info!("✅ Deployed niche: {}", niche_id);
        Ok(niche_id)
    }

    /// Subscribe to real-time events via WebSocket
    ///
    /// Establishes WebSocket connection for receiving real-time events:
    /// - device.added, device.removed
    /// - primal.status
    /// - niche.deployed
    pub async fn subscribe_events(&self) -> Result<()> {
        info!("Subscribing to real-time events from provider");

        // JSON-RPC call to subscribe (registers interest)
        let params = serde_json::json!({
            "events": ["device.added", "device.removed", "primal.status", "niche.deployed"]
        });

        self.call_jsonrpc("events.subscribe", params).await?;

        // Attempt WebSocket connection for real-time event stream
        // Derive WebSocket endpoint from Unix socket path
        let ws_endpoint = self.derive_websocket_endpoint();

        let mut event_stream_guard = self.event_stream.write().await;

        if event_stream_guard.is_none() {
            *event_stream_guard = Some(EventStream::new());
        }

        if let Some(ref mut event_stream) = *event_stream_guard {
            match event_stream.connect(&ws_endpoint).await {
                Ok(()) => {
                    info!("✅ Subscribed to real-time events (WebSocket)");
                }
                Err(e) => {
                    warn!(
                        "⚠️ WebSocket connection failed (falling back to polling): {}",
                        e
                    );
                    // Polling fallback is acceptable - subscription still registered
                }
            }
        }

        Ok(())
    }

    /// Subscribe to events with a callback for real-time handling
    ///
    /// The callback will be invoked for each event received via WebSocket.
    pub async fn subscribe_events_with_callback<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(BiomeOSEvent) + Send + Sync + 'static,
    {
        // First, establish the subscription
        self.subscribe_events().await?;

        // Set callback on event stream
        let mut event_stream_guard = self.event_stream.write().await;

        if let Some(ref mut event_stream) = *event_stream_guard {
            event_stream.set_callback(callback);
            info!("✅ Event callback registered");
        }

        Ok(())
    }

    /// Derive WebSocket endpoint from Unix socket path
    ///
    /// Attempts to discover WebSocket endpoint via:
    /// 1. `BIOMEOS_WS_ENDPOINT` environment variable
    /// 2. Standard port derivation from socket path
    fn derive_websocket_endpoint(&self) -> String {
        if let Ok(ws_endpoint) = std::env::var("BIOMEOS_WS_ENDPOINT") {
            return ws_endpoint;
        }
        constants::default_biomeos_ws_events_url()
    }

    /// Helper: Call JSON-RPC method on device management provider
    async fn call_jsonrpc(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::UnixStream;

        // Connect to provider endpoint
        let mut stream = UnixStream::connect(&self.endpoint)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to device management provider: {e}"))?;

        // Build JSON-RPC 2.0 request
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1,
        });

        // Send request (line-delimited JSON-RPC)
        let request_str = serde_json::to_string(&request)?;
        stream
            .write_all(format!("{request_str}\n").as_bytes())
            .await?;
        stream.flush().await?;

        // Read response
        let (reader, _) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut response_line = String::new();

        reader
            .read_line(&mut response_line)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read response: {e}"))?;

        // Parse JSON-RPC response
        let response: serde_json::Value = serde_json::from_str(&response_line)
            .map_err(|e| anyhow::anyhow!("Failed to parse JSON-RPC response: {e}"))?;

        // Check for error
        if let Some(error) = response.get("error") {
            return Err(anyhow::anyhow!("JSON-RPC error: {error}"));
        }

        // Extract result
        response
            .get("result")
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No result in JSON-RPC response"))
    }

    /// Health check for provider connection
    async fn health_check(&self) -> Result<()> {
        debug!("Performing health check on device management provider");

        // Simple ping to verify connection
        let params = serde_json::json!({});
        self.call_jsonrpc("health.ping", params).await?;

        debug!("✅ Provider health check passed");
        Ok(())
    }

    /// Health check returning status string (for VisualizationDataProvider)
    pub(super) async fn health_check_jsonrpc(&self) -> Result<String> {
        // Try health.check first (semantic method)
        let params = serde_json::json!({});
        let result = self.call_jsonrpc("health.check", params).await;

        if let Ok(res) = result {
            if let Some(status) = res.get("status").and_then(serde_json::Value::as_str) {
                return Ok(status.to_string());
            }
            if let Some(healthy) = res.get("healthy").and_then(serde_json::Value::as_bool) {
                return Ok(if healthy { "healthy" } else { "unhealthy" }.to_string());
            }
            Ok("healthy".to_string())
        } else {
            // Fallback to health.ping
            self.call_jsonrpc("health.ping", serde_json::json!({}))
                .await?;
            Ok("healthy".to_string())
        }
    }

    /// Expose endpoint for VisualizationDataProvider trait impl
    pub(super) fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

#[cfg(test)]
impl BiomeOSProvider {
    /// Create provider for testing (bypasses discovery)
    pub fn new_for_test(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            cache: Arc::new(RwLock::new(ProviderCache::default())),
            event_stream: Arc::new(RwLock::new(None)),
        }
    }

    /// Expose derive_websocket_endpoint for testing
    pub fn derive_websocket_endpoint_for_test(&self) -> String {
        self.derive_websocket_endpoint()
    }
}

#[cfg(test)]
mod provider_tests {
    use super::*;

    #[test]
    fn provider_cache_default() {
        let cache = ProviderCache::default();
        assert!(cache.devices.is_empty());
        assert!(cache.primals.is_empty());
        assert!(cache.niche_templates.is_empty());
        assert!(cache.last_update.is_none());
    }

    #[test]
    fn provider_new_for_test() {
        let provider = BiomeOSProvider::new_for_test("/tmp/test.sock");
        assert_eq!(provider.endpoint(), "/tmp/test.sock");
    }

    #[test]
    fn derive_websocket_endpoint_format() {
        let provider = BiomeOSProvider::new_for_test("dummy");
        let ep = provider.derive_websocket_endpoint_for_test();
        assert!(ep.starts_with("ws://"));
        assert!(ep.ends_with("/events"));
        assert!(
            ep.contains("127.0.0.1") || ep.contains("localhost"),
            "WebSocket URL should use loopback host, got: {ep}"
        );
    }

    #[test]
    fn provider_cache_clone() {
        let cache = ProviderCache::default();
        let cloned = cache.clone();
        assert!(cloned.devices.is_empty());
        assert!(cloned.primals.is_empty());
    }
}
