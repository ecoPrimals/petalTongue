// SPDX-License-Identifier: AGPL-3.0-or-later
//! biomeOS provider - capability-based discovery and integration.
//!
//! Discovers device management by capability, not by name (TRUE PRIMAL).

use crate::error::{BiomeOsIntegrationError, Result};
use petal_tongue_core::constants;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::events::{BiomeOSEvent, EventStream};
use super::types::{Device, NicheTemplate, Primal};

#[must_use]
pub fn build_jsonrpc_request(
    method: &str,
    params: serde_json::Value,
    id: u64,
) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": id,
    })
}

#[must_use]
pub fn parse_jsonrpc_result(response: &serde_json::Value) -> Option<serde_json::Value> {
    response.get("result").cloned()
}

#[must_use]
pub fn parse_jsonrpc_error(response: &serde_json::Value) -> Option<serde_json::Value> {
    response.get("error").cloned()
}

#[must_use]
pub fn health_response_status(value: &serde_json::Value) -> String {
    value
        .get("status")
        .and_then(|s| s.as_str())
        .unwrap_or("unknown")
        .to_string()
}

#[must_use]
pub fn health_response_healthy(value: &serde_json::Value) -> Option<bool> {
    value.get("healthy").and_then(serde_json::Value::as_bool)
}

#[must_use]
pub fn build_assign_device_params(device_id: &str, primal_id: &str) -> serde_json::Value {
    serde_json::json!({
        "device_id": device_id,
        "primal_id": primal_id,
    })
}

#[must_use]
pub fn build_deploy_niche_params(niche: &NicheTemplate) -> serde_json::Value {
    serde_json::json!({
        "name": niche.name,
        "description": niche.description,
        "required_primals": niche.required_primals,
        "optional_primals": niche.optional_primals,
        "metadata": niche.metadata,
    })
}

#[must_use]
pub fn extract_niche_id_from_response(response: &serde_json::Value) -> Option<&serde_json::Value> {
    response.get("niche_id")
}

#[must_use]
pub fn build_subscribe_events_params() -> serde_json::Value {
    serde_json::json!({
        "events": ["device.added", "device.removed", "primal.status", "niche.deployed"]
    })
}

