// SPDX-License-Identifier: AGPL-3.0-or-later

#[cfg(test)]
use crate::json_rpc::JsonRpcRequest;
use crate::server::IpcServerError;
use crate::socket_path;
use crate::unix_socket_connection;
use crate::unix_socket_rpc_handlers::RpcHandlers;
use crate::visualization_handler::VisualizationState;
use petal_tongue_core::graph_engine::GraphEngine;
#[cfg(test)]
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::UnixListener;
use tracing::{debug, error, info};

/// JSON-RPC IPC server for petalTongue.
///
/// Listens on a Unix domain socket (always) and optionally on a TCP port
/// for newline-delimited JSON-RPC per `IPC_COMPLIANCE_MATRIX.md` v1.2.
pub struct UnixSocketServer {
    socket_path: PathBuf,
    family_id: String,
    handlers: RpcHandlers,
    motor_tx: Option<std::sync::mpsc::Sender<petal_tongue_core::MotorCommand>>,
    tcp_port: Option<u16>,
    /// Keeps the PT-06 push delivery thread alive for the server lifetime.
    _push_delivery_thread: std::thread::JoinHandle<()>,
}

impl UnixSocketServer {
    /// Create a new Unix socket server with graph and visualization state
    pub fn new(graph: Arc<std::sync::RwLock<GraphEngine>>) -> Result<Self, IpcServerError> {
        Self::new_with_socket(graph, None)
    }

