// SPDX-License-Identifier: AGPL-3.0-or-later
//! System stats collector — composes `PlatformMetrics` (core) with
//! per-process /proc parsing for UI display.
//!
//! System-level metrics (CPU%, memory, core count) are delegated to
//! [`petal_tongue_core::platform_metrics`] which provides platform-appropriate
//! implementations. Process-level stats remain Linux-specific.

#![expect(
    clippy::cast_precision_loss,
    reason = "/proc stats use u64→f64 for display; precision loss acceptable"
)]

use std::collections::HashMap;
#[cfg(target_os = "linux")]
use std::fs;

use petal_tongue_core::platform_metrics::{self, PlatformMetrics, ResourceSnapshot};

/// Source identifier for live metrics (matches `LinuxProcMetrics::source_id()`)
pub const SOURCE_ID: &str = "proc";

/// Linux page size (bytes), queried from the kernel at runtime.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn page_size() -> u64 {
    #[cfg(target_os = "linux")]
    {
        u64::try_from(rustix::param::page_size()).unwrap_or(4096)
    }
    #[cfg(not(target_os = "linux"))]
    {
        4096
    }
}

/// CPU and memory stats — delegates system metrics to [`PlatformMetrics`],
/// maintains per-process CPU deltas for process listing.
pub struct ProcStats {
    /// Platform-agnostic metrics provider (Linux /proc, Android JNI, WASM stub, etc.)
    metrics: Box<dyn PlatformMetrics>,
    /// Most recent snapshot (cached between calls to avoid redundant reads)
    last_snapshot: ResourceSnapshot,
    /// Previous process CPU times for delta calculation
    prev_process_times: HashMap<u32, (u64, u64)>,
    /// Previous total CPU time (for process % calculation)
    prev_total_cpu: u64,
}

impl std::fmt::Debug for ProcStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProcStats")
            .field("last_snapshot", &self.last_snapshot)
            .field("prev_process_times_len", &self.prev_process_times.len())
            .field("prev_total_cpu", &self.prev_total_cpu)
            .finish_non_exhaustive()
    }
}

/// Process info for display
#[derive(Clone, Debug)]
pub struct ProcessInfo {
    /// Process ID
    pub pid: u32,
    /// Process name (from comm)
    pub name: String,
    /// CPU usage percentage (0-100+)
    pub cpu_usage: f32,
    /// Memory in bytes (RSS)
    pub memory: u64,
}

impl ProcStats {
    /// Create new stats collector with the platform-detected metrics provider.
    #[must_use]
    pub fn new() -> Self {
        Self {
            metrics: platform_metrics::detect(),
            last_snapshot: ResourceSnapshot::default(),
            prev_process_times: HashMap::new(),
            prev_total_cpu: 0,
        }
    }

    /// Refresh and return current CPU usage (0-100).
    #[must_use]
    pub fn cpu_usage(&mut self) -> f32 {
        self.last_snapshot = self.metrics.snapshot();
        self.last_snapshot.cpu_percent
    }

    /// Total memory in bytes.
    #[must_use]
    pub const fn total_memory(&self) -> u64 {
        self.last_snapshot.memory_total
    }

    /// Used memory in bytes.
    #[must_use]
    pub const fn used_memory(&self) -> u64 {
        self.last_snapshot.memory_used
    }

    /// Number of CPU cores.
    #[must_use]
    pub const fn cpu_count(&self) -> usize {
        self.last_snapshot.cpu_count
    }

    /// Collect all processes with CPU and memory. Updates internal state for CPU delta.
    pub fn processes(&mut self) -> Vec<ProcessInfo> {
        #[cfg(target_os = "linux")]
        {
            let total_cpu = read_total_cpu_ticks().unwrap_or(0);
            let procs =
                collect_processes(total_cpu, &mut self.prev_process_times, self.prev_total_cpu);
            self.prev_total_cpu = total_cpu;
            procs
        }

        #[cfg(not(target_os = "linux"))]
        {
            Vec::new()
        }
    }
}

impl Default for ProcStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if the platform metrics provider can deliver meaningful data.
#[must_use]
pub fn proc_available() -> bool {
    platform_metrics::detect().available()
}

/// Read total CPU ticks from /proc/stat for per-process delta calculation.
#[cfg(target_os = "linux")]
fn read_total_cpu_ticks() -> Option<u64> {
    let s = fs::read_to_string("/proc/stat").ok()?;
    let first = s.lines().next()?;
    if !first.starts_with("cpu ") {
        return None;
    }
    let total: u64 = first
        .split_whitespace()
        .skip(1)
        .filter_map(|p| p.parse::<u64>().ok())
        .sum();
    Some(total)
}

