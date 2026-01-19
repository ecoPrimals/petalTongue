//! Web mode - HTTP/WebSocket server
//! 
//! Pure Rust! ✅
//! Dependencies: axum, tower-http (100% Pure Rust)

use anyhow::{Context, Result};
use axum::{
    Router,
    routing::get,
    response::{Html, IntoResponse},
    Json,
};
use std::net::SocketAddr;
use tower_http::services::ServeDir;

pub async fn run(bind: &str, scenario: Option<String>, workers: usize) -> Result<()> {
    tracing::info!(
        bind,
        scenario = ?scenario,
        workers,
        "Starting web UI server (Pure Rust!)"
    );
    
    let addr: SocketAddr = bind
        .parse()
        .context("Failed to parse bind address")?;
    
    // Build router
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/api/status", get(status_handler))
        .route("/api/primals", get(primals_handler))
        .nest_service("/static", ServeDir::new("web/static"));
    
    tracing::info!("🌐 Web UI server listening on http://{}", addr);
    
    // Start server (fully concurrent!)
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .context("Failed to bind to address")?;
    
    axum::serve(listener, app)
        .await
        .context("Web server error")?;
    
    Ok(())
}

async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../web/index.html"))
}

async fn status_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "mode": "web",
        "pure_rust": true,
    }))
}

async fn primals_handler() -> impl IntoResponse {
    // TODO: Integrate with discovery system
    Json(serde_json::json!({
        "primals": []
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_status_endpoint() {
        let response = status_handler().await.into_response();
        assert_eq!(response.status(), 200);
    }

    #[tokio::test]
    async fn test_primals_endpoint() {
        let response = primals_handler().await.into_response();
        assert_eq!(response.status(), 200);
    }
}

