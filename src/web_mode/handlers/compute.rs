// SPDX-License-Identifier: AGPL-3.0-or-later
//! Compute API handlers — HTTP surface for the compute trio.
//!
//! Exposes petalTongue's compute bridge over HTTP so web clients (HUD,
//! notebooks, visualizations) can request math operations. The physics
//! bridge discovers barraCuda (the math primal) at runtime via capability
//! discovery or socket scanning. toadStool abstracts GPU/CPU dispatch.
//!
//! Three kingdoms apply: Human and Agentic get compute. Fleet doesn't
//! reach these endpoints (Caddy routes fleet to scatter).

use axum::extract::Json;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};

/// Request body for statistical operations.
#[derive(Debug, Deserialize)]
pub struct StatRequest {
    /// Operation: `kde`, `smooth`, `bin`, `summary`
    pub op: String,
    /// Operation-specific parameters
    #[serde(default)]
    pub params: serde_json::Value,
}

/// Request body for tessellation operations.
#[derive(Debug, Deserialize)]
pub struct TessellateRequest {
    /// Operation: `sphere`, `cylinder`, `isosurface`
    pub op: String,
    /// Operation-specific parameters
    #[serde(default)]
    pub params: serde_json::Value,
}

/// Request body for projection operations.
#[derive(Debug, Deserialize)]
pub struct ProjectRequest {
    /// Operation: `perspective`, `lighting`
    pub op: String,
    /// Operation-specific parameters
    #[serde(default)]
    pub params: serde_json::Value,
}

/// Compute response envelope.
#[derive(Debug, Serialize)]
pub struct ComputeResponse {
    pub gpu_accelerated: bool,
    pub operation: String,
    pub result: serde_json::Value,
    pub duration_ms: f64,
}

/// GET /api/compute/status — compute trio capability discovery
pub async fn compute_status_handler() -> impl IntoResponse {
    let xdg = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());
    let biomeos_dir = format!("{xdg}/biomeos");
    let eco_dir = format!("{xdg}/ecoPrimals");

    // barraCuda primal socket locations
    let math_sock = format!("{biomeos_dir}/math.sock");
    let barracuda_sock = format!("{biomeos_dir}/barracuda.sock");
    let env_sock = std::env::var("COMPUTE_SOCKET").ok();

    let compute_available = env_sock.is_some()
        || std::path::Path::new(&math_sock).exists()
        || std::path::Path::new(&barracuda_sock).exists();

    // Scan both biomeos and ecoPrimals dirs for sockets
    let mut sockets = Vec::new();
    for dir in [&biomeos_dir, &eco_dir] {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if entry.path().extension().is_some_and(|ext| ext == "sock") {
                    if let Ok(name) = entry.file_name().into_string() {
                        sockets.push(format!("{dir}/{name}"));
                    }
                }
            }
        }
    }

    Json(serde_json::json!({
        "primal": "barraCuda",
        "compute_available": compute_available,
        "cpu_fallback": true,
        "sockets_found": sockets,
        "discovery_paths": [
            math_sock,
            barracuda_sock,
            "$COMPUTE_SOCKET env",
        ],
        "supported_operations": {
            "stat": ["mean", "std_dev", "variance", "correlation", "kde", "smooth", "bin", "summary"],
            "linalg": ["solve", "eigenvalues", "svd", "qr"],
            "spectral": ["fft", "power_spectrum", "stft"],
            "ml": ["mlp_forward", "esn_predict", "attention"],
            "tessellate": ["sphere", "cylinder", "isosurface"],
            "project": ["perspective", "lighting"],
            "physics": ["nbody"],
        },
    }))
}

/// POST /api/compute/stat — dispatch a statistical operation
pub async fn compute_stat_handler(Json(req): Json<StatRequest>) -> impl IntoResponse {
    let full_op = format!("math.stat.{}", req.op);
    let result = petal_tongue_ipc::physics_bridge::dispatch_stat(&full_op, req.params).await;
    Json(ComputeResponse {
        gpu_accelerated: result.gpu_accelerated,
        operation: result.operation,
        result: result.result,
        duration_ms: result.duration_secs * 1000.0,
    })
}

/// POST /api/compute/tessellate — dispatch a tessellation operation
pub async fn compute_tessellate_handler(Json(req): Json<TessellateRequest>) -> impl IntoResponse {
    let full_op = format!("math.tessellate.{}", req.op);
    let result =
        petal_tongue_ipc::physics_bridge::dispatch_tessellate(&full_op, req.params).await;
    Json(ComputeResponse {
        gpu_accelerated: result.gpu_accelerated,
        operation: result.operation,
        result: result.result,
        duration_ms: result.duration_secs * 1000.0,
    })
}

/// POST /api/compute/project — dispatch a projection operation
pub async fn compute_project_handler(Json(req): Json<ProjectRequest>) -> impl IntoResponse {
    let full_op = format!("math.project.{}", req.op);
    let result = petal_tongue_ipc::physics_bridge::dispatch_project(&full_op, req.params).await;
    Json(ComputeResponse {
        gpu_accelerated: result.gpu_accelerated,
        operation: result.operation,
        result: result.result,
        duration_ms: result.duration_secs * 1000.0,
    })
}
