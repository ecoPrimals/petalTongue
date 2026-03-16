// SPDX-License-Identifier: AGPL-3.0-only
//! Neural API self-registration for petalTongue.
//!
//! On startup, discovers the local biomeOS Neural API and announces petalTongue's
//! capabilities (`ui.render`, `visualization.render`, `ipc.json-rpc`,
//! `interaction.sensor_stream`). Sends periodic `lifecycle.status` heartbeats
//! so biomeOS can monitor liveness and route capabilities.

use petal_tongue_discovery::NeuralApiProvider;
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

/// petalTongue capabilities advertised to the Neural API.
#[must_use]
pub fn petaltongue_capabilities() -> Vec<&'static str> {
    vec![
        "ui.render",
        "visualization.render",
        "ipc.json-rpc",
        "interaction.sensor_stream",
    ]
}

/// Register petalTongue with the Neural API lifecycle manager.
///
/// Discovers the Neural API, calls `lifecycle.register` with our socket path
/// and capabilities, then returns the provider for heartbeat use.
pub async fn register_with_neural_api(
    our_socket: &Path,
) -> crate::error::Result<Arc<NeuralApiProvider>> {
    let provider = NeuralApiProvider::discover(None).await?;

    let caps = petaltongue_capabilities();
    let params = json!({
        "name": petal_tongue_core::constants::PRIMAL_NAME,
        "socket_path": our_socket.to_string_lossy(),
        "pid": std::process::id(),
        "capabilities": caps,
    });

    provider
        .call_method("lifecycle.register", Some(params))
        .await
        .map_err(|e| {
            tracing::warn!("lifecycle.register failed (non-fatal): {e}");
            e
        })?;

    tracing::info!("Registered with Neural API: {} capabilities", caps.len());

    Ok(Arc::new(provider))
}

/// Spawn a background heartbeat task that sends `lifecycle.status` every 30s.
///
/// Runs as a tokio task — no extra OS thread or runtime. Exits when the
/// `NeuralApiProvider` call fails (e.g., socket becomes unreachable).
pub fn spawn_heartbeat(provider: Arc<NeuralApiProvider>, our_socket: PathBuf) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(HEARTBEAT_INTERVAL).await;

            let params = json!({
                "name": petal_tongue_core::constants::PRIMAL_NAME,
                "socket_path": our_socket.to_string_lossy(),
                "status": "healthy",
                "pid": std::process::id(),
            });

            match provider.call_method("lifecycle.status", Some(params)).await {
                Ok(_) => tracing::trace!("Neural API heartbeat sent"),
                Err(e) => {
                    tracing::debug!("Neural API heartbeat failed: {e}");
                    break;
                }
            }
        }

        tracing::info!("Neural API heartbeat task exiting");
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capabilities_are_non_empty() {
        let caps = petaltongue_capabilities();
        assert!(!caps.is_empty());
        assert!(caps.contains(&"visualization.render"));
        assert!(caps.contains(&"interaction.sensor_stream"));
    }

    #[test]
    fn capabilities_use_domain_dot_operation_naming() {
        for cap in petaltongue_capabilities() {
            assert!(
                cap.contains('.'),
                "capability '{cap}' should use domain.operation naming"
            );
        }
    }
}
