// SPDX-License-Identifier: AGPL-3.0-or-later
//! Display and UI geometry constants: window size, terminal dimensions, FPS.

use std::time::Duration;

use super::env_or;

/// Default window width in pixels (overridable via `PETALTONGUE_WINDOW_WIDTH` env var).
pub const DEFAULT_WINDOW_WIDTH: u32 = 1920;

/// Default window height in pixels (overridable via `PETALTONGUE_WINDOW_HEIGHT` env var).
pub const DEFAULT_WINDOW_HEIGHT: u32 = 1080;

/// Default terminal columns (for CLI/text UI).
pub const DEFAULT_TERMINAL_COLS: u16 = 80;

/// Default terminal rows (for CLI/text UI).
pub const DEFAULT_TERMINAL_ROWS: u16 = 24;

/// Default SVG export width in pixels.
pub const DEFAULT_SVG_WIDTH: u32 = 800;

/// Default SVG export height in pixels.
pub const DEFAULT_SVG_HEIGHT: u32 = 600;

/// Max FPS for rendering (overridable via config)
pub const DEFAULT_MAX_FPS: u32 = 60;

/// Frame sleep interval for loops targeting ~[`DEFAULT_MAX_FPS`] (e.g. awakening coordinator).
pub const FRAME_PACING_60FPS: Duration = Duration::from_millis(16);

/// Default window size (width, height). Env: `PETALTONGUE_WINDOW_WIDTH`, `PETALTONGUE_WINDOW_HEIGHT`.
#[must_use]
pub fn default_window_size() -> (u32, u32) {
    (
        env_or("PETALTONGUE_WINDOW_WIDTH", DEFAULT_WINDOW_WIDTH),
        env_or("PETALTONGUE_WINDOW_HEIGHT", DEFAULT_WINDOW_HEIGHT),
    )
}

// ---------------------------------------------------------------------------
// Graph rendering geometry (used by IPC scene compilation)
// ---------------------------------------------------------------------------

/// Node circle radius for graph topology visualization.
pub const GRAPH_NODE_RADIUS: f64 = 8.0;

/// Node border stroke width.
pub const GRAPH_NODE_STROKE_WIDTH: f32 = 1.5;

/// Edge line stroke width.
pub const GRAPH_EDGE_STROKE_WIDTH: f32 = 1.0;

/// Text label offset X from node center.
pub const GRAPH_LABEL_OFFSET_X: f64 = 12.0;

/// Text label offset Y from node center.
pub const GRAPH_LABEL_OFFSET_Y: f64 = -6.0;

/// Font size for node labels.
pub const GRAPH_LABEL_FONT_SIZE: f64 = 11.0;

/// Bytes per pixel for RGBA8 texture format.
pub const RGBA8_BYTES_PER_PIXEL: usize = 4;
