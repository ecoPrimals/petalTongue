# petalTongue — Context

**Version:** 1.6.6
**Role:** Universal User Interface primal (visualization, presentation, interaction)
**License:** AGPL-3.0-or-later (scyBorg triple: AGPL + ORC + CC-BY-SA 4.0)

---

## What This Is

petalTongue is ecoPrimals' visualization and user interface primal. It translates
ecosystem state into every available modality — desktop GUI (egui), terminal TUI
(ratatui), web (axum), headless (SVG/PNG/JSON), and WASM. It implements a
Grammar of Graphics engine with a declarative scene graph and animation system.

petalTongue is a **meta-tier** primal: it presents data from other primals but
does not own computation, storage, or security domains.

## Architecture

18 workspace crates, single UniBin binary (`petaltongue`, 7 subcommands):

| Crate | Purpose |
|-------|---------|
| `petal-tongue-core` | Types, config, sensory discovery, capability registry |
| `petal-tongue-ipc` | JSON-RPC 2.0 server (UDS + TCP), BTSP, push delivery |
| `petal-tongue-scene` | Declarative scene graph, modality compilers |
| `petal-tongue-graph` | Chart rendering, sonification |
| `petal-tongue-animation` | Manim-style animation system |
| `petal-tongue-ui` | Native GUI (egui/eframe), feature-gated |
| `petal-tongue-tui` | Terminal UI (ratatui) |
| `petal-tongue-ui-core` | Pure Rust abstract UI (text, SVG, canvas) |
| `petal-tongue-discovery` | Primal/capability discovery clients |
| `petal-tongue-cli` | CLI handler logic |
| `petal-tongue-api` | BiomeOS client, HTTP APIs |
| `petal-tongue-entropy` | Human entropy capture |
| `petal-tongue-adapters` | Adapter framework |
| `petal-tongue-headless` | Headless rendering binary |
| `petal-tongue-telemetry` | Observability and metrics |
| `petal-tongue-types` | WASM-portable data types |
| `petal-tongue-wasm` | Browser rendering module |
| `doom-core` | Doom WAD rendering (platform stress test) |

## IPC Surface

JSON-RPC 2.0 over Unix domain sockets (primary) and TCP (`--port`).
55 methods across domains: `visualization.*` (render, stream, grammar, dashboard,
scene, export, validate, session, texture, introspect, panels, showing, dismiss),
`interaction.*`, `health.*`, `capabilities.*`, `capability.*`, `identity.*`,
`ui.*`, `motor.*`, `audio.*`, `lifecycle.*`, `topology.get`,
`proprioception.get`, `provider.register_capability`, `auth.*`, `btsp.*`,
`primal.announce`.

BTSP Phase 1 complete: family-scoped socket naming, insecure guard,
domain symlinks (`visualization.sock`). BTSP Phase 2 complete: security
provider handshake delegation on both UDS and TCP, length-prefixed and
JSON-line framing, `btsp.session.create`, `btsp.session.verify`, and
`btsp.negotiate` via provider client. BTSP Phase 3 complete:
ChaCha20-Poly1305 AEAD encrypted frame I/O after negotiate; HKDF-SHA256
directional key derivation; 13/13 ecosystem parity.

JH-0 MethodGate: pre-dispatch authorization on all JSON-RPC calls.
Public methods (`health.*`, `identity.get`, `capabilities.list`,
`lifecycle.status`, `auth.*`) always pass. Protected methods
(`visualization.*`, `interaction.*`, `ui.*`, `motor.*`, `audio.*`)
require a bearer token in enforced mode. Default: permissive.
Env: `PETALTONGUE_AUTH_MODE=enforced`. Auth introspection:
`auth.check`, `auth.mode`, `auth.peer_info`.

## Key Design Decisions

- **Two-dimensional universality**: universal across modalities (what you see)
  and substrates (what you run on).
- **Grammar of Graphics**: primals send grammar expressions, petalTongue
  compiles to best available representation.
- **No self-compute**: heavy work (GPU, physics) delegated via IPC to
  compute, display, and ledger capability providers. petalTongue discovers by capability.
- **Feature-gated GUI**: `ui` feature (default) pulls egui/eframe/glow.
  Headless builds (`--no-default-features`) have zero native display deps.
