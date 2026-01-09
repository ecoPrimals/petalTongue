//! Mock biomeOS Topology Server
//!
//! Serves biomeOS-formatted topology data for testing petalTongue integration

use axum::{
    extract::Path as AxumPath,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
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

/// Get mock topology data
async fn get_topology() -> Json<TopologyResponse> {
    let primals = vec![
        Primal {
            id: "beardog-node-alpha".to_string(),
            name: "BearDog Alpha".to_string(),
            primal_type: "beardog".to_string(),
            capabilities: vec![
                "security".to_string(),
                "encryption".to_string(),
                "identity".to_string(),
            ],
            health: "healthy".to_string(),
            endpoints: PrimalEndpoints {
                unix_socket: Some("/tmp/beardog-node-alpha.sock".to_string()),
                http: None,
            },
            metadata: Some(PrimalMetadata {
                version: Some("v0.15.2".to_string()),
                family_id: Some("nat0".to_string()),
                node_id: Some("node-alpha".to_string()),
            }),
        },
        Primal {
            id: "songbird-node-alpha".to_string(),
            name: "Songbird Alpha".to_string(),
            primal_type: "songbird".to_string(),
            capabilities: vec!["discovery".to_string(), "p2p".to_string(), "btsp".to_string()],
            health: "healthy".to_string(),
            endpoints: PrimalEndpoints {
                unix_socket: Some("/tmp/songbird-node-alpha.sock".to_string()),
                http: None,
            },
            metadata: Some(PrimalMetadata {
                version: Some("v3.19.0".to_string()),
                family_id: Some("nat0".to_string()),
                node_id: Some("node-alpha".to_string()),
            }),
        },
        Primal {
            id: "petaltongue-node-alpha".to_string(),
            name: "PetalTongue Alpha".to_string(),
            primal_type: "petaltongue".to_string(),
            capabilities: vec![
                "ui.desktop-interface".to_string(),
                "visualization.graph-rendering".to_string(),
                "ui.multi-modal".to_string(),
            ],
            health: "healthy".to_string(),
            endpoints: PrimalEndpoints {
                unix_socket: Some("/tmp/petaltongue-node-alpha.sock".to_string()),
                http: Some("http://localhost:8080".to_string()),
            },
            metadata: Some(PrimalMetadata {
                version: Some("v0.4.0".to_string()),
                family_id: Some("nat0".to_string()),
                node_id: Some("node-alpha".to_string()),
            }),
        },
    ];

    let connections = vec![Connection {
        from: "songbird-node-alpha".to_string(),
        to: "beardog-node-alpha".to_string(),
        connection_type: "capability_invocation".to_string(),
        capability: Some("encryption".to_string()),
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
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🌱 Mock biomeOS server starting on http://{}", addr);
    println!("   Endpoints:");
    println!("   - GET  /api/v1/topology");
    println!("   - GET  /api/v1/health");
    println!("   - GET  /api/v1/capabilities");
    println!("   - GET  /api/v1/primals/:id");
    println!();
    println!("   Test with: curl http://localhost:3000/api/v1/topology | jq");
    println!();

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
