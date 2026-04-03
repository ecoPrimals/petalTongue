// SPDX-License-Identifier: AGPL-3.0-or-later
//! Discovery service client
//!
//! Queries the ecosystem discovery service for capability-based discovery of primals.
//! petalTongue never hardcodes which primal provides discovery — it connects to
//! whichever service is listening on the `discovery-service` socket role.
//!
//! - Aggressive timeouts to prevent hanging
//! - Non-blocking operations throughout
//! - Proper error propagation

mod methods;
mod protocol;

#[cfg(test)]
mod tests;

use crate::errors::DiscoveryResult;
use std::path::PathBuf;
use tracing::{info, warn};

/// Discovery service client for primal discovery
///
/// Connects to the ecosystem discovery service via Unix socket and queries
/// for registered primals by capability.
#[derive(Debug)]
pub struct DiscoveryServiceClient {
    /// Path to discovery service Unix socket
    socket_path: PathBuf,
}

impl DiscoveryServiceClient {
    /// Discover discovery service Unix socket (capability-based, no hardcoded primal names)
    ///
    /// Uses `DISCOVERY_SERVICE_SOCKET` env for socket basename (default: `discovery-service`).
    /// Override when your deployment installs the discovery registry under a different socket name.
    ///
    /// # Errors
    /// Returns `DiscoveryError::DiscoveryServiceNotFound` if no socket found in search paths.
    pub fn discover(family_id: Option<&str>) -> DiscoveryResult<Self> {
        let family = family_id
            .map(String::from)
            .or_else(|| std::env::var("FAMILY_ID").ok())
            .unwrap_or_else(|| "nat0".to_string());

        let socket_base = petal_tongue_core::constants::discovery_service_socket_name();
        let socket_name = format!("{socket_base}-{family}.sock");

        // Try XDG_RUNTIME_DIR first
        let search_paths = Self::get_search_paths();

        for base_path in search_paths {
            let socket_path = base_path.join(&socket_name);
            if socket_path.exists() {
                info!("🔍 Found discovery service at: {}", socket_path.display());
                return Ok(Self { socket_path });
            }
        }

        // Discovery service not found
        warn!("⚠️ Discovery service not found in standard locations");
        warn!("   Searched for: {}", socket_name);
        warn!("   Search paths:");
        for path in Self::get_search_paths() {
            warn!("     - {}", path.display());
        }

        Err(crate::errors::DiscoveryError::DiscoveryServiceNotFound { socket_name })
    }

    /// Get standard search paths for Unix sockets
    pub(crate) fn get_search_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Priority 1: XDG_RUNTIME_DIR
        if let Ok(xdg_runtime) = std::env::var("XDG_RUNTIME_DIR") {
            paths.push(PathBuf::from(xdg_runtime));
        }

        // Priority 2: /run/user/<uid>
        // EVOLVED: Now using safe rustix-based function from core (was unsafe libc::getuid())
        let uid = petal_tongue_core::system_info::get_current_uid();
        paths.push(PathBuf::from(format!("/run/user/{uid}")));

        // Priority 3: /tmp (development)
        paths.push(PathBuf::from("/tmp"));

        paths
    }

    /// Create client with explicit socket path (for testing)
    #[must_use]
    pub const fn with_socket_path(socket_path: PathBuf) -> Self {
        Self { socket_path }
    }

    /// Get the socket path (for metadata/debugging)
    #[must_use]
    pub const fn socket_path(&self) -> &PathBuf {
        &self.socket_path
    }
}
