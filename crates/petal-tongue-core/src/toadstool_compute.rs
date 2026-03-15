// SPDX-License-Identifier: AGPL-3.0-only
//! # Toadstool Compute Provider
//!
//! GPU compute acceleration via Toadstool primal (discovered at runtime).

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::compute::{ComputeCapability, ComputeProvider};

/// Toadstool Service Info (discovered dynamically)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToadstoolServiceInfo {
    /// Service ID
    pub id: String,

    /// Endpoint (tarpc:// or http://)
    pub endpoint: String,

    /// Available capabilities
    pub capabilities: Vec<String>,

    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Toadstool Compute Provider
///
/// Provides GPU acceleration via Toadstool primal.
/// Discovered at runtime using capability-based discovery.
pub struct ToadstoolCompute {
    /// Service info (if discovered)
    service: Option<ToadstoolServiceInfo>,

    /// Available capabilities
    capabilities: Vec<ComputeCapability>,
}

impl ToadstoolCompute {
    /// Create new Toadstool compute provider
    ///
    /// Attempts to discover Toadstool at creation time.
    pub async fn new() -> Result<Self> {
        // Attempt discovery
        let service = Self::discover_toadstool().await.ok();

        // Determine capabilities based on discovery
        let capabilities = service
            .as_ref()
            .map_or_else(Vec::new, |svc| Self::parse_capabilities(&svc.capabilities));

        Ok(Self {
            service,
            capabilities,
        })
    }

