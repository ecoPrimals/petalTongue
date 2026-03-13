# petalTongue — Evolution Tracker

**Living document**: Updated as evolution progresses.
**Last updated**: March 13, 2026

---

## Verification Numbers

| Metric | Value |
|--------|-------|
| Tests | 3,711 passing, 5 ignored |
| Coverage (line) | 79.5% |
| Coverage (function) | 81.1% |
| Clippy | Zero errors (pedantic + nursery); advisory warnings only |
| `cargo fmt` | Clean |
| `cargo deny` | Clean (advisories, bans, licenses, sources) |
| `cargo doc` | Clean (`--all-features --no-deps`) |
| Unsafe code | `#![forbid(unsafe_code)]` on all 16 crates |
| Largest file | 988 lines (`visualization_handler/state.rs`); all files under 1,000 |
| External C deps | Zero |
| SPDX headers | All 467 source files |

---

## Completed Work

### Code Quality (Phase 1)

- All clippy warnings eliminated (pedantic level)
- `cargo deny` clean: license compliance, advisory compliance, ban compliance
- `cargo fmt` clean across workspace
- SPDX `AGPL-3.0-only` headers on all source files
- `ring` crate eliminated (ecoBin compliance)
- `users` crate replaced with pure Rust `rustix`

### Technical Debt Elimination

- Production `unwrap()` calls replaced with `expect()` or `Result` propagation
- Production stubs evolved to complete implementations:
  - `SystemInfo::discover()` — reads `/proc` (pure Rust)
  - `discover_modalities()` / `discover_compute()` — live registry counts
  - `color_category_count_with_data()` — counts unique string values
- All hardcoded localhost values → env var with constant fallback
- All mocks gated behind `#[cfg(test)]` or `#[cfg(feature = "test-fixtures")]`
- `std::env::set_var` / `remove_var` (unsafe) eliminated from tests

### Smart Refactoring

| File | Before | After | Strategy |
|------|--------|-------|----------|
| `modality.rs` | 1,232 lines | 6 modules (78-449 lines each) | Domain split by compiler |
| `app/mod.rs` | 873 lines | 532 + events.rs + panels.rs | Logic/layout/events split |
| `rendering_awareness.rs` | 850 lines | 327 + types.rs + tests.rs | Types/tests extracted |
| `tufte.rs` | 836 lines | 102 + constraints.rs + pipeline.rs + tests.rs | Domain decomposition |

### GUI Logic Extraction

Architecture principle: **all logic testable outside egui rendering context**.

Extracted pure functions from 15+ UI files:
- Graph canvas: `node_colors`, `edge_color_rgb`, `arrow_geometry`, `grid_line_positions`,
  `hit_test_nodes`, `nodes_in_rect`, `compute_zoom`
- Graph editor: `editor_node_colors`, `node_status_display`, `confidence_color_rgb`
- Keyboard: `KeyModifiers` struct, `map_key_to_action` pure function
- Traffic view: `bezier_control_points`, `primal_lane_layout`
- Timeline: `time_to_x`, `format_events_csv`, `escape_csv`
- Human entropy: `quality_color_rgb`, `format_recording_duration`
- Niche designer: `validation_display_info`, `can_deploy`
- Metrics: `threshold_color_rgb`, `sparkline_points`
- Sensory UI: 10 format helpers
- Primal details: `PrimalDetailsSummary` builder, `health_status_icon/rgb`

16 new headless integration tests for keyboard shortcuts, multi-frame state,
panel navigation, and motor command coverage.

### Property Testing

`proptest` added to `petal-tongue-core` and `petal-tongue-scene`:
- `dynamic_schema.rs`: schema detection robustness
- `modality/svg.rs`: XML output validity
- `tufte.rs`: constraint scoring consistency
- `state_sync.rs`: state serialization round-trip

---

## Completed Work (cont.)

### Live Ecosystem Wiring (March 11, 2026)

Full bidirectional pipeline between petalTongue and the ecoPrimals ecosystem:

| Component | Status | Impact |
|-----------|--------|--------|
| Game loop wiring | Complete | Enables 60 Hz animation and physics |
| IPC-to-UI bridge | Complete | External primals can render live in UI |
| Sensor event feed | Complete | UI pointer/key/scroll broadcast to IPC subscribers |
| Interaction broadcast | Complete | Selection changes broadcast to subscribers |
| Neural API registration | Complete | petalTongue self-registers with biomeOS lifecycle |
| GameDataChannel mapping | Complete | ludoSpring game data renders with game theming |
| Integration tests | Complete | Full pipeline exercised without live primals |

See `specs/REALTIME_COLLABORATIVE_PIPELINE.md` and `docs/LIVE_TESTING.md`.

### Spring Absorption (March 11, 2026)

Cross-spring patterns ingested and evolved into petalTongue core:

