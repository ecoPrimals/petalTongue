# petalTongue -- Project Status

**Updated**: March 11, 2026  
**Version**: 1.6.1  
**Edition**: 2024 (all crates)

---

## Current State

| Area | Status |
|------|--------|
| Build | Clean (`cargo check --workspace`) |
| Tests | 3,245 passing, 0 failures |
| Formatting | `cargo fmt --check` clean |
| Clippy | Zero warnings, pedantic tightened (removed float_cmp, cast_*, too_many_lines, needless_pass_by_value allows; all uses `#[expect]` with documented reasons) |
| Rustdoc | Clean (`RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`) |
| cargo deny | Clean (advisories, bans, licenses, sources) |
| Unsafe | `#![forbid(unsafe_code)]` workspace-wide, zero C deps, zero `unsafe` blocks |
| Files | All production files under 1,000 lines (largest: 968) |
| License | AGPL-3.0-only, SPDX on all source and config files |
| Edition | 2024 (all 16 crates) |
| External C deps | None (`ring` eliminated, `libc`/`nix`/`atty` removed, using `rustix`) |
| ecoBin | Compliant (no ring, aws-lc-sys, openssl-sys, native-tls, zstd-sys) |
| Coverage | 77.4% region / 79.2% function (llvm-cov, workspace) — target 90% |
| JSON-RPC | Semantic method naming (`domain.operation`), 16 visualization methods |
| Mocks | All gated behind `#[cfg(test)]` or `#[cfg(feature = "test-fixtures")]`; PETALTONGUE_MOCK_MODE test-only |
| Primal names | Capability-based constants, zero hardcoded external primal names |
| Hardcoding | Socket names, ports, endpoints all configurable via env vars |
| Domain theming | 6 domain palettes (health, physics, ecology, agriculture, measurement, neural) |
| GUI logic extraction | All UI business logic in pure functions, 16 headless integration tests |
| Game loop | Wired: TickClock in app, begin_frame_with_dt, continuous mode motor commands |
| IPC-to-UI bridge | Complete: shared VisualizationState, LiveSessionsPanel, session polling |
| Sensor streaming | Complete: SensorEventBatch/IPC types, SensorStreamRegistry, subscribe/unsubscribe/poll handlers |
| Sensor event feed | Complete: UI events broadcast to IPC subscribers via `sensor_feed.rs` |
| Interaction broadcast | Complete: Selection changes broadcast to `InteractionSubscriberRegistry` |
| Neural API registration | Complete: `lifecycle.register` + 30s heartbeat via `neural_registration.rs` |
| GameDataChannel mapping | Complete: 7 ludoSpring channels → DataBinding variants via `game_data_channel.rs` |
| Spring IPC | healthSpring DataChannel auto-compile, dashboard layout, wetSpring Scatter/Spectrum, physics bridge, interaction aliases |
| DataChannel compiler | All 9 DataBinding variants (incl. Scatter 2D) auto-compiled to Grammar of Graphics |
| Dashboard engine | Multi-panel grid layout with domain theming and SVG export |
| Scenario loader | JSON scenario files loaded from disk; `--scenario` CLI flag |
| Geometry | Point, Line, Bar, Area, Ribbon (stub), Tile (heatmap/fieldmap), Arc (gauge), Mesh3D (stub) |
| Faceting | `compile_faceted()` supports `FacetLayout::Wrap` and Grid small multiples |
| Threshold coloring | `compile_with_thresholds()` maps ThresholdRange status to domain palette colors |
| Dependencies | Updated: base64→0.22, socket2→0.6, lru→0.16, mdns-sd→0.18 |

---

## Architecture

### IPC-First Design (JSON-RPC + tarpc)

- **JSON-RPC 2.0**: Primary protocol for local IPC (Unix sockets)
- **tarpc**: High-performance binary RPC with `bytes::Bytes` for zero-copy payloads
- **Semantic naming**: All methods follow `{domain}.{operation}` convention
- **Legacy fallbacks**: Clients try semantic names first, fall back to legacy for compatibility

### ecoBin Compliance

- **No TLS in petalTongue**: HTTP calls are localhost-only (biomeOS, discovery)
- **HTTPS delegated**: beardog/songbird provide pure Rust TLS via biomeOS tower atomic
- **reqwest**: Configured without TLS features (no ring, no aws-lc-sys)
- **Zero C dependencies**: All system calls via `rustix`, all crypto via RustCrypto

