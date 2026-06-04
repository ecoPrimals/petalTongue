# Changelog

All notable changes to petalTongue are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Deep Debt Pass 3: TRUE PRIMAL TLS + NESTGATE Removal + Complete Idiom Sweep (June 3, 2026)

Third and final deep debt pass: sovereignty sweep, dead constant removal,
complete idiom migration.

#### Changed
- **TRUE PRIMAL TLS labels**: `viz_data/nucleus.rs` TLS handshake visualization
  evolved from hardcoded `Songbird`/`BearDog` names to protocol-based labels
  (`TLS provider`, `X.509`, `X25519`, `HMAC verify`).
- **NESTGATE_SOCKET removed**: Deprecated constant fully deleted (zero
  references remained after Wave 69 migration to `CONTENT_BACKEND_SOCKET`).
- **Complete idiom sweep**: ALL remaining `"literal".to_string()` in production
  code replaced with `.to_owned()` — 600+ replacements across 195 files.
  Zero instances remain in non-test code.
- **Clippy `assigning_clones`**: All `clone_into` patterns fixed.

### Wave 76 Consolidation: S3 Cutover Readiness (June 3, 2026)

Content backend mesh-aware 4-tier resolution audit and fixes for DNS cutover.

#### Changed
- **FAMILY_ID alignment**: Content backend and `announce_to_neural_api` defaults
  changed from `"default"` to `"nat0"` (ecosystem standard), matching
  `DiscoveryServiceClient::discover()` and `socket_path::get_family_id()`.
- **DISCOVERY_SOCKET wired into Tier 4**: `DiscoveryServiceClient::discover()`
  now honors `DISCOVERY_SOCKET` env var as highest-priority override, aligning
  with NUCLEUS composition pattern where both registration and discovery use
  the same Songbird socket.
- **AppError::TracingInit**: New typed variant for tracing/logging initialization
  errors, eliminating 4 `AppError::Other(format!())` sites in `init_tracing()`.
- **Stale doc cleanup**: Removed duplicate `resolve()` doc comment, stale
  `content.get` module reference.
- **Idiom sweep**: 300+ additional `.to_string()` → `.to_owned()` across 27 files.

#### Verified
- 4-tier content backend resolution chain architecturally complete.
- Tiers 1–3 have integration tests; Tier 4 structurally sound.
- **6,217+ tests pass**, zero Clippy warnings, `unsafe_code = "forbid"` enforced.

### Deep Debt Pass 2: AppError Evolution + Async Safety + Idiom Sweep (June 3, 2026)

Second deep debt pass: evolving error types, fixing async-safety, broad idiom sweep.

#### Changed
- **AppError typed sources**: Added `#[from]` conversions for `ConfigError`,
  `IpcServerError`, `AddrParseError`, `serde_json::Error`, and
  `tokio::task::JoinError`. Eliminated 11 `AppError::Other(format!())`
  call sites in `main.rs`, `web_mode/mod.rs`, `server_mode.rs`,
  `live_mode.rs`, `cli_mode/mod.rs`, and `cli_mode/gather.rs`.
- **Async-safe file I/O**: `content_direct.rs` index and fallback handlers
  migrated from blocking `std::fs::read_to_string` to `tokio::fs::read_to_string`.
- **Idiomatic Rust sweep**: Replaced 220+ additional `"literal".to_string()`
  with `.to_owned()` across 7 more files: `gather.rs`, `scenario/convert.rs`,
  `tutorial_mode.rs`, `trust.rs`, `ai_adapter.rs`, `status_reporter.rs`,
  `jsonrpc_provider.rs`.
- **Clippy cleanup**: Removed unfulfilled `too_many_lines` expectation from
  `main()` (function simplified by typed error evolution); fixed
  `clone_into` pattern in `status_reporter`.

#### Verified
- Telemetry paths follow DH-1 tiered resolution (`PETALTONGUE_TELEMETRY_DIR`
  > `XDG_DATA_HOME` > `/var/lib` > `/tmp` last resort) — not hardcoding debt.
- All `localhost:8080` references are test-only fixtures.
- Clone hotspots in `demo_device_provider` / `status_reporter` are
  trait-mandated or lock-safety patterns — acceptable.
