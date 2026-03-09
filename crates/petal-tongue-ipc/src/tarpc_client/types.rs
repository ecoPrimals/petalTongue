// SPDX-License-Identifier: AGPL-3.0-only
//! tarpc client types

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;

use crate::tarpc_types::PetalTongueRpcClient;

/// Error type for tarpc client operations
#[derive(Debug, thiserror::Error)]
pub enum TarpcClientError {
    /// Connection failed
    #[error("Connection failed: {0}")]
    Connection(String),

    /// RPC call failed
    #[error("RPC call failed: {0}")]
    Rpc(String),

    /// Serialization failed
    #[error("Serialization failed: {0}")]
    Serialization(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Timeout error
    #[error("Timeout: {0}")]
    Timeout(String),
}

/// Result type for tarpc client operations
pub type TarpcResult<T> = Result<T, TarpcClientError>;

/// Modern async tarpc client for petalTongue primal-to-primal communication
///
/// Provides high-performance binary RPC communication with automatic
/// connection management and type-safe method calls.
#[derive(Clone)]
pub struct TarpcClient {
    /// Original endpoint string
    pub(crate) endpoint: String,

    /// Parsed socket address
    pub(crate) addr: SocketAddr,

    /// Client connection (lazy-initialized)
    pub(crate) connection: Arc<RwLock<Option<PetalTongueRpcClient>>>,

    /// Request timeout
    pub(crate) timeout: Duration,
}