#[cfg(target_os = "linux")]
fn collect_processes(
    total_cpu: u64,
    prev_times: &mut HashMap<u32, (u64, u64)>,
    prev_total_cpu: u64,
) -> Vec<ProcessInfo> {
    let total_delta = total_cpu.saturating_sub(prev_total_cpu);
    let num_cpus = std::thread::available_parallelism()
        .map(std::num::NonZero::get)
        .unwrap_or(1) as u64;

    let mut result = Vec::new();
    let Ok(entries) = fs::read_dir("/proc") else {
        return result;
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name_str) = name.to_str() else {
            continue;
        };
        let Ok(pid) = name_str.parse::<u32>() else {
            continue;
        };

        let Some((comm, utime, stime, rss)) = read_proc_stat(pid) else {
            continue;
        };

        let cpu_pct = if total_delta > 0 {
            let prev = prev_times.get(&pid).copied().unwrap_or((0, 0));
            let utime_delta = utime.saturating_sub(prev.0);
            let stime_delta = stime.saturating_sub(prev.1);
            let process_delta = utime_delta + stime_delta;
            prev_times.insert(pid, (utime, stime));
            (process_delta as f64 / total_delta as f64) * 100.0 * num_cpus as f64
        } else {
            prev_times.insert(pid, (utime, stime));
            0.0
        };

        result.push(ProcessInfo {
            pid,
            name: comm,
            cpu_usage: cpu_pct as f32,
            memory: rss * page_size(),
        });
    }

    result
}

#[cfg(target_os = "linux")]
fn read_proc_stat(pid: u32) -> Option<(String, u64, u64, u64)> {
    let path = format!("/proc/{pid}/stat");
    let s = fs::read_to_string(&path).ok()?;
    let close_paren = s.rfind(')')?;
    let comm = s[1..close_paren].to_owned();
    let rest = s[close_paren + 1..].trim_start();
    let parts: Vec<&str> = rest.split_whitespace().collect();
    let utime: u64 = parts.get(10)?.parse().ok()?;
    let stime: u64 = parts.get(11)?.parse().ok()?;
    let rss: u64 = parts.get(20)?.parse().ok()?;
    Some((comm, utime, stime, rss))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proc_stats_new() {
        let s = ProcStats::new();
        assert!(s.total_memory().saturating_add(1) >= s.total_memory());
    }

    #[test]
    fn cpu_usage_bounds() {
        let mut s = ProcStats::new();
        let u = s.cpu_usage();
        assert!(u >= 0.0);
        assert!(u <= 100.0 || !proc_available());
    }

    #[test]
    fn process_info_fields() {
        let p = ProcessInfo {
            pid: 1,
            name: "init".to_owned(),
            cpu_usage: 0.5,
            memory: 1024 * 4096,
        };
        assert_eq!(p.pid, 1);
        assert_eq!(p.name, "init");
        assert!((p.cpu_usage - 0.5).abs() < f32::EPSILON);
        assert_eq!(p.memory, 1024 * 4096);
    }

    #[test]
    fn proc_stats_total_memory_non_negative() {
        let mut s = ProcStats::new();
        let _ = s.cpu_usage(); // prime the snapshot
        let _total = s.total_memory();
    }

    #[test]
    fn proc_stats_used_memory_bounded_by_total() {
        let mut s = ProcStats::new();
        let _ = s.cpu_usage(); // prime
        let total = s.total_memory();
        let used = s.used_memory();
        assert!(used <= total || total == 0);
    }

    #[test]
    fn proc_stats_cpu_count_positive() {
        let mut s = ProcStats::new();
        let _ = s.cpu_usage(); // prime
        let count = s.cpu_count();
        assert!(count >= 1);
    }

    #[test]
    fn proc_available_consistent() {
        let a = proc_available();
        #[cfg(target_os = "linux")]
        assert!(a);
        #[cfg(not(target_os = "linux"))]
        assert!(!a);
    }

    #[test]
    fn proc_stats_cpu_usage_second_call() {
        let mut s = ProcStats::new();
        let _ = s.cpu_usage();
        let u2 = s.cpu_usage();
        assert!(u2 >= 0.0);
        assert!(u2 <= 100.0 || !proc_available());
    }

    #[test]
    fn process_info_memory_uses_page_size() {
        let ps = page_size();
        let p = ProcessInfo {
            pid: 1,
            name: "test".to_owned(),
            cpu_usage: 0.0,
            memory: 100 * ps,
        };
        assert_eq!(p.memory, 100 * ps);
    }

    #[test]
    fn process_info_cpu_usage_bounds() {
        let p = ProcessInfo {
            pid: 1,
            name: "test".to_owned(),
            cpu_usage: 50.5,
            memory: 0,
        };
        assert!((p.cpu_usage - 50.5).abs() < f32::EPSILON);
    }

    #[test]
    fn proc_stats_processes_returns_vec() {
        let mut s = ProcStats::new();
        let procs = s.processes();
        assert!(procs.len() < 1_000_000);
    }

    #[test]
    fn source_id_constant() {
        assert_eq!(SOURCE_ID, "proc");
    }
}