- **6,217 tests pass**, zero Clippy warnings, `unsafe_code = "forbid"` enforced.

### Deep Debt Pass 1: TRUE PRIMAL + Typed Errors + Idiom Sweep (June 3, 2026)

Comprehensive deep debt audit and resolution pass.

#### Changed
- **TRUE PRIMAL compliance**: Removed hardcoded primal names from
  `config/capability_registry.toml` — content owner is now `content-provider`
  (not `nestgate`), discovery owner is `discovery-service` (not `songbird`).
- **Viz data agnostic labels**: `viz_data/nucleus.rs` and `viz_data/kderm.rs`
  now use capability-based labels (e.g., "AI inference", "Content storage",
  "inner-proxy", "gate-user") instead of hardcoded primal names.
- **Typed content backend errors**: `content_backend.rs` RPC helpers evolved
  from `Result<String, String>` to `Result<_, ContentBackendError>` with
  `thiserror`-derived variants: `Connect`, `Write`, `Serialize`, `Base64`,
  `Protocol`.
- **Idiomatic Rust**: Replaced 220+ `"literal".to_string()` calls with
  `.to_owned()` across `network.rs`, `shader_lineage.rs`,
  `gpu_compute_provider.rs`, `demo_device_provider.rs`, and audio backends.
- **CLI doc cleanup**: Removed stale `nestgate` backend alias from `--backend`
  help text.

#### Verified
- Audio stubs (`direct.rs`, `socket.rs`) properly feature-gated behind
  `audio-direct` / `audio-socket` — not compiled into default builds.
- `DemoDeviceProvider` correctly gated behind `#[cfg(feature = "mock")]`.
- Texture attach handler is a valid partial implementation, not a mock leak.
- **6,217 tests pass**, zero warnings, `unsafe_code = "forbid"` enforced.

### Wave 74 Sovereign Verify + Mesh Testing + Coverage (June 3, 2026)

Wave 74 — sovereign rendering verification, mesh content routing integration
tests, WASM bundle profiling, content backend test coverage, security hardening.

#### Changed
- **Path traversal hardening**: `resolve_docroot_path` now filters to
  `Component::Normal` only, stripping `..`, `.`, and root segments. Prevents
  directory escape from docroot.
- **Upstream merge resolution**: Restored `resolve_biomeos_socket_dir()` removed
  by upstream; resolved conflict markers in `content_backend.rs`.

#### Added
- **Content backend integration tests**: 8 new tests covering all 4 discovery
  tiers (socket override, TCP override, convention socket, fallback), tier
  priority (socket beats TCP), `ContentEndpoint` display format, successful
  resolve via UDS mock, JSON-RPC error handling, TCP transport integration test,
  TCP connect failure, `content_index` dashboard fallback.
- **Path traversal test**: Verifies `..` components are stripped from docroot
  resolution.

#### Profiled
- **WASM bundle**: 610K raw, 191K gzipped (release build, wasm32-unknown-unknown)
  - Dep tree: 6 direct deps (wasm-bindgen, console_error_panic_hook, serde,
    serde_json, petal-tongue-types, petal-tongue-scene)
  - `toml` + `tracing` elimination from Wave 73 confirmed effective
  - Next trim targets: feature-gate `petal-tongue-scene` modules unused by WASM,
    slim `bytes` serde feature, consider `serde-wasm-bindgen` for zero-copy

#### Verified
- Sovereign rendering: all 5 components (index.html, WASM, web_mode, handlers,
  content_direct) confirmed origin-agnostic with zero external dependencies.
- `cargo fmt --check`: clean
- `cargo clippy --workspace`: 0 warnings
- `cargo test --workspace`: 6,217 passed, 0 failed

### Wave 73 Sovereign + Mesh + Optimization (June 3, 2026)

Wave 73 — sovereign rendering verification, mesh-aware content routing,
WASM bundle trimming, continued tokio scope reduction.

