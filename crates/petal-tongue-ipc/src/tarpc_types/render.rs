// SPDX-License-Identifier: AGPL-3.0-or-later
//! Graph rendering request/response types for UI operations.

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Graph rendering request
///
/// Request to render a graph topology visualization or raw frame buffer.
/// Supports two modes:
/// 1. Graph topology rendering (topology field populated)
/// 2. Raw frame buffer rendering (data field populated, format="rgba8")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderRequest {
    /// Graph topology data (JSON or binary) - for graph rendering
    #[serde(default)]
    pub topology: Bytes,

    /// Raw pixel data - for frame buffer rendering (e.g., RGBA8)
    #[serde(default)]
    pub data: Bytes,

    /// Render width in pixels
    pub width: u32,

    /// Render height in pixels
    pub height: u32,

    /// Render format ("png", "svg", "jpg", "rgba8")
    /// - "rgba8": Raw 32-bit RGBA pixel data for frame buffer rendering
    /// - "png"/"svg"/"jpg": Graph topology rendering output formats
    pub format: String,

    /// Optional render settings
    #[serde(default)]
    pub settings: HashMap<String, String>,

    /// Optional metadata (capabilities, primal info, etc.)
    #[serde(default)]
    pub metadata: Option<HashMap<String, String>>,
}

/// Graph rendering response
///
/// Rendered visualization or frame buffer output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderResponse {
    /// Success flag
    pub success: bool,

    /// Rendered image data (bytes)
    /// - For graph rendering: PNG/SVG/JPG encoded data
    /// - For frame buffer: RGBA8 pixel data (optional, may be displayed remotely)
    #[serde(default)]
    pub data: Bytes,

    /// Output width in pixels
    pub width: u32,

    /// Output height in pixels
    pub height: u32,

    /// Error message if failed
    pub error: Option<String>,

    /// Render time in milliseconds
    pub render_time_ms: u64,
}
