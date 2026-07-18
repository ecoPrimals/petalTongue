# petalTongue — Ecosystem Status

**Wave**: 150h | **Date**: July 18, 2026 | **From**: petalTongue on eastGate

---

## Standing State

| Metric | Value |
|--------|-------|
| Version | 1.6.6 |
| Crates | 19 workspace members |
| Tests | 6,529 passing, 0 failures |
| Clippy | Zero warnings (pedantic + nursery) |
| Unsafe | Confined to `petal-tongue-platform/src/ffi.rs` (C-FFI boundary) |
| Cross-arch | x86_64-linux, aarch64-linux, aarch64-android, x86_64-windows |
| Files | All production files < 800 LOC |
| Edition | 2024 |

---

## Shipped Capabilities

### NUCLEUS Composition (P1 — COMPLETE)

petalTongue's `/ws` WebSocket JSON-RPC bridge is live and consumed by footPrint.
7 methods available. Both provider and consumer sides confirmed working.

### Phase 2: Abstraction Over Gating (REFERENCE)

petalTongue is the **reference implementation** for Phase 2 Silicon Atheism.

1. **`petal-tongue-platform` crate** — cdylib embedding layer with C-FFI,
   `PlatformLifecycle` trait, `EmbeddedRuntime`, WebSocket JSON-RPC bridge.

2. **`MeshTopologySource` trait** — runtime topology resolution abstraction.
   Static gate mesh data behind `offline-topology` feature.

3. **`PlatformMetrics` trait** — cross-platform system resource queries.
   `LinuxProcMetrics` (production), `StubMetrics` (other platforms).

4. **Deep debt elimination** — zero bare `unwrap()` in production (269 eliminated),
   zero `todo!`/`FIXME`/`HACK`, all mocks confined to `#[cfg(test)]`.

### Scene Unification — 2D-as-3D-slice (Wave 150h)

New architectural evolution making petalTongue a universal rendering engine
(narrative, scientific, geospatial, molecular).

| Component | Status |
|-----------|--------|
| `Transform3D` on `SceneNode` | SHIPPED |
| `Camera` + `Projection` types | SHIPPED |
| Orthographic default (2D compat) | SHIPPED |
| `SceneGraph::flatten_3d()` | SHIPPED |
| Grammar `with_z()` + Perspective3D camera | SHIPPED |
| 4×4 matrix composition | SHIPPED |

**Design**: 2D = orthographic camera at z=0. All non-breaking.

---

## Evolution Target (P2)

| Item | Priority | Notes |
|------|----------|-------|
| SVG renderer camera integration (Phase 3) | P2 | Viewport from camera projection |
| 3D geometry compilation (Phase 4) | P2 | Mesh3D, Sphere, Cylinder primitives → scene nodes |
| `eframe` as opt-in for server builds | P3 | Already feature-gated behind `ui` |

---

## Upstream Dependencies (what we need)

| From | Capability | Status |
|------|-----------|--------|
| songBird | `mesh.peers` IPC → live `MeshTopologySource` impl | Pattern ready |
| songBird | `PROXY_PATH` drawbridge routing | P2 |
| sporeGate ops | Deploy composition on gate | P2 |

---

## Downstream Consumers (what we provide)

| To | What | How |
|----|------|-----|
| footPrint | Chart rendering, topology viz, metrics | WebSocket JSON-RPC (`/ws`) — **WIRED** |
| esotericWebb | Interactive visualization via `ui.render` | WebSocket JSON-RPC |
| sporePrint | Static file serving | `petaltongue web --docroot` |
| primalSpring | Scenario validation, grammar tests | 29 JSON scenarios |
| Any primal | Visualization IPC | `visualization.render.*` over UDS/TCP |
| Mobile hosts | Embedded rendering | C-FFI via `petal-tongue-platform` |
| 3D consumers | Scene unification | `flatten_3d()` + Camera/Projection |

---

*Wave 150h: FULL NUCLEUS COMPOSITION WIRED. Scene unification Phase 1-2 shipped.
6,529 tests. All milestones GREEN. footPrint consumer confirmed. esotericWebb V21
using petalTongue rendering.*
