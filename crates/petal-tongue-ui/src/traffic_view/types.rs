// SPDX-License-Identifier: AGPL-3.0-or-later
//! Traffic view data types

/// User interaction intent produced by the traffic view render method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrafficIntent {
    SetColorScheme(ColorScheme),
    SelectFlow { from: String, to: String },
    CloseDetails,
    ToggleMetrics,
    Clear,
}

/// Pre-computed detail display for a selected flow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowDetailDisplay {
    pub from: String,
    pub to: String,
    pub volume_label: String,
    pub requests_label: String,
    pub latency_label: String,
    pub error_rate_label: String,
}

/// Traffic metrics for an edge
#[derive(Clone, Debug, Default)]
pub struct TrafficMetrics {
    /// Bytes transferred per second
    pub bytes_per_second: u64,
    /// Request count per second
    pub requests_per_second: f64,
    /// Average latency in milliseconds
    pub avg_latency_ms: f64,
    /// Error rate (0.0 to 1.0)
    pub error_rate: f64,
}

/// Traffic flow visualization
#[derive(Clone, Debug)]
pub struct TrafficFlow {
    /// Source primal ID
    pub from: String,
    /// Destination primal ID
    pub to: String,
    /// Traffic metrics
    pub metrics: TrafficMetrics,
    /// Flow color RGBA (based on health/volume)
    pub color: [u8; 4],
}

/// Color scheme for traffic visualization
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorScheme {
    /// Color by traffic volume
    Volume,
    /// Color by latency
    Latency,
    /// Color by error rate
    ErrorRate,
}
