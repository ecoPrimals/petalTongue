# petalTongue — Evolution Record

The changelog is sorted by capability surface, not by date.
Each section tells the arc: what emerged, what it replaced, what it became.
The date-sorted history lives in `CHANGELOG_ARCHIVE.md`.

`H(data|epitope) < H(data|date) < H(data)`

---

## Visualization & Scene Grammar — declarative rendering engine

Rendering began as ad-hoc chart functions and legacy domain renderers; it became a Grammar-of-Graphics pipeline with unified 2D/3D scene graphs, modality compilers, and a registry of named visualizations.

- **Grammar of Graphics pipeline** (Wave 100→150t): `visualization.render.grammar` replaced per-chart hardcoded renderers; `GrammarCompiler` wires z-depth, camera, and projection for universal 2D-as-3D-slice rendering.
- **Scene graph unification** (Wave 150t): `Transform3D`, `Camera`, `Projection` on `SceneNode` — single scene model for SVG, WebGL, and WASM exports.
- **VizRegistry** (Wave 116→140a): Named slugs (`gate-mesh`, `gonzales-ic50`, pharmacology scenes) with scene + animation builders; `/viz/{slug}` serves SVG, scene-json, or animation-json.
- **Legacy renderer excision** (Wave 120): Five pre-SceneGraph domain renderers removed (695→187L); all visualization routes through grammar pipeline.
- **Gonzales tangibles pivot** (Wave 140a): IC50, PK decay, tissue lattice, hormesis scenes — modular `gonzales/` directory replacing monolithic 918L file.
- **DataBinding evolution** (Wave 150t): 13→14 binding variants; `ColorGrid` for bingoCube widget progressive reveal integration.
- **Modality compilers** (Wave 150t): SVG, WebGL, and dashboard modality paths share scene graph input; `compile_modality` returns `&'static str` (zero heap alloc per render).

---

## WebGL & Browser Pipeline — server-push scene streaming

Browser rendering started as client-only WASM SVG; it evolved into a full WebGL compilation bridge with real-time scene streaming over WebSocket.

- **WebGlCompiler** (Wave 150t): Scene graph → GPU draw commands (vertex/index buffers, draw calls); exposed via `pt.render_webgl` JSON-RPC and WASM exports.
- **`/ws/scene` streaming** (Wave 157d): WebSocket endpoint with subscribe/unsubscribe protocol; `SceneStreamState` broadcast channel pushes compiled `WebGlScene` frames to browsers.
- **WebGL compilation bridge** (Wave 157d): `webgl_bridge` module — DoomFrame/SceneGraph → WebGL → broadcast in one call; `compile_rects_and_publish()` for rectangle primitives.
- **G19 auto-publish** (Wave 157e): `visualization.render.grammar` auto-publishes GPU-compiled scenes to `/ws/scene` subscribers when `scene_publish_tx` is set.
- **WebSocket JSON-RPC bridge** (Wave 150g–150h): `/ws` on port 8080 integrates JSON-RPC into Axum web server — footPrint Caddy routes directly, eliminating separate bridge port.
- **FD limit self-healing** (Wave 157d): `platform_substrate::raise_fd_limit()` at startup mirrors biomeOS pattern — gates without `LimitNOFILE=65536` self-heal.

---

## Web Dashboard & Gate Mesh — ecosystem overwatch HUD

The dashboard began as static HTML; it became a live mesh overwatch surface consuming Neural API topology, coordination manifests, and ecosystem validation data.

- **Gate mesh visualization** (Wave 116→121): `gate_mesh` module as single source of truth; WireGuard overlay SVG, enrollment status, latency labels; `gate.mesh.status` IPC method.
- **Live topology (TOPO-VIS)** (Wave 137b): `/api/topology/live` consumes Neural API discovery; routing weights on edges; typed SSE `event: topology` every 30s.
- **Ecosystem dashboard** (Wave 123→124): `/api/ecosystem` returns NUCLEUS composition; GPU target tracking on mesh nodes; primalSpring `gpu-compute-pipeline.json` scenario.
- **Physical topology** (Wave 128): `/api/physical-topology` — edge router, backbone switch, port forwards from manifest; zero hardcoded IPs in handlers.
- **Coordination backend** (Wave 132d–f): Six `/api/coord/*` endpoints read nestGate CAS manifests; mesh peers, sporePrint, K-Derm layers in dashboard panels.
- **Composition serving** (Wave 137b): Axum mounts external web bundles at `/app/{name}/` with SPA fallback; `PETALTONGUE_COMPOSITIONS` env discovery.
- **Manifest-driven handlers** (Wave 136b→140a): sporePrint, ecosystem, mesh handlers evolved from hardcoded test counts to runtime `ecosystem_manifest.toml` parsing.

---

## Site Builder & Static Generation — content to deployable sites

Static site generation emerged from the visualization stack as a separate capability for footPrint GIS and esotericWebb browser surfaces.

- **SiteBuilder foundation** (Wave 150t): `ContentSource` trait + `SiteBuilder` compiles `SiteContent` into HTML/CSS/JSON with layout, navigation, search index.
- **sporePrint pipeline** (Wave 151b): `FilesystemSource` and `ContentDirectState::generate_static_site()` batch-generate sites from content-direct backend.
- **WASM exports** (Wave 150t): `build_site()`, `render_page_with_layout()` for client-side static generation without server roundtrip.
- **InMemorySource** (Wave 150t): Testing/WASM content source for SiteBuilder unit tests.

---

