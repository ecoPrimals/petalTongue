// SPDX-License-Identifier: AGPL-3.0-or-later

//! NestGate content-addressed backend for petalTongue web mode (PT-13).
//!
//! JSON-RPC client for NestGate `content.resolve` / `content.get` operations.
//! Socket discovery follows the ecosystem convention:
//! `NESTGATE_SOCKET` env → `$BIOMEOS_SOCKET_DIR/nestgate-{family}.sock`
//! → `$XDG_RUNTIME_DIR/biomeos/nestgate-default.sock`.

use std::sync::Arc;
use axum::response::{Html, IntoResponse};

pub(crate) struct NestGateContentClient {
    pub(crate) socket_path: std::path::PathBuf,
    pub(crate) request_id: std::sync::atomic::AtomicU64,
}

impl NestGateContentClient {
    /// Resolve NestGate socket from the environment.
    pub(crate) fn from_env() -> Self {
        let socket_path = std::env::var("NESTGATE_SOCKET").map_or_else(
            |_| {
                let family = std::env::var("FAMILY_ID")
                    .or_else(|_| std::env::var("PETALTONGUE_FAMILY_ID"))
                    .unwrap_or_else(|_| "default".to_owned());
                let dir = std::env::var("BIOMEOS_SOCKET_DIR").unwrap_or_else(|_| {
                    let xdg =
                        std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_owned());
                    let seg = petal_tongue_core::constants::ecosystem_runtime_dir_name();
                    format!("{xdg}/{seg}")
                });
                std::path::PathBuf::from(format!("{dir}/nestgate-{family}.sock"))
            },
            std::path::PathBuf::from,
        );
        Self {
            socket_path,
            request_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    pub(crate) fn next_id(&self) -> u64 {
        self.request_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }

    /// Call `content.resolve` — returns `(content_bytes, mime_type)` or `None`.
    pub(crate) async fn resolve(&self, path: &str) -> Result<Option<(Vec<u8>, String)>, String> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
        use tokio::net::UnixStream;

        let mut stream = UnixStream::connect(&self.socket_path)
            .await
            .map_err(|e| format!("NestGate connect({}): {e}", self.socket_path.display()))?;

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
            .map_err(|e| format!("NestGate write: {e}"))?;
        stream
            .flush()
            .await
            .map_err(|e| format!("NestGate flush: {e}"))?;

        let (reader, _) = stream.into_split();
        let mut reader = tokio::io::BufReader::new(reader);
        let mut resp_line = String::new();
        reader
            .read_line(&mut resp_line)
            .await
            .map_err(|e| format!("NestGate read: {e}"))?;

        let resp: serde_json::Value =
            serde_json::from_str(&resp_line).map_err(|e| format!("NestGate parse: {e}"))?;

        if resp.get("error").is_some() {
            return Ok(None);
        }

        let result = resp.get("result").ok_or("NestGate: no result field")?;
        let content_b64 = result
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or("NestGate: missing content field")?;
        let mime = result
            .get("mime_type")
            .and_then(|v| v.as_str())
            .unwrap_or("application/octet-stream");

        use base64::Engine as _;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(content_b64)
            .map_err(|e| format!("NestGate base64 decode: {e}"))?;

        Ok(Some((bytes, mime.to_owned())))
    }
}

/// NestGate-aware index: try `content.resolve("/")` first, fall back to
/// the compiled-in dashboard.
pub(crate) async fn nestgate_index(client: Arc<NestGateContentClient>) -> axum::response::Response {
    match client.resolve("/").await {
        Ok(Some((body, mime))) => super::build_response(body, &mime, 0),
        _ => Html(include_str!("../../web/index.html")).into_response(),
    }
}

/// Axum fallback handler that resolves content via NestGate.
pub(crate) async fn nestgate_fallback(
    req: axum::extract::Request,
    client: Arc<NestGateContentClient>,
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
            tracing::error!(path, error = %e, "NestGate content.resolve failed");
            (
                axum::http::StatusCode::BAD_GATEWAY,
                format!("NestGate backend unavailable: {e}"),
            )
                .into_response()
        }
    }
}
