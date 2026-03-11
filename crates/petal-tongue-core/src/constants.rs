// SPDX-License-Identifier: AGPL-3.0-only
//! Centralized constants for the petalTongue primal.
//!
//! Self-knowledge only -- no hardcoded knowledge of other primals.
//! Other primals are discovered at runtime via socket/mDNS/JSON-RPC.

/// Display name for this primal
pub const PRIMAL_NAME: &str = "petalTongue";

/// Application name used for directory paths (XDG conventions)
pub const APP_DIR_NAME: &str = "petaltongue";

/// Default web port (overridable via config or `PETALTONGUE_WEB_PORT` env)
pub const DEFAULT_WEB_PORT: u16 = 3000;

/// Default headless API port (overridable via config or `PETALTONGUE_HEADLESS_PORT` env)
pub const DEFAULT_HEADLESS_PORT: u16 = 8080;

/// biomeOS socket name template (discovered, not hardcoded to a port)
/// Overridable via BIOMEOS_SOCKET_NAME env var for custom deployments
pub fn biomeos_socket_name() -> String {
    std::env::var("BIOMEOS_SOCKET_NAME").unwrap_or_else(|_| "biomeos-neural-api".to_string())
}

/// Device management socket name (capability: device management)
/// Overridable via BIOMEOS_DEVICE_MANAGEMENT_SOCKET env var
pub fn biomeos_device_management_socket_name() -> String {
    std::env::var("BIOMEOS_DEVICE_MANAGEMENT_SOCKET")
        .unwrap_or_else(|_| "biomeos-device-management".to_string())
}

/// UI socket name (capability: UI/visualization)
/// Overridable via BIOMEOS_UI_SOCKET env var
pub fn biomeos_ui_socket_name() -> String {
    std::env::var("BIOMEOS_UI_SOCKET").unwrap_or_else(|_| "biomeos-ui".to_string())
}

/// Discovery service socket name (capability: discovery/registry)
/// Overridable via DISCOVERY_SERVICE_SOCKET env var
pub fn discovery_service_socket_name() -> String {
    std::env::var("DISCOVERY_SERVICE_SOCKET").unwrap_or_else(|_| "discovery-service".to_string())
}

/// Legacy /tmp biomeOS socket name (fallback)
/// Overridable via BIOMEOS_LEGACY_SOCKET env var
pub fn biomeos_legacy_socket_name() -> String {
    std::env::var("BIOMEOS_LEGACY_SOCKET").unwrap_or_else(|_| "biomeos".to_string())
}

/// Default sandbox security endpoint port.
/// Overridable via `PETALTONGUE_SANDBOX_SECURITY_ENDPOINT` (full URL) env var.
pub const DEFAULT_SANDBOX_SECURITY_PORT: u16 = 9000;

/// Default sandbox discovery endpoint port.
/// Overridable via `PETALTONGUE_SANDBOX_DISCOVERY_ENDPOINT` (full URL) env var.
pub const DEFAULT_SANDBOX_DISCOVERY_PORT: u16 = 8080;

/// Default GPU compute / Toadstool port (overridable via `GPU_RENDERING_ENDPOINT`, `COMPUTE_PROVIDER_ENDPOINT`, or `GPU_COMPUTE_ENDPOINT` env vars).
pub const DEFAULT_TOADSTOOL_PORT: u16 = 9001;

/// Loopback host for local-only connections (used when env/discovery not available).
pub const DEFAULT_LOOPBACK_HOST: &str = "127.0.0.1";

/// Default GPU compute endpoint URL (fallback when discovery fails).
/// Production uses capability discovery; override via `GPU_RENDERING_ENDPOINT`, `COMPUTE_PROVIDER_ENDPOINT`, or `GPU_COMPUTE_ENDPOINT`.
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

/// Default bind address for servers (0.0.0.0 = all interfaces).
/// Overridable via `--bind` CLI or config; port comes from `PETALTONGUE_WEB_PORT` / `PETALTONGUE_HEADLESS_PORT`.
pub const DEFAULT_BIND_ADDR: &str = "0.0.0.0";

/// Legacy /tmp socket fallback path prefix.
/// Used when XDG_RUNTIME_DIR is unavailable; configurable via explicit socket env vars.
pub const LEGACY_TMP_PREFIX: &str = "/tmp";

