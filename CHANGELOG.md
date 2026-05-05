# Changelog

All notable changes to petalTongue are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Port Alignment + Discovery Escalation Hierarchy (May 5, 2026)

#### Added
- **Ecosystem TCP fallback port**: `ECOSYSTEM_TCP_FALLBACK_PORT = 9900` in
  `constants/network.rs`. Aligned with primalSpring's move from 9600 to avoid
  rhizoCrypt tarpc conflict. Port added to `DEFAULT_DISCOVERY_PORTS` for
  Tier-5 TCP probing.
- **Discovery escalation hierarchy docs**: `petal-tongue-discovery/src/lib.rs`
  module docs rewritten with 5-tier hierarchy from primalSpring standard:
  Songbird `ipc.resolve` (Tier 1, future), Neural API (Tier 2),
  UDS filesystem (Tier 3), socket registry (Tier 4), TCP probing (Tier 5).

#### Changed
- **Last 2 hardcoded primal names evolved**: `identity_lifecycle.rs` "sourDough"
  → "discovery agents"; `audio_sonification.rs` "Squirrel" → "AI capability
  providers". All remaining primal names in production code are either legacy
  env vars (documented), historical provenance attribution, ecosystem standard
  references, or test fixtures.

### primalSpring Phase 58 Audit Response (May 4, 2026)

#### Added
- **GAP-12 machine-readable method schemas**: `visualization.capabilities` now
  returns a `methods` object with parameter schemas for all visualization
  methods (`visualization.render.dashboard`, `visualization.render.scene`,
  `visualization.render`, `visualization.export`). Each schema includes
  required/optional params with types, defaults, and descriptions. Enables
  downstream consumers to programmatically discover dashboard parameters.

#### Verified (stale audit items confirmed resolved)
- **Phase 3 transport encryption** (item 1): Shipped — `btsp/phase3.rs` with
  ChaCha20-Poly1305 AEAD, 13/13 ecosystem parity.
- **musl/winit threading panic** (item 2): Fixed — `with_any_thread(true)` in
  all 3 call sites (ui_mode, live_mode, backend/eframe). PG-40 + PG-48.
- **PT-04 HTML export** (item 3): Complete — `ExportFormat::Html` via
  `wrap_svg_in_html`, headless CLI + IPC + e2e tests green.
- **PT-06 push delivery** (item 4): Active — `callback_tx` wired in
  `UnixSocketServer::new_with_socket` via `spawn_push_delivery()`, live mode
  GUI broadcasts through same channel.

### TRUE PRIMAL Name Evolution — Capability-Based Language (May 3, 2026)

#### Changed
- **BTSP `EnforceBearDog` → `EnforceProvider`**: Renamed handshake policy enum
  variant to capability-based term. All doc comments and log messages evolved
  from "BearDog" to "security provider" across 8 BTSP files (types.rs, server.rs,
  json_line.rs, phase3.rs, error.rs, client.rs, mod.rs, tests.rs).
- **Audio backends**: "ToadStool" references evolved to `audio.play` capability
  provider language in socket.rs, direct.rs, network.rs, mod.rs.
- **Provenance trio**: `rhizoCrypt`/`sweetGrass`/`loamSpine` references in
  provenance_trio.rs evolved to capability-based terms (`dag.session`,
  `braid.create`, `spine.create` providers). Historical attribution in
  ipc_errors.rs and resilience.rs preserved as code provenance.
- **HTTP/TLS delegation**: "Songbird" references in http_client.rs,
  https_client.rs, connect.rs, biomeos_client.rs, stream.rs evolved to
  "TLS provider" / "ecosystem provider".
- **Scene signer**: "BearDog" references evolved to "security provider".
- **Visualization handler**: "toadStool Phase 2" evolved to "display capability
  Phase 2" in texture attach placeholder docs.
- **Trust adapter**: "temporary" comment evolved to proper API guidance pointing
  to `from_capability_spec()`.
- **README.md**: BTSP quality row updated from Phase 2 to Phase 3 with
  ChaCha20-Poly1305 and 13/13 ecosystem parity.

#### Verified
- Zero hardcoded primal names in production code (BearDog, ToadStool, Songbird
  only remain in test fixtures and historical code provenance comments).
- `cargo clippy --workspace --all-features`: 0 warnings.
- `cargo doc --workspace --no-deps` with `-D warnings`: 0 warnings.
- `cargo test --workspace --all-features`: 6,200+ passed, 0 failed.

### BTSP Phase 3 Transport Switch — 13/13 Ecosystem Parity (May 3, 2026)

#### Added
- **`btsp/phase3.rs`**: New module implementing ChaCha20-Poly1305 AEAD encrypted
  frame I/O for BTSP Phase 3. Includes `SessionKeys` (HKDF-SHA256 directional
  key derivation), `Phase3Session` (encrypt/decrypt), `read_encrypted_frame`,
  `write_encrypted_frame`, `handle_encrypted_stream`, and `try_phase3_negotiate`
  for post-handshake nonce exchange. 10 unit tests.
- **`HandshakeResult` struct** (types.rs): Bundles session_token + cipher +
  session_key from Phase 2 handshake for Phase 3 upgrade.
- **`KeyDerivationFailed` + `Phase3Crypto` error variants** (error.rs): Typed
  errors for HKDF and AEAD failures.
- **Dependencies**: `chacha20poly1305 0.10`, `hkdf 0.12`, `sha2 0.10`,
  `rand 0.8`, `zeroize 1` — all pure Rust, compatible with digest 0.10 ecosystem.

