# Deep Debt Standards — Updated Wave 156l

**Wave**: 156l | **Date**: August 6, 2026 | **For**: All primal teams (reference)

---

## Standards Achieved

petalTongue now meets all deep debt targets. These standards apply ecosystem-wide.

### Zero bare `unwrap()` in production

- Test code: use `expect("descriptive message")` or `Result`-returning tests with `?`
- Production handlers: propagate with `?`, use `unwrap_or_else`, `map_or`, `let...else`
- Infallible contexts: `#[expect(clippy::unwrap_used, reason = "...")]`

### Zero markers

- No `todo!()`, `unimplemented!()`, `FIXME`, `HACK` in production code
- Use typed errors (`thiserror`) or graceful degradation instead

### Mocks isolated to testing

- All mocks behind `#[cfg(test)]` or `test-fixtures` feature
- Production code uses real implementations with graceful degradation
- Tutorial/demo systems are NOT mocks (they're real features with honest capabilities)

### All files under 800 lines

Smart domain refactoring strategy:
1. Identify logical boundaries (types vs logic vs tests)
2. Create sub-module directory (`foo.rs` → `foo/mod.rs` + `foo/types.rs` + ...)
3. Re-export everything from `mod.rs` — preserve public API
4. Each sub-module owns a single coherent domain

### Named constants (no magic numbers)

```rust
const LATENCY_MS_TO_EDGE_WEIGHT_SCALE: f64 = 100.0;
const NODE_RADIUS_PRIMAL: f32 = 10.0;
```

Name explains purpose, not value. Module-scope or function-scope as appropriate.

### Feature-gated deployment data

Static/fixture data that is deployment-specific (IPs, gate names, topology)
belongs behind a feature flag. Consumers use traits, not statics.

### Self-knowledge only (Wave 156b)

Production code must not name peer primals. A primal knows only:
- Its own name (e.g. `primal_names::PETALTONGUE`)
- The ecosystem orchestration entry point (`primal_names::BIOMEOS` / Neural API socket)

All other primal interactions use **capability discovery** at runtime.
Error messages and API responses use capability-based language:
- "content provider" not "nestGate"
- "mesh routing via capability discovery" not "songBird"
- "security provider" not "bearDog"

Test code may use primal names for fixtures (behind `#[cfg(test)]`).
Offline-topology feature may contain static names (feature-gated, default off).

### Unified configuration resolution

For ecosystem-standard config values (family ID, socket dirs), use the
canonical resolution function rather than raw `std::env::var()` with ad-hoc
defaults. This ensures CLI overrides (OnceLock) take precedence consistently.

---

## Verification Commands

```bash
# Zero unwrap in production (excluding tests)
rg "\.unwrap\(\)" --type rust -g '!*test*' -g '!*_tests*' -l
# Should return empty or only justified #[expect] sites

# Zero markers
rg "todo!\(|unimplemented!\(|FIXME|HACK" --type rust
# Should return empty

# File sizes
find crates/ src/ -name "*.rs" -exec wc -l {} \; | awk '$1 > 800'
# Should return empty

# Clippy clean
cargo clippy --workspace --all-targets -- -D warnings
# Should return zero warnings
```

---

```bash
# Self-knowledge audit
rg '"songBird|"nestGate|"bearDog|"toadStool|"coralReef' --type rust \
  -g '!*test*' -g '!*_tests*' -g '!*nucleus.rs'
# Should return empty (only test fixtures and offline-topology feature)
```

---

### Declarative scene passthrough (Wave 156d)

External primals (tideGlass, footPrint) submit scenes by **name + data** rather
than constructing a full `SceneGraph`. Stored as typed `DeclarativeScene`:
```json
{ "scene": "rges_volcano", "data": {...}, "format": "webgl", "interactive": true }
```

This avoids coupling consumers to petalTongue's internal graph representation.
Query stored scenes via `visualization.scene.declarative` IPC method.

---

### tarpc 0.37 cephalization alignment (Wave 156h)

petalTongue upgraded tarpc 0.34 → 0.37 for G64 convergent evolution. Key facts:

- **No bincode 2.x required** — tarpc 0.37 still uses bincode 1.3 on the wire.
- **tokio-serde 0.8 → 0.9** is the only transitive dependency change.
- **Zero API changes** — `#[tarpc::service]`, `tarpc::context::current()`,
  `tarpc::serde_transport::new()`, `tarpc::client::Config` all unchanged.
- **Wire-compatible** — existing tarpc 0.34 servers can communicate with 0.37
  clients (same bincode 1.3 framing).

Other primals upgrading from tarpc 0.34: bump `tarpc` version and `tokio-serde`
version in workspace `Cargo.toml`. No code changes needed unless using
OpenTelemetry integration (0.37 requires OTel 0.30+).

---

### C2 dual-socket pattern (Wave 156i)

Every primal exposes two UDS sockets for port-agnostic IPC:
- `<primal>.sock` → JSON-RPC (universal, debuggable, browser-compatible)
- `<primal>.tarpc.sock` → tarpc binary bincode (sub-ms, Rust-to-Rust only)

petalTongue is the **first implementation** of this pattern:
- `get_petaltongue_tarpc_socket_path()` resolves the tarpc socket
- `discover_primal_tarpc_socket("songbird")` → `songbird.tarpc.sock`
- Env override: `<PRIMAL>_TARPC_SOCKET` (same pattern as JSON-RPC)
- Server mode spawns both in `tokio::select!` — zero ordering dependency

Other primals adopting C2: create a tarpc server module, implement your
`#[tarpc::service]` trait, bind to `<primal>.tarpc.sock`, add to your
server startup alongside existing JSON-RPC listener.

---

### G65 Protocol Negotiation (Wave 156l — Phase 3)

Single-socket protocol negotiation replaces C2 dual-socket as the Phase 3 destination:
- Client sends `PROTOCOLS: tarpc,jsonrpc\n` as first bytes on connection
- Server selects best match and responds
- No negotiation header = JSON-RPC (backward-compatible with Phase 1 clients)
- Eliminates socket proliferation (30→15 ecosystem-wide)
- Protocol-transparent for songBird routing

**Reference implementation**: squirrel (432 lines, full test coverage).
**Extraction target**: sourDough or cellMembrane (C7 work item).
**Adoption**: All 15 primals + cellMembrane adopt after extraction.

petalTongue will evolve from C2 dual-socket to G65 single-socket once sourDough
publishes the extracted pattern. No action needed until C7 completes.

---

*Wave 156l: G65 spec published. petalTongue C1+C2 DONE. Phase 3 (single-socket
protocol negotiation) pending C7 extraction. Zero clippy pedantic+nursery. 6,615 tests.*
