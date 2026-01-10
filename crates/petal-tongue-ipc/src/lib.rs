//! Inter-process communication for petalTongue
//!
//! This crate enables communication between petalTongue instances and other primals.
//! It provides multiple protocols following ecosystem standards:
//!
//! # Protocol Priority (Ecosystem Standard)
//!
//! 1. **tarpc** (PRIMARY) - High-performance primal-to-primal
//!    - ~10-20 μs latency
//!    - ~100K requests/sec
//!    - Binary protocol (bincode)
//!    - Type-safe at compile time
//!
//! 2. **JSON-RPC** (SECONDARY) - Local IPC and debugging
//!    - Port-free Unix sockets
//!    - Human-readable
//!    - Universal compatibility
//!
//! 3. **HTTPS** (OPTIONAL) - External/browser access
//!    - Network accessible
//!    - TLS encrypted
//!    - REST-like interface
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │              PRIMAL-TO-PRIMAL (tarpc)                      │
//! │  petalTongue → Toadstool (GPU rendering)                   │
//! │  petalTongue → Songbird (discovery)                        │
//! │  Binary RPC, ~10-20 μs latency                             │
//! └─────────────────────────────────────────────────────────────┘
//!                            ↓
//! ┌─────────────────────────────────────────────────────────────┐
//! │              LOCAL IPC (JSON-RPC)                           │
//! │  petalTongue instance → petalTongue instance               │
//! │  petalTongue → CLI tools                                   │
//! │  Unix sockets: /tmp/petaltongue/{uuid}.sock                │
//! └─────────────────────────────────────────────────────────────┘
//!                            ↓
//! ┌─────────────────────────────────────────────────────────────┐
//! │              EXTERNAL ACCESS (HTTPS - Optional)             │
//! │  Browser → petalTongue API                                 │
//! │  External tools → petalTongue                              │
//! └─────────────────────────────────────────────────────────────┘
//! ```

pub mod client;
pub mod json_rpc;
pub mod protocol;
pub mod server;
pub mod socket_path;
pub mod tarpc_client;
pub mod tarpc_types;
pub mod unix_socket_server;

// JSON-RPC (SECONDARY - local IPC)
pub use client::{IpcClient, IpcClientError};
pub use json_rpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
pub use protocol::{InstanceStatus, IpcCommand, IpcResponse};
pub use server::{IpcServer, IpcServerError};
pub use unix_socket_server::UnixSocketServer;

// tarpc (PRIMARY - primal-to-primal)
pub use tarpc_client::{TarpcClient, TarpcClientError, TarpcResult};
pub use tarpc_types::{
    HealthStatus, PetalTongueRpc, PetalTongueRpcClient, PrimalEndpoint, PrimalMetrics,
    ProtocolInfo, RenderRequest, RenderResponse, VersionInfo,
};
