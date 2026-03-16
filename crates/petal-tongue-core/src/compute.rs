// SPDX-License-Identifier: AGPL-3.0-only
//! # Compute Provider System
//!
//! Optional GPU compute acceleration (e.g., Toadstool).

use crate::error::Result;
use async_trait::async_trait;
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
/// Abstracts GPU compute providers (like Toadstool).
#[async_trait]
pub trait ComputeProvider: Send + Sync {
    /// Provider name
    fn name(&self) -> &str;

    /// Available capabilities
    fn capabilities(&self) -> Vec<ComputeCapability>;

    /// Check if provider is available
    async fn is_available(&self) -> bool;

    /// Initialize provider
    async fn initialize(&mut self) -> Result<()>;

    /// Shutdown provider
    async fn shutdown(&mut self) -> Result<()>;
}

/// Compute Registry
///
/// Manages available compute providers.
pub struct ComputeRegistry {
    providers: HashMap<String, Box<dyn ComputeProvider>>,
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
    pub fn register(&mut self, provider: Box<dyn ComputeProvider>) {
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
    pub fn get(&self, name: &str) -> Option<&dyn ComputeProvider> {
        self.providers.get(name).map(std::convert::AsRef::as_ref)
    }

    /// Get mutable provider
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Box<dyn ComputeProvider>> {
        self.providers.get_mut(name)
    }

    /// Get provider with specific capability
    pub async fn get_with_capability(
        &self,
        capability: ComputeCapability,
    ) -> Option<&dyn ComputeProvider> {
        for provider in self.providers.values() {
            if provider.capabilities().contains(&capability) && provider.is_available().await {
                return Some(provider.as_ref());
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
mod tests {
    use super::*;

    // Mock provider for testing
    struct MockComputeProvider {
        name: String,
        caps: Vec<ComputeCapability>,
        available: bool,
    }

    #[async_trait]
    impl ComputeProvider for MockComputeProvider {
        fn name(&self) -> &str {
            &self.name
        }

        fn capabilities(&self) -> Vec<ComputeCapability> {
            self.caps.clone()
        }

        async fn is_available(&self) -> bool {
            self.available
        }

        async fn initialize(&mut self) -> Result<()> {
            Ok(())
        }

        async fn shutdown(&mut self) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_compute_registry() {
        let mut registry = ComputeRegistry::new();

        registry.register(Box::new(MockComputeProvider {
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
        registry.register(Box::new(MockComputeProvider {
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
        registry.register(Box::new(MockComputeProvider {
            name: "a".to_string(),
            caps: vec![],
            available: true,
        }));
        assert_eq!(registry.len(), 1);
        registry.register(Box::new(MockComputeProvider {
            name: "b".to_string(),
            caps: vec![],
            available: true,
        }));
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn test_compute_registry_get_mut() {
        let mut registry = ComputeRegistry::new();
        registry.register(Box::new(MockComputeProvider {
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
        registry.register(Box::new(MockComputeProvider {
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
        registry.register(Box::new(MockComputeProvider {
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
