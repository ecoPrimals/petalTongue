// SPDX-License-Identifier: AGPL-3.0-or-later

use super::handlers::*;

use std::sync::Arc;

use axum::{extract::State, response::IntoResponse};

use crate::data_service::DataService;

#[tokio::test]
async fn test_status_endpoint() {
    let response = status_handler().await.into_response();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_primals_endpoint() {
    let data_service = Arc::new(DataService::new());
    let response = primals_handler(State(data_service)).await.into_response();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_health_endpoint() {
    let response = health_handler().await.into_response();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_index_endpoint_returns_html() {
    let html = index_handler().await;
    assert!(!html.0.is_empty());
}

#[tokio::test]
async fn test_snapshot_endpoint() {
    let data_service = Arc::new(DataService::new());
    let response = snapshot_handler(State(data_service)).await.into_response();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_status_response_body() {
    let response = status_handler().await.into_response();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
    assert_eq!(json["mode"], "web");
    assert_eq!(json["pure_rust"], true);
    assert!(json["version"].is_string());
}

#[tokio::test]
async fn test_health_response_body() {
    let response = health_handler().await.into_response();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn test_primals_response_structure() {
    let data_service = Arc::new(DataService::new());
    let response = primals_handler(State(data_service)).await.into_response();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["primals"].is_array());
    assert!(json["timestamp"].is_number());
}

#[tokio::test]
async fn test_snapshot_response_structure() {
    let data_service = Arc::new(DataService::new());
    let response = snapshot_handler(State(data_service)).await.into_response();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["primals"].is_array());
    assert!(json["edges"].is_array());
    assert!(json["timestamp"].is_number());
}
#[tokio::test]
async fn test_primals_handler_snapshot_error() {
    let data_service = Arc::new(DataService::new());
    let graph = data_service.graph();
    let _ = std::thread::spawn(move || {
        let _guard = graph.write().unwrap();
        panic!("intentional panic to poison lock");
    })
    .join();

    let response = primals_handler(State(data_service)).await.into_response();
    assert_eq!(response.status(), 200);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["error"].as_str().is_some());
    assert!(json["primals"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_snapshot_handler_snapshot_error() {
    let data_service = Arc::new(DataService::new());
    let graph = data_service.graph();
    let _ = std::thread::spawn(move || {
        let _guard = graph.write().unwrap();
        panic!("intentional panic to poison lock");
    })
    .join();

    let response = snapshot_handler(State(data_service)).await.into_response();
    assert_eq!(response.status(), 200);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["error"].as_str().is_some());
}

#[tokio::test]
async fn test_index_handler_contains_doctype() {
    let html = index_handler().await;
    assert!(html.0.contains("<!DOCTYPE html>"));
    assert!(html.0.contains("petalTongue"));
}
// ── Liveness & Readiness handlers ──────────────────────────────────────

#[tokio::test]
async fn test_liveness_handler() {
    let resp = liveness_handler().await.into_response();
    assert_eq!(resp.status(), 200);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["status"], "alive");
}

#[tokio::test]
async fn test_readiness_handler() {
    let resp = readiness_handler().await.into_response();
    assert_eq!(resp.status(), 200);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["status"], "ready");
    assert_eq!(v["ready"], true);
    assert!(v["version"].as_str().is_some());
    assert!(v["primal"].as_str().is_some());
}

#[tokio::test]
async fn test_gate_mesh_endpoint_returns_topology() {
    let resp = gate_mesh_handler().await.into_response();
    assert_eq!(resp.status(), 200);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(v["gates"].as_array().is_some());
    assert!(v["links"].as_array().is_some());
    assert_eq!(v["enrolled_count"], 6);
    assert_eq!(v["total_count"], 11);
    assert_eq!(v["source"], "static");
    let gates = v["gates"].as_array().unwrap();
    assert!(gates.iter().any(|g| g["id"] == "sporeGate"));
    assert!(gates.iter().any(|g| g["id"] == "golgi"));
}

#[tokio::test]
async fn test_viz_handler_renders_gate_mesh_svg() {
    let query = axum::extract::Query(VizQuery { format: None });
    let path = axum::extract::Path("gate-mesh".to_owned());
    let resp = viz_handler(path, query).await;
    assert_eq!(resp.status(), 200);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let svg = String::from_utf8_lossy(&body);
    assert!(svg.contains("<svg"), "should return SVG content");
}

#[tokio::test]
async fn test_viz_handler_scene_json() {
    let query = axum::extract::Query(VizQuery {
        format: Some("scene-json".to_owned()),
    });
    let path = axum::extract::Path("gate-mesh".to_owned());
    let resp = viz_handler(path, query).await;
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_viz_handler_not_found() {
    let query = axum::extract::Query(VizQuery { format: None });
    let path = axum::extract::Path("nonexistent-viz".to_owned());
    let resp = viz_handler(path, query).await;
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_ecosystem_endpoint_returns_nucleus() {
    let resp = ecosystem_handler().await.into_response();
    assert_eq!(resp.status(), 200);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let atomics = v["nucleus"].as_array().expect("nucleus is array");
    assert_eq!(atomics.len(), 4);
    assert_eq!(atomics[0]["name"], "Tower Atomic");
    assert_eq!(atomics[0]["primals"].as_array().unwrap().len(), 3);
    assert_eq!(v["metrics"]["total_primals"], 13);
    assert_eq!(v["metrics"]["total_atomics"], 4);
    assert_eq!(v["metrics"]["gates_enrolled"], 6);
    assert_eq!(v["metrics"]["gpu_capable"], 2);
    assert_eq!(v["compute"]["primary_gate"], "ironGate");
    let gpu_nodes = v["compute"]["gpu_nodes"].as_array().unwrap();
    assert!(gpu_nodes.iter().any(|n| n["gate"] == "ironGate"));
}

#[tokio::test]
async fn test_physical_topology_endpoint() {
    let resp = physical_topology_handler().await.into_response();
    assert_eq!(resp.status(), 200);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["edge_router"]["name"], "Flint H1");
    assert_eq!(v["edge_router"]["wan_ip"], "162.226.225.148");
    assert_eq!(v["backbone_switch"]["name"], "CRS310");
    assert!(v["invariant"].as_str().unwrap().contains("songBird"));
    let services = v["port_forwards"]["services"].as_array().unwrap();
    assert!(services.len() >= 5);
}

#[tokio::test]
async fn test_mesh_peers_endpoint() {
    let data_service = Arc::new(DataService::new());
    let resp = mesh_peers_handler(State(data_service))
        .await
        .into_response();
    assert_eq!(resp.status(), 200);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(v["connected_count"].as_u64().unwrap() >= 4);
    assert!(v["total_count"].as_u64().unwrap() >= 6);
    let peers = v["peers"].as_array().unwrap();
    assert!(!peers.is_empty());
    let east = peers.iter().find(|p| p["gate_id"] == "eastGate").unwrap();
    assert_eq!(east["status"], "connected");
    assert_eq!(east["transport"], "local");
    let iron = peers.iter().find(|p| p["gate_id"] == "ironGate").unwrap();
    assert_eq!(iron["status"], "connected");
    assert_eq!(iron["transport"], "LAN direct");
    let flock = peers.iter().find(|p| p["gate_id"] == "flockGate").unwrap();
    assert_eq!(flock["status"], "relayed");
    let graphene = peers
        .iter()
        .find(|p| p["gate_id"] == "grapheneGate")
        .unwrap();
    assert_eq!(graphene["status"], "connected");
    assert_eq!(graphene["transport"], "ADB USB");
}

#[tokio::test]
async fn test_sporeprint_endpoint() {
    let resp = sporeprint_handler().await.into_response();
    assert_eq!(resp.status(), 200);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["wave"], 132);
    assert_eq!(v["totals"]["known_debt"], 0);
    assert!(v["totals"]["test_count"].as_u64().unwrap() > 40000);
    let gates = v["gates"].as_array().unwrap();
    assert_eq!(gates.len(), 4);
    assert_eq!(v["ci"]["sovereign_ci"], "sporeGate");
}

#[tokio::test]
async fn test_topology_layers_endpoint() {
    let resp = topology_layers_handler().await.into_response();
    assert_eq!(resp.status(), 200);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["layer_count"], 5);
    assert_eq!(v["architecture"], "diderm");
    let layers = v["layers"].as_array().unwrap();
    assert_eq!(layers[0]["name"], "Extracellular");
    assert_eq!(layers[4]["name"], "Cytoplasm");
    let hardening = &v["hardening"];
    assert!(hardening["active_count"].as_u64().unwrap() >= 4);
    assert!(hardening["total_count"].as_u64().unwrap() >= 7);
    let controls = hardening["controls"].as_array().unwrap();
    let dnssec = controls.iter().find(|c| c["id"] == "DNSSEC").unwrap();
    assert_eq!(dnssec["status"], "active");
    assert_eq!(dnssec["layer"], "Outer membrane");
}
