# petalTongue — Ecosystem Status

**Wave**: 157e | **Date**: August 10, 2026 | **From**: petalTongue on eastGate
**Posture**: **ANT COLONY ACTIVE** — Gossip injection module ships. petalTongue announces `surface.web.live`, `surface.scene.streaming`, `content.serve.available`, and `viz.session.*` to swarmVine mesh. WebGL auto-publish wired: `visualization.render.grammar` with GPU modality pushes compiled scenes through `/ws/scene` broadcast. Centralized swarmVine socket discovery. G19 pipeline complete end-to-end. Zero P0/P1.

---

## Standing State

| Metric | Value |
|--------|-------|
| Version | 1.7.0 |
| Crates | 19 workspace members |
| Tests | 6,644 passing, 0 failures |
| Clippy | Zero warnings (pedantic + nursery, all targets) |
| Docs | Zero warnings (`cargo doc --workspace --all-features --no-deps`) |
| Unsafe | Confined to `petal-tongue-platform/src/ffi.rs` (15 usages, all SAFETY-documented) |
| BTSP | **13/13** (ClientHello + server-side, full strict mode) |
| Cross-arch | x86_64-linux, aarch64-linux, aarch64-android, **x86_64-windows (ZERO warnings)** |
| Files | All production files < 800 LOC |
| Edition | 2024 |
| tarpc | **0.37** — C1 DONE, C2 LIVE, **G65 DONE** (negotiate server on `petaltongue.negotiate.sock`) |
| tarpc UDS | `petaltongue.tarpc.sock` — binary RPC alongside JSON-RPC `.sock` |
| tokio-serde | 0.9 (aligned with tarpc 0.37) |
| Idioms | Rust 2024 let-chains, zero redundant allocations, all deps current |
| Self-knowledge | Zero hardcoded peer primal names in production code |
| Config unity | Zero raw `FAMILY_ID` env reads outside canonical resolution |
| TCP bind | Localhost-only by default (`--bind 0.0.0.0` for network exposure) |
| G19 | **COMPLETE** — scene stream + compilation bridge + auto-publish. ironGate (RTX 5070) exp006 22/22 PASS |
| Gossip | **ACTIVE** — 4 injection points: `surface.web.live`, `scene.streaming`, `content.serve`, `viz.session.*` |
| nestgate.io | **LIVE** — petalTongue v1.7.0 serving on sporeGate via WG mesh |
| Declarative scenes | `visualization.render.scene` accepts string scene type + data for tideGlass |
| Dashboard UX | Graceful degradation with specific error messages (no infinite "Loading...") |

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
| Live `MeshTopologySource` via mesh routing capability | P2 | Trait ready, awaiting mesh provider adapter |
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
| nestGate | nestgate.io content backend wiring (4 DIVs identified) | DIVs documented |
| cellMembrane | Serve `StaticSite` output (path → bytes) | Pattern ready |
| songBird | `mesh.peers` IPC → live `MeshTopologySource` impl | Pattern ready |
| sporeGate ops | Deploy composition + dnsmasq for primal.eco (inner membrane) | P2 |

---

## Downstream Consumers (what we provide)

| To | What | How |
|----|------|-----|
| footPrint | Chart rendering, topology viz, metrics | WebSocket JSON-RPC (`/ws`) — **WIRED** |
| esotericWebb | Interactive visualization via `ui.render` | WebSocket JSON-RPC |
| bingoCube | ColorGrid commitment rendering | `pt.render_webgl` → WebGL draw commands |
| sporePrint | Static site generation + serving | `FilesystemSource` + `CasSource` + `petaltongue web --docroot` |
| primalSpring | Scenario validation, grammar tests | 30 JSON scenarios |
| tideGlass | 5 science viz scenes (declarative passthrough) | `visualization.render.scene` with string scene type |
| Any primal | Visualization IPC | `visualization.render.*` over UDS/TCP |
| Mobile hosts | Embedded rendering | C-FFI via `petal-tongue-platform` |
| 3D consumers | Scene unification | `flatten_3d()` + Camera/Projection |
| Browser clients | WebGL rendering | `pt.render_webgl` JSON-RPC method |

---

*Wave 156m (Aug 6): G65 IMPLEMENTED independently from squirrel reference pattern.
`protocol_negotiation` module: `ProtocolId`, `ProtocolRequest/Response`, wire format,
`negotiate_client/server`, `NegotiateServer` on `petaltongue.negotiate.sock`.
29 new tests (+6,644 total). C2 dual-socket retained for backward compat.
Server mode: three concurrent listeners (JSON-RPC .sock + tarpc .tarpc.sock + G65 .negotiate.sock).
Zero clippy pedantic+nursery. Zero doc warnings. Zero P0/P1/P2.*
