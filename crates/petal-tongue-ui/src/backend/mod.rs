// SPDX-License-Identifier: AGPL-3.0-or-later
//! UI Backend Abstraction Layer
//!
//! This module provides a trait-based abstraction for different UI backends,
//! enabling petalTongue to support multiple rendering strategies without
//! code duplication.
//!
//! # Architecture
//!
//! ```text
//! petalTongue App
//!       ↓
//!   UIBackend trait (this module)
//!       ↓
//!   ┌───────────────┬──────────────────────────────┐
//!   ↓               ↓                              ↓
//! EguiBackend   display::backends              (Future backends)
//!   ↓               ↓
//! eframe        discovered display modules
//!   ↓               ↓
//! Wayland/X11   DRM/KMS (Pure Rust!)
//! (C deps)
//! ```
//!
//! # Backends
//!
//! - **`EguiBackend`**: Current backend using eframe/winit (has C dependencies)
//! - **Discovered display**: Pure Rust backends under `display::backends` (capability-discovered at runtime)
//!
//! # Feature Flags
//!
//! - `ui-auto`: Auto-detect best available backend (default)
//! - `ui-eframe`: Force eframe backend
//! - `discovered-display`: Force capability-discovered display backend (requires biomeOS)
//!
//! # Examples
//!
//! ```no_run
//! use petal_tongue_ui::backend::{BackendChoice, UIBackend, create_backend};
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Auto-detect best backend
//! let backend = create_backend(None).await?;
//!
//! // Force specific backend
//! let backend = create_backend(Some(BackendChoice::DiscoveredDisplay)).await?;
//! # Ok(())
//! # }
//! ```

// Backend implementations
#[cfg(feature = "ui-eframe")]
pub mod eframe;

use crate::error::{BackendError, Result};
use async_trait::async_trait;
use petal_tongue_core::{GraphEngine, RenderingCapabilities};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// UI Backend trait - abstraction over different rendering strategies
///
/// This trait defines the minimal interface that any UI backend must implement
/// to work with petalTongue. Backends can be eframe (current), discovered display (future),
/// or any other rendering strategy.
///
/// # Design Principles
///
/// - **Minimal**: Only essential methods, no backend-specific details
/// - **Async**: All operations are async for flexibility
/// - **Graceful**: Errors are recoverable, fallback is possible
/// - **Testable**: Easy to mock for testing
#[async_trait]
pub trait UIBackend: Send + Sync {
    /// Get backend name for logging/debugging
    fn name(&self) -> &'static str;

    /// Check if backend is available on this system
    async fn is_available() -> bool
    where
        Self: Sized;

    /// Initialize the backend
    ///
    /// This is called once at startup before any other methods.
    /// Backends should perform any necessary setup here.
    async fn init(&mut self) -> Result<()>;

    /// Run the UI event loop
    ///
    /// This is the main entry point for the UI. It should:
    /// 1. Create the window
    /// 2. Start the event loop
    /// 3. Render frames
    /// 4. Handle input
    /// 5. Run until window is closed
    ///
    /// # Arguments
    ///
    /// - `scenario`: Optional path to scenario file
    /// - `capabilities`: Detected rendering capabilities
    /// - `shared_graph`: Shared graph engine from `DataService` (TRUE PRIMAL: single source of truth!)
    async fn run(
        &mut self,
        scenario: Option<PathBuf>,
        capabilities: RenderingCapabilities,
        shared_graph: Arc<RwLock<GraphEngine>>,
    ) -> Result<()>;

    /// Shutdown the backend
    ///
    /// This is called when the UI is closing. Backends should clean up
    /// resources here.
    async fn shutdown(&mut self) -> Result<()>;

    /// Get backend capabilities
    ///
    /// Returns information about what this backend can do (e.g., GPU
    /// acceleration, multi-window support, etc.)
    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities::default()
    }
}

/// Backend capabilities - what features a backend supports
#[derive(Debug, Clone, Default)]
pub struct BackendCapabilities {
    /// Backend has GPU acceleration
    pub has_gpu: bool,

    /// Backend supports multiple windows
    pub multi_window: bool,

    /// Backend supports custom cursors
    pub custom_cursor: bool,

    /// Backend supports clipboard operations
    pub clipboard: bool,

    /// Backend is 100% Pure Rust (no C dependencies)
    pub pure_rust: bool,

    /// Backend requires elevated permissions (e.g., DRM access)
    pub needs_privileges: bool,
}

