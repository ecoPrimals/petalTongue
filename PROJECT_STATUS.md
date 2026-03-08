# petalTongue -- Project Status

**Updated**: March 8, 2026
**Version**: 2.0.0

---

## Current State

| Area | Status |
|------|--------|
| Build | Clean (`cargo check --workspace`) |
| Tests | 1,300+ passing, 0 failures |
| Clippy | Pedantic pass, 0 errors |
| Docs | `cargo doc` clean |
| Unsafe | `#![forbid(unsafe_code)]` on 16/17 crates |
| Files | All under 1,000 lines |
| License | AGPL-3.0 on all crates |

---

## Completed (March 2026 session)

### Deep Debt Evolution
- Evolved all production `unwrap()`/`expect()` to proper error handling
- Evolved all hardcoded ports, paths, URLs to centralized `constants` module
- Evolved all `thread::sleep` out of tests (concurrent-first testing)
- Evolved all env-var race conditions in e2e tests to explicit path isolation
- Evolved entropy stubs (gesture, narrative, visual) to real Shannon entropy implementations

### healthSpring Absorption
- Absorbed `DataChannel` enum and `ClinicalRange` struct from healthSpring
- Absorbed chart renderers (`draw_timeseries`, `draw_distribution`, `draw_bar_chart`, `draw_gauge`)
- Absorbed clinical theme (`HEALTHY`, `WARNING`, `CRITICAL`, color palette)
- Created `healthspring-diagnostic.json` scenario for integration testing
- Fixed `DynamicData` version field deserialization (string vs struct)
- Fixed scenario node position loading (was always 0,0)

### Code Quality
- `#![forbid(unsafe_code)]` adopted on 16 crates (healthSpring pattern)
- Clippy pedantic warnings reduced ~60% via crate-level allows and targeted fixes
- Format string inlining, redundant closures, collapsible if-statements auto-fixed
- Smart refactoring: `app.rs`, `visual_2d.rs`, `form.rs` split into cohesive modules
- Zero-copy: IPC buffers evolved to `bytes::Bytes`, discovery cache to `Arc<T>`

### Architecture
- All files under 1,000 lines (was 3 over limit)
- `egui_plot` added for chart rendering
- `NodeDetail` struct for rendering full node panels with data channels
- Centralized constants: `PRIMAL_NAME`, `DEFAULT_WEB_PORT`, `BIOMEOS_SOCKET_NAME`, etc.
- Helper functions: `default_web_bind()`, `biomeos_legacy_socket()`

---

## Remaining Work

### Valid TODOs (~58 items)
Most are Phase 2/3 features:
- ToadStool audio/display backend integration (external team dependency)
- mDNS full packet building in `mdns_provider`
- WebSocket subscription for biomeOS events
- JSON-RPC/HTTPS client in protocol selection
- Canvas rendering with `tiny-skia`
- PNG generation with `image` crate
- Video modality in entropy capture
- Force-directed layout in TUI topology view
- CSV export in timeline view

### Aspirational
- `cargo llvm-cov` coverage analysis
- Regex crate for form validation patterns
- petalTongue signature sound design
- Application icon

---

## Crate Map

```
petaltongue (workspace root -- UniBin entry point)
├── petal-tongue-core         Graph engine, capabilities, config, constants
├── petal-tongue-graph        2D rendering, charts, clinical theme, audio
├── petal-tongue-ui           Desktop GUI, panels, scenarios
├── petal-tongue-tui          Terminal UI
├── petal-tongue-ipc          Unix socket + TCP IPC, tarpc types
├── petal-tongue-discovery    Provider discovery (HTTP, JSON-RPC, mDNS)
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
