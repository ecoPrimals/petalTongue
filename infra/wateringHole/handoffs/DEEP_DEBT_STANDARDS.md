# Deep Debt Standards — Updated Wave 156d

**Wave**: 156d | **Date**: August 5, 2026 | **For**: All primal teams (reference)

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

*Wave 156d: petalTongue achieves all deep debt standards including self-knowledge
and declarative scene passthrough. Reference for all primal teams.*