    /// Discover GPU compute provider via universal discovery.
    ///
    /// Uses capability-based discovery (no hardcoded primal names).
    /// Follows toadStool S139 ecosystem discovery pattern:
    /// 1. Env override (`GPU_RENDERING_ENDPOINT`, `COMPUTE_PROVIDER_ENDPOINT`)
    /// 2. Ecosystem directory (`$XDG_RUNTIME_DIR/ecoPrimals/discovery/`)
    /// 3. Socket scan (`$XDG_RUNTIME_DIR/toadstool/`)
    #[expect(clippy::unused_async, reason = "async for future async discovery APIs")]
    async fn discover_toadstool() -> Result<ToadstoolServiceInfo> {
        if let Ok(endpoint) = std::env::var("GPU_RENDERING_ENDPOINT") {
            tracing::info!("Found GPU rendering service via environment: {endpoint}");
            return Ok(ToadstoolServiceInfo {
                id: "discovered-gpu-renderer".to_string(),
                endpoint,
                capabilities: vec!["gpu.dispatch".to_string(), "display".to_string()],
                metadata: HashMap::new(),
            });
        }

        if let Ok(endpoint) = std::env::var("COMPUTE_PROVIDER_ENDPOINT") {
            tracing::info!("Found compute provider via environment: {endpoint}");
            return Ok(ToadstoolServiceInfo {
                id: "discovered-compute-provider".to_string(),
                endpoint,
                capabilities: vec![
                    "gpu.dispatch".to_string(),
                    "science.gpu.dispatch".to_string(),
                    "display".to_string(),
                ],
                metadata: HashMap::new(),
            });
        }

        // Ecosystem discovery: scan for manifest files (toadStool S139 dual-write)
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
        let discovery_dir = format!("{runtime_dir}/ecoPrimals/discovery");
        if let Ok(entries) = std::fs::read_dir(&discovery_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "json")
                    && let Ok(contents) = std::fs::read_to_string(&path)
                    && let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&contents)
                {
                    let caps = manifest
                        .get("capabilities")
                        .and_then(|c| c.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();

                    if caps.iter().any(|c| c == "gpu.dispatch" || c == "display") {
                        let endpoint = manifest
                            .get("endpoint")
                            .and_then(|e| e.as_str())
                            .unwrap_or("")
                            .to_string();
                        let id = manifest
                            .get("id")
                            .and_then(|i| i.as_str())
                            .unwrap_or("discovered-compute")
                            .to_string();
                        tracing::info!("Found GPU compute provider via ecosystem discovery: {id}");
                        return Ok(ToadstoolServiceInfo {
                            id,
                            endpoint,
                            capabilities: caps,
                            metadata: HashMap::new(),
                        });
                    }
                }
            }
        }

        // Final fallback: GPU_COMPUTE_ENDPOINT when ecosystem discovery fails (env-driven)
        if std::env::var("GPU_COMPUTE_ENDPOINT").is_ok() {
            let endpoint = crate::constants::default_gpu_compute_endpoint();
            if !endpoint.is_empty() {
                tracing::info!("Using GPU compute endpoint from env: {endpoint}");
                return Ok(ToadstoolServiceInfo {
                    id: "env-gpu-compute".to_string(),
                    endpoint,
                    capabilities: vec![
                        "gpu.dispatch".to_string(),
                        "science.gpu.dispatch".to_string(),
                        "display".to_string(),
                    ],
                    metadata: HashMap::new(),
                });
            }
        }

        anyhow::bail!("No GPU compute provider discovered")
    }

    /// Parse capability strings into `ComputeCapability` enum.
    ///
    /// Recognizes both legacy strings and ecosystem capability strings
    /// (barraCuda v0.3.3+, toadStool S139+).
    pub(crate) fn parse_capabilities(caps: &[String]) -> Vec<ComputeCapability> {
        let mut result = Vec::new();

        for cap in caps {
            match cap.as_str() {
                "layout-computation" | "gpu-layout" | "gpu.dispatch" => {
                    result.push(ComputeCapability::LayoutComputation);
                }
                "physics" | "physics-simulation" | "science.gpu.dispatch" => {
                    result.push(ComputeCapability::PhysicsSimulation);
                }
                "ray-tracing" | "raytracing" => {
                    result.push(ComputeCapability::RayTracing);
                }
                "particle-effects" | "particles" => {
                    result.push(ComputeCapability::ParticleEffects);
                }
                "image-processing" | "image" => {
                    result.push(ComputeCapability::ImageProcessing);
                }
                "display" | "shader.compile" => {
                    // Hardware/shader capabilities -- noted but not mapped to compute
                }
                _ => {
                    tracing::debug!("Unrecognized compute capability: {cap}");
                }
            }
        }

        result
    }

    /// Get service info
    #[must_use]
    pub const fn service(&self) -> Option<&ToadstoolServiceInfo> {
        self.service.as_ref()
    }
}

#[async_trait]
impl ComputeProvider for ToadstoolCompute {
    fn name(&self) -> &'static str {
        // Return generic name (not "Toadstool")
        "GPU Compute Provider"
    }

    fn capabilities(&self) -> Vec<ComputeCapability> {
        self.capabilities.clone()
    }

    async fn is_available(&self) -> bool {
        self.service.is_some()
    }

    async fn initialize(&mut self) -> Result<()> {
        if self.service.is_none() {
            // Try discovery again
            self.service = Self::discover_toadstool().await.ok();

            if let Some(ref svc) = self.service {
                self.capabilities = Self::parse_capabilities(&svc.capabilities);
                tracing::info!("✅ GPU compute provider initialized: {}", svc.endpoint);
            } else {
                anyhow::bail!("No GPU compute provider available");
            }
        }

        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("🔇 Shutting down GPU compute provider");
        self.service = None;
        self.capabilities.clear();
        Ok(())
    }
}

/// CPU Fallback Compute Provider
///
/// Provides basic compute capabilities using CPU when GPU is unavailable.
pub struct CPUFallbackCompute {
    capabilities: Vec<ComputeCapability>,
}

impl CPUFallbackCompute {
    /// Create new CPU fallback provider
    #[must_use]
    pub fn new() -> Self {
        Self {
            capabilities: vec![
                ComputeCapability::LayoutComputation,
                // CPU can do basic layout, but not advanced features
            ],
        }
    }
}

