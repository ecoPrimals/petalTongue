# petalTongue Deep Debt Evolution — westGate Wave 155g

**Date**: Jul 28, 2026 18:30 EDT | **Wave**: 155g | **Gate**: westGate
**From**: westGate petalTongue code team
**Primal**: petalTongue v1.7.0 | **Tests**: 6,605 (0 failed) | **Status**: STABLE

---

## Executive Summary

Following the Wave 155g audit, the petalTongue code team executed a comprehensive
deep debt evolution pass covering 10 dimensions: version drift, formatting, overstep
cleanup, hardcoding elimination, large file refactoring, production stub wiring,
unsafe FFI hardening, dependency analysis, and modern idiomatic Rust evolution.

All P0/P1 items from the audit are resolved. All P2 items reviewed (audio backends,
platform screen metrics remain as expected evolution paths, not debt). Zero
TODO/FIXME/HACK markers remain. Zero debris files. Zero clippy warnings.

**Delta from audit**: 6,558 tests → 6,605 tests (+47 net, including 13 new FFI tests).

---

## What Was Done

### 1. Version Drift (P0) — RESOLVED
- `manifest.toml` and `niche.yaml` synced from 1.6.6 to 1.7.0
- All version declarations now consistent across Cargo.toml, manifest.toml, niche.yaml

### 2. Formatting & Warnings (P1) — RESOLVED
- `cargo fmt` applied workspace-wide — zero diffs remaining
- 4 clippy doc-backtick warnings fixed in `client_hello.rs`
- 3 doc link resolution warnings fixed in `dispatch.rs` and `method_gate.rs`
- Zero clippy warnings on pedantic + nursery

### 3. Overstep Cleanup — Topology Architecture Evolution
This was the largest change. The gate mesh subsystem hardcoded static topology
data (`&'static str` IPs, gate names, VPS coordinates) compiled into the binary.

**What changed**:
- `MeshNode` / `MeshLink` evolved from `&'static str` to owned `String` fields
- `MeshTopologySource` trait returns `Vec<MeshNode>` (owned, not `&'static`)
- New `ManifestMeshTopology` loads topology from `ecosystem_manifest.toml` at runtime
- `offline-topology` feature changed from default-on to default-off
- `StaticMeshTopology` preserved as compile-time fallback behind feature gate
- All 7 consumers updated: IPC handlers, web handlers, viz data, data service

**Files touched**: `gate_mesh/mod.rs`, `gate_mesh/peers.rs`, `gate_mesh.rs` (IPC),
`topology.rs`, `mesh.rs`, `gate_mesh.rs` (viz_data)

### 4. Hardcoding Elimination
| Old | New |
|-----|-----|
| `LOCAL_GATE_ID = "eastGate"` | `PETALTONGUE_GATE_ID` env var |
| `"/var/lib/nestgate/"` paths | `COORD_STORAGE_PATH` env > XDG > generic ecoPrimals > legacy |
| `"songBird drawbridge"` K-Derm | `"mesh.routing"` capability domain |
| `"bearDog TLS"` K-Derm | `"tls.gateway"` capability domain |
| Hardcoded wave `136` | Read from `ecosystem_manifest.toml`, null fallback |
| `skunkBat` name matching | Removed — capability inference only |

### 5. Large File Refactoring
| File | Before | After | Method |
|------|--------|-------|--------|
| `main.rs` | 727 lines | 199 lines | Extracted `cli.rs`, `bootstrap.rs`, `dispatch.rs` |
| `geometry.rs` | 783 lines | Module directory | Strategy pattern with `compiler/geometry/` |

### 6. Production Stub Wiring
- `discover_via_mdns`: wired to `MdnsVisualizationProvider::discover_for_service()`
- `discover_via_config`: reads operator `discovery.toml` for service definitions
- `query_unix_socket`: probes UDS endpoints with JSON-RPC `capabilities.list` (500ms timeout)

### 7. Unsafe FFI Hardening
13 new tests in `petal-tongue-platform/src/ffi.rs`:
- Null pointer safety (config, surface, renderer)
- Valid/invalid JSON config parsing
- Full lifecycle roundtrip (init → render → free)
- String freeing and double-free safety

### 8. Dependency Analysis
- All dependencies pure Rust — zero native C libraries
- `tarpc` transitively pulls `opentelemetry 0.18` (not actionable — upstream dep)
- `base64` version duplicate (`ron` v0.21 vs v0.22) — benign, no security impact
- `nokhwa`/`mozjpeg-sys` previously removed — confirmed absent

### 9. Modern Idiomatic Rust
- Edition 2024 on all crates
- Let-chains for control flow
- Owned types over `&'static str` for runtime data
- Trait-based dispatch over hardcoded name matching
- `#[expect]` with reasons over `#[allow]`

---

## Quality Metrics

| Metric | Result |
|--------|--------|
| `cargo test --workspace` | 6,605 passed, 0 failed, 3 ignored |
| `cargo clippy --pedantic --nursery` | 0 warnings |
| `cargo fmt --check` | Clean |
| `cargo doc --workspace --no-deps` | 0 warnings |
| Production `unwrap()` | 0 (deny enforced) |
| `todo!()`/`FIXME`/`HACK`/`STUB` | 0 |
| Max file size | All under 800 LOC |
| Debris files (.bak/.tmp/.swp/.log) | 0 |
| Stale .env files | 0 |
| Orphan root files | 0 |

---

## Remaining Evolution Paths (P2 — Not Debt)

These are expected evolution paths, not blockers:

| Item | Status | Notes |
|------|--------|-------|
| Audio backends (PipeWire/ALSA) | Stub returns empty | Waiting for hardware integration wave |
| Platform screen metrics (Win/macOS) | Returns `None` | Linux-first; other platforms when needed |
| `cas_source` dead code | `#[allow(dead_code)]` | Awaiting CLI integration for sporePrint |
| `doom-core` crate | Workspace member | Platform stress test — intentional |
| WASM runtime validation | Not tested on westGate | Builds compile; needs browser test |
| genomeBin deployment cycle | Not yet deployed | Awaiting depot infrastructure |
| Cross-gate IPC | Not tested multi-gate | Awaiting multi-gate mesh topology |
| `cargo llvm-cov` | Not measured | Requires `llvm-tools-preview` component |

---

## Upstream Dependencies

| Primal | Dependency | Status |
|--------|-----------|--------|
| songBird | Runtime discovery + TLS relay | Graceful fallback if absent |
| bearDog | BTSP family seed | Graceful fallback to unsigned mode |
| biomeOS | Neural API registration | Logs warning and continues |
| nestGate | Coord storage paths | Discovery order with generic fallback |

---

## For Overwatch Review

1. **Topology architecture change**: `ManifestMeshTopology` is the new primary
   data source. Static topology is behind `offline-topology` feature (default off).
   Verify `ecosystem_manifest.toml` is present at deployment targets.

2. **New env vars**: `PETALTONGUE_GATE_ID`, `COORD_STORAGE_PATH` — document in
   deployment playbooks.

3. **K-Derm capability names**: Changed from primal-specific names to domain
   names. Any upstream code referencing old K-Derm component IDs needs updating.

4. **`main.rs` module extraction**: CLI parsing, bootstrapping, and dispatch
   are now in separate modules. Entry point is 199 lines.

---

*westGate petalTongue evolution pass Wave 155g complete. 6,605 tests, 0 failures,
0 clippy warnings, 0 stale markers, 0 debris. All audit P0/P1 resolved.
Topology architecture evolved to runtime manifest discovery. Ready for cascade
push to golgiBody.*
