# petalTongue — Context

**Version:** 1.6.6
**Role:** Universal User Interface primal (visualization, presentation, interaction)
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

18 workspace crates, single UniBin binary (`petaltongue`, 7 subcommands):

| Crate | Purpose |
|-------|---------|
| `petal-tongue-core` | Types, config, sensory discovery, capability registry |
| `petal-tongue-ipc` | JSON-RPC 2.0 server (UDS + TCP), BTSP, push delivery |
| `petal-tongue-scene` | Declarative scene graph, modality compilers |
| `petal-tongue-graph` | Chart rendering, sonification |
| `petal-tongue-animation` | Manim-style animation system |
| `petal-tongue-ui` | Native GUI (egui/eframe), feature-gated |
| `petal-tongue-tui` | Terminal UI (ratatui) |
| `petal-tongue-ui-core` | Pure Rust abstract UI (text, SVG, canvas) |
| `petal-tongue-discovery` | Primal/capability discovery clients |
| `petal-tongue-cli` | CLI handler logic |
| `petal-tongue-api` | BiomeOS client, HTTP APIs |
| `petal-tongue-entropy` | Human entropy capture |
| `petal-tongue-adapters` | Adapter framework |
| `petal-tongue-headless` | Headless rendering binary |
| `petal-tongue-telemetry` | Observability and metrics |
| `petal-tongue-types` | WASM-portable data types |
| `petal-tongue-wasm` | Browser rendering module |
| `doom-core` | Doom WAD rendering (platform stress test) |

## IPC Surface

JSON-RPC 2.0 over Unix domain sockets (primary) and TCP (`--port`).
43 methods across domains: `visualization.*` (incl. `visualization.render.graph`,
`visualization.session.*`), `interaction.*`, `health.*`, `capabilities.*`,
`capability.*`, `identity.*`, `ui.*`, `motor.*`, `audio.*`, `lifecycle.*`.

BTSP Phase 1 complete: family-scoped socket naming, insecure guard,
domain symlinks (`visualization.sock`). BTSP Phase 2 complete: BearDog
handshake delegation on both UDS and TCP, length-prefixed and JSON-line
framing, `btsp.session.create/verify/negotiate` via provider client.

## Key Design Decisions

- **Two-dimensional universality**: universal across modalities (what you see)
  and substrates (what you run on).
- **Grammar of Graphics**: primals send grammar expressions, petalTongue
  compiles to best available representation.
- **No self-compute**: heavy work (GPU, physics) delegated via IPC to
  barraCuda, toadStool, coralReef. petalTongue discovers by capability.
- **Feature-gated GUI**: `ui` feature (default) pulls egui/eframe/glow.
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
- `display.*` — ToadStool (window lifecycle, frame presentation)
- `compute.*` / `math.*` — barraCuda via ToadStool (GPU dispatch)
- `btsp.session.*` — BearDog (transport security)
- `discovery.*` / `ipc.*` — Songbird + biomeOS (registry, routing)
- TLS/HTTPS — Songbird relay (design ready)
- `audio.play` / `audio.stream` — ToadStool (future, wired as Tier 1 stub)
- `storage.put` / `storage.get` — NestGate (future)
- `ai.query` / `ai.complete` — Squirrel (future)

The eframe/glow C/FFI stack exists only behind `ui-eframe` feature as a
development convenience. The architectural path is EguiPixelRenderer →
DisplayManager → ecosystem `display.*` IPC.

## Ecosystem Position

petalTongue discovers other primals at runtime via capability-based IPC.
It has zero compile-time knowledge of primal identities in production builds
(fixture data gated behind `#[cfg(test)]` or `test-fixtures` feature).

Coordinates with biomeOS (orchestration), Songbird (registry), BearDog
(security/BTSP), and any primal that exposes visualization-relevant
capabilities.

## Build

```bash
cargo build --release                     # Full binary (26M musl-static)
cargo build --release --no-default-features  # Headless only
cargo test --workspace --all-features     # ~6,150+ tests, ~90% coverage
```

## Current State

Stadial parity gate cleared (April 17, 2026). All CI gates pass (fmt,
clippy pedantic+nursery, doc, cargo deny, tests). Zero unsafe, zero
TODO/FIXME, zero production `unwrap()`, zero `#[allow(` in production code
(only in `#[cfg(test)]` modules). Lint policy: `#[expect]` with
reasons for justified suppressions; targeted `#[allow]` only where
`#[expect]` cannot apply (e.g. `cfg_attr` platform gates). SPDX headers
on all source files. Edition 2024, deny.toml enforced.
98 test suites, all passing (all-features).

