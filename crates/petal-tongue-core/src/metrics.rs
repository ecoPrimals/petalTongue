// SPDX-License-Identifier: AGPL-3.0-only
//! System Metrics - Real-time system resource and Neural API metrics
//!
//! This module provides data structures for real-time system metrics including:
//! - CPU usage with historical data (sparklines)
//! - Memory usage and allocation
//! - System uptime
//! - Neural API specific metrics (active primals, graphs, executions)

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Maximum number of historical data points to keep for sparklines
const MAX_HISTORY_POINTS: usize = 30;

/// Complete system metrics from Neural API
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// When this metrics snapshot was taken
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// System resource metrics (CPU, memory, uptime)
    pub system: SystemResourceMetrics,

    /// Neural API specific metrics
    pub neural_api: NeuralApiMetrics,
}

/// System resource metrics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemResourceMetrics {
    /// CPU usage percentage (0.0-100.0)
    pub cpu_percent: f32,

    /// Memory used in megabytes
    pub memory_used_mb: u64,

    /// Total memory in megabytes
    pub memory_total_mb: u64,

    /// Memory usage percentage (0.0-100.0)
    pub memory_percent: f32,

    /// System uptime in seconds
    pub uptime_seconds: u64,
}

/// Neural API specific metrics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeuralApiMetrics {
    /// Family ID
    pub family_id: String,

    /// Number of active primals in the ecosystem
    pub active_primals: u32,

    /// Number of available graphs
    pub graphs_available: u32,

    /// Number of currently executing graphs
    pub active_executions: u32,
}

/// CPU usage history with ring buffer for efficient sparkline rendering
#[derive(Clone, Debug)]
pub struct CpuHistory {
    /// Ring buffer of CPU percentages (newest at the back)
    values: VecDeque<f32>,

    /// Maximum number of points to retain
    max_points: usize,
}

impl CpuHistory {
    /// Create a new CPU history tracker
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: VecDeque::with_capacity(MAX_HISTORY_POINTS),
            max_points: MAX_HISTORY_POINTS,
        }
    }

    /// Create with custom max points
    #[must_use]
    pub fn with_capacity(max_points: usize) -> Self {
        Self {
            values: VecDeque::with_capacity(max_points),
            max_points,
        }
    }

    /// Add a new CPU percentage value
    pub fn push(&mut self, cpu_percent: f32) {
        if self.values.len() >= self.max_points {
            self.values.pop_front();
        }
        self.values.push_back(cpu_percent);
    }

    /// Get all values as a slice (for plotting)
    #[must_use]
    pub fn values(&self) -> Vec<f32> {
        self.values.iter().copied().collect()
    }

    /// Get the most recent value
    #[must_use]
    pub fn current(&self) -> Option<f32> {
        self.values.back().copied()
    }

    /// Get the average over all recorded values
    #[must_use]
    #[expect(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    pub fn average(&self) -> f32 {
        if self.values.is_empty() {
            0.0
        } else {
            (f64::from(self.values.iter().sum::<f32>()) / self.values.len() as f64) as f32
        }
    }

    /// Get the maximum value in history
    #[must_use]
    pub fn max(&self) -> f32 {
        self.values.iter().copied().fold(0.0, f32::max)
    }

    /// Get the minimum value in history
    #[must_use]
    pub fn min(&self) -> f32 {
        self.values.iter().copied().fold(100.0, f32::min)
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.values.clear();
    }

    /// Check if there's enough data for a meaningful sparkline (at least 3 points)
    #[must_use]
    pub fn has_sufficient_data(&self) -> bool {
        self.values.len() >= 3
    }
}

impl Default for CpuHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory usage history with ring buffer
#[derive(Clone, Debug)]
pub struct MemoryHistory {
    /// Ring buffer of memory percentages (newest at the back)
    values: VecDeque<f32>,

    /// Maximum number of points to retain
    max_points: usize,
}

impl MemoryHistory {
    /// Create a new memory history tracker
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: VecDeque::with_capacity(MAX_HISTORY_POINTS),
            max_points: MAX_HISTORY_POINTS,
        }
    }

    /// Add a new memory percentage value
    pub fn push(&mut self, memory_percent: f32) {
        if self.values.len() >= self.max_points {
            self.values.pop_front();
        }
        self.values.push_back(memory_percent);
    }

    /// Get all values as a slice (for plotting)
    #[must_use]
    pub fn values(&self) -> Vec<f32> {
        self.values.iter().copied().collect()
    }

    /// Get the most recent value
    #[must_use]
    pub fn current(&self) -> Option<f32> {
        self.values.back().copied()
    }
}

