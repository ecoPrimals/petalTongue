// SPDX-License-Identifier: AGPL-3.0-or-later
//! Network constants: ports, hosts, sockets, endpoints, bind addresses, URLs.
//!
//! All defaults are overridable via environment variables. Socket names use
//! capability-based naming (no hardcoded primal identities).

use super::env_or;

// ---------------------------------------------------------------------------
// Ports
// ---------------------------------------------------------------------------

/// Default web port (overridable via config or `PETALTONGUE_WEB_PORT` env)
pub const DEFAULT_WEB_PORT: u16 = 3000;

/// Default headless API port (overridable via config or `PETALTONGUE_HEADLESS_PORT` env)
pub const DEFAULT_HEADLESS_PORT: u16 = 8080;

/// Default WebSocket streaming port for software renderer remote access.
/// Overridable via `WEBSOCKET_PORT` env var.
pub const DEFAULT_WEBSOCKET_PORT: u16 = 8765;

/// Default sandbox security endpoint port.
/// Overridable via `PETALTONGUE_SANDBOX_SECURITY_ENDPOINT` (full URL) env var.
pub const DEFAULT_SANDBOX_SECURITY_PORT: u16 = 9000;

/// Default sandbox discovery endpoint port.
/// Overridable via `PETALTONGUE_SANDBOX_DISCOVERY_ENDPOINT` (full URL) env var.
pub const DEFAULT_SANDBOX_DISCOVERY_PORT: u16 = 8080;

/// Default display backend / GPU compute port (overridable via `DISPLAY_BACKEND_PORT` env var).
pub const DEFAULT_DISPLAY_BACKEND_PORT: u16 = 9001;

/// Default ports for HTTP discovery when `PETALTONGUE_DISCOVERY_PORTS` / `DISCOVERY_PORTS` not set.
pub const DEFAULT_DISCOVERY_PORTS: &[u16] = &[8080, 8081, 3000, 9000];

// ---------------------------------------------------------------------------
// Hosts
// ---------------------------------------------------------------------------

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

/// Legacy /tmp socket fallback path prefix.
/// Used when `XDG_RUNTIME_DIR` is unavailable; configurable via explicit socket env vars.
pub const LEGACY_TMP_PREFIX: &str = "/tmp";

// ---------------------------------------------------------------------------
// Socket names (capability-based, no primal identity coupling)
// ---------------------------------------------------------------------------

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

/// Legacy /tmp biomeOS socket name (fallback).
///
/// Default is `"biomeos"` — a well-known ecosystem convention, not a
/// compile-time primal dependency. Overridable via `BIOMEOS_LEGACY_SOCKET`.
#[must_use]
pub fn biomeos_legacy_socket_name() -> String {
    std::env::var("BIOMEOS_LEGACY_SOCKET").unwrap_or_else(|_| "biomeos".to_string())
}

// ---------------------------------------------------------------------------
// Port helpers (env-driven with fallback)
// ---------------------------------------------------------------------------

/// Display backend port (env-driven with fallback).
/// Reads `DISPLAY_BACKEND_PORT`; falls back to `DEFAULT_DISPLAY_BACKEND_PORT`.
#[must_use]
pub fn display_backend_port() -> u16 {
    env_or("DISPLAY_BACKEND_PORT", DEFAULT_DISPLAY_BACKEND_PORT)
}

// ---------------------------------------------------------------------------
// Bind addresses
// ---------------------------------------------------------------------------

/// Default bind address for servers (loopback-only for security).
///
/// Overridable via `--bind` CLI, config, or `PETALTONGUE_BIND_ADDR` env var.
/// Port comes from `PETALTONGUE_WEB_PORT` / `PETALTONGUE_HEADLESS_PORT`.
pub fn default_bind_addr() -> &'static str {
    static BIND: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    BIND.get_or_init(|| {
        std::env::var("PETALTONGUE_BIND_ADDR").unwrap_or_else(|_| DEFAULT_LOOPBACK_HOST.to_string())
    })
}

/// Build a default web bind address.
///
/// Port is overridable via `PETALTONGUE_WEB_PORT` env var.
#[must_use]
pub fn default_web_bind() -> String {
    let port = env_or("PETALTONGUE_WEB_PORT", DEFAULT_WEB_PORT);
    let addr = default_bind_addr();
    format!("{addr}:{port}")
}

/// Build a default headless bind address.
///
/// Port is overridable via `PETALTONGUE_HEADLESS_PORT` env var.
#[must_use]
pub fn default_headless_bind() -> String {
    let port = env_or("PETALTONGUE_HEADLESS_PORT", DEFAULT_HEADLESS_PORT);
    let addr = default_bind_addr();
    format!("{addr}:{port}")
}

