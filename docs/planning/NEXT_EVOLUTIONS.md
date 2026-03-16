# Next Evolution Paths for petalTongue

**Current State**: v1.6.6 — 5,244 tests, ~86% line / ~87% branch coverage
**Date**: March 16, 2026

---

## Recently Completed (v1.6.6)

- **`deny(unwrap_used, expect_used)`**: Production code zero-unwrap; all 5 `expect()` evolved to safe fallbacks or justified `#[expect]`
- **`primal_names` module**: 15 primal identity constants in `capability_names.rs`
- **`#[allow]` → `#[expect]` migration**: All production `#[allow]` with documented reasons
- **Enriched `capability.list`**: Returns `primal`, `version`, `transport`, `methods`, `depends_on`, `data_bindings`, `geometry_types`
- **`SpringAdapterError` typed**: `MissingField`, `UnrecognizedFormat`, `UnsupportedChannelType` variants
- **`PrimalRegistration` uses constants**: Zero hardcoded capability strings
- **RwLock poison safety**: All `expect("lock poisoned")` → graceful fallbacks
- **Explicit ecosystem needs doc**: Published to wateringHole

## Previously Completed (v1.6.4)

- **GameScene + Soundscape DataBinding variants**: 9 → 11 channel types for ludoSpring game scenes and layered audio
- **Sprite/Tilemap/GameEntity primitives**: 2D game rendering support in `petal-tongue-scene`
- **Soundscape synthesis**: 5 waveforms, stereo panning, fade envelopes, deterministic
- **JSONL telemetry provider**: hotSpring bridge via `PETALTONGUE_TELEMETRY_DIR`
- **Capability names module**: 60 centralized constants (`self_capabilities`, `discovery_capabilities`, `methods`, `socket_roles`)
- **SpringDataAdapter evolution**: GameScene/Soundscape format detection; UiConfig + thresholds parsing from spring pushes
- **Workspace lint evolution**: pedantic/nursery fully via `[workspace.lints.clippy]`

## Previously Completed (v1.6.3)

- Typed error handling: `anyhow` eliminated from all production code, `thiserror` everywhere
- Clippy pedantic + nursery: zero warnings, enforced in CI
- Smart refactoring: trust_dashboard, scene_bridge, awakening_coordinator, domain_charts, visualization handlers decomposed into cohesive modules
- Redundant `.clone()` elimination and zero-copy improvements
- Server subcommand: `petaltongue server` runs IPC without GUI
- GPU compute endpoint configurable via `PETALTONGUE_GPU_COMPUTE_ENDPOINT`
- Workspace lint centralization: crate-level redundant lint attrs removed
- Full primal interaction wiring: 35 capabilities registered
- Provenance trio client: rhizoCrypt + sweetGrass + loamSpine
- Compute bridge expansion: math.stat.*, math.tessellate.*, math.project.*
- Niche YAML: organisms, interactions, customization sections

---

## Active Evolution Targets

### Coverage: 87% -> 90% (Short-term)

Per-crate gaps:
- `petal-tongue-ui`: 75.7% (egui rendering adapter layer)
- `petal-tongue-api`: 83.5%
- `petal-tongue-ui-core`: 86.3%
- `petal-tongue-animation`: 86.9%

Strategy: deeper headless scenario coverage and mock network infrastructure.
The remaining gap is concentrated in egui `Painter`/`Ui` drawing calls and
async network I/O paths.

### Live Integration Testing (Short-term)

End-to-end with running springs and primals:
- Test tarpc with actual Toadstool/Songbird servers
- Validate visualization.render with live spring data
- Measure real-world latency improvements

### ToadStool Display Backend (Medium-term)

Wire `display.*` methods to ToadStool for GPU-accelerated rendering:
- Direct framebuffer via tarpc
- Capability-discovered display provider
- CPU software renderer fallback

### Cross-Compilation (Medium-term)

- armv7 (Raspberry Pi / embedded)
- macOS (aarch64-apple-darwin)
- Windows (x86_64-pc-windows-msvc)
- WASM (wasm32-unknown-unknown) for web mode

### barraCuda Compute Parity (Medium-term)

Offload heavy visualization compute via IPC:

| Capability | Status |
|-----------|--------|
| `math.physics.nbody` | Implemented (CPU fallback) |
| `math.stat.kde` | Types defined, no IPC client |
| `math.stat.smooth` | Types defined, no IPC client |
| `math.stat.bin` | Types defined, no IPC client |
| `math.tessellate.*` | Types defined, no IPC client |
| `math.project.*` | Types defined, no IPC client |

### GPU Shader Promotion (Long-term)

Tier A (pure math, ready for WGSL): math_objects, transform, domain_palette,
tufte, animation (~5 modules). Tier B (needs abstraction): compiler, physics.
Tier C (stays CPU): UI, IPC, discovery, scene_graph.

### Fuzz Testing (Long-term)

`cargo-fuzz` harnesses for:
- JSON-RPC message parsing
- Grammar expression compilation
- Scenario JSON loading
- Dynamic schema migrations

---

## Completed Phases

| Phase | Version | Summary |
|-------|---------|---------|
| Core Proprioception | v1.0.0 | SAME DAVE bidirectional feedback |
| UI Integration | v1.1.0 | Real-time proprioception display |
| Self-Healing | v1.2.0 | Hang detection, FPS monitoring |
| Ecosystem Alignment | v1.3.0 | tarpc PRIMARY, protocol selection |
| Grammar of Graphics | v1.4.0 | Composable pipeline, Tufte constraints |
| Dashboard Engine | v1.5.0 | Multi-panel grid, domain theming |
| Deep Debt Evolution | v1.6.3 | Typed errors, pedantic lints, smart refactoring |
| Spring Absorption | v1.6.4 | GameScene/Soundscape DataBinding, sprite/soundscape primitives, JSONL telemetry, capability_names |
| Ecosystem Evolution | v1.6.5 | deny(unwrap/expect), primal_names, enriched capability.list, #[expect] migration |
| UUI & Spring Absorption | v1.6.6 | Universal User Interface evolution, safe casts, tempfile isolation, ecosystem doc propagation |