#### Changed
- **`server.rs` + `json_line.rs`**: Both handshake functions now return
  `HandshakeResult` including base64-decoded session_key from
  `btsp.session.verify`.
- **`unix_socket_server.rs`**: After handshake, checks if cipher is
  `chacha20-poly1305` and session_key is present. If so, upgrades connection to
  encrypted frame I/O via `try_phase3_upgrade_split`. Both UDS and TCP paths
  covered. Extracted `run_uds_handshake` helper for line-count compliance.
- **Wire format**: `[4B BE length][12B random nonce][ciphertext + 16B Poly1305 tag]`
  per ecosystem standard.

### Deep Debt Sweep — Idiomatic Rust Evolution (May 3, 2026)

#### Fixed
- **Clippy `--all-features` clean**: Fixed 25+ lints newly surfaced by running
  clippy with `--all-features` (previously CI only ran default features).
  - `map_unwrap_or`: `.map().unwrap_or()` → `.map_or()` / `.map_or_else()` /
    `.is_ok_and()` across 12 files (svg.rs, data_service.rs, detection.rs,
    methods.rs, proprioception.rs, gather.rs, startup_audio.rs, primitives.rs,
    provider_trait.rs, live_data, audio_discovery).
  - `duration_suboptimal_units`: `Duration::from_secs(60)` → `from_mins(1)`
    across 5 files (timeouts.rs, types.rs, provider_trait.rs,
    input_verification.rs, output_verification.rs, live_data).
  - `sort_by` → `sort_by_key` in 4 files (sensor_feed, timeline_view,
    trust_dashboard/compute, process_viewer_integration).
  - Stale `#[expect(dead_code)]` → `#[allow(dead_code)]` in motor_state.rs.
  - Trailing comma removal (dynamic_scenario_provider, process_viewer_integration).
- **Doc warnings**: Fixed 4 broken intra-doc links (TextureRegistry,
  InputAdapter, SocketBackend, DirectBackend) and 2 unclosed HTML tags.

#### Added
- **UniBin v1.1 `--port` flag**: Web and Headless modes now accept `--port`
  per the UniBin Architecture Standard. `--bind` takes precedence; `--port`
  resolves to `0.0.0.0:<PORT>`. New `resolve_bind()` helper and 4 tests.
- **CI `--all-features`**: clippy, test, and doc steps now use `--all-features`.
  Doc step added with `-D warnings` RUSTDOCFLAGS gate.

#### Removed
- **Dead `audio_web.rs`**: 301-line file using `web_audio_api` crate not in
  Cargo.toml, never wired into module tree. Deleted.

#### Changed
- **Hardcoded test ports evolved**: headless_mode.rs and web_mode.rs tests now
  use `constants::default_headless_bind()`, `DEFAULT_WEB_PORT`,
  `DEFAULT_LOOPBACK_HOST` instead of literal `"0.0.0.0:8080"` strings.
- **web/index.html**: "6 subcommands" → "7 subcommands".
- README.md quality table updated (coverage 85%, --all-features gates).

#### Verified
- `cargo fmt --check`: 0 violations.
- `cargo clippy --workspace --all-features -- -D warnings`: 0 warnings.
- `cargo test --workspace --all-features`: 6,200+ passed, 0 failed.
- `cargo doc --workspace --no-deps`: 0 warnings.
- `cargo llvm-cov --workspace --lib`: 85.2% line coverage.

### String Error Elimination — Typed Error Evolution (May 2, 2026)

#### Changed
- **provenance_trio.rs**: `send_rpc` evolved from `Result<Value, String>` to
  `Result<Value, ProvenanceRpcError>` (6 typed variants: Connect, Serialize, Io,
  Parse, RpcError, NoResult).
- **physics_bridge.rs**: 5 functions evolved from `Result<_, String>` to
  `ComputeBridgeError` (7 variants including recursive `Send` for IPC layering).
- **audio.rs** (IPC handler): `encode_wav_base64` evolved from `Result<String, String>`
  to `Result<String, WavEncodeError>` wrapping `hound::Error`.
- **graph_builder/builder.rs**: `add_edge` evolved from `Result<(), String>` to
  `Result<(), GraphEdgeError>` (SourceNotFound, TargetNotFound, Duplicate).
- **event.rs**: `EventBus::broadcast` evolved from `Result<usize, String>` to
  `Result<usize, broadcast::error::SendError<EngineEvent>>` (direct tokio type).
- **capability_taxonomy.rs**: `FromStr::Err` evolved from `String` to
  `ParseCapabilityError` typed error.
- **biomeos_discovery/backend.rs**: `connect_and_forward` evolved from
  `Result<(), String>` to `WebSocketBridgeError` (4 variants).
- **status_reporter/reporter.rs**: `get_status_json` evolved from
  `Result<String, String>` to `Result<String, serde_json::Error>`.
- **startup_audio.rs**: 4 functions evolved from `Result<(), String>` to
  `StartupAudioError` (AudioCanvas, FileRead, Decode variants).
- **data_source.rs**: 3 functions evolved from `Result<_, String>` to
  `DataSourceError` (Discovery, Topology, LockPoisoned).
- **sandbox_provider.rs**: 3 functions evolved from `Result<_, String>` to
  `SandboxError` (NotFound, Read, Parse, DirNotFound, CurrentDir).
- **audio/backends/network.rs**: `send_play_request` evolved from
  `Result<(), String>` to `NetworkAudioError` (5 variants).