- **Audio discovery**: tiered backends — ecosystem primal (Tier 1, via
  capability discovery), socket, direct, software, silent. Socket/direct
  behind optional features; software/silent always available.

## UUI Boundary — Owns vs Leverages

petalTongue is the UUI engine: pure Rust rendering to any modality on any
device. Other primals own platform interaction points.

**Owns (pure Rust, in-crate):**
- egui (layout/interaction), epaint (tessellation), tiny-skia (rasterization)
- crossterm (terminal I/O), symphonia (audio decode/synthesis)
- Grammar of Graphics, scene graph, animation, modality adapters
- IPC server: `visualization.*`, `interaction.*`, `capabilities.sensory.*`

**Leverages (ecosystem primals via `capability.call` / JSON-RPC over UDS):**
- `display.*` — display capability provider (window lifecycle, frame presentation)
- `compute.*` / `math.*` — compute capability provider (GPU dispatch)
- `btsp.session.*` — security provider (transport security)
- `discovery.*` / `ipc.*` — discovery + registry providers (routing)
- TLS/HTTPS — TLS capability provider relay (design ready)
- `audio.play` / `audio.stream` — audio capability provider (stub, Tier 1)
- `storage.put` / `storage.get` — storage capability provider (future)
- `ai.query` / `ai.complete` — AI capability provider (future)

The eframe/glow C/FFI stack exists only behind `ui-eframe` feature as a
development convenience. The architectural path is EguiPixelRenderer →
DisplayManager → ecosystem `display.*` IPC.

## Ecosystem Position

petalTongue discovers other primals at runtime via capability-based IPC.
It has zero compile-time knowledge of primal identities in production builds
(fixture data gated behind `#[cfg(test)]` or `test-fixtures` feature).

Coordinates with biomeOS (orchestration) and any primal that exposes
security, registry, or visualization-relevant
capabilities.

## Build

```bash
cargo build --release                     # Full binary (26M musl-static)
cargo build --release --no-default-features  # Headless only
cargo test --workspace --all-features     # 6,454+ tests, ~85-90% coverage
```

## Current State

Wave 113 riboCipher Prefix Acceptance (June 14, 2026). 6,460+ tests, all passing.
UDS connections now accept and strip the `[0xEC, 0x01]` riboCipher signal
prefix before BTSP classification or JSON-RPC parsing. HEALTH-01 compliant
(bare `"health"` → enriched check with `uptime_s`). Zero `/tmp` literals.
Zero TODO/FIXME/HACK. `TransportEndpoint` type with `connect_transport()`.
Zero self-binding anti-patterns.

Stadial parity gate cleared (April 17, 2026). All CI gates pass (fmt,
clippy pedantic+nursery, doc, cargo deny, tests). Zero unsafe, zero
TODO/FIXME, zero production `unwrap()`, zero `#[allow(` in production code
(only in `#[cfg(test)]` modules). Lint policy: `#[expect]` with
reasons for justified suppressions; targeted `#[allow]` only where
`#[expect]` cannot apply (e.g. `cfg_attr` platform gates). SPDX headers
on all source files. Edition 2024, deny.toml enforced.

Native `async fn` in traits (April 25, 2026): eliminated all manual
`fn -> impl Future + Send` desugaring across 13 production modules.
All traits now use native `async fn` (RPITIT). Zero `manual_async_fn`
suppressions remain. Net −100 lines across 21 files.

macOS cross-arch build fix (April 19, 2026): conflicting `AudioCanvas`
impl blocks resolved; `petal-tongue-ui` now compiles with zero warnings
on x86_64-apple-darwin and aarch64-apple-darwin.

`reqwest` runtime dependency fully eliminated (April 17). Replaced with
thin `LocalHttpClient` (hyper + hyper-util, already transitive from axum).
`ring`, `hyper-rustls`, `rustls`, `rustls-webpki` all removed from lockfile.
petalTongue no longer owns any TLS stack — TLS-capable ecosystem provider
handles external HTTPS via tower atomic IPC.