/// Build a legacy biomeOS socket path (`/tmp/biomeos-neural-api.sock`).
/// Uses `BIOMEOS_SOCKET_NAME` env var if set.
#[must_use]
pub fn biomeos_legacy_socket() -> std::path::PathBuf {
    std::path::PathBuf::from(LEGACY_TMP_PREFIX).join(format!("{}.sock", biomeos_socket_name()))
}

// ---------------------------------------------------------------------------
// Endpoint URLs (env-driven with fallback)
// ---------------------------------------------------------------------------

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
/// Alias for [`default_gpu_compute_endpoint`].
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

/// Default biomeOS connection target for display (e.g. `"localhost:3000"`).
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

/// Default web server URL (e.g. `"http://localhost:3000"`).
/// Uses `PETALTONGUE_WEB_URL` or `BIOMEOS_URL`; fallback from [`default_biomeos_connection_target`].
#[must_use]
pub fn default_web_url() -> String {
    std::env::var("PETALTONGUE_WEB_URL")
        .or_else(|_| std::env::var("BIOMEOS_URL"))
        .unwrap_or_else(|_| format!("http://{}", default_biomeos_connection_target()))
}

/// Default entropy stream endpoint.
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

/// Default headless API URL for display.
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

/// Default sandbox security URL.
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::test_fixtures::env_test_helpers;

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
            assert_eq!(default_bind_addr(), "127.0.0.1");
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
                let bind = default_web_bind();
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
                let bind = default_headless_bind();
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
            let bind = default_web_bind();
            assert!(
                bind.ends_with(":4000"),
                "should use overridden port: {bind}"
            );
        });
    }

    #[test]
    fn test_default_headless_bind_env_override() {
        env_test_helpers::with_env_var("PETALTONGUE_HEADLESS_PORT", "9000", || {
            let bind = default_headless_bind();
            assert!(
                bind.ends_with(":9000"),
                "should use overridden port: {bind}"
            );
        });
    }

    #[test]
    fn test_biomeos_socket_name_default() {
        env_test_helpers::with_env_var_removed("BIOMEOS_SOCKET_NAME", || {
            assert_eq!(biomeos_socket_name(), "biomeos-neural-api");
        });
    }

    #[test]
    fn test_biomeos_socket_name_override() {
        env_test_helpers::with_env_var("BIOMEOS_SOCKET_NAME", "custom-socket", || {
            assert_eq!(biomeos_socket_name(), "custom-socket");
        });
    }

    #[test]
    fn test_biomeos_device_management_socket_override() {
        env_test_helpers::with_env_var(
            "BIOMEOS_DEVICE_MANAGEMENT_SOCKET",
            "custom-device-mgmt",
            || {
                assert_eq!(
                    biomeos_device_management_socket_name(),
                    "custom-device-mgmt"
                );
            },
        );
    }

    #[test]
    fn test_biomeos_ui_socket_override() {
        env_test_helpers::with_env_var("BIOMEOS_UI_SOCKET", "custom-ui", || {
            assert_eq!(biomeos_ui_socket_name(), "custom-ui");
        });
    }

    #[test]
    fn test_discovery_service_socket_override() {
        env_test_helpers::with_env_var("DISCOVERY_SERVICE_SOCKET", "custom-discovery", || {
            assert_eq!(discovery_service_socket_name(), "custom-discovery");
        });
    }

    #[test]
    fn test_biomeos_legacy_socket_override() {
        env_test_helpers::with_env_var("BIOMEOS_LEGACY_SOCKET", "custom-legacy", || {
            assert_eq!(biomeos_legacy_socket_name(), "custom-legacy");
        });
    }

    #[test]
    fn test_biomeos_legacy_socket_path() {
        env_test_helpers::with_env_var("BIOMEOS_SOCKET_NAME", "test-sock", || {
            let path = biomeos_legacy_socket();
            assert!(path.to_string_lossy().ends_with("test-sock.sock"));
            assert!(path.to_string_lossy().contains("/tmp"));
        });
    }

    #[test]
    fn loopback_host_is_ipv4() {
        assert_eq!(DEFAULT_LOOPBACK_HOST, "127.0.0.1");
    }

    #[test]
    fn test_display_backend_port_default() {
        env_test_helpers::with_env_var_removed("DISPLAY_BACKEND_PORT", || {
            assert_eq!(display_backend_port(), DEFAULT_DISPLAY_BACKEND_PORT);
        });
    }

    #[test]
    fn test_display_backend_port_env_override() {
        env_test_helpers::with_env_var("DISPLAY_BACKEND_PORT", "9100", || {
            assert_eq!(display_backend_port(), 9100);
        });
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
