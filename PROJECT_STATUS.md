# petalTongue -- Project Status

**Updated**: March 8, 2026  
**Version**: 1.3.0  
**Edition**: 2024

---

## Current State

| Area | Status |
|------|--------|
| Build | Clean (`cargo check --workspace`) |
| Tests | 1,309 passing, 0 failures, 23 ignored |
| Formatting | `cargo fmt --check` clean |
| Clippy | 76 errors (`-D warnings`), 487 warnings total |
| Docs | `cargo doc` builds, 141 warnings |
| Coverage | 54.10% line (target: 90%) |
| Unsafe | `#![forbid(unsafe_code)]` on 5/17 crates |
| Files | All under 1,000 lines (max: 833) |
| License | AGPL-3.0-only, SPDX on 289+ files |

---

## Known Debt

### Clippy (76 errors under `-D warnings`)

| Category | Count |
|----------|-------|
| Missing struct field docs | 31 |
| Unnecessary `Result` wrapping | 8 |
| More than 3 bools in struct | 8 |
| Unused `self` argument | 4 |
| Precision-loss casts (`u64 as f32`, etc.) | 9 |
| Other (items after statements, identical match arms, etc.) | 16 |

### Coverage Gaps (54% → 90% target)

Worst-covered modules:
- `sensory_ui.rs` -- 0%
- `status_reporter.rs` -- 0%
- `system_monitor_integration.rs` -- 0%
- `system_dashboard.rs` -- 17%
- `main.rs` -- 19%
- `traffic_view.rs` -- 24%
- `trust_dashboard.rs` -- 36%

### Stubs and TODOs (~60 items)

Major incomplete work:
- Toadstool display backend (entire backend is stub)
- JSON-RPC client in protocol_selection.rs
- mDNS full packet building
- PNG/SVG rendering in modalities crate
- Video entropy modality
- WebSocket subscription for biomeOS events

### Hardcoded Values

- Primal names in production code (beardog, songbird, toadstool in ~6 files)
- Default ports (3000, 8080) in constants.rs
- Socket paths in jsonrpc_provider.rs

### Missing Infrastructure

- No `clippy.toml` / `rustfmt.toml` / `deny.toml`
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

## Crate Map

```
petaltongue (workspace root -- UniBin entry point)
├── petal-tongue-core         Graph engine, capabilities, config, constants
├── petal-tongue-graph        2D rendering, charts, clinical theme, audio
├── petal-tongue-ui           Desktop GUI, panels, scenarios
├── petal-tongue-tui          Terminal UI
├── petal-tongue-ipc          Unix socket + TCP IPC, tarpc types
├── petal-tongue-discovery    Provider discovery (JSON-RPC, HTTP, mDNS)
├── petal-tongue-entropy      Human entropy capture
├── petal-tongue-animation    Visual animations
├── petal-tongue-adapters     EcoPrimal adapter traits
├── petal-tongue-primitives   UI primitives (forms, tables, trees)
├── petal-tongue-modalities   SVG/PNG modalities
├── petal-tongue-telemetry    Metrics and events
├── petal-tongue-headless     Headless binary
├── petal-tongue-ui-core      Universal UI traits
├── petal-tongue-api          biomeOS JSON-RPC client
├── petal-tongue-cli          CLI parsing
└── doom-core                 Doom WAD renderer
```