Sprint 8: complete `dyn` trait object elimination (22 custom traits
evolved to enum dispatch / generics), `async-trait` removed from all
first-party code (native `async fn` in traits via RPITIT; may remain
as transitive dep via tarpc/axum), `Pin<Box<dyn Future>>` type aliases
eliminated, 11 production modules refactored below 600 LOC, hardcoded
ecosystem paths evolved to env-configurable constants.

Sprint 7: deep debt resolution across 14 production modules (smart
refactoring by domain, not mechanical splitting), hardcoding evolved to
capability-based defaults, BTSP provider default evolved from primal
identity to capability name (`security`), centralized socket path constants.

UUI boundary analysis (April 17): dead deps removed (`png`, direct `winit`),
capability discovery unified (`GpuComputeProvider` and `physics_bridge` now
use `CapabilityDiscovery<BiomeOsBackend>` as primary path), V2 display backend
fixed (tarpc→JSON-RPC for `display.*` ops), audio Tier 1 wired (`NetworkBackend`
via capability discovery, graceful fallback), `discovered-display` feature
properly gated with `#[cfg]`.

Dependency cleanup (April 19, 2026): dead `petal-tongue-graph` dep removed
from `petal-tongue-ui-core`, headless builds no longer pull egui (graph
`default-features = false`), tarpc trimmed from `full` to specific features.

`petaltongue live` mode (April 21, 2026): new CLI subcommand merging `ui`
(egui/eframe) and `server` (UDS JSON-RPC IPC) into a single process for
interactive desktop NUCLEUS deployment. IPC server runs as background tokio
task, egui on main thread, connected via `Arc<RwLock<VisualizationState>>`
and companion registries. This is the tier-one deployment mode for every
spring/garden cell graph.

BTSP wire-format detection fix (April 21, 2026): three-way classification
in `handle_uds_with_btsp` and `handle_tcp_with_btsp` — non-`{` first byte
→ length-prefixed BTSP, `{` + `"protocol"` → BTSP JSON-line announcement
(from primalSpring), `{` only → plain JSON-RPC. Fixes misclassification of
`{"protocol":"btsp",...}` announcements as invalid JSON-RPC.

Deep debt audit (April 21, 2026): clippy zero warnings achieved (boxed
`DoomPanelWrapper`, removed needless return, fixed unused async). `futures`
→ `futures-util` in discovery crate. All 4 remaining `dyn` usages audited
as idiomatic (callbacks + error trait).

BTSP JSON-line handshake relay (April 23, 2026): primalSpring Phase 45c
upstream debt resolved. New `btsp/json_line.rs` module implements full
4-step JSON-line BTSP relay (ecosystem pattern). UDS/TCP accept now routes
JSON-line BTSP announcements to the new relay instead of the
length-prefixed handler. Provider field names aligned (`session_token`,
`response`, `family_seed`), provider challenge used (not local PRNG),
`SECURITY_PROVIDER_SOCKET`/`CRYPTO_PROVIDER_SOCKET`/`SECURITY_SOCKET`
added to provider socket cascade.

PG-40 fix (April 26, 2026): `petaltongue live` and `petaltongue ui` no
longer panic on Linux. winit event loop now runs on main thread; IPC
server spawns on tokio runtime. `PETALTONGUE_SOCKET` env var bound via
clap `env` attribute.

Eliminate all `dyn` from production code (April 26, 2026): `PanelInstance::on_error`
→ `&impl std::error::Error`, `SseEventConsumer`/`EventStream` callbacks → typed
`tokio::sync::mpsc::UnboundedSender` channels. Zero `dyn` in production Rust code.

PG-43: Texture Primitive + IPC Methods (April 26, 2026): `Primitive::Texture`
variant with `texture_id`, position, size, UV rect, opacity, tint. `TextureRegistry`
in `VisualizationState`. `visualization.texture.upload` (base64 RGBA) and
`visualization.texture.attach` (shared-memory placeholder) IPC methods.
`From<Sprite> for SceneNode` bridge. All 12 exhaustive match sites updated.
Overlay mode deferred (display capability Phase 2 dependency).

Dependency consolidation (April 26, 2026): uuid unified to workspace 1.9,
tokio-tungstenite deduplicated to workspace dep, tarpc `tcp` feature removed
(only Unix transport used), chrono trimmed to clock+serde, physics_bridge
hardcoded paths replaced with LEGACY_TMP_PREFIX constant.

