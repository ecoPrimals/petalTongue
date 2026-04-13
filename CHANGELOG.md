# Changelog

All notable changes to petalTongue are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Sprint 6 — Deep Debt Resolution & Compliance Elevation (April 12, 2026)

#### Added
- **`--socket` CLI flag** on `server` subcommand: explicit UDS path override for
  nucleus_launcher.sh and start_primal.sh alignment. Wired through
  `UnixSocketServer::new_with_socket()` builder — no unsafe env mutation.
  Resolves "petalTongue not starting in NUCLEUS" launcher mismatch.
- **CONTEXT.md** (T8 compliance): 98-line context file per `PUBLIC_SURFACE_STANDARD`.
- **6 companion test files**: `topology_tests.rs`, `interaction_tests.rs`,
  `tutorial_mode_tests.rs`, `startup_audio_tests.rs`, `biomeos_client_tests.rs`,
  `game_scene_renderer_tests.rs` — extracted from production files.

#### Changed
- **Discovery doc evolution**: 30+ production doc comments evolved from primal-brand
  names (Songbird, ToadStool, etc.) to capability-based language ("registry provider",
  "compute provider"). Feature name `toadstool-wasm` → `compute-wasm`.
- **Smart file refactoring** (6 files): Tests extracted to companion files.
  `interaction.rs` 690→246, `tutorial_mode.rs` 690→345, `startup_audio.rs` 675→397,
  `biomeos_client.rs` 684→416, `game_scene_renderer.rs` 692→532,
  `topology.rs` 735→415. Zero production files >700 LOC.
- **Idiomatic Rust**: 22 `format!("{}", x)` → `x.to_string()` across TUI/UI crates.
- **`#[allow(dead_code)]`** in shared test helpers documented with reason comments
  (known Rust limitation: per-binary conditional lint).
- **PII scrubbed**: `/home/user/` test path → `/tmp/scenarios/`.
- **Cargo.toml comments**: Updated to capability-based language across `petal-tongue-ui`,
  `petal-tongue-graph`.

#### Removed
- **Dead `crossterm` dependency** from `petal-tongue-core` (optional, never activated).

#### Verified
- `cargo fmt --check` ✅
- `cargo clippy --workspace --all-features -D warnings` ✅ (0 warnings)
- `cargo doc --workspace --all-features -D warnings` ✅
- `cargo test --workspace --all-features` ✅ (6,087 passed, 0 failures)
- `cargo deny check` ✅ (advisories, bans, licenses, sources ok)
- `ring` absent from dep tree (default AND `--all-features`)

---

### Added
- **BTSP Phase 1** (`crate::btsp`): `validate_insecure_guard()` refuses startup
  when both `FAMILY_ID` and `BIOMEOS_INSECURE=1` are set. Family-scoped socket
  naming (`petaltongue-{family_id}.sock`) in production posture. Domain symlink
  (`visualization-{family_id}.sock`) follows `PRIMAL_SELF_KNOWLEDGE_STANDARD`.
  Guard runs before any subcommand in `main.rs`. (PT-08)
- **BTSP Phase 2 handshake stub** (`HandshakePolicy`): Logs production posture
  warning at server startup when `FAMILY_ID` is set but BearDog enforcement is
  not yet active. Ready for BearDog `btsp.session.*` integration. (PT-09)
- **PT-04 HTML export product validation**: End-to-end test exercises
  `wrap_svg_in_html` → validate → write → read-back → structural check.
- **Audio backend feature gates**: `audio-socket` and `audio-direct` features
  (opt-in, not default). Stubs now return typed `AudioError` variants instead
  of silently succeeding. Module docs describe what is needed for full
  implementation.

### Changed
- **`anyhow` eliminated from ALL production dependencies** including root binary.
  Moved to `[dev-dependencies]` in root, `petal-tongue-ui`, `petal-tongue-tui`,
  `petal-tongue-discovery`; removed entirely from `petal-tongue-ipc`,
  `petal-tongue-api`, `petal-tongue-graph`. `impl From<anyhow::Error> for
  UiError` bridge deleted.
- **`#[allow(` → `#[expect(`** migration: 9 of 11 instances migrated with reason
  annotations. Only 2 legitimate `dead_code` suppressions on shared test helpers
  remain as `#[allow(`.