Native `async fn` in traits (April 25, 2026): eliminated all manual
`fn -> impl Future + Send` desugaring across 13 production modules.
All traits now use native `async fn` (RPITIT). Zero `manual_async_fn`
suppressions remain. Net −100 lines across 21 files.

macOS cross-arch build fix (April 19, 2026): conflicting `AudioCanvas`
impl blocks resolved; `petal-tongue-ui` now compiles with zero warnings
on x86_64-apple-darwin and aarch64-apple-darwin.

`reqwest` runtime dependency fully eliminated (April 17). Replaced with
thin `LocalHttpClient` (hyper + hyper-util, already transitive from axum).
`ring`, `hyper-rustls`, `rustls`, `rustls-webpki` all removed from lockfile.
petalTongue no longer owns any TLS stack — Songbird handles external HTTPS
via tower atomic IPC.

Sprint 8: complete `dyn` trait object elimination (22 custom traits
evolved to enum dispatch / generics), `async-trait` removed from all
first-party code (native `async fn` in traits via RPITIT; may remain
as transitive dep via tarpc/axum), `Pin<Box<dyn Future>>` type aliases
eliminated, 11 production modules refactored below 600 LOC, hardcoded
ecosystem paths evolved to env-configurable constants.

Sprint 7: deep debt resolution across 14 production modules (smart
refactoring by domain, not mechanical splitting), hardcoding evolved to
capability-based defaults, BTSP provider default evolved from primal
identity to capability name (`security`), centralized socket path constants.

UUI boundary analysis (April 17): dead deps removed (`png`, direct `winit`),
capability discovery unified (`GpuComputeProvider` and `physics_bridge` now
use `CapabilityDiscovery<BiomeOsBackend>` as primary path), V2 display backend
fixed (tarpc→JSON-RPC for `display.*` ops), audio Tier 1 wired (`NetworkBackend`
via capability discovery, graceful fallback), `discovered-display` feature
properly gated with `#[cfg]`.

Dependency cleanup (April 19, 2026): dead `petal-tongue-graph` dep removed
from `petal-tongue-ui-core`, headless builds no longer pull egui (graph
`default-features = false`), tarpc trimmed from `full` to specific features.

`petaltongue live` mode (April 21, 2026): new CLI subcommand merging `ui`
(egui/eframe) and `server` (UDS JSON-RPC IPC) into a single process for
interactive desktop NUCLEUS deployment. IPC server runs as background tokio
task, egui on main thread, connected via `Arc<RwLock<VisualizationState>>`
and companion registries. This is the tier-one deployment mode for every
spring/garden cell graph.

BTSP wire-format detection fix (April 21, 2026): three-way classification
in `handle_uds_with_btsp` and `handle_tcp_with_btsp` — non-`{` first byte
→ length-prefixed BTSP, `{` + `"protocol"` → BTSP JSON-line announcement
(from primalSpring), `{` only → plain JSON-RPC. Fixes misclassification of
`{"protocol":"btsp",...}` announcements as invalid JSON-RPC.

Deep debt audit (April 21, 2026): clippy zero warnings achieved (boxed
`DoomPanelWrapper`, removed needless return, fixed unused async). `futures`
→ `futures-util` in discovery crate. All 4 remaining `dyn` usages audited
as idiomatic (callbacks + error trait).

BTSP JSON-line handshake relay (April 23, 2026): primalSpring Phase 45c
upstream debt resolved. New `btsp/json_line.rs` module implements full
4-step JSON-line BTSP relay (same pattern as ToadStool, barraCuda, etc.).
UDS/TCP accept now routes JSON-line BTSP announcements to the new relay
instead of the length-prefixed handler. BearDog field names aligned
(`session_token`, `response`, `family_seed`), BearDog challenge used
(not local PRNG), `SECURITY_PROVIDER_SOCKET`/`CRYPTO_PROVIDER_SOCKET`/
`SECURITY_SOCKET` added to provider socket cascade.

Remaining backlog: BTSP Phase 3 encryption, aarch64 musl cross-compile
for headless, audio backend wire protocols (PipeWire/PulseAudio).
