//! Mock BiomeOS Server
//!
//! Simple HTTP server that serves JSON scenarios for petalTongue development.
//! Supports hot-reload when scenario files change.

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::get,
    Router,
};
use notify::{Watcher, RecursiveMode, Event};
use serde::{Deserialize, Serialize};
use std::{
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, RwLock},
};
use tower_http::cors::CorsLayer;

/// Mock BiomeOS server state
#[derive(Clone)]
struct AppState {
    /// Current scenario data
    scenario: Arc<RwLock<ScenarioData>>,
    /// Path to scenarios directory
    scenarios_dir: PathBuf,
}

/// Scenario data structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ScenarioData {
    name: String,
    description: String,
    primals: Vec<PrimalInfo>,
    topology: Vec<TopologyEdge>,
}

/// Primal information
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PrimalInfo {
    id: String,
    name: String,
    primal_type: String,
    endpoint: String,
    capabilities: Vec<String>,
    health: String,
    last_seen: u64,
}

/// Topology edge
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TopologyEdge {
    from: String,
    to: String,
    edge_type: String,
    label: Option<String>,
}

/// API response wrapper
#[derive(Debug, Clone, Serialize)]
struct DiscoveryResponse {
    primals: Vec<PrimalInfo>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mock_biomeos=info,tower_http=debug".into()),
        )
        .init();

    tracing::info!("🧪 Starting Mock BiomeOS Server");

    // Determine scenarios directory
    let scenarios_dir = std::env::current_dir()?
        .join("../scenarios");
    
    tracing::info!("Scenarios directory: {:?}", scenarios_dir);

    // Load initial scenario (try simple.json first)
    let initial_scenario = load_scenario(&scenarios_dir.join("simple.json"))
        .or_else(|_| load_scenario(&scenarios_dir.join("unhealthy.json")))
        .unwrap_or_default();

    tracing::info!("Loaded scenario: {}", initial_scenario.name);
    tracing::info!("  Primals: {}", initial_scenario.primals.len());
    tracing::info!("  Edges: {}", initial_scenario.topology.len());

    // Create shared state
    let state = AppState {
        scenario: Arc::new(RwLock::new(initial_scenario)),
        scenarios_dir: scenarios_dir.clone(),
    };

    // Set up file watcher for hot reload
    let state_clone = state.clone();
    tokio::spawn(async move {
        if let Err(e) = watch_scenarios(state_clone).await {
            tracing::error!("File watcher error: {}", e);
        }
    });

    // Build router
    let app = Router::new()
        .route("/", get(root))
        .route("/api/v1/primals", get(get_primals))
        .route("/api/v1/topology", get(get_topology))
        .route("/api/v1/health", get(get_health))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3333));
    tracing::info!("🚀 Mock BiomeOS listening on http://{}", addr);
    tracing::info!("");
    tracing::info!("Endpoints:");
    tracing::info!("  GET /api/v1/primals   - Discover primals");
    tracing::info!("  GET /api/v1/topology  - Get topology edges");
    tracing::info!("  GET /api/v1/health    - Ecosystem health");
    tracing::info!("");
    tracing::info!("Test with:");
    tracing::info!("  curl http://localhost:3333/api/v1/primals");
    tracing::info!("");
    tracing::info!("Or run petalTongue:");
    tracing::info!("  BIOMEOS_URL=http://localhost:3333 cargo run --release -p petal-tongue-ui");
    tracing::info!("");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Root endpoint - welcome message
async fn root() -> impl IntoResponse {
    Json(serde_json::json!({
        "service": "Mock BiomeOS",
        "version": "0.1.0",
        "description": "Mock BiomeOS server for petalTongue development",
        "endpoints": {
            "primals": "/api/v1/primals",
            "topology": "/api/v1/topology",
            "health": "/api/v1/health"
        }
    }))
}

/// GET /api/v1/primals - Return discovered primals
async fn get_primals(State(state): State<AppState>) -> impl IntoResponse {
    let scenario = state.scenario.read().unwrap();
    
    tracing::info!("GET /api/v1/primals - Returning {} primals", scenario.primals.len());
    
    Json(DiscoveryResponse {
        primals: scenario.primals.clone(),
    })
}

/// GET /api/v1/topology - Return topology edges
async fn get_topology(State(state): State<AppState>) -> impl IntoResponse {
    let scenario = state.scenario.read().unwrap();
    
    tracing::info!("GET /api/v1/topology - Returning {} edges", scenario.topology.len());
    
    Json(scenario.topology.clone())
}

/// GET /api/v1/health - Return ecosystem health
async fn get_health(State(state): State<AppState>) -> impl IntoResponse {
    let scenario = state.scenario.read().unwrap();
    
    let healthy = scenario.primals.iter().filter(|p| p.health == "healthy").count();
    let warning = scenario.primals.iter().filter(|p| p.health == "warning").count();
    let critical = scenario.primals.iter().filter(|p| p.health == "critical").count();
    
    tracing::info!("GET /api/v1/health - {} primals ({} healthy, {} warning, {} critical)", 
        scenario.primals.len(), healthy, warning, critical);
    
    Json(serde_json::json!({
        "status": if critical > 0 { "critical" } else if warning > 0 { "warning" } else { "healthy" },
        "primal_count": scenario.primals.len(),
        "healthy": healthy,
        "warning": warning,
        "critical": critical,
        "timestamp": chrono::Utc::now().timestamp(),
    }))
}

/// Load scenario from JSON file
fn load_scenario(path: &PathBuf) -> anyhow::Result<ScenarioData> {
    let content = std::fs::read_to_string(path)?;
    let scenario: ScenarioData = serde_json::from_str(&content)?;
    Ok(scenario)
}

/// Watch scenario files for changes and reload
async fn watch_scenarios(state: AppState) -> anyhow::Result<()> {
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            let _ = tx.blocking_send(event);
        }
    })?;
    
    watcher.watch(&state.scenarios_dir, RecursiveMode::NonRecursive)?;
    
    tracing::info!("👁️  Watching for scenario changes...");
    
    while let Some(event) = rx.recv().await {
        if let notify::EventKind::Modify(_) = event.kind {
            if let Some(path) = event.paths.first() {
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    tracing::info!("🔄 Scenario file changed: {:?}", path.file_name());
                    
                    match load_scenario(path) {
                        Ok(scenario) => {
                            *state.scenario.write().unwrap() = scenario.clone();
                            tracing::info!("✅ Reloaded scenario: {}", scenario.name);
                            tracing::info!("   Primals: {}, Edges: {}", 
                                scenario.primals.len(), scenario.topology.len());
                        }
                        Err(e) => {
                            tracing::error!("❌ Failed to reload scenario: {}", e);
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}

