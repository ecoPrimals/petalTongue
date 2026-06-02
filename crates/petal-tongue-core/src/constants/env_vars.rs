// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2025-2026 ecoPrimals Collective
//! Environment variable names for petalTongue runtime configuration.
//!
//! Centralizes string literals used with [`std::env::var`] so production code
//! references a single canonical name per variable.

// ---------------------------------------------------------------------------
// Platform & ecosystem runtime
// ---------------------------------------------------------------------------

/// XDG runtime directory for socket and manifest discovery.
pub const XDG_RUNTIME_DIR: &str = "XDG_RUNTIME_DIR";

/// Ecosystem runtime directory segment under [`XDG_RUNTIME_DIR`].
pub const ECOSYSTEM_RUNTIME_DIR: &str = "ECOSYSTEM_RUNTIME_DIR";

/// BiomeOS socket directory override (family-scoped socket layout).
pub const BIOMEOS_SOCKET_DIR: &str = "BIOMEOS_SOCKET_DIR";

/// BiomeOS family identifier for scoped sockets and BTSP.
pub const FAMILY_ID: &str = "FAMILY_ID";

/// petalTongue-scoped alias for [`FAMILY_ID`].
pub const PETALTONGUE_FAMILY_ID: &str = "PETALTONGUE_FAMILY_ID";

/// Allow insecure development sockets alongside a production family ID.
pub const BIOMEOS_INSECURE: &str = "BIOMEOS_INSECURE";

// ---------------------------------------------------------------------------
// Socket names & discovery
// ---------------------------------------------------------------------------

/// biomeOS Neural API socket name override.
pub const BIOMEOS_SOCKET_NAME: &str = "BIOMEOS_SOCKET_NAME";

/// Explicit biomeOS Neural API socket path override.
pub const BIOMEOS_NEURAL_API_SOCKET: &str = "BIOMEOS_NEURAL_API_SOCKET";

/// Device management socket name override.
pub const BIOMEOS_DEVICE_MANAGEMENT_SOCKET: &str = "BIOMEOS_DEVICE_MANAGEMENT_SOCKET";

/// UI socket name override.
pub const BIOMEOS_UI_SOCKET: &str = "BIOMEOS_UI_SOCKET";

/// Discovery service socket name override.
pub const DISCOVERY_SERVICE_SOCKET: &str = "DISCOVERY_SERVICE_SOCKET";

/// Explicit discovery service socket path override.
pub const DISCOVERY_SOCKET: &str = "DISCOVERY_SOCKET";

/// Legacy biomeOS socket name override.
pub const BIOMEOS_LEGACY_SOCKET: &str = "BIOMEOS_LEGACY_SOCKET";

/// Provenance trio shared socket path override.
pub const PROVENANCE_TRIO_SOCKET: &str = "PROVENANCE_TRIO_SOCKET";

/// Physics compute socket name override.
pub const PHYSICS_COMPUTE_SOCKET_NAME: &str = "PHYSICS_COMPUTE_SOCKET_NAME";

/// Explicit compute provider socket path override.
pub const COMPUTE_SOCKET: &str = "COMPUTE_SOCKET";

// ---------------------------------------------------------------------------
// BTSP & security providers
// ---------------------------------------------------------------------------

/// BTSP handshake provider socket path override.
pub const BTSP_PROVIDER_SOCKET: &str = "BTSP_PROVIDER_SOCKET";

/// Security provider socket path override.
pub const SECURITY_PROVIDER_SOCKET: &str = "SECURITY_PROVIDER_SOCKET";

/// Crypto provider socket path override.
pub const CRYPTO_PROVIDER_SOCKET: &str = "CRYPTO_PROVIDER_SOCKET";

/// Legacy security socket path override.
pub const SECURITY_SOCKET: &str = "SECURITY_SOCKET";

/// BTSP handshake provider prefix override.
pub const BTSP_PROVIDER: &str = "BTSP_PROVIDER";

/// Canonical BTSP family seed (hex) for session creation.
pub const FAMILY_SEED: &str = "FAMILY_SEED";

/// BTSP-scoped alias for [`FAMILY_SEED`].
pub const BTSP_FAMILY_SEED: &str = "BTSP_FAMILY_SEED";

// ---------------------------------------------------------------------------
// Network bind addresses, ports, URLs
// ---------------------------------------------------------------------------

/// petalTongue server bind address override.
pub const PETALTONGUE_BIND_ADDR: &str = "PETALTONGUE_BIND_ADDR";

/// petalTongue TCP server bind host override.
pub const PETALTONGUE_TCP_BIND_HOST: &str = "PETALTONGUE_TCP_BIND_HOST";

/// petalTongue web server port override.
pub const PETALTONGUE_WEB_PORT: &str = "PETALTONGUE_WEB_PORT";

/// petalTongue headless API port override.
pub const PETALTONGUE_HEADLESS_PORT: &str = "PETALTONGUE_HEADLESS_PORT";

/// Display backend port override.
pub const DISPLAY_BACKEND_PORT: &str = "DISPLAY_BACKEND_PORT";

/// petalTongue GPU compute endpoint URL override.
pub const PETALTONGUE_GPU_COMPUTE_ENDPOINT: &str = "PETALTONGUE_GPU_COMPUTE_ENDPOINT";

