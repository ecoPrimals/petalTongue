// SPDX-License-Identifier: AGPL-3.0-only
#![cfg_attr(not(test), forbid(unsafe_code))]
#![warn(missing_docs)]
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
//! # Primal Registration
//!
//! Per `PRIMAL_IPC_PROTOCOL.md`, primals must register with Songbird on startup:
//!
//! ```rust,no_run
//! use petal_tongue_ipc::primal_registration::{RegistrationManager, PrimalRegistration};
//!
//! #[tokio::main]
//! async fn main() {
//!     let registration = PrimalRegistration::petaltongue();
//!     let manager = RegistrationManager::new(registration);
//!     
//!     // Register on startup (graceful if Songbird unavailable)
//!     manager.register_on_startup().await;
//!     
//!     // Start heartbeat task
//!     let _heartbeat_handle = manager.spawn_heartbeat_task();
//!     
//!     // ... rest of primal startup ...
//! }
//! ```
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

/// Runtime capability detection for display modalities
pub mod capability_detection;
pub mod client;
pub mod json_rpc;
pub mod json_rpc_client;
/// Compute bridge: async IPC for barraCuda math/physics/stat/tessellate/project operations
pub mod physics_bridge;
pub mod primal_registration;
pub mod protocol;
/// Provenance trio: rhizoCrypt + sweetGrass + loamSpine session lineage
pub mod provenance_trio;
pub mod server;
pub mod socket_path;
pub mod tarpc_client;
pub mod tarpc_types;
/// Unix socket connection handling (JSON-RPC over newline-delimited JSON)
pub mod unix_socket_connection;
/// JSON-RPC method dispatch and handlers
pub mod unix_socket_rpc_handlers;
/// Unix socket server for petalTongue IPC
pub mod unix_socket_server;
pub mod visualization_handler;

// JSON-RPC (SECONDARY - local IPC)
pub use client::{IpcClient, IpcClientError};
pub use json_rpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
pub use json_rpc_client::{JsonRpcClient, JsonRpcClientError, JsonRpcResult, TopologyData};
pub use protocol::{InstanceStatus, IpcCommand, IpcResponse};
pub use server::{IpcServer, IpcServerError};
pub use unix_socket_server::UnixSocketServer;
pub use visualization_handler::{
    BackpressureConfig, ConstraintResult, DismissRequest, DismissResponse, ExportRequest,
    ExportResponse, GrammarRenderRequest, GrammarRenderResponse, InteractionApplyRequest,
    InteractionApplyResponse, InteractionEventNotification, InteractionSubscriberRegistry,
    Perspective, PipelineRegistry, SensorStreamRegistry, SessionStatusRequest,
    SessionStatusResponse, StreamOperation, StreamUpdateRequest, StreamUpdateResponse,
    ValidateRequest, ValidateResponse, VisualizationRenderRequest, VisualizationRenderResponse,
    VisualizationState,
};

// tarpc (PRIMARY - primal-to-primal)
pub use tarpc_client::{TarpcClient, TarpcClientError, TarpcResult};
pub use tarpc_types::{
    HealthStatus, PetalTongueRpc, PetalTongueRpcClient, PrimalEndpoint, PrimalMetrics,
    ProtocolInfo, RenderRequest, RenderResponse, VersionInfo,
};
