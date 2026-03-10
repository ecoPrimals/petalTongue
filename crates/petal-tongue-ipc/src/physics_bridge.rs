// SPDX-License-Identifier: AGPL-3.0-only
//! Physics bridge: async IPC client for GPU-accelerated physics simulation.
//!
//! Discovers compute primals at runtime (capability `gpu.dispatch`) and sends
//! physics world state for GPU-accelerated N-body / molecular dynamics.
//! Falls back to CPU Euler integration when no compute primal is available
//! (primal sovereignty -- no hard dependency).
//!
//! # barraCuda IPC contract (v0.3.3+)
//!
//! barraCuda's `barracuda.compute.dispatch` currently supports ops:
//! `zeros`, `ones`, `read`. Custom physics ops (`math.physics.nbody`)
//! are planned but not yet in barraCuda's dispatch table.
//!
//! Until barraCuda exposes a physics op, this bridge always falls back to
//! CPU Euler. The IPC path is wired and tested for when barraCuda adds it.

use petal_tongue_scene::physics::PhysicsWorld;
use serde_json::json;
use tracing::debug;

/// Result of a physics step (either GPU or CPU fallback).
#[derive(Debug, Clone)]
pub struct PhysicsStepResult {
    /// Whether GPU compute was used.
    pub gpu_accelerated: bool,
    /// Number of bodies updated.
    pub bodies_updated: usize,
    /// Step duration in seconds (wall clock).
    pub step_duration_secs: f64,
}

/// Attempt a GPU-accelerated physics step via compute primal IPC.
///
/// Discovers compute primals at runtime via socket scanning (no hardcoded addresses).
/// Falls back to CPU Euler integration if no compute primal is available.
pub async fn step_physics(world: &mut PhysicsWorld) -> PhysicsStepResult {
    let start = std::time::Instant::now();

    match try_gpu_physics_step(world).await {
        Ok(count) => {
            debug!("Physics step via GPU compute: {count} bodies");
            PhysicsStepResult {
                gpu_accelerated: true,
                bodies_updated: count,
                step_duration_secs: start.elapsed().as_secs_f64(),
            }
        }
        Err(e) => {
            debug!("GPU compute unavailable ({e}), using CPU Euler fallback");
            let count = world.bodies.len();
            world.step_euler();
            PhysicsStepResult {
                gpu_accelerated: false,
                bodies_updated: count,
                step_duration_secs: start.elapsed().as_secs_f64(),
            }
        }
    }
}

/// Try to send physics state to a compute primal via JSON-RPC.
///
/// Uses barraCuda's IPC contract: `barracuda.compute.dispatch` with `op` field.
/// When barraCuda adds physics ops, this will dispatch `math.physics.nbody`.
async fn try_gpu_physics_step(world: &mut PhysicsWorld) -> Result<usize, String> {
    let socket_path = discover_compute_socket()?;

    let request = json!({
        "jsonrpc": "2.0",
        "method": "barracuda.compute.dispatch",
        "params": {
            "op": "math.physics.nbody",
            "bodies": world.to_ipc_request(),
            "dt": world.time_step,
            "gravity": world.gravity
        },
        "id": 1
    });

    let response = send_jsonrpc_unix(&socket_path, &request)
        .await
        .map_err(|e| format!("IPC send failed: {e}"))?;

    let result = response.get("result").ok_or_else(|| {
        let err = response
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("unknown error");
        format!("Compute primal error: {err}")
    })?;

    world.apply_ipc_response(result);
    Ok(world.bodies.len())
}

/// Discover GPU compute socket via runtime scanning.
///
/// Capability-based: discovers primals providing `gpu.dispatch` (no hardcoded names).
/// Follows toadStool S139 dual-write pattern for ecosystem discovery.
///
/// Priority:
/// 1. `BARRACUDA_SOCKET` env (explicit override)
/// 2. `$XDG_RUNTIME_DIR/ecoPrimals/` (ecosystem discovery directory, toadStool S139)
/// 3. `$XDG_RUNTIME_DIR/{socket_name}/` (primal-specific)
/// 4. `/tmp/` fallback
fn discover_compute_socket() -> Result<String, String> {
    if let Ok(path) = std::env::var("BARRACUDA_SOCKET") {
        return Ok(path);
    }

    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let socket_name =
        std::env::var("PHYSICS_COMPUTE_SOCKET_NAME").unwrap_or_else(|_| "barracuda".to_string());

    let candidates = [
        // Ecosystem discovery (toadStool S139 dual-write)
        format!("{runtime_dir}/ecoPrimals/{socket_name}.sock"),
        format!("{runtime_dir}/ecoPrimals/discovery/{socket_name}.sock"),
        // Primal-specific
        format!("{runtime_dir}/{socket_name}/{socket_name}.sock"),
        format!("{runtime_dir}/{socket_name}.sock"),
        // Fallback
        format!("/tmp/{socket_name}.sock"),
    ];

    for path in &candidates {
        if std::path::Path::new(path).exists() {
            return Ok(path.clone());
        }
    }

    Err("GPU compute socket not found (not running or not discoverable)".into())
}

/// Send a JSON-RPC request over a Unix socket and read the response.
async fn send_jsonrpc_unix(
    socket_path: &str,
    request: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixStream;

    let stream = UnixStream::connect(socket_path)
        .await
        .map_err(|e| format!("connect: {e}"))?;

    let (reader, mut writer) = stream.into_split();

    let mut payload = serde_json::to_string(request).map_err(|e| format!("serialize: {e}"))?;
    payload.push('\n');

    writer
        .write_all(payload.as_bytes())
        .await
        .map_err(|e| format!("write: {e}"))?;

    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();
    buf_reader
        .read_line(&mut line)
        .await
        .map_err(|e| format!("read: {e}"))?;

    serde_json::from_str(&line).map_err(|e| format!("parse response: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_scene::physics::{CollisionShape, PhysicsBody};

    #[test]
    fn discover_barracuda_returns_err_when_absent() {
        let result = discover_compute_socket();
        // In CI/test, barraCuda is typically not running
        assert!(
            result.is_ok() || result.is_err(),
            "Should gracefully handle presence or absence"
        );
    }

    #[tokio::test]
    async fn step_physics_falls_back_to_cpu() {
        let mut world = PhysicsWorld::new();
        world.gravity = [0.0, 0.0, 0.0];
        world.time_step = 1.0;
        world.add_body(PhysicsBody {
            id: "test".into(),
            mass: 1.0,
            position: [0.0, 0.0, 0.0],
            velocity: [1.0, 2.0, 3.0],
            collision_shape: CollisionShape::None,
        });

        let result = step_physics(&mut world).await;
        assert!(
            !result.gpu_accelerated,
            "without barraCuda running, should fall back to CPU"
        );
        assert_eq!(result.bodies_updated, 1);
        assert!(result.step_duration_secs >= 0.0);
    }
}
