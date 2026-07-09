// SPDX-License-Identifier: AGPL-3.0-or-later
//! HTTP route handlers, static-file fallback, and shared response utilities.

use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    response::{
        Html, IntoResponse,
        sse::{Event, Sse},
    },
};
use petal_tongue_core::constants::DEFAULT_SSE_KEEPALIVE_SECS;
use tokio_stream::StreamExt as _;
use tokio_stream::wrappers::BroadcastStream;
use tower_http::services::ServeDir;

use crate::data_service::DataService;

// ── Filesystem fallback ─────────────────────────────────────────────────

/// Filesystem docroot fallback — serves static files with `.ipynb` rendering.
///
/// When `spa` is `true`, missing paths serve `{docroot}/index.html` instead of
/// 404, enabling client-side routing for single-page applications.
pub(super) async fn docroot_fallback(
    req: axum::extract::Request,
    docroot: String,
    nb_config: Arc<crate::notebook_render::NotebookRenderConfig>,
    cache_ttl: u64,
    spa: bool,
) -> axum::response::Response {
    let uri_path = req.uri().path();

    if is_ipynb(uri_path) {
        let file_path = resolve_docroot_path(&docroot, uri_path);
        match tokio::fs::read(&file_path).await {
            Ok(bytes) => {
                if let Some(html) = crate::notebook_render::render_notebook(&bytes, &nb_config) {
                    return build_response(
                        html.into_bytes(),
                        "text/html; charset=utf-8",
                        cache_ttl,
                    );
                }
                build_response(bytes, "application/json", cache_ttl)
            }
            Err(_) if spa => serve_spa_index(&docroot, cache_ttl).await,
            Err(_) => serve_custom_404(&docroot).await,
        }
    } else {
        let serve = ServeDir::new(&docroot).append_index_html_on_directories(true);
        let resp = tower::ServiceExt::oneshot(serve, req).await.map_or_else(
            |_| {
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal error",
                )
                    .into_response()
            },
            IntoResponse::into_response,
        );

        if resp.status() == axum::http::StatusCode::NOT_FOUND {
            if spa {
                return serve_spa_index(&docroot, cache_ttl).await;
            }
            return serve_custom_404(&docroot).await;
        }

        if cache_ttl > 0 && resp.status().is_success() {
            let (mut parts, body) = resp.into_parts();
            parts.headers.insert(
                axum::http::header::CACHE_CONTROL,
                format!("public, max-age={cache_ttl}")
                    .parse()
                    .unwrap_or_else(|_| axum::http::HeaderValue::from_static("public")),
            );
            axum::response::Response::from_parts(parts, body)
        } else {
            resp
        }
    }
}

/// Serve `{docroot}/index.html` for SPA catch-all routing.
async fn serve_spa_index(docroot: &str, cache_ttl: u64) -> axum::response::Response {
    let index = std::path::Path::new(docroot).join("index.html");
    tokio::fs::read(&index).await.map_or_else(
        |_| (axum::http::StatusCode::NOT_FOUND, "Not Found").into_response(),
        |bytes| build_response(bytes, "text/html; charset=utf-8", cache_ttl),
    )
}

/// Serve `{docroot}/404.html` if it exists, otherwise plain text 404.
/// GitHub Pages convention: site-level custom error page.
async fn serve_custom_404(docroot: &str) -> axum::response::Response {
    let page = std::path::Path::new(docroot).join("404.html");
    if let Ok(bytes) = tokio::fs::read(&page).await {
        let mut resp = axum::response::Response::builder()
            .status(axum::http::StatusCode::NOT_FOUND)
            .header(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(axum::body::Body::from(bytes))
            .unwrap_or_else(|_| (axum::http::StatusCode::NOT_FOUND, "Not Found").into_response());
        resp.headers_mut().insert(
            axum::http::header::CACHE_CONTROL,
            axum::http::HeaderValue::from_static("no-cache"),
        );
        return resp;
    }
    (axum::http::StatusCode::NOT_FOUND, "Not Found").into_response()
}

// ── Shared utilities ────────────────────────────────────────────────────

/// Build an HTTP response with optional `Cache-Control`.
pub fn build_response(
    body: Vec<u8>,
    content_type: &str,
    cache_ttl: u64,
) -> axum::response::Response {
    let mut builder = axum::response::Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(axum::http::header::CONTENT_TYPE, content_type);
    if cache_ttl > 0 {
        builder = builder.header(
            axum::http::header::CACHE_CONTROL,
            format!("public, max-age={cache_ttl}"),
        );
    }
    builder
        .body(axum::body::Body::from(body))
        .unwrap_or_else(|_| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "response build error",
            )
                .into_response()
        })
}