#### Changed
- **Mesh-aware content routing**: `ContentBackendClient` evolved from Unix-only
  to multi-transport (`ContentEndpoint::Unix` | `Tcp`). New 4-tier resolution:
  `CONTENT_BACKEND_SOCKET` → `CONTENT_BACKEND_ENDPOINT` (TCP) → socket-dir
  convention → `discovery.query("content")` mesh fallback. Cross-gate content
  rendering (e.g. flockGate → NestGate on eastGate) now possible via TCP
  JSON-RPC or discovery service.
- **WASM bundle trim**: Removed `toml` and `tracing` from `petal-tongue-scene`
  deps (zero usage in scene sources). `PageMeta.extra` type changed from
  `HashMap<String, toml::Value>` to `HashMap<String, serde_json::Value>`,
  eliminating the `toml`+`winnow` parser chain from WASM builds. `PageMeta`
  gained `Eq` derive (now possible without `toml::Value`).
- **Tokio scope reduction (9→6 production crates)**: Removed dead `tokio` from
  `petal-tongue-ui-core` and `petal-tongue-headless`. Removed dead
  `petal-tongue-discovery` from headless (eliminates transitive ipc/tokio from
  headless binary). Removed dead `tokio-util` from `petal-tongue-tui`.

#### Added
- `CONTENT_BACKEND_ENDPOINT` env var for explicit TCP cross-gate content routing.
- Discovery-based content resolution via `discovery.query("content")` capability.

#### Verified
- Sovereign rendering: zero hardcoded GitHub Pages URLs, zero CDN deps.
  WASM exports are synchronous and origin-agnostic. `web/index.html` uses
  only relative `/api/` paths. Ready for Caddy-served sovereign infrastructure.
- `cargo fmt --check`: clean
- `cargo clippy --workspace`: 0 warnings
- `cargo test --workspace`: 6,209 passed, 0 failed

### Wave 69 Deep Debt + Modernization Pass (June 2, 2026)

Wave 69 — error typing evolution, dependency narrowing, dead code elimination,
idiomatic Rust modernization.

#### Changed
- **Error typing**: `DirError` manual `Display`/`Error` → `thiserror` derive.
  `HeadlessError::IoError(String)` → `Io(#[from] std::io::Error)` with typed
  `ScenarioLoad` variant for JSON parse errors. `AppError` gained typed `Io`
  variant, manual `From<io::Error>` removed.
- **Tokio dep narrowing**: Removed tokio from 4 crates that didn't use it in
  production (graph, animation, adapters, telemetry). Moved to dev-deps for 2
  test-only crates (entropy, cli). Narrowed features for api to
  `[net, io-util, time, rt]`.
- **Dead code elimination**: `VizEntry` gained `Serialize`, `slug`/`title`/
  `description` fields now live (API response type). `VizRegistry::get()` wired
  into `build_scene`/`build_animation`. New `list()` method. `ContentDirectState.nav`
  wired to `/api/nav` endpoint. New `/api/viz` listing endpoint.
- **ProcStats non-Linux**: `cpu_count()` uses `std::thread::available_parallelism()`
  instead of hardcoded `1`. `total_memory()` reads `PETALTONGUE_TOTAL_MEMORY_BYTES`
  env fallback.
- **Idiomatic Rust**: `.to_string()` on string literals → `.to_owned()` across
  viz_data, IPC handlers, WASM compilers, headless graph_loader, socket_path.
- **TRUE PRIMAL (Wave 69)**: Removed `nestgate` backend alias from `web_mode`,
  removed env fallback from `content_backend`, deprecated `NESTGATE_SOCKET`
  constant. Dep trim: `tarpc/unix`, `egui_extras`, `rustix` 0.38→1.x. IPC
  evolution: `grammar_placeholder` → `identity_grammar`, texture attach slot
  registration semantics.

#### Verified
- `cargo fmt --check`: clean
- `cargo clippy --workspace`: 0 warnings
- `cargo test --workspace`: 6,208 passed, 0 failed

### flockGate W67/W68 Review + Content Pipeline Wiring (June 1, 2026)

Wave 67 — reviewed and integrated flockGate deliverables: content rendering pipeline,
VizRegistry pattern, and document scene graph types.

#### Added
- **Module wiring**: `content_render`, `viz_data/`, and `web_mode/content_direct`
  modules are now compiled into the binary (previously orphaned dead code).
