//! Inter-process communication for petalTongue
//!
//! This crate enables communication between petalTongue instances via Unix domain sockets.
//! It provides:
//! - IPC server for receiving commands
//! - IPC client for sending commands
//! - Command/response protocol
//! - CLI tools integration
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     IPC Server                              │
//! │  Listens: /tmp/petaltongue/{uuid}.sock                     │
//! │  Handles: Commands from other instances/CLI                 │
//! └─────────────────────────────────────────────────────────────┘
//!                            ↓
//! ┌─────────────────────────────────────────────────────────────┐
//! │                   IPC Protocol                              │
//! │  Commands: GetStatus, TransferState, Show, etc.            │
//! │  Responses: Success, Status, State, Error                   │
//! └─────────────────────────────────────────────────────────────┘
//!                            ↓
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    IPC Client                               │
//! │  Connects: To instance socket                               │
//! │  Sends: Commands and receives responses                     │
//! └─────────────────────────────────────────────────────────────┘
//! ```

pub mod protocol;
pub mod server;
pub mod client;

pub use protocol::{IpcCommand, IpcResponse, InstanceStatus};
pub use server::{IpcServer, IpcServerError};
pub use client::{IpcClient, IpcClientError};