PG-48 fix (April 27, 2026): musl/plasmidBin binaries no longer panic in
`live` mode. Added `EventLoopBuilderExtX11::with_any_thread(true)` hook
and enabled explicit `x11` + `wayland` eframe features. `winit` added as
direct workspace dep for platform traits (zero new crate).

PG-53: `proprioception.get` IPC method (April 27, 2026): synthetic
proprioception snapshot for composition scripts. Server mode returns
`frame_rate: 0`, `window: null`; live/UI returns `frame_rate: 60`,
`window: { present: true }`. Also returns `active_scenes`, `total_frames`,
`user_interactivity`, `mode`, `uptime_secs`.

PG-53 follow-up (April 27, 2026): `rendering_awareness` was unconditionally
`Some(...)` in `UnixSocketServer::new_with_socket`, so server-mode
`proprioception.get` falsely reported `mode: "live"` and `frame_rate: 60`.
Fixed: removed unconditional init from server constructor; only `live` mode
now wires `rendering_awareness` via `with_rendering_awareness()`.

`--socket` CLI flag (reconfirmed April 27, 2026): already wired since
PT-10 (April 10). Both `--socket` flag and `PETALTONGUE_SOCKET` env var
functional on `server` and `live` subcommands.

Deep debt audit (April 27, 2026): comprehensive audit of 847 .rs files.
Zero unsafe, zero dyn in production, zero #[allow(] in production, zero
TODO/FIXME/HACK. No files >650 lines. tempfile version skew (3.8/3.10/3)
unified to workspace dep. petal-tongue-cli clap consolidated to workspace.
SVG viewport, SSE keepalive, HTTP stream timeout, web static dir all
extracted to named constants. All mocks gated behind #[cfg(test)] or
test-fixtures feature.

Deep debt pass 2 (April 27, 2026): toml, tokio-util, rustix consolidated
to workspace deps. Stale `external-display` feature alias removed.
`universal_discovery.rs` socket search now includes XDG_RUNTIME_DIR as
priority-1 (was missing — only searched /tmp and /var/run).

Phase 55 audit response (April 28, 2026): three primalSpring asks addressed.
(1) AWAKENING_ENABLED default changed to false — awakening now off by default,
compositions invoke via new `motor.set_awakening` IPC method. Awakening is
invocable, not a hardcoded default. (2) Scene push signing implemented: new
`SceneSigner` module uses BLAKE3 keyed hash with `PETALTONGUE_SCENE_KEY` env
var (visualization purpose key per NUCLEUS Two-Tier Crypto Model). Scene pushes
include `signature` field; `visualization.scene.verify` IPC method added.
(3) sensor_stream evolved: new discrete event types — `focus_gained`,
`focus_lost`, `window_resize`, `text_input` — added to `SensorEventIpc`.
Focus and text input wired in `sensor_feed.rs` collection. 6,045+ tests.

Deep debt audit (April 28, 2026): 15 crates consolidated to workspace deps
(futures-util, crossterm, terminal_size, tiny-skia, epaint, png, svg,
indexmap, colored, socket2, dashmap, lru, ron, ratatui, symphonia).
Telemetry fallback path `/tmp/petaltongue-telemetry` extracted to constant.
Comprehensive audit confirmed: zero unsafe, zero dyn in production, zero
TODO/FIXME/HACK, zero #[allow(] in production, all mocks properly gated,
all unwrap/expect in test code only. 6,045+ tests.

Phase 56 gap resolution (April 29, 2026): primalSpring v0.9.24 deployed
a live 12-primal Desktop NUCLEUS; gap report identified 3 petalTongue issues.
(1) GAP-01: `RegistrationClient` now reads `DISCOVERY_SOCKET` env var as
highest-priority override for heartbeat/registration. Heartbeat uses
exponential backoff on failure. (2) Motor P0: live mode's IPC motor channel
was a dead end (logging thread). Replaced with `replace_motor_channel` —
IPC motor commands now flow directly to the GUI's `drain_motor_commands`.
New `motor.panel.update` and `motor.notification` methods + MotorCommand
variants + PanelContentStore and NotificationQueue. (3) GAP-17: confirmed
`visualization-{family}.sock` symlink already created at server startup
via `btsp::domain_symlink_filename`. 6,054+ tests.