- **Self-knowledge enforcement**: `BEARDOG`/`SONGBIRD` primal constants now
  gated behind `#[cfg(feature = "test-fixtures")]` — production builds have
  zero compile-time knowledge of other primal identities. Sandbox updated to
  enable `test-fixtures`.
- **Clone/allocation density reduction** in hot paths:
  - `property_panel`: `clone_from` for HashMap capacity reuse, in-place
    `get_mut` + `text_edit_singleline` eliminates per-frame string clones.
  - `engine.rs`: `Arc::clone(&self)` replaces `self.clone()` in `initialize`
    and `render_multi`; single `name_for_events` string built once and moved.
  - `structure.rs`: `ValidationIssue::with_suggestion` accepts `impl
    Into<String>` so callers pass `&'static str` without `.to_string()`.
  - `modality.rs`: `available`/`by_tier`/`auto_select` return `&'static str`
    instead of allocating `String` names.
- **Smart file refactoring** (20 files total across sprints 4–5): Tests extracted
  into sibling modules. Sprint 5 added 9 files: `sensory_capabilities`,
  `unix_socket_rpc_handlers`, `audio_sonification`, `svg`, `engine`,
  `visual_flower`, `primal_details`, `neural_graph_client`, `provenance_trio`.
  Max production file now 414 lines (topology.rs); 6 files >700 lines remain
  (all test-only or domain-specific renderers).
- **Sandbox mock-biomeos**: Uses `capability_names::primal_names` constants,
  `discover_primal_socket` for path construction, `.expect()` instead of
  bare `.unwrap()`.

### Fixed
- **PT-06**: Push delivery startup confirmation log added; `callback_tx` wired
  in all JSON-RPC paths via `UnixSocketServer::new()`. Modes without IPC
  (web/tui/headless/ui) documented as intentionally push-free.
- **PT-09**: `log_handshake_policy()` now called at `UnixSocketServer::start()`.
  Domain symlinks use BTSP-aware family-scoped names. `Drop` impl cleans up
  the correct family-scoped symlink.
- Socket path `get_petaltongue_socket_path()` now delegates to BTSP posture for
  family-scoped naming instead of hardcoded `{APP_DIR_NAME}.sock`.
- **Flaky test fixed**: `test_resolve_instance_id_error_message_invalid` now
  uses `XDG_DATA_HOME` env isolation to prevent filesystem race conditions.

### Security
- `BIOMEOS_INSECURE` env var guard prevents production FAMILY_ID from running
  in insecure mode — conflicting posture is a fatal startup error.

- **Sensory Capability Matrix** (`SensoryCapabilityMatrix`): Formal type system
  that maps input capabilities × output capabilities. Consumer primals
  (ludoSpring, primalSpring, Squirrel) call `capabilities.sensory` to discover
  what interaction paths are available for a given user or agent session.
- **`capabilities.sensory` IPC method**: Returns the full sensory matrix from
  runtime hardware discovery. Supports `"agent": true` for AI-only sessions.
- **`capabilities.sensory.negotiate` IPC method**: Accepts explicit input/output
  capability overrides (e.g. from NestGate preferences) and returns a tailored
  matrix with validated paths and recommended modality.
- **`SwitchInputAdapter`**: Binary switch access for motor-impaired users
  (sip-and-puff, head switch, eye blink, BCI binary intent). Supports
  single-switch auto-advance and two-switch scan+select modes.
- **`AudioInversePipeline`**: Resolves sonification back to data targets. When
  a blind user hears a tone and presses "select", the pipeline maps the current
  audio position to a `DataObjectId` — the "6 vs 9" principle for audio.
- **`AgentInputAdapter`**: Formalizes agentic AI (Squirrel) as an `InputAdapter`
  at the interaction engine level. Translates JSON commands into the same
  semantic `InteractionIntent` pipeline as human input.
- **`InputModality::Agent`**: New enum variant for machine interactors.
- **SensorEvent variants**: `VoiceCommand`, `Gesture`, `Touch`, `GazePosition`,
  `SwitchActivation`, `AgentCommand` — contracts for Toadstool hardware
  integration. Includes `GestureType` and `GestureDirection` enums.
