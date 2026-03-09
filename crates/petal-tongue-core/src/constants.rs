// SPDX-License-Identifier: AGPL-3.0-only
//! Centralized constants for the petalTongue primal.
//!
//! Self-knowledge only -- no hardcoded knowledge of other primals.
//! Other primals are discovered at runtime via socket/mDNS/JSON-RPC.

/// Display name for this primal
pub const PRIMAL_NAME: &str = "petalTongue";

/// Application name used for directory paths (XDG conventions)
pub const APP_DIR_NAME: &str = "petaltongue";

/// Socket file prefix for IPC
pub const SOCKET_PREFIX: &str = "petaltongue";

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

/// Default sandbox security endpoint port (fallback when PETALTONGUE_SANDBOX_SECURITY_ENDPOINT not set)
pub const DEFAULT_SANDBOX_SECURITY_PORT: u16 = 9000;

/// Default sandbox discovery endpoint port (fallback when PETALTONGUE_SANDBOX_DISCOVERY_ENDPOINT not set)
pub const DEFAULT_SANDBOX_DISCOVERY_PORT: u16 = 8080;

/// Max FPS for rendering (overridable via config)
pub const DEFAULT_MAX_FPS: u32 = 60;

/// Default bind address for servers
pub const DEFAULT_BIND_ADDR: &str = "0.0.0.0";

/// Legacy /tmp socket fallback path prefix
pub const LEGACY_TMP_PREFIX: &str = "/tmp";

/// Build a default web bind address
#[must_use]
pub fn default_web_bind() -> String {
    format!("{DEFAULT_BIND_ADDR}:{DEFAULT_WEB_PORT}")
}

/// Build a default headless bind address
#[must_use]
pub fn default_headless_bind() -> String {
    format!("{DEFAULT_BIND_ADDR}:{DEFAULT_HEADLESS_PORT}")
}

/// Build a legacy biomeOS socket path (/tmp/biomeos-neural-api.sock)
/// Uses BIOMEOS_SOCKET_NAME env var if set
#[must_use]
pub fn biomeos_legacy_socket() -> std::path::PathBuf {
    std::path::PathBuf::from(LEGACY_TMP_PREFIX).join(format!("{}.sock", biomeos_socket_name()))
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
        assert_eq!(super::default_web_bind(), "0.0.0.0:3000");
    }

    #[test]
    fn test_default_headless_bind() {
        assert_eq!(super::default_headless_bind(), "0.0.0.0:8080");
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
}
