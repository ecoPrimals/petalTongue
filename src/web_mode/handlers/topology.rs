// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use axum::{Json, extract::State, response::IntoResponse};

use crate::data_service::DataService;

use super::manifest::load_ecosystem_manifest;

// ── Gate mesh status ─────────────────────────────────────────────────────

/// Returns gate mesh topology as JSON (mirrors `gate.mesh.status` IPC method).
///
/// Loads topology from `ecosystem_manifest.toml` at runtime rather than
/// serving compile-time static data.
pub async fn gate_mesh_handler() -> Json<serde_json::Value> {
    use petal_tongue_core::gate_mesh::{self, MeshTopologySource};

    let source = gate_mesh::ManifestMeshTopology::discover();
    let nodes = source.nodes();
    let links = source.links();

    let gates: Vec<serde_json::Value> = nodes
        .iter()
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

    let link_json: Vec<serde_json::Value> = links
        .iter()
        .map(|link| {
            serde_json::json!({
                "from": link.from,
                "to": link.to,
                "latency_ms": link.latency_ms,
            })
        })
        .collect();

    let enrolled = source.count_by_enrollment(gate_mesh::GateEnrollment::Enrolled);

    Json(serde_json::json!({
        "gates": gates,
        "links": link_json,
        "enrolled_count": enrolled,
        "total_count": nodes.len(),
        "source": if nodes.is_empty() { "empty" } else { "manifest" },
    }))
}

// ── Ecosystem composition ────────────────────────────────────────────────

/// Returns the NUCLEUS composition and ecosystem metrics.
///
/// Reads ecosystem metadata from `ecosystem_manifest.toml` at runtime.
/// NUCLEUS atomic composition data comes from the offline-topology feature
/// when enabled, or returns empty when topology is purely runtime-discovered.
pub async fn ecosystem_handler() -> Json<serde_json::Value> {
    use petal_tongue_core::gate_mesh::{self, MeshTopologySource};

    let manifest = load_ecosystem_manifest();
    let source = gate_mesh::ManifestMeshTopology::discover();

    #[cfg(feature = "offline-topology")]
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

    #[cfg(not(feature = "offline-topology"))]
    let atomics: Vec<serde_json::Value> = Vec::new();

    let gpu_nodes: Vec<serde_json::Value> = source
        .gpu_nodes()
        .iter()
        .map(|n| {
            serde_json::json!({
                "gate": n.id,
                "target": n.gpu_target,
                "enrolled": n.enrollment == gate_mesh::GateEnrollment::Enrolled,
            })
        })
        .collect();

    let primary_gate = manifest
        .get("compute")
        .and_then(|c| c.get("primary_gate"))
        .and_then(toml::Value::as_str)
        .map_or_else(
            || {
                source
                    .gpu_nodes()
                    .first()
                    .map_or_else(|| "unknown".to_owned(), |n| n.id.clone())
            },
            String::from,
        );

    let wave = manifest
        .get("ecosystem")
        .and_then(|e| e.get("wave"))
        .and_then(toml::Value::as_integer);

    let posture = manifest
        .get("ecosystem")
        .and_then(|e| e.get("posture"))
        .and_then(toml::Value::as_str);

    let has_manifest = !manifest.is_empty();
    let nodes = source.nodes();

    Json(serde_json::json!({
        "nucleus": atomics,
        "compute": {
            "gpu_nodes": gpu_nodes,
            "primary_gate": primary_gate,
        },
        "metrics": {
            "gates_enrolled": source.count_by_enrollment(gate_mesh::GateEnrollment::Enrolled),
            "gpu_capable": source.gpu_nodes().len(),
            "total_gates": nodes.len(),
            "wave": wave,
            "posture": posture,
            "source": if has_manifest { "ecosystem_manifest" } else { "discovery_required" },
        },
    }))
}

// ── Physical topology ────────────────────────────────────────────────────

