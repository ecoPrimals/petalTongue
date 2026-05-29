// SPDX-License-Identifier: AGPL-3.0-or-later
//! Web mode - HTTP server with SSE push
//!
//! Pure Rust! Dependencies: axum, tower-http (100% Pure Rust)
//!
//! **IPC / PT-06:** This mode does not start the JSON-RPC Unix socket server
//! ([`petal_tongue_ipc::UnixSocketServer`]). Live updates use HTTP SSE only, not
//! `callback_tx` push over UDS.

pub mod content_backend;
mod handlers;
#[cfg(test)]
mod tests;

pub use handlers::{build_response, is_ipynb};

use crate::data_service::DataService;
use crate::error::AppError;
use handlers::{
    docroot_fallback, events_sse_handler, health_handler, index_handler, liveness_handler,
    primals_handler, readiness_handler, snapshot_handler, status_handler,
};

use std::sync::Arc;

use axum::{Router, routing::get};
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

/// Static assets directory for the web UI.
const WEB_STATIC_DIR: &str = "web/static";

/// Web server configuration for [`run`].
pub struct WebConfig<'a> {
    /// Bind address (`host:port`).
    pub bind: &'a str,
    /// Scenario JSON file to load (currently logged only).
    pub scenario: Option<String>,
    /// Static file document root for catch-all serving (`--docroot`).
    pub docroot: Option<String>,
    /// Content backend: `"filesystem"` or `"content-provider"` (capability-based).
    pub backend: &'a str,
    /// Number of tokio worker threads (wired to runtime in `main`).
    pub workers: usize,
    /// Hide code input cells when rendering `.ipynb` notebooks.
    pub strip_sources: bool,
    /// `Cache-Control: max-age=<n>` seconds for static assets.
    pub cache_ttl_secs: u64,
    /// Enable SPA mode: missing paths serve `index.html` instead of 404.
    pub spa: bool,
    /// Allowed CORS origins (empty = no CORS layer, `["*"]` = wildcard).
    pub allowed_origins: Vec<String>,
}

/// Start the web server with the given configuration and shared data service.
#[expect(
    clippy::too_many_lines,
    reason = "sequential router setup with all middleware layers"
)]
pub async fn run(cfg: WebConfig<'_>, data_service: Arc<DataService>) -> Result<(), AppError> {
    tracing::info!(
        bind = cfg.bind,
        scenario = ?cfg.scenario,
        docroot = ?cfg.docroot,
        backend = cfg.backend,
        workers = cfg.workers,
        strip_sources = cfg.strip_sources,
        cache_ttl_secs = cfg.cache_ttl_secs,
        spa = cfg.spa,
        allowed_origins = ?cfg.allowed_origins,
        "Starting web UI server (Pure Rust!)"
    );

    let addr: std::net::SocketAddr = cfg
        .bind
        .parse()
        .map_err(|e| AppError::Other(format!("Failed to parse bind address: {e}")))?;

    let nb_config = Arc::new(crate::notebook_render::NotebookRenderConfig {
        strip_sources: cfg.strip_sources,
    });
    let cache_ttl = cfg.cache_ttl_secs;
    let spa = cfg.spa;

    let mut app = Router::new()
        .route("/health", get(health_handler))
        .route("/health/liveness", get(liveness_handler))
        .route("/health/readiness", get(readiness_handler))
        .route("/api/status", get(status_handler))
        .route("/api/primals", get(primals_handler))
        .route("/api/snapshot", get(snapshot_handler))
        .route("/api/events", get(events_sse_handler))
        .nest_service("/static", ServeDir::new(WEB_STATIC_DIR));

    if cfg.backend == "content-provider" || cfg.backend == "nestgate" {
        let client = Arc::new(content_backend::ContentBackendClient::from_env());
        tracing::info!(
            socket = %client.socket_path.display(),
            "Content backend active (capability: content.resolve)"
        );
        let index_client = Arc::clone(&client);
        app = app.route(
            "/",
            get(move || content_backend::content_index(Arc::clone(&index_client))),
        );
        let nb = Arc::clone(&nb_config);
        app = app.fallback(move |req: axum::extract::Request| {
            content_backend::content_fallback(req, Arc::clone(&client), Arc::clone(&nb), cache_ttl)
        });
    } else {
        app = app.route("/", get(index_handler));
        if let Some(ref docroot) = cfg.docroot {
            let docroot_path = std::path::Path::new(docroot);
            if !docroot_path.is_dir() {
                return Err(AppError::Other(format!(
                    "--docroot path does not exist or is not a directory: {docroot}"
                )));
            }
            tracing::info!(
                docroot,
                spa,
                "Serving static files from docroot (PT-1 catch-all)"
            );
            let docroot_owned = docroot.clone();
            let nb = Arc::clone(&nb_config);
            app = app.fallback(move |req: axum::extract::Request| {
                docroot_fallback(req, docroot_owned, Arc::clone(&nb), cache_ttl, spa)
            });
        }
    }

    app = app
        .layer(CompressionLayer::new())
        .layer(axum::middleware::from_fn(security_headers_middleware))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|req: &axum::http::Request<_>| {
                    tracing::info_span!(
                        "http",
                        method = %req.method(),
                        uri = %req.uri(),
                    )
                })
                .on_response(
                    |resp: &axum::http::Response<_>,
                     latency: std::time::Duration,
                     _span: &tracing::Span| {
                        tracing::info!(
                            status = resp.status().as_u16(),
                            latency_ms = latency.as_millis() as u64,
                            "response"
                        );
                    },
                ),
        );

    let app = if cfg.allowed_origins.is_empty() {
        app.with_state(data_service)
    } else {
        let cors = build_cors_layer(&cfg.allowed_origins);
        tracing::info!(origins = ?cfg.allowed_origins, "CORS enabled");
        app.layer(cors).with_state(data_service)
    };

    tracing::info!("Web UI server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| AppError::Other(format!("Failed to bind to address: {e}")))?;

    axum::serve(listener, app)
        .with_graceful_shutdown(crate::signal::shutdown_signal())
        .await
        .map_err(|e| AppError::Other(format!("Web server error: {e}")))?;

    tracing::info!("Web server shut down gracefully");
    Ok(())
}

// ── Middleware ───────────────────────────────────────────────────────────

fn build_cors_layer(origins: &[String]) -> tower_http::cors::CorsLayer {
    use tower_http::cors::{AllowOrigin, CorsLayer};

    let cors = CorsLayer::new()
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ]);

    if origins.len() == 1 && origins[0] == "*" {
        cors.allow_origin(AllowOrigin::any())
    } else {
        let parsed: Vec<axum::http::HeaderValue> =
            origins.iter().filter_map(|o| o.parse().ok()).collect();
        cors.allow_origin(parsed)
    }
}

/// Security response headers for GitHub Pages parity (S3 shadow run).
async fn security_headers_middleware(
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let mut resp = next.run(req).await;
    let headers = resp.headers_mut();
    headers.insert(
        axum::http::header::X_CONTENT_TYPE_OPTIONS,
        axum::http::HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        axum::http::header::X_FRAME_OPTIONS,
        axum::http::HeaderValue::from_static("DENY"),
    );
    headers.insert(
        axum::http::HeaderName::from_static("referrer-policy"),
        axum::http::HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        axum::http::HeaderName::from_static("permissions-policy"),
        axum::http::HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );
    resp
}