- **SensorType variants**: `Touch`, `EyeTracker`, `Switch`, `Agent` for
  runtime sensor discovery classification.
- **InteractionPattern enum**: `PointAndClick`, `KeyboardNavigation`,
  `VoiceAndAudio`, `SwitchScanning`, `GazeDwell`, `TouchGesture`, `AgentApi`,
  `BrailleRouting`, `HapticExploration`.
- **Validation test scenarios**: 12 integration tests covering blind keyboard
  user, deaf haptic user, motor-impaired switch user, screen reader,
  agentic AI, audio inverse pipeline, Toadstool sensor events, BCI pathway,
  and multi-user shared perspective.
- **wateringHole contracts**: `SENSORY_CAPABILITY_MATRIX.md` (full matrix
  specification for ecosystem alignment) and `TOADSTOOL_SENSOR_CONTRACT.md`
  (hardware sensor event IPC protocol).
- **UUI modality-agnostic descriptions**: `describe_binding()` produces rich
  text descriptions for ALL `DataBinding` variants including GameScene and
  Soundscape. A blind user hears: "Hero (player) at (16,16), health 80%,
  moving right. Goblin at (30,20), health 30%." A deaf user reads:
  "Soundscape 'Forest': wind white_noise 200Hz left, birdsong sine 800Hz
  right, starts at 5s."
- **GameScene audio sonification**: `sonify_game_scene()` maps entities to
  tones — position → stereo pan, entity type → frequency (player=A4,
  enemy=A3), health → amplitude. Blind users "hear" the battlefield.
- **GameScene haptic feedback**: `hapticize_game_scene()` converts entities
  to haptic commands — player=sustained pulse, enemy=intensity-from-damage,
  projectile=ramp, item=texture.
- **Soundscape haptic translation**: `hapticize_soundscape()` converts audio
  layers to haptic channels — frequency → pattern speed, amplitude →
  intensity, pan → spatial position. Deaf users feel the rhythm.
- **`compile_binding_modality()`**: IPC modality handler now routes
  GameScene/Soundscape to rich semantic output (description, audio, haptic)
  instead of generic scene graph compilation. Export path (`visualization.export`)
  automatically uses binding-aware compilation when available.
- **GameScene egui renderer**: Full 2D game scene rendering with tilemap
  painting, sprite positioning (z-order), entity rendering with health bars,
  velocity trails, and camera transform (zoom + pan). Replaces stub label.
- **Soundscape egui renderer**: Waveform preview visualization for each
  sound layer with frequency/amplitude indicators, stereo pan field display,
  and active-region highlighting. Replaces stub label.
- **Narrative/RPGPT scene detection**: Auto-detects narrative scenes
  (dialogue trees, combat grids, narration) from JSON structure. Renders
  description panels, NPC lists with health bars and status colors, numbered
  choice options, and combat grid overlays with entity icons.
- **`audio.synthesize` IPC method**: On-demand soundscape synthesis via
  JSON-RPC. Accepts a soundscape definition, returns sample metadata and
  optionally base64-encoded 16-bit stereo WAV.
- `visualization.render.scene` added to `self_capabilities::ALL` for
  proper capability advertisement to consumer primals.
- `audio.synthesize` added to `self_capabilities::ALL` and method dispatch.
- `SCENE_FORMAT_REFERENCE.md` published to wateringHole with full JSON
  schemas for GameScene (tilemap/sprite, narrative), Soundscape, and
  consumer primal guidance (ludoSpring, esotericWebb, primalSpring).
- `VISUALIZATION_INTEGRATION_GUIDE.md` v2.1.0: added `game_scene` and
  `soundscape` channel types, RPGPT dialogue/narrative examples, and
  `audio.synthesize` method documentation.

- Push delivery for callback dispatches (PT-06): subscribers providing
  `callback_socket` and `callback_method` now receive JSON-RPC notifications
  via UDS or TCP instead of polling. Background delivery task with graceful
  fallback to poll on failure.
- `push_delivery` module in `petal-tongue-ipc` with `spawn_push_delivery()`.
- `callback_socket` field on `CallbackDispatch` and `InteractionSubscriber`.
- `sensor_stream` module extracted from `interaction.rs` — `SensorStreamRegistry`
  now lives in its own module for clearer domain separation.
