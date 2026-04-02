// SPDX-License-Identifier: AGPL-3.0-or-later
//! Centralized constants for the petalTongue primal.
//!
//! Self-knowledge only -- no hardcoded knowledge of other primals.
//! Other primals are discovered at runtime via socket/mDNS/JSON-RPC.

use std::time::Duration;

use crate::capability_names::primal_names;

/// Read an env var, parse it as `T`, or return `default`.
fn env_or<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse::<T>().ok())
        .unwrap_or(default)
}

/// Read an env var as `Duration` in seconds, or return `default_secs`.
fn env_duration_secs(key: &str, default_secs: u64) -> Duration {
    Duration::from_secs(env_or(key, default_secs))
}

/// Read an env var as `Duration` in milliseconds, or return `default_ms`.
fn env_duration_ms(key: &str, default_ms: u64) -> Duration {
    Duration::from_millis(env_or(key, default_ms))
}

/// Display name for this primal
pub const PRIMAL_NAME: &str = "petalTongue";

/// Application name used for directory paths (XDG conventions)
pub const APP_DIR_NAME: &str = "petaltongue";

/// Default web port (overridable via config or `PETALTONGUE_WEB_PORT` env)
pub const DEFAULT_WEB_PORT: u16 = 3000;

/// Default headless API port (overridable via config or `PETALTONGUE_HEADLESS_PORT` env)
pub const DEFAULT_HEADLESS_PORT: u16 = 8080;

/// biomeOS socket name template (discovered, not hardcoded to a port)
/// Overridable via `BIOMEOS_SOCKET_NAME` env var for custom deployments
#[must_use]
pub fn biomeos_socket_name() -> String {
    std::env::var("BIOMEOS_SOCKET_NAME").unwrap_or_else(|_| "biomeos-neural-api".to_string())
}

/// Device management socket name (capability: device management)
/// Overridable via `BIOMEOS_DEVICE_MANAGEMENT_SOCKET` env var
#[must_use]
pub fn biomeos_device_management_socket_name() -> String {
    std::env::var("BIOMEOS_DEVICE_MANAGEMENT_SOCKET")
        .unwrap_or_else(|_| "biomeos-device-management".to_string())
}

/// UI socket name (capability: UI/visualization)
/// Overridable via `BIOMEOS_UI_SOCKET` env var
#[must_use]
pub fn biomeos_ui_socket_name() -> String {
    std::env::var("BIOMEOS_UI_SOCKET").unwrap_or_else(|_| "biomeos-ui".to_string())
}

/// Discovery service socket name (capability: discovery/registry)
/// Overridable via `DISCOVERY_SERVICE_SOCKET` env var
#[must_use]
pub fn discovery_service_socket_name() -> String {
    std::env::var("DISCOVERY_SERVICE_SOCKET").unwrap_or_else(|_| "discovery-service".to_string())
}

/// Legacy /tmp biomeOS socket name (fallback)
/// Overridable via `BIOMEOS_LEGACY_SOCKET` env var
#[must_use]
pub fn biomeos_legacy_socket_name() -> String {
    std::env::var("BIOMEOS_LEGACY_SOCKET").unwrap_or_else(|_| primal_names::BIOMEOS.to_string())
}

/// Default sandbox security endpoint port.
/// Overridable via `PETALTONGUE_SANDBOX_SECURITY_ENDPOINT` (full URL) env var.
pub const DEFAULT_SANDBOX_SECURITY_PORT: u16 = 9000;

/// Default sandbox discovery endpoint port.
/// Overridable via `PETALTONGUE_SANDBOX_DISCOVERY_ENDPOINT` (full URL) env var.
pub const DEFAULT_SANDBOX_DISCOVERY_PORT: u16 = 8080;

/// Default GPU compute / Toadstool port (overridable via `TOADSTOOL_PORT` env var).
pub const DEFAULT_TOADSTOOL_PORT: u16 = 9001;

/// Toadstool port (env-driven with fallback).
/// Reads `TOADSTOOL_PORT`; falls back to `DEFAULT_TOADSTOOL_PORT`.
#[must_use]
pub fn toadstool_port() -> u16 {
    env_or("TOADSTOOL_PORT", DEFAULT_TOADSTOOL_PORT)
}

