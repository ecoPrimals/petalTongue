// SPDX-License-Identifier: AGPL-3.0-or-later
//! Primal metrics and telemetry types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Primal metrics
///
/// Performance and operational metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalMetrics {
    /// Frames per second (for UI primals)
    pub fps: Option<f32>,

    /// Time since last frame in seconds
    pub time_since_last_frame: Option<f32>,

    /// Is primal hanging (no frames for >5s)
    pub is_hanging: bool,

    /// Total frames rendered
    pub total_frames: u64,

    /// CPU usage percentage (0-100)
    pub cpu_usage: Option<f32>,

    /// Memory usage in bytes
    pub memory_usage: Option<u64>,

    /// Uptime in seconds
    pub uptime_seconds: u64,

    /// Custom metrics
    #[serde(default)]
    pub custom: HashMap<String, String>,
}
