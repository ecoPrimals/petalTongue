// SPDX-License-Identifier: AGPL-3.0-only
#![forbid(unsafe_code)]
//! Ecosystem-specific adapters for petalTongue
//!
//! # Philosophy
//!
//! petalTongue core is **universal** and has **zero knowledge** of specific
//! ecosystems (trust levels, family IDs, etc.). This crate provides the
//! adapter system that allows ecosystem-specific UI to be composed on top.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │  Ecosystem Adapters (this crate)       │
//! │  ┌──────────┐ ┌───────────┐           │
//! │  │  Trust   │ │  Family   │  ...      │
//! │  └──────────┘ └───────────┘           │
//! ├─────────────────────────────────────────┤
//! │  Adapter Registry (runtime routing)     │
//! ├─────────────────────────────────────────┤
//! │  petalTongue Core (universal)           │
//! └─────────────────────────────────────────┘
//! ```
//!
//! # Core Principles
//!
//! 1. **Self-knowledge only**: petalTongue knows nothing about ecosystems
//! 2. **Runtime discovery**: Adapters are loaded based on discovered capabilities
//! 3. **Configuration from ecosystem**: Adapters get config FROM ecosystem, not hardcoded
//! 4. **Fallback to generic**: Unknown properties still display (key-value)
//!
//! # Example
//!
//! ```ignore
//! use petal_tongue_adapters::*;
//!
//! // Create registry
//! let registry = AdapterRegistry::new();
//!
//! // Discover ecosystem capabilities
//! if ecosystem_has("trust-management") {
//!     let spec = ecosystem_spec("trust-management");
//!     let adapter = EcoPrimalTrustAdapter::from_spec(spec);
//!     registry.register(Box::new(adapter));
//! }
//!
//! // Later, render properties
//! registry.render_property("trust_level", &value, ui);
//! ```

pub mod adapter_trait;
pub mod registry;

// Re-exports
pub use adapter_trait::{BoxedAdapter, NodeDecoration, PropertyAdapter};
pub use registry::AdapterRegistry;

// ecoPrimals-specific adapters
pub mod ecoprimal;
pub use ecoprimal::{EcoPrimalCapabilityAdapter, EcoPrimalFamilyAdapter, EcoPrimalTrustAdapter};
