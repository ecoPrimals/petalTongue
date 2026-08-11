# petalTongue — Context

**Version:** 1.7.0
**Role:** Universal User Interface primal (visualization, presentation, interaction, static site generation)
**License:** AGPL-3.0-or-later (scyBorg triple: AGPL + ORC + CC-BY-SA 4.0)

---

## What This Is

petalTongue is ecoPrimals' visualization and user interface primal. It translates
ecosystem state into every available modality — desktop GUI (egui), terminal TUI
(ratatui), web (axum), headless (SVG/PNG/JSON), and WASM. It implements a
Grammar of Graphics engine with a declarative scene graph and animation system.

petalTongue is a **meta-tier** primal: it presents data from other primals but
does not own computation, storage, or security domains.

## Architecture

18 workspace crates, single UniBin binary (`petaltongue`, 7 subcommands).
`offline-topology` feature gates static mesh topology (default off; enable only for offline demos).
Production code uses `MeshTopologySource` trait with `ManifestMeshTopology` for runtime discovery:

| Crate | Purpose |
|-------|---------|
| `petal-tongue-core` | Types, config, sensory discovery, capability registry |
| `petal-tongue-ipc` | JSON-RPC 2.0 server (UDS + TCP), BTSP, push delivery |
| `petal-tongue-scene` | Declarative scene graph, modality compilers |
| `petal-tongue-graph` | Chart rendering, sonification |
| `petal-tongue-animation` | Manim-style animation system |
| `petal-tongue-platform` | Platform embedding layer (Android/iOS cdylib, C-FFI, lifecycle) |
| `petal-tongue-ui` | Native GUI (egui/eframe), feature-gated |
| `petal-tongue-tui` | Terminal UI (ratatui) |
| `petal-tongue-ui-core` | Pure Rust abstract UI (text, SVG, canvas) |
| `petal-tongue-discovery` | Primal/capability discovery clients |
| `petal-tongue-cli` | CLI handler logic |
| `petal-tongue-api` | BiomeOS client, HTTP APIs |
| `petal-tongue-entropy` | Human entropy capture |
| `petal-tongue-adapters` | Adapter framework |
| `petal-tongue-headless` | Headless rendering binary |
| `petal-tongue-types` | WASM-portable data types |
| `petal-tongue-wasm` | Browser rendering module |
| `doom-core` | Doom WAD rendering (platform stress test) |

## IPC Surface

JSON-RPC 2.0 over Unix domain sockets (primary) and TCP (`--port`).
55 methods across domains: `visualization.*` (render, stream, grammar, dashboard,
scene, export, validate, session, texture, introspect, panels, showing, dismiss),
`interaction.*`, `health.*`, `capabilities.*`, `capability.*`, `identity.*`,
`ui.*`, `motor.*`, `audio.*`, `lifecycle.*`, `topology.get`,
`proprioception.get`, `provider.register_capability`, `auth.*`, `btsp.*`,
`primal.announce`.

BTSP Phase 1 complete: family-scoped socket naming, insecure guard,
domain symlinks (`visualization.sock`). BTSP Phase 2 complete: security
provider handshake delegation on both UDS and TCP, length-prefixed and
JSON-line framing, `btsp.session.create`, `btsp.session.verify`, and
`btsp.negotiate` via provider client. BTSP Phase 3 complete:
ChaCha20-Poly1305 AEAD encrypted frame I/O after negotiate; HKDF-SHA256
directional key derivation; 13/13 ecosystem parity. **BTSP ClientHello**
(Wave 151c): client-side 4-step handshake for authenticated outbound
connections (HMAC-SHA256 challenge-response). 13/13 primals compliant.

JH-0 MethodGate: pre-dispatch authorization on all JSON-RPC calls.
Public methods (`health.*`, `identity.get`, `capabilities.list`,
`lifecycle.status`, `auth.*`) always pass. Protected methods
(`visualization.*`, `interaction.*`, `ui.*`, `motor.*`, `audio.*`)
require a bearer token in enforced mode. Default: permissive.
Env: `PETALTONGUE_AUTH_MODE=enforced`. Auth introspection:
`auth.check`, `auth.mode`, `auth.peer_info`.

## Key Design Decisions

- **Two-dimensional universality**: universal across modalities (what you see)
  and substrates (what you run on).
- **Grammar of Graphics**: primals send grammar expressions, petalTongue
  compiles to best available representation.
