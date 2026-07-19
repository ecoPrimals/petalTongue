// SPDX-License-Identifier: AGPL-3.0-or-later
//! Platform metrics trait — abstract system resource queries.
//!
//! Phase 2 abstraction over `#[cfg(target_os)]` gating. Instead of
//! conditional compilation in every call site, implementations of
//! [`PlatformMetrics`] provide platform-appropriate resource data:
//!
//! - **Linux desktop**: reads `/proc/stat`, `/proc/meminfo` — see [`LinuxProcMetrics`]
//! - **Android**: queries `ActivityManager` via JNI (future)
//! - **iOS**: queries `mach_task_info` (future)
//! - **Embedded/WASM**: returns static stubs or host-injected values — see [`StubMetrics`]
//!
//! This trait lives in `petal-tongue-core` so that both `petal-tongue-ui`
//! (desktop) and `petal-tongue-platform` (mobile) can implement it.

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "linux")]
pub use linux::LinuxProcMetrics;

/// System resource snapshot — the universal output of any [`PlatformMetrics`] impl.
#[derive(Debug, Clone, Default)]
pub struct ResourceSnapshot {
    /// CPU usage percentage (0.0–100.0), averaged across all cores.
    pub cpu_percent: f32,
    /// Total physical memory in bytes.
    pub memory_total: u64,
    /// Used physical memory in bytes.
    pub memory_used: u64,
    /// Number of CPU cores/threads available.
    pub cpu_count: usize,
}

impl ResourceSnapshot {
    /// Memory usage as a percentage (0.0–100.0).
    #[must_use]
    #[expect(
        clippy::cast_precision_loss,
        reason = "memory values fit well within f64 mantissa range in practice"
    )]
    pub fn memory_percent(&self) -> f32 {
        if self.memory_total == 0 {
            return 0.0;
        }
        #[expect(
            clippy::cast_possible_truncation,
            reason = "percentage always fits f32"
        )]
        let pct = (self.memory_used as f64 / self.memory_total as f64 * 100.0) as f32;
        pct
    }
}

/// Trait for platform-specific resource metric collection.
///
/// Implementations MUST be lightweight — they are polled at UI refresh rate
/// (typically 1–10 Hz). Heavy operations should be cached internally.
///
/// # Platform Implementations
///
/// | Platform | Strategy |
/// |----------|----------|
/// | Linux | `/proc/stat`, `/proc/meminfo` |
/// | macOS | `host_statistics64`, `sysctl` |
/// | Windows | `GetSystemTimes`, `GlobalMemoryStatusEx` |
/// | Android | Host injects via `on_configuration_changed` or JNI `ActivityManager` |
/// | iOS | `mach_task_info` |
/// | WASM | Stub (no OS access) |
pub trait PlatformMetrics: Send + Sync {
    /// Refresh internal state and return the latest resource snapshot.
    ///
    /// Implementations should delta-compute CPU usage from the previous call.
    fn snapshot(&mut self) -> ResourceSnapshot;

    /// Whether this implementation can provide meaningful data.
    ///
    /// Returns `false` on platforms where no metrics are available (e.g. WASM).
    fn available(&self) -> bool;

    /// Platform-specific source identifier for telemetry tagging.
    fn source_id(&self) -> &'static str;
}

/// Stub implementation that always returns zeros.
///
/// Used on platforms without native metric access (WASM, some embedded targets)
/// or as a fallback when the real provider fails to initialize.
#[derive(Debug, Default)]
pub struct StubMetrics;

impl PlatformMetrics for StubMetrics {
    fn snapshot(&mut self) -> ResourceSnapshot {
        ResourceSnapshot {
            cpu_count: std::thread::available_parallelism()
                .map(std::num::NonZero::get)
                .unwrap_or(1),
            ..ResourceSnapshot::default()
        }
    }

    fn available(&self) -> bool {
        false
    }

    fn source_id(&self) -> &'static str {
        "stub"
    }
}

/// Create the platform-appropriate metrics provider.
///
/// Returns [`LinuxProcMetrics`] on Linux, [`StubMetrics`] elsewhere.
/// This is the recommended entry point for code that needs metrics
/// without caring about the underlying platform.
#[must_use]
pub fn detect() -> Box<dyn PlatformMetrics> {
    #[cfg(target_os = "linux")]
    {
        Box::new(LinuxProcMetrics::new())
    }

    #[cfg(not(target_os = "linux"))]
    {
        Box::new(StubMetrics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_returns_provider() {
        let mut m = detect();
        let snap = m.snapshot();
        assert!(snap.cpu_count >= 1);
    }

    #[test]
    fn stub_metrics_not_available() {
        let stub = StubMetrics;
        assert!(!stub.available());
        assert_eq!(stub.source_id(), "stub");
    }

    #[test]
    fn stub_metrics_snapshot_defaults() {
        let mut stub = StubMetrics;
        let snap = stub.snapshot();
        assert_eq!(snap.cpu_percent, 0.0);
        assert_eq!(snap.memory_total, 0);
        assert_eq!(snap.memory_used, 0);
        assert!(snap.cpu_count >= 1);
    }

    #[test]
    fn resource_snapshot_memory_percent() {
        let snap = ResourceSnapshot {
            cpu_percent: 0.0,
            memory_total: 1000,
            memory_used: 250,
            cpu_count: 4,
        };
        let pct = snap.memory_percent();
        assert!((pct - 25.0).abs() < 0.01);
    }

    #[test]
    fn resource_snapshot_memory_percent_zero_total() {
        let snap = ResourceSnapshot::default();
        assert_eq!(snap.memory_percent(), 0.0);
    }
}