- **tool_integration.rs**: `ToolPanel::handle_action` evolved from
  `Result<(), String>` to `Result<(), ToolActionError>`.

#### Verified
- `cargo clippy`: 0 warnings.
- `cargo fmt`: 0 violations.
- `cargo test --workspace --all-features`: 6,191 passed, 0 failed.
- `cargo deny check bans`: passes.
- Zero `Result<_, String>` remaining in IPC, bridge, or public API surfaces.

### primalSpring Phase 56 Audit Response (May 1, 2026)

#### Fixed
- **PG-48**: Verified musl `live` mode panic already resolved (`with_any_thread(true)` on
  X11+Wayland across `ui_mode.rs`, `live_mode.rs`, and `backend/eframe.rs`).
- **GAP-12**: Added wire-level JSON-RPC schema documentation for `visualization.render.dashboard`
  (required: `session_id`, `title`, `bindings`; optional with defaults: `domain`, `modality`,
  `max_columns`). Full request/response examples and error code table.

#### Changed
- **deny.toml**: Added `async-trait` to `[bans] deny` with wrappers for transitive deps
  (`axum`, `axum-core`, `opentelemetry_sdk`). Prevents regression to pre-edition-2024 patterns.
- **BTSP Phase 2 → Phase 2 in README**: Quality table now reflects operational Phase 2
  status (typed `BtspHandshakeError`, BearDog provider delegation, NULL cipher handshake).

#### Verified
- `cargo deny check bans`: passes (interstadial quality gate).
- `cargo clippy`: 0 warnings.
- `cargo fmt`: 0 violations.
- `cargo test --workspace --all-features`: 6,191 passed, 0 failed.
- `cargo doc`: 0 warnings.
- Edition 2024, `async-trait` eliminated (zero direct usage).
- BTSP Phase 2 operational (20+ handshake tests).

### Phase 56: Desktop NUCLEUS Gap Resolution (April 29, 2026)

#### Fixed
- **GAP-01 (P1)**: `RegistrationClient` now reads `DISCOVERY_SOCKET` env var
  as highest-priority override for heartbeat/registration target. Falls back to
  `DISCOVERY_SERVICE_SOCKET` basename resolution. Heartbeat task uses exponential
  backoff (2^n × interval, capped at 64×) instead of fixed-interval on failure.
- **Motor P0**: Live mode motor channel was a dead end — IPC motor commands went
  to a logging thread, never reaching the GUI. `run_on_main_thread` now passes the
  IPC motor channel directly into `PetalTongueApp::replace_motor_channel`, so
  `motor.set_panel`, `motor.set_zoom`, `motor.set_awakening` etc. are applied
  every frame by `drain_motor_commands`.

#### Added
- **`motor.panel.update`**: New IPC method for compositions to push content to
  named panels (title + JSON payload). Backed by `PanelContentStore`.
- **`motor.notification`**: New IPC method for compositions to display
  notifications (level, message, optional auto-dismiss duration). Backed by
  `NotificationQueue`.
- **`DISCOVERY_SOCKET`** env var documented in `ENV_VARS.md`.
- 9 new tests (motor panel update, notification, discovery socket override,
  panel content store, notification queue).

#### Verified
- **GAP-17**: `visualization-{family}.sock` symlink is already created by
  `UnixSocketServer::start` via `btsp::domain_symlink_filename`. No code change
  needed; symlink confirmed functional with `FAMILY_ID=desktop-nucleus`.

### Deep Debt Audit: Workspace Dependency Consolidation (April 28, 2026)

#### Changed
- **15 crates consolidated to workspace dependencies**: `futures-util`, `crossterm`,
  `terminal_size`, `tiny-skia`, `epaint`, `png`, `svg`, `indexmap`, `colored`, `socket2`,
  `dashmap`, `lru`, `ron`, `ratatui`, `symphonia` — all moved from per-crate version pins
  to `{ workspace = true }` in `[workspace.dependencies]`.
- **Telemetry fallback path**: `/tmp/petaltongue-telemetry` in `jsonl_provider.rs` extracted
  to `DEFAULT_TELEMETRY_FALLBACK_DIR` constant in `petal-tongue-core`.
- **Root `png` optional dep**: aligned to workspace reference.

### Phase 55: Awakening Evolution + Scene Signing + Sensor Stream (April 28, 2026)

#### Changed
- **Awakening default changed to OFF**: `AWAKENING_ENABLED` defaults to `false` (was `true`).
  Compositions invoke awakening via new `motor.set_awakening` IPC method instead of getting
  hardcoded defaults. Standalone `ui` mode users set `AWAKENING_ENABLED=true` or use scenario config.
- **`SetAwakening` motor handler**: now supports both `start()` and `skip()` — compositions can
  enable or disable awakening at any time via IPC.

#### Added
- **Scene push signing** (`SceneSigner`): BLAKE3 keyed-hash integrity signatures for scene graphs.
  Uses `PETALTONGUE_SCENE_KEY` env var (hex-encoded 32-byte visualization purpose key per NUCLEUS
  Two-Tier Crypto Model). Scene push responses include `signed: bool` and optional `signature` field.
- **`visualization.scene.verify` IPC method**: compositions verify stored scene integrity by providing
  `session_id` and `signature`.
- **`motor.set_awakening` IPC method**: compositions control awakening overlay on/off via JSON-RPC.
- **Sensor stream new event types**: `focus_gained`, `focus_lost`, `window_resize`, `text_input`
  added to `SensorEventIpc`. Focus and text events wired in `sensor_feed.rs` egui collection.