- `DEFAULT_WEBSOCKET_PORT` constant in `petal-tongue-core::constants`.
- `#![warn(missing_docs)]` on `petal-tongue-headless`.
- `#![expect(missing_docs)]` on `doom-core` (tracked for incremental completion).
- Zero-copy `Arc<InteractionEventNotification>` in subscriber queues —
  broadcast shares one allocation across N subscribers instead of N clones.
- Awakening coordinator `process_event` test coverage for all 6 event types.
- `DataService::snapshot_sync` tests (happy path, populated graph, poisoned lock).
- `CHANGELOG.md` (this file).
- Phase 3 wateringHole handoff document.
- **CONTEXT.md**: Ecosystem context file for AI tooling and primal discovery
- **`petal-tongue-wasm`**: Client-side WASM rendering module — grammar→SVG pipeline compiles to `wasm32-unknown-unknown` for offline-capable browser rendering (Gap 6 resolution)
- **`petal-tongue-types`**: Portable data types crate (`DataBinding`, `ThresholdRange`) extracted from `petal-tongue-core` for WASM compatibility
- **CI WASM check**: `cargo check --target wasm32-unknown-unknown -p petal-tongue-wasm` added to CI pipeline

### Changed
- **Smart refactoring**: `primitive.rs` (816L) → `primitive/mod.rs` (239L) + `primitive/tests.rs` (576L) — directory module pattern
- **Dependency dedup**: Unified `crossterm` from split 0.28/0.29 to 0.29 across all 4 crates
- `common_config::default_host()` now uses `DEFAULT_LOOPBACK_HOST` constant
  instead of duplicating `"127.0.0.1"`.
- `common_config::default_port()` now uses `DEFAULT_HEADLESS_PORT` constant
  instead of duplicating `8080`.
- `constants::default_bind_addr()` fallback uses `DEFAULT_LOOPBACK_HOST`
  constant instead of literal string.
- Scenario topology detection (`scenario_provider`, `dynamic_scenario_provider`)
  now uses `primal_type == "nucleus"` instead of `name == "NUCLEUS"` —
  type-based discovery, not name-based.
- `ENV_VARS.md`: `PETALTONGUE_MOCK_MODE` → `PETALTONGUE_FIXTURE_MODE` with
  migration note and removal timeline.
- `README.md`: updated `forbid(unsafe_code)` to reflect unconditional status.
- `wateringHole/petaltongue/README.md`: updated stats (coverage, date, forbid).
- `sandbox/scenarios/README.md`: corrected stale directory structure, updated date.
- **BREAKING**: Renamed `mock_mode` → `fixture_mode` across config, API client,
  and UI manager. `BiomeOSClient::with_mock_mode()` → `with_fixture_mode()`,
  `is_mock_mode()` → `is_fixture_mode()`, `MockModeUnavailable` →
  `FixtureModeUnavailable`. Config field `mock_mode` accepted as serde alias
  for backwards compatibility.
- UI label changed from "Mock Mode" to "Fixture Mode (offline)".
- **Capability naming**: `DisplayCapabilities::toadstool()` → `network_display()`; `"squirrel-ui-adapter"` → `"ai-interaction-adapter"`
- **Health triad compliance**: `health.readiness` and `health.check` now include `version` and `primal` fields per `DEPLOYMENT_VALIDATION_STANDARD.md`
- **Workspace lints**: `[workspace.lints.rust] unsafe_code = "forbid"` (was per-crate, now workspace-wide); `rust-version = "1.87"` added
- **Capability symlink**: Server creates `visualization.sock` → `petaltongue.sock` on start per `CAPABILITY_BASED_DISCOVERY_STANDARD.md`
- **Lint tightening**: Migrated 33 `#[allow(clippy::unwrap_used)]` to `#[expect]` with reasons; removed where unused
- **Zero-copy RPC**: Eliminated `req.params.clone()` in `visualization.render`, `visualization.render.scene`, and `capabilities.sensory.negotiate`
- **Smart refactoring**: `graph_manager.rs` → directory module (314+462 lines); `headless_integration_tests.rs` → 5 themed test files; `dns_parser.rs` → directory module; `spring_adapter.rs` → directory module; `constants.rs` → directory module
- **Dead code removed**: `audio_providers/` (5 files), `audio_playback.rs`, 13 unused primal name constants, empty feature flags