### Sovereignty & Human Dignity

- **Self-knowledge only**: Primal knows its own name/capabilities, discovers others at runtime
- **Capability-based discovery**: Socket names, service names configurable via env vars
- **No hardcoded external primal names**: All references use capability constants
- **Accessibility-first**: Multi-modal rendering (GUI, TUI, audio, SVG, headless)
- **Tufte constraints**: Machine-checked visualization quality

---

## Known Debt

### Stubs and TODOs (~1 item in active code)

Active TODO: Animation capability test exception in `capability_integration_tests.rs`.

Delegated/roadmap items (not TODOs, documented as roadmap markers):
- mDNS full DNS packet building (delegate to songbird)
- HTTPS client connection (delegate to beardog/songbird via IPC)
- Video/Visual/Gesture entropy modalities (phases 3-6)
- WebSocket subscription for biomeOS events
- Canvas rendering with tiny-skia
- Windows audio direct access (WDM/WASAPI platform integration)
- Audio hardware playback via capability provider
- TUI force-directed layout

### Remaining Evolution Targets (P2)

- `visualization.interact.sync` (perspective sync mode) -- needs multi-user state
- `visualization.render.stream` grammar subscription mode -- needs capability-based data resolution
- Capability-based data resolution (`"source": "capability:X"`) -- requires Songbird integration

### Test Coverage Gap

Current: 77.4% region coverage, 79.2% function coverage (3,245 tests).
Target: 90%.

Well-covered areas (>80%):
- Core engine, graph builder, graph validation, types, interaction engine
- Session, data channel, telemetry, data bindings, config, constants
- Discovery (JSON-RPC, HTTP, Songbird, cache, unix socket)
- IPC (Unix socket, tarpc client, JSON-RPC handlers, server, visualization)
- Scenario builder, domain theme, filtering, timeline types
- Rendering awareness, state sync, awakening coordinator, sensor
- Dynamic schema, instance lifecycle/registry, capabilities
- CLI argument parsing, process viewer, graph metrics
- Proprioception, sensory capabilities, display traits, entropy state
- TUI rendering via TestBackend (all 8 views tested)
- Scene engine: determinism tests (100-run identical output), modality round-trips
- Math objects: edge cases (NaN, infinity, degenerate parameters)
- Animation: all easing functions at boundaries, sequences, zero duration
- Tufte constraints: data-ink ratio, chartjunk, data density

Remaining uncovered areas:
- egui rendering adapter layer (thin `Ui` calls after pure logic extraction)
- Visual 2D renderer, animation renderer (egui-dependent drawing)
- Chart renderer rendering bodies (egui `Ui` drawing)

Strategy: Logic extraction complete — all business logic in pure testable
functions. Remaining coverage gap is the thin rendering adapter layer, which
requires the headless harness for integration testing. 16 headless integration
tests already cover keyboard shortcuts, motor commands, and panel navigation.

### Legacy Modules (feature-gated, frozen)

- `legacy-toadstool`: Toadstool display backend stub
- `legacy-audio`: Audio providers (rodio-based)
- `legacy-http`: HTTP discovery provider

### Missing Infrastructure

- No CI/CD pipeline
- No genomeBin manifest
- No validation binaries (hotSpring pattern: hardcoded expected, exit 0/1)
- No Python baselines for cross-validation
- No benchmark suite (criterion/iai/divan)

Property-based testing (`proptest`) added for: dynamic_schema, SVG modality,
Tufte constraints, state sync.

---

## Ecosystem Alignment (March 11, 2026)

### Primal Versions Tracked

| Primal | Version | Aligned |
|--------|---------|---------|
| barraCuda | v0.3.3 (unreleased HEAD) | Yes — `compute.dispatch` uses `op` field; ecosystem discovery |
| toadStool | S141 (Mar 10) | Yes — dual-write discovery at `$XDG_RUNTIME_DIR/ecoPrimals/` |
| coralReef | Phase 10, Iter 26 | N/A — petalTongue does not call coralReef directly |
| groundSpring | V100 (Mar 8) | Reviewed — sovereign rewire guidance applied |
| ludoSpring | V2 (Mar 11) | Yes — 7 GameDataChannel types mapped, sensor stream wired |
| biomeOS | Tower stable (Mar 11) | Yes — Neural API `lifecycle.register` + heartbeat |

