// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::manual_async_fn)] // explicit `impl Future + Send` for public trait object safety
//! # Compute Provider System
//!
//! Optional GPU compute acceleration via capability-discovered providers.

use crate::error::Result;
use std::collections::HashMap;

/// Compute Capability
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComputeCapability {
    /// Force-directed layout computation
    LayoutComputation,

    /// Physics simulation
    PhysicsSimulation,

    /// Ray tracing
    RayTracing,

    /// Particle effects
    ParticleEffects,

    /// Image processing
    ImageProcessing,
}

/// Compute Provider Interface
///
/// Abstracts GPU compute providers discovered at runtime via capability IPC.
pub trait ComputeProvider: Send + Sync {
    /// Provider name
    fn name(&self) -> &str;

    /// Available capabilities
    fn capabilities(&self) -> Vec<ComputeCapability>;

    /// Check if provider is available
    fn is_available(&self) -> impl std::future::Future<Output = bool> + Send;

    /// Initialize provider
    fn initialize(&mut self) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Shutdown provider
    fn shutdown(&mut self) -> impl std::future::Future<Output = Result<()>> + Send;
}

/// Production compute provider implementations (see `crate::gpu_compute`).
pub enum ComputeProviderImpl {
    /// Capability-discovered GPU compute
    Gpu(crate::gpu_compute::GpuComputeProvider),
    /// CPU fallback when GPU is unavailable
    CpuFallback(crate::gpu_compute::CPUFallbackCompute),
    #[cfg(test)]
    Mock(MockComputeProvider),
}

impl ComputeProvider for ComputeProviderImpl {
    fn name(&self) -> &str {
        match self {
            Self::Gpu(p) => p.name(),
            Self::CpuFallback(p) => p.name(),
            #[cfg(test)]
            Self::Mock(p) => p.name(),
        }
    }

    fn capabilities(&self) -> Vec<ComputeCapability> {
        match self {
            Self::Gpu(p) => p.capabilities(),
            Self::CpuFallback(p) => p.capabilities(),
            #[cfg(test)]
            Self::Mock(p) => p.capabilities(),
        }
    }

    fn is_available(&self) -> impl std::future::Future<Output = bool> + Send {
        async move {
            match self {
                Self::Gpu(p) => p.is_available().await,
                Self::CpuFallback(p) => p.is_available().await,
                #[cfg(test)]
                Self::Mock(p) => p.is_available().await,
            }
        }
    }

    fn initialize(&mut self) -> impl std::future::Future<Output = Result<()>> + Send {
        async move {
            match self {
                Self::Gpu(p) => p.initialize().await,
                Self::CpuFallback(p) => p.initialize().await,
                #[cfg(test)]
                Self::Mock(p) => p.initialize().await,
            }
        }
    }

    fn shutdown(&mut self) -> impl std::future::Future<Output = Result<()>> + Send {
        async move {
            match self {
                Self::Gpu(p) => p.shutdown().await,
                Self::CpuFallback(p) => p.shutdown().await,
                #[cfg(test)]
                Self::Mock(p) => p.shutdown().await,
            }
        }
    }
}

/// Compute Registry
///
/// Manages available compute providers.
pub struct ComputeRegistry {
    providers: HashMap<String, ComputeProviderImpl>,
}

impl ComputeRegistry {
    /// Create new empty registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Register a compute provider
    pub fn register(&mut self, provider: ComputeProviderImpl) {
        let name = provider.name().to_string();
        self.providers.insert(name, provider);
    }

    /// Number of registered compute providers.
    #[must_use]
    pub fn len(&self) -> usize {
        self.providers.len()
    }

    /// Returns true if no compute providers are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }

    /// Get provider by name
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&ComputeProviderImpl> {
        self.providers.get(name)
    }

    /// Get mutable provider
    pub fn get_mut(&mut self, name: &str) -> Option<&mut ComputeProviderImpl> {
        self.providers.get_mut(name)
    }

