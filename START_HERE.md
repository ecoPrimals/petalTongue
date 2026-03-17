# petalTongue -- Start Here

**Updated**: March 16, 2026

---

## Build & Run

```bash
cargo build --release

petaltongue ui                     # Desktop display (egui)
petaltongue tui                    # Terminal display (ratatui)
petaltongue web                    # Web interface (axum)
petaltongue headless --mode svg -o out.svg   # Export to SVG
petaltongue server                 # IPC server (no display)
petaltongue status                 # System info
```

## Configuration

Priority: Environment > Config file > Defaults

```bash
export PETALTONGUE_WEB_PORT=8080
export PETALTONGUE_HEADLESS_PORT=9000
export BIOMEOS_NEURAL_API_SOCKET=/run/user/$(id -u)/biomeos-neural-api.sock

# Tuning & timing (optional)
export PETALTONGUE_RPC_TIMEOUT_SECS=5
export PETALTONGUE_HEARTBEAT_INTERVAL_SECS=30
export PETALTONGUE_REFRESH_INTERVAL_SECS=2
export PETALTONGUE_DISCOVERY_TIMEOUT_SECS=5
export PETALTONGUE_TUI_TICK_MS=100
export PETALTONGUE_TELEMETRY_BUFFER=10000
export PETALTONGUE_RETRY_INITIAL_MS=100
export PETALTONGUE_RETRY_MAX_SECS=10
```

```toml
# $XDG_CONFIG_HOME/petaltongue/config.toml
[network]
web_port = 8080
headless_port = 9000

[discovery]
timeout_ms = 5000
```

Full reference: [ENV_VARS.md](./ENV_VARS.md)

---

## Development

```bash
cargo test --workspace --all-features           # 5,447 tests
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check                               # Format check (clean)
cargo doc --workspace --no-deps                 # Docs (clean)
cargo llvm-cov --workspace --summary-only       # Coverage (~90% line)
```

### Scenarios

```bash
petaltongue ui --scenario sandbox/scenarios/paint-simple.json
petaltongue ui --scenario sandbox/scenarios/healthspring-diagnostic.json
```

### Architecture Rules

1. **Self-knowledge only** -- petalTongue knows its own name, ports, and capabilities.
   Other primals discovered at runtime via capability-based discovery.
2. **Constants centralized** -- All self-knowledge in `petal_tongue_core::constants`.
3. **IPC priority** -- JSON-RPC over Unix sockets (primary), tarpc (secondary), HTTP (fallback).
4. **Typed error handling** -- `thiserror` everywhere, no `anyhow` in production; `deny(unwrap_used, expect_used)` with `#[expect]` for justified cases.
5. **`#![forbid(unsafe_code)]`** unless hardware FFI is unavoidable. Document with `// SAFETY:`.
6. **Concurrent testing** -- No `thread::sleep`. Use `tokio::time::timeout`.
7. **Files under 1,000 lines** -- Split into cohesive modules at ~800 lines.
8. **SPDX headers** -- `// SPDX-License-Identifier: AGPL-3.0-or-later` on all `.rs` files.
9. **Semantic naming** -- JSON-RPC methods follow `{domain}.{operation}` pattern.

---

## Key Modules

### Core (`petal-tongue-core`)
- `constants.rs` -- Centralized self-knowledge (name, ports, socket names)
- `graph_engine.rs` -- Graph data model (nodes, edges, layout)
- `config_system.rs` -- XDG-compliant configuration (env > file > defaults)
- `data_channel.rs` -- DataChannel enum (11 variants: TimeSeries, Distribution, Bar, Gauge, Spectrum, Heatmap, Scatter, Scatter3D, FieldMap, GameScene, Soundscape)
- `capability_names.rs` -- Centralized capability/method/socket/primal constants (60+ capabilities, 15 primal identities)
- `telemetry_adapter.rs` -- JSONL telemetry ingestion (hotSpring)
- `or_exit.rs` -- `OrExit<T>` trait for zero-panic validation binaries

### IPC (`petal-tongue-ipc`)
- `unix_socket_server.rs` -- JSON-RPC 2.0 server over Unix sockets
- `tarpc_client.rs` -- tarpc binary RPC client
- `tarpc_types/` -- tarpc types split into submodules
- `socket_path.rs` -- XDG-compliant socket path discovery
- `ipc_errors.rs` -- `IpcErrorPhase`, `StreamItem` (NDJSON), `extract_rpc_error()`
- `resilience.rs` -- `CircuitBreaker`, `RetryPolicy` for IPC fault tolerance
- `discovery_helpers.rs` -- Primal socket resolution, env var helpers

### Discovery (`petal-tongue-discovery`)
- `lib.rs` -- Provider discovery orchestrator
- `unix_socket_provider.rs` -- Unix socket JSON-RPC discovery (primary)
- `neural_api_provider.rs` -- biomeOS Neural API discovery
- `songbird_client.rs` -- Songbird capability discovery
- `capability_parse.rs` -- Dual-format capability parsing (flat + enriched)
- `http_provider.rs` -- HTTP fallback (feature-gated `legacy-http`)

### Specs

Architectural specifications in `specs/` -- read these before making major changes:
- `GRAMMAR_OF_GRAPHICS_ARCHITECTURE.md` -- Next evolution (composable grammar)
- `UNIVERSAL_VISUALIZATION_PIPELINE.md` -- Data→render pipeline
- `TUFTE_CONSTRAINT_SYSTEM.md` -- Visualization quality constraints

---

## Cross-Primal Integration

- **biomeOS** -- Topology visualization (JSON-RPC), Neural API lifecycle
- **healthSpring** -- Diagnostic data channels, clinical theme, streaming sessions
- **hotSpring** -- JSONL telemetry ingestion to TimeSeries
- **neuralSpring** -- Pipeline DAGs, diverging color scales
- **wetSpring** -- Backpressure-aware streaming, Scatter 2D ordinations
- **ludoSpring** -- 7 GameDataChannel types, 60 Hz sensor feed
- **ToadStool** -- Display backend (tarpc, capability-discovered), provider registry
- **barraCuda** -- GPU compute offload for heavy visualization
- **Songbird** -- Discovery protocol

See `ecoPrimals/wateringHole/petaltongue/` for inter-primal standards.

---

## License

AGPL-3.0-or-later -- See [LICENSE](./LICENSE).