/// Returns the physical network topology derived from `ecosystem_manifest.toml`.
///
/// Reads the manifest at runtime so topology changes propagate without recompilation.
pub async fn physical_topology_handler() -> Json<serde_json::Value> {
    let manifest = load_ecosystem_manifest();
    let phys = manifest.get("physical_topology");

    let public_ip = phys
        .and_then(|p| p.get("public_ip"))
        .and_then(toml::Value::as_str)
        .unwrap_or("unknown");

    let outer_membrane = phys
        .and_then(|p| p.get("outer_membrane"))
        .and_then(toml::Value::as_str)
        .unwrap_or("Cloudflare");

    let edge_router = phys
        .and_then(|p| p.get("edge_router"))
        .and_then(toml::Value::as_str)
        .unwrap_or("Flint H1");

    let lan_subnet = phys
        .and_then(|p| p.get("lan_subnet"))
        .and_then(toml::Value::as_str)
        .unwrap_or("192.168.4.0/22");

    let public_domain = phys
        .and_then(|p| p.get("public_domain"))
        .and_then(toml::Value::as_str)
        .unwrap_or("lab.primals.eco");

    let backbone_switch = phys
        .and_then(|p| p.get("backbone_switch"))
        .and_then(toml::Value::as_str)
        .unwrap_or("CRS310");

    let port_fwd_target = phys
        .and_then(|p| p.get("port_forwards"))
        .and_then(|pf| pf.get("target"))
        .and_then(toml::Value::as_str)
        .unwrap_or("unknown");

    let port_fwd_services: Vec<&str> = phys
        .and_then(|p| p.get("port_forwards"))
        .and_then(|pf| pf.get("services"))
        .and_then(|s| s.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str())
        .collect();

    Json(serde_json::json!({
        "outer_membrane": {
            "name": outer_membrane,
            "role": "K-Derm outer membrane (DDoS, TLS edge)",
            "domain": public_domain,
        },
        "edge_router": {
            "name": edge_router,
            "role": "Plasma membrane (edge router)",
            "wan_ip": public_ip,
            "lan_subnet": lan_subnet,
        },
        "backbone_switch": {
            "name": backbone_switch,
            "role": "L2 backbone (10G/2.5G)",
        },
        "port_forwards": {
            "target": port_fwd_target,
            "services": port_fwd_services,
        },
        "abg_compute": {
            "entry_point": public_domain,
            "routing": "mesh routing via capability discovery (mesh.route)",
        },
        "invariant": "The mesh routing primal solves ports. Services bind to localhost. No ports exposed externally. Mesh handles all routing.",
        "source": "ecosystem_manifest",
    }))
}

// ── Mesh peers (songBird mesh.peers) ────────────────────────────────────

/// Returns live mesh peer connectivity state from songBird UDS.
///
/// Queries songBird's `mesh.peers` via JSON-RPC on the UDS socket.
/// Falls back to static manifest-derived peers if songBird is unavailable.
pub async fn mesh_peers_handler(
    State(service): State<Arc<DataService>>,
) -> Json<serde_json::Value> {
    use crate::data_service::mesh::query_songbird_peers;

    if let Some(live_result) = query_songbird_peers().await {
        let peers = live_result
            .get("peers")
            .and_then(|p| p.as_array())
            .cloned()
            .unwrap_or_default();
        let online = live_result
            .get("online")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);

        return Json(serde_json::json!({
            "peers": peers,
            "connected_count": online,
            "total_count": peers.len(),
            "source": "songbird_live",
        }));
    }

    let peers: Vec<serde_json::Value> = DataService::mesh_peers()
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
        "source": if service.has_neural_api() { "topology_enriched" } else { "static_derived" },
    }))
}

// ── K-Derm topology layers (TOPO-VIS) ───────────────────────────────────

