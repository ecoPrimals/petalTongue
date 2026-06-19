// SPDX-License-Identifier: AGPL-3.0-or-later
//! Centralized constants for the petalTongue primal.
//!
//! Self-knowledge only -- no hardcoded knowledge of other primals.
//! Other primals are discovered at runtime via socket/mDNS/JSON-RPC.
//!
//! Organized by domain:
//! - Network — ports, hosts, sockets, endpoints, bind addresses, URLs
//! - Display — window geometry, terminal, FPS, frame pacing
//! - Timeouts — RPC, lifecycle, cache; [`discovery_timeouts`] — HTTP/UDS
//! - [`thresholds`] — health warnings; [`tufte_tolerances`] — visualization

mod display;
mod env_vars;
mod network;
mod network_defaults;
mod timeouts;

pub mod thresholds;
pub mod tufte_tolerances;

pub use display::*;
pub use env_vars::*;
pub use network::*;
pub use network_defaults::NetworkDefaults;
pub use timeouts::*;

use std::borrow::Cow;
use std::time::Duration;

/// Read an env var or return a borrowed compile-time default (zero-copy when unset).
#[must_use]
pub(crate) fn env_or_borrowed_str(key: &str, default: &'static str) -> Cow<'static, str> {
    std::env::var(key).map_or(Cow::Borrowed(default), Cow::Owned)
}

/// Read an env var, parse it as `T`, or return `default`.
pub(crate) fn env_or<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse::<T>().ok())
        .unwrap_or(default)
}

/// Read an env var as `Duration` in seconds, or return `default_secs`.
pub(crate) fn env_duration_secs(key: &str, default_secs: u64) -> Duration {
    Duration::from_secs(env_or(key, default_secs))
}

/// Read an env var as `Duration` in milliseconds, or return `default_ms`.
pub(crate) fn env_duration_ms(key: &str, default_ms: u64) -> Duration {
    Duration::from_millis(env_or(key, default_ms))
}

/// Display name for this primal
pub const PRIMAL_NAME: &str = "petalTongue";

/// Application name used for directory paths (XDG conventions)
pub const APP_DIR_NAME: &str = "petaltongue";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primal_name() {
        assert_eq!(PRIMAL_NAME, "petalTongue");
    }

    #[test]
    fn app_dir_name() {
        assert_eq!(APP_DIR_NAME, "petaltongue");
    }
}
