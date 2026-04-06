// SPDX-License-Identifier: AGPL-3.0-or-later
//! Timeout constants for HTTP, discovery, RPC, and lifecycle operations.

use std::time::Duration;

use super::{env_duration_ms, env_duration_secs, env_or};

// ---------------------------------------------------------------------------
// Discovery HTTP client timeouts
// ---------------------------------------------------------------------------

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

    /// Discovery service UDS connect timeout (aggressive for non-blocking discovery)
    pub const DISCOVERY_SERVICE_CONNECT_TIMEOUT: Duration = Duration::from_millis(200);
    /// Registration / availability probe: max wait for `UnixStream::connect` to discovery socket
    pub const DISCOVERY_SERVICE_REGISTRATION_PROBE_TIMEOUT: Duration = Duration::from_millis(100);
    /// Discovery service UDS write timeout
    pub const DISCOVERY_SERVICE_WRITE_TIMEOUT: Duration = Duration::from_millis(100);
    /// Discovery service UDS read timeout
    pub const DISCOVERY_SERVICE_READ_TIMEOUT: Duration = Duration::from_millis(500);
}

// ---------------------------------------------------------------------------
// RPC and lifecycle defaults
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Duration helpers (env-driven with fallback)
// ---------------------------------------------------------------------------

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
