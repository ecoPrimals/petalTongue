// SPDX-License-Identifier: AGPL-3.0-or-later

//! Transport endpoint abstraction (sourDough canonical standard).
//!
//! Implements the ecosystem-wide `TransportEndpoint` wire format defined by
//! sourDough Wave 100. Primals accept a `TRANSPORT_ENDPOINT` env var as JSON;
//! the launcher/Tower Atomic decides the transport — primals never self-bind
//! in production.
//!
//! Wire format (serde internally-tagged):
//! ```json
//! { "transport": "uds", "path": "/run/user/1000/biomeos/beardog.sock" }
//! { "transport": "tcp", "host": "127.0.0.1", "port": 9100 }
//! { "transport": "mesh_relay", "peer_id": "strandgate", "capability": "security" }
//! ```
//!
//! ## Capability-gated transports
//!
//! [`TransportEndpoint::MeshRelay`] is a **capability-gated transport**: it is
//! available only when the ecosystem provides the [`MESH_RELAY_CAPABILITY`]
//! (`mesh.relay`) capability at runtime. petalTongue does not know where the
//! relay lives — discovery populates [`MeshRelayConfig`] when songBird (or
//! equivalent) announces the relay gateway. Until then,
//! [`connect_transport`] returns [`TransportError::CapabilityNotDiscovered`]
//! so callers and UI can distinguish "not yet discovered" from hard failure.
//!
//! When `sourdough-core` ships the canonical `TransportEndpoint`, this module
//! should be replaced with a re-export.

use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

use thiserror::Error;

/// Capability domain required for mesh relay transport.
///
/// Populated at runtime when the ecosystem announces a mesh relay gateway.
pub const MESH_RELAY_CAPABILITY: &str = "mesh.relay";

/// Runtime mesh relay configuration, populated when [`MESH_RELAY_CAPABILITY`]
/// is discovered.
///
/// petalTongue holds self-knowledge only — the relay endpoint is never
/// hardcoded; discovery sets this via [`set_mesh_relay_config`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MeshRelayConfig {
    /// How to reach the local mesh relay gateway (UDS or TCP).
    pub relay: TransportEndpoint,
}

/// Errors from transport connection attempts.
///
/// Distinct variants allow UI layers to show appropriate messaging: capability
/// discovery pending vs relay present but unreachable.
#[derive(Debug, Error)]
pub enum TransportError {
    /// Required ecosystem capability has not been discovered at runtime.
    #[error("capability not discovered: {capability} (required for this transport)")]
    CapabilityNotDiscovered {
        /// Capability domain that must be discovered (e.g. `mesh.relay`).
        capability: String,
    },

    /// Capability was discovered but the transport could not be established.
    #[error("transport unavailable: {message}")]
    TransportUnavailable {
        /// Human-readable detail (connection failure, poisoned lock, etc.).
        message: String,
    },

    /// Underlying I/O failure for direct UDS/TCP connections.
    #[error("connection failed: {0}")]
    Io(#[from] std::io::Error),
}

impl TransportError {
    /// Whether the error indicates the mesh relay capability has not been discovered.
    #[must_use]
    pub const fn is_capability_not_discovered(&self) -> bool {
        matches!(self, Self::CapabilityNotDiscovered { .. })
    }

    /// Whether the error indicates discovery succeeded but connection failed.
    #[must_use]
    pub const fn is_transport_unavailable(&self) -> bool {
        matches!(self, Self::TransportUnavailable { .. })
    }
}

impl From<TransportError> for std::io::Error {
    fn from(err: TransportError) -> Self {
        match err {
            TransportError::Io(source) => source,
            TransportError::CapabilityNotDiscovered { capability } => Self::new(
                std::io::ErrorKind::NotFound,
                format!("capability not discovered: {capability}"),
            ),
            TransportError::TransportUnavailable { message } => {
                Self::new(std::io::ErrorKind::ConnectionRefused, message)
            }
        }
    }
}

static MESH_RELAY_CONFIG: OnceLock<RwLock<Option<MeshRelayConfig>>> = OnceLock::new();

fn mesh_relay_config_lock() -> &'static RwLock<Option<MeshRelayConfig>> {
    MESH_RELAY_CONFIG.get_or_init(|| RwLock::new(None))
}