### IPC Contract Status

| Contract | Aligned | Notes |
|----------|---------|-------|
| `barracuda.compute.dispatch` | Partial | Uses `op` field (v0.3.3+); `math.physics.nbody` wired but not in barraCuda dispatch table |
| ToadStool display IPC | Planned | `display.*` methods defined; integration is Phase 2 |
| coralReef `compiler.*` | N/A | petalTongue does not compile shaders |
| Ecosystem discovery | Yes | Scans `$XDG_RUNTIME_DIR/ecoPrimals/discovery/` per S139 |

### Capability Constants

| Constant | Source | Used In |
|----------|--------|---------|
| `gpu.dispatch` | barraCuda v0.3.3 | toadstool_compute.rs, physics_bridge.rs |
| `science.gpu.dispatch` | toadStool S139 | toadstool_compute.rs |
| `display` | toadStool display backend | toadstool_compute.rs |
| `shader.compile` | coralReef Phase 10 | toadstool_compute.rs (noted, not used) |

---

## Evolution Readiness (GPU Shader Promotion)

Evolution path: Python baseline → barraCuda CPU → GPU kokoks → GPU barraCuda → sovereign pipeline

### Module-to-Shader Mapping

| Rust Module | Tier | WGSL Shader Target | Pipeline Stage | Blocking |
|-------------|------|-------------------|----------------|----------|
| `math_objects.rs` (NumberLine, Axes, FunctionPlot) | A: Ready | `petal_math.wgsl` | Compute | Pure math, no I/O |
| `transform.rs` (Transform2D, Transform3D) | A: Ready | `petal_transform.wgsl` | Compute | Pure affine/matrix ops |
| `domain_palette.rs` (categorical colors) | A: Ready | `petal_palette.wgsl` | Compute | Pure color math |
| `tufte.rs` (constraint evaluation) | A: Ready | `petal_tufte.wgsl` | Compute | Pure validation logic |
| `animation.rs` (easing functions) | A: Ready | `petal_easing.wgsl` | Compute | Pure interpolation |
| `compiler.rs` (grammar → scene) | B: Adapt | `petal_compiler.wgsl` | Compute | Needs trait abstraction for GPU dispatch; JSON parsing stays CPU |
| `physics.rs` (PhysicsWorld types) | B: Adapt | Already uses barraCuda IPC | Compute | Types serialize; compute delegated to barraCuda |
| `modality.rs` (SVG/audio/terminal) | C: New | N/A | Render | Platform-specific output, not promotable |
| `scene_graph.rs` (hierarchical tree) | C: New | N/A | CPU-only | Tree traversal inherently sequential |
| UI crates (egui, ratatui) | C: New | N/A | Render | Platform-specific |
| IPC crates (tarpc, JSON-RPC) | C: New | N/A | Transport | Platform-specific |
| Discovery crate | C: New | N/A | Transport | Platform-specific |

### Tier Definitions

- **Tier A (Ready)**: Pure computation, no I/O, no allocation, composable. Can be
  transcribed to WGSL with minimal changes. ~5 modules.
- **Tier B (Adapt)**: Needs trait abstraction or data format changes for GPU dispatch.
  CPU fallback required. ~2 modules.
- **Tier C (New)**: Platform-specific or inherently sequential. Stays on CPU. ~8+ modules.

### barraCuda Integration Status

| Capability | Spec | Implemented | Gap |
|-----------|------|-------------|-----|
| `math.physics.nbody` | Yes | Yes (physics_bridge.rs) | CPU fallback only; no GPU parity test |
| `math.physics.md_forces` | Yes | Types only | No IPC client |
| `math.stat.kde` | Yes | No | Offload at ≥10K rows not implemented |
| `math.stat.smooth` | Yes | No | LOESS/moving average not implemented |
| `math.stat.bin` | Yes | No | Histogram binning not implemented |
| `math.stat.summary` | Yes | No | Grouped aggregation not implemented |
| `math.tessellate.*` | Yes | No | 3D tessellation not implemented |
| `math.project.*` | Yes | No | 3D projection not implemented |