- **Capability advertisements**: `visualization.texture.upload`, `visualization.texture.attach`,
  `visualization.scene.verify`, `motor.set_awakening` added to `capabilities.list` response.

### Deep Debt Audit: Dependency Consolidation + Discovery Evolution (April 27, 2026)

#### Changed
- **`toml`** in `petal-tongue-core`: consolidated to `{ workspace = true }` (was standalone `0.8`).
- **`tokio-util`** in `petal-tongue-tui`: consolidated to `{ workspace = true }` (was standalone
  `0.7`, now inherits workspace `codec` feature).
- **`rustix`** added to workspace dependencies (`0.38`): `petal-tongue-core` (process) and
  `petal-tongue-ui` (param) now use `{ workspace = true, features = [...] }`.
- **`tempfile`** root dev-dep: changed from `"3.10"` to `{ workspace = true }`.
- **`external-display`** stale feature alias removed from `petal-tongue-ui` (zero cfg references).
- **`universal_discovery.rs`**: socket search paths now include XDG_RUNTIME_DIR
  (`/run/user/{uid}`) as priority-1, matching `unix_socket_provider.rs` pattern.
  Previously only searched `/tmp` and `/var/run`, missing the standard biomeOS socket location.

### PG-53 Follow-up: rendering_awareness Server Mode Bug (April 27, 2026)

#### Fixed
- **`rendering_awareness` unconditionally `Some` in server mode**: `UnixSocketServer::new_with_socket`
  was unconditionally setting `rendering_awareness = Some(...)`, causing `proprioception.get` to
  report `frame_rate: 60`, `mode: "live"`, `window: { present: true }` even in headless `server`
  mode. Removed the unconditional initialization; `rendering_awareness` now defaults to `None`
  (the `RpcHandlers::new()` default). Only `live` mode explicitly wires it via
  `with_rendering_awareness()`.
- Server mode now correctly reports `frame_rate: 0`, `mode: "server"`, `window: null`.
- Updated `test_default_rendering_awareness_initialized` and `test_introspect_works_with_default_awareness`
  to test the correct server-mode behavior (graceful degradation, not false positives).

#### Also fixed (pre-existing lints)
- `rich_test_scene()` test fixture: added `#[expect(clippy::too_many_lines)]`.
- `pixel_renderer_demo.rs`: borrow fix on `save_as_png` path argument.
- `base64::encode` needless borrow in visualization test.
- `AppError::TaskPanic`: `#[expect(dead_code)]` → `#[cfg_attr(not(test), allow(dead_code))]`
  (dead in bin target, used in test target).

### PG-48: musl/plasmidBin winit Main-Thread Panic (April 27, 2026)

#### Fixed
- **winit `any_thread` for musl**: musl libc reports thread IDs differently,
  causing winit's `is_main_thread()` check to fail even on the OS main thread.
  Added `EventLoopBuilderExtX11::with_any_thread(true)` via `NativeOptions.event_loop_builder`
  hook on Linux. Our PG-40 code already guarantees main-thread dispatch, so
  this just bypasses the check that was false-positive under musl.
- **eframe backend features**: enabled explicit `x11` + `wayland` features
  (previously stripped by `default-features = false`). `winit` added as direct
  workspace dep for platform extension traits (zero new crate — already transitive).
- Shared `native_options_with_any_thread()` helper used by both `ui_mode` and
  `live_mode`.

### PG-53: Server Mode Proprioception (April 27, 2026)

#### Added
- **`proprioception.get`** IPC method — returns a synthetic proprioception snapshot
  usable by composition scripts in all modes. Server mode returns `frame_rate: 0`,
  `window: null`, `mode: "server"`. Live/UI mode returns `frame_rate: 60`,
  `window: { present: true }`, `mode: "live"`.
- Fields: `frame_rate`, `active_scenes`, `total_frames`, `user_interactivity`,
  `mode`, `uptime_secs`, `window`.
- Tests: server-mode zero FPS, with-sessions scene count.

#### Note
- `--socket` flag on `server` mode: confirmed fully wired (PT-10, April 10).
  ludoSpring report was likely from a stale binary. `--socket` + `PETALTONGUE_SOCKET`
  env both functional.

### PG-43: Texture Primitive + IPC Methods (April 26, 2026)

#### Added
- **`Primitive::Texture`** variant in the scene graph — raster content (sprites,
  images, external engine framebuffers) with `texture_id`, position, size,
  optional UV sub-region (`UvRect`), opacity, and tint.
- **`TextureRegistry`** in `VisualizationState` — stores pixel data keyed by
  `texture_id`, versioned for lazy GPU re-upload.
- **`visualization.texture.upload`** IPC method — push base64-encoded RGBA pixel
  data, get a `texture_id` back.
- **`visualization.texture.attach`** IPC method — register a shared-memory source
  (memfd URI); actual mapping deferred to toadStool Display Phase 2.
- **`From<Sprite> for SceneNode`** bridge — converts game `Sprite` models directly
  into scene graph `Primitive::Texture` nodes.
- `UvRect` struct for texture atlas sub-region selection.
- `rich_test_scene()` fixture now includes a Texture primitive for all modality
  compiler tests.
- Tests: Primitive serde round-trip (Texture, Texture-no-UV, UvRect), Sprite
  bridge, TextureRegistry (insert/get/remove/re-upload version), IPC handler
  (upload success/bad-format/bad-size, attach success).

