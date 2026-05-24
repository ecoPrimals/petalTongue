// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared shutdown signal handler (Wave 47).
//!
//! Per `DEPLOYMENT_BEHAVIOR_STANDARD.md`: all long-running primals must handle
//! both SIGINT (Ctrl+C) and SIGTERM so `nucleus_launcher.sh` and systemd can
//! shut them down cleanly.

/// Wait for SIGTERM or Ctrl+C.
pub async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();
    #[cfg(unix)]
    {
        #[expect(clippy::expect_used, reason = "SIGTERM registration is unrecoverable")]
        let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("SIGTERM handler registration");
        tokio::select! {
            _ = ctrl_c => tracing::info!("SIGINT received, shutting down"),
            _ = sigterm.recv() => tracing::info!("SIGTERM received, shutting down"),
        }
    }
    #[cfg(not(unix))]
    {
        ctrl_c.await.ok();
        tracing::info!("SIGINT received, shutting down");
    }
}
