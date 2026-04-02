// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! # petal-tongue-api
//!
//! API clients for petalTongue integration
//!
//! # TRUE PRIMAL Architecture
//!
//! Per IPC Protocol v3.1, inter-primal communication should use:
//! 1. **JSON-RPC 2.0** over Unix sockets (REQUIRED universal protocol) - Use `BiomeOSJsonRpcClient`
//! 2. **tarpc** (MAY for Rust-to-Rust hot paths)
//! 3. **HTTP/REST** only for external/browser access (FALLBACK) - Use `BiomeOSClient`

#![warn(missing_docs)]
#![expect(
    clippy::redundant_closure_for_method_calls,
    reason = "explicit closures preferred for readability in async chains"
)]

pub mod biomeos_client; // HTTP client (fallback for external use)
pub mod biomeos_error;
pub mod biomeos_jsonrpc_client; // JSON-RPC client (universal fallback)

pub use biomeos_client::{BiomeOSClient, DiscoveredPrimal, DiscoveryResponse};
pub use biomeos_error::BiomeOsClientError;
pub use biomeos_jsonrpc_client::BiomeOSJsonRpcClient;
