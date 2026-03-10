# Quick Start -- petalTongue

**Last Updated**: March 9, 2026
**Version**: 1.5.0

---

## Build & Run

```bash
cargo build --release

petaltongue ui          # Desktop GUI (egui)
petaltongue tui         # Terminal UI (ratatui)
petaltongue web         # Web server (axum)
petaltongue headless    # Headless rendering (SVG/PNG/JSON)
petaltongue status      # System status
```

## Development

```bash
cargo test --workspace                          # 1,896 tests
cargo clippy --workspace -- -D warnings         # Lint (pedantic, clean)
cargo fmt --check                               # Format check (clean)
cargo doc --workspace --no-deps                 # Docs (clean)
```

## With Scenarios

```bash
petaltongue ui --scenario sandbox/scenarios/paint-simple.json
petaltongue ui --scenario sandbox/scenarios/healthspring-diagnostic.json
```

## Configuration

Priority: Environment > Config file > Defaults.

```bash
export PETALTONGUE_WEB_PORT=8080
export BIOMEOS_NEURAL_API_SOCKET=/run/user/$(id -u)/biomeos-neural-api.sock
```

See [ENV_VARS.md](../../ENV_VARS.md) for full reference.

---

## Project Structure

```
petalTongue/
├── crates/          # 16 Rust crates
├── specs/           # Architectural specifications
├── sandbox/         # Scenarios, scripts, mock server
├── archive/         # Fossil record (archived code, docs)
└── docs/            # Supplementary documentation
```

## Key Documentation

- [README.md](../../README.md) -- Project overview
- [START_HERE.md](../../START_HERE.md) -- Development guide
- [PROJECT_STATUS.md](../../PROJECT_STATUS.md) -- Current metrics

---

**License**: AGPL-3.0-only