impl Default for MemoryHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemMetrics {
    /// Create empty metrics (for graceful fallback)
    #[must_use]
    pub fn empty(family_id: impl Into<String>) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            system: SystemResourceMetrics {
                cpu_percent: 0.0,
                memory_used_mb: 0,
                memory_total_mb: 0,
                memory_percent: 0.0,
                uptime_seconds: 0,
            },
            neural_api: NeuralApiMetrics {
                family_id: family_id.into(),
                active_primals: 0,
                graphs_available: 0,
                active_executions: 0,
            },
        }
    }

    /// Get formatted uptime string (e.g., "1d 2h 34m")
    #[must_use]
    pub fn uptime_formatted(&self) -> String {
        format_uptime(self.system.uptime_seconds)
    }

    /// Get CPU threshold level for color coding
    #[must_use]
    pub const fn cpu_threshold(&self) -> ThresholdLevel {
        match self.system.cpu_percent {
            x if x < 50.0 => ThresholdLevel::Low,
            x if x < 80.0 => ThresholdLevel::Medium,
            _ => ThresholdLevel::High,
        }
    }

    /// Get memory threshold level for color coding
    #[must_use]
    pub const fn memory_threshold(&self) -> ThresholdLevel {
        match self.system.memory_percent {
            x if x < 50.0 => ThresholdLevel::Low,
            x if x < 80.0 => ThresholdLevel::Medium,
            _ => ThresholdLevel::High,
        }
    }
}

/// Threshold levels for color coding
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThresholdLevel {
    /// Low usage (<50%)
    Low,

    /// Medium usage (50-80%)
    Medium,

    /// High usage (>80%)
    High,
}

impl ThresholdLevel {
    /// Get RGB color for this threshold level
    #[must_use]
    pub const fn color_rgb(&self) -> (u8, u8, u8) {
        match self {
            Self::Low => (34, 197, 94),    // green-500
            Self::Medium => (234, 179, 8), // yellow-500
            Self::High => (239, 68, 68),   // red-500
        }
    }
}

/// Format uptime seconds into human-readable string
fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;

    if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_history_push() {
        let mut history = CpuHistory::new();
        history.push(10.0);
        history.push(20.0);
        history.push(30.0);

        assert_eq!(history.values(), vec![10.0, 20.0, 30.0]);
        assert_eq!(history.current(), Some(30.0));
    }

    #[test]
    fn test_cpu_history_ring_buffer() {
        let mut history = CpuHistory::with_capacity(3);
        history.push(10.0);
        history.push(20.0);
        history.push(30.0);
        history.push(40.0); // Should evict 10.0

        assert_eq!(history.values(), vec![20.0, 30.0, 40.0]);
    }

    #[test]
    fn test_cpu_history_stats() {
        let mut history = CpuHistory::new();
        history.push(10.0);
        history.push(20.0);
        history.push(30.0);

        assert_eq!(history.average(), 20.0);
        assert_eq!(history.max(), 30.0);
        assert_eq!(history.min(), 10.0);
    }

    #[test]
    fn test_uptime_formatting() {
        assert_eq!(format_uptime(0), "0m");
        assert_eq!(format_uptime(60), "1m");
        assert_eq!(format_uptime(3_600), "1h 0m");
        assert_eq!(format_uptime(86_400), "1d 0h 0m");
        assert_eq!(format_uptime(90_061), "1d 1h 1m");
    }

    #[test]
    fn test_threshold_levels() {
        let metrics = SystemMetrics {
            timestamp: chrono::Utc::now(),
            system: SystemResourceMetrics {
                cpu_percent: 45.0,
                memory_used_mb: 4_096,
                memory_total_mb: 8_192,
                memory_percent: 70.0,
                uptime_seconds: 3_600,
            },
            neural_api: NeuralApiMetrics {
                family_id: "test".to_string(),
                active_primals: 3,
                graphs_available: 5,
                active_executions: 1,
            },
        };

        assert_eq!(metrics.cpu_threshold(), ThresholdLevel::Low);
        assert_eq!(metrics.memory_threshold(), ThresholdLevel::Medium);
    }

    #[test]
    fn test_serde_roundtrip() {
        let metrics = SystemMetrics::empty("test");
        let json = serde_json::to_string(&metrics).unwrap();
        let decoded: SystemMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.neural_api.family_id, "test");
    }
}