/// Map a URI path to a filesystem path under docroot, preventing traversal.
///
/// Strips leading `/`, rejects `..` path components to prevent directory
/// escape, and canonicalizes to ensure the result stays under `docroot`.
pub(super) fn resolve_docroot_path(docroot: &str, uri_path: &str) -> std::path::PathBuf {
    let cleaned = uri_path.trim_start_matches('/');
    let safe: std::path::PathBuf = std::path::Path::new(cleaned)
        .components()
        .filter(|c| matches!(c, std::path::Component::Normal(_)))
        .collect();
    std::path::Path::new(docroot).join(safe)
}

/// Case-insensitive `.ipynb` extension check.
pub fn is_ipynb(path: &str) -> bool {
    std::path::Path::new(path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("ipynb"))
}

/// Detect notebook content by MIME type (for content-addressable hash URLs
/// where the path has no extension).
pub fn is_notebook_mime(mime: &str) -> bool {
    let m = mime.split(';').next().unwrap_or(mime).trim();
    m == "application/x-ipynb+json" || m == "application/x-jupyter"
}

// ── Route handlers ──────────────────────────────────────────────────────

pub(super) async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../../web/index.html"))
}

pub(super) async fn health_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "primal": petal_tongue_core::constants::PRIMAL_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        "mode": "web",
    }))
}

pub(super) async fn liveness_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "alive",
    }))
}

pub(super) async fn readiness_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ready",
        "ready": true,
        "version": env!("CARGO_PKG_VERSION"),
        "primal": petal_tongue_core::constants::PRIMAL_NAME,
    }))
}

pub(super) async fn status_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "mode": "web",
        "pure_rust": true,
    }))
}

pub(super) async fn primals_handler(State(service): State<Arc<DataService>>) -> impl IntoResponse {
    match service.snapshot().await {
        Ok(snapshot) => Json(serde_json::json!({
            "primals": snapshot.primals,
            "timestamp": snapshot.timestamp,
        })),
        Err(e) => {
            if e.to_string().contains("Graph lock poisoned") {
                tracing::debug!("Failed to get snapshot: {}", e);
            } else {
                tracing::error!("Failed to get snapshot: {}", e);
            }
            Json(serde_json::json!({
                "error": "Failed to fetch primals",
                "primals": []
            }))
        }
    }
}

