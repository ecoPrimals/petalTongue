// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pure Rust Display System
//!
//! Provides multiple display backends for rendering the interface without requiring
//! traditional display servers (X11/Wayland).
//!
//! # Four-Tier Strategy
//!
//! 1. **Capability-discovered display** - tarpc primary path, JSON-RPC fallback; endpoint from biomeOS (no hardcoded provider)
//! 2. **Software Rendering** - Pure Rust, works everywhere, no GPU needed
//! 3. **Framebuffer Direct** - Linux console mode, embedded systems
//! 4. **External Display** - Traditional display server (benchmark/fallback)
//!
//! # Architecture
//!
//! ```text
//! DisplayManager
//!   ├── discover_backends()
//!   ├── init_best_backend()
//!   └── present(pixel_buffer)
//!       ↓
//!   [DiscoveredDisplayBackend, DiscoveredDisplayBackendV2, SoftwareDisplay, FramebufferDisplay, ExternalDisplay]
//! ```

pub mod backends;
pub mod manager;
pub mod prompt;
pub mod renderer;
pub mod traits;

pub use backends::{
    discovered_display::DiscoveredDisplayBackend,
    discovered_display_v2::DiscoveredDisplayBackendV2, external::ExternalDisplay,
    framebuffer::FramebufferDisplay, software::SoftwareDisplay,
};
pub use manager::DisplayManager;
pub use prompt::prompt_for_display_server;
pub use renderer::EguiPixelRenderer;
pub use traits::{DisplayBackend, DisplayCapabilities};

use crate::error::Result;

/// Initialize display system and return best available backend
///
/// # Errors
///
/// Returns an error if no display backends are available or initialization fails.
pub async fn init_display() -> Result<DisplayManager> {
    tracing::info!("🌸 Initializing Pure Rust display system...");
    DisplayManager::init().await
}
