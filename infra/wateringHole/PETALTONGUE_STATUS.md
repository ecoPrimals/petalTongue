# petalTongue — Ecosystem Status

**Wave**: 150o | **Date**: July 20, 2026 | **From**: petalTongue on eastGate

---

## Standing State

| Metric | Value |
|--------|-------|
| Version | 1.7.0 |
| Crates | 19 workspace members |
| Tests | 5,800+ passing, 0 failures |
| Clippy | Zero warnings (pedantic + nursery, all targets) |
| Unsafe | Confined to `petal-tongue-platform/src/ffi.rs` (15 usages, all SAFETY-documented) |
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

### Scene Unification — 2D-as-3D-slice (ALL PHASES COMPLETE)

Universal rendering engine (narrative, scientific, geospatial, molecular).
All 4 phases shipped and tested.

| Component | Status |
|-----------|--------|
| `Transform3D` on `SceneNode` | SHIPPED |
| `Camera` + `Projection` types | SHIPPED |
| Orthographic default (2D compat) | SHIPPED |
| `SceneGraph::flatten_3d()` | SHIPPED |
| Grammar `with_z()` + Perspective3D camera | SHIPPED |
| SVG viewport from camera projection | SHIPPED |
| Sphere mesh generation (UV tessellation) | SHIPPED |
| Cylinder mesh generation (ring tessellation) | SHIPPED |
| Mesh3D passthrough (vertex/index data) | SHIPPED |
| Ribbon (confidence band polygon) | SHIPPED |
| ErrorBar geometry (whisker + caps) | SHIPPED |
| Text geometry (positioned labels) | SHIPPED |
| 4×4 matrix composition | SHIPPED |
| **WebGL modality compiler** | SHIPPED |

**Design**: 2D = orthographic camera at z=0. All non-breaking.

### bingoCube Widget Integration (SHIPPED)

`DataBinding::ColorGrid` — pre-computed RGBA color grids with progressive reveal.
Used by bingoCube's crypto commitment visualization on primals.eco.
Exposed via `pt.render_webgl` JSON-RPC method for browser-side WebGL rendering.

### WebGL Rendering Pipeline (SHIPPED)

`WebGlCompiler` — scene graph to GPU draw commands (vertex arrays + index buffers + draw calls).
Supports 2D primitives (projected to clip space) and 3D meshes (camera view-projection).
Available via JSON-RPC `pt.render_webgl` method on the `/ws` bridge.

---

## Evolution Target (P2)

| Item | Priority | Notes |
|------|----------|-------|
| `eframe` as opt-in for server builds | P3 | Already feature-gated behind `ui` |
| Live `MeshTopologySource` via songBird | P2 | Trait ready, awaiting songBird adapter |

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
| bingoCube | ColorGrid commitment rendering | `pt.render_webgl` → WebGL draw commands |
| sporePrint | Static file serving | `petaltongue web --docroot` |
| primalSpring | Scenario validation, grammar tests | 30 JSON scenarios |
| Any primal | Visualization IPC | `visualization.render.*` over UDS/TCP |
| Mobile hosts | Embedded rendering | C-FFI via `petal-tongue-platform` |
| 3D consumers | Scene unification | `flatten_3d()` + Camera/Projection |
| Browser clients | WebGL rendering | `pt.render_webgl` JSON-RPC method |

---

*Wave 150o: WebGL modality compiler SHIPPED. bingoCube ColorGrid integration COMPLETE.
FFI safety documentation added. Deep debt lint warnings eliminated (0 warnings, all targets).
5,800+ tests. All milestones GREEN. Ready for deployment to flockGate.*