| Feature | Source Springs | Implementation |
|---------|---------------|----------------|
| Server-side backpressure | wetSpring, healthSpring, ludoSpring | `BackpressureConfig` in `VisualizationState` — rate limiting, cooldown, burst tolerance for 60 Hz streaming |
| JSONL telemetry ingestion | hotSpring | `TelemetryAdapter` — parses JSONL telemetry to `DataBinding::TimeSeries` by section/observable |
| Diverging color scales | neuralSpring (S139) | `DivergingScale` with `interpolate()` — three-stop color interpolation for heatmaps |
| Game domain palette | ludoSpring | 7th domain palette `(220, 160, 80)` for game/ludology visualizations |
| Session health IPC | all springs | `visualization.session.status` — queries frame count, uptime, backpressure state |
| Provider registry | toadStool (S145) | `provider.register_capability` IPC method for capability announcement |
| Pipeline DAG | neuralSpring, groundSpring | `PipelineRegistry` with topological sort, progressive binding collection, multi-stage workflows |

New files: `telemetry_adapter.rs` (core), `pipeline.rs` (IPC), updates to `domain_palette.rs`, `state.rs`, `types.rs`, `system.rs`, `mod.rs`.

+17 new tests from absorption work (3,409 total).

---

## In Progress

No active gaps. Next evolution targets:

- `visualization.interact.sync` (perspective sync mode)
- `visualization.render.stream` grammar subscription mode
- Capability-based data resolution (`"source": "capability:X"`)
- Coverage target: 90% (currently 79.5%)

### Coverage Expansion (March 12, 2026)

+302 tests (3,409 → 3,711). Major additions across 35+ files:

| Crate | Key areas covered |
|-------|-------------------|
| `petal-tongue-core` | state_sync, dynamic_schema, engine, toadstool_compute |
| `petal-tongue-ui` | traffic_view, timeline_view, scene_bridge, sensory_ui, trust_dashboard |
| `petal-tongue-graph` | domain_charts, basic_charts |
| `petal-tongue-scene` | scene_graph, animation |
| `petal-tongue-ipc` | json_rpc_client, server, tarpc_client, visualization_handler |
| `petal-tongue-discovery` | cache |
| `petal-tongue-tui` | app, state |
| `petal-tongue-cli` | resolve_instance_id, format_show_output |
| `petal-tongue-animation` | visual_flower |
| workspace root | main, cli_mode, web_mode, data_service, tui_mode, ui_mode, headless_mode |

Idiomatic Rust improvements: `let-else` patterns, moved `use` to top of
test functions, tightened drop scopes, removed stale `#[expect]` attributes.

Smart refactoring: `petal-tongue-telemetry` split into `types.rs` + `collector.rs`
(847 → 787 lines across 3 files).

---

## Architecture Principles Established

1. **Testable logic, thin rendering**: All business logic extracted into pure
   functions. Rendering is a thin adapter that calls pure functions and draws.

2. **Data-only intermediates**: `EguiShapeDesc`, `ModalityOutput`, `RenderPlan`
   are data structures, not drawing commands. Testable without a display.

3. **Capability-based discovery**: No hardcoded primal names. All external
   references use capability constants resolved at runtime.

4. **Self-knowledge only**: A primal knows its own name and capabilities.
   Everything else is discovered, never assumed.

5. **Unified delta time**: One source (`ctx.input(|i| i.stable_dt)`) feeds
   all animation, physics, and timing systems.

6. **Modality independence**: Every data visualization can be rendered as GUI,
   TUI, audio, SVG, braille, haptic, or accessibility description.

7. **Sovereignty**: No telemetry, no cloud, no vendor lock-in. Data stays local.
   Human controls modality. AGPL-3.0-only.

---

## What Each Gap Enables

### Gap 1: Game Loop → Continuous Animation

Today: petalTongue can compile a grammar expression to a scene graph and render
it. Animations exist but are not driven by a tick loop.

After: Scene animations play at 60 Hz. Physics simulations run in real time.
Molecular dynamics from barraCuda update live. The visualization *breathes*.

### Gap 2: IPC-to-UI Bridge → Live Multi-Primal Dashboard

Today: External primals can request exports (SVG, audio) via IPC. They cannot
place a live, interactive visualization in the running UI.

After: healthSpring pushes vital signs → live time series panel appears.
Squirrel detects anomaly → anomaly highlight appears in user's view. ludoSpring
measures engagement → flow state indicator updates continuously.

### Gap 3: Sensor Streaming → Human Behavior Analysis

Today: petalTongue captures user input (pointer, keys, scroll) but consumes it
internally for interaction resolution.

After: ludoSpring subscribes to sensor events → evaluates Fitts's law cost,
Hick's law decision time, engagement curves. Squirrel subscribes → adapts
visualization complexity to user's flow state. The human's interaction *feeds*
the collaborative intelligence.

---

## Coverage Target Path

| Milestone | Tests | Coverage | Date |
|-----------|-------|----------|------|
| Baseline | 2,025 | 63% / 67% | March 10 |
| Debt elimination | 2,430 | 68% / 71% | March 10 |
| Logic extraction | 3,180 | 77.4% / 79.2% | March 10 |
| Real-time pipeline | 3,211 | TBD | March 10 |
| Ecosystem wiring | 3,245 | TBD | March 11 |
| Spring absorption | 3,409 | 77.4% / 79.2% | March 11 |
| Deep coverage expansion | 3,711 | 79.5% / 81.1% | March 12 |
| Target | TBD | 90% / 90% | — |