/// Loopback host for local-only connections (used when env/discovery not available).
pub const DEFAULT_LOOPBACK_HOST: &str = "127.0.0.1";

/// Default bind host for servers (listen on all interfaces).
/// Use when binding web/headless servers for external access.
pub const DEFAULT_BIND_HOST: &str = "0.0.0.0";

/// Default GPU compute endpoint URL (fallback when discovery fails).
///
/// Production uses capability discovery; override via `PETALTONGUE_GPU_COMPUTE_ENDPOINT`,
/// `GPU_RENDERING_ENDPOINT`, `COMPUTE_PROVIDER_ENDPOINT`, or `GPU_COMPUTE_ENDPOINT`.
pub const DEFAULT_GPU_COMPUTE_ENDPOINT: &str = "tarpc://localhost:9001";

/// Default ports for HTTP discovery when `PETALTONGUE_DISCOVERY_PORTS` / `DISCOVERY_PORTS` not set.
pub const DEFAULT_DISCOVERY_PORTS: &[u16] = &[8080, 8081, 3000, 9000];

/// Default window width in pixels (overridable via `PETALTONGUE_WINDOW_WIDTH` env var).
pub const DEFAULT_WINDOW_WIDTH: u32 = 1920;

/// Default window height in pixels (overridable via `PETALTONGUE_WINDOW_HEIGHT` env var).
pub const DEFAULT_WINDOW_HEIGHT: u32 = 1080;

/// Default terminal columns (for CLI/text UI).
pub const DEFAULT_TERMINAL_COLS: u16 = 80;

/// Default terminal rows (for CLI/text UI).
pub const DEFAULT_TERMINAL_ROWS: u16 = 24;

/// Max FPS for rendering (overridable via config)
pub const DEFAULT_MAX_FPS: u32 = 60;

/// Default bind address for servers (loopback-only for security).
///
/// Overridable via `--bind` CLI, config, or `PETALTONGUE_BIND_ADDR` env var.
/// Port comes from `PETALTONGUE_WEB_PORT` / `PETALTONGUE_HEADLESS_PORT`.
pub fn default_bind_addr() -> &'static str {
    static BIND: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    BIND.get_or_init(|| {
        std::env::var("PETALTONGUE_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1".to_string())
    })
}

/// Legacy /tmp socket fallback path prefix.
/// Used when `XDG_RUNTIME_DIR` is unavailable; configurable via explicit socket env vars.
pub const LEGACY_TMP_PREFIX: &str = "/tmp";

/// Build a default web bind address
///
/// Port is overridable via `PETALTONGUE_WEB_PORT` env var.
#[must_use]
pub fn default_web_bind() -> String {
    let port = env_or("PETALTONGUE_WEB_PORT", DEFAULT_WEB_PORT);
    let addr = default_bind_addr();
    format!("{addr}:{port}")
}

/// Build a default headless bind address
///
/// Port is overridable via `PETALTONGUE_HEADLESS_PORT` env var.
#[must_use]
pub fn default_headless_bind() -> String {
    let port = env_or("PETALTONGUE_HEADLESS_PORT", DEFAULT_HEADLESS_PORT);
    let addr = default_bind_addr();
    format!("{addr}:{port}")
}

/// Build a legacy biomeOS socket path (/tmp/biomeos-neural-api.sock)
/// Uses `BIOMEOS_SOCKET_NAME` env var if set
#[must_use]
pub fn biomeos_legacy_socket() -> std::path::PathBuf {
    std::path::PathBuf::from(LEGACY_TMP_PREFIX).join(format!("{}.sock", biomeos_socket_name()))
}

/// Default GPU compute endpoint (env-driven with fallback).
/// Priority: `PETALTONGUE_GPU_COMPUTE_ENDPOINT` > `GPU_RENDERING_ENDPOINT` >
/// `COMPUTE_PROVIDER_ENDPOINT` > `GPU_COMPUTE_ENDPOINT` > constant.
#[must_use]
pub fn default_gpu_compute_endpoint() -> String {
    std::env::var("PETALTONGUE_GPU_COMPUTE_ENDPOINT")
        .or_else(|_| std::env::var("GPU_RENDERING_ENDPOINT"))
        .or_else(|_| std::env::var("COMPUTE_PROVIDER_ENDPOINT"))
        .or_else(|_| std::env::var("GPU_COMPUTE_ENDPOINT"))
        .unwrap_or_else(|_| DEFAULT_GPU_COMPUTE_ENDPOINT.to_string())
}