- **Inline variants**: `Inline::Strikethrough` and `Inline::Image` added to document
  scene graph and handled by both `compile_markdown` and modality compilers (HTML +
  description). Closes gap where pulldown-cmark parsed them but they were silently dropped.
- **Table shortcode resolution**: `resolve_shortcodes` now walks `DocumentNode::Table` cells.

#### Changed
- **TRUE PRIMAL fix**: Removed hardcoded `/primals/{key}/` and `/springs/{key}/` URL
  fallbacks from `expand_entity_shortcodes`. Entity href now comes from registry `page`
  field only — no ecosystem layout assumptions baked into petalTongue.
- **Serde robustness**: `PageMeta`, `EntityRegistryEntry`, `SiteContent` gained
  `#[serde(default)]` for safe partial deserialization.
- **Type quality**: All document types gained `PartialEq` (+ `Eq` where `toml::Value`
  absence allows). Enables `assert_eq!` in downstream tests.
- **Doc coupling cleanup**: Removed sporePrint/NestGate name coupling from module docs
  and HTML output in `content_render.rs` and `content_direct.rs`.

#### Fixed
- `toml` workspace dependency added to root binary crate (was missing, blocked compilation
  of `content_render`).

### Deep Debt Cleanup + DH-1 Compliance (May 29, 2026)

Wave 61 ecosystem tightening — dep trim, TRUE PRIMAL fix, mock isolation, DH-1 /tmp cleanup.

#### Changed
- **DH-1 /tmp cleanup**: All production socket and data writes now resolve through
  the tiered chain: `BIOMEOS_SOCKET_DIR` > `XDG_RUNTIME_DIR` > `/run/user/{uid}` > `/tmp`.
  `resolve_biomeos_socket_dir()` is the canonical resolver. Unblocks `ProtectSystem=strict`.
- **Dependency trim**: Removed dead `mdns-sd`, trimmed `tokio/full` → explicit 8-feature set,
  dropped unused `serde/rc`, `clap/cargo`, `tower-http/set-header`. Bumped `tower` 0.4 → 0.5.
- **TRUE PRIMAL**: `content_backend.rs` default provider changed from `"nestgate"` to
  `"content-provider"` (capability-based, not primal-coupled).
- **Mock leak isolation**: UI auto-fallback no longer injects fake primals on empty discovery.
  Headless binary demo topology now requires `--demo` or `SHOWCASE_MODE=true`.
  Sensory discovery probes Linux audio subsystems before reporting capabilities.
- **Global CLI flags**: `--socket`, `--port`, `--family-id` accepted before subcommands (Wave 54).

#### Removed
- `mdns-sd` workspace dependency (never imported; custom mDNS in `mdns_provider/`)
- `mdns_discovery.rs` module and `mdns` feature from `petal-tongue-discovery`
- Unconditional demo topology from headless binary default path

### Deep Debt Resolution + TRUE PRIMAL Evolution (May 24, 2026)

Wave 47 behavioral convergence + capability-based discovery rewiring.

#### Changed
- **web_mode/mod.rs smart refactor** (1136L → 3 focused files):
  `mod.rs` (185L orchestrator), `handlers.rs` (263L HTTP + SSE + fallback),
  `tests.rs` (652L). No file over 800 lines in the workspace.
- **NestGate → capability-based content_backend**: `nestgate.rs` replaced by
  `content_backend.rs`. `NestGateContentClient` → `ContentBackendClient`.
  Socket discovery: `CONTENT_BACKEND_SOCKET` > `NESTGATE_SOCKET` (compat) >
  `CONTENT_BACKEND_PROVIDER`-based convention. CLI accepts both `"nestgate"`
  (compat) and `"content-provider"` as `--backend` values.
- **BTSP overstep removed**: `BEARDOG_SOCKET` and `BEARDOG_FAMILY_SEED`
  replaced with role-based `BTSP_PROVIDER_SOCKET` / `SECURITY_PROVIDER_SOCKET` /
  `BTSP_FAMILY_SEED` / `FAMILY_SEED`. No production code references another
  primal by name.
- **Display V1 discovery**: Added `DISPLAY_BACKEND_SOCKET` role-based env
  override (was orchestrator-only `BIOMEOS_SOCKET`).
