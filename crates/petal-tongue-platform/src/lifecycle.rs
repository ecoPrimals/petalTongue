// SPDX-License-Identifier: AGPL-3.0-or-later
//! Platform lifecycle traits — the embedding contract.
//!
//! The host application (Android Activity, iOS `AppDelegate`, desktop main)
//! drives petalTongue through this lifecycle. The model mirrors the Android
//! Activity lifecycle (the most restrictive) so it maps cleanly to all targets:
//!
//! ```text
//! create → start → resume ←→ pause → stop → destroy
//!                     ↑                  │
//!                     └── (restart) ─────┘
//! ```

use crate::config::{EmbedConfig, PlatformConfig};

/// Events emitted from petalTongue back to the host platform.
///
/// The host can subscribe to these to update native UI or forward to
/// other components (e.g. accessibility services, notifications).
#[derive(Debug, Clone)]
pub enum PlatformEvent {
    /// A new SVG render is available.
    RenderReady {
        /// Scene slug that was rendered.
        slug: String,
        /// SVG content string.
        content: String,
    },
    /// An IPC response arrived (for async bridge calls).
    IpcResponse {
        /// Correlation ID from the original request.
        request_id: u64,
        /// JSON-RPC response body.
        json: String,
    },
    /// The runtime encountered a non-fatal error.
    Warning(String),
    /// The runtime wants the host to schedule a redraw.
    RedrawRequested,
    /// Primal state changed (starting, running, stopping).
    StateChanged(RuntimeState),
}

/// Runtime state visible to the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeState {
    /// Created but not yet started.
    Created,
    /// Actively running (transport connected, rendering available).
    Running,
    /// Paused (background — reduced activity, transport may be suspended).
    Paused,
    /// Stopped (no active transport, can be restarted).
    Stopped,
}

/// The platform lifecycle contract.
///
/// Host applications implement this flow by calling methods in order.
/// petalTongue manages internal state transitions and transport accordingly.
///
/// # Platform Mapping
///
/// | Method | Android | iOS | Desktop |
/// |--------|---------|-----|---------|
/// | `on_create` | `onCreate` | `application(_:didFinishLaunching:)` | `main()` entry |
/// | `on_start` | `onStart` | `applicationDidBecomeActive` | after init |
/// | `on_resume` | `onResume` | `applicationWillEnterForeground` | — (always foreground) |
/// | `on_pause` | `onPause` | `applicationDidEnterBackground` | — |
/// | `on_stop` | `onStop` | `applicationWillTerminate` | before exit |
/// | `on_destroy` | `onDestroy` | dealloc | drop |
/// | `on_low_memory` | `onTrimMemory` | `applicationDidReceiveMemoryWarning` | — |
pub trait PlatformLifecycle {
    /// Initialize the runtime with the given configuration.
    ///
    /// Called once when the host creates the petalTongue instance.
    /// Allocates the tokio runtime, graph engine, and transport layer.
    ///
    /// # Errors
    /// Returns error if runtime initialization fails.
    fn on_create(&mut self, config: EmbedConfig) -> Result<(), PlatformError>;

    /// Start active operation (transport connects, discovery begins).
    ///
    /// # Errors
    /// Returns error if transport connection fails.
    fn on_start(&mut self) -> Result<(), PlatformError>;

    /// Resume foreground operation (full rendering, sensor polling).
    ///
    /// # Errors
    /// Returns error if resume fails.
    fn on_resume(&mut self) -> Result<(), PlatformError>;

    /// Pause to background (reduce rendering, suspend non-essential I/O).
    ///
    /// # Errors
    /// Returns error if pause transition fails.
    fn on_pause(&mut self) -> Result<(), PlatformError>;

    /// Stop active operation (disconnect transport, flush state).
    ///
    /// # Errors
    /// Returns error if shutdown fails.
    fn on_stop(&mut self) -> Result<(), PlatformError>;

    /// Destroy the runtime and free all resources.
    ///
    /// # Errors
    /// Returns error if cleanup fails.
    fn on_destroy(&mut self) -> Result<(), PlatformError>;

    /// Host signals memory pressure — drop caches, reduce allocations.
    fn on_low_memory(&mut self);

    /// Host reports configuration change (rotation, locale, dark mode).
    fn on_configuration_changed(&mut self, config: PlatformConfig);

    /// Get current runtime state.
    fn state(&self) -> RuntimeState;
}

/// Errors from the platform lifecycle.
#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    /// Configuration is invalid or missing required fields.
    #[error("invalid configuration: {0}")]
    Config(String),

    /// Transport (IPC/TCP) failed to connect or bind.
    #[error("transport error: {0}")]
    Transport(String),

    /// Internal runtime error (tokio, graph, rendering).
    #[error("runtime error: {0}")]
    Runtime(String),

    /// Operation invalid in current state (e.g. start before create).
    #[error("invalid state transition: cannot {attempted} from {current:?}")]
    InvalidState {
        /// The state at the time of the attempted transition.
        current: RuntimeState,
        /// The operation that was attempted.
        attempted: String,
    },

    /// Serialization/deserialization failure.
    #[error("serialization error: {0}")]
    Serialization(String),
}