pub(super) async fn snapshot_handler(State(service): State<Arc<DataService>>) -> impl IntoResponse {
    match service.snapshot().await {
        Ok(snapshot) => Json(serde_json::json!(snapshot)),
        Err(e) => {
            if e.to_string().contains("Graph lock poisoned") {
                tracing::debug!("Failed to get snapshot: {e}");
            } else {
                tracing::error!("Failed to get snapshot: {e}");
            }
            Json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    }
}

/// SSE endpoint that pushes `DataUpdate` events from `DataService::subscribe()`.
///
/// Per PT-02 / `IPC_COMPLIANCE_MATRIX.md` v1.2: the browser receives live
/// topology changes without polling.
pub(super) async fn events_sse_handler(
    State(service): State<Arc<DataService>>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let rx = service.subscribe();
    let service = Arc::clone(&service);

    let stream = BroadcastStream::new(rx).filter_map(move |msg| {
        let service = Arc::clone(&service);
        match msg {
            Ok(_update) => {
                let snapshot = service.snapshot_sync();
                match serde_json::to_string(&snapshot) {
                    Ok(json) => Some(Ok(Event::default().data(json))),
                    Err(e) => {
                        tracing::warn!("SSE serialization error: {e}");
                        None
                    }
                }
            }
            Err(_lagged) => None,
        }
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(DEFAULT_SSE_KEEPALIVE_SECS))
            .text("keepalive"),
    )
}

// ── Gate mesh status ─────────────────────────────────────────────────────

/// Returns gate mesh topology as JSON (mirrors `gate.mesh.status` IPC method).
pub(super) async fn gate_mesh_handler() -> Json<serde_json::Value> {
    use petal_tongue_core::gate_mesh;

    let gates: Vec<serde_json::Value> = gate_mesh::all_nodes()
        .map(|node| {
            serde_json::json!({
                "id": node.id,
                "label": node.label,
                "zone": node.zone,
                "lan_ip": node.lan_ip,
                "wg_ip": node.wg_ip,
                "enrollment": node.enrollment.as_str(),
                "nucleus_count": node.nucleus_count,
                "role": node.role,
                "kderm_layer": node.kderm_layer,
                "gpu_target": node.gpu_target,
            })
        })
        .collect();

    let links: Vec<serde_json::Value> = gate_mesh::WG_LINKS
        .iter()
        .map(|link| {
            serde_json::json!({
                "from": link.from,
                "to": link.to,
                "latency_ms": link.latency_ms,
            })
        })
        .collect();

    let enrolled = gates
        .iter()
        .filter(|g| g["enrollment"] == "Enrolled")
        .count();

    Json(serde_json::json!({
        "gates": gates,
        "links": links,
        "enrolled_count": enrolled,
        "total_count": gates.len(),
        "source": "static",
    }))
}

// ── Ecosystem composition ────────────────────────────────────────────────

/// Returns the NUCLEUS composition (4 atomics, 13 primals) and ecosystem metrics.
pub(super) async fn ecosystem_handler() -> Json<serde_json::Value> {
    use petal_tongue_core::gate_mesh;

    let atomics: Vec<serde_json::Value> = gate_mesh::NUCLEUS_ATOMICS
        .iter()
        .map(|atomic| {
            let primals: Vec<serde_json::Value> = atomic
                .primals
                .iter()
                .map(|p| {
                    serde_json::json!({
                        "id": p.id,
                        "role": p.role,
                        "gate": p.gate,
                    })
                })
                .collect();
            serde_json::json!({
                "name": atomic.name,
                "primals": primals,
            })
        })
        .collect();

    let gpu_nodes: Vec<serde_json::Value> = gate_mesh::gpu_nodes()
        .map(|n| {
            serde_json::json!({
                "gate": n.id,
                "target": n.gpu_target,
                "enrolled": n.enrollment == gate_mesh::GateEnrollment::Enrolled,
            })
        })
        .collect();

    Json(serde_json::json!({
        "nucleus": atomics,
        "compute": {
            "gpu_nodes": gpu_nodes,
            "primary_gate": "ironGate",
        },
        "metrics": {
            "total_primals": gate_mesh::nucleus_primal_count(),
            "total_atomics": gate_mesh::NUCLEUS_ATOMICS.len(),
            "gates_enrolled": gate_mesh::count_by_enrollment(gate_mesh::GateEnrollment::Enrolled),
            "gpu_capable": gate_mesh::gpu_nodes().count(),
            "source": "static",
        },
    }))
}

// ── Physical topology ────────────────────────────────────────────────────

/// Returns the physical network topology (LAN, switches, edge router, port forwards).
pub(super) async fn physical_topology_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "outer_membrane": {
            "name": "Cloudflare",
            "role": "K-Derm outer membrane (DDoS, TLS edge)",
            "domains": ["lab.primals.eco", "membrane.primals.eco", "git.primals.eco"],
            "origin_flint": "162.226.225.148",
            "origin_golgi": "157.230.3.183",
        },
        "edge_router": {
            "name": "Flint H1",
            "role": "Plasma membrane (edge router)",
            "wan_ip": "162.226.225.148",
            "lan_ip": "192.168.4.1",
            "services": ["NAT", "DHCP", "DNS (91k blocklist)", "Firewall", "WiFi"],
        },
        "backbone_switch": {
            "name": "CRS310",
            "role": "L2 backbone (10G/2.5G)",
            "ports": ["sporeGate (.3)", "eastGate (.244, 10G)", "northGate", "Omada uplink"],
        },
        "bridge": {
            "name": "Flint H2",
            "role": "Bridge WiFi AP (House 2)",
            "lan_ip": "192.168.4.250",
            "services": ["WiFi AP (ApertureScience)"],
        },
        "port_forwards": {
            "target": "sporeGate (192.168.4.3)",
            "services": ["WG (51820)", "SSH (22)", "Forgejo (2222/3000)", "HTTP/S (80/443)", "TURN", "NestGate"],
        },
        "abg_compute": {
            "entry_point": "lab.primals.eco",
            "routing": "Caddy → songBird → LAN direct-connect",
            "nodes": ["ironGate (GPU, RTX 5070, JupyterHub)", "strandGate (CPU, 64-core EPYC)"],
        },
        "invariant": "songBird IS the port solver. Services bind to localhost. No ports exposed externally. Mesh handles all routing.",
    }))
}