Deep debt audit (April 29, 2026): dependency consolidation — axum, tower,
tower-http, tokio-stream moved to [workspace.dependencies] (root crate was
last holdout with literal version pins). aes-gcm and zeroize in
petal-tongue-entropy likewise consolidated to workspace deps. Hardcoded
values extracted: `/var/run/ecoPrimals` → `constants::ALTERNATIVE_RUN_DIR`,
`"nucleus"` primal type → `capability_names::primal_types::NUCLEUS` (in
both scenario providers), proprioception staleness threshold `10` →
`constants::PROPRIOCEPTION_STALENESS_SECS`. Hot-path clone elimination:
`req.params.clone()` removed from texture upload/attach handlers (moved to
ownership via `let id = req.id; serde_json::from_value(req.params)`) —
avoids duplicating large texture payloads. Sparkline renderer simplified
(removed redundant nested length check). Comprehensive audit confirmed:
zero unsafe, zero dyn in production, zero TODO/FIXME/HACK, all mocks
gated, no files >710 lines. 6,054+ tests, 0 Clippy warnings.

PG-48 + Motor P0 resolution (April 30, 2026): primalSpring v0.9.24 remote
validation surfaced two blockers. (1) PG-48: musl plasmidBin binary panics
on live mode startup — `native_options_with_any_thread` only applied X11
`with_any_thread`, not Wayland. Fixed: both X11 and Wayland extension traits
now called with fully-qualified syntax. Also applied to `EguiBackend`
fallback path. winit added as direct dep to petal-tongue-ui for platform
traits. (2) Motor P0: `motor.panel.update` and `motor.notification` data
was stored in `PanelContentStore` and `NotificationQueue` but never rendered.
Fixed: composition panel content renders in floating "Composition Panels"
egui window with recursive JSON display. Notifications render as floating
toast overlays (up to 5) with level-appropriate colors (info/warn/error/
success). `drain_expired()` called each frame to auto-dismiss timed toasts.
6,054+ tests, 0 Clippy warnings.

Socket path centralization (April 30, 2026): 8+ duplicated socket search
path constructions across 6 crates replaced with single `socket_search_dirs()`
helper in `constants::network`. Canonical priority: XDG_RUNTIME_DIR →
/run/user/{uid} → /tmp → /var/run/ecoPrimals. All remaining inline `/tmp`
literals now use `LEGACY_TMP_PREFIX` constant. All 6 bare `#[expect]`
attributes gained reason strings (struct_excessive_bools, unnecessary_wraps,
upper_case_acronyms). Zero hardcoded `/tmp` in production code. 6,054+ tests,
0 Clippy warnings.

PT-04/PT-09/dev dep audit (April 30, 2026): primalSpring Phase 56c
audit items resolved. (1) PT-04: HTML export `compile_html` was using a
duplicated inline HTML template; now calls shared `wrap_svg_in_html`.
(2) PT-09 (Phase 56c): BTSP JSON-line relay path now calls `btsp.negotiate`
(was missing — only length-prefixed path called it). Both paths log negotiate
results. PT-09 (Phase 60): enforcement gate upgraded — unauthenticated
connections rejected (not just warned) when `FAMILY_ID` set. (3) PT-06:
push delivery confirmed already activated in all IPC modes (stale audit note). Dev deps consolidated: tokio-test, wiremock,
assert_cmd, predicates, criterion, temp-env, mdns-sd moved to workspace.
Graph rendering magic numbers extracted to `constants::display`
(GRAPH_NODE_RADIUS, stroke widths, label offsets, RGBA8_BYTES_PER_PIXEL).
6,054+ tests, 0 Clippy warnings.

BTSP Phase 3 transport switch (May 3, 2026): ChaCha20-Poly1305 AEAD
encrypted frame I/O after `btsp.negotiate` handshake. HKDF-SHA256
directional key derivation (client→server / server→client). Wire format:
`[4B BE length][12B nonce][ciphertext+tag]`. Both UDS and TCP paths handle
Phase 3 upgrade. 13/13 ecosystem parity achieved.