/// Install runtime mesh relay configuration after capability discovery.
///
/// Call when the ecosystem announces [`MESH_RELAY_CAPABILITY`].
pub fn set_mesh_relay_config(config: MeshRelayConfig) {
    if let Ok(mut guard) = mesh_relay_config_lock().write() {
        tracing::info!(relay = %config.relay, "mesh relay capability discovered; config installed");
        *guard = Some(config);
    }
}

/// Clear mesh relay configuration (e.g. on capability loss or test teardown).
pub fn clear_mesh_relay_config() {
    if let Ok(mut guard) = mesh_relay_config_lock().write() {
        *guard = None;
    }
}

/// Current mesh relay configuration, if the capability has been discovered.
#[must_use]
pub fn mesh_relay_config() -> Option<MeshRelayConfig> {
    mesh_relay_config_lock()
        .read()
        .ok()
        .and_then(|guard| guard.clone())
}

/// Transport endpoint — how to reach a primal or how a primal is reached.
///
/// Parsed from `TRANSPORT_ENDPOINT` env var (JSON) or constructed
/// programmatically during discovery.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "transport", rename_all = "snake_case")]
pub enum TransportEndpoint {
    /// Unix domain socket (filesystem-authenticated).
    Uds {
        /// Absolute path to the socket file.
        path: PathBuf,
    },
    /// TCP socket (host:port).
    Tcp {
        /// Bind/connect host address.
        host: String,
        /// Port number.
        port: u16,
    },
    /// Mesh relay via songBird federation.
    MeshRelay {
        /// Remote peer identifier (gate name).
        peer_id: String,
        /// Capability to route to on the remote peer.
        capability: String,
    },
}

impl TransportEndpoint {
    /// Parse from the `TRANSPORT_ENDPOINT` environment variable.
    ///
    /// Returns `None` if the variable is unset. Returns `Err` if set but
    /// contains invalid JSON.
    ///
    /// # Errors
    ///
    /// Returns a JSON parse error if the env var value is not valid
    /// `TransportEndpoint` JSON.
    pub fn from_env() -> Result<Option<Self>, serde_json::Error> {
        std::env::var(crate::constants::TRANSPORT_ENDPOINT)
            .map_or(Ok(None), |val| serde_json::from_str(&val).map(Some))
    }

    /// Construct a UDS endpoint from a socket path.
    #[must_use]
    pub fn uds(path: impl Into<PathBuf>) -> Self {
        Self::Uds { path: path.into() }
    }

    /// Construct a TCP endpoint from host and port.
    #[must_use]
    pub fn tcp(host: impl Into<String>, port: u16) -> Self {
        Self::Tcp {
            host: host.into(),
            port,
        }
    }

    /// Whether this is a local transport (UDS).
    #[must_use]
    pub const fn is_local(&self) -> bool {
        matches!(self, Self::Uds { .. })
    }
}

impl std::fmt::Display for TransportEndpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uds { path } => write!(f, "uds:{}", path.display()),
            Self::Tcp { host, port } => write!(f, "tcp:{host}:{port}"),
            Self::MeshRelay {
                peer_id,
                capability,
            } => write!(f, "mesh:{peer_id}/{capability}"),
        }
    }
}

/// A connected transport stream — platform-appropriate local IPC or TCP.
///
/// Implements `AsyncRead` + `AsyncWrite` via delegation to the inner stream.
/// On Unix: UDS (filesystem-authenticated). On Windows: Named Pipes.
#[derive(Debug)]
pub enum TransportStream {
    /// Unix domain socket connection (Unix only).
    #[cfg(unix)]
    Uds(tokio::net::UnixStream),
    /// Named pipe connection (Windows only).
    #[cfg(windows)]
    NamedPipe(tokio::net::windows::named_pipe::NamedPipeClient),
    /// TCP connection (all platforms).
    Tcp(tokio::net::TcpStream),
}

