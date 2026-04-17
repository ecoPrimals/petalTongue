// SPDX-License-Identifier: AGPL-3.0-or-later
//! Display Manager
//!
//! Manages multiple display backends with capability-based discovery.
//!
//! TRUE PRIMAL Evolution:
//! - Discovers backends via capabilities (no hardcoded names)
//! - Automatic fallback on failure
//! - Priority-based selection

use crate::display::prompt::prompt_for_display_server;
use crate::display::traits::{BackendPriority, DisplayBackend, DisplayCapabilities};
#[cfg(feature = "discovered-display")]
use crate::display::{DiscoveredDisplayBackend, DiscoveredDisplayBackendV2};
use crate::display::{ExternalDisplay, FramebufferDisplay, SoftwareDisplay};
use crate::error::{DisplayError, Result};
use tracing::{info, warn};

/// Enum dispatch for display backends (replaces `Box<dyn DisplayBackend>`).
pub enum DisplayBackendImpl {
    #[cfg(feature = "discovered-display")]
    DiscoveredV2(DiscoveredDisplayBackendV2),
    #[cfg(feature = "discovered-display")]
    DiscoveredJsonRpc(DiscoveredDisplayBackend),
    Software(SoftwareDisplay),
    Framebuffer(FramebufferDisplay),
    External(ExternalDisplay),
}

impl DisplayBackend for DisplayBackendImpl {
    async fn init(&mut self) -> Result<()> {
        match self {
            #[cfg(feature = "discovered-display")]
            Self::DiscoveredV2(b) => b.init().await,
            #[cfg(feature = "discovered-display")]
            Self::DiscoveredJsonRpc(b) => b.init().await,
            Self::Software(b) => b.init().await,
            Self::Framebuffer(b) => b.init().await,
            Self::External(b) => b.init().await,
        }
    }

    fn dimensions(&self) -> (u32, u32) {
        match self {
            #[cfg(feature = "discovered-display")]
            Self::DiscoveredV2(b) => b.dimensions(),
            #[cfg(feature = "discovered-display")]
            Self::DiscoveredJsonRpc(b) => b.dimensions(),
            Self::Software(b) => b.dimensions(),
            Self::Framebuffer(b) => b.dimensions(),
            Self::External(b) => b.dimensions(),
        }
    }

    async fn present(&mut self, buffer: &[u8]) -> Result<()> {
        match self {
            #[cfg(feature = "discovered-display")]
            Self::DiscoveredV2(b) => b.present(buffer).await,
            #[cfg(feature = "discovered-display")]
            Self::DiscoveredJsonRpc(b) => b.present(buffer).await,
            Self::Software(b) => b.present(buffer).await,
            Self::Framebuffer(b) => b.present(buffer).await,
            Self::External(b) => b.present(buffer).await,
        }
    }

    fn is_available() -> bool {
        #[cfg(feature = "discovered-display")]
        if DiscoveredDisplayBackendV2::is_available()
            || DiscoveredDisplayBackend::is_available()
        {
            return true;
        }
        SoftwareDisplay::is_available()
            || FramebufferDisplay::is_available()
            || ExternalDisplay::is_available()
    }

    fn name(&self) -> &str {
        match self {
            #[cfg(feature = "discovered-display")]
            Self::DiscoveredV2(b) => b.name(),
            #[cfg(feature = "discovered-display")]
            Self::DiscoveredJsonRpc(b) => b.name(),
            Self::Software(b) => b.name(),
            Self::Framebuffer(b) => b.name(),
            Self::External(b) => b.name(),
        }
    }

    fn capabilities(&self) -> DisplayCapabilities {
        match self {
            #[cfg(feature = "discovered-display")]
            Self::DiscoveredV2(b) => b.capabilities(),
            #[cfg(feature = "discovered-display")]
            Self::DiscoveredJsonRpc(b) => b.capabilities(),
            Self::Software(b) => b.capabilities(),
            Self::Framebuffer(b) => b.capabilities(),
            Self::External(b) => b.capabilities(),
        }
    }

    async fn shutdown(&mut self) -> Result<()> {
        match self {
            #[cfg(feature = "discovered-display")]
            Self::DiscoveredV2(b) => b.shutdown().await,
            #[cfg(feature = "discovered-display")]
            Self::DiscoveredJsonRpc(b) => b.shutdown().await,
            Self::Software(b) => b.shutdown().await,
            Self::Framebuffer(b) => b.shutdown().await,
            Self::External(b) => b.shutdown().await,
        }
    }
}

/// Display manager - coordinates multiple backends
pub struct DisplayManager {
    backends: Vec<BackendEntry>,
    active_backend_idx: Option<usize>,
}