- **Provenance trio discovery**: Refactored hardcoded match arms to generic
  domain-prefix extraction with well-known aliases. Added
  `PROVENANCE_TRIO_SOCKET` shared env override.
- **Stale references cleaned**: Removed "rodio" from audio log messages
  (replaced with "hound" / "pure Rust audio"), removed "cpal" comments,
  removed cross-primal port comment (rhizoCrypt 9600) from network constants.

#### Added
- **Graceful shutdown (SIGTERM + SIGINT)**: Shared `signal.rs` module used by
  all three long-running modes (web via `with_graceful_shutdown`, server via
  `tokio::select!`, live via spawned signal task). Per
  `DEPLOYMENT_BEHAVIOR_STANDARD.md`.
- **`health.liveness` normalized**: Returns exactly `{"status":"alive"}` on
  both HTTP (`/health/liveness`) and IPC (`health.liveness`). Removed legacy
  `"alive":true` field.

### S3 Shadow Parity: GitHub Pages Equivalence (May 19, 2026)

Wave 24 content hosting shadow run (S3) — proving sovereign petalTongue +
NestGate can replace GitHub Pages without regression.

#### Added
- **Gzip + Brotli compression**: `tower_http::compression::CompressionLayer`
  on all HTTP responses. Automatic content negotiation via `Accept-Encoding`.
  Enabled `compression-gzip` and `compression-br` tower-http features.
- **Security headers**: `X-Content-Type-Options: nosniff`,
  `X-Frame-Options: DENY`, `Referrer-Policy: strict-origin-when-cross-origin`,
  `Permissions-Policy: camera=(), microphone=(), geolocation=()` on all
  responses via axum middleware. Matches GitHub Pages security posture.
- **HTTP request tracing**: `tower_http::trace::TraceLayer` with structured
  spans (`method`, `uri`) and response logging (`status`, `latency_ms`).
  Feeds shadow run TTFB and 404 rate metrics via tracing subscriber.
- **Custom 404 pages**: `{docroot}/404.html` served with status 404 when
  present (GitHub Pages / Jekyll convention). Falls back to plain text if
  no custom page exists. `Cache-Control: no-cache` on error pages.
- tower-http features: `set-header`, `compression-gzip`, `compression-br`.

### Stale Socket Cleanup + PID File (May 18, 2026)

#### Changed
- **Socket startup**: Replaced `exists()` + conditional remove with
  unconditional `remove_file()` ignoring `NotFound`. Eliminates TOCTOU race
  between existence check and removal. Both `unix_socket_server.rs` and
  `server.rs` paths hardened.
- **Socket shutdown (Drop)**: Simplified to unconditional `remove_file` with
  `is_dir` fallback (for edge cases). PID file cleaned up alongside socket.

#### Added
- **PID file**: `petaltongue.pid` written alongside `petaltongue.sock` on
  startup, containing the server PID. Enables instant `kill(pid, 0)` liveness
  checks by consumer primals without connect overhead. Removed on shutdown.
  Per `DEPLOYMENT_VALIDATION_STANDARD.md` §stale-socket-cleanup.

### Stadial Gate Readiness (May 17, 2026)

#### Added
- **`btsp.capabilities` JSON-RPC method**: Returns supported BTSP protocol
  version, cipher suite (chacha20-poly1305), key derivation (hkdf-sha256),
  and active BTSP status. Classified Public in MethodGate.
- **`primal.announce` dispatch alias**: Routes to `capability.announce` per
  stadial gate requirement. Classified Public in MethodGate.
- **`/health/liveness` + `/health/readiness` HTTP routes**: Web mode now
  exposes the health triad as HTTP endpoints matching the JSON-RPC shape.
- **`count` field in `capabilities.list` response**: Method count is computed
  from the actual methods vec (stadial wire standard compliance).
- **`proprioception.get` + `btsp.capabilities`** added to capabilities methods list.
- **`checksums.toml`**: BLAKE3 hashes for Cargo.toml, Cargo.lock, manifest.toml,
  src/main.rs per PLASMIDBIN_PUSH_AUTOMATION_STANDARD.