/// Build a default web bind address
///
/// Port is overridable via `PETALTONGUE_WEB_PORT` env var.
#[must_use]
pub fn default_web_bind() -> String {
    let port = std::env::var("PETALTONGUE_WEB_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(DEFAULT_WEB_PORT);
    format!("{DEFAULT_BIND_ADDR}:{port}")
}

/// Build a default headless bind address
///
/// Port is overridable via `PETALTONGUE_HEADLESS_PORT` env var.
#[must_use]
pub fn default_headless_bind() -> String {
    let port = std::env::var("PETALTONGUE_HEADLESS_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(DEFAULT_HEADLESS_PORT);
    format!("{DEFAULT_BIND_ADDR}:{port}")
}

/// Build a legacy biomeOS socket path (/tmp/biomeos-neural-api.sock)
/// Uses BIOMEOS_SOCKET_NAME env var if set
#[must_use]
pub fn biomeos_legacy_socket() -> std::path::PathBuf {
    std::path::PathBuf::from(LEGACY_TMP_PREFIX).join(format!("{}.sock", biomeos_socket_name()))
}

/// Default GPU compute endpoint (env-driven with fallback).
/// Priority: `GPU_RENDERING_ENDPOINT` > `COMPUTE_PROVIDER_ENDPOINT` > `GPU_COMPUTE_ENDPOINT` > constant.
#[must_use]
pub fn default_gpu_compute_endpoint() -> String {
    std::env::var("GPU_RENDERING_ENDPOINT")
        .or_else(|_| std::env::var("COMPUTE_PROVIDER_ENDPOINT"))
        .or_else(|_| std::env::var("GPU_COMPUTE_ENDPOINT"))
        .unwrap_or_else(|_| DEFAULT_GPU_COMPUTE_ENDPOINT.to_string())
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
    let w = std::env::var("PETALTONGUE_WINDOW_WIDTH")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_WINDOW_WIDTH);
    let h = std::env::var("PETALTONGUE_WINDOW_HEIGHT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_WINDOW_HEIGHT);
    (w, h)
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

/// Default web server URL (e.g. "http://localhost:3000").
/// Uses `PETALTONGUE_WEB_URL` or `BIOMEOS_URL`; fallback: `http://{default_biomeos_connection_target}`.
#[must_use]
pub fn default_web_url() -> String {
    std::env::var("PETALTONGUE_WEB_URL")
        .or_else(|_| std::env::var("BIOMEOS_URL"))
        .unwrap_or_else(|_| format!("http://{}", default_biomeos_connection_target()))
}

/// Default entropy stream endpoint (e.g. "http://localhost:3000/api/v1/entropy/stream").
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

/// Default headless API URL for display (e.g. "http://localhost:8080").
/// Uses `PETALTONGUE_WEB_URL` or `PETALTONGUE_HEADLESS_URL`; fallback from port.
#[must_use]
pub fn default_headless_url() -> String {
    std::env::var("PETALTONGUE_WEB_URL")
        .or_else(|_| std::env::var("PETALTONGUE_HEADLESS_URL"))
        .unwrap_or_else(|_| {
            let port = std::env::var("PETALTONGUE_HEADLESS_PORT")
                .ok()
                .and_then(|s| s.parse::<u16>().ok())
                .unwrap_or(DEFAULT_HEADLESS_PORT);
            let target = default_biomeos_connection_target();
            let host = target.split(':').next().unwrap_or("localhost");
            format!("http://{host}:{port}")
        })
}

/// Default sandbox security URL (e.g. "http://localhost:9000").
/// Uses `PETALTONGUE_SANDBOX_SECURITY_ENDPOINT` or `PETALTONGUE_HEADLESS_ENDPOINT`; fallback from port.
#[must_use]
pub fn default_sandbox_security_url() -> String {
    std::env::var("PETALTONGUE_SANDBOX_SECURITY_ENDPOINT")
        .or_else(|_| std::env::var("PETALTONGUE_HEADLESS_ENDPOINT"))
        .unwrap_or_else(|_| {
            let port = std::env::var("PETALTONGUE_SANDBOX_SECURITY_PORT")
                .ok()
                .and_then(|s| s.parse::<u16>().ok())
                .unwrap_or(DEFAULT_SANDBOX_SECURITY_PORT);
            let target = default_biomeos_connection_target();
            let host = target.split(':').next().unwrap_or("localhost");
            format!("http://{host}:{port}")
        })
}

/// Default WebSocket URL for biomeOS topology updates.
/// Priority: `PETALTONGUE_WS_ENDPOINT` > `BIOMEOS_WS_PORT` + loopback > constant fallback.
#[must_use]
pub fn default_biomeos_ws_topology_url() -> String {
    if let Ok(url) = std::env::var("PETALTONGUE_WS_ENDPOINT") {
        return url;
    }
    let port = std::env::var("BIOMEOS_WS_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(DEFAULT_HEADLESS_PORT);
    format!("ws://{DEFAULT_LOOPBACK_HOST}:{port}/topology")
}

/// Default WebSocket URL for biomeOS event streaming.
/// Priority: `PETALTONGUE_WS_ENDPOINT` > `BIOMEOS_WS_PORT` + loopback > constant fallback.
#[must_use]
pub fn default_biomeos_ws_events_url() -> String {
    if let Ok(url) = std::env::var("PETALTONGUE_WS_ENDPOINT") {
        return url;
    }
    let port = std::env::var("BIOMEOS_WS_PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(DEFAULT_HEADLESS_PORT);
    format!("ws://{DEFAULT_LOOPBACK_HOST}:{port}/events")
}

/// Health check thresholds
pub mod thresholds {
    /// CPU usage percentage above which to emit a warning
    pub const CPU_WARNING: f64 = 80.0;
    /// Memory usage percentage above which to emit a warning
    pub const MEMORY_WARNING: f64 = 50.0;
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
    fn test_default_web_bind() {
        env_test_helpers::with_env_vars(
            &[
                ("PETALTONGUE_WEB_PORT", None),
                ("PETALTONGUE_HEADLESS_PORT", None),
            ],
            || {
                assert_eq!(super::default_web_bind(), "0.0.0.0:3000");
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
                assert_eq!(super::default_headless_bind(), "0.0.0.0:8080");
            },
        );
    }

    #[test]
    fn test_default_web_bind_env_override() {
        env_test_helpers::with_env_var("PETALTONGUE_WEB_PORT", "4000", || {
            assert_eq!(super::default_web_bind(), "0.0.0.0:4000");
        });
    }

    #[test]
    fn test_default_headless_bind_env_override() {
        env_test_helpers::with_env_var("PETALTONGUE_HEADLESS_PORT", "9000", || {
            assert_eq!(super::default_headless_bind(), "0.0.0.0:9000");
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
}
