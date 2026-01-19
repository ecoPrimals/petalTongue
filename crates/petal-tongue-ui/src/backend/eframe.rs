//! eframe Backend - Current GUI using egui/eframe
//!
//! This backend wraps the existing eframe-based GUI implementation.
//! It provides compatibility with the current UI while enabling future
//! migration to alternative backends.
//!
//! # Status
//!
//! - ✅ Production-ready
//! - ⚠️ Has C dependencies (wayland-sys, x11rb)
//! - ✅ Cross-platform (Linux, Windows, macOS)
//! - ✅ Full feature support
//!
//! # C Dependencies
//!
//! This backend depends on:
//! - `wayland-sys` (Wayland display server protocol)
//! - `x11rb` (X11 display server protocol)
//! - Platform-specific window management APIs
//!
//! These dependencies will be eliminated in ecoBlossom via Toadstool backend.

use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::backend::{BackendCapabilities, UIBackend};
use crate::PetalTongueApp;
use petal_tongue_core::{GraphEngine, RenderingCapabilities};

/// eframe/egui backend implementation
///
/// This wraps the existing PetalTongueApp and eframe integration,
/// providing the UIBackend trait interface.
pub struct EguiBackend {
    /// Whether backend has been initialized
    initialized: bool,
}

impl EguiBackend {
    /// Create a new eframe backend
    pub fn new() -> Self {
        tracing::info!("🎨 Creating eframe backend");
        Self {
            initialized: false,
        }
    }
}

impl Default for EguiBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UIBackend for EguiBackend {
    fn name(&self) -> &'static str {
        "eframe"
    }

    async fn is_available() -> bool {
        // eframe is always available (it's our fallback)
        // Check if we have a display server
        let has_display = std::env::var("DISPLAY").is_ok()
            || std::env::var("WAYLAND_DISPLAY").is_ok()
            || cfg!(target_os = "windows")
            || cfg!(target_os = "macos");

        if !has_display {
            tracing::warn!("⚠️  No display server detected (no DISPLAY or WAYLAND_DISPLAY)");
            tracing::warn!("   eframe backend may not work without display server");
        }

        true // eframe is always "available" (it will error at runtime if no display)
    }

    async fn init(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        tracing::info!("🔧 Initializing eframe backend...");

        // Check display availability
        let has_display = std::env::var("DISPLAY").is_ok()
            || std::env::var("WAYLAND_DISPLAY").is_ok()
            || cfg!(target_os = "windows")
            || cfg!(target_os = "macos");

        if !has_display {
            tracing::warn!("⚠️  No display server detected");
            tracing::info!("   Alternatives: Toadstool backend, TUI mode, web mode");
        }

        self.initialized = true;
        tracing::info!("✅ eframe backend initialized");

        Ok(())
    }

    async fn run(
        &mut self,
        scenario: Option<PathBuf>,
        capabilities: RenderingCapabilities,
        shared_graph: Arc<RwLock<GraphEngine>>,
    ) -> Result<()> {
        if !self.initialized {
            self.init().await?;
        }

        tracing::info!("🚀 Running eframe backend...");
        tracing::info!("   Scenario: {:?}", scenario);
        tracing::info!("   Capabilities: device={}, complexity={:?}",
            capabilities.device_type, capabilities.ui_complexity);
        tracing::info!("   Using shared graph from DataService (TRUE PRIMAL!)");

        // Create native options
        let native_options = crate::eframe::NativeOptions {
            viewport: crate::egui::ViewportBuilder::default()
                .with_inner_size([1920.0, 1080.0])
                .with_min_inner_size([800.0, 600.0])
                .with_icon(load_icon()),
            ..Default::default()
        };

        // Run eframe (this blocks until window is closed)
        tracing::info!("🪟 Creating window...");
        
        crate::eframe::run_native(
            "petalTongue",
            native_options,
            Box::new(move |cc| {
                tracing::info!("✅ eframe context created");
                
                // Create app with shared graph from DataService
                // TRUE PRIMAL: Single source of truth across ALL UI modes!
                let app = PetalTongueApp::new_with_shared_graph(
                    cc,
                    scenario,
                    capabilities,
                    shared_graph,
                );
                
                Ok(Box::new(app))
            }),
        )
        .map_err(|e| anyhow::anyhow!("eframe::run_native failed: {}", e))?;

        tracing::info!("✅ eframe backend finished");
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("🛑 Shutting down eframe backend...");
        self.initialized = false;
        Ok(())
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            has_gpu: true,            // eframe uses GPU via egui
            multi_window: false,       // Current implementation is single-window
            custom_cursor: true,       // Supported by egui
            clipboard: true,           // Supported by egui
            pure_rust: false,          // ❌ Has C dependencies (wayland-sys, x11rb)
            needs_privileges: false,   // No special permissions needed
        }
    }
}

/// Load application icon
///
/// Returns the petalTongue icon for the window title bar.
fn load_icon() -> Arc<crate::egui::IconData> {
    // For now, use a placeholder
    // TODO: Load actual petalTongue icon
    let (icon_rgba, icon_width, icon_height) = {
        // 32x32 pink flower icon (placeholder)
        let size = 32;
        let mut rgba = vec![0u8; size * size * 4];
        
        // Simple pink color
        for pixel in rgba.chunks_exact_mut(4) {
            pixel[0] = 255; // R
            pixel[1] = 182; // G
            pixel[2] = 193; // B (light pink)
            pixel[3] = 255; // A
        }
        
        (rgba, size as u32, size as u32)
    };

    Arc::new(crate::egui::IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_eframe_backend_creation() {
        let backend = EguiBackend::new();
        assert_eq!(backend.name(), "eframe");
        assert!(!backend.initialized);
    }

    #[tokio::test]
    async fn test_eframe_backend_init() {
        let mut backend = EguiBackend::new();
        let result = backend.init().await;
        assert!(result.is_ok());
        assert!(backend.initialized);
    }

    #[tokio::test]
    async fn test_eframe_capabilities() {
        let backend = EguiBackend::new();
        let caps = backend.capabilities();
        
        assert!(caps.has_gpu);
        assert!(caps.clipboard);
        assert!(!caps.pure_rust); // Has C dependencies
    }

    #[tokio::test]
    async fn test_eframe_is_available() {
        // eframe should always report as available
        assert!(EguiBackend::is_available().await);
    }
}

