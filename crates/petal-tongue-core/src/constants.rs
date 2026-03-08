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
pub const BIOMEOS_SOCKET_NAME: &str = "biomeos-neural-api";

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
#[must_use]
pub fn biomeos_legacy_socket() -> std::path::PathBuf {
    std::path::PathBuf::from(LEGACY_TMP_PREFIX).join(format!("{BIOMEOS_SOCKET_NAME}.sock"))
}

/// Health check thresholds
pub mod thresholds {
    /// CPU usage percentage above which to emit a warning
    pub const CPU_WARNING: f64 = 80.0;
    /// Memory usage percentage above which to emit a warning
    pub const MEMORY_WARNING: f64 = 50.0;
}
