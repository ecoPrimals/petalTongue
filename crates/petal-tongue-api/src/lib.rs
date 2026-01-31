//! # petal-tongue-api
//!
//! API clients for petalTongue integration
//!
//! # TRUE PRIMAL Architecture
//!
//! Per PRIMAL_IPC_PROTOCOL.md, inter-primal communication should use:
//! 1. **JSON-RPC 2.0** over Unix sockets (PRIMARY) - Use `BiomeOSJsonRpcClient`
//! 2. **tarpc** for high-performance needs (SECONDARY)
//! 3. **HTTP/REST** only for external/browser access (FALLBACK) - Use `BiomeOSClient`

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
// Allow some pedantic warnings - addressing in future refactoring
#![allow(clippy::unused_self)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::redundant_closure_for_method_calls)]

pub mod biomeos_client;           // HTTP client (fallback for external use)
pub mod biomeos_jsonrpc_client;   // JSON-RPC client (TRUE PRIMAL)

pub use biomeos_client::{BiomeOSClient, DiscoveredPrimal, DiscoveryResponse};
pub use biomeos_jsonrpc_client::BiomeOSJsonRpcClient;
