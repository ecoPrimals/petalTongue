// SPDX-License-Identifier: AGPL-3.0-or-later
//! Linux `/proc`-based implementation of [`PlatformMetrics`].
//!
//! Reads `/proc/stat` for CPU usage and `/proc/meminfo` for memory stats.
//! Zero external dependencies — pure `std::fs` parsing.

#![expect(
    clippy::cast_precision_loss,
    reason = "/proc stats use u64→f64 for display; precision loss acceptable"
)]

use std::fs;

use super::{PlatformMetrics, ResourceSnapshot};

/// Linux `/proc` metrics provider.
///
/// Maintains internal state for CPU delta computation (requires at least
/// two calls to `snapshot()` before CPU percentage is meaningful).
#[derive(Debug, Default)]
pub struct LinuxProcMetrics {
    prev_total: u64,
    prev_busy: u64,
}

impl PlatformMetrics for LinuxProcMetrics {
    fn snapshot(&mut self) -> ResourceSnapshot {
        let cpu_percent = self.read_cpu_percent();
        let (memory_total, memory_used) = read_memory();
        let cpu_count = read_cpu_count();

        ResourceSnapshot {
            cpu_percent,
            memory_total,
            memory_used,
            cpu_count,
        }
    }

    fn available(&self) -> bool {
        fs::metadata("/proc/stat").is_ok()
    }

    fn source_id(&self) -> &'static str {
        "proc"
    }
}

impl LinuxProcMetrics {
    /// Create a new Linux proc metrics provider.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn read_cpu_percent(&mut self) -> f32 {
        let Some((total, busy)) = parse_cpu_aggregate() else {
            return 0.0;
        };

        let total_delta = total.saturating_sub(self.prev_total);
        let busy_delta = busy.saturating_sub(self.prev_busy);

        self.prev_total = total;
        self.prev_busy = busy;

        if total_delta == 0 {
            return 0.0;
        }

        (busy_delta as f64 / total_delta as f64 * 100.0) as f32
    }
}

/// Parse aggregate CPU line from /proc/stat, returns (total_ticks, busy_ticks).
fn parse_cpu_aggregate() -> Option<(u64, u64)> {
    let s = fs::read_to_string("/proc/stat").ok()?;
    let first = s.lines().next()?;
    if !first.starts_with("cpu ") {
        return None;
    }
    let parts: Vec<u64> = first
        .split_whitespace()
        .skip(1)
        .filter_map(|p| p.parse().ok())
        .collect();

    if parts.len() < 4 {
        return None;
    }

    let total: u64 = parts.iter().sum();
    // idle = parts[3], iowait = parts[4] (if present)
    let idle = parts[3] + parts.get(4).copied().unwrap_or(0);
    let busy = total.saturating_sub(idle);

    Some((total, busy))
}

/// Read total and used memory from /proc/meminfo.
fn read_memory() -> (u64, u64) {
    let Ok(s) = fs::read_to_string("/proc/meminfo") else {
        return (0, 0);
    };

    let mut total_kb: u64 = 0;
    let mut available_kb: u64 = 0;

    for line in s.lines() {
        if let Some(rest) = line.strip_prefix("MemTotal:") {
            total_kb = parse_kb_value(rest);
        } else if let Some(rest) = line.strip_prefix("MemAvailable:") {
            available_kb = parse_kb_value(rest);
        }
        if total_kb > 0 && available_kb > 0 {
            break;
        }
    }

    let total = total_kb * 1024;
    let used = total_kb.saturating_sub(available_kb) * 1024;
    (total, used)
}

/// Parse a value like "  16384000 kB" → 16384000
fn parse_kb_value(s: &str) -> u64 {
    s.split_whitespace()
        .next()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

/// Count CPU cores from /proc/stat (cpuN lines minus aggregate).
fn read_cpu_count() -> usize {
    let Ok(s) = fs::read_to_string("/proc/stat") else {
        return 1;
    };
    let count = s.lines().filter(|l| l.starts_with("cpu")).count();
    count.saturating_sub(1).max(1)
}

#[cfg(test)]
#[cfg(target_os = "linux")]
mod tests {
    use super::*;

    #[test]
    fn linux_metrics_available() {
        let m = LinuxProcMetrics::new();
        assert!(m.available());
    }

    #[test]
    fn linux_metrics_source_id() {
        let m = LinuxProcMetrics::new();
        assert_eq!(m.source_id(), "proc");
    }

    #[test]
    fn linux_metrics_snapshot_reasonable() {
        let mut m = LinuxProcMetrics::new();
        let snap = m.snapshot();
        assert!(snap.memory_total > 0, "should detect some memory");
        assert!(snap.cpu_count >= 1, "should detect at least 1 CPU");
    }

    #[test]
    fn linux_metrics_cpu_delta() {
        let mut m = LinuxProcMetrics::new();
        // First call establishes baseline
        let _ = m.snapshot();
        // Second call computes delta
        let snap = m.snapshot();
        // CPU percent should be non-negative
        assert!(snap.cpu_percent >= 0.0);
        assert!(snap.cpu_percent <= 100.0 * snap.cpu_count as f32);
    }

    #[test]
    fn read_memory_returns_nonzero() {
        let (total, _used) = read_memory();
        assert!(total > 0);
    }

    #[test]
    fn read_cpu_count_positive() {
        let count = read_cpu_count();
        assert!(count >= 1);
    }
}
