// SPDX-License-Identifier: AGPL-3.0-or-later

//! Capability-based content backend for petalTongue web mode (PT-13).
//!
//! JSON-RPC client for any primal exposing `content.resolve`.
//!
//! Endpoint resolution follows a capability-first, mesh-aware tier chain:
//!
//! 1. `CONTENT_BACKEND_SOCKET`   — explicit Unix socket override
//! 2. `CONTENT_BACKEND_ENDPOINT` — explicit TCP `host:port` (cross-gate)
//! 3. `$BIOMEOS_SOCKET_DIR/{provider}-{family}.sock` convention
//! 4. Discovery service `discovery.query("content")` (mesh / cross-gate)
//!
//! TRUE PRIMAL: The backend is primal-agnostic. petalTongue discovers the content
//! provider by capability, never by primal name.

use axum::response::{Html, IntoResponse};
use std::sync::Arc;

/// Typed errors for the content backend RPC layer.
#[derive(Debug, thiserror::Error)]
pub enum ContentBackendError {
    #[error("connect({endpoint}): {source}")]
    Connect {
        endpoint: String,
        source: std::io::Error,
    },
    #[error("write: {0}")]
    Write(#[from] std::io::Error),
    #[error("serialize: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("base64 decode: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("{0}")]
    Protocol(String),
}

/// Transport for reaching the content backend.
#[derive(Debug, Clone)]
pub enum ContentEndpoint {
    /// Local or mesh-forwarded Unix domain socket.
    Unix(std::path::PathBuf),
    /// TCP `host:port` — used for cross-gate mesh routing.
    Tcp(String),
}

impl std::fmt::Display for ContentEndpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unix(p) => write!(f, "unix:{}", p.display()),
            Self::Tcp(addr) => write!(f, "tcp:{addr}"),
        }
    }
}

/// A JSON-RPC client for any primal that implements `content.resolve`.
pub struct ContentBackendClient {
    pub endpoint: ContentEndpoint,
    pub request_id: std::sync::atomic::AtomicU64,
}

impl ContentBackendClient {
    /// Resolve content backend endpoint from the environment.
    ///
    /// Resolution order (first reachable wins):
    /// 1. `CONTENT_BACKEND_SOCKET`   — explicit Unix socket path
    /// 2. `CONTENT_BACKEND_ENDPOINT` — explicit TCP `host:port` (cross-gate)
    /// 3. `$BIOMEOS_SOCKET_DIR/{provider}-{family}.sock` convention
    /// 4. Discovery service `discovery.query("content")` — mesh-aware fallback
    ///
    /// TRUE PRIMAL: The default provider name is capability-based (`content-provider`),
    /// not coupled to any specific primal identity.
    pub async fn from_env() -> Self {
        let endpoint = Self::resolve_endpoint().await;
        Self {
            endpoint,
            request_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    async fn resolve_endpoint() -> ContentEndpoint {
        // Tier 1: explicit Unix socket
        if let Ok(sock) = std::env::var(petal_tongue_core::constants::CONTENT_BACKEND_SOCKET) {
            tracing::info!(socket = %sock, "content backend: explicit socket");
            return ContentEndpoint::Unix(std::path::PathBuf::from(sock));
        }

        // Tier 2: explicit TCP endpoint (cross-gate)
        if let Ok(addr) = std::env::var(petal_tongue_core::constants::CONTENT_BACKEND_ENDPOINT) {
            tracing::info!(endpoint = %addr, "content backend: explicit TCP endpoint");
            return ContentEndpoint::Tcp(addr);
        }

        // Tier 3: socket-dir convention
        let provider = std::env::var(petal_tongue_core::constants::CONTENT_BACKEND_PROVIDER)
            .unwrap_or_else(|_| "content-provider".to_owned());
        let family = std::env::var(petal_tongue_core::constants::FAMILY_ID)
            .or_else(|_| std::env::var(petal_tongue_core::constants::PETALTONGUE_FAMILY_ID))
            .unwrap_or_else(|_| "nat0".to_owned());
        let dir = petal_tongue_core::constants::resolve_biomeos_socket_dir();
        let convention_sock = dir.join(format!("{provider}-{family}.sock"));
        if convention_sock.exists() {
            tracing::info!(socket = %convention_sock.display(), "content backend: convention socket");
            return ContentEndpoint::Unix(convention_sock);
        }

        // Tier 4: discovery service (mesh-aware)
        if let Some(ep) = Self::discover_content_endpoint().await {
            return ep;
        }

        // Fallback: convention path (will fail on connect with a clear error)
        tracing::warn!(
            socket = %convention_sock.display(),
            "content backend: no socket found, using convention path"
        );
        ContentEndpoint::Unix(convention_sock)
    }

    /// Query the discovery service for a primal with `content` capability.
    async fn discover_content_endpoint() -> Option<ContentEndpoint> {
        use petal_tongue_discovery::DiscoveryServiceClient;

        let family = std::env::var(petal_tongue_core::constants::FAMILY_ID)
            .or_else(|_| std::env::var(petal_tongue_core::constants::PETALTONGUE_FAMILY_ID))
            .ok();
        let client = DiscoveryServiceClient::discover(family.as_deref()).ok()?;
        let primals = client.discover_by_capability("content").await.ok()?;

        for primal in &primals {
            // Prefer endpoints struct with explicit transport info
            if let Some(ref eps) = primal.endpoints {
                if let Some(ref sock) = eps.unix_socket {
                    let path = std::path::PathBuf::from(sock);
                    if path.exists() {
                        tracing::info!(
                            primal = %primal.name, socket = %sock,
                            "content backend: discovered local Unix socket via mesh"
                        );
                        return Some(ContentEndpoint::Unix(path));
                    }
                }
                if let Some(ref http) = eps.http {
                    if let Some(addr) = http.strip_prefix("http://") {
                        tracing::info!(
                            primal = %primal.name, endpoint = %addr,
                            "content backend: discovered TCP endpoint via mesh"
                        );
                        return Some(ContentEndpoint::Tcp(addr.to_owned()));
                    }
                }
            }

            // Fall back to generic endpoint field
            let ep = &primal.endpoint;
            if std::path::Path::new(ep)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("sock"))
                || ep.starts_with('/')
            {
                let path = std::path::PathBuf::from(ep);
                if path.exists() {
                    tracing::info!(
                        primal = %primal.name, socket = %ep,
                        "content backend: discovered socket"
                    );
                    return Some(ContentEndpoint::Unix(path));
                }
            } else if ep.contains(':') && !ep.starts_with("http") {
                tracing::info!(
                    primal = %primal.name, endpoint = %ep,
                    "content backend: discovered TCP endpoint"
                );
                return Some(ContentEndpoint::Tcp(ep.clone()));
            }
        }

        None
    }

    pub fn next_id(&self) -> u64 {
        self.request_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    /// Call `content.resolve` — returns `(content_bytes, mime_type)` or `None`.
    pub async fn resolve(
        &self,
        path: &str,
    ) -> Result<Option<(Vec<u8>, String)>, ContentBackendError> {
        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "content.resolve",
            "params": { "path": path },
            "id": self.next_id(),
        });
        let mut line = serde_json::to_string(&req)?;
        line.push('\n');

        let resp_line = match &self.endpoint {
            ContentEndpoint::Unix(sock) => self.rpc_unix(sock, &line).await?,
            ContentEndpoint::Tcp(addr) => self.rpc_tcp(addr, &line).await?,
        };

        let resp: serde_json::Value = serde_json::from_str(&resp_line)?;

        if resp.get("error").is_some() {
            return Ok(None);
        }

        let result = resp
            .get("result")
            .ok_or_else(|| ContentBackendError::Protocol("no result field".to_owned()))?;
        let content_b64 = result
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ContentBackendError::Protocol("missing content field".to_owned()))?;
        let mime = result
            .get("mime_type")
            .and_then(|v| v.as_str())
            .unwrap_or("application/octet-stream");

