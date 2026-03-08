// SPDX-License-Identifier: AGPL-3.0-only
//! # Compute Provider System
//!
//! Optional GPU compute acceleration (e.g., Toadstool).

use anyhow::Result;
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
            name: "toadstool".to_string(),
            caps: vec![ComputeCapability::LayoutComputation],
            available: true,
        }));

        assert!(registry.get("toadstool").is_some());

        let provider = registry
            .get_with_capability(ComputeCapability::LayoutComputation)
            .await;
        assert!(provider.is_some());
    }
}