/// GPU compute endpoint (env-driven with fallback).
/// Alias for `default_gpu_compute_endpoint()`; uses `PETALTONGUE_GPU_COMPUTE_ENDPOINT` or constant.
#[must_use]
pub fn gpu_compute_endpoint() -> String {
    default_gpu_compute_endpoint()
}

/// Discovery ports for HTTP probing (env-driven with fallback).
/// Reads `PETALTONGUE_DISCOVERY_PORTS` or `DISCOVERY_PORTS` (comma-separated).
#[must_use]
pub fn default_discovery_ports() -> Vec<u16> {
    std::env::var("PETALTONGUE_DISCOVERY_PORTS")
        .or_else(|_| std::env::var("DISCOVERY_PORTS"))
        .map_or_else(
            |_| DEFAULT_DISCOVERY_PORTS.to_vec(),
            |s| {
                s.split(',')
                    .filter_map(|p| p.trim().parse::<u16>().ok())
                    .collect()
            },
        )
}

/// Default window size (width, height). Env: `PETALTONGUE_WINDOW_WIDTH`, `PETALTONGUE_WINDOW_HEIGHT`.
#[must_use]
pub fn default_window_size() -> (u32, u32) {
    (
        env_or("PETALTONGUE_WINDOW_WIDTH", DEFAULT_WINDOW_WIDTH),
        env_or("PETALTONGUE_WINDOW_HEIGHT", DEFAULT_WINDOW_HEIGHT),
    )
}

/// Default biomeOS connection target for display (e.g. "localhost:3000").
/// Parses `BIOMEOS_URL` or uses `PETALTONGUE_LIVE_TARGET`; fallback: `localhost:{DEFAULT_WEB_PORT}`.
#[must_use]
pub fn default_biomeos_connection_target() -> String {
    if let Ok(t) = std::env::var("PETALTONGUE_LIVE_TARGET") {
        return t;
    }
    if let Ok(url) = std::env::var("BIOMEOS_URL")
        && let Some(rest) = url
            .strip_prefix("http://")
            .or_else(|| url.strip_prefix("https://"))
    {
        return rest.split('/').next().unwrap_or(rest).to_string();
    }
    format!("localhost:{DEFAULT_WEB_PORT}")
}

/// Default web server URL (e.g. "<http://localhost:3000>").
/// Uses `PETALTONGUE_WEB_URL` or `BIOMEOS_URL`; fallback: `http://{default_biomeos_connection_target}`.
#[must_use]
pub fn default_web_url() -> String {
    std::env::var("PETALTONGUE_WEB_URL")
        .or_else(|_| std::env::var("BIOMEOS_URL"))
        .unwrap_or_else(|_| format!("http://{}", default_biomeos_connection_target()))
}

/// Default entropy stream endpoint (e.g. "<http://localhost:3000/api/v1/entropy/stream>").
/// Uses `PETALTONGUE_ENTROPY_ENDPOINT`; fallback: host from env + default port.
#[must_use]
pub fn default_entropy_stream_endpoint() -> String {
    std::env::var("PETALTONGUE_ENTROPY_ENDPOINT").unwrap_or_else(|_| {
        format!(
            "http://{}/api/v1/entropy/stream",
            default_biomeos_connection_target()
        )
    })
}

/// Default headless API URL for display (e.g. "<http://localhost:8080>").
/// Uses `PETALTONGUE_WEB_URL` or `PETALTONGUE_HEADLESS_URL`; fallback from port.
#[must_use]
pub fn default_headless_url() -> String {
    std::env::var("PETALTONGUE_WEB_URL")
        .or_else(|_| std::env::var("PETALTONGUE_HEADLESS_URL"))
        .unwrap_or_else(|_| {
            let port = env_or("PETALTONGUE_HEADLESS_PORT", DEFAULT_HEADLESS_PORT);
            let target = default_biomeos_connection_target();
            let host = target.split(':').next().unwrap_or("localhost");
            format!("http://{host}:{port}")
        })
}

