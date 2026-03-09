// SPDX-License-Identifier: AGPL-3.0-only
//! Toadstool Backend - Legacy Stub (DEPRECATED)
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
//! ```rust
//! // OLD (this module - deprecated)
//! use crate::backend::toadstool::ToadstoolBackend;
//!
//! // NEW (recommended)
//! use crate::display::backends::toadstool_v2::ToadstoolDisplay;
//! ```
//!
//! This module is retained for backward compatibility but will be removed
//! in a future version.
//!
//! See `crates/petal-tongue-ui/src/display/backends/toadstool_v2.rs` for the
//! complete implementation.

use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::backend::{BackendCapabilities, UIBackend};
use petal_tongue_core::{GraphEngine, RenderingCapabilities};

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
        "toadstool"
    }

    async fn is_available() -> bool {
        // NOTE: Legacy stub - check env var for stub testing only
        if std::env::var("PETALTONGUE_TOADSTOOL_STUB").is_ok() {
            tracing::info!("🍄 Toadstool stub mode enabled");
            return true;
        }

        // NOTE: Legacy stub - no actual connection check
        tracing::debug!("Checking for Toadstool display service...");

        // For now, not available (requires Toadstool implementation)
        false
    }

    async fn init(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        tracing::info!("🔧 Initializing Toadstool backend...");

        // NOTE: Legacy stub - no actual connection
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
        tracing::info!(
            "   Capabilities: device={}, complexity={:?}",
            capabilities.device_type,
            capabilities.ui_complexity
        );
        tracing::info!("   Using shared graph from DataService (TRUE PRIMAL!)");

        // NOTE: Legacy stub - no actual Toadstool integration
        if std::env::var("PETALTONGUE_TOADSTOOL_STUB").is_ok() {
            tracing::warn!("🍄 Toadstool STUB: Would create window and run UI here");
            tracing::warn!(
                "   Would use shared_graph ({} nodes) from DataService",
                shared_graph.read().map(|g| g.nodes().len()).unwrap_or(0)
            );
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
#[allow(dead_code)]
struct WindowId(u64);

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::test_fixtures::env_test_helpers;

    #[tokio::test]
    async fn test_toadstool_backend_name() {
        env_test_helpers::with_env_var_async("PETALTONGUE_TOADSTOOL_STUB", "1", || async {
            let backend = ToadstoolBackend::new().await.unwrap();
            assert_eq!(backend.name(), "toadstool");
        })
        .await;
    }

    #[tokio::test]
    async fn test_toadstool_capabilities() {
        env_test_helpers::with_env_var_async("PETALTONGUE_TOADSTOOL_STUB", "1", || async {
            let backend = ToadstoolBackend::new().await.unwrap();
            let caps = backend.capabilities();
            assert!(caps.pure_rust);
            assert!(caps.has_gpu);
            assert!(caps.multi_window);
        })
        .await;
    }

    #[tokio::test]
    async fn test_toadstool_stub_init() {
        env_test_helpers::with_env_var_async("PETALTONGUE_TOADSTOOL_STUB", "1", || async {
            let mut backend = ToadstoolBackend::new().await.unwrap();
            let result = backend.init().await;
            assert!(result.is_ok());
            assert!(backend.initialized);
        })
        .await;
    }

    #[tokio::test]
    async fn test_toadstool_not_available_without_stub() {
        env_test_helpers::with_env_var_removed_async("PETALTONGUE_TOADSTOOL_STUB", || async {
            let available = ToadstoolBackend::is_available().await;
            assert!(!available);
        })
        .await;
    }
}
