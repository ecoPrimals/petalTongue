// SPDX-License-Identifier: AGPL-3.0-or-later
//! UI mode - Desktop display
//!
//! Platform dependencies: wayland-sys, x11-sys (acceptable for ecoBud)
//! This is the 1 mode (out of 5) that has platform dependencies
//!
//! **IPC / PT-06:** The UniBin `petaltongue ui` path uses `PetalTongueApp::new_with_shared_graph`
//! without spawning [`petal_tongue_ipc::UnixSocketServer`], so there is no JSON-RPC UDS or
//! PT-06 `callback_tx` here. For IPC with push delivery, run `petaltongue server` or the
//! standalone `petal-tongue` UI binary (which starts [`petal_tongue_ipc::UnixSocketServer`] in `main`).

use petal_tongue_core::constants::PRIMAL_NAME;

use crate::error::AppError;

type Result<T> = std::result::Result<T, AppError>;
use std::path::PathBuf;
use std::sync::Arc;

/// Convert scenario string to `PathBuf` (pure, testable without display).
#[must_use]
pub fn scenario_to_path(scenario: Option<String>) -> Option<PathBuf> {
    scenario.map(PathBuf::from)
}

/// Build the main window title (pure, testable without launching display).
#[must_use]
pub fn window_title() -> String {
    format!("🌸 {PRIMAL_NAME} - Universal Representation System")
}

#[cfg(feature = "ui")]
pub async fn run(
    scenario: Option<String>,
    no_audio: bool,
    data_service: Arc<crate::data_service::DataService>,
) -> Result<()> {
    tracing::info!(
        scenario = ?scenario,
        no_audio,
        "Starting desktop display mode"
    );

    // Run in blocking context (egui is not async)
    tokio::task::spawn_blocking(move || run_ui_blocking(scenario, no_audio, &data_service))
        .await
        .map_err(|e| AppError::TaskPanic(e.to_string()))?
}

#[cfg(feature = "ui")]
fn run_ui_blocking(
    scenario: Option<String>,
    _no_audio: bool,
    data_service: &Arc<crate::data_service::DataService>,
) -> Result<()> {
    use petal_tongue_core::{InstanceId, RenderingCapabilities};
    use petal_tongue_ui::PetalTongueApp;

    // Create instance
    let instance_id = InstanceId::new();
    tracing::info!(
        "🌸 Starting {} UI instance: {}",
        PRIMAL_NAME,
        instance_id.as_str()
    );

    // Convert scenario to PathBuf
    let scenario_path = scenario_to_path(scenario);

    // Detect rendering capabilities
    let capabilities = RenderingCapabilities::detect();

    // Setup eframe options
    let options = petal_tongue_ui::eframe::NativeOptions {
        viewport: petal_tongue_ui::egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title(window_title())
            .with_visible(true)
            .with_active(true),
        ..Default::default()
    };

    // Create and run app
    // IMPORTANT: We pass the shared graph from DataService directly
    // This ensures the display uses the SAME data as all other modes
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
    .map_err(|e| AppError::Eframe(e.to_string()))
}

