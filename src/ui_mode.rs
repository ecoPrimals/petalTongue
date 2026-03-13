// SPDX-License-Identifier: AGPL-3.0-only
//! UI mode - Desktop GUI
//!
//! Platform dependencies: wayland-sys, x11-sys (acceptable for ecoBud)
//! This is the 1 mode (out of 5) that has platform dependencies

use anyhow::{Context, Result};
use petal_tongue_core::constants::PRIMAL_NAME;
use std::sync::Arc;

#[cfg(feature = "ui")]
pub async fn run(
    scenario: Option<String>,
    no_audio: bool,
    data_service: Arc<crate::data_service::DataService>,
) -> Result<()> {
    tracing::info!(
        scenario = ?scenario,
        no_audio,
        "Starting desktop GUI mode"
    );

    // Run in blocking context (egui is not async)
    tokio::task::spawn_blocking(move || run_ui_blocking(scenario, no_audio, &data_service))
        .await
        .context("UI task panicked")?
}

#[cfg(feature = "ui")]
fn run_ui_blocking(
    scenario: Option<String>,
    _no_audio: bool,
    data_service: &Arc<crate::data_service::DataService>,
) -> Result<()> {
    use petal_tongue_core::{InstanceId, RenderingCapabilities};
    use petal_tongue_ui::PetalTongueApp;
    use std::path::PathBuf;

    // Create instance
    let instance_id = InstanceId::new();
    tracing::info!(
        "🌸 Starting {} UI instance: {}",
        PRIMAL_NAME,
        instance_id.as_str()
    );

    // Convert scenario to PathBuf
    let scenario_path = scenario.map(PathBuf::from);

    // Detect rendering capabilities
    let capabilities = RenderingCapabilities::detect();

    // Setup eframe options
    let options = petal_tongue_ui::eframe::NativeOptions {
        viewport: petal_tongue_ui::egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title(format!(
                "🌸 {PRIMAL_NAME} - Universal Representation System"
            ))
            .with_visible(true)
            .with_active(true),
        ..Default::default()
    };

    // Create and run app
    // IMPORTANT: We pass the shared graph from DataService directly
    // This ensures the GUI uses the SAME data as all other modes
    let shared_graph = data_service.graph();

    petal_tongue_ui::eframe::run_native(
        PRIMAL_NAME,
        options,
        Box::new(move |cc| {
            let app = PetalTongueApp::new_with_shared_graph(
                cc,
                scenario_path,
                capabilities,
                shared_graph,
            )?;
            Ok(Box::new(app))
        }),
    )
    .map_err(|e| anyhow::anyhow!("eframe error: {e}"))
}

#[cfg(not(feature = "ui"))]
pub async fn run(
    _scenario: Option<String>,
    _no_audio: bool,
    _data_service: std::sync::Arc<crate::data_service::DataService>,
) -> Result<()> {
    anyhow::bail!(
        "UI mode not available in this build\n\
        Tip: Rebuild with --features ui or use:\n\
        - petaltongue tui (terminal UI)\n\
        - petaltongue web (web UI)"
    )
}

#[cfg(test)]
mod tests {
    #[cfg(not(feature = "ui"))]
    use std::sync::Arc;

    #[cfg(not(feature = "ui"))]
    use super::*;

    #[tokio::test]
    #[cfg(feature = "ui")]
    async fn test_ui_mode_signature() {
        // Can't test actual UI in headless environment
        // This just verifies the function signature compiles
    }

    #[tokio::test]
    #[cfg(not(feature = "ui"))]
    async fn test_ui_mode_not_available() {
        let data_service = Arc::new(crate::data_service::DataService::new());
        let result = run(None, false, data_service).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not available"));
    }

    #[tokio::test]
    #[cfg(not(feature = "ui"))]
    async fn test_ui_mode_error_suggests_alternatives() {
        let data_service = Arc::new(crate::data_service::DataService::new());
        let result = run(None, false, data_service).await;
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("tui"));
        assert!(err_msg.contains("web"));
    }

    #[tokio::test]
    #[cfg(not(feature = "ui"))]
    async fn test_ui_mode_with_scenario_still_fails() {
        let data_service = Arc::new(crate::data_service::DataService::new());
        let result = run(Some("scenario.json".to_string()), true, data_service).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[cfg(not(feature = "ui"))]
    async fn test_ui_mode_error_contains_rebuild_tip() {
        let data_service = Arc::new(crate::data_service::DataService::new());
        let result = run(None, false, data_service).await;
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("--features ui") || err_msg.contains("Rebuild"),
            "Error should suggest rebuild: {err_msg}"
        );
    }

    #[tokio::test]
    #[cfg(not(feature = "ui"))]
    async fn test_ui_mode_no_audio_arg_still_fails() {
        let data_service = Arc::new(crate::data_service::DataService::new());
        let result = run(None, true, data_service).await;
        assert!(
            result.is_err(),
            "no_audio=true should still fail when ui disabled"
        );
    }

    #[tokio::test]
    #[cfg(not(feature = "ui"))]
    async fn test_ui_mode_empty_scenario_string_still_fails() {
        let data_service = Arc::new(crate::data_service::DataService::new());
        let result = run(Some(String::new()), false, data_service).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[cfg(not(feature = "ui"))]
    async fn test_ui_mode_invalid_path_scenario_still_fails() {
        let data_service = Arc::new(crate::data_service::DataService::new());
        let result = run(
            Some("/nonexistent/path/to/scenario.json".to_string()),
            false,
            data_service,
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[cfg(not(feature = "ui"))]
    async fn test_ui_mode_error_is_anyhow() {
        let data_service = Arc::new(crate::data_service::DataService::new());
        let result = run(None, false, data_service).await;
        let err = result.unwrap_err();
        // Verify it's a proper anyhow chain (has at least one error in chain)
        assert!(err.chain().next().is_some());
    }
}
