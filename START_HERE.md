# petalTongue -- Start Here

**Updated**: March 8, 2026

---

## Build & Run

```bash
# Build
cargo build --release

# Run modes
petaltongue ui                     # Desktop GUI (discovers display provider)
petaltongue tui                    # Terminal UI (Pure Rust)
petaltongue web                    # Web server
petaltongue headless --mode svg -o out.svg   # Export to SVG
petaltongue status                 # System info
```

## Configuration

Priority: Environment > Config file > Defaults

```bash
# Environment variables
export PETALTONGUE_WEB_PORT=8080
export PETALTONGUE_HEADLESS_PORT=9000
export BIOMEOS_NEURAL_API_SOCKET=/run/user/$(id -u)/biomeos-neural-api.sock
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
cargo test --workspace             # All tests (1,300+)
cargo clippy --workspace -- -W clippy::pedantic
cargo fmt --check
cargo doc --workspace --no-deps
```

### Scenarios

Load a scenario for testing:
```bash
petaltongue ui --scenario sandbox/scenarios/paint-simple.json
petaltongue ui --scenario sandbox/scenarios/healthspring-diagnostic.json
```

### Architecture Rules

1. **Self-knowledge only** -- petalTongue knows its own name, ports, and capabilities. Other primals are discovered at runtime via socket/mDNS/JSON-RPC.
2. **Constants centralized** -- All self-knowledge lives in `petal_tongue_core::constants`. No magic strings.
3. **IPC priority** -- JSON-RPC over Unix sockets (primary), tarpc (secondary), HTTP (fallback).
4. **No `unwrap()` in production** -- Use `?`, `if let`, or graceful early return with `tracing::error!`.
5. **No `unsafe` unless FFI** -- 16/17 crates have `#![forbid(unsafe_code)]`.
6. **Concurrent testing** -- No `thread::sleep`, no serial tests except chaos. Use `tokio::time::timeout` for async.
7. **Files under 1,000 lines** -- Split into cohesive modules when approaching limit.

---

## Key Modules

### Core (`petal-tongue-core`)
- `constants.rs` -- Centralized self-knowledge (name, ports, socket names)
- `graph_engine.rs` -- Graph data model (nodes, edges, layout)
- `capability_discovery.rs` -- Runtime capability-based primal discovery
- `config_system.rs` -- XDG-compliant configuration (env > file > defaults)
- `data_channel.rs` -- `DataChannel` enum (TimeSeries, Distribution, Bar, Gauge)
- `dynamic_schema.rs` -- Flexible scenario/primal schema with version handling

### Graph (`petal-tongue-graph`)
- `visual_2d/` -- 2D force-directed graph renderer (egui)
- `chart_renderer.rs` -- Data channel visualization (egui_plot)
- `clinical_theme.rs` -- Clinical color palette (healthSpring-derived)
- `audio_sonification.rs` -- Graph-to-audio mapping

### IPC (`petal-tongue-ipc`)
- `unix_socket_server.rs` -- JSON-RPC 2.0 server over Unix sockets
- `tarpc_client.rs` -- tarpc binary RPC client
- `socket_path.rs` -- XDG-compliant socket path discovery

### Discovery (`petal-tongue-discovery`)
- `lib.rs` -- Provider discovery orchestrator
- `http_provider.rs` -- HTTP fallback provider
- `mdns_provider.rs` -- mDNS multicast discovery
- `dynamic_scenario_provider.rs` -- JSON scenario loading

---

## Cross-Primal Integration

petalTongue integrates with the ecoPrimals ecosystem:
- **biomeOS** -- Device/primal topology visualization (JSON-RPC)
- **healthSpring** -- Diagnostic data channels, clinical theme
- **ToadStool** -- Audio/display backend (tarpc, capability-discovered)
- **Songbird** -- Encrypted discovery protocol

See `ecoPrimals/wateringHole/` for inter-primal standards and handoffs.

---

## License

AGPL-3.0 -- See [LICENSE](./LICENSE).