- **No self-compute**: heavy work (GPU, physics) delegated via IPC to
  compute, display, and ledger capability providers. petalTongue discovers by capability.
- **Feature-gated GUI**: `ui` feature (default) pulls egui/eframe/glow.
  `tui` feature (default) pulls ratatui/crossterm.
  Headless builds (`--no-default-features`) have zero native display deps.
- **Audio discovery**: tiered backends — ecosystem primal (Tier 1, via
  capability discovery), socket, direct, software, silent. Socket/direct
  behind optional features; software/silent always available.

## UUI Boundary — Owns vs Leverages

petalTongue is the UUI engine: pure Rust rendering to any modality on any
device. Other primals own platform interaction points.

**Owns (pure Rust, in-crate):**
- egui (layout/interaction), epaint (tessellation), tiny-skia (rasterization)
- crossterm (terminal I/O), symphonia (audio decode/synthesis)
- Grammar of Graphics, scene graph, animation, modality adapters
- IPC server: `visualization.*`, `interaction.*`, `capabilities.sensory.*`

**Leverages (ecosystem primals via `capability.call` / JSON-RPC over UDS):**
- `display.*` — display capability provider (window lifecycle, frame presentation)
- `compute.*` / `math.*` — compute capability provider (GPU dispatch)
- `btsp.session.*` — security provider (transport security)
- `discovery.*` / `ipc.*` — discovery + registry providers (routing)
- TLS/HTTPS — TLS capability provider relay (design ready)
- `audio.play` / `audio.stream` — audio capability provider (stub, Tier 1)
- `storage.put` / `storage.get` — storage capability provider (future)
- `ai.query` / `ai.complete` — AI capability provider (future)

The eframe/glow C/FFI stack exists only behind `ui-eframe` feature as a
development convenience. The architectural path is EguiPixelRenderer →
DisplayManager → ecosystem `display.*` IPC.

## Ecosystem Position

petalTongue discovers other primals at runtime via capability-based IPC.
It has zero compile-time knowledge of primal identities in production builds
(fixture data gated behind `#[cfg(test)]` or `test-fixtures` feature).

Coordinates with biomeOS (orchestration) and any primal that exposes
security, registry, or visualization-relevant
capabilities.

## Build

```bash
cargo build --release                     # Full binary (26M musl-static)
cargo build --release --no-default-features  # Headless only
cargo test --workspace --all-features     # 6,755 workspace tests, ~85-90% coverage
```

## Current State

Wave 156m — G65 PROTOCOL NEGOTIATION IMPLEMENTED (August 6, 2026).

petalTongue is **G65-operational**. Three-socket transitional pattern deployed:
- `petaltongue.sock` (JSON-RPC, universal — C2 Phase 2)
- `petaltongue.tarpc.sock` (binary bincode — C2 Phase 2)
- `petaltongue.negotiate.sock` (G65 Phase 3 — single-socket protocol negotiation)

tarpc 0.37 (C1 DONE). Server mode spawns all three listeners concurrently via
`tokio::select!`. G19 live render pipeline PROVEN on ironGate.
All P1 and P2 items resolved. Zero P0/P1/P2. Self-knowledge enforced.
Declarative scene passthrough LIVE (O6 DONE).

**G65 (Phase 3)**: Independently implemented from squirrel reference pattern.
Client sends `PROTOCOLS: tarpc,jsonrpc\n`, server selects best match.
No negotiation header = JSON-RPC (backward-compatible).
Transitional: C2 sockets remain for legacy clients. Once ecosystem adopts G65,
dual-socket paths deprecate → single `petaltongue.negotiate.sock` becomes canonical.

**BTSP**: 13/13 strict mode (ClientHello + server-side). Wired into outbound
`primal.announce` and `content.resolve` connections (HMAC-SHA256).

**sporePrint Pipeline**: `FilesystemSource` + `CasSource` implementing
`ContentSource` trait. Full Zola replacement backend wired. Awaits Nest Atomic
Phase 0 (G3) for live CAS integration.

**Deep Debt**: Zero `todo!`/`FIXME`/`HACK`, zero production `unwrap()`, zero
unsafe (except necessary FFI). Attribute-based mesh topology.

**Scene Unification**: ALL 4 PHASES COMPLETE. Universal 2D-as-3D-slice rendering
(Transform3D, Camera/Projection, flatten_3d, Sphere/Cylinder/Mesh3D/Ribbon/ErrorBar/Text).