#### Changed
- All 12 exhaustive `match Primitive` sites updated across `petal-tongue-scene`
  (7 sites) and `petal-tongue-ui` (3 sites + 2 wildcards).
- egui renderer shows a tinted placeholder rect for Texture primitives; full
  `egui::Shape::image` with `TextureResolver` deferred to display-phase evolution.
- Terminal modality renders `[IMG]` for Texture primitives.
- SVG modality emits `<image>` element with `data-texture-id` attribute.
- Braille modality renders an outline rect for Texture primitives.

#### Deferred
- **Overlay mode** (transparent windows over external game engines) — depends on
  toadStool Display Phase 2 and Wayland `wlr-layer-shell` support.

### Eliminate all `dyn` from production code (April 26, 2026)

#### Changed
- `PanelInstance::on_error` — `&dyn std::error::Error` → `&impl std::error::Error`
  (generic parameter; enum dispatch makes object safety unnecessary).
- `SseEventConsumer` — `Box<dyn Fn(EcosystemEvent)>` callback replaced with
  `tokio::sync::mpsc::UnboundedSender<EcosystemEvent>` (async channel pattern).
- `EventStream` — `Box<dyn Fn(BiomeOSEvent)>` callback replaced with
  `tokio::sync::mpsc::UnboundedSender<BiomeOSEvent>`.
- `BiomeOSProvider::subscribe_events_with_callback` →
  `subscribe_events_with_sender(tx: UnboundedSender<BiomeOSEvent>)`.
- `EventStream::new()` is now `const fn` (possible after removing heap-allocated
  closure field).
- **Result**: 0 `dyn` in production Rust code. All remaining mentions are in comments
  documenting completed enum-dispatch migrations.

### PG-40: Fix winit main-thread panic on Linux + PETALTONGUE_SOCKET env binding (April 26, 2026)

#### Fixed
- **Native display modes (`petaltongue live`, `petaltongue ui`) no longer panic on
  Linux**. winit 0.30 requires the event loop to be initialized on the main thread
  (X11/Wayland). The previous code used `tokio::task::spawn_blocking` which ran
  eframe on a tokio worker thread. Restructured `main()` to build the tokio runtime
  manually: UI modes run eframe directly on the main thread, non-UI modes dispatch
  via `runtime.block_on()`.
- `live_mode.rs`: IPC server, motor drain, and discovery refresh now spawn on the
  runtime's thread pool (via `runtime.spawn()` / `std::thread::spawn`), while eframe
  runs on the calling (main) thread.
- `ui_mode.rs`: `run_on_main_thread()` replaces `spawn_blocking` path.

#### Changed
- `main()` is no longer `#[tokio::main] async fn`; it is a regular `fn main()` that
  builds a `tokio::runtime::Runtime` manually and dispatches to `dispatch_async()`
  for non-GUI modes.
- `--socket` flag on `server` and `live` commands now reads `PETALTONGUE_SOCKET` env
  var as fallback via clap's `env` attribute (clap `env` feature enabled).
- Added doc comment on `Server` command: socket path priority is
  `--socket flag > PETALTONGUE_SOCKET env > XDG default`.

### BTSP family_seed Base64 Encoding Fix (April 24, 2026)

#### Fixed
- **`load_family_seed()` now base64-encodes** the raw env var string before
  returning it. BearDog's `btsp.session.create` handler base64-decodes the
  `family_seed` parameter; sending raw hex caused HMAC mismatches (guidestone
  error: "BTSP verification failed: unknown"). Aligns with all other converged
  relay primals (barraCuda, Songbird, coralReef, sweetGrass, etc.).
- **Whitespace trimming** preserved — env value is trimmed before base64 encoding.

#### Changed
- **6 BTSP tests updated** — tests now use raw string inputs and verify base64
  output. Covers: env priority, fallback, raw hex encoding, trim-then-encode,
  empty-after-trim, and unset returns `None`.

### Native `async fn` in Traits — Manual Desugaring Elimination (April 25, 2026)

#### Changed
- **13 production modules** converted from `fn -> impl Future + Send` to native
  `async fn` in traits (RPITIT). Traits affected: `ComputeProvider`, `GUIModality`,
  `Sensor`, `DiscoveryBackend`, `PrimalLifecycle`, `PrimalHealth`, `AudioBackend`,
  `DisplayBackend`, `UIBackend`, `VisualizationDataProvider`.
- All enum-dispatch implementations updated to match new `async fn` signatures.
- `render_multi` in `engine.rs` simplified to sequential awaits (was spawning
  tasks that contended on the same write lock).
- Net reduction: **−100 lines** across 21 files.

#### Removed
- All `#![allow(clippy::manual_async_fn)]` suppressions (13 modules).
- Redundant `async { }` / `async move { }` wrapper blocks in all impl methods.

### BTSP JSON-Line Handshake Relay + BearDog Field Alignment (April 23, 2026)

#### Added
- **`btsp/json_line.rs`** — new JSON-line (newline-delimited) BTSP handshake relay module.
  Full 4-step protocol: read ClientHello line, call BearDog `btsp.session.create` (with
  real base64 `family_seed`, not reference), send ServerHello line (using BearDog's
  challenge, not local PRNG), read ChallengeResponse line, call `btsp.session.verify`
  (with `session_token` + `response` field names), send HandshakeComplete line (`"status":"ok"`).
