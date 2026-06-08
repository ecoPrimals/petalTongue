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
//! When `sourdough-core` ships the canonical `TransportEndpoint`, this module
//! should be replaced with a re-export.

use std::path::PathBuf;

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

/// A connected transport stream — either UDS or TCP.
///
/// Implements `AsyncRead` + `AsyncWrite` via delegation to the inner stream.
#[derive(Debug)]
pub enum TransportStream {
    /// Unix domain socket connection.
    Uds(tokio::net::UnixStream),
    /// TCP connection.
    Tcp(tokio::net::TcpStream),
}

impl tokio::io::AsyncRead for TransportStream {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            Self::Uds(s) => std::pin::Pin::new(s).poll_read(cx, buf),
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
            Self::Uds(s) => std::pin::Pin::new(s).poll_write(cx, buf),
            Self::Tcp(s) => std::pin::Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            Self::Uds(s) => std::pin::Pin::new(s).poll_flush(cx),
            Self::Tcp(s) => std::pin::Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            Self::Uds(s) => std::pin::Pin::new(s).poll_shutdown(cx),
            Self::Tcp(s) => std::pin::Pin::new(s).poll_shutdown(cx),
        }
    }
}

/// Connect to a `TransportEndpoint`, returning a [`TransportStream`].
///
/// For `Uds` and `Tcp`, this opens a connection. `MeshRelay` is not yet
/// implemented (returns an error).
///
/// # Errors
///
/// Returns an IO error if the connection fails.
pub async fn connect_transport(endpoint: &TransportEndpoint) -> std::io::Result<TransportStream> {
    match endpoint {
        TransportEndpoint::Uds { path } => {
            let stream = tokio::net::UnixStream::connect(path).await?;
            Ok(TransportStream::Uds(stream))
        }
        TransportEndpoint::Tcp { host, port } => {
            let stream = tokio::net::TcpStream::connect(format!("{host}:{port}")).await?;
            Ok(TransportStream::Tcp(stream))
        }
        TransportEndpoint::MeshRelay { peer_id, .. } => Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            format!("mesh_relay transport to {peer_id} not yet implemented"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::env_test_helpers;

    #[test]
    fn serialize_uds() {
        let ep = TransportEndpoint::uds("/run/user/1000/biomeos/petaltongue.sock");
        let json = serde_json::to_string(&ep).unwrap();
        assert!(json.contains("\"transport\":\"uds\""));
        assert!(json.contains("petaltongue.sock"));
    }

    #[test]
    fn serialize_tcp() {
        let ep = TransportEndpoint::tcp("127.0.0.1", 9100);
        let json = serde_json::to_string(&ep).unwrap();
        assert!(json.contains("\"transport\":\"tcp\""));
        assert!(json.contains("\"port\":9100"));
    }

    #[test]
    fn serialize_mesh_relay() {
        let ep = TransportEndpoint::MeshRelay {
            peer_id: "strandgate".into(),
            capability: "security".into(),
        };
        let json = serde_json::to_string(&ep).unwrap();
        assert!(json.contains("\"transport\":\"mesh_relay\""));
        assert!(json.contains("\"peer_id\":\"strandgate\""));
    }

    #[test]
    fn deserialize_uds() {
        let json = r#"{"transport":"uds","path":"/tmp/biomeos/test.sock"}"#;
        let ep: TransportEndpoint = serde_json::from_str(json).unwrap();
        assert_eq!(ep, TransportEndpoint::uds("/tmp/biomeos/test.sock"));
    }

    #[test]
    fn deserialize_tcp() {
        let json = r#"{"transport":"tcp","host":"192.168.1.1","port":7700}"#;
        let ep: TransportEndpoint = serde_json::from_str(json).unwrap();
        assert_eq!(ep, TransportEndpoint::tcp("192.168.1.1", 7700));
    }

    #[test]
    fn deserialize_mesh_relay() {
        let json = r#"{"transport":"mesh_relay","peer_id":"eastgate","capability":"content"}"#;
        let ep: TransportEndpoint = serde_json::from_str(json).unwrap();
        assert!(matches!(ep, TransportEndpoint::MeshRelay { .. }));
    }

    #[test]
    fn roundtrip_all_variants() {
        let endpoints = vec![
            TransportEndpoint::uds("/run/membrane/petaltongue.sock"),
            TransportEndpoint::tcp("0.0.0.0", 3000),
            TransportEndpoint::MeshRelay {
                peer_id: "westgate".into(),
                capability: "visualization".into(),
            },
        ];
        for ep in &endpoints {
            let json = serde_json::to_string(ep).unwrap();
            let parsed: TransportEndpoint = serde_json::from_str(&json).unwrap();
            assert_eq!(&parsed, ep, "roundtrip failed for {ep}");
        }
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
    fn from_env_unset() {
        env_test_helpers::with_env_vars(&[("TRANSPORT_ENDPOINT", None)], || {
            let result = TransportEndpoint::from_env().unwrap();
            assert!(result.is_none());
        });
    }

    #[test]
    fn from_env_valid_uds() {
        let json = r#"{"transport":"uds","path":"/tmp/biomeos/test.sock"}"#;
        env_test_helpers::with_env_vars(&[("TRANSPORT_ENDPOINT", Some(json))], || {
            let ep = TransportEndpoint::from_env().unwrap().unwrap();
            assert_eq!(ep, TransportEndpoint::uds("/tmp/biomeos/test.sock"));
        });
    }

    #[test]
    fn from_env_invalid_json() {
        env_test_helpers::with_env_vars(&[("TRANSPORT_ENDPOINT", Some("not json"))], || {
            let result = TransportEndpoint::from_env();
            assert!(result.is_err());
        });
    }

    #[tokio::test]
    async fn connect_transport_mesh_relay_unsupported() {
        let ep = TransportEndpoint::MeshRelay {
            peer_id: "test".into(),
            capability: "test".into(),
        };
        let result = connect_transport(&ep).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::Unsupported);
    }
}