/// Returns the K-Derm diderm layer topology for live visualization.
///
/// Renders all 5 layers with components, security properties, data flow,
/// and current hardening control status (Wave 136b).
pub async fn topology_layers_handler() -> Json<serde_json::Value> {
    use petal_tongue_core::gate_mesh;

    let manifest = load_ecosystem_manifest();
    let wave = manifest
        .get("ecosystem")
        .and_then(|e| e.get("wave"))
        .and_then(toml::Value::as_integer);

    let layers: Vec<serde_json::Value> = gate_mesh::KDERM_LAYERS
        .iter()
        .map(|layer| {
            serde_json::json!({
                "name": layer.name,
                "role": layer.role,
                "components": layer.components,
                "path": layer.path,
                "security": layer.security,
                "data_flow": layer.data_flow,
            })
        })
        .collect();

    let controls: Vec<serde_json::Value> = gate_mesh::HARDENING_CONTROLS
        .iter()
        .map(|c| {
            serde_json::json!({
                "id": c.id,
                "layer": c.layer,
                "status": c.status,
                "description": c.description,
            })
        })
        .collect();

    let active_count = gate_mesh::HARDENING_CONTROLS
        .iter()
        .filter(|c| c.status == gate_mesh::HardeningStatus::Active)
        .count();

    Json(serde_json::json!({
        "layers": layers,
        "layer_count": layers.len(),
        "hardening": {
            "controls": controls,
            "active_count": active_count,
            "total_count": controls.len(),
        },
        "architecture": "diderm",
        "principle": "Defense in depth, not obscurity. Outer membrane data reinforces inner membrane.",
        "wave": wave,
    }))
}

// ── Live topology (Neural API + mesh peers) ─────────────────────────────

/// Returns live topology state from Neural API discovery.
///
/// When Neural API is available and has data, returns discovered primals,
/// capability edges, and routing weights. Falls back to static mesh peer
/// data when Neural API is unavailable. This is the TOPO-VIS primary endpoint.
pub async fn live_topology_handler(
    State(service): State<Arc<DataService>>,
) -> Json<serde_json::Value> {
    let topo = service.live_topology();
    Json(serde_json::to_value(&topo).unwrap_or_default())
}

// ── sporePrint validation summary ───────────────────────────────────────