impl Default for CPUFallbackCompute {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ComputeProvider for CPUFallbackCompute {
    fn name(&self) -> &'static str {
        "CPU Fallback"
    }

    fn capabilities(&self) -> Vec<ComputeCapability> {
        self.capabilities.clone()
    }

    async fn is_available(&self) -> bool {
        // CPU is always available
        true
    }

    async fn initialize(&mut self) -> Result<()> {
        tracing::info!("✅ CPU fallback compute initialized");
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("🔇 Shutting down CPU fallback compute");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_capabilities_ecosystem_strings() {
        let caps = vec![
            "gpu.dispatch".to_string(),
            "science.gpu.dispatch".to_string(),
            "display".to_string(),
            "shader.compile".to_string(),
        ];
        let parsed = ToadstoolCompute::parse_capabilities(&caps);
        // gpu.dispatch -> LayoutComputation, science.gpu.dispatch -> PhysicsSimulation
        // display and shader.compile are noted but not mapped to compute
        assert!(parsed.contains(&ComputeCapability::LayoutComputation));
        assert!(parsed.contains(&ComputeCapability::PhysicsSimulation));
        assert_eq!(parsed.len(), 2);
    }

    #[tokio::test]
    async fn test_discover_toadstool_with_manifest_in_temp_dir() {
        let temp = tempfile::tempdir().unwrap();
        let discovery_dir = temp.path().join("ecoPrimals").join("discovery");
        std::fs::create_dir_all(&discovery_dir).unwrap();

        let manifest = serde_json::json!({
            "id": "test-gpu-compute",
            "endpoint": crate::constants::DEFAULT_GPU_COMPUTE_ENDPOINT,
            "capabilities": ["gpu.dispatch", "display"]
        });
        let manifest_path = discovery_dir.join("gpu-compute.json");
        std::fs::write(&manifest_path, serde_json::to_string(&manifest).unwrap()).unwrap();

        let runtime_dir = temp.path().to_str().unwrap().to_string();
        let provider = crate::test_fixtures::env_test_helpers::with_env_var_async(
            "XDG_RUNTIME_DIR",
            &runtime_dir,
            || async { ToadstoolCompute::new().await.unwrap() },
        )
        .await;

        assert!(provider.is_available().await);
        assert!(
            provider
                .capabilities()
                .contains(&ComputeCapability::LayoutComputation)
        );
    }

    #[tokio::test]
    async fn test_cpu_fallback_lifecycle_init_available_shutdown() {
        let mut provider = CPUFallbackCompute::new();

        assert!(
            provider.is_available().await,
            "CPU fallback always available"
        );

        provider.initialize().await.unwrap();
        assert!(provider.is_available().await, "Available after init");

        provider.shutdown().await.unwrap();
        assert!(
            provider.is_available().await,
            "CPU fallback still available after shutdown"
        );
    }

    #[tokio::test]
    async fn test_toadstool_creation() {
        let provider = ToadstoolCompute::new().await.unwrap();
        assert_eq!(provider.name(), "GPU Compute Provider");
    }

    #[tokio::test]
    async fn test_toadstool_without_discovery() {
        use crate::test_fixtures::env_test_helpers;

        let provider = env_test_helpers::with_env_vars_removed_async(
            &[
                "GPU_RENDERING_ENDPOINT",
                "COMPUTE_PROVIDER_ENDPOINT",
                "GPU_COMPUTE_ENDPOINT",
                "XDG_RUNTIME_DIR",
            ],
            || async { ToadstoolCompute::new().await.unwrap() },
        )
        .await;
        assert!(!provider.is_available().await);
    }

    #[tokio::test]
    async fn test_capability_parsing() {
        let caps = vec![
            "layout-computation".to_string(),
            "physics".to_string(),
            "unknown-capability".to_string(),
        ];

        let parsed = ToadstoolCompute::parse_capabilities(&caps);

        assert_eq!(parsed.len(), 2); // unknown should be skipped
        assert!(parsed.contains(&ComputeCapability::LayoutComputation));
        assert!(parsed.contains(&ComputeCapability::PhysicsSimulation));
    }

    #[tokio::test]
    async fn test_cpu_fallback_creation() {
        let provider = CPUFallbackCompute::new();
        assert_eq!(provider.name(), "CPU Fallback");
        assert!(provider.is_available().await);
    }

    #[tokio::test]
    async fn test_cpu_fallback_capabilities() {
        let provider = CPUFallbackCompute::new();
        let caps = provider.capabilities();

        assert!(!caps.is_empty());
        assert!(caps.contains(&ComputeCapability::LayoutComputation));
    }

    #[tokio::test]
    async fn test_cpu_fallback_lifecycle() {
        let mut provider = CPUFallbackCompute::new();

        // Initialize
        let result = provider.initialize().await;
        assert!(result.is_ok());

        // Shutdown
        let result = provider.shutdown().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_capabilities_raytracing() {
        let caps = vec!["ray-tracing".to_string(), "raytracing".to_string()];
        let parsed = ToadstoolCompute::parse_capabilities(&caps);
        assert_eq!(parsed.len(), 2);
        assert!(parsed.iter().all(|c| *c == ComputeCapability::RayTracing));
    }

    #[test]
    fn test_parse_capabilities_particle_effects() {
        let caps = vec!["particle-effects".to_string(), "particles".to_string()];
        let parsed = ToadstoolCompute::parse_capabilities(&caps);
        assert_eq!(parsed.len(), 2);
        assert!(
            parsed
                .iter()
                .all(|c| *c == ComputeCapability::ParticleEffects)
        );
    }

    #[test]
    fn test_parse_capabilities_image_processing() {
        let caps = vec!["image-processing".to_string(), "image".to_string()];
        let parsed = ToadstoolCompute::parse_capabilities(&caps);
        assert_eq!(parsed.len(), 2);
        assert!(
            parsed
                .iter()
                .all(|c| *c == ComputeCapability::ImageProcessing)
        );
    }

    #[test]
    fn test_parse_capabilities_legacy_strings() {
        let caps = vec![
            "layout-computation".to_string(),
            "gpu-layout".to_string(),
            "physics".to_string(),
            "physics-simulation".to_string(),
        ];
        let parsed = ToadstoolCompute::parse_capabilities(&caps);
        assert!(parsed.contains(&ComputeCapability::LayoutComputation));
        assert!(parsed.contains(&ComputeCapability::PhysicsSimulation));
    }

    #[test]
    fn test_parse_capabilities_display_skipped() {
        let caps = vec!["display".to_string(), "shader.compile".to_string()];
        let parsed = ToadstoolCompute::parse_capabilities(&caps);
        assert!(parsed.is_empty());
    }

    #[test]
    fn test_parse_capabilities_empty() {
        let caps: Vec<String> = vec![];
        let parsed = ToadstoolCompute::parse_capabilities(&caps);
        assert!(parsed.is_empty());
    }

    #[test]
    fn test_toadstool_service_info() {
        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), "1.0.0".to_string());

        let info = ToadstoolServiceInfo {
            id: "test-service".to_string(),
            endpoint: crate::constants::DEFAULT_GPU_COMPUTE_ENDPOINT.to_string(),
            capabilities: vec!["gpu-rendering".to_string()],
            metadata,
        };

        assert_eq!(info.id.as_str(), "test-service");
        assert_eq!(
            info.endpoint,
            crate::constants::DEFAULT_GPU_COMPUTE_ENDPOINT
        );
        assert_eq!(info.capabilities.len(), 1);
        assert_eq!(info.metadata.get("version").unwrap(), "1.0.0");
    }
}
