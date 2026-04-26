// SPDX-License-Identifier: AGPL-3.0-or-later
//! Request/response types for `visualization.texture.upload` and
//! `visualization.texture.attach` IPC methods.

use serde::{Deserialize, Serialize};

/// Request for `visualization.texture.upload`.
#[derive(Debug, Clone, Deserialize)]
pub struct TextureUploadRequest {
    /// Unique texture identifier (e.g. `"sprite-hero-idle-01"`).
    pub texture_id: String,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Pixel format (currently only `"rgba8"`).
    #[serde(default = "default_rgba8")]
    pub format: String,
    /// Base64-encoded pixel data.
    pub data: String,
}

/// Response for `visualization.texture.upload`.
#[derive(Debug, Clone, Serialize)]
pub struct TextureUploadResponse {
    /// The texture ID that was stored.
    pub texture_id: String,
    /// `"loaded"` on success.
    pub status: String,
}

/// Request for `visualization.texture.attach` (shared-memory / external source).
#[derive(Debug, Clone, Deserialize)]
pub struct TextureAttachRequest {
    /// Unique texture identifier.
    pub texture_id: String,
    /// Source URI (e.g. `"memfd://godot-fb-0"`).
    pub source: String,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Pixel format (currently only `"rgba8"`).
    #[serde(default = "default_rgba8")]
    pub format: String,
    /// Refresh strategy: `"per_frame"` or `"on_change"`.
    #[serde(default = "default_on_change")]
    pub refresh: String,
}

/// Response for `visualization.texture.attach`.
#[derive(Debug, Clone, Serialize)]
pub struct TextureAttachResponse {
    /// The texture ID that was attached.
    pub texture_id: String,
    /// `"attached"` on success.
    pub status: String,
}

fn default_rgba8() -> String {
    "rgba8".to_string()
}

fn default_on_change() -> String {
    "on_change".to_string()
}