/// Returns sporePrint validation summary for the ecosystem dashboard.
///
/// Derives wave state from `ecosystem_manifest.toml` and attempts to read
/// live validation data from the coordination manifest. Falls back to
/// compiled defaults when coordination data is unavailable.
pub async fn sporeprint_handler() -> Json<serde_json::Value> {
    let manifest = load_ecosystem_manifest();

    let wave = manifest
        .get("ecosystem")
        .and_then(|e| e.get("wave"))
        .and_then(toml::Value::as_integer);

    let posture = manifest
        .get("ecosystem")
        .and_then(|e| e.get("posture"))
        .and_then(toml::Value::as_str)
        .unwrap_or("unknown")
        .to_owned();

    let nucleus_count = manifest
        .get("ecosystem")
        .and_then(|e| e.get("nucleus_count"))
        .and_then(toml::Value::as_integer)
        .unwrap_or(13);

    let gates_table = manifest.get("gates").and_then(|g| g.as_table());
    let enrolled_gates: Vec<&str> = gates_table
        .into_iter()
        .flat_map(|t| t.iter())
        .filter(|(_, v)| {
            v.get("enrollment")
                .and_then(|e| e.as_str())
                .is_some_and(|e| e == "enrolled")
        })
        .map(|(name, _)| name.as_str())
        .collect();

    let ci_gate = gates_table
        .into_iter()
        .flat_map(|t| t.iter())
        .find(|(_, v)| {
            v.get("role")
                .and_then(|r| r.as_str())
                .is_some_and(|r| r.contains("CI"))
        })
        .map_or("unknown", |(name, _)| name.as_str());

    Json(serde_json::json!({
        "wave": wave,
        "posture": posture,
        "nucleus_count": nucleus_count,
        "enrolled_gates": enrolled_gates,
        "enrolled_gate_count": enrolled_gates.len(),
        "totals": {
            "primals_validated": nucleus_count,
            "gates_enrolled": enrolled_gates.len(),
            "known_debt": 0,
        },
        "ci": {
            "sovereign_ci": ci_gate,
            "targets": ["x86_64-unknown-linux-musl", "x86_64-unknown-linux-gnu"],
        },
        "source": "ecosystem_manifest",
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
pub async fn viz_handler(
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

/// Returns per-primal health liveness state from UDS queries.
///
/// Queries `health.liveness` on each primal's UDS socket concurrently
/// with BTSP framing. Returns alive/dead status, version, and errors.
pub async fn primal_health_handler() -> Json<serde_json::Value> {
    let health = crate::data_service::health::query_all_health().await;

    let alive_count = health.iter().filter(|h| h.alive).count();

    Json(serde_json::json!({
        "primals": health,
        "alive_count": alive_count,
        "total_count": health.len(),
        "source": "uds_liveness",
    }))
}

#[derive(serde::Deserialize)]
pub struct VizQuery {
    pub format: Option<String>,
}

// ── Public Record network graph (lithoSpore data) ───────────────────────

/// Serves the actor/entity network graph for detroit.primals.eco.
///
/// Data source: reads actor/entity markdown from the publicRecord repo
/// worktree if available, otherwise returns the compiled static graph.
/// This is the NUCLEUS backing for the detroit network visualization.
pub async fn public_record_network_handler() -> Json<serde_json::Value> {
    // Try to discover the lithoSpore deployment (repo worktree on this gate).
    let repo_path = discover_public_record_path();

    if let Some(path) = repo_path {
        if let Some(graph) = build_graph_from_repo(&path) {
            return Json(graph);
        }
    }

    // Fallback: compiled static graph
    Json(static_detroit_graph())
}

fn discover_public_record_path() -> Option<std::path::PathBuf> {
    let mut candidates = vec![
        std::path::PathBuf::from("/opt/ecoPrimals/detroit/repo"),
        std::path::PathBuf::from("/opt/ecoPrimals/detroit"),
    ];
    if let Ok(home) = std::env::var("HOME") {
        candidates.push(std::path::PathBuf::from(home).join("Development/detroit"));
    }
    for candidate in candidates {
        if candidate.join("actors").is_dir() || candidate.join("entities").is_dir() {
            return Some(candidate);
        }
    }
    None
}

fn build_graph_from_repo(path: &std::path::Path) -> Option<serde_json::Value> {
    let mut nodes = Vec::new();
    let mut links = Vec::new();

    // Read actors
    let actors_dir = path.join("actors");
    if actors_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&actors_dir) {
            for entry in entries.flatten() {
                let fp = entry.path();
                if fp.extension().is_some_and(|e| e == "md") {
                    if let Some(node) = parse_actor_md(&fp) {
                        nodes.push(node);
                    }
                }
            }
        }
    }

    // Read entities
    let entities_dir = path.join("entities");
    if entities_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&entities_dir) {
            for entry in entries.flatten() {
                let fp = entry.path();
                if fp.extension().is_some_and(|e| e == "md")
                    && !fp.file_name().is_some_and(|f| f == "ENTITY_MAP.md")
                {
                    if let Some(node) = parse_entity_md(&fp) {
                        nodes.push(node);
                    }
                }
            }
        }
    }

    // Read ENTITY_MAP for links if available
    let entity_map = path.join("entities/ENTITY_MAP.md");
    if entity_map.is_file() {
        if let Ok(content) = std::fs::read_to_string(&entity_map) {
            links = parse_entity_map_links(&content, &nodes);
        }
    }

    if nodes.is_empty() {
        return None;
    }

    Some(serde_json::json!({
        "nodes": nodes,
        "links": links,
        "source": "lithoSpore",
        "repo": path.to_string_lossy(),
    }))
}

fn parse_actor_md(path: &std::path::Path) -> Option<serde_json::Value> {
    let content = std::fs::read_to_string(path).ok()?;
    let stem = path.file_stem()?.to_string_lossy().to_string();

    // Extract title from first # heading
    let label = content
        .lines()
        .find(|l| l.starts_with("# "))
        .map(|l| l.trim_start_matches("# ").to_string())
        .unwrap_or_else(|| stem.replace('_', " "));

    // Detect tier from content
    let tier = if content.contains("Tier 1") { 1 }
        else if content.contains("Tier 2") { 2 }
        else if content.contains("Tier 3") { 3 }
        else if content.contains("judge") || content.contains("Judge") { 2 }
        else { 1 };

    let node_type = if content.contains("Judge") || content.contains("judge") {
        "judge"
    } else if content.contains("BMF") || content.contains("Black Mafia") {
        "bmf"
    } else if content.contains("Representative") || content.contains("legislat") || content.contains("political") {
        "political"
    } else {
        "actor"
    };

    // First paragraph after heading as detail
    let detail = content
        .lines()
        .skip_while(|l| l.starts_with('#') || l.is_empty())
        .find(|l| !l.is_empty() && !l.starts_with('#'))
        .unwrap_or("")
        .to_string();

    Some(serde_json::json!({
        "id": stem.to_lowercase().replace(' ', "_"),
        "label": label,
        "tier": tier,
        "type": node_type,
        "detail": truncate(&detail, 80),
        "file": path.file_name().map(|f| f.to_string_lossy().to_string()),
    }))
}

fn parse_entity_md(path: &std::path::Path) -> Option<serde_json::Value> {
    let content = std::fs::read_to_string(path).ok()?;
    let stem = path.file_stem()?.to_string_lossy().to_string();

    let label = content
        .lines()
        .find(|l| l.starts_with("# "))
        .map(|l| l.trim_start_matches("# ").to_string())
        .unwrap_or_else(|| stem.replace('_', " "));

    let entity_type = if content.contains("school") || content.contains("Academy") || content.contains("Prep") {
        "school"
    } else {
        "entity"
    };

    let detail = content
        .lines()
        .skip_while(|l| l.starts_with('#') || l.is_empty())
        .find(|l| !l.is_empty() && !l.starts_with('#'))
        .unwrap_or("")
        .to_string();

    Some(serde_json::json!({
        "id": stem.to_lowercase().replace(' ', "_"),
        "label": label,
        "tier": 0,
        "type": entity_type,
        "detail": truncate(&detail, 80),
        "file": path.file_name().map(|f| f.to_string_lossy().to_string()),
    }))
}

fn parse_entity_map_links(content: &str, nodes: &[serde_json::Value]) -> Vec<serde_json::Value> {
    let mut links = Vec::new();
    let node_ids: Vec<String> = nodes
        .iter()
        .filter_map(|n| n.get("id").and_then(|v| v.as_str()).map(String::from))
        .collect();

    // Look for "→" or "-->" arrows in entity map markdown
    for line in content.lines() {
        if let Some(arrow_idx) = line.find("→").or_else(|| line.find("-->")) {
            let before = &line[..arrow_idx].trim().to_lowercase().replace(' ', "_");
            let after_start = if line[arrow_idx..].starts_with("→") {
                arrow_idx + "→".len()
            } else {
                arrow_idx + 3
            };
            let after = &line[after_start..].trim().to_lowercase().replace(' ', "_");

            let source = node_ids.iter().find(|id| before.contains(id.as_str()));
            let target = node_ids.iter().find(|id| after.contains(id.as_str()));

            if let (Some(s), Some(t)) = (source, target) {
                let link_type = if line.contains('$') || line.contains("fee") || line.contains("pay") {
                    "money"
                } else if line.contains("board") || line.contains("Board") {
                    "controls"
                } else {
                    "associate"
                };
                links.push(serde_json::json!({
                    "source": s,
                    "target": t,
                    "type": link_type,
                    "label": truncate(line.trim(), 60),
                }));
            }
        }
    }
    links
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max { s.to_string() }
    else { format!("{}…", &s[..max]) }
}

// ── Public Record Timeline API ─────────────────────────────────────

/// Serve timeline events for the detroit public-record investigation.
/// Parses the timeline markdown from the lithoSpore deployment (repo).
pub async fn public_record_timeline_handler() -> Json<serde_json::Value> {
    let repo_path = discover_public_record_path();

    if let Some(path) = repo_path {
        // Try site content first (Zola source), then repo root
        let timeline_candidates = [
            path.join("site/content/timeline/_index.md"),
            path.join("timeline.md"),
        ];
        for candidate in &timeline_candidates {
            if candidate.exists() {
                if let Some(events) = parse_timeline_md(candidate) {
                    return Json(serde_json::json!({
                        "events": events,
                        "source": "repo",
                    }));
                }
            }
        }
    }

    // Static fallback
    Json(static_detroit_timeline())
}

/// Parse timeline markdown tables into structured events.
fn parse_timeline_md(path: &std::path::Path) -> Option<Vec<serde_json::Value>> {
    let content = std::fs::read_to_string(path).ok()?;
    let mut events = Vec::new();
    let mut current_era = String::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Track section headers as eras
        if trimmed.starts_with("## ") {
            current_era = trimmed.trim_start_matches("## ").to_string();
            continue;
        }

        // Parse markdown table rows: | Date | Event | Source |
        if trimmed.starts_with('|') && !trimmed.contains("---") && !trimmed.contains("Date") {
            let cols: Vec<&str> = trimmed
                .split('|')
                .filter(|s| !s.is_empty())
                .map(|s| s.trim())
                .collect();

            if cols.len() >= 2 {
                let date_raw = cols[0].replace("**", "");
                let event_raw = cols[1].replace("**", "");
                let source = if cols.len() >= 3 {
                    cols[2].replace("**", "")
                } else {
                    String::new()
                };

                // Determine which network nodes are involved
                let actors = detect_actors(&event_raw);
                let is_upcoming = current_era.contains("Upcoming")
                    || event_raw.contains("scheduled")
                    || event_raw.contains("Prepared");

                events.push(serde_json::json!({
                    "date": date_raw,
                    "event": event_raw,
                    "source": source,
                    "era": current_era,
                    "actors": actors,
                    "upcoming": is_upcoming,
                }));
            }
        }
    }

    if events.is_empty() { None } else { Some(events) }
}

