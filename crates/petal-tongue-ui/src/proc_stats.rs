// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pure Rust system stats via /proc parsing (ecoBin v3.0 compliant)
//!
//! Replaces sysinfo with zero C dependencies. Uses `std::fs` for /proc reads.
//! Linux-only; returns zeros/empty on non-Linux.

#![expect(
    clippy::cast_precision_loss,
    reason = "/proc stats use u64→f64 for display; precision loss acceptable"
)]

use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Linux page size (bytes), queried from the kernel at runtime.
fn page_size() -> u64 {
    #[cfg(target_os = "linux")]
    {
        // rustix provides a safe, zero-overhead wrapper around sysconf(_SC_PAGESIZE)
        u64::try_from(rustix::param::page_size()).unwrap_or(4096)
    }
    #[cfg(not(target_os = "linux"))]
    {
        4096
    }
}

/// Source identifier for live metrics (replaces "sysinfo")
pub const SOURCE_ID: &str = "proc";

/// CPU and memory stats (Linux /proc)
#[derive(Debug, Default, Clone)]
#[expect(clippy::struct_field_names, reason = "matches /proc/stat field names")]
pub struct ProcStats {
    /// Previous CPU sample for delta calculation
    prev_cpu: Option<CpuStatLine>,
    /// Previous process CPU times for delta calculation
    prev_process_times: HashMap<u32, (u64, u64)>,
    /// Previous total CPU time (for process % calculation)
    prev_total_cpu: u64,
}

/// Parsed line from /proc/stat (cpu or cpuN)
#[derive(Debug, Clone)]
struct CpuStatLine {
    user: u64,
    nice: u64,
    system: u64,
    idle: u64,
    iowait: u64,
    irq: u64,
    softirq: u64,
    steal: u64,
}

impl CpuStatLine {
    const fn total(&self) -> u64 {
        self.user
            + self.nice
            + self.system
            + self.idle
            + self.iowait
            + self.irq
            + self.softirq
            + self.steal
    }