struct BackendEntry {
    backend: DisplayBackendImpl,
    priority: BackendPriority,
    initialized: bool,
}

impl DisplayManager {
    /// Initialize display manager and discover available backends
    ///
    /// # Errors
    ///
    /// Returns an error if no display backends are available or initialization fails.
    pub async fn init() -> Result<Self> {
        info!("🌸 Discovering display backends via capabilities...");

        let mut backends = Vec::new();

        // Tier 1: Capability-discovered display backends (biomeOS Neural API)
        #[cfg(feature = "discovered-display")]
        {
            info!("Discovering 'display' capability provider...");
            match DiscoveredDisplayBackendV2::new() {
                Ok(discovered_v2) => {
                    info!("Display capability provider discovered (V2, capability discovery)");
                    backends.push(BackendEntry {
                        backend: DisplayBackendImpl::DiscoveredV2(discovered_v2),
                        priority: BackendPriority::DiscoveredDisplay,
                        initialized: false,
                    });
                }
                Err(e) => {
                    info!("V2 display discovery failed: {e}, trying JSON-RPC fallback...");
                    if DiscoveredDisplayBackend::is_available() {
                        match DiscoveredDisplayBackend::new() {
                            Ok(discovered_json_rpc) => {
                                info!("Display capability provider discovered (JSON-RPC fallback)");
                                backends.push(BackendEntry {
                                    backend: DisplayBackendImpl::DiscoveredJsonRpc(
                                        discovered_json_rpc,
                                    ),
                                    priority: BackendPriority::DiscoveredDisplay,
                                    initialized: false,
                                });
                            }
                            Err(e) => {
                                info!("JSON-RPC display also unavailable: {e}");
                            }
                        }
                    }
                }
            }
        }

        // Tier 2: Try Software Rendering (always available)
        info!("🎨 Checking for software rendering...");
        if SoftwareDisplay::is_available() {
            info!("✅ Software rendering available");
            backends.push(BackendEntry {
                backend: DisplayBackendImpl::Software(SoftwareDisplay::new()),
                priority: BackendPriority::Software,
                initialized: false,
            });
        }

        // Tier 3: Try Framebuffer Direct
        info!("🖥️  Checking for framebuffer direct...");
        if FramebufferDisplay::is_available() {
            info!("✅ Framebuffer direct available");
            match FramebufferDisplay::new() {
                Ok(fb) => {
                    backends.push(BackendEntry {
                        backend: DisplayBackendImpl::Framebuffer(fb),
                        priority: BackendPriority::Framebuffer,
                        initialized: false,
                    });
                }
                Err(e) => {
                    warn!("Failed to create framebuffer: {}", e);
                }
            }
        }

        // Tier 4: Check External Display (lowest priority)
        info!("🪟 Checking for external display server...");
        if ExternalDisplay::is_available() {
            info!("✅ External display server available");
            backends.push(BackendEntry {
                backend: DisplayBackendImpl::External(ExternalDisplay::new()),
                priority: BackendPriority::External,
                initialized: false,
            });
        } else {
            // Prompt user to start display server
            info!("⚠️  No external display server detected");
            if prompt_for_display_server().await? {
                // User may have started display server
                if ExternalDisplay::is_available() {
                    info!("✅ External display server now available");
                    backends.push(BackendEntry {
                        backend: DisplayBackendImpl::External(ExternalDisplay::new()),
                        priority: BackendPriority::External,
                        initialized: false,
                    });
                }
            }
        }

        if backends.is_empty() {
            return Err(DisplayError::NoBackendsAvailable.into());
        }

        // Sort by priority (lower number = higher priority)
        backends.sort_by_key(|entry| entry.priority);

        info!("🌸 Found {} display backend(s)", backends.len());
        for entry in &backends {
            info!(
                "   - {} (Priority: {:?})",
                entry.backend.name(),
                entry.priority
            );
        }

        let mut manager = Self {
            backends,
            active_backend_idx: None,
        };

        // Initialize first available backend
        manager.init_best_backend().await?;

        Ok(manager)
    }

