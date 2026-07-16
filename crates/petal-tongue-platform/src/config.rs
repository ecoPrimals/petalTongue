// SPDX-License-Identifier: AGPL-3.0-or-later
//! Configuration types for platform embedding.

use serde::{Deserialize, Serialize};

/// Target platform identifier.
///
/// Determines transport strategy, rendering capabilities, and lifecycle model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    /// Linux/macOS/Windows desktop — full capabilities.
    Desktop,
    /// Android device — `cdylib` loaded by Kotlin/Java Activity.
    Android,
    /// iOS device — `dylib` loaded by Swift `AppDelegate`.
    Ios,
    /// WebAssembly in browser — no filesystem, no sockets.
    Wasm,
    /// Game console or embedded kiosk — constrained I/O.
    Console,
}

impl Platform {
    /// Whether this platform supports Unix domain sockets natively.
    #[must_use]
    pub const fn supports_uds(&self) -> bool {
        matches!(self, Self::Desktop)
    }

    /// Whether this platform should use TCP as primary transport.
    #[must_use]
    pub const fn prefers_tcp(&self) -> bool {
        matches!(self, Self::Android | Self::Ios | Self::Console)
    }

    /// Whether filesystem access is available.
    #[must_use]
    pub const fn has_filesystem(&self) -> bool {
        !matches!(self, Self::Wasm)
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Desktop => write!(f, "desktop"),
            Self::Android => write!(f, "android"),
            Self::Ios => write!(f, "ios"),
            Self::Wasm => write!(f, "wasm"),
            Self::Console => write!(f, "console"),
        }
    }
}

/// Runtime configuration changed by the host (orientation, locale, density).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    /// Screen width in logical pixels.
    pub width: u32,
    /// Screen height in logical pixels.
    pub height: u32,
    /// Display density (1.0 = standard, 2.0 = retina/xxhdpi).
    pub density: f32,
    /// Dark mode preference from the host OS.
    pub dark_mode: bool,
    /// BCP-47 locale tag (e.g. "en-US").
    pub locale: String,
}

impl Default for PlatformConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            density: 1.0,
            dark_mode: false,
            locale: "en-US".to_owned(),
        }
    }
}

/// Configuration passed to [`EmbeddedRuntime`](crate::EmbeddedRuntime) on creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedConfig {
    /// Target platform.
    pub platform: Platform,
    /// Initial display configuration.
    #[serde(default)]
    pub display: PlatformConfig,
    /// Optional TCP port for IPC (overrides auto-discovery).
    pub tcp_port: Option<u16>,
    /// Optional socket path hint (used on Unix; ignored on platforms without UDS).
    pub socket_path: Option<String>,
    /// Enable tracing/logging output.
    #[serde(default = "default_true")]
    pub logging: bool,
}

const fn default_true() -> bool {
    true
}

impl EmbedConfig {
    /// Create a minimal config for the given platform with defaults.
    #[must_use]
    pub fn new(platform: Platform) -> Self {
        Self {
            platform,
            display: PlatformConfig::default(),
            tcp_port: None,
            socket_path: None,
            logging: true,
        }
    }
}
