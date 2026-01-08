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
        let capabilities = if let Some(ref svc) = service {
            Self::parse_capabilities(&svc.capabilities)
        } else {
            Vec::new()
        };

        Ok(Self {
            service,
            capabilities,
        })
    }

    /// Discover Toadstool via universal discovery
    ///
    /// Uses capability-based discovery (no hardcoded names or endpoints).
    async fn discover_toadstool() -> Result<ToadstoolServiceInfo> {
        // Try environment variable first
        if let Ok(endpoint) = std::env::var("GPU_RENDERING_ENDPOINT") {
            tracing::info!(
                "🔍 Found GPU rendering service via environment: {}",
                endpoint
            );

            return Ok(ToadstoolServiceInfo {
                id: "discovered-gpu-renderer".to_string(),
                endpoint,
                capabilities: vec![
                    "gpu-rendering".to_string(),
                    "layout-computation".to_string(),
                ],
                metadata: HashMap::new(),
            });
        }

        // Try COMPUTE_PROVIDER_ENDPOINT
        if let Ok(endpoint) = std::env::var("COMPUTE_PROVIDER_ENDPOINT") {
            tracing::info!("🔍 Found compute provider via environment: {}", endpoint);

            return Ok(ToadstoolServiceInfo {
                id: "discovered-compute-provider".to_string(),
                endpoint,
                capabilities: vec![
                    "gpu-rendering".to_string(),
                    "layout-computation".to_string(),
                    "particle-effects".to_string(),
                ],
                metadata: HashMap::new(),
            });
        }

        // TODO: Implement mDNS discovery
        // TODO: Implement Unix socket probing
        // TODO: Implement HTTP probing

        anyhow::bail!("No GPU compute provider discovered")
    }

    /// Parse capability strings into ComputeCapability enum
    fn parse_capabilities(caps: &[String]) -> Vec<ComputeCapability> {
        let mut result = Vec::new();

        for cap in caps {
            match cap.as_str() {
                "layout-computation" | "gpu-layout" => {
                    result.push(ComputeCapability::LayoutComputation);
                }
                "physics" | "physics-simulation" => {
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
                _ => {
                    tracing::warn!("Unknown compute capability: {}", cap);
                }
            }
        }

        result
    }

    /// Get service info
    pub fn service(&self) -> Option<&ToadstoolServiceInfo> {
        self.service.as_ref()
    }
}

#[async_trait]
impl ComputeProvider for ToadstoolCompute {
    fn name(&self) -> &str {
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
    fn name(&self) -> &str {
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

    #[tokio::test]
    async fn test_toadstool_creation() {
        let provider = ToadstoolCompute::new().await.unwrap();
        assert_eq!(provider.name(), "GPU Compute Provider");
    }

    #[tokio::test]
    async fn test_toadstool_without_discovery() {
        // Without environment variables, should not find service
        let provider = ToadstoolCompute::new().await.unwrap();
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
    fn test_toadstool_service_info() {
        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), "1.0.0".to_string());

        let info = ToadstoolServiceInfo {
            id: "test-service".to_string(),
            endpoint: "tarpc://localhost:9001".to_string(),
            capabilities: vec!["gpu-rendering".to_string()],
            metadata,
        };

        assert_eq!(info.id, "test-service");
        assert_eq!(info.endpoint, "tarpc://localhost:9001");
        assert_eq!(info.capabilities.len(), 1);
        assert_eq!(info.metadata.get("version").unwrap(), "1.0.0");
    }
}