- **`BtspHandshakeConfig::load_family_seed()`** — resolves `BEARDOG_FAMILY_SEED` >
  `FAMILY_SEED` from environment for passing to BearDog `btsp.session.create`.
- **9 new BTSP tests** — env cascade for `SECURITY_PROVIDER_SOCKET`, `CRYPTO_PROVIDER_SOCKET`,
  `SECURITY_SOCKET`; `load_family_seed` priority and fallback; `json_str_or` helper.

#### Changed
- **UDS/TCP accept routing** — JSON-line BTSP announcements (`{"protocol":"btsp",...}`)
  now route to `relay_json_line_handshake` instead of `perform_server_handshake`
  (which uses length-prefixed framing). Three-way classification: non-`{` →
  length-prefixed BTSP, `{` + `"protocol"` → JSON-line relay, `{` only → plain JSON-RPC.
- **`btsp.session.create`** — sends actual `family_seed` (base64 from env) instead of
  `family_seed_ref: "env:FAMILY_SEED"`. Uses BearDog's returned challenge (not local
  `rand_u128()`). Accepts both `session_token` and `session_id` in response.
- **`btsp.session.verify`** — sends `session_token` (was `session_id`) and `response`
  (was `client_response`). Sends `preferred_cipher` + `client_ephemeral_pub` per BearDog spec.
- **HandshakeComplete** — sends `"status":"ok"` (was `"complete"`) per upstream spec.
- **Provider socket env cascade** — now checks `SECURITY_PROVIDER_SOCKET`,
  `CRYPTO_PROVIDER_SOCKET`, `SECURITY_SOCKET` between `BEARDOG_SOCKET` and the
  family-scoped default path.
- **Removed `rand_u128()`** — no longer generating local challenges; BearDog provides them.

#### Verified
- `cargo clippy --workspace --all-targets --all-features` — 0 warnings
- `cargo test --workspace --all-features` — all passing
- `cargo check --target x86_64-apple-darwin` — macOS cross-check clean
- 21 BTSP-specific tests all passing (12 existing + 9 new)

### `petaltongue live` Mode + BTSP Wire-Format Fix + Deep Debt Zero (April 21, 2026)

#### Added
- **`petaltongue live` CLI subcommand** — merges `ui` (egui/eframe native window) and
  `server` (UDS JSON-RPC IPC) into a single process for interactive desktop NUCLEUS
  deployment. IPC server runs as background `tokio::spawn` task, egui on main thread,
  connected via `Arc<RwLock<VisualizationState>>`, `SensorStreamRegistry`,
  `InteractionSubscriberRegistry`, and `CallbackDispatch` sender. Feature-gated behind
  `ui` with graceful `UiNotAvailable` fallback.
- **BTSP wire-format detection** — `is_btsp_json_announcement()` three-way classifier:
  non-`{` → length-prefixed BTSP, `{` + `"protocol"` → BTSP JSON-line announcement,
  `{` only → plain JSON-RPC. Applied to both `handle_uds_with_btsp` (BufReader peek)
  and `handle_tcp_with_btsp` (64-byte peek buffer, was 1). 5 unit tests.
- **`src/live_mode.rs`** — new module combining server + UI with shared state handles.

#### Changed
- **`DoomPanelWrapper` boxed** in `PanelInstanceImpl` enum — reduces stack size from
  432 to 8 bytes. All 19 match arm methods migrated from UFCS to method syntax for
  clean auto-deref through `Box`.
- **`futures` → `futures-util`** in `petal-tongue-discovery` — lighter dependency,
  same API surface (`join_all`, `select_all`). 3 source files + 2 test files updated.
- **`needless_return`** fixed in socket audio backend.
- **`unused_async`** fixed in chaos test helper (`async fn` → `fn` returning `impl Future`).

#### Verified
- `cargo clippy --workspace --all-targets --all-features` — **0 warnings**
- `cargo test --workspace --all-features` — all passing
- `cargo check --target x86_64-apple-darwin` — macOS cross-check clean
- 4 remaining `dyn` usages audited: all idiomatic (`Box<dyn Fn>` callbacks,
  `&dyn std::error::Error`), no evolution needed

### Deep Debt Cleanup — Clippy Zero, Typed Errors, Modern Rust (April 15, 2026)

#### Changed
- **Clippy warnings eliminated** — resolved all 35 remaining warnings across the
  workspace: unused imports, `const fn` upgrades, `async fn` simplification
  (replaced verbose `impl Future<Output = …> + Send` with native `async fn`),
  redundant closures, large enum variant boxing (`MdnsVisualizationProvider`,
  `HttpsClient`), privacy alignment, missing docs on test variants.
- **Typed error returns** — `unix_socket_server.rs` connection handlers evolved
  from `Box<dyn std::error::Error + Send + Sync>` to typed `ConnectionError`.
- **Module visibility corrected** — `doom-core` endian/map_parse functions use
  `pub(super)` instead of `pub(crate)` inside private modules; persistence and
  compute test types visibility aligned with their `#[cfg(test)]` gates.
- **Enum variant boxing** — `KnownVisualizationProvider::Mdns` and
  `PrimalConnection::Https` now boxed to eliminate 300+ byte variant size
  differences.

#### Verified
- All mocks confirmed `#[cfg(test)]` or `#[cfg(feature = "mock")]` gated — zero
  production exposure
- All `dyn` usage audited — remaining are standard `Box<dyn Error>` and
  `Box<dyn Fn>` callbacks, both idiomatic Rust