/// Detect which network actors/entities are mentioned in an event.
fn detect_actors(text: &str) -> Vec<String> {
    let mut actors = Vec::new();
    let lower = text.to_lowercase();

    let patterns = [
        ("banks", "banks"),
        ("holland", "holland"),
        ("miller", "miller"),
        ("yancey", "yancey"),
        ("sabree", "sabree"),
        ("perkins", "perkins_d"),
        ("gay-dagnogo", "gay_dagnogo"),
        ("pca", "pca"),
        ("purpose charter", "pca"),
        ("macdowell", "macdowell"),
        ("purpose group", "purpose_group"),
        ("purpose foundation", "purpose_foundation"),
        ("banks strategy", "banks_strategy"),
        ("pac", "pacs"),
        ("bmf", "od_banks"),
        ("od banks", "od_banks"),
        ("welch", "welch"),
        ("dpscd", "gay_dagnogo"),
    ];

    for (pattern, actor_id) in &patterns {
        if lower.contains(pattern) && !actors.contains(&actor_id.to_string()) {
            actors.push(actor_id.to_string());
        }
    }

    actors
}

fn static_detroit_timeline() -> serde_json::Value {
    serde_json::json!({
        "events": [
            { "date": "Nov 1998", "event": "Banks convicted — NSF Check, Lincoln Park",
              "era": "Pre-2017: Criminal History", "actors": ["banks"], "source": "ICHAT" },
            { "date": "Apr 1999", "event": "Banks convicted — 3 felonies (U&P + 2×FTD), Oakland",
              "era": "Pre-2017: Criminal History", "actors": ["banks"], "source": "ICHAT" },
            { "date": "2005", "event": "BMF federal indictment — OD Banks is Defendant #22",
              "era": "Pre-2017: Criminal History", "actors": ["od_banks", "banks"], "source": "PACER" },
            { "date": "2014", "event": "Banks runs for MI Senate; Bank on Banks PAC; Holland as Treasurer",
              "era": "2014–2016: Political Career", "actors": ["banks", "holland", "pacs"], "source": "MI Campaign Finance" },
            { "date": "2017", "event": "Banks Strategy & Consultants, LLC filed",
              "era": "2017: The Pivot", "actors": ["banks", "banks_strategy"], "source": "LARA" },
            { "date": "Nov 2024", "event": "The Purpose Group, LLC filed",
              "era": "2024: Entity Expansion", "actors": ["banks", "purpose_group"], "source": "LARA" },
            { "date": "Dec 2024", "event": "Purpose Foundation filed — Banks + Holland hold all positions",
              "era": "2024: Entity Expansion", "actors": ["banks", "holland", "purpose_foundation"], "source": "LARA" },
            { "date": "2024", "event": "Judge Yancey campaign pays $2,283 to Banks Strategy",
              "era": "2024: Entity Expansion", "actors": ["yancey", "banks_strategy", "banks"], "source": "TransparencyUSA" },
            { "date": "Jul 2025", "event": "PCA authorized by DPSCD Board",
              "era": "2025: Charter Authorization", "actors": ["pca", "gay_dagnogo"], "source": "DPSCD" },
            { "date": "May 2026", "event": "Holland discharged from MDOC — WITHOUT IMPROVEMENT",
              "era": "2026: Exposure", "actors": ["holland"], "source": "MDOC OTIS" },
            { "date": "Sep 2026", "event": "ICHAT pulled — 9 convictions confirmed",
              "era": "2026: Exposure", "actors": ["banks"], "source": "MI ICHAT" },
            { "date": "Nov 3, 2026", "event": "Judge Cylenthia Miller — election day",
              "era": "Upcoming", "actors": ["miller"], "upcoming": true, "source": "" },
        ],
        "source": "static_fallback",
    })
}