- **`seed_fingerprint`** in manifest.toml: BLAKE3 hash of source identity.
- **Manifest method registry**: Updated from 12 to 55 methods to match the
  full dispatch table (was missing health triad, auth, visualization session,
  motor, interaction, and system methods).
- **Stability tiers**: All methods annotated as Stable or Evolving in CONTEXT.md.
- **Degradation documentation**: What happens when petalTongue is down.
- **Downstream pairing**: esotericWebb, lithoSpore, projectNUCLEUS, wetSpring.
- **Platform audio documentation**: Per-crate audio dependency breakdown.

#### Changed
- **Web `/health` response enriched**: Now returns `primal`, `version`, `mode`
  alongside `status` (was bare `{"status":"ok"}`).
- **README.md**: Added version (v1.6.6) to header.
- **START_HERE.md**: Updated date to May 17, 2026.

#### Fixed
- **Pre-existing clippy lints** (petal-tongue-scene, petal-tongue-graph):
  `cast_sign_loss`, `manual_midpoint`, `too_many_lines`, `redundant_closure`,
  `let_else`, `long_literal_lacking_separators`, `manual_clamp`,
  `expect_used` in tests, `format_args`, `match_same_arms`, `const_fn`.

### Live Dashboard + NestGate Index Routing (May 13, 2026)

#### Changed
- **`web/index.html` live dashboard**: Rewrote the embedded dashboard from a
  static status dump to a live ecosystem view.  Subscribes to `/api/events`
  (SSE) for real-time `DataSnapshot` updates, fetches `/api/snapshot` on load,
  renders discovered primals (health pill, type, capabilities, endpoint) and
  topology edges.  Dark/light mode via `prefers-color-scheme`.  Status bar
  shows mode, backend, and event counter.
- **NestGate-aware `GET /`**: When `backend=nestgate`, the root route tries
  `content.resolve("/")` from NestGate first, falling back to the compiled-in
  dashboard if NestGate has no published root document.  Filesystem backend
  continues to serve the embedded dashboard directly.
- **Stale doc cleanup**: `CONTEXT.md` updated — `backend=nestgate` is
  UNBLOCKED (NestGate Session 60 shipped `content.*` transport parity).

### SPA Catch-All + CORS Production Config (May 11, 2026)

#### Added
- **`--spa` / `PETALTONGUE_SPA`**: SPA (single-page application) mode. When
  enabled, missing paths serve `{docroot}/index.html` instead of 404, enabling
  client-side routing for React/Vue/Svelte/etc. SPAs.
- **`--allowed-origins` / `PETALTONGUE_ALLOWED_ORIGINS`**: CORS configuration
  via `tower_http::cors::CorsLayer`. Comma-separated origins, or `"*"` for
  permissive. Allows GET/POST/OPTIONS with Content-Type + Authorization headers.
- `WebServeConfig.spa` and `WebServeConfig.allowed_origins` fields with
  TOML, env, and CLI override support.
- `build_cors_layer()` — constructs tower-http CORS middleware from config.
- `serve_spa_index()` — serves docroot `index.html` for SPA catch-all.
- `tower-http` `"cors"` feature enabled.
- 5 new tests: SPA routing, non-SPA 404, CORS wildcard, CORS specific origins,
  CORS preflight response.

### Notebook Rendering + PT-3 Completion (May 10, 2026)

#### Added
- **Jupyter notebook renderer** (`src/notebook_render.rs`, 553 LOC).
  `.ipynb` files served via docroot or NestGate are rendered to complete HTML
  documents with responsive styling and dark-mode support.
- **`metadata.title`** → `<title>` + `<h1>` page header; falls back to "Notebook".
- **`--strip-sources` / `PETALTONGUE_STRIP_SOURCES`**: hides code input cells,
  showing only outputs (for documentation/presentation mode).
- **`--cache-ttl` / `PETALTONGUE_CACHE_TTL`**: `Cache-Control: max-age` header
  on all served static content (wires `WebServeConfig.cache_ttl_secs`).
- Markdown cells rendered via `pulldown-cmark` (CommonMark + tables, strikethrough,
  task lists, footnotes). Code cells as `<pre><code>` with language annotation.
  Rich outputs: HTML passthrough, SVG, base64 images, plain text, error tracebacks.
