// SPDX-License-Identifier: AGPL-3.0-or-later
//! Mock biomeOS Topology Server
//!
//! Serves biomeOS-formatted topology data for testing petalTongue integration

use axum::{Router, extract::Path as AxumPath, http::StatusCode, response::Json, routing::get};
use petal_tongue_core::capability_names::{
    discovery_capabilities, primal_names, self_capabilities,
};
use petal_tongue_core::constants::{DEFAULT_HEADLESS_PORT, DEFAULT_WEB_PORT};
use petal_tongue_ipc::socket_path::discover_primal_socket;
use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddr};
use tower_http::cors::CorsLayer;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PrimalEndpoints {
    unix_socket: Option<String>,
    http: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PrimalMetadata {
    version: Option<String>,
    family_id: Option<String>,
    node_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Primal {
    id: String,
    name: String,
    #[serde(rename = "type")]
    primal_type: String,
    capabilities: Vec<String>,
    health: String,
    endpoints: PrimalEndpoints,
    metadata: Option<PrimalMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConnectionMetrics {
    request_count: Option<u64>,
    avg_latency_ms: Option<f64>,
    error_count: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Connection {
    from: String,
    to: String,
    #[serde(rename = "type")]
    connection_type: String,
    capability: Option<String>,
    metrics: Option<ConnectionMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TopologyResponse {
    primals: Vec<Primal>,
    connections: Vec<Connection>,
    health_status: HealthStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HealthStatus {
    overall: String,
    primals_healthy: usize,
    primals_total: usize,
}

fn mock_unix_socket_for_primal(primal_socket_base: &str) -> String {
    discover_primal_socket(primal_socket_base, None, None)
        .expect("discover_primal_socket resolves a path via XDG or /tmp fallback")
        .to_string_lossy()
        .into_owned()
}

/// Get mock topology data
async fn get_topology() -> Json<TopologyResponse> {
    let bd = primal_names::BEARDOG;
    let sb = primal_names::SONGBIRD;
    let pt = primal_names::PETALTONGUE;

    let primals = vec![
        Primal {
            id: format!("{bd}-node-alpha"),
            name: "BearDog Alpha".to_string(),
            primal_type: bd.to_string(),
            capabilities: vec![
                self_capabilities::IDENTITY_GET.to_string(),
                self_capabilities::HEALTH_CHECK.to_string(),
                self_capabilities::CAPABILITIES_LIST.to_string(),
            ],
            health: "healthy".to_string(),
            endpoints: PrimalEndpoints {
                unix_socket: Some(mock_unix_socket_for_primal(bd)),
                http: None,
            },
            metadata: Some(PrimalMetadata {
                version: Some("v0.15.2".to_string()),
                family_id: Some("nat0".to_string()),
                node_id: Some("node-alpha".to_string()),
            }),
        },
        Primal {
            id: format!("{sb}-node-alpha"),
            name: "Songbird Alpha".to_string(),
            primal_type: sb.to_string(),
            capabilities: vec![
                discovery_capabilities::IPC_DISCOVER.to_string(),
                discovery_capabilities::IPC_REGISTER.to_string(),
                discovery_capabilities::LIFECYCLE_REGISTER.to_string(),
            ],
            health: "healthy".to_string(),
            endpoints: PrimalEndpoints {
                unix_socket: Some(mock_unix_socket_for_primal(sb)),
                http: None,
            },
            metadata: Some(PrimalMetadata {
                version: Some("v3.19.0".to_string()),
                family_id: Some("nat0".to_string()),
                node_id: Some("node-alpha".to_string()),
            }),
        },
        Primal {
            id: format!("{pt}-node-alpha"),
            name: "PetalTongue Alpha".to_string(),
            primal_type: pt.to_string(),
            capabilities: vec![
                "ui.desktop-interface".to_string(),
                "visualization.graph-rendering".to_string(),
                "ui.multi-modal".to_string(),
            ],
            health: "healthy".to_string(),
            endpoints: PrimalEndpoints {
                unix_socket: Some(mock_unix_socket_for_primal(pt)),
                http: Some(format!("http://localhost:{DEFAULT_HEADLESS_PORT}")),
            },
            metadata: Some(PrimalMetadata {
                version: Some("v0.4.0".to_string()),
                family_id: Some("nat0".to_string()),
                node_id: Some("node-alpha".to_string()),
            }),
        },
    ];

    let connections = vec![Connection {
        from: format!("{sb}-node-alpha"),
        to: format!("{bd}-node-alpha"),
        connection_type: "capability_invocation".to_string(),
        capability: Some(self_capabilities::IDENTITY_GET.to_string()),
        metrics: Some(ConnectionMetrics {
            request_count: Some(42),
            avg_latency_ms: Some(2.3),
            error_count: Some(0),
        }),
    }];

    let health_status = HealthStatus {
        overall: "healthy".to_string(),
        primals_healthy: 3,
        primals_total: 3,
    };

    Json(TopologyResponse {
        primals,
        connections,
        health_status,
    })
}

/// Get health status
async fn get_health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "version": "mock-biomeos-v0.1.0",
        "uptime_seconds": 12345
    }))
}

/// Get capabilities
async fn get_capabilities() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "capabilities": [
            "topology.query",
            "primal.discovery",
            "unix-socket.json-rpc"
        ],
        "version": "mock-biomeos-v0.1.0"
    }))
}

/// Get specific primal info
async fn get_primal(AxumPath(primal_id): AxumPath<String>) -> Result<Json<Primal>, StatusCode> {
    let topology = get_topology().await.0;

    topology
        .primals
        .into_iter()
        .find(|p| p.id == primal_id)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Build router
    let app = Router::new()
        .route("/api/v1/topology", get(get_topology))
        .route("/api/v1/health", get(get_health))
        .route("/api/v1/capabilities", get(get_capabilities))
        .route("/api/v1/primals/:id", get(get_primal))
        .layer(CorsLayer::permissive());

    // Start server
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, DEFAULT_WEB_PORT));
    println!("🌱 Mock biomeOS server starting on http://{}", addr);
    println!("   Endpoints:");
    println!("   - GET  /api/v1/topology");
    println!("   - GET  /api/v1/health");
    println!("   - GET  /api/v1/capabilities");
    println!("   - GET  /api/v1/primals/:id");
    println!();
    println!("   Test with: curl http://localhost:{DEFAULT_WEB_PORT}/api/v1/topology | jq");
    println!();

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind mock-biomeos TCP listener (check DEFAULT_WEB_PORT not in use)");
    axum::serve(listener, app)
        .await
        .expect("mock-biomeos axum server exited with error");
}