impl tokio::io::AsyncRead for TransportStream {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Uds(s) => std::pin::Pin::new(s).poll_read(cx, buf),
            #[cfg(windows)]
            Self::NamedPipe(s) => std::pin::Pin::new(s).poll_read(cx, buf),
            Self::Tcp(s) => std::pin::Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl tokio::io::AsyncWrite for TransportStream {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Uds(s) => std::pin::Pin::new(s).poll_write(cx, buf),
            #[cfg(windows)]
            Self::NamedPipe(s) => std::pin::Pin::new(s).poll_write(cx, buf),
            Self::Tcp(s) => std::pin::Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Uds(s) => std::pin::Pin::new(s).poll_flush(cx),
            #[cfg(windows)]
            Self::NamedPipe(s) => std::pin::Pin::new(s).poll_flush(cx),
            Self::Tcp(s) => std::pin::Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            #[cfg(unix)]
            Self::Uds(s) => std::pin::Pin::new(s).poll_shutdown(cx),
            #[cfg(windows)]
            Self::NamedPipe(s) => std::pin::Pin::new(s).poll_shutdown(cx),
            Self::Tcp(s) => std::pin::Pin::new(s).poll_shutdown(cx),
        }
    }
}

/// Connect to a `TransportEndpoint`, returning a [`TransportStream`].
///
/// `Uds` and `Tcp` open direct connections. [`TransportEndpoint::MeshRelay`] is
/// capability-gated: connection is attempted only when [`mesh_relay_config`]
/// is populated after [`MESH_RELAY_CAPABILITY`] discovery; otherwise returns
/// [`TransportError::CapabilityNotDiscovered`].
///
/// # Errors
///
/// Returns [`TransportError`] if the connection fails or the mesh relay
/// capability has not been discovered.
pub async fn connect_transport(
    endpoint: &TransportEndpoint,
) -> Result<TransportStream, TransportError> {
    match endpoint {
        TransportEndpoint::Uds { .. } | TransportEndpoint::Tcp { .. } => {
            connect_direct(endpoint).await
        }
        TransportEndpoint::MeshRelay {
            peer_id,
            capability,
        } => connect_mesh_relay(peer_id, capability).await,
    }
}

async fn connect_direct(endpoint: &TransportEndpoint) -> Result<TransportStream, TransportError> {
    match endpoint {
        TransportEndpoint::Uds { path } => connect_local(path).await,
        TransportEndpoint::Tcp { host, port } => {
            let stream = tokio::net::TcpStream::connect(format!("{host}:{port}")).await?;
            Ok(TransportStream::Tcp(stream))
        }
        TransportEndpoint::MeshRelay { .. } => Err(TransportError::TransportUnavailable {
            message: "internal error: connect_direct called for mesh relay endpoint".to_owned(),
        }),
    }
}

/// Connect via platform-appropriate local IPC.
///
/// Unix: connects to the UDS path directly.
/// Windows: derives a Named Pipe name from the socket path and connects via
/// `NamedPipeClient`. Convention: last path component with `.sock` stripped,
/// prefixed with `\\.\pipe\ecoPrimals-`.
#[cfg(unix)]
async fn connect_local(path: &std::path::Path) -> Result<TransportStream, TransportError> {
    let stream = tokio::net::UnixStream::connect(path).await?;
    Ok(TransportStream::Uds(stream))
}

#[cfg(windows)]
#[expect(
    clippy::unused_async,
    reason = "async signature required for platform parity with Unix connect_local"
)]
async fn connect_local(path: &std::path::Path) -> Result<TransportStream, TransportError> {
    let pipe_name = uds_path_to_pipe_name(path);
    let client = tokio::net::windows::named_pipe::ClientOptions::new()
        .open(&pipe_name)
        .map_err(TransportError::Io)?;
    Ok(TransportStream::NamedPipe(client))
}

/// Derive a Windows Named Pipe name from a Unix socket path.
///
/// Convention follows songBird reference implementation:
/// `/run/user/1000/biomeos/petaltongue.sock` → `\\.\pipe\ecoPrimals-petaltongue`
#[cfg(windows)]
fn uds_path_to_pipe_name(path: &std::path::Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    format!(r"\\.\pipe\ecoPrimals-{stem}")
}