// ── Mesh peers (songBird mesh.peers) ────────────────────────────────────

/// Returns live mesh peer connectivity state.
///
/// In production, this calls songBird's `mesh.peers` IPC method.
/// Currently returns derived static state as the offline fallback.
pub(super) async fn mesh_peers_handler(
    State(service): State<Arc<DataService>>,
) -> Json<serde_json::Value> {
    let peers: Vec<serde_json::Value> = service
        .mesh_peers()
        .iter()
        .map(|p| {
            serde_json::json!({
                "gate_id": p.gate_id,
                "status": p.status,
                "transport": p.transport,
                "latency_ms": if p.latency_ms == u32::MAX { None } else { Some(p.latency_ms) },
                "capabilities": p.capabilities,
            })
        })
        .collect();

    let connected = peers.iter().filter(|p| p["status"] == "connected").count();

    Json(serde_json::json!({
        "peers": peers,
        "connected_count": connected,
        "total_count": peers.len(),
        "source": "static_derived",
        "note": "Will be replaced by live songBird mesh.peers IPC when available",
    }))
}

// ── sporePrint validation summary ───────────────────────────────────────

/// Returns sporePrint validation summary for the ecosystem dashboard.
///
/// Aggregates validation state across known gates — test counts, coverage,
/// CI status, and last validated wave.
pub(super) async fn sporeprint_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "wave": 132,
        "posture": "tower actioned",
        "gates": [
            {
                "gate": "eastGate",
                "primals": ["petalTongue", "primalSpring", "cellMembrane", "biomeOS", "squirrel"],
                "tests": { "petalTongue": 360, "primalSpring": 1060, "cellMembrane": 913 },
                "status": "green",
            },
            {
                "gate": "flockGate",
                "primals": ["songBird", "bearDog", "skunkBat"],
                "tests": { "songBird": 8929, "bearDog": 13866, "skunkBat": 539 },
                "status": "green",
            },
            {
                "gate": "ironGate",
                "primals": ["barraCuda", "toadStool", "coralReef"],
                "tests": { "barraCuda": 4619, "toadStool": 9171, "coralReef": 3631 },
                "status": "green",
            },
            {
                "gate": "sporeGate",
                "primals": ["nestGate", "rhizoCrypt", "loamSpine", "sweetGrass"],
                "tests": { "sweetGrass": 1658 },
                "status": "green",
            },
        ],
        "totals": {
            "test_count": 44746,
            "primals_validated": 13,
            "gates_green": 4,
            "known_debt": 0,
        },
        "ci": {
            "sovereign_ci": "sporeGate",
            "targets": ["x86_64-unknown-linux-musl", "x86_64-unknown-linux-gnu"],
            "last_build": "Wave 132d",
        },
    }))
}