- `WebServeConfig.strip_sources` field with TOML + env override.
- `is_ipynb()` case-insensitive extension helper.
- `docroot_fallback()` — custom Axum handler replacing `ServeDir` for `.ipynb`
  rendering and `Cache-Control` injection on all responses.
- New dependency: `pulldown-cmark 0.13` (pure Rust, `html` feature only).
- 14 notebook rendering unit tests + 4 web-mode integration tests.

#### Changed
- `web_mode::run()` now takes `WebConfig` struct (was 8 positional params).
- NestGate fallback now renders `.ipynb` content as HTML before serving raw bytes.
- `WebConfig.workers` doc updated (was stale "currently logged only").

### JH-0 MethodGate Pre-Dispatch Authorization (May 8, 2026)

#### Added
- **JH-0: MethodGate module** (`crates/petal-tongue-ipc/src/method_gate.rs`, 494 LOC).
  Every JSON-RPC call now passes through `MethodGate::check()` before reaching the
  handler dispatch table.
- **Method classification**: Public methods (always allowed): `health.*`, `identity.get`,
  `capabilities.list`, `capability.list`, `lifecycle.status`, `auth.*`, legacy aliases
  (`ping`, `status`, `check`). Everything else is Protected.
- **Auth introspection methods**: `auth.check`, `auth.mode`, `auth.peer_info` —
  intercepted inline before dispatch, advertised in `capabilities.list`.
- **CallerContext threading**: Connection origin (Unix/Loopback/Remote) and bearer
  token tracked per-connection through NDJSON, BTSP Phase 3, and all dispatch paths.
- **Error codes**: `-32001 PERMISSION_DENIED`, `-32000 UNAUTHORIZED`.
- **`PETALTONGUE_AUTH_MODE` env**: `permissive` (default, backward-compatible) or
  `enforced` (rejects unauthenticated protected calls).
- 26 unit tests in `method_gate::tests`.

#### Changed
- `RpcHandlers::handle_request()` now accepts `&CallerContext`.
- `handle_connection`, `handle_connection_split`, `handle_encrypted_stream`,
  `try_phase3_negotiate`, `try_phase3_upgrade_split` — all accept `&CallerContext`.
- `handle_tcp_with_btsp` now accepts `peer_addr: SocketAddr` for origin detection.
- All existing test call sites updated to pass `CallerContext::unix()`.

### primalSpring Phase 60 — PT-09 + PT-13 (May 7, 2026)

#### Added
- **PT-13 (P2): NestGate content-addressed backend for `web` mode.** When
  `--backend nestgate` (or `PETALTONGUE_WEB_BACKEND=nestgate`), the HTTP fallback
  queries NestGate `content.resolve` via JSON-RPC over UDS instead of serving
  from the filesystem. Socket discovery follows ecosystem convention
  (`NESTGATE_SOCKET` env → `$BIOMEOS_SOCKET_DIR/nestgate-{family}.sock`).
  Returns content with correct MIME type; 404 on missing; 502 on backend failure.
- **PT-09 (P2): BTSP Phase 2 enforcement.** When `FAMILY_ID` is set (production
  posture), unauthenticated connections are now **rejected** instead of warned.
  UDS: `run_uds_handshake` returning `None` (plain JSON-RPC, handshake failure,
  or EOF) results in connection drop with `warn!` log. TCP: plain JSON-RPC
  without BTSP announcement is rejected with `warn!`. petalTongue is now aligned
  with all 12 other primals on BTSP enforcement.
- `--backend` CLI flag for `web` mode (default `"filesystem"`).
- 9 new tests: NestGate client construction, env override, ID increment,
  fallback 502, backend install, CLI `--backend` parsing (2), backend default.

#### Changed
- Extracted `dispatch_web()` from `dispatch_async()` to stay under line limit.
- `web_mode::run` signature expanded: accepts `backend: &str`.

### projectNUCLEUS Sovereignty Gaps (PT-1 through PT-5) — May 7, 2026

