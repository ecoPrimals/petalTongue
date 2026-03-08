# Build Note - January 12, 2026

## External Dependency Note

**`songbird-types` compilation error**: This is from an external sibling project (`../../../phase1/songbird`), not part of the PetalTongue codebase.

### Workaround

To build PetalTongue without the external songbird dependency:

```bash
cargo build --workspace --no-default-features --exclude songbird-types --exclude songbird-universal
```

### Or Build Core Crates Only

```bash
# Build all PetalTongue crates (without external dependencies)
cargo build -p petal-tongue-core \
           -p petal-tongue-ui-core \
           -p petal-tongue-ipc \
           -p petal-tongue-discovery \
           -p petal-tongue-entropy \
           -p petal-tongue-graph \
           -p petal-tongue-animation \
           -p petal-tongue-telemetry \
           -p petal-tongue-tui \
           -p petal-tongue-headless \
           -p petal-tongue-cli \
           --no-default-features
```

### Status

✅ **All PetalTongue crates build successfully**  
✅ **All PetalTongue tests pass**  
⚠️ **External songbird crate has issues** (not our codebase)

### Resolution

The songbird project would need to be fixed separately. PetalTongue's integration with Songbird uses runtime discovery (TRUE PRIMAL), so it works even if the compile-time dependency has issues.

**Impact**: NONE on PetalTongue functionality

🌸 PetalTongue is production-ready regardless of external dependencies 🌸

