# petalTongue — Ecosystem Status

**Wave**: 142b | **Date**: July 16, 2026 | **From**: petalTongue on eastGate

---

## Standing State

| Metric | Value |
|--------|-------|
| Version | 1.6.6 |
| Crates | 19 workspace members |
| Tests | 6,511 passing, 0 failures |
| Clippy | Zero warnings (pedantic + nursery) |
| Unsafe | Confined to `petal-tongue-platform/src/ffi.rs` (C-FFI boundary) |
| Cross-arch | x86_64-linux, aarch64-linux, aarch64-android, x86_64-windows |
| Files | All production files < 800 LOC |
| Edition | 2024 |

---

## Phase 2: Abstraction Over Gating

petalTongue is the **reference implementation** for Phase 2 Silicon Atheism.

### What shipped

1. **`petal-tongue-platform` crate** — cdylib embedding layer with C-FFI,
   `PlatformLifecycle` trait, `EmbeddedRuntime`, WebSocket JSON-RPC bridge.
   This is the pattern for other primals targeting Android/iOS embedding.

2. **`MeshTopologySource` trait** — runtime topology resolution abstraction.
   Static gate mesh data now behind `offline-topology` feature. Production
   code written against the trait, not statics.

3. **`PlatformMetrics` trait** — cross-platform system resource queries.
   `LinuxProcMetrics` (production), `StubMetrics` (other platforms).
   Pattern for platform-aware telemetry.

4. **Deep debt elimination** — zero bare `unwrap()` in production, zero
   `todo!`/`FIXME`/`HACK`, all mocks confined to `#[cfg(test)]`.

### The pattern for other primals

```
trait FooSource: Send + Sync {
    fn data(&self) -> Vec<&'static FooItem>;
}

#[cfg(feature = "offline-topology")]
struct StaticFoo;

#[cfg(feature = "offline-topology")]
impl FooSource for StaticFoo { ... }
```

Gate deployment-specific data behind features. Consumers work against the trait.
Live implementations query capabilities at runtime.

---

## Remaining Work (petalTongue local)

| Item | Priority | Notes |
|------|----------|-------|
| `main.rs` dispatch extraction | P3 | 727L — acceptable for UniBin entry point |
| `socket2` → `mdns` feature gate | P3 | Only needed for mDNS multicast |
| `eframe` as opt-in for server builds | P3 | Already feature-gated behind `ui` |
| CI lint: no `unsafe` outside `ffi.rs` | P3 | Workspace forbid already covers this |

---

## Upstream Dependencies (what we need)

| From | Capability | Status |
|------|-----------|--------|
| songBird | `mesh.peers` IPC → live `MeshTopologySource` impl | Pattern ready, needs songBird adapter |
| biomeOS | `gate.mesh.live` capability → live topology | Pattern ready |
| footPrint | WebSocket client → `PETALTONGUE_WS_PORT` | Bridge shipped, integration TODO |
| bearDog | `crypto.sign` delegation for scene signing | Design ready, env override exists |

---

## Downstream Consumers (what we provide)

| To | What | How |
|----|------|-----|
| footPrint | Chart rendering, topology viz | WebSocket JSON-RPC bridge |
| sporePrint | Static file serving | `petaltongue web --docroot` |
| primalSpring | Scenario validation, grammar tests | 29 JSON scenarios in `sandbox/scenarios/` |
| Any primal | Visualization IPC | `visualization.render.*` over UDS/TCP |
| Mobile hosts | Embedded rendering | C-FFI via `petal-tongue-platform` |

---

*Wave 142b: Phase 2 reference. Abstraction over gating. 6,511 tests. Clean.*