TRUE PRIMAL name evolution (May 3, 2026): comprehensive sweep removing
all hardcoded primal names from production code. BearDog → "security
provider", ToadStool → "display capability provider", Songbird → "TLS
provider", rhizoCrypt/sweetGrass/loamSpine → capability-based terms.
Test fixtures and historical provenance comments preserved.

Deep debt sweep (May 3, 2026): clippy pedantic+nursery zero warnings,
`--port` UniBin flag, dead code removal (`audio_web.rs`), broken doc
links fixed, CI evolved to `--all-features`. BTSP docs evolved from
"BearDog-derived" to "Ecosystem Transport Security Profile".

primalSpring Phase 58 audit response (May 4, 2026): all 5 audit items
resolved. Phase 3 encryption, musl/winit PG-48, PT-04 HTML export,
PT-06 push delivery all confirmed already shipped. GAP-12 closed:
`visualization.capabilities` now returns machine-readable `methods`
object with parameter schemas for all visualization methods (dashboard,
scene, render, export). BTSP uses role-based env vars
(`BTSP_PROVIDER_SOCKET`, `SECURITY_PROVIDER_SOCKET`, `BTSP_FAMILY_SEED`).
6,200+ tests, 0 Clippy warnings.

Port alignment + discovery hierarchy (May 5, 2026): ecosystem TCP fallback
port aligned to 9900 (moved from 9600 to avoid rhizoCrypt tarpc conflict).
`ECOSYSTEM_TCP_FALLBACK_PORT` constant added, included in
`DEFAULT_DISCOVERY_PORTS`. Discovery crate docs updated with 5-tier
escalation hierarchy aligned to primalSpring standard.

Cross-cutting audit response (May 6, 2026): primalSpring downstream audit
items resolved:

- **Tier-1 Songbird registration**: `ipc.register` now advertises concrete
  transport endpoints (`transports: { uds: "...", tcp: "0.0.0.0:PORT" }`).
  TCP endpoint included when `--port` is active (`server`/`live` modes).
  Songbird `ipc.resolve` can now route directly without probing.
- **BufReader post-negotiate fix**: TCP JSON-line BTSP path restructured
  to split + BufReader **before** handshake; same BufReader carried through
  to Phase 3 negotiate and encrypted framing. Prevents prefetch byte loss
  (barraCuda Sprint 51b / coralReef Iter 90 class of bug).
- **Whitespace-tolerant TCP protocol detection**: Both TCP and UDS accept
  paths now skip leading ASCII whitespace before classifying first byte
  (sweetGrass `detect_protocol` tolerance pattern). `is_btsp_json_announcement`
  also whitespace-tolerant.
- **Wire Standard L3**: Confirmed already compliant — `capabilities.list`
  returns `protocol: "json-rpc-2.0"` and `transport: ["unix-socket", "tcp"]`
  dynamically.

- **PG-55 `--bind` flag**: `server` and `live` modes now accept `--bind <IP>`
  (or `PETALTONGUE_IPC_HOST` env var) to configure TCP bind host. Secure
  default `127.0.0.1` — Docker/network deployments use `--bind 0.0.0.0`.
  Matches Squirrel SQ-04 / coralReef ecosystem pattern. Songbird
  `ipc.register` payload carries the actual bind host.

- **projectNUCLEUS sovereignty gaps (PT-1 through PT-5)**: PT-1 `--docroot`
  catch-all static file serving (RESOLVED). PT-3 `WebServeConfig` schema
  (RESOLVED). PT-4 `--ipc` dual-port mode for NUCLEUS (RESOLVED). PT-5
  `--workers` wired to tokio runtime (RESOLVED). PT-2/PT-13 content backend
  (RESOLVED — `--backend content-provider` queries `content.resolve` via
  capability-based socket discovery).

- **primalSpring Phase 60 (PT-09 + PT-13)**: PT-09 BTSP Phase 2 enforcement
  (RESOLVED — unauthenticated connections rejected when `FAMILY_ID` set,
  petalTongue now matches all 12 other primals). PT-13 content-addressed
  backend for `web` mode (RESOLVED — `--backend content-provider` with UDS
  JSON-RPC, capability-based discovery via `CONTENT_BACKEND_SOCKET`).