- No hardcoded addresses in production code (all in tests or documentation)
- No `TODO`/`FIXME`/`HACK` markers in production code
- `cargo clippy --workspace --all-targets` — 0 warnings
- `cargo test --workspace --all-features` — 6,144 tests passing, 0 failures

### UUI Boundary Analysis — Owns vs Leverages (April 17, 2026)

#### Changed
- **Dead direct deps removed** from `petal-tongue-ui` — `png` (zero source usage),
  `winit` (never imported; transitive via eframe).
- **Capability discovery unified** — `GpuComputeProvider` and `physics_bridge`
  now use `CapabilityDiscovery<BiomeOsBackend>` as primary discovery path,
  falling back to existing env vars and filesystem scans.
- **V2 display backend fixed** — `DiscoveredDisplayBackendV2` replaced broken
  `TarpcClient` with direct JSON-RPC over Unix sockets for `display.*` ops.
- **Audio Tier 1 `NetworkBackend`** wired — discovers `audio` capability
  providers via `CapabilityDiscovery` and delegates playback via `audio.play`
  over JSON-RPC/UDS. Graceful fallback to software/silent when no ecosystem
  provider exists.
- **`discovered-display` feature gate** properly wired — `#[cfg(feature =
  "discovered-display")]` applied to `DiscoveredDisplayBackend`,
  `DiscoveredDisplayBackendV2`, and their `DisplayBackendImpl` variants.

#### Verified
- `cargo test --workspace --all-features` — 6,144 tests passing, 0 failures
- `cargo clippy --workspace --all-targets` clean
- Compiles cleanly with and without `discovered-display` feature

### reqwest Elimination — Songbird TLS Delegation (April 17, 2026)

#### Changed
- **`reqwest` runtime dependency fully eliminated** — replaced with thin
  `LocalHttpClient` in `petal-tongue-ipc` built on `hyper` + `hyper-util`
  (already transitive from `axum`, zero new crate additions).
- **Entire TLS transitive chain removed from lockfile** — `reqwest`,
  `hyper-rustls`, `rustls`, `rustls-webpki`, `ring` all gone. petalTongue
  no longer owns any TLS stack; Songbird handles that via tower atomic IPC.
- **6 crates migrated**: `petal-tongue-api`, `petal-tongue-core`,
  `petal-tongue-discovery`, `petal-tongue-entropy`, `petal-tongue-ui`,
  `petal-tongue-adapters` (dead dep removed).
- **13 code sites replaced** — BiomeOS client, mDNS provider, entropy
  streaming, SSE consumer, protocol selection, universal discovery all
  migrated to `LocalHttpClient`.
- **`LocalHttpClient`** provides GET, POST (JSON + raw), streaming SSE,
  configurable timeouts, connection pooling — designed for Songbird IPC
  backend substitution.

#### Verified
- `cargo tree -i reqwest` → "did not match any packages"
- Zero `reqwest`, `ring`, `hyper-rustls`, `rustls` in Cargo.lock
- 6,010+ tests passing, 0 failures
- `cargo clippy --workspace --all-targets` clean

### Stadial Parity Gate Response (April 16, 2026)

#### Changed
- **`reqwest` 0.12 → 0.13** — zero-breakage upgrade; `default-features = false`,
  no TLS features, ecoBin clean. `ring` confirmed phantom lockfile entry (Cargo
  resolver artifact from optional `hyper-rustls`; never compiled).
- **Test fixture gating fix** — `HangHealthCheckProvider` / `FailingHealthCheckProvider`
  evolved from `#[cfg(test)]` to `#[cfg(any(test, feature = "test-fixtures"))]` so
  integration tests (`chaos_tests.rs`) can access them.
- **CLI duplicate import fix** — removed duplicate `use` block in
  `petal-tongue-cli/src/handlers/tests.rs`.
- **Discovery doctest fix** — added `VisualizationDataProvider` trait import to
  `petal-tongue-discovery` crate-level doctest.

#### Verified
- `ring` not in build graph (`cargo tree -i ring` empty for all targets/features/edges)
- `dyn` audit: 23 total occurrences, 6 production (closures + std Error), all non-trait-object
- Zero production `.unwrap()`, zero TODO/FIXME, zero unsafe
- `cargo deny check bans` passes, Edition 2024, deny.toml enforced
- 6,110+ tests passing, 0 failures

### Sprint 8 — dyn Elimination & Modern Rust Evolution (April 16, 2026)

#### Changed
- **22 custom `dyn` trait objects eliminated** — all evolved to enum dispatch or generics:
  - 8 async traits: `ComputeProvider`, `GUIModality`, `DiscoveryBackend`, `Sensor`,
    `AudioBackend`, `DisplayBackend`, `UIBackend`, `VisualizationDataProvider`
  - 14 non-async traits: `PanelInstance`, `PanelFactory`, `ToolPanel`, `TufteConstraint`,
    `DataStream`, `AdaptiveUIRenderer`, `SensoryUIRenderer`, `PropertyAdapter`,
    `SchemaMigration`, `StatePersistence`, `InputAdapter`, `InversePipeline`,
    `MathObject`, `TelemetrySubscriber`
- **`async-trait` crate fully removed** — zero `#[async_trait]` annotations (was 47),
  zero `Pin<Box<dyn Future>>` type aliases, native `async fn` / `impl Future` throughout
- **11 production modules refactored** — `panel_registry`, `btsp`, `cli_mode`, `braille`,
  `biomeos_discovery`, `handlers` (CLI), `metrics_dashboard`, `interaction` (graph),
  `adaptive_rendering`, `instance`, `state` (TUI) — all under 600 LOC