// ── Visualization renderer ───────────────────────────────────────────────

/// Renders a registered visualization as SVG (or JSON scene/animation).
///
/// Query params: `?format=svg` (default), `scene-json`, `animation-json`
#[expect(
    clippy::option_if_let_else,
    reason = "match arms with different response types are clearer than map_or_else"
)]
pub(super) async fn viz_handler(
    axum::extract::Path(slug): axum::extract::Path<String>,
    query: axum::extract::Query<VizQuery>,
) -> axum::response::Response {
    use crate::viz_data::VizRegistry;
    use petal_tongue_scene::modality::{ModalityCompiler, ModalityOutput, SvgCompiler};

    let registry = VizRegistry::discover(None);

    match query.format.as_deref().unwrap_or("svg") {
        "scene-json" => match registry.build_scene(&slug) {
            Some(scene) => Json(serde_json::to_value(&scene).unwrap_or_default()).into_response(),
            None => (
                axum::http::StatusCode::NOT_FOUND,
                format!("visualization '{slug}' not found"),
            )
                .into_response(),
        },
        "animation-json" => match registry.build_animation(&slug) {
            Some(anim) => Json(serde_json::to_value(&anim).unwrap_or_default()).into_response(),
            None => (
                axum::http::StatusCode::NOT_FOUND,
                format!("no animation for '{slug}'"),
            )
                .into_response(),
        },
        _ => match registry.build_scene(&slug) {
            Some(scene) => {
                let compiler = SvgCompiler;
                match compiler.compile(&scene) {
                    ModalityOutput::Svg(bytes) => {
                        let svg = String::from_utf8_lossy(bytes.as_ref()).into_owned();
                        ([(axum::http::header::CONTENT_TYPE, "image/svg+xml")], svg).into_response()
                    }
                    _ => (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "SVG compilation failed",
                    )
                        .into_response(),
                }
            }
            None => (
                axum::http::StatusCode::NOT_FOUND,
                format!("visualization '{slug}' not found"),
            )
                .into_response(),
        },
    }
}

#[derive(serde::Deserialize)]
pub(super) struct VizQuery {
    pub format: Option<String>,
}

// ── Coordination backend — reads nestGate CAS on shared filesystem ──────

fn coord_storage_base() -> std::path::PathBuf {
    if let Ok(base) = std::env::var("NESTGATE_STORAGE_BASE_PATH") {
        return std::path::PathBuf::from(base);
    }
    if let Ok(home) = std::env::var("HOME") {
        let xdg = std::path::PathBuf::from(home)
            .join(".local/share/nestgate/storage");
        if xdg.exists() {
            return xdg;
        }
    }
    std::path::PathBuf::from("/var/lib/nestgate/storage")
}

fn coord_manifest_path() -> std::path::PathBuf {
    let family = std::env::var("NESTGATE_FAMILY_ID").unwrap_or_else(|_| String::from("default"));
    coord_storage_base()
        .join("datasets")
        .join(family)
        .join("_coordination")
        .join("manifest.json")
}

fn coord_artifact_path(hash: &str) -> std::path::PathBuf {
    let family = std::env::var("NESTGATE_FAMILY_ID").unwrap_or_else(|_| String::from("default"));
    coord_storage_base()
        .join("datasets")
        .join(family)
        .join("_coordination")
        .join("artifacts")
        .join(hash)
}

fn load_coord_manifest() -> serde_json::Value {
    let path = coord_manifest_path();
    if !path.exists() {
        return serde_json::json!({
            "status": "no_data",
            "note": "No coordination data ingested yet. Run coord.ingest via nestGate JSON-RPC.",
            "artifacts": {}, "heads": {}, "blurb_history": [], "frago_history": []
        });
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| serde_json::json!({"status": "error", "note": "Failed to read manifest"}))
}

