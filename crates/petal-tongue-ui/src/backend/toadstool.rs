//! Toadstool Backend - Future Pure Rust GUI
//!
//! This backend connects to Toadstool display service for 100% Pure Rust GUI.
//! Currently a stub implementation, ready for Toadstool team integration.
//!
//! # Status
//!
//! - 🔬 **STUB IMPLEMENTATION** (ready for integration)
//! - ✅ Will be 100% Pure Rust (no C dependencies!)
//! - ⏱️ Timeline: 4-6 weeks (Toadstool team implementation)
//! - 🍄 Requires Toadstool display service running
//!
//! # Architecture
//!
//! ```text
//! petalTongue
//!     ↓
//! ToadstoolBackend (this module)
//!     ↓ RPC (tarpc)
//! Toadstool Display Service
//!     ↓
//! DRM/KMS (drm-rs) + evdev (evdev-rs) + wgpu
//!     ↓
//! Hardware (100% Pure Rust!)
//! ```
//!
//! # When Ready
//!
//! Once Toadstool team implements display backend:
//! 1. Replace stub methods with actual RPC calls
//! 2. Implement event streaming
//! 3. Add framebuffer management
//! 4. Enable feature flag in CI
//! 5. Update documentation
//!
//! See `TOADSTOOL_DISPLAY_BACKEND_REQUEST.md` for full specification.

use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::backend::{BackendCapabilities, UIBackend};
use petal_tongue_core::{GraphEngine, RenderingCapabilities};

/// Toadstool backend - Pure Rust GUI via Toadstool display service
///
/// This is currently a stub implementation. It will connect to Toadstool's
/// display service once that's implemented.
///
/// # TODO (Toadstool Team)
///
/// - [ ] Implement ToadstoolDisplayClient (RPC client)
/// - [ ] Add window creation via DRM/KMS
/// - [ ] Add input handling via evdev
/// - [ ] Add framebuffer management
/// - [ ] Add multi-window support
/// - [ ] Add VSync support
/// - [ ] Performance optimization
///
/// See `TOADSTOOL_DISPLAY_BACKEND_REQUEST.md` for complete specification.
pub struct ToadstoolBackend {
    /// Toadstool display client (stub for now)
    client: Option<ToadstoolDisplayClient>,
    
    /// Current window ID (if created)
    window: Option<WindowId>,
    
    /// Whether backend is initialized
    initialized: bool,
}

impl ToadstoolBackend {
    /// Create a new Toadstool backend
    ///
    /// This attempts to connect to Toadstool display service.
    /// Returns error if Toadstool is not running or not available.
    pub async fn new() -> Result<Self> {
        tracing::info!("🍄 Creating Toadstool backend");
        
        // TODO: Connect to Toadstool display service
        // For now, return stub
        Ok(Self {
            client: None,
            window: None,
            initialized: false,
        })
    }
}

