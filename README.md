# petalTongue

**The Universal User Interface Primal for ecoPrimals**

[![License: AGPL-3.0-or-later](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue.svg)](./LICENSE)

> One Engine, Infinite Representations — translating any computational universe
> into any modality for any user.

---

## Quick Start

```bash
cargo build --release

petaltongue ui          # Desktop display (egui)
petaltongue tui         # Terminal display (ratatui)
petaltongue web         # Web interface (axum)
petaltongue headless    # Headless rendering (SVG/PNG/JSON)
petaltongue server      # IPC server (no display)
petaltongue status      # System status
```

See [START_HERE.md](./START_HERE.md) for configuration and development setup.

---

## What is petalTongue?

petalTongue is ecoPrimals' **Universal User Interface** — a single UniBin
binary that translates ecosystem state into every available modality for every
type of user. It implements a composable **Grammar of Graphics** engine with a
**declarative scene graph** and **Manim-style animation system**, allowing any
primal to send a grammar expression that petalTongue compiles to the best
available representation. Heavy compute (GPU shaders, physics simulations) is
delegated to barraCuda, Toadstool, and coralReef via IPC.

The UUI philosophy is **two-dimensional universality**: universal across
*computational universes* (any data from any primal) and universal across
*user types* (sighted human, blind hiker, paraplegic developer, AI agent,
dolphin, fungal network). Accessibility is not a feature — it is the
architecture. Every modality is a first-class compilation target.

```
petaltongue
├── ui        Desktop display (egui, pluggable backend)
├── tui       Terminal display (ratatui)
├── web       Web interface (axum)
├── headless  Batch rendering (SVG/PNG/JSON)
├── server    IPC server (no display)
└── status    System info
```

### Architecture

- **Universal User Interface** -- any computational universe → any modality → any user type
- **SAME DAVE model** -- Sensory Afferent / Motor Efferent bidirectional feedback loops
- **UUI glossary** -- canonical terminology in `petal_tongue_core::uui_glossary`
- **tarpc** binary RPC with `bytes::Bytes` zero-copy (primary primal-to-primal)
- **JSON-RPC 2.0** over Unix sockets (secondary, local IPC and debugging)
- **HTTP** for browser/external clients only (fallback)
- **Capability-based discovery** -- zero hardcoded primal names, runtime-only
- **Self-knowledge only** -- other primals discovered at runtime
- **Graceful degradation** -- works standalone or in full ecosystem
- **Grammar of Graphics** -- composable data→representation pipeline
- **DataBinding auto-compiler** -- all 11 chart types auto-compile to grammar
- **Tufte constraints** -- machine-checked visualization quality
- **Dashboard layout engine** -- multi-panel grid with domain theming and SVG export
- **Domain-aware rendering** -- automatic palette selection per domain
- **Multi-modal output** -- visual, audio, haptic, terminal, braille, JSON API (tiered)
- **Server-side backpressure** -- rate limiting for 60 Hz streaming
- **Pipeline DAG orchestration** -- multi-stage workflows with topological sort
- **Scenario loader** -- load JSON scenario files from disk (`--scenario` CLI flag)
- **Zero-copy state management** -- Arc-wrapped shared state
- **Centralized configurable constants** -- all timeouts, ports env-overridable

### Crates (16)

| Crate | Purpose |
|-------|---------|
| `petal-tongue-core` | Graph engine, capabilities, config, interaction engine, data bindings, UUI glossary |
| `petal-tongue-graph` | Domain-aware chart renderers, 2D rendering, audio sonification |
| `petal-tongue-ui` | Desktop display (egui/eframe), panels, scenarios, biomeOS |
| `petal-tongue-tui` | Terminal display (ratatui) |
| `petal-tongue-ipc` | Unix socket IPC, JSON-RPC server, tarpc client, visualization handler |
| `petal-tongue-discovery` | Provider discovery (JSON-RPC, mDNS, Unix socket, scenarios) |
| `petal-tongue-scene` | Scene graph, animation, grammar compiler, DataBinding compiler, dashboard layout, Tufte constraints, modality compilers, physics bridge |
| `petal-tongue-entropy` | Human entropy capture (gesture, narrative, visual, audio) |
| `petal-tongue-animation` | Visual animations |
| `petal-tongue-adapters` | EcoPrimal adapter traits |
| `petal-tongue-telemetry` | Telemetry and metrics |
| `petal-tongue-headless` | Headless rendering (zero display deps) |
| `petal-tongue-ui-core` | Universal interface traits and headless renderers |
| `petal-tongue-api` | biomeOS JSON-RPC client |
| `petal-tongue-cli` | CLI argument parsing |
| `doom-core` | Doom WAD renderer (platform testing, optional) |