/// Backend choice for manual selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendChoice {
    /// Auto-detect best available backend
    Auto,

    /// Use eframe backend (current, has C deps)
    Eframe,

    /// Use capability-discovered display backend (Pure Rust!)
    DiscoveredDisplay,
}

impl std::str::FromStr for BackendChoice {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "eframe" | "egui" => Ok(Self::Eframe),
            "compute.provider" | "pure-rust" | "discovered" => Ok(Self::DiscoveredDisplay),
            _ => Err(()),
        }
    }
}

/// Create a UI backend based on choice and availability
///
/// # Errors
///
/// Returns an error if no backends are available, eframe init fails, or the display backend requires biomeOS and it is unavailable.
///
/// This is the main entry point for creating backends. It handles:
/// - Auto-detection of best available backend
/// - Fallback logic if preferred backend unavailable
/// - Feature flag checking
/// - Logging and diagnostics
///
/// # Arguments
///
/// - `choice`: Optional backend preference (None = auto-detect)
///
/// # Returns
///
/// A boxed backend ready to use, or error if no backends available
///
/// # Examples
///
/// ```no_run
/// use petal_tongue_ui::backend::{BackendChoice, create_backend};
///
/// # async fn example() -> anyhow::Result<()> {
/// // Auto-detect
/// let backend = create_backend(None).await?;
///
/// // Force capability-discovered display
/// let backend = create_backend(Some(BackendChoice::DiscoveredDisplay)).await?;
/// # Ok(())
/// # }
/// ```
pub async fn create_backend(choice: Option<BackendChoice>) -> Result<Box<dyn UIBackend>> {
    let choice = choice.unwrap_or(BackendChoice::Auto);

    tracing::info!("🎨 Creating UI backend (choice: {:?})", choice);

    match choice {
        BackendChoice::Auto => create_auto_backend().await,
        BackendChoice::Eframe => create_eframe_backend().await,
        BackendChoice::DiscoveredDisplay => create_discovered_display_backend(),
    }
}

/// Auto-detect best available backend
///
/// Priority order:
/// 1. eframe (current default)
/// 2. Discovered display backends are in `display::backends`
async fn create_auto_backend() -> Result<Box<dyn UIBackend>> {
    tracing::info!("🔍 Auto-detecting best UI backend...");

    // Use eframe (capability-discovered display backends are in display::backends)
    tracing::info!("📦 Using eframe backend (has C dependencies)");
    create_eframe_backend().await
}

/// Create eframe backend
async fn create_eframe_backend() -> Result<Box<dyn UIBackend>> {
    #[cfg(feature = "ui-eframe")]
    {
        use crate::backend::eframe::EguiBackend;
        let mut backend = EguiBackend::new();
        backend
            .init()
            .await
            .map_err(|e| crate::error::BackendError::EframeRunFailed(e.to_string()))?;
        Ok(Box::new(backend))
    }

    #[cfg(not(feature = "ui-eframe"))]
    {
        return Err(BackendError::EframeNotAvailable.into());
    }
}

/// Create capability-discovered display backend
///
/// The legacy UIBackend-style backend has been removed. Use
/// `display::backends` with biomeOS instead.
fn create_discovered_display_backend() -> Result<Box<dyn UIBackend>> {
    Err(BackendError::DisplayBackendNotAvailable.into())
}

/// Parse backend choice from environment variable
///
/// Checks `PETALTONGUE_UI_BACKEND` environment variable.
///
/// Valid values: "auto", "eframe", "egui", "compute.provider", "pure-rust", "discovered"
#[must_use]
pub fn backend_from_env() -> Option<BackendChoice> {
    std::env::var("PETALTONGUE_UI_BACKEND")
        .ok()
        .and_then(|s| s.parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_choice_parsing() {
        assert_eq!("auto".parse::<BackendChoice>(), Ok(BackendChoice::Auto));
        assert_eq!("eframe".parse::<BackendChoice>(), Ok(BackendChoice::Eframe));
        assert_eq!("egui".parse::<BackendChoice>(), Ok(BackendChoice::Eframe));
        assert_eq!(
            "compute.provider".parse::<BackendChoice>(),
            Ok(BackendChoice::DiscoveredDisplay)
        );
        assert_eq!(
            "pure-rust".parse::<BackendChoice>(),
            Ok(BackendChoice::DiscoveredDisplay)
        );
        assert!("invalid".parse::<BackendChoice>().is_err());
    }

    #[test]
    fn test_backend_capabilities_default() {
        let caps = BackendCapabilities::default();
        assert!(!caps.has_gpu);
        assert!(!caps.multi_window);
        assert!(!caps.pure_rust);
    }
}