### Python Baseline Status

petalTongue is a visualization primal, not a compute Spring. The evolution
path for petalTongue is:

1. **Rust-native** (current): Grammar of Graphics compiler, modality compilers, Tufte constraints
2. **barraCuda delegation** (next): Heavy statistics/3D offloaded via IPC
3. **ToadStool integration** (future): GPU display pipeline, direct framebuffer

Python benchmarks exist in the compute Springs (healthSpring, groundSpring,
airSpring, wetSpring) for their numerical algorithms. petalTongue's validation
targets are visualization-specific (compile latency, rendering fidelity,
Tufte constraint compliance) rather than numerical parity.

### Cross-Spring Benchmark Infrastructure

Other Springs have extensive Python → Rust → GPU validation:
- **healthSpring**: `bench_barracuda_cpu_vs_python.py` (Hill eq, PK, Shannon/Simpson)
- **groundSpring**: `bench_barracuda_cpu_vs_python.py` (FAO-56, decompose, rawr)
- **airSpring**: 24/24 CPU parity at 20.6× speedup, 1,237/1,237 Python baselines
- **wetSpring**: 35/35 domains bit-identical to SciPy/NumPy, 173/173 PASS chain
- **hotSpring**: Kokkos-CUDA parity (9 cases, gap 27× → 3.7×)

---

## Scene Engine (petal-tongue-scene)

The `petal-tongue-scene` crate implements the declarative visualization layer
defined in specs. This is the bridge between grammar expressions and rendered
output.

| Module | Purpose |
|--------|---------|
| `primitive.rs` | Atomic rendering primitives (Point, Line, Rect, Text, Polygon, Arc, BezierPath, Mesh) |
| `transform.rs` | 2D affine and 3D transforms |
| `scene_graph.rs` | Hierarchical scene graph (SceneNode, flatten, find_by_data_id) |
| `animation.rs` | Easing functions, AnimationTarget, Sequence (Sequential/Parallel/Group) |
| `math_objects.rs` | Manim-style math objects (NumberLine, Axes, FunctionPlot, ParametricCurve, VectorField) |
| `grammar.rs` | Grammar of Graphics expression types (GrammarExpr, VariableBinding, scales, facets) |
| `tufte.rs` | Machine-checkable Tufte constraints (Data-Ink Ratio, Lie Factor, Chartjunk, Accessibility) |
| `compiler.rs` | Grammar compiler (GrammarExpr + data to SceneGraph, with constraint evaluation) |
| `modality.rs` | Modality compilers (SvgCompiler, AudioCompiler, DescriptionCompiler, TerminalCompiler) |
| `domain_palette.rs` | Domain-specific color palettes (6 domains: health, physics, ecology, agriculture, measurement, neural) |
| `physics.rs` | Physics bridge (PhysicsWorld, IPC serialization for barraCuda N-body/molecular dynamics) |

Related crates:
- `petal-tongue-ui/src/scene_bridge.rs` -- egui scene graph renderer
- `petal-tongue-tui/src/scene_bridge.rs` -- ratatui scene graph renderer
- `petal-tongue-ipc/src/physics_bridge.rs` -- async barraCuda IPC client with CPU fallback

Related specs:
- `specs/GRAMMAR_OF_GRAPHICS_ARCHITECTURE.md` -- Composable type-safe grammar
- `specs/UNIVERSAL_VISUALIZATION_PIPELINE.md` -- End-to-end pipeline + barraCuda
- `specs/TUFTE_CONSTRAINT_SYSTEM.md` -- Machine-checked visualization quality

---

## Crate Map (16 crates)

```
petaltongue (workspace root -- UniBin entry point)
├── petal-tongue-core         Graph engine, capabilities, interaction engine, data bindings
├── petal-tongue-graph        2D rendering, charts, domain themes, audio sonification
├── petal-tongue-ui           Desktop GUI, panels, scenarios, interaction adapters
├── petal-tongue-tui          Terminal UI (ratatui)
├── petal-tongue-ipc          Unix socket IPC, JSON-RPC server, visualization handler
├── petal-tongue-discovery    Provider discovery (JSON-RPC, mDNS, Unix socket)
├── petal-tongue-scene        Scene graph, animation, grammar compiler, Tufte constraints
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
