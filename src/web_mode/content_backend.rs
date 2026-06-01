// SPDX-License-Identifier: AGPL-3.0-or-later

//! Capability-based content backend for petalTongue web mode (PT-13).
//!
//! JSON-RPC client for any primal exposing `content.resolve` / `content.get`.
//! Socket discovery follows the capability-first pattern:
//! `CONTENT_BACKEND_SOCKET` env → ecosystem socket-dir convention
//! → `$BIOMEOS_SOCKET_DIR/{content-provider}-{family}.sock`.
//!
//! TRUE PRIMAL: The backend is primal-agnostic. petalTongue discovers the content
//! provider by capability socket, never by primal name. The default socket prefix
//! is `content-provider` (capability-based), overridable via `CONTENT_BACKEND_PROVIDER`.

use axum::response::{Html, IntoResponse};
use std::sync::Arc;

/// A JSON-RPC client for any primal that implements `content.resolve`.
pub struct ContentBackendClient {
    pub socket_path: std::path::PathBuf,
    pub request_id: std::sync::atomic::AtomicU64,
}

impl ContentBackendClient {
    /// Resolve content backend socket from the environment.
    ///
    /// Resolution order (first match wins):
    /// 1. `CONTENT_BACKEND_SOCKET` — explicit override
    /// 2. `$BIOMEOS_SOCKET_DIR/{provider}-{family}.sock` where provider is
    ///    `CONTENT_BACKEND_PROVIDER` env var (default `"content-provider"`)
    /// 3. `$XDG_RUNTIME_DIR/biomeos/{provider}-{family}.sock` fallback
    ///
    /// TRUE PRIMAL: The default provider name is capability-based (`content-provider`),
    /// not coupled to any specific primal identity. Override via `CONTENT_BACKEND_PROVIDER`
    /// or `CONTENT_BACKEND_SOCKET` for explicit routing.
    pub fn from_env() -> Self {
        let socket_path = std::env::var(petal_tongue_core::constants::CONTENT_BACKEND_SOCKET)
            .or_else(|_| std::env::var(petal_tongue_core::constants::NESTGATE_SOCKET))
            .map_or_else(
                |_| {
                    let provider =
                        std::env::var(petal_tongue_core::constants::CONTENT_BACKEND_PROVIDER)
                            .unwrap_or_else(|_| "content-provider".to_owned());
                    let family = std::env::var(petal_tongue_core::constants::FAMILY_ID)
                        .or_else(|_| {
                            std::env::var(petal_tongue_core::constants::PETALTONGUE_FAMILY_ID)
                        })
                        .unwrap_or_else(|_| "default".to_owned());
                    let dir = petal_tongue_core::constants::resolve_biomeos_socket_dir()
                        .to_string_lossy()
                        .into_owned();
                    std::path::PathBuf::from(format!("{dir}/{provider}-{family}.sock"))
                },
                std::path::PathBuf::from,
            );
        Self {
            socket_path,
            request_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    pub fn next_id(&self) -> u64 {
        self.request_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    /// Call `content.resolve` — returns `(content_bytes, mime_type)` or `None`.
    pub async fn resolve(&self, path: &str) -> Result<Option<(Vec<u8>, String)>, String> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
        use tokio::net::UnixStream;

        let mut stream = UnixStream::connect(&self.socket_path).await.map_err(|e| {
            format!(
                "content backend connect({}): {e}",
                self.socket_path.display()
            )
        })?;

        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "content.resolve",
            "params": { "path": path },
            "id": self.next_id(),
        });
        let mut line = serde_json::to_string(&req).map_err(|e| e.to_string())?;
        line.push('\n');
        stream
            .write_all(line.as_bytes())
            .await
            .map_err(|e| format!("content backend write: {e}"))?;
        stream
            .flush()
            .await
            .map_err(|e| format!("content backend flush: {e}"))?;

        let (reader, _) = stream.into_split();
        let mut reader = tokio::io::BufReader::new(reader);
        let mut resp_line = String::new();
        reader
            .read_line(&mut resp_line)
            .await
            .map_err(|e| format!("content backend read: {e}"))?;

        let resp: serde_json::Value =
            serde_json::from_str(&resp_line).map_err(|e| format!("content backend parse: {e}"))?;

        if resp.get("error").is_some() {
            return Ok(None);
        }

        let result = resp
            .get("result")
            .ok_or("content backend: no result field")?;
        let content_b64 = result
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or("content backend: missing content field")?;
        let mime = result
            .get("mime_type")
            .and_then(|v| v.as_str())
            .unwrap_or("application/octet-stream");

        use base64::Engine as _;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(content_b64)
            .map_err(|e| format!("content backend base64 decode: {e}"))?;

        Ok(Some((bytes, mime.to_owned())))
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
            if super::is_ipynb(&path) {
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