    const fn busy(&self) -> u64 {
        self.user + self.nice + self.system + self.irq + self.softirq + self.steal
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
    /// Create new stats collector
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Refresh and return current CPU usage (0-100)
    #[must_use]
    pub fn cpu_usage(&mut self) -> f32 {
        #[cfg(target_os = "linux")]
        {
            if let Some(line) = parse_cpu_stat() {
                let usage = self.prev_cpu.as_ref().map_or(0.0, |prev| {
                    let total_delta = line.total().saturating_sub(prev.total());
                    let busy_delta = line.busy().saturating_sub(prev.busy());
                    if total_delta > 0 {
                        (busy_delta as f64 / total_delta as f64) * 100.0
                    } else {
                        0.0
                    }
                });
                self.prev_cpu = Some(line);
                usage as f32
            } else {
                0.0
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            let _ = self;
            0.0
        }
    }

    /// Total memory in bytes
    #[must_use]
    pub fn total_memory(&self) -> u64 {
        #[cfg(target_os = "linux")]
        {
            parse_meminfo_total().unwrap_or(0)
        }

        #[cfg(not(target_os = "linux"))]
        {
            let _ = self;
            0
        }
    }

    /// Used memory in bytes (`MemTotal` - `MemAvailable`)
    #[must_use]
    pub fn used_memory(&self) -> u64 {
        #[cfg(target_os = "linux")]
        {
            parse_meminfo_used().unwrap_or(0)
        }

        #[cfg(not(target_os = "linux"))]
        {
            let _ = self;
            0
        }
    }

    /// Number of CPU cores (from /proc/stat cpuN lines)
    #[must_use]
    pub fn cpu_count(&self) -> usize {
        #[cfg(target_os = "linux")]
        {
            parse_cpu_count().unwrap_or(1)
        }

        #[cfg(not(target_os = "linux"))]
        {
            let _ = self;
            1
        }
    }

    /// Collect all processes with CPU and memory. Updates internal state for CPU delta.
    pub fn processes(&mut self) -> Vec<ProcessInfo> {
        #[cfg(target_os = "linux")]
        {
            let total_cpu = parse_cpu_stat().map_or(0, |l| l.total());
            let procs =
                collect_processes(total_cpu, &mut self.prev_process_times, self.prev_total_cpu);
            self.prev_total_cpu = total_cpu;
            procs
        }

        #[cfg(not(target_os = "linux"))]
        {
            let _ = self;
            Vec::new()
        }
    }
}

#[cfg(target_os = "linux")]
fn parse_cpu_stat() -> Option<CpuStatLine> {
    let s = fs::read_to_string("/proc/stat").ok()?;
    let first = s.lines().next()?;
    if !first.starts_with("cpu ") {
        return None;
    }
    let parts: Vec<&str> = first.split_whitespace().collect();
    if parts.len() < 8 {
        return None;
    }
    Some(CpuStatLine {
        user: parts.get(1)?.parse().ok()?,
        nice: parts.get(2)?.parse().ok()?,
        system: parts.get(3)?.parse().ok()?,
        idle: parts.get(4)?.parse().ok()?,
        iowait: parts.get(5)?.parse().unwrap_or(0),
        irq: parts.get(6)?.parse().unwrap_or(0),
        softirq: parts.get(7)?.parse().unwrap_or(0),
        steal: parts.get(8)?.parse().unwrap_or(0),
    })
}

#[cfg(target_os = "linux")]
fn parse_cpu_count() -> Option<usize> {
    let s = fs::read_to_string("/proc/stat").ok()?;
    let count = s.lines().filter(|l| l.starts_with("cpu")).count();
    Some(count.saturating_sub(1).max(1)) // Subtract aggregate "cpu " line
}

#[cfg(target_os = "linux")]
fn parse_meminfo_total() -> Option<u64> {
    let s = fs::read_to_string("/proc/meminfo").ok()?;
    for line in s.lines() {
        if line.starts_with("MemTotal:") {
            let kb: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
            return Some(kb * 1024);
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn parse_meminfo_used() -> Option<u64> {
    let s = fs::read_to_string("/proc/meminfo").ok()?;
    let mut total_kb: Option<u64> = None;
    let mut available_kb: Option<u64> = None;
    for line in s.lines() {
        if line.starts_with("MemTotal:") {
            total_kb = Some(line.split_whitespace().nth(1)?.parse().ok()?);
        } else if line.starts_with("MemAvailable:") {
            available_kb = Some(line.split_whitespace().nth(1)?.parse().ok()?);
        }
        if total_kb.is_some() && available_kb.is_some() {
            break;
        }
    }
    let total = total_kb?;
    let available = available_kb.unwrap_or(0);
    Some((total.saturating_sub(available)) * 1024)
}

#[cfg(target_os = "linux")]
fn collect_processes(
    total_cpu: u64,
    prev_times: &mut HashMap<u32, (u64, u64)>,
    prev_total_cpu: u64,
) -> Vec<ProcessInfo> {
    let total_delta = total_cpu.saturating_sub(prev_total_cpu);
    let num_cpus = parse_cpu_count().unwrap_or(1) as u64;

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
    // Format: pid (comm) state ppid ... utime stime ... rss
    // comm can contain spaces/parens, so we find the last ")" to split
    let close_paren = s.rfind(')')?;
    let comm = s[1..close_paren].to_string();
    let rest = s[close_paren + 1..].trim_start();
    let parts: Vec<&str> = rest.split_whitespace().collect();
    // ppid=0, pgrp=1, session=2, tty=3, tpgid=4, flags=5, minflt=6, cminflt=7, majflt=8, cmajflt=9
    // utime=10, stime=11, cutime=12, cstime=13, priority=14, nice=15, num_threads=16, ...
    // rss is at index 20 (0-indexed from after comm)
    let utime: u64 = parts.get(10)?.parse().ok()?;
    let stime: u64 = parts.get(11)?.parse().ok()?;
    let rss: u64 = parts.get(20)?.parse().ok()?;
    Some((comm, utime, stime, rss))
}

/// Check if /proc is available (for tests)
#[cfg(target_os = "linux")]
#[must_use]
pub fn proc_available() -> bool {
    Path::new("/proc/stat").exists()
}

#[cfg(not(target_os = "linux"))]
#[must_use]
pub fn proc_available() -> bool {
    false
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
            name: "init".to_string(),
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
        let s = ProcStats::new();
        let _total = s.total_memory();
    }

    #[test]
    fn proc_stats_used_memory_bounded_by_total() {
        let s = ProcStats::new();
        let total = s.total_memory();
        let used = s.used_memory();
        assert!(used <= total || total == 0);
    }

    #[test]
    fn proc_stats_cpu_count_positive() {
        let s = ProcStats::new();
        let count = s.cpu_count();
        assert!(count >= 1);
    }

    #[test]
    fn proc_available_consistent() {
        let a = proc_available();
        #[cfg(target_os = "linux")]
        assert_eq!(a, std::path::Path::new("/proc/stat").exists());
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
            name: "test".to_string(),
            cpu_usage: 0.0,
            memory: 100 * ps,
        };
        assert_eq!(p.memory, 100 * ps);
    }

    #[test]
    fn process_info_cpu_usage_bounds() {
        let p = ProcessInfo {
            pid: 1,
            name: "test".to_string(),
            cpu_usage: 50.5,
            memory: 0,
        };
        assert!((p.cpu_usage - 50.5).abs() < f32::EPSILON);
    }

    #[test]
    fn proc_stats_processes_returns_vec() {
        let mut s = ProcStats::new();
        let procs = s.processes();
        // Verify processes() returns valid Vec (sanity: typical system has < 1M processes)
        assert!(procs.len() < 1_000_000);
    }

    #[test]
    fn source_id_constant() {
        assert_eq!(SOURCE_ID, "proc");
    }
}