        use base64::Engine as _;
        let bytes = base64::engine::general_purpose::STANDARD.decode(content_b64)?;

        Ok(Some((bytes, mime.to_owned())))
    }

    async fn rpc_unix(
        &self,
        sock: &std::path::Path,
        request: &str,
    ) -> Result<String, ContentBackendError> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
        use tokio::net::UnixStream;

        let mut stream =
            UnixStream::connect(sock)
                .await
                .map_err(|source| ContentBackendError::Connect {
                    endpoint: format!("unix:{}", sock.display()),
                    source,
                })?;
        stream.write_all(request.as_bytes()).await?;
        stream.flush().await?;

        let (reader, _) = stream.into_split();
        let mut reader = tokio::io::BufReader::new(reader);
        let mut resp = String::new();
        reader.read_line(&mut resp).await?;
        Ok(resp)
    }

    async fn rpc_tcp(&self, addr: &str, request: &str) -> Result<String, ContentBackendError> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
        use tokio::net::TcpStream;

        let mut stream =
            TcpStream::connect(addr)
                .await
                .map_err(|source| ContentBackendError::Connect {
                    endpoint: format!("tcp:{addr}"),
                    source,
                })?;
        stream.write_all(request.as_bytes()).await?;
        stream.flush().await?;

        let (reader, _) = stream.into_split();
        let mut reader = tokio::io::BufReader::new(reader);
        let mut resp = String::new();
        reader.read_line(&mut resp).await?;
        Ok(resp)
    }
}

/// Content-aware index: try `content.resolve("/")` first, fall back to
/// the compiled-in dashboard.
pub async fn content_index(client: Arc<ContentBackendClient>) -> axum::response::Response {
    match client.resolve("/").await {
        Ok(Some((body, mime))) => super::build_response(body, &mime, 0),
        _ => Html(include_str!("../../web/index.html")).into_response(),
    }
}

/// Axum fallback handler that resolves content via the backend.
pub async fn content_fallback(
    req: axum::extract::Request,
    client: Arc<ContentBackendClient>,
    nb_config: Arc<crate::notebook_render::NotebookRenderConfig>,
    cache_ttl: u64,
) -> axum::response::Response {
    let path = req.uri().path().to_owned();
    match client.resolve(&path).await {
        Ok(Some((body, mime))) => {
            if super::is_ipynb(&path) || super::is_notebook_mime(&mime) {
                if let Some(html) = crate::notebook_render::render_notebook(&body, &nb_config) {
                    return super::build_response(
                        html.into_bytes(),
                        "text/html; charset=utf-8",
                        cache_ttl,
                    );
                }
            }
            super::build_response(body, &mime, cache_ttl)
        }
        Ok(None) => (axum::http::StatusCode::NOT_FOUND, "Not Found").into_response(),
        Err(e) => {
            tracing::error!(path, error = %e, "content.resolve failed");
            (
                axum::http::StatusCode::BAD_GATEWAY,
                format!("Content backend unavailable: {e}"),
            )
                .into_response()
        }
    }
}