**WebGL Pipeline**: WebGlCompiler compiles scene graphs to GPU-ready vertex/index
buffers. Exposed via `pt.render_webgl` JSON-RPC and WASM `render_binding_webgl()`.

**Static Site Builder**: SiteBuilder + ContentSource trait + SiteLayout composition.
Foundation for Zola replacement (Sovereignty Evolution Tier 1). WASM exports:
`build_site()`, `render_page_with_layout()`.

**bingoCube Integration**: ColorGrid DataBinding + `render_color_grid_webgl()` WASM
export for browser-side commitment grid rendering.

**Platform Embedding** (`petal-tongue-platform`): `cdylib` + `rlib` for
Android/iOS/desktop host apps. C-FFI surface with SAFETY-documented unsafe blocks.
WebSocket JSON-RPC bridge on `/ws` (port 8080) and standalone (port 8765).

**Deep debt**: Zero production `unwrap()`, zero TODO/FIXME/HACK, zero clippy warnings
(pedantic+nursery, all targets), zero doc warnings, all files <800 LOC, 6,755 tests passing.

**Cross-architecture**: x86_64-linux, aarch64-linux, aarch64-android, x86_64-windows.

### Evolution Timeline (condensed)

| Wave/Date | Milestone |
|-----------|-----------|
| 156d (Aug 4) | STABLE — TCP bind hardened, family ID unified, nestgate.io LIVE on mesh, K-derm 3/3 |
| 156b (Aug 4) | STABLE — G19 PROVEN, self-knowledge enforced (zero peer names in prod), deps current |
| 155m (Jul 30) | STABLE — Modern idiom pass: let-chains, zero redundant alloc, deps current, test race fixed |
| 155k (Jul 30) | STABLE — Provenance 7/7, P2 divergences fixed, 6,755 tests, zero clippy pedantic+nursery |
| 155b (Jul 27) | genomeBin convergence, tracks converged |
| 151c (Jul 26) | BTSP ClientHello shipped (12/13), attribute-based mesh topology, 6,589 tests |
| 151b (Jul 26) | sporePrint pipeline: FilesystemSource + CasSource shipped |
| 150t (Jul 21) | Sovereignty Evolution: SiteBuilder, WebGL WASM, bingoCube ColorGrid |
| 150i (Jul 20) | Scene unification ALL 4 PHASES, v1.7.0 deployed to golgiBody depot |
| 150g (Jul 19) | `/ws` WebSocket JSON-RPC bridge on port 8080 — LAST P1 resolved |
| 145b (Jul 16) | Phase 2 14/14, CAC 6/6, Glacial 8/8 — all milestones achieved |
| 141b (Jul 15) | `petal-tongue-platform` crate: PlatformLifecycle, C-FFI, cdylib |
| 141a (Jul 15) | Cross-architecture transport: Windows Named Pipes + Android NDK |
| 140a (Jul 15) | Tangibles Pivot: Gonzales refactoring, manifest-driven handlers |
| 137b (Jul 13) | Neural API live topology visualization, SSE typed events |
| 136b (Jul 11) | K-Derm diderm topology, DNSSEC, 8/8 stadial criteria clear |
| 132d-f (Jul 4-5) | Tower Atomic, grapheneGate, coordination backend |
| 124 (Jun 22) | GPU Compute Topology, ecosystem manifest |
| 116 (Jun 19) | Gate Mesh, HEALTH-01 compliance, TransportEndpoint |
| 76 (Jun 3) | TRUE PRIMAL sweep, typed errors, zero literal `.to_string()` |
| 69 (Jun 2) | dep trimming, tarpc/unix removed, thiserror modernization |
| 61 (May 29) | DH-1 /tmp cleanup, mock isolation, sensory discovery |
| Wave 47+ (May) | BTSP Phase 3 (ChaCha20-Poly1305), TRUE PRIMAL name evolution, Phase 58/60 audits |
| Sprint 7-8 (Apr) | dyn elimination (22 traits), async-trait removal (RPITIT), 14-module refactoring |
| Stadial gate (Apr 17) | Interstadial exit cleared — all CI gates pass, zero debt markers |

Full per-wave changelogs preserved in git history and `infra/wateringHole/` AARs.

### Remaining Backlog

- Audio backend wire protocols (via `audio.play` capability discovery)
- `GpuCompiler` → coralReef → toadStool wiring for native WGSL shader execution
- `egui-wgpu` backend alongside glow (enables custom render passes for science viz)
- sporePrint full deployment: cellMembrane serving of petalTongue-generated StaticSite
- Nest Atomic CAS integration: live `CasSource` connected to nestGate (blocked on G3)
- G53 maturation: footPrint GPS visualization + esotericWebb lattice viz via petalTongue

