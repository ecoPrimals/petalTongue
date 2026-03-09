# petalTongue -- Project Status

**Updated**: March 9, 2026  
**Version**: 1.4.4  
**Edition**: 2024 (all crates)

---

## Current State

| Area | Status |
|------|--------|
| Build | Clean (`cargo check --workspace`) |
| Tests | 1,914 passing, 0 failures, 2 ignored |
| Formatting | `cargo fmt --check` clean |
| Clippy | Zero warnings, pedantic enabled (`clippy::pedantic` via workspace lints) |
| Rustdoc | Clean (`RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`) |
| cargo deny | Clean (advisories, bans, licenses, sources) |
| Unsafe | `#![forbid(unsafe_code)]` workspace-wide, zero C deps, zero `unsafe` blocks |
| Files | All production files under 650 lines |
| License | AGPL-3.0-only, SPDX on all source and config files |
| Edition | 2024 (all 16 crates) |
| External C deps | None (`ring` eliminated, `libc`/`nix`/`atty` removed, using `rustix`) |
| ecoBin | Compliant (no ring, aws-lc-sys, openssl-sys, native-tls, zstd-sys) |
| Coverage | 63% line / 67% function (llvm-cov, workspace) |
| JSON-RPC | Semantic method naming (`domain.operation`), 15 visualization methods |
| Mocks | All gated behind `#[cfg(test)]` or `#[cfg(feature = "mock")]` |
| Primal names | Capability-based constants, zero hardcoded external primal names |
| Domain theming | 6 domain palettes (health, physics, ecology, agriculture, measurement, neural) |
| Spring IPC | healthSpring callback subscriptions, wetSpring Spectrum renderer, physics bridge |

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

### Stubs and TODOs (~47 items)

Major incomplete work (delegated to other primals or future phases):
- mDNS full DNS packet building (delegate to songbird)
- HTTPS client connection (delegate to beardog/songbird via IPC)
- Video/Visual/Gesture entropy modalities (phases 3-6)
- WebSocket subscription for biomeOS events
- Canvas rendering with tiny-skia
- Windows audio direct access
- Audio socket/software backend completion (ToadStool integration)
- CSV export for timeline view
- TUI force-directed layout

### Remaining Evolution Targets (P2)

- `visualization.interact.sync` (perspective sync mode) -- needs multi-user state
- `visualization.render.stream` grammar subscription mode -- needs capability-based data resolution
- Capability-based data resolution (`"source": "capability:X"`) -- requires Songbird integration

### Test Coverage Gap

Current: 63% line coverage, 67% function coverage (1,914 tests).
Target: 90%.

Well-covered areas (>80%):
- Core engine, graph builder, graph validation, types, interaction engine
- Session, data channel, telemetry, data bindings, config, constants
- Discovery (JSON-RPC, HTTP, Songbird, cache, unix socket)
- IPC (Unix socket, tarpc client, JSON-RPC handlers, server)
- Scenario builder, domain theme, filtering, timeline types
- Rendering awareness, state sync, awakening coordinator, sensor
- Dynamic schema, instance lifecycle/registry, capabilities
- CLI argument parsing, process viewer, graph metrics
- Proprioception, sensory capabilities, display traits, entropy state
- TUI rendering via TestBackend (all 8 views tested)

Remaining uncovered areas (require display/terminal runtime):
- egui app module, app_panels/builders, app/init: 0% (require egui Context)
- Visual 2D renderer, interaction, animation: 0% (require egui)
- Graph canvas, niche designer rendering, sensory UI renderers: 0%
- System dashboard panels, human entropy rendering: 0%
- Chart renderer rendering bodies: 0% (require egui `Ui`)

Strategy: Logic extraction pattern -- extract pure data transforms from
rendering functions, test those. Rendering itself needs headless egui or
screenshot-based testing (future infrastructure).

### Legacy Modules (feature-gated, frozen)

- `legacy-toadstool`: Toadstool display backend stub
- `legacy-audio`: Audio providers (rodio-based)
- `legacy-http`: HTTP discovery provider

### Missing Infrastructure

- No CI/CD pipeline
- No property-based testing
- No genomeBin manifest

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