### Fixed
- Removed unfulfilled `#[expect(clippy::unused_self)]` and
  `#[expect(clippy::cast_sign_loss)]` in `petal-tongue-api` — clippy now
  passes with zero warnings across the workspace.
- Replaced `as u64` cast with `.cast_unsigned()` in `biomeos_client.rs`.
- Eliminated the last `unsafe` block in `petal-tongue-ipc` (provenance_trio test):
  replaced raw `env::set_var`/`remove_var` with `temp_env::with_vars` via
  `petal_tongue_core::test_fixtures::env_test_helpers`. Upgraded crate to
  unconditional `#![forbid(unsafe_code)]`.
- Hardcoded `127.0.0.1` in `display/backends/software.rs` replaced with
  `constants::DEFAULT_LOOPBACK_HOST`; hardcoded port `8765` replaced with
  `constants::DEFAULT_WEBSOCKET_PORT`.
- **PII scrub regression**: `test_a_record_parse` fixture data corrected (192.168.1.100 → 192.0.2.100)
- **Broken doc paths**: Stale `docs/` references updated

## [1.6.6] — 2026-04-01

### Added
- `visualization.render.scene` RPC method (PT-04) — direct `SceneGraph` submission.
- `visualization.export` RPC method (PT-06 listen surface).
- `ExportFormat::Html` for headless CLI export (SVG wrapped in standalone HTML).
- `CallbackDispatch` struct and `pending_callbacks` response field (PT-06 data model).
- `RenderingAwareness` auto-init in `UnixSocketServer` (PT-05).
- Server-mode periodic discovery refresh (PT-07).
- blake3 provenance hashing (pure Rust, zero C/asm deps).
- Centralized discovery timeouts (12 named constants, env-overridable).
- `primal_names` and `socket_roles` constants — no hardcoded primal strings.
- `rust-toolchain.toml` (pins stable + rustfmt, clippy, llvm-tools-preview).
- `deny.toml` for advisory, license, ban, and source auditing.
- `llvm-cov.toml` with 90% `fail-under` threshold.
- Spring data adapter (`SpringDataAdapter`) for ludoSpring / ecoPrimals formats.
- `DispatchOutcome`, `IpcErrorPhase`, `StreamItem` structured IPC tracing.
- `CircuitBreaker` and `RetryPolicy` in resilience module.

### Changed
- All JSON-RPC serialization hot paths evolved from `serde_json::to_string` to
  `serde_json::to_vec` (zero intermediate `String` allocation per frame).
- Documentation reconciled to "JSON-RPC 2.0 REQUIRED, tarpc MAY for Rust-to-Rust hot paths".
- TUI devices panel renders real discovered `PrimalInfo` instead of placeholders.
- Extracted `parse_primal_array()`, `world_to_screen()`, `adjacency_list()`
  to reduce duplication.

### Fixed
- `#![forbid(unsafe_code)]` on all crates (one justified test exception).
- Zero clippy warnings (pedantic + nursery, `-D warnings`).
- Zero `TODO`/`FIXME`/`HACK` markers in committed code.
- All files under 1000 lines (max 852).

### Removed
- `DefaultHasher` placeholder in `provenance_trio.rs` (replaced by blake3).
- Hardcoded primal name strings in 7 files.

## [1.6.5] — 2026-03-28

### Added
- `deny(clippy::unwrap_used, clippy::expect_used)` at workspace level.
- Enriched `capability.list` response with transport and protocol metadata.

## [1.6.4] — 2026-03-26

### Changed
- Version bump, root doc cleanup, stale script path fixes.

[Unreleased]: https://github.com/ecoPrimals/petalTongue/compare/v1.6.6...HEAD
[1.6.6]: https://github.com/ecoPrimals/petalTongue/compare/v1.6.5...v1.6.6
[1.6.5]: https://github.com/ecoPrimals/petalTongue/compare/v1.6.4...v1.6.5
[1.6.4]: https://github.com/ecoPrimals/petalTongue/releases/tag/v1.6.4
