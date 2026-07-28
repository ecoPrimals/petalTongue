# petalTongue — Ecosystem Status

**Wave**: 155g | **Date**: July 28, 2026 | **From**: petalTongue on westGate
**Posture**: **STABLE** — deep debt evolution complete. Infrastructure ready.

---

## Standing State

| Metric | Value |
|--------|-------|
| Version | 1.7.0 |
| Crates | 19 workspace members |
| Tests | 6,605 passing, 0 failures |
| Clippy | Zero warnings (pedantic + nursery, all targets) |
| Unsafe | Confined to `petal-tongue-platform/src/ffi.rs` (15 usages, all SAFETY-documented) |
| BTSP | **13/13** (ClientHello + server-side, full strict mode) |
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
   `ManifestMeshTopology` loads from `ecosystem_manifest.toml`. Static data behind
   `offline-topology` feature (default off).

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

### Static Site Builder — Zola Replacement (sporePrint Pipeline ACTIVE)

Sovereignty Evolution Tier 1: replace Zola with petalTongue rendering.

| Component | Status |
|-----------|--------|
| `ContentSource` trait | SHIPPED |
| `SiteBuilder` + `SiteLayout` | SHIPPED |
| `InMemorySource` | SHIPPED |
| `FilesystemSource` (Zola-compatible .md scanner) | SHIPPED |
| `CasSource` (nestGate CAS hash resolution) | SHIPPED |
| WASM `build_site()` export | SHIPPED |
| WASM `render_page_with_layout()` export | SHIPPED |
| Dark/light responsive CSS | SHIPPED |
| Nav tree composition | SHIPPED |
| Search index generation | SHIPPED |
| Text preview extraction for search | SHIPPED |
| Entity shortcode resolution | SHIPPED |

**Next**: nestGate publishes `site-manifest.json` (Nest Atomic Phase 0).
`cellMembrane` serves `StaticSite` output files. Full Zola replacement end-to-end.

---

## Evolution Target (P2)

| Item | Priority | Notes |
|------|----------|-------|
| `eframe` as opt-in for server builds | P3 | Already feature-gated behind `ui` |
| Live `MeshTopologySource` via songBird | P2 | Trait ready, awaiting songBird adapter |
| Nest Atomic CAS integration | P2 | `CasSource` ready, blocked on G3 |

---

## Glacial Goal Relevance

petalTongue owns **no glacial goals directly** but supports:

| Goal | How petalTongue Helps |
|------|----------------------|
| G3 (Nest Atomic Phase 0) | `CasSource` ready to consume `site-manifest.json` |
| G5 (Chimera) | `petal-tongue-platform` is the embedding pattern reference |
| G7 (Gate enmeshment) | WASM genomeBin target enables cross-platform |
| G9 (JOSS publication) | Visualization rendering for Gonzales NF paper figures |

---

## Upstream Dependencies (what we need)

| From | Capability | Status |
|------|-----------|--------|
| nestGate | `site-manifest.json` in CAS dataset (for CasSource) | Awaiting Nest Atomic Phase 0 |
| cellMembrane | Serve `StaticSite` output (path → bytes) | Pattern ready |
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
| sporePrint | Static site generation + serving | `FilesystemSource` + `CasSource` + `petaltongue web --docroot` |
| primalSpring | Scenario validation, grammar tests | 30 JSON scenarios |
| Any primal | Visualization IPC | `visualization.render.*` over UDS/TCP |
| Mobile hosts | Embedded rendering | C-FFI via `petal-tongue-platform` |
| 3D consumers | Scene unification | `flatten_3d()` + Camera/Projection |
| Browser clients | WebGL rendering | `pt.render_webgl` JSON-RPC method |

---

*Wave 155g: petalTongue STABLE. BTSP 13/13. 6,605 tests. Deep debt evolution
complete — topology evolved to runtime manifest discovery, large files refactored,
production stubs wired, FFI hardened, all hardcoding eliminated. Infrastructure
ready for G3 (Nest Atomic CAS), G5 (Chimera embedding pattern), G7 (cross-platform
via WASM/platform crate). All P0/P1 audit items resolved. Remaining backlog is
P2+ ecosystem dependencies.*