/// Cached data for graceful degradation
#[derive(Debug, Clone, Default)]
pub struct ProviderCache {
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
    ///
    /// # Errors
    ///
    /// Returns an error if capability discovery fails or the backend health check fails.
    pub async fn discover() -> Result<Option<Self>> {
        use petal_tongue_core::{
            biomeos_discovery::BiomeOsBackend,
            capability_discovery::{
                CapabilityQuery, DiscoveryBackend, PrimalEndpoint, PrimalHealth,
            },
        };

        info!("🔍 Discovering device management provider (capability-based)...");

        let capability = "device.management";

        // Try capability discovery via biomeOS Neural API
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
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON-RPC call fails or the response cannot be parsed.
    pub async fn get_devices(&self) -> Result<Vec<Device>> {
        debug!("Fetching devices from device management provider");

        // JSON-RPC call via capability-discovered endpoint
        let response = self
            .call_jsonrpc("device.list", serde_json::json!({}))
            .await?;

        // Parse device list
        let devices: Vec<Device> = serde_json::from_value(response)
            .map_err(|e| BiomeOsIntegrationError::ParseDevicesResponse(e.to_string()))?;

        // Update cache for offline fallback
        {
            let mut cache = self.cache.write().await;
            cache.devices = devices.clone();
            cache.last_update = Some(std::time::Instant::now());
        }

        debug!("✅ Fetched {} devices", devices.len());
        Ok(devices)
    }

    /// Get list of discovered primals
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON-RPC call fails or the response cannot be parsed.
    pub async fn get_primals_extended(&self) -> Result<Vec<Primal>> {
        debug!("Fetching primals from device management provider");

        // JSON-RPC call via capability-discovered endpoint
        let response = self
            .call_jsonrpc("primal.list", serde_json::json!({}))
            .await?;

        // Parse primal list
        let primals: Vec<Primal> = serde_json::from_value(response)
            .map_err(|e| BiomeOsIntegrationError::ParsePrimalsResponse(e.to_string()))?;

        // Update cache for offline fallback
        {
            let mut cache = self.cache.write().await;
            cache.primals = primals.clone();
            cache.last_update = Some(std::time::Instant::now());
        }

        debug!("✅ Fetched {} primals", primals.len());
        Ok(primals)
    }

    /// Get niche templates
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON-RPC call fails or the response cannot be parsed.
    pub async fn get_niche_templates(&self) -> Result<Vec<NicheTemplate>> {
        debug!("Fetching niche templates from device management provider");

        // JSON-RPC call via capability-discovered endpoint
        let response = self
            .call_jsonrpc("niche.list_templates", serde_json::json!({}))
            .await?;

        // Parse template list
        let templates: Vec<NicheTemplate> = serde_json::from_value(response)
            .map_err(|e| BiomeOsIntegrationError::ParseNicheTemplates(e.to_string()))?;

        // Update cache for offline fallback
        {
            let mut cache = self.cache.write().await;
            cache.niche_templates = templates.clone();
            cache.last_update = Some(std::time::Instant::now());
        }

        debug!("✅ Fetched {} niche templates", templates.len());
        Ok(templates)
    }

    /// Assign device to primal
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON-RPC call fails.
    pub async fn assign_device(&self, device_id: &str, primal_id: &str) -> Result<()> {
        info!("Assigning device {} to primal {}", device_id, primal_id);

        self.call_jsonrpc(
            "device.assign",
            build_assign_device_params(device_id, primal_id),
        )
        .await?;

        info!("✅ Device assigned successfully");
        Ok(())
    }

    /// Deploy niche
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON-RPC call fails, the response lacks `niche_id`, or parsing fails.
    pub async fn deploy_niche(&self, niche: &NicheTemplate) -> Result<String> {
        info!("Deploying niche: {}", niche.name);

        let response = self
            .call_jsonrpc("niche.deploy", build_deploy_niche_params(niche))
            .await?;

        let niche_id: String = serde_json::from_value(
            extract_niche_id_from_response(&response)
                .cloned()
                .ok_or(BiomeOsIntegrationError::NoNicheId)?,
        )
        .map_err(|e| BiomeOsIntegrationError::ParseNicheId(e.to_string()))?;

        info!("✅ Deployed niche: {}", niche_id);
        Ok(niche_id)
    }

    /// Subscribe to real-time events via WebSocket
    ///
    /// Establishes WebSocket connection for receiving real-time events:
    /// - device.added, device.removed
    /// - primal.status
    /// - niche.deployed
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON-RPC subscription call fails.
    pub async fn subscribe_events(&self) -> Result<()> {
        info!("Subscribing to real-time events from provider");

        self.call_jsonrpc("events.subscribe", build_subscribe_events_params())
            .await?;

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
        drop(event_stream_guard);

        Ok(())
    }

    /// Subscribe to events with a callback for real-time handling
    ///
    /// The callback will be invoked for each event received via WebSocket.
    ///
    /// # Errors
    ///
    /// Returns an error if `subscribe_events` fails.
    pub async fn subscribe_events_with_callback<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(BiomeOSEvent) + Send + Sync + 'static,
    {
        // First, establish the subscription
        self.subscribe_events().await?;

        // Set callback on event stream
        {
            let mut event_stream_guard = self.event_stream.write().await;

            if let Some(ref mut event_stream) = *event_stream_guard {
                event_stream.set_callback(callback);
                info!("✅ Event callback registered");
            }
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
            .map_err(|e| BiomeOsIntegrationError::ConnectToProvider(e.to_string()))?;

        let request = build_jsonrpc_request(method, params, 1);

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
            .map_err(|e| BiomeOsIntegrationError::ReadResponse(e.to_string()))?;

        // Parse JSON-RPC response
        let response: serde_json::Value = serde_json::from_str(&response_line)
            .map_err(|e| BiomeOsIntegrationError::ParseJsonRpcResponse(e.to_string()))?;

        if let Some(error) = parse_jsonrpc_error(&response) {
            return Err(BiomeOsIntegrationError::JsonRpcError(error.to_string()).into());
        }

        parse_jsonrpc_result(&response)
            .ok_or(BiomeOsIntegrationError::NoJsonRpcResult)
            .map_err(Into::into)
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

    /// Health check returning status string (for `VisualizationDataProvider`)
    ///
    /// # Errors
    ///
    /// Returns an error if both `health.check` and `health.ping` JSON-RPC calls fail.
    pub(super) async fn health_check_jsonrpc(&self) -> Result<String> {
        // Try health.check first (semantic method)
        let params = serde_json::json!({});
        let result = self.call_jsonrpc("health.check", params).await;

        if let Ok(res) = result {
            let status = health_response_status(&res);
            if status != "unknown" {
                return Ok(status);
            }
            if let Some(healthy) = health_response_healthy(&res) {
                return Ok(if healthy { "healthy" } else { "unhealthy" }.to_string());
            }
        } else {
            // Fallback to health.ping
            self.call_jsonrpc("health.ping", serde_json::json!({}))
                .await?;
        }
        Ok("healthy".to_string())
    }

    /// Expose endpoint for `VisualizationDataProvider` trait impl
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

    /// Expose `derive_websocket_endpoint` for testing
    #[must_use]
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

    #[test]
    fn assign_device_params_structure() {
        let params = serde_json::json!({
            "device_id": "dev-1",
            "primal_id": "primal-1"
        });
        assert_eq!(params["device_id"], "dev-1");
        assert_eq!(params["primal_id"], "primal-1");
    }

    #[test]
    fn deploy_niche_params_structure() {
        let params = serde_json::json!({
            "name": "test-niche",
            "description": "A test",
            "required_primals": ["p1"],
            "optional_primals": [],
            "metadata": {}
        });
        assert_eq!(params["name"], "test-niche");
        assert!(params["required_primals"].is_array());
    }

    #[test]
    fn jsonrpc_request_structure() {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "device.list",
            "params": {},
            "id": 1
        });
        assert_eq!(request["jsonrpc"], "2.0");
        assert_eq!(request["method"], "device.list");
    }

    #[test]
    fn subscribe_events_params_structure() {
        let params = serde_json::json!({
            "events": ["device.added", "device.removed", "primal.status", "niche.deployed"]
        });
        let arr = params["events"].as_array().expect("array");
        assert!(arr.contains(&serde_json::json!("device.added")));
    }

    #[test]
    fn health_check_jsonrpc_params() {
        let params = serde_json::json!({});
        assert!(params.is_object());
    }

    #[test]
    fn build_jsonrpc_request_structure() {
        let req = build_jsonrpc_request("device.list", serde_json::json!({}), 1);
        assert_eq!(req["jsonrpc"], "2.0");
        assert_eq!(req["method"], "device.list");
        assert_eq!(req["id"], 1);
    }

    #[test]
    fn parse_jsonrpc_result_present() {
        let res = serde_json::json!({"result": {"ok": true}, "id": 1});
        let r = parse_jsonrpc_result(&res);
        assert!(r.is_some());
        assert_eq!(r.unwrap()["ok"], true);
    }

    #[test]
    fn parse_jsonrpc_result_absent() {
        let res = serde_json::json!({"id": 1});
        assert!(parse_jsonrpc_result(&res).is_none());
    }

    #[test]
    fn parse_jsonrpc_error_present() {
        let res = serde_json::json!({"error": {"code": -32600, "message": "Invalid"}});
        assert!(parse_jsonrpc_error(&res).is_some());
    }

    #[test]
    fn test_health_response_status() {
        let v = serde_json::json!({"status": "ok"});
        assert_eq!(super::health_response_status(&v), "ok");
    }

    #[test]
    fn test_health_response_status_unknown() {
        let v = serde_json::json!({});
        assert_eq!(super::health_response_status(&v), "unknown");
    }

    #[test]
    fn test_health_response_healthy() {
        let v = serde_json::json!({"healthy": true});
        assert_eq!(super::health_response_healthy(&v), Some(true));
    }

    #[test]
    fn test_build_assign_device_params() {
        let params = build_assign_device_params("dev-1", "primal-1");
        assert_eq!(params["device_id"], "dev-1");
        assert_eq!(params["primal_id"], "primal-1");
    }

    #[test]
    fn test_build_deploy_niche_params() {
        use crate::biomeos_integration::NicheTemplate;
        let niche = NicheTemplate {
            id: "id1".to_string(),
            name: "test".to_string(),
            description: "desc".to_string(),
            required_primals: vec!["p1".to_string()],
            optional_primals: vec![],
            metadata: serde_json::json!({}),
        };
        let params = build_deploy_niche_params(&niche);
        assert_eq!(params["name"], "test");
        assert!(params["required_primals"].is_array());
    }

    #[test]
    fn test_extract_niche_id_from_response() {
        let v = serde_json::json!({"niche_id": "niche-123"});
        let id = extract_niche_id_from_response(&v);
        assert!(id.is_some());
        assert_eq!(id.unwrap(), "niche-123");
    }

    #[test]
    fn test_build_subscribe_events_params() {
        let params = build_subscribe_events_params();
        let arr = params["events"].as_array().expect("array");
        assert!(arr.contains(&serde_json::json!("device.added")));
    }
}
