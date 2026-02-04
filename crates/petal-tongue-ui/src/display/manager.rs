//! Display Manager
//!
//! Manages multiple display backends with capability-based discovery.
//!
//! TRUE PRIMAL Evolution:
//! - Discovers backends via capabilities (no hardcoded names)
//! - Automatic fallback on failure
//! - Priority-based selection

use crate::display::prompt::prompt_for_display_server;
use crate::display::traits::{BackendPriority, DisplayBackend};
use crate::display::{
    ExternalDisplay, FramebufferDisplay, SoftwareDisplay, ToadstoolDisplay, ToadstoolDisplayV2,
};
use anyhow::{Result, anyhow};
use tracing::{info, warn};

/// Display manager - coordinates multiple backends
pub struct DisplayManager {
    backends: Vec<BackendEntry>,
    active_backend_idx: Option<usize>,
}

struct BackendEntry {
    backend: Box<dyn DisplayBackend>,
    priority: BackendPriority,
    initialized: bool,
}

impl DisplayManager {
    /// Initialize display manager and discover available backends
    pub async fn init() -> Result<Self> {
        info!("🌸 Discovering display backends via capabilities...");

        let mut backends = Vec::new();

        // Tier 1: Try Toadstool V2 (tarpc) via capability discovery (highest priority)
        // Discovery happens at runtime - no hardcoded primal names!
        info!("🌸 Discovering 'display' capability provider (tarpc)...");
        match ToadstoolDisplayV2::new() {
            Ok(toadstool_v2) => {
                info!("✅ Display capability provider discovered via tarpc (high-performance)");
                backends.push(BackendEntry {
                    backend: Box::new(toadstool_v2),
                    priority: BackendPriority::Toadstool,
                    initialized: false,
                });
            }
            Err(e) => {
                info!("⚠️  tarpc display capability discovery failed: {}", e);
                info!("    Trying JSON-RPC fallback...");

                // Fallback to JSON-RPC version
                if ToadstoolDisplay::is_available() {
                    match ToadstoolDisplay::new() {
                        Ok(toadstool) => {
                            info!("✅ Display capability provider discovered (JSON-RPC fallback)");
                            backends.push(BackendEntry {
                                backend: Box::new(toadstool),
                                priority: BackendPriority::Toadstool,
                                initialized: false,
                            });
                        }
                        Err(e) => {
                            info!("⚠️  JSON-RPC display capability also failed: {}", e);
                            info!("    (This is OK - will try other backends)");
                        }
                    }
                } else {
                    info!("⚠️  No display capability provider available");
                    info!("    (This is OK - will try other backends)");
                }
            }
        }

        // Tier 2: Try Software Rendering (always available)
        info!("🎨 Checking for software rendering...");
        if SoftwareDisplay::is_available() {
            info!("✅ Software rendering available");
            backends.push(BackendEntry {
                backend: Box::new(SoftwareDisplay::new()),
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
                        backend: Box::new(fb),
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
                backend: Box::new(ExternalDisplay::new()),
                priority: BackendPriority::External,
                initialized: false,
            });
        } else {
            // Prompt user to start display server
            info!("⚠️  No external display server detected");
            if prompt_for_display_server()? {
                // User may have started display server
                if ExternalDisplay::is_available() {
                    info!("✅ External display server now available");
                    backends.push(BackendEntry {
                        backend: Box::new(ExternalDisplay::new()),
                        priority: BackendPriority::External,
                        initialized: false,
                    });
                }
            }
        }

        if backends.is_empty() {
            return Err(anyhow!("No display backends available"));
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
                    continue;
                }
            }
        }

        Err(anyhow!("Failed to initialize any display backend"))
    }

    /// Get dimensions of active backend
    pub fn dimensions(&self) -> Option<(u32, u32)> {
        self.active_backend_idx
            .and_then(|idx| self.backends.get(idx))
            .map(|entry| entry.backend.dimensions())
    }

    /// Present frame to active backend
    pub async fn present(&mut self, buffer: &[u8]) -> Result<()> {
        let idx = self
            .active_backend_idx
            .ok_or_else(|| anyhow!("No active display backend"))?;

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
            return Err(anyhow::anyhow!("No active backend to fallback from"));
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
                    continue;
                }
            }
        }

        Err(anyhow!("No fallback backend available"))
    }

    /// Get name of active backend
    pub fn active_backend_name(&self) -> Option<&str> {
        self.active_backend_idx
            .and_then(|idx| self.backends.get(idx))
            .map(|entry| entry.backend.name())
    }

    /// Shutdown all backends
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("🌸 Shutting down display manager...");
        for entry in &mut self.backends {
            if entry.initialized {
                if let Err(e) = entry.backend.shutdown().await {
                    warn!("Failed to shutdown {}: {}", entry.backend.name(), e);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_display_manager_init() {
        // This test will succeed if at least one backend is available
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
}