/// Default sandbox security URL (e.g. "<http://localhost:9000>").
/// Uses `PETALTONGUE_SANDBOX_SECURITY_ENDPOINT` or `PETALTONGUE_HEADLESS_ENDPOINT`; fallback from port.
#[must_use]
pub fn default_sandbox_security_url() -> String {
    std::env::var("PETALTONGUE_SANDBOX_SECURITY_ENDPOINT")
        .or_else(|_| std::env::var("PETALTONGUE_HEADLESS_ENDPOINT"))
        .unwrap_or_else(|_| {
            let port = env_or(
                "PETALTONGUE_SANDBOX_SECURITY_PORT",
                DEFAULT_SANDBOX_SECURITY_PORT,
            );
            let target = default_biomeos_connection_target();
            let host = target.split(':').next().unwrap_or("localhost");
            format!("http://{host}:{port}")
        })
}

/// Build a WebSocket URL with `path` suffix, respecting env overrides.
///
/// If `PETALTONGUE_WS_ENDPOINT` is set, returns it verbatim (assumed complete).
/// Otherwise builds from `BIOMEOS_WS_PORT` (or default) + loopback + path.
fn ws_url(path: &str) -> String {
    if let Ok(url) = std::env::var("PETALTONGUE_WS_ENDPOINT") {
        return url;
    }
    let port = env_or("BIOMEOS_WS_PORT", DEFAULT_HEADLESS_PORT);
    format!("ws://{DEFAULT_LOOPBACK_HOST}:{port}/{path}")
}

/// Default WebSocket URL for biomeOS topology updates.
/// Priority: `PETALTONGUE_WS_ENDPOINT` > `BIOMEOS_WS_PORT` + loopback > constant fallback.
#[must_use]
pub fn default_biomeos_ws_topology_url() -> String {
    ws_url("topology")
}

/// Default WebSocket URL for biomeOS event streaming.
/// Priority: `PETALTONGUE_WS_ENDPOINT` > `BIOMEOS_WS_PORT` + loopback > constant fallback.
#[must_use]
pub fn default_biomeos_ws_events_url() -> String {
    ws_url("events")
}

/// Health check thresholds
pub mod thresholds {
    /// CPU usage percentage above which to emit a warning
    pub const CPU_WARNING: f64 = 80.0;
    /// Memory usage percentage above which to emit a warning
    pub const MEMORY_WARNING: f64 = 50.0;
}

/// Tufte visualization tolerances (absorbed from ludoSpring V14).
///
/// These constants define maximum acceptable deviations for golden pixel
/// testing and data-ink ratio validation.
pub mod tufte_tolerances {
    /// Maximum fraction of non-data-ink pixels tolerated in a visualization.
    /// A ratio of 0.01 means ≤1% chartjunk is acceptable.
    pub const UI_DATA_INK_TOL: f64 = 0.01;

    /// Maximum fraction of viewport area that may remain uncovered by data.
    /// A ratio of 0.05 means ≥95% coverage is required.
    pub const UI_COVERAGE_TOL: f64 = 0.05;

    /// Maximum tolerable distance error in raycaster depth calculations.
    pub const RAYCASTER_DISTANCE_TOL: f64 = 0.001;

    /// Minimum coherence threshold for noise-based procedural generation.
    pub const NOISE_COHERENCE_TOL: f64 = 0.01;
}

/// Discovery HTTP client timeouts
pub mod discovery_timeouts {
    use std::time::Duration;

    /// HTTP total request timeout
    pub const HTTP_TIMEOUT: Duration = Duration::from_secs(30);
    /// HTTP connect timeout
    pub const HTTP_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
    /// HTTP connection pool idle timeout
    pub const HTTP_POOL_IDLE_TIMEOUT: Duration = Duration::from_secs(90);
    /// TCP keepalive interval
    pub const HTTP_TCP_KEEPALIVE: Duration = Duration::from_secs(60);
    /// HTTP/2 keepalive ping interval
    pub const HTTP2_KEEPALIVE_INTERVAL: Duration = Duration::from_secs(30);
    /// HTTP/2 keepalive ping timeout
    pub const HTTP2_KEEPALIVE_TIMEOUT: Duration = Duration::from_secs(10);

