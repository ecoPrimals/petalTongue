+++
title = "petalTongue Validation Summary"
description = "Universal user interface primal — 6,755 workspace tests, 56+ IPC methods, 19 crates, 7 modes, pure Rust, BTSP 13/13 + WebGL + sporePrint pipeline"
date = 2026-07-30

[taxonomies]
primals = ["petaltongue"]
springs = []
+++

## Status

- **v1.7.0** — workspace edition 2024, `forbid(unsafe_code)`
- **6,755 workspace tests** passing, 0 failed (unit + integration + doc + property)
- **56+ IPC methods** across 10 domain categories (health, identity, auth,
  capabilities, visualization, interaction, audio, UI, motor, BTSP, gate mesh)
- **19 workspace crates** (core, IPC, graph, scene, discovery, adapters,
  entropy, UI, TUI, headless, API, WASM, platform, and domain crates)
- **7 runtime modes**: `server`, `web`, `ui`, `tui`, `live`, `headless`, `status`
- **Zero unsafe code** (except confined C-FFI in `petal-tongue-platform/ffi.rs`, 15 SAFETY-documented usages)
- **BTSP Phase 3** encrypted transport (ChaCha20-Poly1305 + HKDF-SHA256)
- **MethodGate** (JH-0): public/protected method classification with auth enforcement
- **Scene unification**: 2D-as-3D-slice, all 4 phases complete (Transform3D, Camera/Projection, 3D geometry)
- **WebGL pipeline**: Scene graph → GPU draw commands via `WebGlCompiler`
- **Static site builder**: `ContentSource` trait + `SiteBuilder` + WASM exports (Zola replacement foundation)
- **bingoCube integration**: `ColorGrid` DataBinding + `render_color_grid_webgl` WASM export
- **Cross-architecture**: x86_64-linux, aarch64-linux, aarch64-android, x86_64-windows
- **Platform embedding**: `petal-tongue-platform` cdylib for Android/iOS with C-FFI
- **Stadial gate**: READY — all 8 glacial criteria clear
- **cargo deny**: fully clean (zero advisories)

## Key Capabilities

| Domain | Methods | Description |
|--------|---------|-------------|
| Health | 4 | Health triad (`liveness`, `readiness`, `check`) + `health.get` |
| Identity | 3 | `identity.get`, `lifecycle.status`, `proprioception.get` |
| Auth | 3 | `auth.check`, `auth.mode`, `auth.peer_info` (JH-0 MethodGate) |
| Capabilities | 5 | `capabilities.list` (with `count`), sensory matrix, negotiate |
| Visualization | 18 | Render, validate, export, grammar, dashboard, scene, texture, session |
| Interaction | 6 | Subscribe, poll, unsubscribe + sensor stream |
| Motor | 8 | Panel, zoom, fit, mode, navigate, awakening, notification |
| Audio | 1 | `audio.synthesize` (WAV via hound, pure Rust) |
| UI | 2 | `ui.render`, `ui.display_status` |
| BTSP | 1 | `btsp.capabilities` (cipher suite introspection) |

## Web Mode (S3 Shadow Parity)

- **Static file serving** from `--docroot` with directory index (`ServeDir`)
- **Content backend** (`--backend content-provider`): capability-based content via `content.resolve`
- **SPA catch-all** (`--spa`): client-side routing support
- **CORS** (`--allowed-origins`): configurable origin allowlist
- **Gzip + Brotli compression** via `CompressionLayer`
- **Security headers**: `X-Content-Type-Options`, `X-Frame-Options`, `Referrer-Policy`, `Permissions-Policy`
- **HTTP tracing**: structured `TraceLayer` (method, uri, status, latency_ms)
- **Custom 404**: `{docroot}/404.html` (GitHub Pages / Jekyll convention)
- **Jupyter notebook rendering**: `.ipynb` → HTML with `metadata.title`, `--strip-sources`
- **Live dashboard**: SSE topology stream, primal grid, health pills, gate mesh SVG, NUCLEUS composition panel

## Build & Deployment

- **Pure Rust**: `pure_rust = true`, `c_dependencies = []`
- **musl targets**: `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`
- **plasmidBin**: `manifest.toml` v1.7.0, `checksums.toml` (BLAKE3), `seed_fingerprint`
- **CI**: fmt, clippy (pedantic + nursery, `-D warnings`), test, doc — all green
- **Stale socket hygiene**: unconditional `unlink()` before `bind()`, PID file,
  Drop cleanup

## Stability Tiers

- **Stable**: health triad, identity, lifecycle, capabilities, auth, BTSP,
  core visualization (render, validate, export, capabilities, introspect)
- **Evolving**: grammar rendering, dashboard, scene, texture, session management,
  interaction pipeline, motor, audio, sensory negotiation

## Downstream Pairing

| Partner | Integration |
|---------|-------------|
| esotericWebb | Game UI via `visualization.render.scene` + `motor.*` |
| lithoSpore | Validation dashboard via `visualization.render.dashboard` + SSE |
| projectNUCLEUS | sporePrint sovereign serving via `web` mode + content backend |
| wetSpring | Fermentation visualization via `visualization.render.grammar` |
| bingoCube | Crypto commitment grid via `render_color_grid_webgl` WASM |
| footPrint | Composition integration via `/ws` WebSocket JSON-RPC bridge |

## See Also

- [START_HERE.md](../START_HERE.md) — quickstart and configuration
- [CONTEXT.md](../CONTEXT.md) — full architectural context
- [CHANGELOG.md](../CHANGELOG.md) — evolution history
