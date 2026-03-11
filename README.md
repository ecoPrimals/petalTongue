# petalTongue

**The Universal Representation Primal for ecoPrimals**

[![License: AGPL-3.0-only](https://img.shields.io/badge/license-AGPL--3.0--only-blue.svg)](./LICENSE)

> Makes data human-understandable across every sensory modality.

---

## Quick Start

```bash
cargo build --release

petaltongue ui          # Desktop GUI (egui)
petaltongue tui         # Terminal UI (ratatui)
petaltongue web         # Web server (axum)
petaltongue headless    # Headless rendering (SVG/PNG/JSON)
petaltongue status      # System status
```

See [START_HERE.md](./START_HERE.md) for configuration and development setup.

---

## What is petalTongue?

petalTongue is ecoPrimals' universal user interface -- a single UniBin binary
that renders ecosystem state across every available modality. It combines a
composable **Grammar of Graphics** engine with a **declarative scene graph**
and **Manim-style animation system**, allowing any primal to send a grammar
expression that petalTongue compiles to the best available output. Heavy
compute (GPU shaders, physics simulations) is delegated to barraCuda,
Toadstool, and coralReef via IPC.

```
petaltongue
├── ui        Desktop GUI (egui, pluggable backend)
├── tui       Terminal UI (ratatui)
├── web       Web server (axum)
├── headless  Batch rendering (SVG/PNG/JSON)
└── status    System info
```

### Architecture

- **JSON-RPC 2.0** over Unix sockets (primary IPC)
- **tarpc** binary RPC with `bytes::Bytes` zero-copy (secondary)
- **HTTP** for browser/external clients only (fallback)
- **Capability-based discovery** via Songbird (zero hardcoded primal names)
- **Self-knowledge only** -- other primals discovered at runtime
- **Graceful degradation** -- works standalone or in full ecosystem
- **Grammar of Graphics** -- composable data→visualization pipeline
- **DataBinding auto-compiler** -- all 9 chart types (TimeSeries, Distribution, Bar, Gauge, Spectrum, Heatmap, Scatter, Scatter3D, FieldMap) auto-compile to grammar
- **Geometry & faceting** -- Tile (heatmap/fieldmap), Arc (gauge), faceting (small multiples), threshold coloring
- **Dashboard layout engine** -- multi-panel grid with domain theming and SVG export
- **Tufte constraints** -- machine-checked visualization quality
- **Domain-aware rendering** -- automatic palette selection per domain (health, physics, ecology, game...)
- **Diverging color scales** -- three-stop interpolation for heatmaps (neuralSpring Kokkos parity)
- **Server-side backpressure** -- rate limiting for 60 Hz streaming (configurable `BackpressureConfig`)
- **JSONL telemetry ingestion** -- hotSpring telemetry parsed to `DataBinding::TimeSeries`
- **Pipeline DAG orchestration** -- multi-stage visualization workflows with topological sort
- **Provider registry** -- `provider.register_capability` IPC for toadStool S145 compliance
- **Session health** -- `visualization.session.status` queries backpressure and frame metrics
- **Spring IPC** -- springs push data via `visualization.render`, petalTongue auto-compiles and renders
- **Scenario loader** -- load JSON scenario files from disk (`--scenario` CLI flag)

### Crates (16)

| Crate | Purpose |
|-------|---------|
| `petal-tongue-core` | Graph engine, capabilities, config, interaction engine, data bindings |
| `petal-tongue-graph` | Domain-aware chart renderers, 2D rendering, audio sonification |
| `petal-tongue-ui` | Desktop GUI (egui/eframe), panels, scenarios, biomeOS |
| `petal-tongue-tui` | Terminal UI (ratatui) |
| `petal-tongue-ipc` | Unix socket IPC, JSON-RPC server, tarpc client, visualization handler |
| `petal-tongue-discovery` | Provider discovery (JSON-RPC, mDNS, Unix socket, scenarios) |
| `petal-tongue-scene` | Scene graph, animation, grammar compiler, DataBinding compiler, dashboard layout, Tufte constraints, modality compilers, physics bridge |
| `petal-tongue-entropy` | Human entropy capture (gesture, narrative, visual, audio) |
| `petal-tongue-animation` | Flower/visual animations |
| `petal-tongue-adapters` | EcoPrimal adapter traits |
| `petal-tongue-telemetry` | Telemetry and metrics |
| `petal-tongue-headless` | Headless binary (zero GUI deps) |
| `petal-tongue-ui-core` | Universal UI traits and headless renderers |
| `petal-tongue-api` | biomeOS JSON-RPC client |
| `petal-tongue-cli` | CLI argument parsing |
| `doom-core` | Doom WAD renderer (platform testing, optional) |

---

## Quality

| Metric | Status |
|--------|--------|
| Tests | 3,409 passing, 0 failures, 17 ignored |
| Formatting | `cargo fmt --check` clean |
| Clippy | Zero warnings, pedantic + nursery enabled (`[workspace.lints.clippy]`) |
| Docs | `RUSTDOCFLAGS="-D warnings" cargo doc` clean |
| Coverage | 77.4% region / 79.2% function (llvm-cov) — target 90% |
| Unsafe | `#![forbid(unsafe_code)]` workspace-wide, zero C deps |
| License | AGPL-3.0-only, SPDX headers on all source files |
| Files | All production files under 1,000 lines (max: `animation.rs` 1,086 — refactor pending) |
| Edition | 2024 (all 16 crates) |
| External C deps | None -- pure Rust (`rustix` for syscalls) |

---

## Development

```bash
# Prerequisites: Rust nightly (edition 2024)
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check
cargo doc --workspace --no-deps
cargo llvm-cov --workspace --summary-only   # Coverage
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
| `GRAMMAR_OF_GRAPHICS_ARCHITECTURE.md` | Composable grammar type system |
| `UNIVERSAL_VISUALIZATION_PIPELINE.md` | End-to-end data→render pipeline, barraCuda integration |
| `TUFTE_CONSTRAINT_SYSTEM.md` | Machine-checked visualization quality |
| `INTERACTION_ENGINE_ARCHITECTURE.md` | Bidirectional interaction, perspective system |
| `BIDIRECTIONAL_UUI_ARCHITECTURE.md` | SAME DAVE cognitive model |
| `UNIVERSAL_USER_INTERFACE_SPECIFICATION.md` | UUI for any universe and user |
| `JSONRPC_PROTOCOL_SPECIFICATION.md` | JSON-RPC 2.0 IPC protocol |

---

## Cross-Primal Integration

See `ecoPrimals/wateringHole/petaltongue/` for inter-primal standards:
- `VISUALIZATION_INTEGRATION_GUIDE.md` -- How other primals send data to petalTongue
- `BIOMEOS_API_SPECIFICATION.md` -- biomeOS API contract
- `QUICK_START_FOR_BIOMEOS.md` -- 5-minute integration guide

---

## Contributing

- Discover capabilities at runtime, never hardcode primal names
- Pure Rust, edition 2024, `async`/`await`, `Arc`/`RwLock`
- Proper error handling (`Result<T>`, no `unwrap()` in production)
- `#![forbid(unsafe_code)]` unless hardware FFI is unavoidable
- Semantic method naming (`domain.operation`)
- JSON-RPC + tarpc first, HTTP fallback only
- All files under 1,000 lines
- SPDX headers on all source files

---

## License

AGPL-3.0-only -- See [LICENSE](./LICENSE).
