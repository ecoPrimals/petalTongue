//! Pure Rust Display System
//!
//! Provides multiple display backends for rendering the GUI without requiring
//! traditional display servers (X11/Wayland).
//!
//! # Four-Tier Strategy
//!
//! 1. **Toadstool WASM** - Network effect, primal collaboration, GPU acceleration
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
//!   [ToadstoolDisplay, SoftwareDisplay, FramebufferDisplay, ExternalDisplay]
//! ```

pub mod backends;
pub mod manager;
pub mod prompt;
pub mod renderer;
pub mod traits;

pub use backends::{
    external::ExternalDisplay, framebuffer::FramebufferDisplay, software::SoftwareDisplay,
    toadstool::ToadstoolDisplay, toadstool_v2::ToadstoolDisplay as ToadstoolDisplayV2,
};
pub use manager::DisplayManager;
pub use prompt::prompt_for_display_server;
pub use renderer::EguiPixelRenderer;
pub use traits::{DisplayBackend, DisplayCapabilities};

use anyhow::Result;

/// Initialize display system and return best available backend
pub async fn init_display() -> Result<DisplayManager> {
    tracing::info!("🌸 Initializing Pure Rust display system...");
    DisplayManager::init().await
}