All items are P2+ strategic evolution paths. None are blockers for petalTongue's
stable operation. G19 is PROVEN — the live render pipeline works on downstream hosts.

## Stadial Readiness (May 17, 2026)

**Gate status**: 9.5/10 — interstadial exit CLEARED.

### Method Stability Tiers

| Tier | Methods | Meaning |
|------|---------|---------|
| **Stable** | `health.check`, `health.liveness`, `health.readiness`, `health.get`, `identity.get`, `lifecycle.status`, `capabilities.list`, `capability.announce`, `primal.announce`, `btsp.capabilities`, `auth.check`, `auth.mode`, `auth.peer_info`, `topology.get`, `proprioception.get` | Wire-compatible across versions; no breaking changes without major version bump |
| **Stable** | `visualization.render`, `visualization.validate`, `visualization.export`, `visualization.capabilities`, `visualization.introspect`, `visualization.panels`, `visualization.showing`, `visualization.dismiss` | Core rendering pipeline — stabilized since v1.4 |
| **Evolving** | `visualization.render.stream`, `visualization.render.grammar`, `visualization.render.dashboard`, `visualization.render.scene`, `visualization.render.graph`, `visualization.session.list`, `visualization.session.status`, `visualization.texture.upload`, `visualization.texture.attach`, `visualization.scene.verify` | Functional but response shapes may evolve |
| **Evolving** | `visualization.interact.apply`, `visualization.interact.perspectives`, `interaction.subscribe`, `interaction.poll`, `interaction.unsubscribe`, `interaction.sensor_stream.*` | Interaction pipeline — stabilizing |
| **Evolving** | `audio.synthesize`, `ui.render`, `ui.display_status`, `motor.*`, `provider.register_capability`, `capabilities.sensory`, `capabilities.sensory.negotiate` | Domain-specific — stable within current consumers |

### Degradation Behavior

When petalTongue is unavailable:

- **Ecosystem impact**: No visualization, no live dashboard, no web-mode content
  serving. All primals continue operating — petalTongue is a representation
  layer, not a control plane.
- **Springs**: Springs that render dashboards (esotericWebb, lithoSpore) fall
  back to text/JSON output or cached state. No data loss.
- **projectNUCLEUS**: Static site serving stops if petalTongue hosts sporePrint.
  Content remains in content provider or filesystem; another HTTP server can serve it.
- **Composition graphs**: `petaltongue_deploy.toml` marks petalTongue as
  non-critical. biomeOS skips visualization steps when petalTongue is absent.
- **IPC callers**: Get connection refused → standard JSON-RPC retry/fallback.

### Downstream Pairing

| Partner | Integration | Status |
|---------|-------------|--------|
| **esotericWebb** | Game UI rendering via `visualization.render.scene` + `motor.*` | Functional |
| **lithoSpore** | Validation dashboard via `visualization.render.dashboard` + `/api/events` SSE | Spring-side |
| **projectNUCLEUS** | sporePrint sovereign serving via `web` mode + content backend | Functional |
| **wetSpring** | Fermentation visualization via `visualization.render.grammar` | Spring-side |
| **bingoCube** | Crypto commitment grid via `render_color_grid_webgl` WASM + WebGL draw commands | Functional |
| **footPrint** | Composition integration via `/ws` WebSocket JSON-RPC bridge | Functional |

### Platform Audio Dependencies

petalTongue is **pure Rust** (`deny.toml` bans C crypto and native TLS). Audio:

- **Graph engine** (`petal-tongue-graph`): WAV-only output via `hound` crate.
  No native audio playback deps. Pure Rust on all platforms.
- **UI mode** (`petal-tongue-ui`): Decoding via `symphonia` (mp3, wav features).
  No system audio library required for decode. Playback delegates to
  `audio.play` capability discovery at runtime (not compiled in).
- **Headless/server/web modes**: No audio dependencies active. Build with
  `--no-default-features` to exclude the `ui` feature entirely.
- **Linux note**: `eframe` (egui backend) may pull transitive windowing deps
  (`wayland-sys`, `x11-dl`) for the `ui` mode. These are display deps, not
  audio. Build with `--no-default-features --features=""` for a zero-GUI binary.
- **macOS/Windows**: No additional system deps beyond standard windowing.
