+++
title = "petalTongue Validation Summary"
description = "Universal user interface primal — 366 workspace tests, 56 IPC methods, 18 crates, 7 modes, pure Rust"
date = 2026-07-15

[taxonomies]
primals = ["petaltongue"]
springs = []
+++

## Status

- **v1.6.6** — workspace edition 2024, `forbid(unsafe_code)`
- **366 workspace tests** passing, 0 failed (unit + integration + doc + property)
- **56 IPC methods** across 10 domain categories (health, identity, auth,
  capabilities, visualization, interaction, audio, UI, motor, BTSP, gate mesh)
- **18 workspace crates** (core, IPC, graph, scene, discovery, adapters,
  entropy, UI, TUI, headless, API, WASM, and domain crates)
- **7 runtime modes**: `server`, `web`, `ui`, `tui`, `live`, `headless`, `status`
- **Zero unsafe code**, zero C dependencies (`deny.toml` bans ring/openssl/aws-lc-sys)
- **BTSP Phase 3** encrypted transport (ChaCha20-Poly1305 + HKDF-SHA256)
- **MethodGate** (JH-0): public/protected method classification with auth enforcement
- **DH-1 /tmp cleanup**: All socket paths through `BIOMEOS_SOCKET_DIR` tier chain
- **Stadial gate**: READY — all 8 glacial criteria clear (Wave 136b)
- **Content pipeline**: `content_render` + `viz_data` + `content_direct` wired and compiled
- **S3 cutover**: 4-tier content backend audited; FAMILY_ID aligned; DISCOVERY_SOCKET wired
- **Wave 136b K-Derm diderm**: `/api/topology-layers` renders 5-layer defense-in-depth. Hardening controls tracked. DNSSEC live.
- **Wave 132d coordination backend**: `/api/coord/*` (6 endpoints) reading nestGate CAS manifest. grapheneGate enrolled. DataService → songBird `mesh.peers` wired.
- **Manifest-driven**: sporePrint and physical_topology handlers evolved from hardcoding to runtime `ecosystem_manifest.toml` parsing. Zero hardcoded IPs/test counts in production.
- **cargo deny**: fully clean (zero advisories). `crossbeam-epoch` 0.9.20. `ttf-parser`/`quick-xml` exemptions documented for transitive egui chain.

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
- **plasmidBin**: `manifest.toml` v1.6.6, `checksums.toml` (BLAKE3), `seed_fingerprint`
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

## See Also

- [START_HERE.md](../START_HERE.md) — quickstart and configuration
- [CONTEXT.md](../CONTEXT.md) — full architectural context
- [CHANGELOG.md](../CHANGELOG.md) — evolution history
