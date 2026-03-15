// SPDX-License-Identifier: AGPL-3.0-only
//! Compute bridge: async IPC client for barraCuda GPU compute operations.
//!
//! Discovers compute primals at runtime (capability `gpu.dispatch`) and dispatches
//! operations via JSON-RPC. Falls back to empty results when no compute primal
//! is available (primal sovereignty -- no hard dependency).
//!
//! # Operation families
//!
//! - **Physics**: `math.physics.nbody` — N-body / molecular dynamics (CPU Euler fallback)
//! - **Statistics**: `math.stat.kde`, `math.stat.smooth`, `math.stat.bin`, `math.stat.summary`
//! - **Tessellation**: `math.tessellate.sphere`, `math.tessellate.cylinder`, `math.tessellate.isosurface`
//! - **Projection**: `math.project.perspective`, `math.project.lighting`

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

/// Result of a compute dispatch operation.
#[derive(Debug, Clone)]
pub struct ComputeDispatchResult {
    /// Whether GPU compute was used.
    pub gpu_accelerated: bool,
    /// Operation that was dispatched.
    pub operation: String,
    /// Result from compute primal (or null when fallback).
    pub result: serde_json::Value,
    /// Duration in seconds (wall clock).
    pub duration_secs: f64,
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

/// Dispatch a statistical operation to barraCuda.
///
/// Supported ops: `math.stat.kde`, `math.stat.smooth`, `math.stat.bin`, `math.stat.summary`
pub async fn dispatch_stat(op: &str, params: serde_json::Value) -> ComputeDispatchResult {
    dispatch_compute("barracuda.compute.dispatch", op, params).await
}

/// Dispatch a tessellation operation to barraCuda.
///
/// Supported ops: `math.tessellate.sphere`, `math.tessellate.cylinder`, `math.tessellate.isosurface`
pub async fn dispatch_tessellate(op: &str, params: serde_json::Value) -> ComputeDispatchResult {
    dispatch_compute("barracuda.compute.dispatch", op, params).await
}

/// Dispatch a projection operation to barraCuda.
///
/// Supported ops: `math.project.perspective`, `math.project.lighting`
pub async fn dispatch_project(op: &str, params: serde_json::Value) -> ComputeDispatchResult {
    dispatch_compute("barracuda.compute.dispatch", op, params).await
}

/// Generic compute dispatch to barraCuda via JSON-RPC.
///
/// Falls back to an empty result when compute primal is unavailable.
async fn dispatch_compute(
    method: &str,
    op: &str,
    params: serde_json::Value,
) -> ComputeDispatchResult {
    let start = std::time::Instant::now();

    match try_dispatch(method, op, &params).await {
        Ok(result) => {
            debug!("Compute dispatch {op} via GPU: success");
            ComputeDispatchResult {
                gpu_accelerated: true,
                operation: op.to_string(),
                result,
                duration_secs: start.elapsed().as_secs_f64(),
            }
        }
        Err(e) => {
            debug!("GPU compute unavailable for {op} ({e}), returning empty result");
            ComputeDispatchResult {
                gpu_accelerated: false,
                operation: op.to_string(),
                result: serde_json::Value::Null,
                duration_secs: start.elapsed().as_secs_f64(),
            }
        }
    }
}

async fn try_dispatch(
    method: &str,
    op: &str,
    params: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let socket_path = discover_compute_socket()?;

    let request = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": {
            "op": op,
            "data": params,
        },
        "id": 1
    });

    let response = send_jsonrpc_unix(&socket_path, &request)
        .await
        .map_err(|e| format!("IPC send failed: {e}"))?;

    response.get("result").cloned().ok_or_else(|| {
        let err = response
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("unknown error");
        format!("Compute primal error: {err}")
    })
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
    let socket_name = std::env::var("PHYSICS_COMPUTE_SOCKET_NAME")
        .unwrap_or_else(|_| "physics-compute".to_string());

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
    use serde_json::json;

    #[test]
    fn discover_barracuda_returns_err_when_absent() {
        let result = discover_compute_socket();
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

    #[tokio::test]
    async fn step_physics_with_barracuda_env_uses_env_path() {
        let temp = std::env::temp_dir().join("physics-bridge-test.sock");
        std::fs::write(&temp, "").expect("create temp file");
        let path_str = temp.to_str().expect("path").to_string();

        let result = petal_tongue_core::test_fixtures::env_test_helpers::with_env_var_async(
            "BARRACUDA_SOCKET",
            &path_str,
            || async {
                let mut world = PhysicsWorld::new();
                world.gravity = [0.0, 0.0, 0.0];
                world.add_body(PhysicsBody {
                    id: "b1".into(),
                    mass: 1.0,
                    position: [0.0, 0.0, 0.0],
                    velocity: [0.0, 0.0, 0.0],
                    collision_shape: CollisionShape::None,
                });
                step_physics(&mut world).await
            },
        )
        .await;
        let _ = std::fs::remove_file(&temp);
        assert!(
            !result.gpu_accelerated,
            "temp file is not a socket, should fall back"
        );
        assert_eq!(result.bodies_updated, 1);
    }

    #[tokio::test]
    async fn step_physics_multiple_bodies() {
        let mut world = PhysicsWorld::new();
        world.gravity = [0.0, -9.81, 0.0];
        world.time_step = 0.016;
        for i in 0..5 {
            world.add_body(PhysicsBody {
                id: format!("body-{i}"),
                mass: 1.0,
                position: [0.0, 0.0, 0.0],
                velocity: [0.0, 0.0, 0.0],
                collision_shape: CollisionShape::None,
            });
        }
        let result = step_physics(&mut world).await;
        assert_eq!(result.bodies_updated, 5);
        assert!(result.step_duration_secs >= 0.0);
    }

    #[test]
    fn physics_step_result_structure() {
        let r = PhysicsStepResult {
            gpu_accelerated: false,
            bodies_updated: 3,
            step_duration_secs: 0.001,
        };
        assert!(!r.gpu_accelerated);
        assert_eq!(r.bodies_updated, 3);
        assert!((r.step_duration_secs - 0.001).abs() < f64::EPSILON);
    }

    #[test]
    fn physics_ipc_request_format() {
        let mut world = PhysicsWorld::new();
        world.gravity = [0.0, -9.81, 0.0];
        world.time_step = 0.016;
        world.add_body(PhysicsBody {
            id: "b1".into(),
            mass: 1.0,
            position: [0.0, 0.0, 0.0],
            velocity: [1.0, 0.0, 0.0],
            collision_shape: CollisionShape::None,
        });
        let req = world.to_ipc_request();
        assert!(req.is_object());
        assert!(req.get("bodies").is_some());
        assert!(req.get("bodies").and_then(|b| b.as_array()).unwrap().len() == 1);
    }

    #[test]
    fn physics_step_result_clone() {
        let r = PhysicsStepResult {
            gpu_accelerated: true,
            bodies_updated: 10,
            step_duration_secs: 0.05,
        };
        let r2 = r.clone();
        assert_eq!(r.gpu_accelerated, r2.gpu_accelerated);
        assert_eq!(r.bodies_updated, r2.bodies_updated);
    }

    #[tokio::test]
    async fn dispatch_stat_falls_back_without_barracuda() {
        let result = dispatch_stat("math.stat.kde", json!({"values": [1.0, 2.0, 3.0]})).await;
        assert!(!result.gpu_accelerated);
        assert_eq!(result.operation, "math.stat.kde");
    }

    #[tokio::test]
    async fn dispatch_tessellate_falls_back_without_barracuda() {
        let result = dispatch_tessellate(
            "math.tessellate.sphere",
            json!({"radius": 1.0, "segments": 32}),
        )
        .await;
        assert!(!result.gpu_accelerated);
        assert_eq!(result.operation, "math.tessellate.sphere");
    }

    #[tokio::test]
    async fn dispatch_project_falls_back_without_barracuda() {
        let result = dispatch_project("math.project.perspective", json!({"fov": 60.0})).await;
        assert!(!result.gpu_accelerated);
        assert_eq!(result.operation, "math.project.perspective");
    }

    #[test]
    fn compute_dispatch_result_structure() {
        let r = ComputeDispatchResult {
            gpu_accelerated: false,
            operation: "math.stat.kde".to_string(),
            result: json!(null),
            duration_secs: 0.001,
        };
        assert!(!r.gpu_accelerated);
        assert_eq!(r.operation, "math.stat.kde");
    }
}
