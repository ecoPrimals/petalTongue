// SPDX-License-Identifier: AGPL-3.0-or-later
//! eframe Backend - Current display using egui/eframe
//!
//! This backend wraps the existing eframe-based display implementation.
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

use crate::error::{BackendError, Result};
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::PetalTongueApp;
use crate::backend::{BackendCapabilities, UIBackend};
use petal_tongue_core::{GraphEngine, RenderingCapabilities};

/// eframe/egui backend implementation
///
/// This wraps the existing `PetalTongueApp` and eframe integration,
/// providing the `UIBackend` trait interface.
pub struct EguiBackend {
    /// Whether backend has been initialized
    initialized: bool,
}

impl EguiBackend {
    /// Create a new eframe backend
    pub fn new() -> Self {
        tracing::info!("🎨 Creating eframe backend");
        Self { initialized: false }
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
        tracing::info!(
            "   Capabilities: device={}, complexity={:?}",
            capabilities.device_type,
            capabilities.ui_complexity
        );
        tracing::info!("   Using shared graph from DataService (TRUE PRIMAL!)");

        // Create native options (env: PETALTONGUE_WINDOW_WIDTH, PETALTONGUE_WINDOW_HEIGHT)
        let (w, h) = petal_tongue_core::constants::default_window_size();
        let native_options = crate::eframe::NativeOptions {
            viewport: crate::egui::ViewportBuilder::default()
                .with_inner_size([w as f32, h as f32])
                .with_min_inner_size([800.0, 600.0])
                .with_icon(load_icon()),
            ..Default::default()
        };

        // Run eframe (this blocks until window is closed)
        tracing::info!("🪟 Creating window...");

        crate::eframe::run_native(
            petal_tongue_core::constants::PRIMAL_NAME,
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
                )?;

                Ok(Box::new(app))
            }),
        )
        .map_err(|e| BackendError::EframeRunFailed(e.to_string()))?;

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
            has_gpu: true,           // eframe uses GPU via egui
            multi_window: false,     // Current implementation is single-window
            custom_cursor: true,     // Supported by egui
            clipboard: true,         // Supported by egui
            pure_rust: false,        // ❌ Has C dependencies (wayland-sys, x11rb)
            needs_privileges: false, // No special permissions needed
        }
    }
}

/// Load application icon
///
/// Returns the petalTongue icon for the window title bar.
/// Programmatically generates a 32x32 flower/petal design (no external file).
fn load_icon() -> Arc<crate::egui::IconData> {
    let size = 32;
    let mut rgba = vec![0u8; size * size * 4];

    let cx = size as f32 / 2.0 - 0.5;
    let cy = size as f32 / 2.0 - 0.5;

    // Petal colors (light pink petals, rose center)
    let petal: (u8, u8, u8) = (255, 182, 193); // Light pink
    let center: (u8, u8, u8) = (219, 112, 147); // Pale violet red (flower center)
    let outline: (u8, u8, u8) = (199, 21, 133); // Medium violet red (accent)

    for y in 0..size {
        for x in 0..size {
            let fx = x as f32;
            let fy = y as f32;
            let dx = fx - cx;
            let dy = fy - cy;
            let dist = dx.hypot(dy);
            let angle = dy.atan2(dx);

            let idx = (y * size + x) * 4;

            // Center circle (radius ~4)
            if dist < 4.0 {
                rgba[idx] = center.0;
                rgba[idx + 1] = center.1;
                rgba[idx + 2] = center.2;
                rgba[idx + 3] = 255;
            }
            // 5 petals - teardrop shapes at 72° intervals
            else if dist < 14.0 {
                let mut in_petal = false;
                for i in 0..5 {
                    let petal_angle = (i as f32) * std::f32::consts::TAU / 5.0;
                    let angle_diff = (angle - petal_angle).abs();
                    let angle_diff = angle_diff.min(std::f32::consts::TAU - angle_diff);
                    // Petal: teardrop shape, wider near center
                    let r = 12.0 * 0.4f32.mul_add(-angle_diff, 1.0);
                    if dist < r {
                        in_petal = true;
                        break;
                    }
                }
                if in_petal {
                    rgba[idx] = petal.0;
                    rgba[idx + 1] = petal.1;
                    rgba[idx + 2] = petal.2;
                    rgba[idx + 3] = 255;
                } else if dist < 15.0 {
                    rgba[idx] = outline.0;
                    rgba[idx + 1] = outline.1;
                    rgba[idx + 2] = outline.2;
                    rgba[idx + 3] = 200;
                } else {
                    rgba[idx + 3] = 0;
                }
            } else {
                rgba[idx + 3] = 0;
            }
        }
    }

    Arc::new(crate::egui::IconData {
        rgba,
        width: size as u32,
        height: size as u32,
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