    /// Create a server with an explicit socket path override.
    ///
    /// When `socket_override` is `Some`, it takes priority over `PETALTONGUE_SOCKET`
    /// and the default XDG resolution. This is the preferred way for CLI `--socket`
    /// flags to propagate without `unsafe` env mutation in Rust 2024.
    pub fn new_with_socket(
        graph: Arc<std::sync::RwLock<GraphEngine>>,
        socket_override: Option<PathBuf>,
    ) -> Result<Self, IpcServerError> {
        let family_id = socket_path::get_family_id();
        let socket_path = match socket_override {
            Some(p) => {
                if let Some(parent) = p.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| {
                        IpcServerError::SocketError(format!(
                            "Failed to create socket parent dir: {e}"
                        ))
                    })?;
                }
                p
            }
            None => socket_path::get_petaltongue_socket_path()?,
        };
        let viz_state = Arc::new(std::sync::RwLock::new(VisualizationState::new()));

        let mut handlers = RpcHandlers::new(graph, family_id.clone(), viz_state);
        handlers.rendering_awareness = Some(Arc::new(std::sync::RwLock::new(
            petal_tongue_core::RenderingAwareness::new(),
        )));
        let (callback_tx, push_thread) = crate::push_delivery::spawn_push_delivery();
        handlers.callback_tx = Some(callback_tx);
        info!("📡 PT-06: push delivery activated (callback_tx wired on RPC handlers)");

        Ok(Self {
            socket_path,
            family_id,
            handlers,
            motor_tx: None,
            tcp_port: None,
            _push_delivery_thread: push_thread,
        })
    }

    /// Attach a motor command sender so IPC motor commands are forwarded
    /// to the UI's efferent channel.
    #[must_use]
    pub fn with_motor_sender(
        mut self,
        tx: std::sync::mpsc::Sender<petal_tongue_core::MotorCommand>,
    ) -> Self {
        self.motor_tx = Some(tx.clone());
        self.handlers.motor_tx = Some(tx);
        self
    }

    /// Inject a shared `VisualizationState` so the UI can poll IPC sessions.
    #[must_use]
    pub fn with_visualization_state(
        mut self,
        viz_state: Arc<std::sync::RwLock<VisualizationState>>,
    ) -> Self {
        self.handlers.viz_state = viz_state;
        self
    }

    /// Return a handle to the shared visualization state.
    #[must_use]
    pub fn visualization_state_handle(&self) -> Arc<std::sync::RwLock<VisualizationState>> {
        Arc::clone(&self.handlers.viz_state)
    }

    /// Return a handle to the sensor stream subscriber registry.
    #[must_use]
    pub fn sensor_stream_handle(
        &self,
    ) -> Arc<std::sync::RwLock<crate::visualization_handler::SensorStreamRegistry>> {
        Arc::clone(&self.handlers.sensor_stream_subscribers)
    }

    /// Return a clone of the push-delivery callback sender (PT-06).
    ///
    /// The UI can use this sender to forward `CallbackDispatch` values from
    /// `InteractionSubscriberRegistry::broadcast()` so that subscribers with
    /// `callback_socket` receive push notifications for GUI-originated events.
    #[must_use]
    pub fn callback_sender(
        &self,
    ) -> Option<tokio::sync::mpsc::UnboundedSender<crate::visualization_handler::CallbackDispatch>>
    {
        self.handlers.callback_tx.clone()
    }

    /// Return a handle to the interaction subscriber registry.
    #[must_use]
    pub fn interaction_subscribers_handle(
        &self,
    ) -> Arc<std::sync::RwLock<crate::visualization_handler::InteractionSubscriberRegistry>> {
        Arc::clone(&self.handlers.interaction_subscribers)
    }

    /// Attach rendering awareness so IPC can serve introspection queries.
    #[must_use]
    pub fn with_rendering_awareness(
        mut self,
        awareness: Arc<std::sync::RwLock<petal_tongue_core::RenderingAwareness>>,
    ) -> Self {
        self.handlers.rendering_awareness = Some(awareness);
        self
    }

    /// Enable a TCP JSON-RPC listener alongside the Unix socket.
    ///
    /// Per `IPC_COMPLIANCE_MATRIX.md` v1.2, `server --port <PORT>` binds
    /// newline-delimited TCP JSON-RPC for mobile and cross-gate access.
    #[must_use]
    pub const fn with_tcp_port(mut self, port: u16) -> Self {
        self.tcp_port = Some(port);
        self.handlers.tcp_enabled = true;
        self
    }

    /// Start the server: bind UDS (always) and optionally TCP, then accept connections.
    ///
    /// BTSP Phase 2: when `BtspHandshakeConfig` is available from the environment,
    /// every accepted connection must complete a handshake (delegated to BearDog)
    /// before JSON-RPC is served. Development mode (no FAMILY_ID) skips handshake.
    pub async fn start(self: Arc<Self>) -> Result<(), IpcServerError> {
        let posture = crate::btsp::current_btsp_posture();
        crate::btsp::log_handshake_policy(&crate::btsp::handshake_policy(&posture));

        let btsp_config = crate::btsp::BtspHandshakeConfig::from_env().map(Arc::new);
        if let Some(ref cfg) = btsp_config {
            info!(
                "BTSP Phase 2 active: family={}, provider={}",
                cfg.family_id,
                cfg.provider_socket.display()
            );
        }

        if self.socket_path.exists() {
            let remove_result = if self.socket_path.is_file() {
                std::fs::remove_file(&self.socket_path)
            } else if self.socket_path.is_dir() {
                std::fs::remove_dir(&self.socket_path)
            } else {
                Ok(())
            };
            remove_result.map_err(|e| IpcServerError::IoError(format!("{e}")))?;
            debug!("Removed old socket: {}", self.socket_path.display());
        }

        let uds_listener = UnixListener::bind(&self.socket_path)
            .map_err(|e| IpcServerError::SocketError(format!("{e}")))?;
        info!(
            "🔌 Unix socket server listening: {}",
            self.socket_path.display()
        );
        info!("   Family ID: {}", self.family_id);

        if let Some(parent) = self.socket_path.parent() {
            let symlink_name = crate::btsp::domain_symlink_filename(&posture);
            let symlink_path = parent.join(&symlink_name);
            let _ = std::fs::remove_file(&symlink_path);
            if let Err(e) = std::os::unix::fs::symlink(&self.socket_path, &symlink_path) {
                debug!("Could not create capability symlink {symlink_name}: {e}");
            } else {
                info!(
                    "🔗 Capability symlink: {} → {}",
                    symlink_path.display(),
                    self.socket_path.display()
                );
            }
        }

        let tcp_listener = if let Some(port) = self.tcp_port {
            let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
            let listener = tokio::net::TcpListener::bind(addr)
                .await
                .map_err(|e| IpcServerError::SocketError(format!("TCP bind {addr}: {e}")))?;
            info!("🔌 TCP JSON-RPC server listening: {addr}");
            Some(listener)
        } else {
            None
        };

        loop {
            tokio::select! {
                result = uds_listener.accept() => {
                    match result {
                        Ok((stream, _addr)) => {
                            let server = Arc::clone(&self);
                            let btsp = btsp_config.clone();
                            tokio::spawn(async move {
                                if let Err(e) =
                                    Self::handle_uds_with_btsp(&server.handlers, stream, btsp).await
                                {
                                    error!("UDS connection error: {e}");
                                }
                            });
                        }
                        Err(e) => error!("Failed to accept UDS connection: {e}"),
                    }
                }
                result = async {
                    match &tcp_listener {
                        Some(l) => l.accept().await,
                        None => std::future::pending().await,
                    }
                } => {
                    match result {
                        Ok((stream, addr)) => {
                            let server = Arc::clone(&self);
                            let btsp = btsp_config.clone();
                            tokio::spawn(async move {
                                debug!("TCP connection from {addr}");
                                if let Err(e) =
                                    Self::handle_tcp_with_btsp(&server.handlers, stream, btsp).await
                                {
                                    error!("TCP connection error: {e}");
                                }
                            });
                        }
                        Err(e) => error!("Failed to accept TCP connection: {e}"),
                    }
                }
            }
        }
    }

    /// Handle a UDS connection with optional BTSP handshake.
    ///
    /// Uses the same first-byte peek as TCP and NestGate:
    /// `BufReader::fill_buf()` non-destructively reads the first byte.
    /// `{` (0x7B) → plain JSON-RPC from biomeOS composition, bypass
    /// handshake. Otherwise run BTSP. The `BufReader` is kept alive so
    /// peeked bytes flow into the subsequent read path (handshake or
    /// JSON-RPC loop) without data loss.
    async fn handle_uds_with_btsp(
        handlers: &RpcHandlers,
        stream: tokio::net::UnixStream,
        btsp_config: Option<Arc<crate::btsp::BtspHandshakeConfig>>,
    ) -> Result<(), unix_socket_connection::ConnectionError> {
        if let Some(ref cfg) = btsp_config {
            use tokio::io::AsyncBufReadExt;

            let (reader, mut writer) = stream.into_split();
            let mut buf_reader = tokio::io::BufReader::new(reader);

            let first_byte = match buf_reader.fill_buf().await {
                Ok(buf) if !buf.is_empty() => buf[0],
                Ok(_) => {
                    debug!("BTSP: UDS connection sent EOF before any data, rejecting");
                    return Ok(());
                }
                Err(e) => {
                    error!("BTSP: UDS peek failed: {e}");
                    return Ok(());
                }
            };

            if first_byte == b'{' {
                debug!("BTSP: first byte is '{{' on UDS, bypassing handshake (JSON-RPC)");
            } else {
                match crate::btsp::perform_server_handshake_split(&mut buf_reader, &mut writer, cfg)
                    .await
                {
                    Ok(session_id) => {
                        debug!("BTSP authenticated on UDS: session={session_id}");
                    }
                    Err(e) => {
                        error!("BTSP handshake failed on UDS, rejecting connection: {e}");
                        return Ok(());
                    }
                }
            }
            unix_socket_connection::handle_connection_split(handlers, buf_reader, writer).await?;
        } else {
            unix_socket_connection::handle_connection(handlers, stream).await?;
        }
        Ok(())
    }

    /// Handle a TCP connection with optional BTSP handshake.
    ///
    /// Applies the same first-byte peek as BearDog: if the first byte is `{`,
    /// treat it as plain JSON-RPC (biomeOS composition). Otherwise, run BTSP.
    async fn handle_tcp_with_btsp(
        handlers: &RpcHandlers,
        mut stream: tokio::net::TcpStream,
        btsp_config: Option<Arc<crate::btsp::BtspHandshakeConfig>>,
    ) -> Result<(), unix_socket_connection::ConnectionError> {
        if let Some(ref cfg) = btsp_config {
            let mut peek_buf = [0u8; 1];
            let n = stream.peek(&mut peek_buf).await?;
            if n == 0 {
                debug!("BTSP: TCP connection sent EOF before any data, rejecting");
                return Ok(());
            }
            if peek_buf[0] != b'{' {
                match crate::btsp::perform_server_handshake(&mut stream, cfg).await {
                    Ok(session_id) => {
                        debug!("BTSP authenticated on TCP: session={session_id}");
                    }
                    Err(e) => {
                        error!("BTSP handshake failed on TCP, rejecting connection: {e}");
                        return Ok(());
                    }
                }
            }
        }
        unix_socket_connection::handle_connection(handlers, stream).await?;
        Ok(())
    }
}

