// SPDX-License-Identifier: AGPL-3.0-or-later
//! Headless mode - Pure Rust rendering without display
//!
//! Pure Rust! ✅
//! No display dependencies

use crate::data_service::DataService;
use crate::error::AppError;
use std::io::Write;
use std::sync::Arc;

/// Run headless mode with output written to the given writer (for testing).
pub async fn run_with_output<W: Write + Send>(
    _bind: &str,
    _workers: usize,
    data_service: Arc<DataService>,
    out: &mut W,
) -> Result<(), AppError> {
    tracing::info!("Starting headless rendering mode (Pure Rust!)");
    tracing::info!("✅ Using shared DataService (zero duplication!)");

    writeln!(out, "🌸 petalTongue headless mode (Pure Rust!)")?;
    writeln!(out, "Headless mode active - Pure Rust rendering ready")?;

    let snapshot_result = data_service.snapshot().await;
    match snapshot_result {
        Ok(snapshot) => {
            writeln!(out)?;
            writeln!(out, "📊 Data from unified service:")?;
            writeln!(out, "  Primals: {}", snapshot.primals.len())?;
            writeln!(out, "  Edges: {}", snapshot.edges.len())?;
        }
        Err(e) => {
            tracing::warn!("Failed to get snapshot: {}", e);
        }
    }

    tracing::info!("Headless mode started successfully");
    Ok(())
}

/// Run headless mode (writes to stdout).
pub async fn run(
    bind: &str,
    workers: usize,
    data_service: Arc<DataService>,
) -> Result<(), AppError> {
    run_with_output(bind, workers, data_service, &mut std::io::stdout()).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_headless_mode() {
        let data_service = Arc::new(DataService::new());
        let result = run(
            &petal_tongue_core::constants::default_headless_bind(),
            4,
            data_service,
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_headless_concurrent() {
        // Test runs in parallel with others - no sleeps needed!
        let handles: Vec<_> = (0..4)
            .map(|i| {
                tokio::spawn(async move {
                    let port = format!("0.0.0.0:{}", 8080 + i);
                    let data_service = Arc::new(DataService::new());
                    run(&port, 1, data_service).await
                })
            })
            .collect();

        for handle in handles {
            assert!(handle.await.is_ok());
        }
    }

    #[tokio::test]
    async fn test_headless_output_format() {
        let mut output = Vec::new();
        let data_service = Arc::new(DataService::new());
        run_with_output("0.0.0.0:8080", 4, data_service, &mut output)
            .await
            .unwrap();

        let stdout = String::from_utf8_lossy(&output);
        assert!(
            stdout.contains("petalTongue headless mode"),
            "expected headless mode banner: {stdout}"
        );
        assert!(
            stdout.contains("Pure Rust"),
            "expected Pure Rust branding: {stdout}"
        );
        assert!(
            stdout.contains("Data from unified service"),
            "expected data section: {stdout}"
        );
        assert!(
            stdout.contains("Primals:") && stdout.contains("Edges:"),
            "expected primals/edges counts: {stdout}"
        );
    }

    #[tokio::test]
    async fn test_headless_snapshot_error_path() {
        // Poison the graph lock so snapshot() returns Err
        let data_service = Arc::new(DataService::new());
        let graph = data_service.graph();
        let _ = std::thread::spawn(move || {
            let _guard = graph.write().unwrap();
            panic!("intentional panic to poison lock");
        })
        .join();

        // run_with_output should still return Ok (error is logged, not propagated)
        let mut output = Vec::new();
        let result = run_with_output("0.0.0.0:8080", 4, data_service, &mut output).await;
        assert!(
            result.is_ok(),
            "run should succeed even when snapshot fails"
        );

        let stdout = String::from_utf8_lossy(&output);
        // Banner should still be printed before snapshot
        assert!(stdout.contains("petalTongue headless mode"));
        assert!(stdout.contains("Headless mode active"));
        // Data section should NOT appear (snapshot failed)
        assert!(
            !stdout.contains("Primals:"),
            "data section should be omitted when snapshot fails"
        );
    }
}