/// GPU rendering service endpoint URL override.
pub const GPU_RENDERING_ENDPOINT: &str = "GPU_RENDERING_ENDPOINT";

/// Compute provider endpoint URL override.
pub const COMPUTE_PROVIDER_ENDPOINT: &str = "COMPUTE_PROVIDER_ENDPOINT";

/// GPU compute endpoint URL override (legacy alias).
pub const GPU_COMPUTE_ENDPOINT: &str = "GPU_COMPUTE_ENDPOINT";

/// petalTongue HTTP discovery ports override (comma-separated).
pub const PETALTONGUE_DISCOVERY_PORTS: &str = "PETALTONGUE_DISCOVERY_PORTS";

/// Generic HTTP discovery ports override (comma-separated).
pub const DISCOVERY_PORTS: &str = "DISCOVERY_PORTS";

/// Live biomeOS connection target override (host:port).
pub const PETALTONGUE_LIVE_TARGET: &str = "PETALTONGUE_LIVE_TARGET";

/// biomeOS base URL override.
pub const BIOMEOS_URL: &str = "BIOMEOS_URL";

/// petalTongue web server URL override.
pub const PETALTONGUE_WEB_URL: &str = "PETALTONGUE_WEB_URL";

/// petalTongue headless API URL override.
pub const PETALTONGUE_HEADLESS_URL: &str = "PETALTONGUE_HEADLESS_URL";

/// petalTongue entropy stream endpoint URL override.
pub const PETALTONGUE_ENTROPY_ENDPOINT: &str = "PETALTONGUE_ENTROPY_ENDPOINT";

/// petalTongue sandbox security endpoint URL override.
pub const PETALTONGUE_SANDBOX_SECURITY_ENDPOINT: &str = "PETALTONGUE_SANDBOX_SECURITY_ENDPOINT";

/// petalTongue headless endpoint URL override (sandbox security alias).
pub const PETALTONGUE_HEADLESS_ENDPOINT: &str = "PETALTONGUE_HEADLESS_ENDPOINT";

/// petalTongue sandbox security port override.
pub const PETALTONGUE_SANDBOX_SECURITY_PORT: &str = "PETALTONGUE_SANDBOX_SECURITY_PORT";

/// petalTongue WebSocket endpoint URL override.
pub const PETALTONGUE_WS_ENDPOINT: &str = "PETALTONGUE_WS_ENDPOINT";

/// biomeOS WebSocket port override.
pub const BIOMEOS_WS_PORT: &str = "BIOMEOS_WS_PORT";

/// biomeOS WebSocket endpoint URL override.
pub const BIOMEOS_WS_ENDPOINT: &str = "BIOMEOS_WS_ENDPOINT";

// ---------------------------------------------------------------------------
// Content backend & IPC auth
// ---------------------------------------------------------------------------

/// Content backend socket path override.
pub const CONTENT_BACKEND_SOCKET: &str = "CONTENT_BACKEND_SOCKET";

/// Legacy content provider socket alias. Use `CONTENT_BACKEND_SOCKET` instead.
#[deprecated(since = "1.6.7", note = "use CONTENT_BACKEND_SOCKET (TRUE PRIMAL)")]
pub const NESTGATE_SOCKET: &str = "NESTGATE_SOCKET";

/// Content backend provider name prefix override.
pub const CONTENT_BACKEND_PROVIDER: &str = "CONTENT_BACKEND_PROVIDER";

/// JSON-RPC authentication mode (`permissive` or `enforced`).
pub const PETALTONGUE_AUTH_MODE: &str = "PETALTONGUE_AUTH_MODE";

/// Generic JSON-RPC authentication mode override.
pub const AUTH_MODE: &str = "AUTH_MODE";

/// Scene graph visualization purpose key (hex-encoded 32 bytes).
pub const PETALTONGUE_SCENE_KEY: &str = "PETALTONGUE_SCENE_KEY";

// ---------------------------------------------------------------------------
// Application configuration
// ---------------------------------------------------------------------------

/// petalTongue TOML config file path override.
pub const PETALTONGUE_CONFIG: &str = "PETALTONGUE_CONFIG";

/// Web mode document root override.
pub const PETALTONGUE_DOCROOT: &str = "PETALTONGUE_DOCROOT";

/// Web mode static asset cache TTL override (seconds).
pub const PETALTONGUE_CACHE_TTL: &str = "PETALTONGUE_CACHE_TTL";

/// Strip source maps from web responses when truthy.
pub const PETALTONGUE_STRIP_SOURCES: &str = "PETALTONGUE_STRIP_SOURCES";

/// Enable SPA routing mode when truthy.
pub const PETALTONGUE_SPA: &str = "PETALTONGUE_SPA";

/// Comma-separated CORS allowed origins override.
pub const PETALTONGUE_ALLOWED_ORIGINS: &str = "PETALTONGUE_ALLOWED_ORIGINS";

/// Capability discovery timeout override (milliseconds).
pub const PETALTONGUE_DISCOVERY_TIMEOUT: &str = "PETALTONGUE_DISCOVERY_TIMEOUT";

// ---------------------------------------------------------------------------
// Platform display detection
// ---------------------------------------------------------------------------

/// X11 display identifier (graphical session detection).
pub const DISPLAY: &str = "DISPLAY";

/// Wayland display identifier (graphical session detection).
pub const WAYLAND_DISPLAY: &str = "WAYLAND_DISPLAY";
