// SPDX-License-Identifier: AGPL-3.0-or-later
#![cfg_attr(not(test), forbid(unsafe_code))]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
#![warn(missing_docs)]
#![expect(
    clippy::missing_errors_doc,
    reason = "IPC handlers evolving — error docs tracked"
)]
//! Inter-process communication for petalTongue
//!
//! This crate enables communication between petalTongue instances and other primals.
//! It provides multiple protocols following ecosystem standards:
//!
//! # Protocol Priority (`PRIMAL_IPC_PROTOCOL.md` / `manifest.toml`)
//!
//! 1. **tarpc** (PRIMARY for inter-primal RPC)
//!    - Used when petalTongue connects *to* other primals on hot paths
//!    - High-performance binary protocol (bincode)
//!    - Type-safe at compile time
//!
//! 2. **JSON-RPC 2.0** (universal fallback — listen surface)
//!    - Newline-delimited over UDS (`$XDG_RUNTIME_DIR/biomeos/petaltongue.sock`)
//!    - Optional TCP via `server --port <PORT>`
//!    - Human-readable; maximum compatibility when tarpc is unavailable
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
//! │              JSON-RPC listen surface (universal fallback)   │
//! │  petalTongue instance → petalTongue instance               │
//! │  petalTongue → CLI tools                                   │
//! │  Unix sockets: $XDG_RUNTIME_DIR/biomeos/petaltongue.sock   │
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
pub mod discovery_helpers;
pub mod ipc_errors;
pub mod json_rpc;
pub mod json_rpc_client;
/// Compute bridge: async IPC for barraCuda math/physics/stat/tessellate/project operations
pub mod physics_bridge;
pub mod primal_registration;
pub mod primal_registration_error;
pub mod protocol;
/// Provenance trio: rhizoCrypt + sweetGrass + loamSpine session lineage
pub mod provenance_trio;
pub mod resilience;
pub mod server;
pub mod socket_path;
pub mod socket_path_error;
pub mod tarpc_client;
pub mod tarpc_types;
/// Unix socket connection handling (JSON-RPC over newline-delimited JSON)
pub mod unix_socket_connection;
/// JSON-RPC method dispatch and handlers
pub mod unix_socket_rpc_handlers;
/// Unix socket server for petalTongue IPC
pub mod unix_socket_server;
pub mod visualization_handler;

// JSON-RPC (universal fallback — listen surface)
pub use client::{IpcClient, IpcClientError};
pub use discovery_helpers::{address_env_var, resolve_primal_socket, socket_env_var};
pub use ipc_errors::{DispatchOutcome, IpcErrorPhase, StreamItem, exit_code, extract_rpc_error};
pub use json_rpc::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
pub use json_rpc_client::{JsonRpcClient, JsonRpcClientError, JsonRpcResult, TopologyData};
pub use protocol::{InstanceStatus, IpcCommand, IpcResponse};
pub use resilience::{CircuitBreaker, CircuitState, RetryPolicy};
pub use server::{IpcServer, IpcServerError};
pub use unix_socket_server::UnixSocketServer;
pub use visualization_handler::{
    BackpressureConfig, CallbackDispatch, ConstraintResult, DismissRequest, DismissResponse,
    ExportRequest, ExportResponse, GrammarRenderRequest, GrammarRenderResponse,
    InteractionApplyRequest, InteractionApplyResponse, InteractionEventNotification,
    InteractionSubscriberRegistry, Perspective, PipelineRegistry, SensorStreamRegistry,
    SessionStatusRequest, SessionStatusResponse, StreamOperation, StreamUpdateRequest,
    StreamUpdateResponse, ValidateRequest, ValidateResponse, VisualizationRenderRequest,
    VisualizationRenderResponse, VisualizationState,
};

// tarpc (PRIMARY - primal-to-primal)
pub use tarpc_client::{TarpcClient, TarpcClientError, TarpcResult};
pub use tarpc_types::{
    HealthStatus, PetalTongueRpc, PetalTongueRpcClient, PrimalEndpoint, PrimalMetrics,
    ProtocolInfo, RenderRequest, RenderResponse, VersionInfo,
};
