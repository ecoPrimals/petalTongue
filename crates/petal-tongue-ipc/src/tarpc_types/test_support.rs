// SPDX-License-Identifier: AGPL-3.0-or-later

use petal_tongue_core::constants::{DEFAULT_DISPLAY_BACKEND_PORT, DEFAULT_LOOPBACK_HOST};

pub use petal_tongue_core::constants::PRIMAL_NAME;

/// Default tarpc endpoint (loopback:port) for tests and fallbacks.
#[must_use]
pub fn default_tarpc_endpoint() -> String {
    format!("tarpc://{DEFAULT_LOOPBACK_HOST}:{DEFAULT_DISPLAY_BACKEND_PORT}")
}

/// Default tcp endpoint (loopback:port) for tests and fallbacks.
#[must_use]
pub fn default_tcp_endpoint() -> String {
    format!("tcp://{DEFAULT_LOOPBACK_HOST}:{DEFAULT_DISPLAY_BACKEND_PORT}")
}