async fn connect_mesh_relay(
    peer_id: &str,
    capability: &str,
) -> Result<TransportStream, TransportError> {
    let config = mesh_relay_config().ok_or_else(|| {
        tracing::warn!(
            peer_id = %peer_id,
            remote_capability = %capability,
            required_capability = MESH_RELAY_CAPABILITY,
            "mesh relay transport unavailable: capability not discovered at runtime"
        );
        TransportError::CapabilityNotDiscovered {
            capability: MESH_RELAY_CAPABILITY.to_owned(),
        }
    })?;

    tracing::debug!(
        peer_id = %peer_id,
        remote_capability = %capability,
        relay = %config.relay,
        "attempting mesh relay connection via discovered gateway"
    );

    connect_direct(&config.relay)
        .await
        .map_err(|err| match err {
            TransportError::Io(source) => {
                tracing::warn!(
                    peer_id = %peer_id,
                    remote_capability = %capability,
                    relay = %config.relay,
                    error = %source,
                    "mesh relay gateway connection failed"
                );
                TransportError::TransportUnavailable {
                    message: format!("mesh relay to {peer_id}/{capability} unavailable: {source}"),
                }
            }
            other @ (TransportError::CapabilityNotDiscovered { .. }
            | TransportError::TransportUnavailable { .. }) => {
                tracing::warn!(
                    peer_id = %peer_id,
                    remote_capability = %capability,
                    relay = %config.relay,
                    error = %other,
                    "mesh relay gateway connection failed"
                );
                other
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::env_test_helpers;

    #[test]
    fn serialize_uds() -> Result<(), Box<dyn std::error::Error>> {
        let ep = TransportEndpoint::uds("/run/user/1000/biomeos/petaltongue.sock");
        let json = serde_json::to_string(&ep)?;
        assert!(json.contains("\"transport\":\"uds\""));
        assert!(json.contains("petaltongue.sock"));
        Ok(())
    }

    #[test]
    fn serialize_tcp() -> Result<(), Box<dyn std::error::Error>> {
        let ep = TransportEndpoint::tcp("127.0.0.1", 9100);
        let json = serde_json::to_string(&ep)?;
        assert!(json.contains("\"transport\":\"tcp\""));
        assert!(json.contains("\"port\":9100"));
        Ok(())
    }

    #[test]
    fn serialize_mesh_relay() -> Result<(), Box<dyn std::error::Error>> {
        let ep = TransportEndpoint::MeshRelay {
            peer_id: "strandgate".into(),
            capability: "security".into(),
        };
        let json = serde_json::to_string(&ep)?;
        assert!(json.contains("\"transport\":\"mesh_relay\""));
        assert!(json.contains("\"peer_id\":\"strandgate\""));
        Ok(())
    }

    #[test]
    fn deserialize_uds() -> Result<(), Box<dyn std::error::Error>> {
        let json = r#"{"transport":"uds","path":"/tmp/biomeos/test.sock"}"#;
        let ep: TransportEndpoint = serde_json::from_str(json)?;
        assert_eq!(ep, TransportEndpoint::uds("/tmp/biomeos/test.sock"));
        Ok(())
    }

    #[test]
    fn deserialize_tcp() -> Result<(), Box<dyn std::error::Error>> {
        let json = r#"{"transport":"tcp","host":"192.168.1.1","port":7700}"#;
        let ep: TransportEndpoint = serde_json::from_str(json)?;
        assert_eq!(ep, TransportEndpoint::tcp("192.168.1.1", 7700));
        Ok(())
    }

    #[test]
    fn deserialize_mesh_relay() -> Result<(), Box<dyn std::error::Error>> {
        let json = r#"{"transport":"mesh_relay","peer_id":"eastgate","capability":"content"}"#;
        let ep: TransportEndpoint = serde_json::from_str(json)?;
        assert!(matches!(ep, TransportEndpoint::MeshRelay { .. }));
        Ok(())
    }

    #[test]
    fn roundtrip_all_variants() -> Result<(), Box<dyn std::error::Error>> {
        let endpoints = vec![
            TransportEndpoint::uds("/run/membrane/petaltongue.sock"),
            TransportEndpoint::tcp("0.0.0.0", 3000),
            TransportEndpoint::MeshRelay {
                peer_id: "westgate".into(),
                capability: "visualization".into(),
            },
        ];
        for ep in &endpoints {
            let json = serde_json::to_string(ep)?;
            let parsed: TransportEndpoint = serde_json::from_str(&json)?;
            assert_eq!(&parsed, ep, "roundtrip failed for {ep}");
        }
        Ok(())
    }

    #[test]
    fn display_format() {
        assert_eq!(
            TransportEndpoint::uds("/tmp/test.sock").to_string(),
            "uds:/tmp/test.sock"
        );
        assert_eq!(
            TransportEndpoint::tcp("127.0.0.1", 9000).to_string(),
            "tcp:127.0.0.1:9000"
        );
        assert_eq!(
            TransportEndpoint::MeshRelay {
                peer_id: "east".into(),
                capability: "viz".into()
            }
            .to_string(),
            "mesh:east/viz"
        );
    }

    #[test]
    fn is_local() {
        assert!(TransportEndpoint::uds("/tmp/test.sock").is_local());
        assert!(!TransportEndpoint::tcp("127.0.0.1", 9000).is_local());
    }

    #[test]
    fn from_env_unset() -> Result<(), Box<dyn std::error::Error>> {
        env_test_helpers::with_env_vars(&[("TRANSPORT_ENDPOINT", None)], || {
            let result = TransportEndpoint::from_env()?;
            assert!(result.is_none());
            Ok(())
        })
    }

    #[test]
    fn from_env_valid_uds() -> Result<(), Box<dyn std::error::Error>> {
        let json = r#"{"transport":"uds","path":"/tmp/biomeos/test.sock"}"#;
        env_test_helpers::with_env_vars(&[("TRANSPORT_ENDPOINT", Some(json))], || {
            let ep = TransportEndpoint::from_env()?
                .expect("TRANSPORT_ENDPOINT should be set to valid UDS JSON");
            assert_eq!(ep, TransportEndpoint::uds("/tmp/biomeos/test.sock"));
            Ok(())
        })
    }

    #[test]
    fn from_env_invalid_json() {
        env_test_helpers::with_env_vars(&[("TRANSPORT_ENDPOINT", Some("not json"))], || {
            let result = TransportEndpoint::from_env();
            assert!(result.is_err());
        });
    }

    #[tokio::test]
    async fn connect_transport_mesh_relay_capability_not_discovered() {
        clear_mesh_relay_config();
        let ep = TransportEndpoint::MeshRelay {
            peer_id: "test".into(),
            capability: "test".into(),
        };
        let err = connect_transport(&ep).await.unwrap_err();
        assert!(err.is_capability_not_discovered());
        assert!(matches!(
            err,
            TransportError::CapabilityNotDiscovered { capability } if capability == MESH_RELAY_CAPABILITY
        ));
    }

    #[tokio::test]
    async fn connect_transport_mesh_relay_uses_discovered_config()
    -> Result<(), Box<dyn std::error::Error>> {
        clear_mesh_relay_config();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral TCP listener");
        let port = listener
            .local_addr()
            .expect("listener local address")
            .port();
        let accept = tokio::spawn(async move {
            listener.accept().await.ok();
        });

        set_mesh_relay_config(MeshRelayConfig {
            relay: TransportEndpoint::tcp("127.0.0.1", port),
        });

        let ep = TransportEndpoint::MeshRelay {
            peer_id: "strandgate".into(),
            capability: "content".into(),
        };
        let stream = connect_transport(&ep)
            .await
            .expect("connect via discovered mesh relay config");
        assert!(matches!(stream, TransportStream::Tcp(_)));

        accept.abort();
        clear_mesh_relay_config();
        Ok(())
    }

    #[tokio::test]
    async fn connect_transport_mesh_relay_gateway_unreachable() {
        clear_mesh_relay_config();
        set_mesh_relay_config(MeshRelayConfig {
            relay: TransportEndpoint::tcp("127.0.0.1", 1),
        });

        let ep = TransportEndpoint::MeshRelay {
            peer_id: "westgate".into(),
            capability: "viz".into(),
        };
        let err = connect_transport(&ep).await.unwrap_err();
        assert!(err.is_transport_unavailable());

        clear_mesh_relay_config();
    }

    #[test]
    fn mesh_relay_config_roundtrip() {
        clear_mesh_relay_config();
        assert!(mesh_relay_config().is_none());

        let config = MeshRelayConfig {
            relay: TransportEndpoint::uds("/tmp/mesh-relay.sock"),
        };
        set_mesh_relay_config(config.clone());
        assert_eq!(mesh_relay_config(), Some(config));

        clear_mesh_relay_config();
        assert!(mesh_relay_config().is_none());
    }

    #[test]
    fn transport_error_io_conversion() {
        let err = TransportError::CapabilityNotDiscovered {
            capability: MESH_RELAY_CAPABILITY.to_owned(),
        };
        let io_err: std::io::Error = err.into();
        assert_eq!(io_err.kind(), std::io::ErrorKind::NotFound);
    }
}