- **Notebook rendering** (May 10, 2026): Jupyter `.ipynb` files served from
  docroot or content backend are rendered as styled HTML pages. `metadata.title`
  populates `<title>` + `<h1>` header. `--strip-sources` /
  `PETALTONGUE_STRIP_SOURCES` hides code input cells. `--cache-ttl` /
  `PETALTONGUE_CACHE_TTL` sets `Cache-Control` headers. Markdown cells via
  `pulldown-cmark`; code cells as `<pre><code>`; rich outputs (HTML, SVG,
  base64 images). Pure Rust, dark-mode responsive CSS.

- **SPA catch-all + CORS** (May 11, 2026): `--spa` / `PETALTONGUE_SPA` serves
  `index.html` for missing paths (React/Vue/Svelte client-side routing).
  `--allowed-origins` / `PETALTONGUE_ALLOWED_ORIGINS` wires CORS via
  `tower_http::cors::CorsLayer`. Enables production deployment for SPA
  frontends and cross-origin API consumers.

6,217+ tests, 0 Clippy warnings, 0 doc warnings, 0 unsafe blocks.

Wave 76 consolidation + deep debt passes 1–3 (June 3, 2026): Five
passes in one session. (1) TRUE PRIMAL: `capability_registry.toml` evolved
from hardcoded `nestgate`/`songbird` to `content-provider`/`discovery-service`;
viz data labels genericized; TLS handshake viz evolved from `Songbird`/`BearDog`
to protocol-based labels. (2) Typed errors: `ContentBackendError`,
`AppError::TracingInit`, 5 `#[from]` typed conversions eliminating 15
`Other(format!())` sites total. (3) Async safety: `content_direct.rs`
blocking `std::fs` → `tokio::fs`. (4) S3 cutover readiness: FAMILY_ID
default aligned to `"nat0"` ecosystem-wide; `DISCOVERY_SOCKET` wired into
Tier 4 discovery; 4-tier content backend chain audited. (5) Complete idiom
sweep: **zero** `"literal".to_string()` remains in production code (1,000+
replacements across 195+ files). `NESTGATE_SOCKET` constant removed.

Wave 61 status (May 29, 2026): DH-1 /tmp cleanup complete (all socket paths
through `resolve_biomeos_socket_dir()` tiered chain). Dep trim: dead `mdns-sd`
removed, `tokio/full` → explicit features, `tower` 0.4→0.5. TRUE PRIMAL fix:
content backend default is `"content-provider"` (capability-based). Mock leaks
isolated: auto-fallback and headless demo gated behind explicit opt-in. Sensory
discovery probes Linux audio subsystems.

Wave 69 deep debt + modernization (June 2, 2026): TRUE PRIMAL — removed
`nestgate` backend alias and env fallback (constant fully removed in Wave 76).
Dep trimming — removed `tarpc/unix`, removed `egui_extras`,
bumped `rustix` 0.38→1.x. IPC evolution — `grammar_placeholder` →
`identity_grammar`, `handle_texture_attach` to slot registration semantics.
Modernization pass — `DirError` manual Display/Error → `thiserror` derive,
`HeadlessError::IoError(String)` → `Io(#[from] std::io::Error)` + typed
`ScenarioLoad` variant, `AppError` typed `Io` variant. Tokio dep narrowing:
removed from 4 crates (graph, animation, adapters, telemetry), moved to
dev-deps for 2 (entropy, cli), narrowed features for api. Dead code
eliminated: `VizEntry`/`VizRegistry` fully wired with `Serialize`, `list()`,
`get()`, `/api/nav` + `/api/viz` endpoints. `ProcStats` non-Linux evolution:
`cpu_count()` uses `available_parallelism()`, `total_memory()` reads env
fallback. Mechanical `.to_string()` → `.to_owned()` on string literals.

Remaining backlog: aarch64 musl cross-compile for headless, audio backend wire
protocols (via `audio.play` capability discovery), overlay mode (display
capability Phase 2), egui texture resolution (TextureResolver with
`egui::Shape::image`), `crypto.sign` delegation to security provider for scene
signing (currently local BLAKE3), Phase 3 self-hosted sporePrint (requires
petalTongue + Songbird + content provider coordination).
`backend=content-provider` is UNBLOCKED — content backend uses capability-based
socket discovery (`CONTENT_BACKEND_SOCKET` / `CONTENT_BACKEND_PROVIDER`).
Live dashboard wires SSE topology stream (`/api/events`), primal grid, and
content-aware index routing (`GET /` resolves through `content.resolve("/")` when
`backend=content-provider`, falling back to the compiled-in dashboard).