- **Hardcoded ecosystem path** `"biomeos"` in server.rs evolved to
  `ecosystem_runtime_dir_name()` (env-configurable via `ECOSYSTEM_RUNTIME_DIR`)
- **1 production `.unwrap()` fixed** in `neural_api_provider/mock_server.rs`

#### Verified
- Zero `dyn` for custom traits (only `dyn std::error::Error` + `dyn Fn` remain — idiomatic Rust)
- Zero `#[async_trait]` annotations, zero `Pin<Box<dyn Future>>` type aliases
- Zero production `.unwrap()` outside `#[cfg(test)]`
- Zero TODO/FIXME/HACK, zero unsafe blocks, zero compiler warnings
- All 19 crate roots + main.rs have `#![forbid(unsafe_code)]`
- `cargo fmt`, `cargo clippy -D warnings`, `cargo test --all-features` all clean
- 183 tests passing (workspace binary + integration), 6,100+ total with `--workspace`

### Sprint 7 — Deep Domain Refactoring & Capability Evolution (April 15, 2026)

#### Changed
- **14 production modules smart-refactored by domain** — `wad_loader`, `raycast_renderer`,
  `sensory_matrix`, `visualization_handler/types`, `graph_editor/rpc_methods`,
  `protocol_selection`, `unix_socket_rpc_handlers/system`, `primal_panel`,
  `collector` (telemetry), `traffic_view/view`, `graph_canvas/rendering`,
  `device_panel`, `config_system`, `compiler` — each decomposed into
  single-responsibility submodules.
- **4 test files refactored** — `app/tests`, `traffic_view/tests_extended`,
  `trust_dashboard/tests`, `headless_panel_coverage_tests` — split by test domain.
- **BTSP provider default**: `"beardog"` → `"security"` (capability-based, not
  primal-specific). Socket fallback paths now use centralized `LEGACY_TMP_PREFIX`.
- **Duplicate socket path templates**: `primal_registration.rs` fallback centralized
  through `constants::LEGACY_TMP_PREFIX`.
- **Root docs unified**: Test counts, file size policy, sprint status synchronized
  across README.md, START_HERE.md, CONTEXT.md.

#### Verified
- Zero `.unwrap()` in production code (all confined to `#[cfg(test)]`).
- Zero TODO/FIXME/HACK markers.
- `ring` not in dependency tree for Linux target; `deny.toml` ban operational.
- No C dependencies, no `build.rs`, ecoBin pure-Rust compliant.
- Production mocks properly feature-gated (`mock`, `test-fixtures`).
- Provenance trio already capability-based (discovers by `dag.session` etc.).
- 5,960+ tests passing, 0 failures across entire workspace.

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
- **Dead `nokhwa`/`mozjpeg-sys` C dependency** from `petal-tongue-entropy` (video feature
  never wired; mozjpeg-sys pulls C compiler).
- **Dead optional deps** from `petal-tongue-ui`: `softbuffer`, `pixels`, `wasm-bindgen`,
  `wasm-bindgen-futures`, `web-sys` (features declared but never used in code).
- **3 dead features**: `software-rendering`, `compute-wasm`, `video`.

#### Lint & Quality Evolution
- **4 crates graduated**: `#[allow(missing_docs)]` → `#[warn(missing_docs)]` (docs complete
  for petal-tongue-tui, petal-tongue-cli, petal-tongue-api, petal-tongue-ui-core).
- **5 conditional `#[allow(dead_code)]`** → `#[expect(dead_code, reason = "...")]`.
- **6 more test modules extracted** to companion `_tests.rs` files:
  `output_verification`, `multimodal_stream`, `biomeos_ui_manager`,
  `discovered_display`, `accessibility`, `universal_discovery`.
- **5 more `format!("{}", x)`** → `.to_string()` (axes.rs, discovery_service_provider.rs).
- **~25 more doc comments** evolved to capability-based language across 12+ files.

#### Verified
- `cargo fmt --check` ✅
- `cargo clippy --workspace --all-features -D warnings` ✅ (0 warnings)
- `cargo doc --workspace --all-features -D warnings` ✅
- `cargo test --workspace --all-features` ✅ (6,090+ passed, 0 failures)
- `cargo deny check` ✅ (advisories, bans, licenses, sources ok)
- `ring` absent from dep tree (default AND `--all-features`)
- Zero production files over 680 LOC (32 test modules extracted across sprints)

---

### Added
- **BTSP Phase 1** (`crate::btsp`): `validate_insecure_guard()` refuses startup
  when both `FAMILY_ID` and `BIOMEOS_INSECURE=1` are set. Family-scoped socket
  naming (`petaltongue-{family_id}.sock`) in production posture. Domain symlink
  (`visualization-{family_id}.sock`) follows `PRIMAL_SELF_KNOWLEDGE_STANDARD`.
  Guard runs before any subcommand in `main.rs`. (PT-08)
- **BTSP Phase 2 handshake enforcement** (`perform_server_handshake`): Real BearDog
  delegation via `btsp.session.create`, `btsp.session.verify`, and `btsp.negotiate`
  JSON-RPC methods. First-byte peek on both TCP (`TcpStream::peek`) and UDS
  (`BufReader::fill_buf`) detects BTSP vs plain JSON-RPC. Handshake failure
  rejects the connection with `error!` logging. (PT-09)
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