    /// Discovery cache: primals TTL
    pub const CACHE_PRIMALS_TTL: Duration = Duration::from_secs(30);
    /// Discovery cache: topology TTL
    pub const CACHE_TOPOLOGY_TTL: Duration = Duration::from_secs(60);
    /// Discovery cache: health TTL
    pub const CACHE_HEALTH_TTL: Duration = Duration::from_secs(10);

    /// Songbird UDS connect timeout (aggressive for non-blocking discovery)
    pub const SONGBIRD_CONNECT_TIMEOUT: Duration = Duration::from_millis(200);
    /// Songbird UDS write timeout
    pub const SONGBIRD_WRITE_TIMEOUT: Duration = Duration::from_millis(100);
    /// Songbird UDS read timeout
    pub const SONGBIRD_READ_TIMEOUT: Duration = Duration::from_millis(500);
}

/// Default client RPC timeout (overridable via `PETALTONGUE_RPC_TIMEOUT_SECS`)
pub const DEFAULT_RPC_TIMEOUT_SECS: u64 = 5;

/// Default heartbeat interval (overridable via `PETALTONGUE_HEARTBEAT_INTERVAL_SECS`)
pub const DEFAULT_HEARTBEAT_INTERVAL_SECS: u64 = 30;

/// Default biomeOS refresh interval (overridable via `PETALTONGUE_REFRESH_INTERVAL_SECS`)
pub const DEFAULT_REFRESH_INTERVAL_SECS: u64 = 2;

/// Default telemetry buffer size (overridable via `PETALTONGUE_TELEMETRY_BUFFER`)
pub const DEFAULT_TELEMETRY_BUFFER: usize = 10_000;

/// Default discovery timeout (overridable via `PETALTONGUE_DISCOVERY_TIMEOUT_SECS`)
pub const DEFAULT_DISCOVERY_TIMEOUT_SECS: u64 = 5;

/// Default retry initial delay in ms (overridable via `PETALTONGUE_RETRY_INITIAL_MS`)
pub const DEFAULT_RETRY_INITIAL_MS: u64 = 100;

/// Default retry max delay in secs (overridable via `PETALTONGUE_RETRY_MAX_SECS`)
pub const DEFAULT_RETRY_MAX_SECS: u64 = 10;

/// Default TUI tick rate in ms (overridable via `PETALTONGUE_TUI_TICK_MS`)
pub const DEFAULT_TUI_TICK_MS: u64 = 100;

/// Default RPC timeout. Env: `PETALTONGUE_RPC_TIMEOUT_SECS`.
#[must_use]
pub fn default_rpc_timeout() -> Duration {
    env_duration_secs("PETALTONGUE_RPC_TIMEOUT_SECS", DEFAULT_RPC_TIMEOUT_SECS)
}

/// Default heartbeat interval. Env: `PETALTONGUE_HEARTBEAT_INTERVAL_SECS`.
#[must_use]
pub fn default_heartbeat_interval() -> Duration {
    env_duration_secs(
        "PETALTONGUE_HEARTBEAT_INTERVAL_SECS",
        DEFAULT_HEARTBEAT_INTERVAL_SECS,
    )
}

/// Default biomeOS refresh interval. Env: `PETALTONGUE_REFRESH_INTERVAL_SECS`.
#[must_use]
pub fn default_refresh_interval() -> Duration {
    env_duration_secs(
        "PETALTONGUE_REFRESH_INTERVAL_SECS",
        DEFAULT_REFRESH_INTERVAL_SECS,
    )
}

/// Default telemetry buffer size. Env: `PETALTONGUE_TELEMETRY_BUFFER`.
#[must_use]
pub fn default_telemetry_buffer() -> usize {
    env_or("PETALTONGUE_TELEMETRY_BUFFER", DEFAULT_TELEMETRY_BUFFER)
}

/// Default discovery timeout. Env: `PETALTONGUE_DISCOVERY_TIMEOUT_SECS`.
#[must_use]
pub fn default_discovery_timeout() -> Duration {
    env_duration_secs(
        "PETALTONGUE_DISCOVERY_TIMEOUT_SECS",
        DEFAULT_DISCOVERY_TIMEOUT_SECS,
    )
}