#[cfg(test)]
impl UnixSocketServer {
    /// Whether JSON-RPC handlers have a push delivery sender (PT-06).
    #[must_use]
    pub(crate) fn push_delivery_wired_for_tests(&self) -> bool {
        self.handlers.callback_tx.is_some()
    }

    fn get_capabilities(&self, id: Value) -> crate::json_rpc::JsonRpcResponse {
        self.handlers.get_capabilities(id)
    }

    fn get_health(&self, id: Value) -> crate::json_rpc::JsonRpcResponse {
        self.handlers.get_health(id)
    }

    async fn handle_request(&self, request: JsonRpcRequest) -> crate::json_rpc::JsonRpcResponse {
        self.handlers.handle_request(request).await
    }

    fn get_topology(&self, id: Value) -> crate::json_rpc::JsonRpcResponse {
        self.handlers.get_topology(id)
    }

    fn handle_health_check(&self, request: JsonRpcRequest) -> crate::json_rpc::JsonRpcResponse {
        self.handlers.handle_health_check(request)
    }

    fn handle_announce_capabilities(
        &self,
        request: JsonRpcRequest,
    ) -> crate::json_rpc::JsonRpcResponse {
        self.handlers.handle_announce_capabilities(request)
    }

    fn handle_ui_display_status(
        &self,
        request: JsonRpcRequest,
    ) -> crate::json_rpc::JsonRpcResponse {
        self.handlers.handle_ui_display_status(request)
    }

    async fn render_graph(&self, params: Value, id: Value) -> crate::json_rpc::JsonRpcResponse {
        self.handlers.render_graph(params, id).await
    }
}

impl Drop for UnixSocketServer {
    fn drop(&mut self) {
        if let Some(parent) = self.socket_path.parent() {
            let posture = crate::btsp::current_btsp_posture();
            let symlink_path = parent.join(crate::btsp::domain_symlink_filename(&posture));
            if symlink_path.symlink_metadata().is_ok()
                && let Err(e) = std::fs::remove_file(&symlink_path)
            {
                error!("Failed to remove capability symlink: {e}");
            }
        }
        if self.socket_path.exists() {
            let result = if self.socket_path.is_file() {
                std::fs::remove_file(&self.socket_path)
            } else if self.socket_path.is_dir() {
                std::fs::remove_dir(&self.socket_path)
            } else {
                Ok(())
            };
            if let Err(e) = result {
                error!("Failed to remove socket: {}", e);
            } else {
                info!("Cleaned up socket: {}", self.socket_path.display());
            }
        }
    }
}

#[cfg(test)]
#[path = "unix_socket_server_tests.rs"]
mod tests;
