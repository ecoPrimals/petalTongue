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
that renders ecosystem state across every available modality. It is evolving
from fixed-widget rendering toward a composable **Grammar of Graphics** engine
where any primal can send a declarative grammar expression and petalTongue
compiles it to the best available output.

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
- **Grammar of Graphics** -- composable data→visualization pipeline (design phase)
- **Tufte constraints** -- machine-checked visualization quality (design phase)

### Crates (17)

| Crate | Purpose |
|-------|---------|
| `petal-tongue-core` | Graph engine, capabilities, config, constants, data channels |
| `petal-tongue-graph` | 2D rendering, charts, clinical theme, audio sonification |
| `petal-tongue-ui` | Desktop GUI (egui/eframe), panels, scenarios, biomeOS |
| `petal-tongue-tui` | Terminal UI (ratatui) |
| `petal-tongue-ipc` | Unix socket + TCP IPC, JSON-RPC server, tarpc types |
| `petal-tongue-discovery` | Provider discovery (JSON-RPC, HTTP, mDNS, scenarios) |
| `petal-tongue-entropy` | Human entropy capture (gesture, narrative, visual, audio) |
| `petal-tongue-animation` | Flower/visual animations |
| `petal-tongue-adapters` | EcoPrimal adapter traits |
| `petal-tongue-primitives` | UI primitives (forms, tables, trees, panels, command palette) |
| `petal-tongue-modalities` | SVG/PNG GUI modalities |
| `petal-tongue-telemetry` | Telemetry and metrics |
| `petal-tongue-headless` | Headless binary (zero GUI deps) |
| `petal-tongue-ui-core` | Universal UI traits and headless renderers |
| `petal-tongue-api` | biomeOS JSON-RPC client |
| `petal-tongue-cli` | CLI argument parsing |
| `doom-core` | Doom WAD renderer (platform testing) |

---

## Quality

| Metric | Actual Status |
|--------|---------------|
| Tests | 1,309 passing, 0 failures, 23 ignored |
| Formatting | `cargo fmt --check` clean |
| Clippy | 76 errors (missing docs), 487 warnings -- needs pedantic config |
| Docs | 141 warnings (missing field docs, deprecated API) |
| Coverage | 54.10% line (target: 90%) |
| Unsafe | `#![forbid(unsafe_code)]` on 5/17 crates |
| License | AGPL-3.0-only, SPDX headers on 289+ files |
| Files | All under 1,000 lines (max: 833) |
| Edition | 2024 |

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