/// Default retry initial delay. Env: `PETALTONGUE_RETRY_INITIAL_MS`.
#[must_use]
pub fn default_retry_initial_delay() -> Duration {
    env_duration_ms("PETALTONGUE_RETRY_INITIAL_MS", DEFAULT_RETRY_INITIAL_MS)
}

/// Default retry max delay. Env: `PETALTONGUE_RETRY_MAX_SECS`.
#[must_use]
pub fn default_retry_max_delay() -> Duration {
    env_duration_secs("PETALTONGUE_RETRY_MAX_SECS", DEFAULT_RETRY_MAX_SECS)
}

/// Default TUI tick rate. Env: `PETALTONGUE_TUI_TICK_MS`.
#[must_use]
pub fn default_tui_tick_rate() -> Duration {
    env_duration_ms("PETALTONGUE_TUI_TICK_MS", DEFAULT_TUI_TICK_MS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::env_test_helpers;

    #[test]
    fn primal_name() {
        assert_eq!(PRIMAL_NAME, "petalTongue");
    }

    #[test]
    fn app_dir_name() {
        assert_eq!(APP_DIR_NAME, "petaltongue");
    }

    #[test]
    fn default_web_port() {
        assert_eq!(DEFAULT_WEB_PORT, 3000);
    }

    #[test]
    fn default_headless_port() {
        assert_eq!(DEFAULT_HEADLESS_PORT, 8080);
    }

    #[test]
    fn test_default_bind_addr_is_loopback() {
        env_test_helpers::with_env_var_removed("PETALTONGUE_BIND_ADDR", || {
            assert_eq!(super::default_bind_addr(), "127.0.0.1");
        });
    }

    #[test]
    fn test_default_web_bind() {
        env_test_helpers::with_env_vars(
            &[
                ("PETALTONGUE_WEB_PORT", None),
                ("PETALTONGUE_HEADLESS_PORT", None),
            ],
            || {
                let bind = super::default_web_bind();
                assert!(
                    bind.ends_with(":3000"),
                    "should use default web port: {bind}"
                );
            },
        );
    }

    #[test]
    fn test_default_headless_bind() {
        env_test_helpers::with_env_vars(
            &[
                ("PETALTONGUE_WEB_PORT", None),
                ("PETALTONGUE_HEADLESS_PORT", None),
            ],
            || {
                let bind = super::default_headless_bind();
                assert!(
                    bind.ends_with(":8080"),
                    "should use default headless port: {bind}"
                );
            },
        );
    }

    #[test]
    fn test_default_web_bind_env_override() {
        env_test_helpers::with_env_var("PETALTONGUE_WEB_PORT", "4000", || {
            let bind = super::default_web_bind();
            assert!(
                bind.ends_with(":4000"),
                "should use overridden port: {bind}"
            );
        });
    }

    #[test]
    fn test_default_headless_bind_env_override() {
        env_test_helpers::with_env_var("PETALTONGUE_HEADLESS_PORT", "9000", || {
            let bind = super::default_headless_bind();
            assert!(
                bind.ends_with(":9000"),
                "should use overridden port: {bind}"
            );
        });
    }

    #[test]
    fn test_biomeos_socket_name_default() {
        env_test_helpers::with_env_var_removed("BIOMEOS_SOCKET_NAME", || {
            assert_eq!(super::biomeos_socket_name(), "biomeos-neural-api");
        });
    }

    #[test]
    fn test_biomeos_socket_name_override() {
        env_test_helpers::with_env_var("BIOMEOS_SOCKET_NAME", "custom-socket", || {
            assert_eq!(super::biomeos_socket_name(), "custom-socket");
        });
    }

    #[test]
    fn test_biomeos_device_management_socket_override() {
        env_test_helpers::with_env_var(
            "BIOMEOS_DEVICE_MANAGEMENT_SOCKET",
            "custom-device-mgmt",
            || {
                assert_eq!(
                    super::biomeos_device_management_socket_name(),
                    "custom-device-mgmt"
                );
            },
        );
    }

    #[test]
    fn test_biomeos_ui_socket_override() {
        env_test_helpers::with_env_var("BIOMEOS_UI_SOCKET", "custom-ui", || {
            assert_eq!(super::biomeos_ui_socket_name(), "custom-ui");
        });
    }

    #[test]
    fn test_discovery_service_socket_override() {
        env_test_helpers::with_env_var("DISCOVERY_SERVICE_SOCKET", "custom-discovery", || {
            assert_eq!(super::discovery_service_socket_name(), "custom-discovery");
        });
    }

    #[test]
    fn test_biomeos_legacy_socket_override() {
        env_test_helpers::with_env_var("BIOMEOS_LEGACY_SOCKET", "custom-legacy", || {
            assert_eq!(super::biomeos_legacy_socket_name(), "custom-legacy");
        });
    }

    #[test]
    fn test_biomeos_legacy_socket_path() {
        env_test_helpers::with_env_var("BIOMEOS_SOCKET_NAME", "test-sock", || {
            let path = super::biomeos_legacy_socket();
            assert!(path.to_string_lossy().ends_with("test-sock.sock"));
            assert!(path.to_string_lossy().contains("/tmp"));
        });
    }

    #[test]
    fn thresholds_constants() {
        assert!((thresholds::CPU_WARNING - 80.0).abs() < f64::EPSILON);
        assert!((thresholds::MEMORY_WARNING - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn loopback_host_is_ipv4() {
        assert_eq!(DEFAULT_LOOPBACK_HOST, "127.0.0.1");
    }

    #[test]
    fn test_default_biomeos_ws_topology_url() {
        env_test_helpers::with_env_vars(
            &[("PETALTONGUE_WS_ENDPOINT", None), ("BIOMEOS_WS_PORT", None)],
            || {
                let url = default_biomeos_ws_topology_url();
                assert!(url.starts_with("ws://"), "should be ws:// scheme");
                assert!(url.ends_with("/topology"), "should end with /topology");
                assert!(url.contains(DEFAULT_LOOPBACK_HOST), "should use loopback");
            },
        );
    }

    #[test]
    fn test_default_biomeos_ws_events_url() {
        env_test_helpers::with_env_vars(
            &[("PETALTONGUE_WS_ENDPOINT", None), ("BIOMEOS_WS_PORT", None)],
            || {
                let url = default_biomeos_ws_events_url();
                assert!(url.starts_with("ws://"), "should be ws:// scheme");
                assert!(url.ends_with("/events"), "should end with /events");
                assert!(url.contains(DEFAULT_LOOPBACK_HOST), "should use loopback");
            },
        );
    }

    #[test]
    fn test_ws_topology_env_override() {
        env_test_helpers::with_env_var("PETALTONGUE_WS_ENDPOINT", "ws://custom:9999/topo", || {
            assert_eq!(default_biomeos_ws_topology_url(), "ws://custom:9999/topo");
        });
    }

    #[test]
    fn test_ws_events_env_override() {
        env_test_helpers::with_env_var("PETALTONGUE_WS_ENDPOINT", "ws://custom:9999/ev", || {
            assert_eq!(default_biomeos_ws_events_url(), "ws://custom:9999/ev");
        });
    }

    #[test]
    fn test_ws_port_env_override() {
        env_test_helpers::with_env_vars(
            &[
                ("PETALTONGUE_WS_ENDPOINT", None),
                ("BIOMEOS_WS_PORT", Some("7777")),
            ],
            || {
                let topo = default_biomeos_ws_topology_url();
                assert!(topo.contains(":7777/"), "topology URL should use port 7777");

                let events = default_biomeos_ws_events_url();
                assert!(events.contains(":7777/"), "events URL should use port 7777");
            },
        );
    }

    #[test]
    fn test_default_rpc_timeout() {
        env_test_helpers::with_env_var_removed("PETALTONGUE_RPC_TIMEOUT_SECS", || {
            assert_eq!(
                default_rpc_timeout(),
                Duration::from_secs(DEFAULT_RPC_TIMEOUT_SECS)
            );
        });
    }

    #[test]
    fn test_default_rpc_timeout_env_override() {
        env_test_helpers::with_env_var("PETALTONGUE_RPC_TIMEOUT_SECS", "15", || {
            assert_eq!(default_rpc_timeout(), Duration::from_secs(15));
        });
    }

    #[test]
    fn test_default_heartbeat_interval() {
        env_test_helpers::with_env_var_removed("PETALTONGUE_HEARTBEAT_INTERVAL_SECS", || {
            assert_eq!(
                default_heartbeat_interval(),
                Duration::from_secs(DEFAULT_HEARTBEAT_INTERVAL_SECS)
            );
        });
    }

    #[test]
    fn test_default_heartbeat_interval_env_override() {
        env_test_helpers::with_env_var("PETALTONGUE_HEARTBEAT_INTERVAL_SECS", "60", || {
            assert_eq!(default_heartbeat_interval(), Duration::from_secs(60));
        });
    }

    #[test]
    fn test_default_refresh_interval() {
        env_test_helpers::with_env_var_removed("PETALTONGUE_REFRESH_INTERVAL_SECS", || {
            assert_eq!(
                default_refresh_interval(),
                Duration::from_secs(DEFAULT_REFRESH_INTERVAL_SECS)
            );
        });
    }

    #[test]
    fn test_default_refresh_interval_env_override() {
        env_test_helpers::with_env_var("PETALTONGUE_REFRESH_INTERVAL_SECS", "5", || {
            assert_eq!(default_refresh_interval(), Duration::from_secs(5));
        });
    }

    #[test]
    fn test_default_telemetry_buffer() {
        env_test_helpers::with_env_var_removed("PETALTONGUE_TELEMETRY_BUFFER", || {
            assert_eq!(default_telemetry_buffer(), DEFAULT_TELEMETRY_BUFFER);
        });
    }

    #[test]
    fn test_default_telemetry_buffer_env_override() {
        env_test_helpers::with_env_var("PETALTONGUE_TELEMETRY_BUFFER", "5000", || {
            assert_eq!(default_telemetry_buffer(), 5000);
        });
    }

    #[test]
    fn test_default_discovery_timeout() {
        env_test_helpers::with_env_var_removed("PETALTONGUE_DISCOVERY_TIMEOUT_SECS", || {
            assert_eq!(
                default_discovery_timeout(),
                Duration::from_secs(DEFAULT_DISCOVERY_TIMEOUT_SECS)
            );
        });
    }

    #[test]
    fn test_default_discovery_timeout_env_override() {
        env_test_helpers::with_env_var("PETALTONGUE_DISCOVERY_TIMEOUT_SECS", "10", || {
            assert_eq!(default_discovery_timeout(), Duration::from_secs(10));
        });
    }

    #[test]
    fn test_default_retry_initial_delay() {
        env_test_helpers::with_env_var_removed("PETALTONGUE_RETRY_INITIAL_MS", || {
            assert_eq!(
                default_retry_initial_delay(),
                Duration::from_millis(DEFAULT_RETRY_INITIAL_MS)
            );
        });
    }

    #[test]
    fn test_default_retry_initial_delay_env_override() {
        env_test_helpers::with_env_var("PETALTONGUE_RETRY_INITIAL_MS", "500", || {
            assert_eq!(default_retry_initial_delay(), Duration::from_millis(500));
        });
    }

    #[test]
    fn test_default_retry_max_delay() {
        env_test_helpers::with_env_var_removed("PETALTONGUE_RETRY_MAX_SECS", || {
            assert_eq!(
                default_retry_max_delay(),
                Duration::from_secs(DEFAULT_RETRY_MAX_SECS)
            );
        });
    }

    #[test]
    fn test_default_retry_max_delay_env_override() {
        env_test_helpers::with_env_var("PETALTONGUE_RETRY_MAX_SECS", "30", || {
            assert_eq!(default_retry_max_delay(), Duration::from_secs(30));
        });
    }

    #[test]
    fn test_default_tui_tick_rate() {
        env_test_helpers::with_env_var_removed("PETALTONGUE_TUI_TICK_MS", || {
            assert_eq!(
                default_tui_tick_rate(),
                Duration::from_millis(DEFAULT_TUI_TICK_MS)
            );
        });
    }

    #[test]
    fn test_default_tui_tick_rate_env_override() {
        env_test_helpers::with_env_var("PETALTONGUE_TUI_TICK_MS", "50", || {
            assert_eq!(default_tui_tick_rate(), Duration::from_millis(50));
        });
    }
}
