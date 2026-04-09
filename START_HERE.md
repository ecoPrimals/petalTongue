# petalTongue -- Start Here

**Updated**: April 9, 2026

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
cargo test --workspace --all-features           # 5,967+ tests
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
3. **IPC priority** -- JSON-RPC 2.0 REQUIRED (Unix sockets / TCP listen surface), tarpc MAY for Rust-to-Rust hot paths, HTTP for external/browser access only.
4. **Typed error handling** -- `thiserror` everywhere, no `anyhow` in production; `deny(unwrap_used, expect_used)` with `#[expect]` for justified cases.
5. **`#![forbid(unsafe_code)]`** unless hardware FFI is unavoidable. Document with `// SAFETY:`.
6. **Concurrent testing** -- No `thread::sleep`. Use `tokio::time::timeout`.
7. **Files under 1,000 lines** -- Split into cohesive modules at ~800 lines.
8. **SPDX headers** -- `// SPDX-License-Identifier: AGPL-3.0-or-later` on all `.rs` files.
9. **Semantic naming** -- JSON-RPC methods follow `{domain}.{operation}` pattern.
10. **BTSP Phase 1** -- `validate_insecure_guard()` at startup; family-scoped sockets when `FAMILY_ID` set; `BIOMEOS_INSECURE` guard prevents conflicting posture.

---

## Key Modules

### Core (`petal-tongue-core`)
- `constants/` -- Centralized self-knowledge (name, ports, socket names); submodules: `mod.rs`, `network.rs`, `display.rs`, `timeouts.rs`, `thresholds.rs`, `tufte_tolerances.rs`
- `graph_engine/` -- Graph data model (nodes, edges, layout); submodules: `mod.rs`, `types.rs`, `layout.rs`, `tests.rs`
- `config_system.rs` -- XDG-compliant configuration (env > file > defaults)
- `data_channel.rs` -- Re-exports `DataBinding` and `ThresholdRange` from `petal-tongue-types` (11 variants: TimeSeries, Distribution, Bar, Gauge, Spectrum, Heatmap, Scatter, Scatter3D, FieldMap, GameScene, Soundscape)
- `capability_names.rs` -- Centralized capability/method/socket/primal constants (62+ capabilities, 2 self-knowledge identities)
- `sensory_matrix.rs` -- Sensory Capability Matrix (input×output negotiation for consumer primals)
- `telemetry_adapter.rs` -- JSONL telemetry ingestion (hotSpring)
- `or_exit.rs` -- `OrExit<T>` trait for zero-panic validation binaries

### IPC (`petal-tongue-ipc`)
- `unix_socket_server.rs` -- JSON-RPC 2.0 server over Unix sockets
- `tarpc_client.rs` -- tarpc binary RPC client
- `tarpc_types/` -- tarpc types split into submodules
- `socket_path.rs` -- XDG-compliant socket path discovery
- `ipc_errors.rs` -- `IpcErrorPhase`, `StreamItem` (NDJSON), `DispatchOutcome<T>`, `exit_code`, `extract_rpc_error()`
- `resilience.rs` -- `CircuitBreaker`, `RetryPolicy` for IPC fault tolerance
- `discovery_helpers.rs` -- Primal socket resolution, env var helpers

### Discovery (`petal-tongue-discovery`)
- `lib.rs` -- Provider discovery orchestrator
- `unix_socket_provider.rs` -- Unix socket JSON-RPC discovery (universal fallback path)
- `neural_api_provider/` -- biomeOS Neural API discovery (provider, parse, tests)
- `discovery_service_client/` -- Discovery service capability queries (mod, protocol, methods)
- `discovery_service_provider.rs` -- Discovery service visualization provider (topology inference)
- `jsonrpc_provider/` -- JSON-RPC visualization provider
- `mdns_provider/` -- mDNS/DNS-SD zero-config discovery (optional `mdns` feature)
- `capability_parse.rs` -- 4-format capability parsing (flat, enriched, nested, result-wrapped)
- `cache.rs` -- LRU discovery result cache
- `dns_parser/` -- Pure-Rust DNS packet parser (SRV, TXT, PTR, A records); submodules: `header.rs`, `name.rs`, `record.rs`

### UI (`petal-tongue-ui`)
- `scene_bridge/paint/` (color, geometry, primitives)
- `device_panel/` (list_view, detail_view)
- `graph_editor/graph/` (validation, serialization)
- `app/init.rs` + `panel_init.rs`, `provider_init.rs`, `scenario_init.rs`

### Specs

Architectural specifications in `specs/` -- read these before making major changes:
- `GRAMMAR_OF_GRAPHICS_ARCHITECTURE.md` -- Next evolution (composable grammar)
- `UNIVERSAL_VISUALIZATION_PIPELINE.md` -- Data→render pipeline
- `TUFTE_CONSTRAINT_SYSTEM.md` -- Visualization quality constraints

---

## Cross-Primal Integration

All cross-primal connections use **capability-first discovery** via biomeOS Neural API.
petalTongue never hardcodes primal names in routing or socket resolution — names exist
only in `primal_names` constants for logging context.

- **biomeOS** -- Topology visualization (JSON-RPC), Neural API lifecycle
- **healthSpring** -- Diagnostic data channels, clinical theme, streaming sessions
- **hotSpring** -- JSONL telemetry ingestion to TimeSeries
- **neuralSpring** -- Pipeline DAGs, diverging color scales
- **wetSpring** -- Backpressure-aware streaming, Scatter 2D ordinations
- **ludoSpring** -- 7 GameDataChannel types, 60 Hz sensor feed, GameScene/Soundscape rendering
- **Display backend** -- Discovered via `display` capability (tarpc, capability-discovered)
- **Audio backend** -- Discovered via `audio.synthesize` capability
- **GPU compute** -- Discovered via `compute.dispatch` / `physics-compute` capabilities
- **Discovery service** -- Discovered via `discovery.query_capability`
- **AI agent adapter** -- `ai_adapter` (InputModality::Agent, AgentInputAdapter)

See `ecoPrimals/wateringHole/petaltongue/` for inter-primal standards.

---

## License

AGPL-3.0-or-later -- See [LICENSE](./LICENSE).
