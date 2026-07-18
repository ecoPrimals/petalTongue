# Health Monitoring Trait — Ecosystem Pattern Handoff

**Wave**: 150g | **Date**: July 18, 2026 | **For**: ecosystem teams (bearDog, songBird, loamSpine, etc.)
**Status**: petalTongue reference implementation **SHIPPED**. Available for adoption.

---

## Problem

The ecosystem P2 demand signal identifies "Health monitoring trait — Not procfs-hardcoded"
as a remaining need. Direct `/proc` parsing ties health monitoring to Linux, breaking
Silicon Atheism Phase 2 (abstraction over gating).

## Solution: `PlatformMetrics` Trait

petalTongue ships a trait-based abstraction in `petal-tongue-core::platform_metrics`:

```rust
pub trait PlatformMetrics: Send + Sync {
    /// Refresh and return latest resource snapshot.
    fn snapshot(&mut self) -> ResourceSnapshot;

    /// Whether this implementation provides meaningful data.
    fn available(&self) -> bool;

    /// Platform-specific source identifier for telemetry tagging.
    fn source_id(&self) -> &'static str;
}

pub struct ResourceSnapshot {
    pub cpu_percent: f32,
    pub memory_total: u64,
    pub memory_used: u64,
    pub cpu_count: usize,
}
```

### Implementations

| Platform | Implementation | Strategy |
|----------|---------------|----------|
| Linux | `LinuxProcMetrics` | `/proc/stat`, `/proc/meminfo` |
| macOS | (future) | `host_statistics64`, `sysctl` |
| Windows | (future) | `GetSystemTimes`, `GlobalMemoryStatusEx` |
| Android | (future) | JNI `ActivityManager` or host injection |
| iOS | (future) | `mach_task_info` |
| WASM / stub | `StubMetrics` | Returns zeros, `available() = false` |

### Auto-Detection

```rust
use petal_tongue_core::platform_metrics;

let mut metrics = platform_metrics::detect(); // returns Box<dyn PlatformMetrics>
let snap = metrics.snapshot();
println!("CPU: {}%, Memory: {}/{}", snap.cpu_percent, snap.memory_used, snap.memory_total);
```

---

## Exposure Over JSON-RPC

The `pt.metrics` method exposes resource data over WebSocket (or any JSON-RPC transport):

```json
// Request
{"jsonrpc":"2.0","id":1,"method":"pt.metrics","params":{}}

// Response
{"jsonrpc":"2.0","id":1,"result":{
  "cpu_percent": 12.5,
  "memory_total": 16777216000,
  "memory_used": 8388608000,
  "memory_percent": 50.0,
  "cpu_count": 8,
  "source": "linux-proc"
}}
```

---

## Adoption Guide for Other Primals

### Option A: Depend on `petal-tongue-core` (simplest)

```toml
[dependencies]
petal-tongue-core = { git = "ssh://git@git.primals.eco:2222/ecoPrimals/petalTongue.git", default-features = false }
```

Then use `platform_metrics::detect()` to get a metrics provider.

### Option B: Copy the trait pattern (independent)

Copy the trait definition and `ResourceSnapshot` struct into your crate.
Implement platform-specific backends as needed.

### Option C: Consume via JSON-RPC (composition)

Connect to petalTongue's `/ws` endpoint and call `pt.metrics`.
No Rust dependency needed — works from any language.

---

## Integration with `primal-transport` (Future)

When the ecosystem publishes a shared `primal-transport` crate, the
`PlatformMetrics` trait should migrate there alongside `TransportEndpoint`.
This gives every primal standardized health reporting without coupling
to petalTongue's codebase.

---

*Wave 150g: petalTongue reference pattern for ecosystem health monitoring.
Trait-based, not procfs-hardcoded, with auto-detection and JSON-RPC exposure.
Available for immediate adoption by any primal team.*