## Stadial Readiness (May 17, 2026)

**Gate status**: 9.5/10 — interstadial exit CLEARED.

### Method Stability Tiers

| Tier | Methods | Meaning |
|------|---------|---------|
| **Stable** | `health.check`, `health.liveness`, `health.readiness`, `health.get`, `identity.get`, `lifecycle.status`, `capabilities.list`, `capability.announce`, `primal.announce`, `btsp.capabilities`, `auth.check`, `auth.mode`, `auth.peer_info`, `topology.get`, `proprioception.get` | Wire-compatible across versions; no breaking changes without major version bump |
| **Stable** | `visualization.render`, `visualization.validate`, `visualization.export`, `visualization.capabilities`, `visualization.introspect`, `visualization.panels`, `visualization.showing`, `visualization.dismiss` | Core rendering pipeline — stabilized since v1.4 |
| **Evolving** | `visualization.render.stream`, `visualization.render.grammar`, `visualization.render.dashboard`, `visualization.render.scene`, `visualization.render.graph`, `visualization.session.list`, `visualization.session.status`, `visualization.texture.upload`, `visualization.texture.attach`, `visualization.scene.verify` | Functional but response shapes may evolve |
| **Evolving** | `visualization.interact.apply`, `visualization.interact.perspectives`, `interaction.subscribe`, `interaction.poll`, `interaction.unsubscribe`, `interaction.sensor_stream.*` | Interaction pipeline — stabilizing |
| **Evolving** | `audio.synthesize`, `ui.render`, `ui.display_status`, `motor.*`, `provider.register_capability`, `capabilities.sensory`, `capabilities.sensory.negotiate` | Domain-specific — stable within current consumers |

### Degradation Behavior

When petalTongue is unavailable:

- **Ecosystem impact**: No visualization, no live dashboard, no web-mode content
  serving. All primals continue operating — petalTongue is a representation
  layer, not a control plane.
- **Springs**: Springs that render dashboards (esotericWebb, lithoSpore) fall
  back to text/JSON output or cached state. No data loss.
- **projectNUCLEUS**: Static site serving stops if petalTongue hosts sporePrint.
  Content remains in content provider or filesystem; another HTTP server can serve it.
- **Composition graphs**: `petaltongue_deploy.toml` marks petalTongue as
  non-critical. biomeOS skips visualization steps when petalTongue is absent.
- **IPC callers**: Get connection refused → standard JSON-RPC retry/fallback.

### Downstream Pairing

| Partner | Integration | Status |
|---------|-------------|--------|
| **esotericWebb** | Game UI rendering via `visualization.render.scene` + `motor.*` | Functional |
| **lithoSpore** | Validation dashboard via `visualization.render.dashboard` + `/api/events` SSE | Spring-side |
| **projectNUCLEUS** | sporePrint sovereign serving via `web` mode + content backend | Functional |
| **wetSpring** | Fermentation visualization via `visualization.render.grammar` | Spring-side |

### Platform Audio Dependencies

petalTongue is **pure Rust** (`deny.toml` bans C crypto and native TLS). Audio:

- **Graph engine** (`petal-tongue-graph`): WAV-only output via `hound` crate.
  No native audio playback deps. Pure Rust on all platforms.
- **UI mode** (`petal-tongue-ui`): Decoding via `symphonia` (mp3, wav features).
  No system audio library required for decode. Playback delegates to
  `audio.play` capability discovery at runtime (not compiled in).
- **Headless/server/web modes**: No audio dependencies active. Build with
  `--no-default-features` to exclude the `ui` feature entirely.
- **Linux note**: `eframe` (egui backend) may pull transitive windowing deps
  (`wayland-sys`, `x11-dl`) for the `ui` mode. These are display deps, not
  audio. Build with `--no-default-features --features=""` for a zero-GUI binary.
- **macOS/Windows**: No additional system deps beyond standard windowing.