## IPC, RPC & Capability Registry — the served method surface

The RPC surface grew from scattered handlers to a self-audited dispatch table with 57 served methods aligned to capability registry.

- **Self-audit convergence** (Wave 157a): All 57 served methods match `dispatch.rs` exactly; phantom `viz.*` and `render.dashboard` namespaces removed.
- **G65 protocol negotiation** (Wave 156m): Single-socket tarpc/JSON-RPC selection via `PROTOCOLS:` wire handshake; backward-compatible legacy default.
- **C2 dual-socket tarpc** (Wave 156i): tarpc UDS server alongside JSON-RPC on shared transport infrastructure.
- **BTSP ClientHello** (Wave 151c): Consumer-side handshake for bearDog strict mode; JSON-line and binary Phase 2 entry points.
- **riboCipher prefix acceptance** (Wave 113): Accept loops detect `0xEC`/`0xED`/`0xEE` before protocol fork — Anderson membrane tier mapping.
- **HEALTH-01 compliance** (Wave 110): Bare `{"method":"health"}` returns `{status, primal, version, uptime_s}` per ecosystem contract.
- **Neural API announce** (Wave 137b): Outbound `primal.announce` with routing metadata, cost hints, latency estimates on startup.

---

## Transport & Platform Substrate — cross-architecture IPC

Transport evolved from hardcoded Unix sockets to platform-agnostic endpoints with Windows Named Pipe and Android cdylib support.

- **TransportEndpoint abstraction** (Wave 141a→143b): `petal-tongue-platform` crate with `PlatformLifecycle`, C-FFI entry points; UDS on Unix, Named Pipe on Windows.
- **G68 platform substrate** (Wave 157a): Centralized symlinks, permissions, UID queries, page size, socket detection — all inline `#[cfg(unix)]` routed through one module.
- **Runtime discovery** (Wave 157g): Dynamic primal discovery via filesystem scan replaces hardcoded 13-primal endpoint list; gossip socket dirs resolve via env at runtime.
- **Cross-arch compliance** (Wave 156v→157i): x86_64-linux, aarch64-apple-darwin, x86_64-windows all pass; darwin `rustix::process::test_kill_process` fix.
- **Hardcoding elimination** (Wave 155g→157i): Zero hardcoded peer socket paths, loopback addresses centralized in `constants::DEFAULT_LOOPBACK_HOST`.
- **Axum 0.8 upgrade** (Wave 157i): `Message::Text` takes `Utf8Bytes`; route params `/{param}`; tower-http 0.5→0.6.

---

## Gossip & Swarm Integration — ant colony announcements

petalTongue joined the swarmVine mesh as an announcing surface, broadcasting availability without blocking render paths.

- **Gossip injection module** (Wave 157e): `inject_gossip()` async best-effort spread; constructors for `surface.web.live`, `scene_stream_active`, `content_serve_available`.
- **Startup announcements** (Wave 157e): `web_mode::run()` injects `surface.web.live` on server start; scene stream injects on subscriber connect.
- **SwarmVine socket discovery** (Wave 157e→157g): Centralized `discover_swarmvine_socket()` via env-resolved search dirs, not hardcoded `/run/membrane`.

---

## doom-core & Game Engine — extractable rendering substrate

The Doom-style raycast engine decoupled from petalTongue scene crate for future `ludoSpring` extraction.

- **doom-core decoupling** (Wave 157a): `DoomFrame`/`FrameRect` scene-agnostic output; `DoomInstance::render_frame()` returns `DoomFrame` not `SceneGraph`.
- **Dependency slimming** (Wave 157a): doom-core down to 3 deps (was 5); `petal-tongue-scene` dependency removed; 8 unused workspace deps excised across 5 crates.

---

## Health, Discovery & Platform Metrics — knowing system state

Health and metrics evolved from stubs to real probes with cross-platform abstraction.

- **Dynamic health discovery** (Wave 157g): `data_service/health.rs` rewritten — filesystem scan of runtime socket dir extracts primal names, prefers `-default.sock`.
- **PlatformMetrics trait** (Wave 150f): Abstracts CPU/memory/uptime across Linux (`/proc`), Android, iOS, WASM stub fallback.
- **NestGate integration readiness** (Wave 77d): Provider cache, capability discovery wiring for storage-backed visualization data.
- **AEAD consolidation** (Wave 116): Dropped `aes-gcm`; entropy crate uses `XChaCha20-Poly1305` single cipher story.

---

## Architecture & Engineering Discipline — how the codebase evolved

The codebase transformed through systematic debt elimination, feature gating, and cross-architecture hardening.

- **Feature gating** (Wave 120): `tui` and `cache` features optional — headless deployments skip ratatui and LRU provider cache.
- **Telemetry excision** (Wave 157g): `petal-tongue-telemetry` vestigial crate removed (1,038 LOC, zero imports).
- **Clippy zero-warning** (Wave 156l→157i): Pedantic + nursery clean across 16 crates; `#[allow]` → `#[expect]` Rust 2024 idiom.
- **Tarpc convergence** (Wave 156h): tarpc 0.37 cephalization — dual-protocol accept on single socket infrastructure.
- **Topology deduplication** (Wave 116): `DataSourceId`/`GrammarId` serde bugs fixed; gate mesh constants as single source of truth.
- **Test scale** (Wave 157i): 6,644 passing, 0 failures; cross-arch clean on linux/darwin/windows.

---

*Full date-sorted history: `CHANGELOG_ARCHIVE.md`*
*Last compressed: Wave 172*
