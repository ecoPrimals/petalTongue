// SPDX-License-Identifier: AGPL-3.0-or-later
//! tarpc server implementation serving `PetalTongueRpc` over UDS.

use crate::tarpc_types::{
    HealthStatus, PetalTongueRpc, PrimalEndpoint, PrimalMetrics, ProtocolInfo, RenderRequest,
    RenderResponse, VersionInfo,
};
use futures_util::StreamExt;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tarpc::server::{self, Channel};
use thiserror::Error;
use tracing::{debug, error, info};

/// Errors from the tarpc UDS server.
#[derive(Debug, Error)]
pub enum TarpcServerError {
    /// Socket bind failure (path conflict, permissions, etc.).
    #[error("socket bind failed: {0}")]
    Bind(String),
    /// Underlying IO error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// tarpc UDS server state.
pub struct TarpcServer {
    socket_path: PathBuf,
    start_time: Instant,
}

impl TarpcServer {
    /// Create a new tarpc server that will bind to the given UDS path.
    pub fn new(socket_path: PathBuf) -> Self {
        Self {
            socket_path,
            start_time: Instant::now(),
        }
    }

    /// Create from default socket path resolution.
    pub fn from_default_path() -> Result<Self, TarpcServerError> {
        let path = crate::socket_path::get_petaltongue_tarpc_socket_path()
            .map_err(|e| TarpcServerError::Bind(e.to_string()))?;
        Ok(Self::new(path))
    }

    /// The socket path this server will bind to.
    #[must_use]
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    /// Start accepting tarpc connections on the UDS socket.
    ///
    /// This future runs indefinitely (until cancelled).
    pub async fn serve(self) -> Result<(), TarpcServerError> {
        if let Err(e) = std::fs::remove_file(&self.socket_path)
            && e.kind() != std::io::ErrorKind::NotFound
        {
            return Err(TarpcServerError::Io(e));
        }

        if let Some(parent) = self.socket_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let listener = tokio::net::UnixListener::bind(&self.socket_path)
            .map_err(|e| TarpcServerError::Bind(format!("{}: {e}", self.socket_path.display())))?;

        info!(
            "tarpc UDS server listening: {} (C2 dual-socket)",
            self.socket_path.display()
        );

        let start_time = self.start_time;

        loop {
            let (stream, _addr) = listener.accept().await?;
            debug!("tarpc UDS connection accepted");

            let codec = tokio_util::codec::LengthDelimitedCodec::builder()
                .max_frame_length(16 * 1024 * 1024)
                .new_framed(stream);

            let transport = tarpc::serde_transport::new(
                codec,
                tokio_serde::formats::Bincode::default(),
            );

            let handler = PetalTongueRpcHandler { start_time };

            let channel = server::BaseChannel::with_defaults(transport);
            tokio::spawn(channel.execute(handler.serve()).for_each(|resp| async {
                tokio::spawn(resp);
            }));
        }
    }
}

impl Drop for TarpcServer {
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_file(&self.socket_path)
            && e.kind() != std::io::ErrorKind::NotFound
        {
            error!("Failed to remove tarpc socket: {e}");
        }
    }
}

#[derive(Clone)]
struct PetalTongueRpcHandler {
    start_time: Instant,
}

impl PetalTongueRpc for PetalTongueRpcHandler {
    async fn capabilities_list(self, _: tarpc::context::Context) -> Vec<String> {
        crate::capability_detection::detect_capability_strings()
            .into_iter()
            .map(str::to_owned)
            .collect()
    }

    async fn discovery_find_capability(
        self,
        _: tarpc::context::Context,
        _capability: String,
    ) -> Vec<PrimalEndpoint> {
        Vec::new()
    }

    async fn health_check(self, _: tarpc::context::Context) -> HealthStatus {
        HealthStatus {
            status: "healthy".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
            capabilities: crate::capability_detection::detect_capability_strings()
                .into_iter()
                .map(str::to_owned)
                .collect(),
            details: std::collections::HashMap::new(),
        }
    }

    async fn version_get(self, _: tarpc::context::Context) -> VersionInfo {
        VersionInfo {
            version: env!("CARGO_PKG_VERSION").to_owned(),
            tarpc_version: "0.37".to_owned(),
            jsonrpc_version: "2.0".to_owned(),
            https_version: None,
            capabilities: crate::capability_detection::detect_capability_strings()
                .into_iter()
                .map(str::to_owned)
                .collect(),
        }
    }

    async fn protocols_list(self, _: tarpc::context::Context) -> Vec<ProtocolInfo> {
        let tarpc_endpoint = crate::socket_path::get_petaltongue_tarpc_socket_path()
            .map_or_else(|_| "unknown".to_owned(), |p| p.display().to_string());
        let jsonrpc_endpoint = crate::socket_path::get_petaltongue_socket_path()
            .map_or_else(|_| "unknown".to_owned(), |p| p.display().to_string());

        vec![
            ProtocolInfo {
                name: "tarpc".to_owned(),
                endpoint: format!("unix://{tarpc_endpoint}"),
                enabled: true,
                priority: 1,
                info: std::collections::HashMap::new(),
            },
            ProtocolInfo {
                name: "jsonrpc".to_owned(),
                endpoint: format!("unix://{jsonrpc_endpoint}"),
                enabled: true,
                priority: 2,
                info: std::collections::HashMap::new(),
            },
        ]
    }

    async fn ui_render_graph(
        self,
        _: tarpc::context::Context,
        _request: RenderRequest,
    ) -> RenderResponse {
        RenderResponse {
            success: true,
            data: bytes::Bytes::new(),
            width: 0,
            height: 0,
            error: None,
            render_time_ms: 0,
        }
    }

    async fn metrics_get(self, _: tarpc::context::Context) -> PrimalMetrics {
        PrimalMetrics {
            fps: None,
            time_since_last_frame: None,
            is_hanging: false,
            total_frames: 0,
            cpu_usage: None,
            memory_usage: None,
            uptime_seconds: self.start_time.elapsed().as_secs(),
            custom: std::collections::HashMap::new(),
        }
    }
}