    /// Initialize the best available backend
    async fn init_best_backend(&mut self) -> Result<()> {
        info!("🌸 Initializing best available display backend...");

        for (idx, entry) in self.backends.iter_mut().enumerate() {
            info!("   Trying: {}", entry.backend.name());
            match entry.backend.init().await {
                Ok(()) => {
                    info!("✅ Active display: {}", entry.backend.name());
                    let caps = entry.backend.capabilities();
                    info!("   Capabilities:");
                    info!("      Network: {}", caps.requires_network);
                    info!("      GPU: {}", caps.requires_gpu);
                    info!("      Root: {}", caps.requires_root);
                    info!("      Display Server: {}", caps.requires_display_server);
                    info!("      Max FPS: {}", caps.max_fps);
                    info!("      Latency: {}ms", caps.latency_ms);

                    entry.initialized = true;
                    self.active_backend_idx = Some(idx);
                    return Ok(());
                }
                Err(e) => {
                    warn!("   Failed: {}", e);
                }
            }
        }

        Err(DisplayError::InitFailed.into())
    }

    /// Get dimensions of active backend
    #[must_use]
    pub fn dimensions(&self) -> Option<(u32, u32)> {
        self.active_backend_idx
            .and_then(|idx| self.backends.get(idx))
            .map(|entry| entry.backend.dimensions())
    }

    /// Present frame to active backend
    ///
    /// # Errors
    ///
    /// Returns an error if no backend is active, presentation fails, or fallback fails.
    pub async fn present(&mut self, buffer: &[u8]) -> Result<()> {
        let idx = self
            .active_backend_idx
            .ok_or(DisplayError::NoActiveBackend)?;

        match self.backends[idx].backend.present(buffer).await {
            Ok(()) => Ok(()),
            Err(e) => {
                warn!(
                    "Present failed on {}: {}",
                    self.backends[idx].backend.name(),
                    e
                );
                // Try to fallback to next backend
                self.fallback().await
            }
        }
    }

    /// Fallback to next available backend
    async fn fallback(&mut self) -> Result<()> {
        warn!("🔄 Attempting fallback to next display backend...");

        let Some(current_idx) = self.active_backend_idx else {
            return Err(DisplayError::NoActiveBackendToFallback.into());
        };

        // Try remaining backends
        for idx in (current_idx + 1)..self.backends.len() {
            info!("   Trying: {}", self.backends[idx].backend.name());
            if self.backends[idx].initialized {
                info!("   Already initialized, using it");
                self.active_backend_idx = Some(idx);
                return Ok(());
            }

            match self.backends[idx].backend.init().await {
                Ok(()) => {
                    info!(
                        "✅ Fallback successful: {}",
                        self.backends[idx].backend.name()
                    );
                    self.backends[idx].initialized = true;
                    self.active_backend_idx = Some(idx);
                    return Ok(());
                }
                Err(e) => {
                    warn!("   Failed: {}", e);
                }
            }
        }

        Err(DisplayError::NoFallbackBackend.into())
    }

    /// Get name of active backend
    #[must_use]
    pub fn active_backend_name(&self) -> Option<&str> {
        self.active_backend_idx
            .and_then(|idx| self.backends.get(idx))
            .map(|entry| entry.backend.name())
    }

    /// Shutdown all backends
    ///
    /// # Errors
    ///
    /// Currently always returns `Ok(())`; shutdown failures are logged but not propagated.
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("🌸 Shutting down display manager...");
        for entry in &mut self.backends {
            if entry.initialized
                && let Err(e) = entry.backend.shutdown().await
            {
                warn!("Failed to shutdown {}: {}", entry.backend.name(), e);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::traits::BackendPriority;

    #[test]
    fn backend_priority_ordering() {
        assert!(BackendPriority::DiscoveredDisplay < BackendPriority::Software);
        assert!(BackendPriority::Software < BackendPriority::Framebuffer);
        assert!(BackendPriority::Framebuffer < BackendPriority::External);
        assert_eq!(
            BackendPriority::DiscoveredDisplay.cmp(&BackendPriority::External),
            std::cmp::Ordering::Less
        );
    }

    #[tokio::test]
    async fn test_display_manager_init() {
        let result = DisplayManager::init().await;
        match result {
            Ok(manager) => {
                info!("Display manager initialized successfully");
                if let Some(name) = manager.active_backend_name() {
                    info!("Active backend: {}", name);
                }
            }
            Err(e) => {
                warn!(
                    "Display manager init failed (expected in some environments): {}",
                    e
                );
            }
        }
    }

    #[test]
    fn test_backend_priority_sort_order() {
        use std::cmp::Ordering;
        assert_eq!(
            BackendPriority::DiscoveredDisplay.cmp(&BackendPriority::Software),
            Ordering::Less
        );
        assert_eq!(
            BackendPriority::External.cmp(&BackendPriority::DiscoveredDisplay),
            Ordering::Greater
        );
        assert_eq!(
            BackendPriority::Software.cmp(&BackendPriority::Software),
            Ordering::Equal
        );
    }
}