fn static_detroit_graph() -> serde_json::Value {
    serde_json::json!({
        "nodes": [
            { "id": "banks", "label": "Brian R. Banks", "tier": 1, "type": "actor",
              "detail": "9 convictions (6 felony, 3 misdemeanor)" },
            { "id": "holland", "label": "Joseph Holland Jr.", "tier": 1, "type": "actor",
              "detail": "Drug offender, MDOC #443789" },
            { "id": "miller", "label": "Judge C. Miller", "tier": 2, "type": "judge",
              "detail": "Board Chair, Anchor Rock Foundation" },
            { "id": "yancey", "label": "Judge T. Yancey", "tier": 2, "type": "judge",
              "detail": "Campaign paid $2,283 to Banks Strategy" },
            { "id": "pca", "label": "Purpose Charter Academy", "tier": 0, "type": "school",
              "detail": "K-8, DPSCD authorized" },
            { "id": "macdowell", "label": "MacDowell Prep", "tier": 0, "type": "school",
              "detail": "$4.9M revenue, 72.67% extracted" },
            { "id": "purpose_group", "label": "Purpose Group LLC", "tier": 0, "type": "entity",
              "detail": "CMO — takes 72.67% of revenue" },
        ],
        "links": [
            { "source": "banks", "target": "pca", "type": "controls", "label": "superintendent" },
            { "source": "banks", "target": "macdowell", "type": "controls", "label": "superintendent" },
            { "source": "banks", "target": "purpose_group", "type": "controls", "label": "sole member" },
            { "source": "banks", "target": "holland", "type": "associate", "label": "co-resident" },
            { "source": "macdowell", "target": "purpose_group", "type": "money", "label": "72.67%" },
            { "source": "miller", "target": "banks", "type": "judicial", "label": "Board Chair" },
            { "source": "yancey", "target": "banks", "type": "judicial", "label": "$2,283" },
        ],
        "source": "static_fallback",
    })
}