#[cfg(not(feature = "ui"))]
pub async fn run(
    _scenario: Option<String>,
    _no_audio: bool,
    _data_service: std::sync::Arc<crate::data_service::DataService>,
) -> Result<()> {
    Err(AppError::UiNotAvailable)
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    #[cfg(not(feature = "ui"))]
    use std::sync::Arc;

    use petal_tongue_core::constants::PRIMAL_NAME;

    use super::*;

    #[test]
    fn test_scenario_to_path_none() {
        assert!(scenario_to_path(None).is_none());
    }

    #[test]
    fn test_scenario_to_path_some() {
        let path = scenario_to_path(Some("scenario.json".to_string()));
        assert!(path.is_some());
        assert_eq!(path.as_ref().unwrap().as_os_str(), "scenario.json");
    }

    #[test]
    fn test_scenario_to_path_empty_string() {
        let path = scenario_to_path(Some(String::new()));
        assert!(path.is_some());
        assert_eq!(path.as_ref().unwrap().as_os_str(), "");
    }

    #[test]
    fn test_window_title_contains_primal_name() {
        let title = window_title();
        assert!(
            title.contains("petalTongue"),
            "Title should contain primal name: {title}"
        );
        assert!(
            title.contains("Universal Representation System"),
            "Title should contain subtitle: {title}"
        );
    }

    #[test]
    fn test_window_title_starts_with_flower_emoji() {
        let title = window_title();
        assert!(
            title.starts_with("🌸"),
            "Title should start with flower emoji: {title}"
        );
    }

    #[test]
    fn test_scenario_to_path_with_path_separators() {
        let path = scenario_to_path(Some("/tmp/scenarios/demo.json".to_string()));
        assert!(path.is_some());
        let p = path.unwrap();
        assert!(p.to_string_lossy().contains("demo.json"));
    }

    #[test]
    fn test_scenario_to_path_with_relative_path() {
        let path = scenario_to_path(Some("./relative/path.json".to_string()));
        assert!(path.is_some());
        assert_eq!(path.as_ref().unwrap().as_os_str(), "./relative/path.json");
    }

    #[test]
    fn test_scenario_to_path_parent_dir() {
        let path = scenario_to_path(Some("..".to_string()));
        assert!(path.is_some());
        assert_eq!(path.as_ref().unwrap().as_os_str(), "..");
    }

    #[test]
    fn test_scenario_to_path_current_dir() {
        let path = scenario_to_path(Some(".".to_string()));
        assert!(path.is_some());
        assert_eq!(path.as_ref().unwrap().as_os_str(), ".");
    }

    #[test]
    fn test_scenario_to_path_whitespace() {
        let path = scenario_to_path(Some("  path.json  ".to_string()));
        assert!(path.is_some());
        assert_eq!(path.as_ref().unwrap().as_os_str(), "  path.json  ");
    }

    #[test]
    fn test_window_title_contains_representation() {
        let title = window_title();
        assert!(title.contains("Representation"));
    }

    #[test]
    fn test_window_title_exact_format() {
        let title = window_title();
        assert_eq!(
            title, "🌸 petalTongue - Universal Representation System",
            "Window title must match exact format"
        );
    }

    #[test]
    fn test_window_title_structure() {
        let title = window_title();
        let parts: Vec<&str> = title.split(" - ").collect();
        assert_eq!(parts.len(), 2, "Title should have format 'prefix - suffix'");
        assert!(parts[0].starts_with("🌸"));
        assert!(parts[0].contains("petalTongue"));
        assert_eq!(parts[1], "Universal Representation System");
    }

    /// Exercises the Eframe error path used in run_ui_blocking when eframe::run_native fails.
    /// Same map_err pattern as run_ui_blocking line 97.
    #[test]
    fn test_eframe_error_creation() {
        let err = AppError::Eframe("display failed".to_string());
        assert!(matches!(err, AppError::Eframe(_)));
        let msg = err.to_string();
        assert!(msg.contains("eframe error"));
        assert!(msg.contains("display failed"));
    }

    #[test]
    fn test_scenario_to_path_unicode() {
        let path = scenario_to_path(Some("scenario_日本語.json".to_string()));
        assert!(path.is_some());
        assert!(path.as_ref().unwrap().to_string_lossy().contains("日本語"));
    }

    #[test]
    fn test_scenario_to_path_special_chars() {
        let path = scenario_to_path(Some("path with spaces.json".to_string()));
        assert!(path.is_some());
        assert_eq!(path.as_ref().unwrap().as_os_str(), "path with spaces.json");
    }

    #[tokio::test]
    #[cfg(feature = "ui")]
    async fn test_ui_mode_signature() {
        // Can't test actual UI in headless environment
        // This just verifies the function signature compiles
    }

    /// Exercises the TaskPanic error path used in run() when spawn_blocking panics.
    /// Same map_err pattern as run() line 42.
    #[tokio::test]
    #[cfg(feature = "ui")]
    async fn test_run_task_panic_error_path() {
        let result = tokio::task::spawn_blocking(|| panic!("test panic for coverage"))
            .await
            .map_err(|e| AppError::TaskPanic(e.to_string()));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("panicked"));
        assert!(err.to_string().contains("test panic for coverage"));
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
        // Verify it's a proper error chain (has at least one error in chain)
        assert!(err.chain().next().is_some());
    }

    #[test]
    fn test_window_title_uses_primal_const() {
        let title = window_title();
        assert_eq!(
            title,
            format!("🌸 {PRIMAL_NAME} - Universal Representation System")
        );
    }

    #[tokio::test]
    #[cfg(not(feature = "ui"))]
    async fn test_run_not_ui_returns_ui_not_available_variant() {
        let data_service = Arc::new(crate::data_service::DataService::new());
        let result = run(None, false, data_service).await;
        assert!(matches!(result, Err(AppError::UiNotAvailable)));
    }

    #[test]
    fn test_app_error_ui_not_available_display() {
        let err = AppError::UiNotAvailable;
        let msg = err.to_string();
        assert!(msg.contains("not available"));
        assert!(msg.contains("tui"));
        assert!(msg.contains("web"));
    }

    #[test]
    fn test_scenario_to_path_with_null_bytes() {
        let path = scenario_to_path(Some("path\x00null.json".to_string()));
        assert!(path.is_some());
        assert!(
            path.as_ref()
                .unwrap()
                .as_os_str()
                .to_string_lossy()
                .contains("path")
        );
    }

    #[test]
    fn test_scenario_to_path_very_long_path() {
        let long = "a".repeat(4096);
        let path = scenario_to_path(Some(long.clone()));
        assert!(path.is_some());
        assert_eq!(path.as_ref().unwrap().as_os_str(), long.as_str());
    }

    #[test]
    fn test_eframe_error_debug_display() {
        let err = AppError::Eframe("test".to_string());
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("Eframe"));
    }

    #[test]
    fn test_task_panic_error_display() {
        let err = AppError::TaskPanic("worker died".to_string());
        assert!(err.to_string().contains("Task panicked"));
        assert!(err.to_string().contains("worker died"));
    }

    #[test]
    fn test_app_error_other_display() {
        let err = AppError::Other("config load failed".to_string());
        assert_eq!(err.to_string(), "config load failed");
    }
}
