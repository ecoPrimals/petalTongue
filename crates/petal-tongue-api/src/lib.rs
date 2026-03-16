// SPDX-License-Identifier: AGPL-3.0-only
#![forbid(unsafe_code)]
//! # petal-tongue-api
//!
//! API clients for petalTongue integration
//!
//! # TRUE PRIMAL Architecture
//!
//! Per `PRIMAL_IPC_PROTOCOL.md`, inter-primal communication should use:
//! 1. **JSON-RPC 2.0** over Unix sockets (PRIMARY) - Use `BiomeOSJsonRpcClient`
//! 2. **tarpc** for high-performance needs (SECONDARY)
//! 3. **HTTP/REST** only for external/browser access (FALLBACK) - Use `BiomeOSClient`

#![warn(missing_docs)]
#![allow(clippy::unused_self)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::redundant_closure_for_method_calls)]

pub mod biomeos_client; // HTTP client (fallback for external use)
pub mod biomeos_error;
pub mod biomeos_jsonrpc_client; // JSON-RPC client (TRUE PRIMAL)

pub use biomeos_client::{BiomeOSClient, DiscoveredPrimal, DiscoveryResponse};
pub use biomeos_error::BiomeOsClientError;
pub use biomeos_jsonrpc_client::BiomeOSJsonRpcClient;
