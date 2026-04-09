// SPDX-License-Identifier: AGPL-3.0-or-later
//! # 🚀 tarpc Types and Traits for petalTongue
//!
//! **HIGH-PERFORMANCE PRIMAL-TO-PRIMAL RPC**
//!
//! Provides shared types and service traits for tarpc-based communication.
//! This module defines the interface used by both clients and servers.
//!
//! ## Performance
//! - ~10-20 μs latency (vs 50-100 μs for JSON-RPC)
//! - ~100K requests/sec (vs 10K for JSON-RPC)
//! - Zero-copy binary serialization with bincode
//! - Type-safe at compile time
//!
//! ## Philosophy
//! - tarpc PRIMARY for primal-to-primal communication
//! - JSON-RPC SECONDARY for local IPC and debugging
//! - HTTPS OPTIONAL for external/browser access
//! - Protocol-agnostic architecture
//! - Zero unsafe blocks in this module
//! - Modern idiomatic Rust
//!
//! ## Safety
//! The `#[tarpc::service]` macro generates safe code using the tarpc framework.
//! All serialization is handled by serde with compile-time type checking.
//! No manual memory manipulation or unsafe operations are performed.
//! The generated client/server implementations use only safe Rust abstractions.

mod discovery;
mod health;
mod metrics;
mod render;
mod service;

// Re-export all public types for external consumers (no API change)
pub use discovery::PrimalEndpoint;
pub use health::{HealthStatus, ProtocolInfo, VersionInfo};
pub use metrics::PrimalMetrics;
pub use render::{RenderRequest, RenderResponse};
pub use service::{PetalTongueRpc, PetalTongueRpcClient};

#[cfg(test)]
mod test_support;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod proptest_tests;
