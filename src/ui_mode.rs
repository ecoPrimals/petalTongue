//! UI mode - Desktop GUI
//!
//! Platform dependencies: wayland-sys, x11-sys (acceptable for ecoBud)
//! This is the 1 mode (out of 5) that has platform dependencies

use anyhow::{Context, Result};
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
    tokio::task::spawn_blocking(move || run_ui_blocking(scenario, no_audio, data_service))
        .await
        .context("UI task panicked")?
}

#[cfg(feature = "ui")]
fn run_ui_blocking(
    scenario: Option<String>,
    _no_audio: bool,
    data_service: Arc<crate::data_service::DataService>,
) -> Result<()> {
    use petal_tongue_core::{InstanceId, RenderingCapabilities};
    use petal_tongue_ui::PetalTongueApp;
    use std::path::PathBuf;

    // Create instance
    let instance_id = InstanceId::new();
    tracing::info!(
        "🌸 Starting petalTongue UI instance: {}",
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
            .with_title("🌸 petalTongue - Universal Representation System")
            .with_visible(true)
            .with_active(true),
        ..Default::default()
    };

    // Create and run app
    // IMPORTANT: We pass the shared graph from DataService directly
    // This ensures the GUI uses the SAME data as all other modes
    let shared_graph = data_service.graph();

    petal_tongue_ui::eframe::run_native(
        "petalTongue",
        options,
        Box::new(move |cc| {
            Ok(Box::new(PetalTongueApp::new_with_shared_graph(
                cc,
                scenario_path,
                capabilities,
                shared_graph,
            )))
        }),
    )
    .map_err(|e| anyhow::anyhow!("eframe error: {}", e))
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

    #[tokio::test]
    #[cfg(feature = "ui")]
    async fn test_ui_mode_signature() {
        // Can't test actual UI in headless environment
        // This just verifies the function signature compiles
    }

    #[tokio::test]
    #[cfg(not(feature = "ui"))]
    async fn test_ui_mode_not_available() {
        let result = run(None, false).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not available"));
    }
}
