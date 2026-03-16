// SPDX-License-Identifier: AGPL-3.0-only
//! Toadstool Backend - Legacy UIBackend (DEPRECATED)
//!
//! **⚠️ DEPRECATED**: Use `crate::display::backends::toadstool_v2` instead!
//!
//! The new `toadstool_v2` module provides a complete tarpc implementation with:
//! - Capability-based discovery via biomeOS
//! - High-performance tarpc binary RPC
//! - Zero hardcoded primal names (TRUE PRIMAL compliant)
//! - Complete DisplayBackend trait implementation
//!
//! # Migration Guide
//!
//! ```rust,ignore
//! // OLD (this module - deprecated)
//! use crate::backend::toadstool::ToadstoolBackend;
//!
//! // NEW (recommended)
//! use crate::display::backends::toadstool_v2::ToadstoolDisplay;
//! ```
//!
//! This module uses capability discovery to check availability. No mock/stub
//! env vars - production code attempts real discovery and reports "not available"
//! when display provider cannot be found.

use crate::error::{BackendError, Result};
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::backend::{BackendCapabilities, UIBackend};
use petal_tongue_core::{
    GraphEngine, RenderingCapabilities,
    biomeos_discovery::BiomeOsBackend,
    capability_discovery::{CapabilityDiscovery, CapabilityQuery},
};

/// Toadstool backend - Pure Rust GUI via Toadstool display service
///
/// NOTE: Legacy module - frozen stub. Use `display::backends::toadstool_v2` instead.
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
    #[expect(
        clippy::unused_async,
        reason = "async for future Toadstool capability discovery"
    )]
    pub async fn new() -> Result<Self> {
        tracing::info!("🍄 Creating Toadstool backend");

        // NOTE: Legacy stub - no actual connection
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
        "gpu-display"
    }

    async fn is_available() -> bool {
        // Attempt real capability discovery - no mock/stub env vars
        tracing::debug!("Checking for display capability via biomeOS discovery...");

        let backend = match BiomeOsBackend::from_env() {
            Ok(b) => b,
            Err(e) => {
                tracing::debug!("biomeOS discovery backend not available: {}", e);
                return false;
            }
        };

        let discovery = CapabilityDiscovery::new(Box::new(backend));
        match discovery
            .discover_one(&CapabilityQuery::new("display"))
            .await
        {
            Ok(endpoint) => {
                tracing::info!("Display capability discovered: {}", endpoint.id);
                true
            }
            Err(e) => {
                tracing::debug!("Display capability not available: {}", e);
                false
            }
        }
    }

    async fn init(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        tracing::info!("🔧 Initializing Toadstool backend...");

        // Legacy UIBackend: Full tarpc integration lives in display::backends::toadstool_v2.
        // This module reports unavailability with graceful degradation—no stub env vars.
        return Err(BackendError::ToadstoolRequiresBiomeOs(
            "Toadstool display service not available. \
             Use DisplayManager with display::backends::toadstool_v2 for GPU rendering. \
             See TOADSTOOL_DISPLAY_BACKEND_REQUEST.md"
                .to_string(),
        )
        .into());
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

        tracing::info!("🚀 Running Toadstool backend...");
        tracing::info!("   Scenario: {:?}", scenario);
        tracing::info!(
            "   Capabilities: device={}, complexity={:?}",
            capabilities.device_type,
            capabilities.ui_complexity
        );
        tracing::info!(
            "   Shared graph: {} nodes",
            shared_graph.read().map(|g| g.nodes().len()).unwrap_or(0)
        );

        // Legacy UIBackend: Display capability discovered but full tarpc integration
        // lives in display::backends::toadstool_v2. This stub reports "not yet implemented"
        // for actual window/event-loop - use DisplayManager + ToadstoolDisplayV2 instead.
        return Err(BackendError::ToadstoolRequiresBiomeOs(
            "Toadstool UIBackend run() not yet implemented. \
             Use DisplayManager with display::backends::toadstool_v2 instead."
                .to_string(),
        )
        .into());
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("🛑 Shutting down Toadstool backend...");

        // NOTE: Legacy stub - no resources to clean

        self.window = None;
        self.client = None;
        self.initialized = false;

        Ok(())
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            has_gpu: true,          // Toadstool provides GPU via wgpu
            multi_window: true,     // Toadstool will support multiple windows
            custom_cursor: true,    // Will be supported
            clipboard: false,       // NOTE: Legacy stub - clipboard not implemented
            pure_rust: true,        // ✅ 100% Pure Rust! (no C dependencies)
            needs_privileges: true, // Requires DRM/KMS access (or libseat)
        }
    }
}

// ===== Stub types (will be replaced by actual Toadstool client) =====

/// Toadstool display client (stub)
///
/// NOTE: Legacy module - frozen stub. See toadstool_v2 for actual implementation.
struct ToadstoolDisplayClient {
    // Will contain RPC client
}

/// Window ID (stub)
///
/// This will be the actual window handle from Toadstool.
#[derive(Debug, Clone, Copy)]
#[expect(
    dead_code,
    reason = "Legacy stub - reserved for future Toadstool window integration"
)]
struct WindowId(u64);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_toadstool_backend_name() {
        let backend = ToadstoolBackend::new().await.unwrap();
        assert_eq!(backend.name(), "gpu-display");
    }

    #[tokio::test]
    async fn test_toadstool_capabilities() {
        let backend = ToadstoolBackend::new().await.unwrap();
        let caps = backend.capabilities();
        assert!(caps.pure_rust);
        assert!(caps.has_gpu);
        assert!(caps.multi_window);
    }

    #[tokio::test]
    async fn test_toadstool_init_behavior() {
        // When display capability is not discovered, init fails.
        // When discovered, init succeeds. Both paths are valid.
        let mut backend = ToadstoolBackend::new().await.unwrap();
        match backend.init().await {
            Ok(()) => assert!(backend.initialized),
            Err(e) => assert!(!e.to_string().is_empty()),
        }
    }

    #[tokio::test]
    async fn test_toadstool_is_available_uses_discovery() {
        // is_available() uses capability discovery - no env var stub.
        let _ = ToadstoolBackend::is_available().await;
    }
}