#[async_trait]
impl UIBackend for ToadstoolBackend {
    fn name(&self) -> &'static str {
        "toadstool"
    }

    async fn is_available() -> bool {
        // TODO: Actually check if Toadstool display service is running
        // For now, check for env var to enable stub testing
        if std::env::var("PETALTONGUE_TOADSTOOL_STUB").is_ok() {
            tracing::info!("🍄 Toadstool stub mode enabled");
            return true;
        }
        
        // Try to connect to Toadstool
        // TODO: Implement actual connection check
        tracing::debug!("Checking for Toadstool display service...");
        
        // For now, not available (requires Toadstool implementation)
        false
    }

    async fn init(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        tracing::info!("🔧 Initializing Toadstool backend...");

        // TODO: Connect to Toadstool display service
        // let client = ToadstoolDisplayClient::connect("localhost:8084").await?;
        // self.client = Some(client);

        // For now, stub implementation
        if std::env::var("PETALTONGUE_TOADSTOOL_STUB").is_ok() {
            tracing::warn!("⚠️  Using Toadstool STUB implementation");
            tracing::warn!("   Set PETALTONGUE_TOADSTOOL_STUB to enable");
            tracing::warn!("   This will not actually display anything!");
            // Continue with stub
        } else {
            anyhow::bail!(
                "Toadstool display service not available. \
                 Is Toadstool running? \
                 See TOADSTOOL_DISPLAY_BACKEND_REQUEST.md"
            );
        }

        self.initialized = true;
        tracing::info!("✅ Toadstool backend initialized (stub mode)");

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

        tracing::info!("🚀 Running Toadstool backend (stub)...");
        tracing::info!("   Scenario: {:?}", scenario);
        tracing::info!("   Capabilities: device={}, complexity={:?}",
            capabilities.device_type, capabilities.ui_complexity);
        tracing::info!("   Using shared graph from DataService (TRUE PRIMAL!)");

        // TODO: Implement actual Toadstool integration
        // 1. Create window via Toadstool RPC
        // 2. Set up input event stream
        // 3. Create PetalTongueApp with shared_graph (like eframe backend)
        // 4. Render loop:
        //    - Render egui to pixels
        //    - Send pixels to Toadstool framebuffer
        //    - Poll input events
        //    - Handle events in egui
        // 5. Clean up on exit

        // For now, stub implementation
        if std::env::var("PETALTONGUE_TOADSTOOL_STUB").is_ok() {
            tracing::warn!("🍄 Toadstool STUB: Would create window and run UI here");
            tracing::warn!("   Would use shared_graph ({} nodes) from DataService",
                shared_graph.read().map(|g| g.nodes().len()).unwrap_or(0));
            tracing::warn!("   Actual implementation pending Toadstool display service");
            tracing::warn!("   See TOADSTOOL_DISPLAY_BACKEND_REQUEST.md for spec");
            
            // Simulate running for a bit
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            
            tracing::info!("✅ Toadstool backend finished (stub)");
            return Ok(());
        }

        anyhow::bail!("Toadstool backend not yet implemented")
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("🛑 Shutting down Toadstool backend...");
        
        // TODO: Clean up Toadstool resources
        // - Destroy window
        // - Close RPC connection
        // - Release input devices
        
        self.window = None;
        self.client = None;
        self.initialized = false;
        
        Ok(())
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            has_gpu: true,            // Toadstool provides GPU via wgpu
            multi_window: true,        // Toadstool will support multiple windows
            custom_cursor: true,       // Will be supported
            clipboard: false,          // TODO: Add clipboard support
            pure_rust: true,           // ✅ 100% Pure Rust! (no C dependencies)
            needs_privileges: true,    // Requires DRM/KMS access (or libseat)
        }
    }
}

// ===== Stub types (will be replaced by actual Toadstool client) =====

/// Toadstool display client (stub)
///
/// This will be replaced by actual RPC client once Toadstool implements
/// display backend.
///
/// # TODO (Toadstool Team)
///
/// ```rust,ignore
/// use tarpc::client;
///
/// #[tarpc::service]
/// pub trait ToadstoolDisplay {
///     async fn create_window(width: u32, height: u32) -> WindowId;
///     async fn present(window: WindowId, pixels: Vec<u8>) -> ();
///     async fn poll_events() -> Vec<InputEvent>;
///     async fn destroy_window(window: WindowId) -> ();
/// }
/// ```
struct ToadstoolDisplayClient {
    // Will contain RPC client
}

/// Window ID (stub)
///
/// This will be the actual window handle from Toadstool.
#[derive(Debug, Clone, Copy)]
struct WindowId(u64);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_toadstool_backend_name() {
        std::env::set_var("PETALTONGUE_TOADSTOOL_STUB", "1");
        let backend = ToadstoolBackend::new().await.unwrap();
        assert_eq!(backend.name(), "toadstool");
    }

    #[tokio::test]
    async fn test_toadstool_capabilities() {
        std::env::set_var("PETALTONGUE_TOADSTOOL_STUB", "1");
        let backend = ToadstoolBackend::new().await.unwrap();
        let caps = backend.capabilities();
        
        assert!(caps.pure_rust); // ✅ Key feature!
        assert!(caps.has_gpu);
        assert!(caps.multi_window);
    }

    #[tokio::test]
    async fn test_toadstool_stub_init() {
        std::env::set_var("PETALTONGUE_TOADSTOOL_STUB", "1");
        let mut backend = ToadstoolBackend::new().await.unwrap();
        let result = backend.init().await;
        assert!(result.is_ok());
        assert!(backend.initialized);
    }

    #[tokio::test]
    async fn test_toadstool_not_available_without_stub() {
        std::env::remove_var("PETALTONGUE_TOADSTOOL_STUB");
        let available = ToadstoolBackend::is_available().await;
        // Should be false until Toadstool implements display service
        assert!(!available);
    }
}

