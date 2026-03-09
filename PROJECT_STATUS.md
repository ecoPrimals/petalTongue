# petalTongue -- Project Status

**Updated**: March 9, 2026  
**Version**: 1.4.0  
**Edition**: 2024 (all crates)

---

## Current State

| Area | Status |
|------|--------|
| Build | Clean (`cargo check --workspace`) |
| Tests | 1,427 passing, 0 failures, 14 ignored |
| Formatting | `cargo fmt --check` clean |
| Clippy | Zero warnings (`cargo clippy --all-targets -- -D warnings`) |
| Unsafe | `#![forbid(unsafe_code)]` workspace-wide, zero C deps |
| Files | All production files under 650 lines |
| License | AGPL-3.0-only, SPDX on all source files |
| Edition | 2024 (all 15 crates) |
| External C deps | None (libc/nix/atty removed, using rustix) |

---

## Known Debt

### Clippy

Zero warnings under `cargo clippy --all-targets -- -D warnings`.
Lint suppression uses `#[expect(...)]` (warns when lint no longer applies).

### Stubs and TODOs (~35 items)

Major incomplete work:
- mDNS full DNS packet building
- HTTPS client connection
- Video entropy modality
- WebSocket subscription for biomeOS events
- Canvas rendering with tiny-skia
- Windows audio direct access

### Legacy Modules (feature-gated, frozen)

- `legacy-toadstool`: Toadstool display backend stub
- `legacy-audio`: Audio providers (rodio-based)
- `legacy-http`: HTTP discovery provider

### Missing Infrastructure

- No CI/CD pipeline
- No property-based testing
- No genomeBin manifest

---

## Grammar of Graphics Evolution (Design Phase)

Three new specs define the next evolution:

1. `specs/GRAMMAR_OF_GRAPHICS_ARCHITECTURE.md` -- Composable type-safe grammar
2. `specs/UNIVERSAL_VISUALIZATION_PIPELINE.md` -- End-to-end pipeline + barraCuda
3. `specs/TUFTE_CONSTRAINT_SYSTEM.md` -- Machine-checked visualization quality

Phase 1 (foundation): `petal-tongue-grammar` crate with core traits.
See specs for full evolution path.

---

## Crate Map (15 crates)

```
petaltongue (workspace root -- UniBin entry point)
├── petal-tongue-core         Graph engine, capabilities, interaction engine, data bindings
├── petal-tongue-graph        2D rendering, charts, domain themes, audio sonification
├── petal-tongue-ui           Desktop GUI, panels, scenarios, interaction adapters
├── petal-tongue-tui          Terminal UI (ratatui)
├── petal-tongue-ipc          Unix socket IPC, JSON-RPC server, visualization handler
├── petal-tongue-discovery    Provider discovery (JSON-RPC, mDNS, Unix socket)
├── petal-tongue-entropy      Human entropy capture
├── petal-tongue-animation    Visual animations
├── petal-tongue-adapters     EcoPrimal adapter traits
├── petal-tongue-telemetry    Metrics and events
├── petal-tongue-headless     Headless binary (zero GUI deps)
├── petal-tongue-ui-core      Universal UI traits and headless renderers
├── petal-tongue-api          biomeOS JSON-RPC client
├── petal-tongue-cli          CLI parsing
└── doom-core                 Doom WAD renderer (optional)
```

Archived crates (in `archive/crates/`): `petal-tongue-primitives`, `petal-tongue-modalities`