---

## Quality

| Metric | Status |
|--------|--------|
| Tests | 5,447 passing, 0 failures |
| Formatting | `cargo fmt --check` clean |
| Clippy | Zero warnings (pedantic + nursery via workspace lints) |
| Docs | `cargo doc --workspace --no-deps` clean |
| Coverage | ~90% line (llvm-cov) |
| Unsafe | `#![forbid(unsafe_code)]` on all 16 crates + UniBin, zero C deps |
| License | AGPL-3.0-or-later, SPDX headers on all source files |
| Files | All under 1,000 lines; largest 854 lines (`tests_extended.rs`) |
| Cargo Deny | advisories, bans, licenses, sources all clean |
| Edition | 2024 (all 16 crates + sandbox) |
| External C deps | None -- pure Rust (`rustix` for syscalls) |
| Error handling | Typed `thiserror` errors throughout -- zero `anyhow` in production |
| Zero-copy IDs | `DataSourceId`, `GrammarId` are `Arc<str>` -- O(1) clone |
| Property tests | `proptest` for JSON-RPC parser + core data types |
| Cross-primal e2e | Real Unix socket server ↔ JSON-RPC client integration tests |
| IPC resilience | `IpcErrorPhase` structured errors, `CircuitBreaker`, `RetryPolicy` |
| Health triad | `health.check` + `health.liveness` + `health.readiness` (K8s/biomeOS) |
| NDJSON streaming | `StreamItem` (Data/Progress/End/Error) for pipeline consumption |
| Zero-panic | `OrExit<T>` trait for validation binaries |
| Dual-format discovery | Parses both flat string and enriched object capability arrays |

---

## Development

```bash
# Prerequisites: Rust nightly (edition 2024)
cargo build --workspace
cargo test --workspace --all-features        # 5,447 tests
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
cargo doc --workspace --no-deps
cargo llvm-cov --workspace --summary-only    # Coverage (~90% line)
```

### Configuration

Priority: Environment > Config file > Defaults.

```bash
export PETALTONGUE_WEB_PORT=8080
export PETALTONGUE_HEADLESS_PORT=9000
export BIOMEOS_NEURAL_API_SOCKET=/run/user/$(id -u)/biomeos-neural-api.sock
```

See [ENV_VARS.md](./ENV_VARS.md) for the full reference.

---

## Specs

Architectural specifications live in `specs/`:

| Spec | Purpose |
|------|---------|
| `UNIVERSAL_USER_INTERFACE_SPECIFICATION.md` | UUI philosophy — any universe, any user, any modality |
| `BIDIRECTIONAL_UUI_ARCHITECTURE.md` | SAME DAVE cognitive model |
| `PRIMAL_MULTIMODAL_RENDERING_SPECIFICATION.md` | Multi-modal rendering pipeline |
| `GRAMMAR_OF_GRAPHICS_ARCHITECTURE.md` | Composable grammar type system |
| `UNIVERSAL_VISUALIZATION_PIPELINE.md` | End-to-end data→render pipeline, barraCuda integration |
| `TUFTE_CONSTRAINT_SYSTEM.md` | Machine-checked visualization quality |
| `INTERACTION_ENGINE_ARCHITECTURE.md` | Bidirectional interaction, perspective system |
| `JSONRPC_PROTOCOL_SPECIFICATION.md` | JSON-RPC 2.0 IPC protocol |

---

## Cross-Primal Integration

See `ecoPrimals/wateringHole/petaltongue/` for inter-primal standards:
- `VISUALIZATION_INTEGRATION_GUIDE.md` -- How other primals send data to petalTongue
- `BIOMEOS_API_SPECIFICATION.md` -- biomeOS API contract
- `QUICK_START_FOR_BIOMEOS.md` -- 5-minute integration guide

---

## Contributing

- **UUI-first language**: Use "display" not "GUI", "activate" not "click", "perceivable" not "visible" — see `petal_tongue_core::uui_glossary`
- Discover capabilities at runtime, never hardcode primal names
- Pure Rust, edition 2024, `async`/`await`, `Arc`/`RwLock`
- Typed error handling (`thiserror`, no `anyhow` in production); `deny(unwrap_used, expect_used)` with `#[expect]` for justified suppressions
- `#![forbid(unsafe_code)]` unless hardware FFI is unavoidable
- Semantic method naming (`domain.operation`)
- JSON-RPC + tarpc first, HTTP fallback only
- All files under 1,000 lines
- SPDX headers on all source files

---

## License

AGPL-3.0-or-later -- See [LICENSE](./LICENSE).