/// `GET /api/coord/blurbs` — current blurb + blurb history.
pub(super) async fn coord_blurbs_handler() -> impl IntoResponse {
    let manifest = load_coord_manifest();
    let current = manifest["current_blurb"].as_str();
    let blurbs: Vec<&serde_json::Value> = manifest["blurb_history"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|h| h.as_str().and_then(|s| manifest["artifacts"].get(s)))
        .collect();

    let current_content = current
        .and_then(|h| std::fs::read_to_string(coord_artifact_path(h)).ok());

    Json(serde_json::json!({
        "count": blurbs.len(),
        "current": current,
        "current_content": current_content,
        "blurbs": blurbs,
    }))
}

/// `GET /api/coord/waves` — current wave state + history.
pub(super) async fn coord_waves_handler() -> impl IntoResponse {
    let manifest = load_coord_manifest();
    let current_hash = manifest["current_wave"].as_str();
    let current_content = current_hash
        .and_then(|h| std::fs::read_to_string(coord_artifact_path(h)).ok());

    let history: Vec<serde_json::Value> = manifest["blurb_history"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|h| {
            let hash = h.as_str()?;
            let art = manifest["artifacts"].get(hash)?;
            Some(serde_json::json!({
                "wave": art["wave"],
                "hash": hash,
                "title": art["title"],
                "ingested_at": art["ingested_at"],
            }))
        })
        .collect();

    Json(serde_json::json!({
        "current_wave": current_hash,
        "current_content": current_content,
        "history": history,
    }))
}

/// `GET /api/coord/heads` — all gate HEAD states.
pub(super) async fn coord_heads_handler() -> impl IntoResponse {
    let manifest = load_coord_manifest();
    let heads = manifest["heads"].as_object().cloned().unwrap_or_default();
    let entries: Vec<serde_json::Value> = heads
        .iter()
        .map(|(gate, hash_val)| {
            let hash = hash_val.as_str().unwrap_or("");
            serde_json::json!({
                "gate": gate,
                "hash": hash,
                "artifact": manifest["artifacts"].get(hash),
            })
        })
        .collect();

    Json(serde_json::json!({
        "count": entries.len(),
        "heads": entries,
    }))
}

/// `GET /api/coord/fragos` — FRAGO/AAR list.
pub(super) async fn coord_fragos_handler() -> impl IntoResponse {
    let manifest = load_coord_manifest();
    let fragos: Vec<&serde_json::Value> = manifest["frago_history"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|h| h.as_str().and_then(|s| manifest["artifacts"].get(s)))
        .collect();

    Json(serde_json::json!({
        "count": fragos.len(),
        "fragos": fragos,
    }))
}

/// `GET /api/coord/topology` — mesh topology from coordination manifest.
pub(super) async fn coord_topology_handler() -> impl IntoResponse {
    let manifest = load_coord_manifest();
    let gates: Vec<&str> = manifest["heads"]
        .as_object()
        .into_iter()
        .flat_map(|m| m.keys().map(|s| s.as_str()))
        .collect();

    Json(serde_json::json!({
        "gates": gates,
        "head_count": gates.len(),
        "source": "coordination_manifest",
    }))
}

/// `GET /api/coord/depot` — depot binary inventory.
pub(super) async fn coord_depot_handler() -> impl IntoResponse {
    let depot_path = std::path::Path::new("/opt/ecoPrimals/depot");
    if !depot_path.exists() {
        return Json(serde_json::json!({
            "status": "no_depot",
            "message": "No depot directory found"
        }));
    }
    let mut binaries = Vec::new();
    if let Ok(entries) = std::fs::read_dir(depot_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let meta = std::fs::metadata(&path).ok();
                binaries.push(serde_json::json!({
                    "name": name,
                    "size": meta.as_ref().map(|m| m.len()),
                }));
            }
        }
    }
    binaries.sort_by(|a, b| {
        let an = a["name"].as_str().unwrap_or("");
        let bn = b["name"].as_str().unwrap_or("");
        an.cmp(bn)
    });
    Json(serde_json::json!({
        "binary_count": binaries.len(),
        "binaries": binaries,
    }))
}