    /// Get provider with specific capability
    pub async fn get_with_capability(
        &self,
        capability: ComputeCapability,
    ) -> Option<&ComputeProviderImpl> {
        for provider in self.providers.values() {
            if provider.capabilities().contains(&capability) && provider.is_available().await {
                return Some(provider);
            }
        }
        None
    }
}

impl Default for ComputeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
pub(crate) struct MockComputeProvider {
    name: String,
    caps: Vec<ComputeCapability>,
    available: bool,
}

#[cfg(test)]
impl ComputeProvider for MockComputeProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn capabilities(&self) -> Vec<ComputeCapability> {
        self.caps.clone()
    }

    fn is_available(&self) -> impl std::future::Future<Output = bool> + Send {
        let available = self.available;
        async move { available }
    }

    fn initialize(&mut self) -> impl std::future::Future<Output = Result<()>> + Send {
        async { Ok(()) }
    }

    fn shutdown(&mut self) -> impl std::future::Future<Output = Result<()>> + Send {
        async { Ok(()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_compute_registry() {
        let mut registry = ComputeRegistry::new();

        registry.register(ComputeProviderImpl::Mock(MockComputeProvider {
            name: "gpu-compute".to_string(),
            caps: vec![ComputeCapability::LayoutComputation],
            available: true,
        }));

        assert!(registry.get("gpu-compute").is_some());

        let provider = registry
            .get_with_capability(ComputeCapability::LayoutComputation)
            .await;
        assert!(provider.is_some());
    }

    #[test]
    fn test_compute_registry_default() {
        let registry = ComputeRegistry::default();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_compute_registry_is_empty() {
        let mut registry = ComputeRegistry::new();
        assert!(registry.is_empty());
        registry.register(ComputeProviderImpl::Mock(MockComputeProvider {
            name: "test".to_string(),
            caps: vec![],
            available: true,
        }));
        assert!(!registry.is_empty());
    }

    #[test]
    fn test_compute_registry_len() {
        let mut registry = ComputeRegistry::new();
        assert_eq!(registry.len(), 0);
        registry.register(ComputeProviderImpl::Mock(MockComputeProvider {
            name: "a".to_string(),
            caps: vec![],
            available: true,
        }));
        assert_eq!(registry.len(), 1);
        registry.register(ComputeProviderImpl::Mock(MockComputeProvider {
            name: "b".to_string(),
            caps: vec![],
            available: true,
        }));
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn test_compute_registry_get_mut() {
        let mut registry = ComputeRegistry::new();
        registry.register(ComputeProviderImpl::Mock(MockComputeProvider {
            name: "test".to_string(),
            caps: vec![ComputeCapability::PhysicsSimulation],
            available: true,
        }));
        let provider = registry.get_mut("test");
        assert!(provider.is_some());
        assert!(registry.get_mut("missing").is_none());
    }

    #[tokio::test]
    async fn test_compute_registry_get_with_capability_unavailable() {
        let mut registry = ComputeRegistry::new();
        registry.register(ComputeProviderImpl::Mock(MockComputeProvider {
            name: "unavail".to_string(),
            caps: vec![ComputeCapability::LayoutComputation],
            available: false,
        }));
        let provider = registry
            .get_with_capability(ComputeCapability::LayoutComputation)
            .await;
        assert!(provider.is_none());
    }

    #[tokio::test]
    async fn test_compute_registry_get_with_capability_missing() {
        let mut registry = ComputeRegistry::new();
        registry.register(ComputeProviderImpl::Mock(MockComputeProvider {
            name: "other".to_string(),
            caps: vec![ComputeCapability::ImageProcessing],
            available: true,
        }));
        let provider = registry
            .get_with_capability(ComputeCapability::RayTracing)
            .await;
        assert!(provider.is_none());
    }

    #[test]
    fn test_compute_capability_variants() {
        let _ = ComputeCapability::LayoutComputation;
        let _ = ComputeCapability::PhysicsSimulation;
        let _ = ComputeCapability::RayTracing;
        let _ = ComputeCapability::ParticleEffects;
        let _ = ComputeCapability::ImageProcessing;
    }
}
