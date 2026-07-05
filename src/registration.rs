// SPDX-License-Identifier: AGPL-3.0-or-later
//! Ecosystem registration and Neural API announcement.
//!
//! Implements the `ipc.register` standard from `PRIMAL_IPC_PROTOCOL.md` and
//! the `primal.announce` protocol for biomeOS Neural API routing (Wave 43).

/// Register petalTongue with the ecosystem discovery service.
///
/// This implements the `ipc.register` standard from `PRIMAL_IPC_PROTOCOL.md`.
/// Uses capability-based discovery to find the registration service (any primal
/// providing the "discovery" capability).
///
/// When `tcp_port` is `Some`, the registration advertises the TCP endpoint
/// so Songbird can return it for tier-1 `ipc.resolve` routing.
///
/// # TRUE PRIMAL: Capability-Based Registration
/// - Discovers the registration service at runtime (no hardcoded primal name)
/// - Gracefully handles service unavailability (standalone mode works fine)
/// - Self-knowledge only: petalTongue knows its own capabilities, not others
pub async fn register_with_discovery_service(
    tcp_port: Option<u16>,
    tcp_bind_host: std::net::IpAddr,
) {
    use petal_tongue_ipc::primal_registration::{PrimalRegistration, RegistrationManager};

    let mut registration = PrimalRegistration::petaltongue();
    if let Some(port) = tcp_port {
        registration = registration.with_tcp_endpoint(tcp_bind_host, port);
    }

    tracing::debug!(
        "📝 Registration: {} v{} with {} capabilities, transports={:?}",
        registration.name,
        registration.version,
        registration.capabilities.len(),
        registration.transports,
    );

    let manager = RegistrationManager::new(registration);
    manager.register_on_startup().await;
    let _heartbeat_handle = manager.spawn_heartbeat_task();

    tracing::debug!("✅ Primal registration complete (heartbeat task spawned)");
}

/// Announce to biomeOS Neural API for capability-based routing (Wave 43).
///
/// Sends `primal.announce` with cost hints and latency estimates so the Neural
/// API can make intelligent routing decisions. Fire-and-forget: standalone mode
/// works fine if the Neural API is unavailable.
pub async fn announce_to_neural_api() {
    use petal_tongue_core::capability_names::primal_names;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let family = std::env::var(petal_tongue_core::constants::FAMILY_ID)
        .unwrap_or_else(|_| "nat0".to_owned());
    let socket_dir = std::env::var(petal_tongue_core::constants::XDG_RUNTIME_DIR)
        .unwrap_or_else(|_| petal_tongue_core::constants::LEGACY_TMP_PREFIX.to_owned());
    let socket = format!("{socket_dir}/biomeos/neural-api-{family}.sock");

    let uds_path = petal_tongue_ipc::socket_path::get_petaltongue_socket_path()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "primal.announce",
        "params": {
            "primal": primal_names::PETALTONGUE,
            "version": env!("CARGO_PKG_VERSION"),
            "socket": uds_path,
            "capabilities": ["render", "ui", "accessibility"],
            "methods": petal_tongue_core::capability_names::self_capabilities::ALL,
            "signal_tiers": ["meta"],
            "cost_hints": {
                "render": 30.0,
                "ui": 20.0,
                "accessibility": 10.0,
            },
            "latency_estimates": {
                "render": 16,
                "ui": 10,
                "accessibility": 5,
            },
        },
        "id": 1,
    });

    let Ok(mut stream) = tokio::time::timeout(
        std::time::Duration::from_millis(500),
        tokio::net::UnixStream::connect(&socket),
    )
    .await
    .unwrap_or(Err(std::io::ErrorKind::TimedOut.into())) else {
        tracing::debug!(
            socket,
            "Neural API not reachable — skipping primal.announce"
        );
        return;
    };

    let mut buf = serde_json::to_vec(&payload).unwrap_or_default();
    buf.push(b'\n');
    if stream.write_all(&buf).await.is_err() || stream.flush().await.is_err() {
        tracing::debug!("Neural API write failed — skipping primal.announce");
        return;
    }

    let (reader, _) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    if let Ok(Ok(_)) = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        reader.read_line(&mut line),
    )
    .await
    {
        tracing::info!("Neural API primal.announce accepted");
    } else {
        tracing::debug!("Neural API response timeout — announce may still have succeeded");
    }
}