#### Added
- **PT-1 (High): `--docroot` static file catch-all** for `web` mode. When
  `--docroot <path>` (or `PETALTONGUE_DOCROOT` env) is provided, a
  `tower_http::ServeDir` fallback serves arbitrary files from that directory.
  `append_index_html_on_directories(true)` makes `GET /` serve `index.html`.
  API routes (`/health`, `/api/*`) take precedence. Unblocks sovereign static
  site serving (sporePrint, Zola builds, GitHub Pages replacement).
- **PT-3 (Medium): `WebServeConfig` schema** in `config_system::types`.
  New config section `[web]` with `docroot`, `backend` (filesystem|nestgate),
  `index_file` (default `index.html`), `cache_ttl_secs` (default 3600).
  CLI `--docroot` overrides config. `PETALTONGUE_DOCROOT` env override wired.
- **PT-4 (Medium): `--ipc` flag for `web` mode** (NUCLEUS dual-port mode).
  When `--ipc` is passed, `web` mode co-starts the UDS JSON-RPC server
  alongside the HTTP server. Optional `--ipc-port` for TCP JSON-RPC.
  Enables single-process HTTP+IPC for NUCLEUS deployments.
- **PT-5 (Low): `--workers` flag now wired to tokio runtime**. `web` and
  `headless` modes' `--workers N` value is passed to
  `tokio::runtime::Builder::worker_threads(N)`. Previously logged but ignored.
- 7 new tests: docroot path validation, catch-all serving, API route precedence,
  CLI `--docroot` parsing, docroot default None.

#### Changed
- `web_mode::run` signature expanded: accepts `docroot: Option<String>`.
- Extracted `WebConfig` struct and `run_with_config` for testability.

### PG-55 `--bind` Flag — primalSpring Phase 60 (May 6, 2026)

#### Added
- **`--bind` flag for `server` and `live` modes** (PG-55): TCP bind host is now
  configurable via `--bind <IP>` or `PETALTONGUE_IPC_HOST` env var. Secure
  default `127.0.0.1` — Docker/network deployments use `--bind 0.0.0.0`.
  Matches Squirrel SQ-04 / coralReef `--bind` ecosystem pattern.
- `UnixSocketServer::with_tcp_bind_host()` builder method for programmatic override.
- `PrimalRegistration::with_tcp_endpoint(host, port)` replaces `with_tcp_port(port)`,
  carrying the actual bind host into the Songbird `ipc.register` payload.
- 5 new tests: CLI `--bind` parsing, `parse_ipc_bind_host` for wildcard/IPv6/invalid/default.

### Cross-Cutting Audit Response — primalSpring Phase 59 (May 6, 2026)

#### Added
- **Tier-1 Songbird registration with transport endpoints**: `ipc.register`
  payload now includes `transports: { uds: "<socket_path>", tcp: "0.0.0.0:PORT" }`.
  TCP endpoint advertised when `--port` is active (`server`/`live` modes).
  Songbird `ipc.resolve` can now route to petalTongue directly without probing.
  New `PrimalRegistration::with_tcp_port()` builder method. 3 new tests.
- **Whitespace-tolerant TCP/UDS protocol detection**: Both accept paths now
  skip leading ASCII whitespace before classifying the first meaningful byte
  (sweetGrass `detect_protocol` tolerance pattern). `is_btsp_json_announcement`
  also whitespace-tolerant. Prevents misclassification when peers send
  leading CR/LF/spaces.

#### Fixed
- **BufReader post-negotiate byte loss on TCP JSON-line path**: TCP JSON-line
  BTSP handshake now splits + BufReaders the stream **before** the handshake
  (calling `relay_json_line_handshake_split` instead of the combined
  `relay_json_line_handshake`). The same BufReader is carried through to
  Phase 3 negotiate + encrypted framing, preventing prefetched post-handshake
  bytes from being lost when a transient BufReader was dropped. Aligned with
  barraCuda Sprint 51b / coralReef Iter 90 fix pattern.

#### Verified
- **Wire Standard L3 on `capabilities.list`**: Already compliant —
  `capabilities.list` returns `protocol: "json-rpc-2.0"` and
  `transport: ["unix-socket", "tcp"]` dynamically (TCP included when
  `tcp_enabled` is set via `--port`).

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
