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

18 workspace crates, single UniBin binary (`petaltongue`):

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
41 methods across domains: `visualization.*`, `interaction.*`, `graph.*`,
`health.*`, `capability.*`, `identity.*`, `session.*`, `sensor.*`.

BTSP Phase 1 complete: family-scoped socket naming, insecure guard,
domain symlinks (`visualization.sock`).

## Key Design Decisions

- **Two-dimensional universality**: universal across modalities (what you see)
  and substrates (what you run on).
- **Grammar of Graphics**: primals send grammar expressions, petalTongue
  compiles to best available representation.
- **No self-compute**: heavy work (GPU, physics) delegated via IPC to
  barraCuda, toadStool, coralReef. petalTongue discovers by capability.
- **Feature-gated GUI**: `ui` feature (default) pulls egui/eframe/glow.
  Headless builds (`--no-default-features`) have zero native display deps.
- **Audio discovery**: runtime heuristic backends (socket, direct, software,
  silent). Socket/direct behind optional features; software/silent always
  available.

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
cargo test --workspace --all-features     # ~5,800 tests, ~90% coverage
```

## Current State

Sprint 5 complete. All CI gates pass (fmt, clippy pedantic+nursery, doc,
cargo deny, tests). Zero unsafe, zero TODO/FIXME, zero production mocks,
zero `#[allow(]` in production. SPDX headers on all source files.

Remaining backlog: BTSP Phase 2 consumer wiring (cross-primal dep on
BearDog), aarch64 musl cross-compile for headless, tarpc feature-gating.
