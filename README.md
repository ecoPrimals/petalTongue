# petalTongue

**The Universal Representation System for ecoPrimals**

[![License: AGPL-3.0](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](./LICENSE)

> A sensory coordination layer that composes primal capabilities into coherent experiences.

---

## Quick Start

```bash
cargo build --release

./target/release/petaltongue ui          # Desktop GUI
./target/release/petaltongue tui         # Terminal UI (Pure Rust)
./target/release/petaltongue web         # Web server (Pure Rust)
./target/release/petaltongue headless    # Headless rendering
./target/release/petaltongue status      # System status
```

See [START_HERE.md](./START_HERE.md) for configuration and development setup.

---

## What is petalTongue?

petalTongue is ecoPrimals' universal UI platform -- a single unified binary (UniBin) with 5 modes:

```
petaltongue
├── ui        Desktop GUI (egui, pluggable backend abstraction)
├── tui       Terminal UI (ratatui -- Pure Rust)
├── web       Web server (axum -- Pure Rust)
├── headless  Batch rendering to SVG/PNG/JSON/DOT (Pure Rust)
└── status    System info (Pure Rust)
```

### Architecture

- **JSON-RPC 2.0** over Unix sockets (primary IPC)
- **tarpc** for high-performance binary RPC (secondary, 5-10x faster)
- **HTTP** for external/browser clients only (fallback)
- **Capability-based discovery** -- zero hardcoded primal names, ports, or paths
- **Self-knowledge only** -- discovers other primals at runtime
- **Graceful degradation** -- works standalone or in full ecosystem

### Crates (17)

| Crate | Purpose |
|-------|---------|
| `petal-tongue-core` | Graph engine, capabilities, config, constants, data channels |
| `petal-tongue-graph` | 2D visual rendering, chart rendering, clinical theme, audio sonification |
| `petal-tongue-ui` | Desktop GUI (egui/eframe), panels, scenarios, biomeOS integration |
| `petal-tongue-tui` | Terminal UI (ratatui) |
| `petal-tongue-ipc` | Unix socket + TCP fallback IPC, tarpc types |
| `petal-tongue-discovery` | Provider discovery (HTTP, JSON-RPC, mDNS, dynamic scenarios) |
| `petal-tongue-entropy` | Human entropy capture (gesture, narrative, visual, audio) |
| `petal-tongue-animation` | Flower/visual animations |
| `petal-tongue-adapters` | EcoPrimal adapter traits |
| `petal-tongue-primitives` | UI primitives (forms, tables, trees, panels) |
| `petal-tongue-modalities` | SVG/PNG GUI modalities |
| `petal-tongue-telemetry` | Telemetry and metrics |
| `petal-tongue-headless` | Headless binary (Pure Rust, zero GUI deps) |
| `petal-tongue-ui-core` | Universal UI traits and headless renderers |
| `petal-tongue-api` | biomeOS JSON-RPC client |
| `petal-tongue-cli` | CLI argument parsing |
| `doom-core` | Doom WAD renderer (demonstration) |

---

## Quality

| Metric | Status |
|--------|--------|
| Tests | 1,300+ passing, 0 failures |
| Build | `cargo check --workspace` clean |
| Clippy | Pedantic, 0 errors |
| Docs | `cargo doc --workspace` clean |
| Formatting | `cargo fmt --check` clean |
| Unsafe | `#![forbid(unsafe_code)]` on 16/17 crates |
| License | AGPL-3.0 on all crates |
| Files | All under 1,000 lines |
| Constants | Centralized, zero hardcoding |

---

## Development

```bash
# Prerequisites: Rust 1.75+
cargo build --workspace            # Build all crates
cargo test --workspace             # Run all tests
cargo clippy --workspace -- -W clippy::pedantic   # Lint
cargo fmt --check                  # Check formatting
cargo doc --workspace --no-deps    # Build docs
```

### Configuration

Environment variables (highest priority):
```bash
export PETALTONGUE_WEB_PORT=8080
export PETALTONGUE_HEADLESS_PORT=9000
export BIOMEOS_NEURAL_API_SOCKET=/run/user/$(id -u)/biomeos-neural-api.sock
```

Config file (`$XDG_CONFIG_HOME/petaltongue/config.toml`):
```toml
[network]
web_port = 8080
headless_port = 9000

[discovery]
timeout_ms = 5000
```

See [ENV_VARS.md](./ENV_VARS.md) for the full reference.

---

## Contributing

Follow TRUE PRIMAL principles:
- Discover capabilities at runtime, never hardcode
- Pure Rust, modern idioms (`async`/`await`, `Arc`/`RwLock`)
- Proper error handling (`Result<T>`, no `unwrap()` in production)
- Concurrent testing (no sleeps, no serial except chaos tests)
- `#![forbid(unsafe_code)]` unless FFI is unavoidable
- Semantic naming (`domain.operation`)
- JSON-RPC/tarpc first, HTTP fallback

---

## License

AGPL-3.0 -- See [LICENSE](./LICENSE) for full text.
