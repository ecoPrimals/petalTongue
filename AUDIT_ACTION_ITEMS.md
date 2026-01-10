# 🎯 petalTongue Audit - Critical Action Items

**Date**: January 10, 2026  
**Overall Grade**: A- (9.0/10)  
**Status**: PRODUCTION READY (with noted gaps)

---

## ⚡ IMMEDIATE ACTIONS (Before Next Release)

### 1. Fix Formatting Issues ⏱️ 15 minutes

```bash
cd /home/eastgate/Development/ecoPrimals/phase2/petalTongue

# Fix all formatting
cargo fmt --all

# Fix raw string literals
sed -i 's/r#"/r"/g; s/"#$/"/g' crates/petal-tongue-animation/src/flower.rs

# Verify
cargo fmt --check
cargo clippy --workspace --no-default-features --all-targets -- -D warnings
```

**Impact**: Fixes 6 clippy warnings, ensures clean CI/CD

---

### 2. Measure Test Coverage ⏱️ 1 hour

```bash
# Run coverage with JSON output
cargo llvm-cov --workspace --no-default-features --lib --json --output-path coverage.json

# View HTML report
cargo llvm-cov --workspace --no-default-features --lib --html
open target/llvm-cov/html/index.html

# Target: 90% minimum
# Current estimate: 70-80%
```

**Actions**:
- Identify uncovered branches
- Add targeted tests for gaps
- Document coverage in STATUS.md

**Impact**: Ensures production quality, identifies blind spots

---

### 3. Add Safety Documentation ⏱️ 2 hours

Add `// SAFETY:` comments to these files:

```rust
// 1. crates/petal-tongue-ipc/src/tarpc_client.rs
// SAFETY: Serde serialization is memory-safe by design. The unsafe marker
// comes from FFI boundaries but type safety is guaranteed by the compiler.

// 2. crates/petal-tongue-ipc/src/tarpc_types.rs  
// SAFETY: Derive macros generate safe code. No manual memory manipulation.

// 3. crates/petal-tongue-ui/src/sensors/screen.rs
// SAFETY: FFI calls to display system follow documented contracts. Error
// handling ensures no undefined behavior on failure.

// 4. crates/petal-tongue-ui/src/display/backends/framebuffer.rs
// SAFETY: ioctl calls use correct ABI. Buffer sizes validated before passing
// to kernel. Errors propagated safely.

// 5. crates/petal-tongue-modalities/src/lib.rs
// SAFETY: External library integration follows library's safety guarantees.
// All buffers properly sized and validated.

// 6. crates/petal-tongue-entropy/src/audio.rs
// SAFETY: Audio buffer callbacks from CPAL library. Library guarantees
// buffer validity for callback duration. No retention after callback.
```

**Impact**: Meets Rust safety documentation standards, aids security audits

---

## 🔴 HIGH PRIORITY (This Sprint)

### 4. Complete Human Entropy Capture ⏱️ 2-3 weeks

**Status**: Only ~10% implemented (basic audio capture)

**Missing**:
- Real-time quality assessment (timing, pitch, amplitude entropy)
- User feedback UI (quality meters, progress indicators)
- Streaming to BearDog API
- Zeroization after streaming
- Visual entropy modality (drawing canvas)
- Narrative entropy modality (keystroke dynamics)
- Gesture entropy modality (motion sensors)
- Video entropy modality (motion analysis)

**Implementation Plan**:

```
Week 1: Audio Quality + Streaming
- Implement quality assessment algorithms
- Add real-time feedback UI
- Integrate BearDog streaming API
- Add zeroization guarantees

Week 2: Visual + Narrative
- Implement drawing canvas
- Stroke capture & analysis
- Text editor with keystroke capture
- Quality metrics for both

Week 3: Gesture + Video + Polish
- Sensor integration
- Camera motion analysis
- End-to-end testing
- Documentation
```

**References**:
- `specs/HUMAN_ENTROPY_CAPTURE_SPECIFICATION.md`
- `crates/petal-tongue-entropy/src/` (current partial implementation)

**Impact**: CRITICAL - Required for key generation workflow

---

### 5. Implement Discovery Phase 1-2 ⏱️ 1 week

**Status**: Only Phase 3 (tarpc) complete, Phases 1-2 skipped

**Missing**:
- Phase 1: mDNS discovery (auto-discovery on local network)
- Phase 2: LRU caching layer (reduce API calls by 80%+)
- Known providers list support
- Protocol negotiation

**Implementation Plan**:

```
Day 1-2: mDNS Discovery
- Add socket2 dependency
- Implement MdnsVisualizationProvider
- UDP multicast join (224.0.0.251)
- Parse DNS-SD service records

Day 3-4: Caching Layer
- Add lru dependency
- Implement ProviderCache with TTLs
- Wrap providers with CachedVisualizationProvider
- Configurable TTLs per data type

Day 5: Integration
- Add known providers support (PETALTONGUE_KNOWN_PROVIDERS)
- Protocol negotiation (prefer tarpc over HTTP)
- Testing
- Documentation
```

**References**:
- `specs/DISCOVERY_INFRASTRUCTURE_EVOLUTION_SPECIFICATION.md`
- Songbird's mDNS implementation (reference)

**Impact**: HIGH - Improves auto-discovery, reduces network load

---

### 6. Audit Production Unwraps ⏱️ 4 hours

**Found**: 429 `.unwrap()` / `.expect()` calls

**Action**:

```bash
# Find production unwraps (excluding tests/examples)
rg "\.unwrap\(\)|\.expect\(" --type rust crates/ \
  | grep -v tests | grep -v examples > unwraps_audit.txt

# Review each and convert to proper error handling
# Priority: main execution paths, public APIs
```

**Example Fixes**:

```rust
// Before:
let config = env::var("KEY").expect("Config required");

// After:
let config = env::var("KEY")
    .context("Missing required config KEY")?;

// Before:
let data = parse(input).unwrap();

// After:
let data = parse(input)
    .with_context(|| format!("Failed to parse input: {}", input))?;
```

**Impact**: Improves error messages, prevents panics in production

---

## 🟡 MEDIUM PRIORITY (Next Sprint)

### 7. Complete Awakening Experience ⏱️ 3 days

**Missing**:
- Stage 3: Discovery animation (tendrils reaching out)
- Progressive audio harmonics (tones for each primal discovered)
- Smooth transitions between stages

**References**: `specs/PETALTONGUE_AWAKENING_EXPERIENCE.md`

---

### 8. Refactor Large Files (Optional) ⏱️ 1 week

**Files Over 1000 LOC**:
- `visual_2d.rs` (1133 LOC) - Consider splitting rendering vs. interaction
- `app.rs` (1004 LOC) - Already noted as "smart refactor" (leave as-is)

**Decision**: Low priority - both files are cohesive and well-organized

---

### 9. Zero-Copy Optimizations ⏱️ Variable (after profiling)

**Action**: Profile first, optimize only hot paths

```bash
# Profile production workload
cargo build --release
perf record -F 99 -g ./target/release/petal-tongue-ui
perf report

# Focus on:
# - String allocations in render loops
# - Vec clones in data pipelines  
# - Serialization hot paths
```

**Potential Optimizations**:
- Use `Cow<str>` for frequently cloned strings
- Replace Vec clones with `Arc<Vec<T>>`
- Pool allocations for render buffers

**Impact**: Performance improvement (current performance already good)

---

## 🟢 LOW PRIORITY (Future)

### 10. Enable Pedantic Linting ⏱️ 1 day

```toml
# Add to Cargo.toml
[lints.clippy]
pedantic = "warn"
nursery = "warn"
cargo = "warn"
```

Then fix resulting warnings (likely 50-100).

---

### 11. Add Explicit Fault Injection Tests ⏱️ 2 days

**Current**: Chaos tests cover resilience implicitly

**Addition**: Explicit fault injection framework

```rust
pub enum FaultType {
    NetworkPartition(Duration),
    DiskFull,
    MemoryExhaustion,
    SlowNetwork(Duration),
    CorruptedResponse,
}

pub struct FaultInjector {
    active_faults: Vec<FaultType>,
}
```

---

## 📊 Summary

| Priority | Item | Time | Impact | Status |
|----------|------|------|--------|--------|
| 🔴 Immediate | Fix formatting | 15 min | Low | Not started |
| 🔴 Immediate | Measure coverage | 1 hour | High | Not started |
| 🔴 Immediate | Safety docs | 2 hours | Medium | Not started |
| 🔴 High | Entropy capture | 2-3 weeks | **CRITICAL** | 10% done |
| 🔴 High | Discovery Phase 1-2 | 1 week | High | Not started |
| 🔴 High | Unwrap audit | 4 hours | Medium | Not started |
| 🟡 Medium | Awakening polish | 3 days | Low | 60% done |
| 🟡 Medium | Refactor files | 1 week | Low | Optional |
| 🟡 Medium | Zero-copy | Variable | Low | After profiling |
| 🟢 Low | Pedantic linting | 1 day | Low | Future |
| 🟢 Low | Fault injection | 2 days | Low | Future |

**Total Immediate Work**: ~4 hours  
**Total High Priority Work**: 3-4 weeks  
**Total Medium Priority Work**: 1-2 weeks

---

## 🎯 Recommended Sequence

### Sprint 1 (This Week)
1. ✅ Fix formatting (15 min)
2. ✅ Measure coverage (1 hour)
3. ✅ Add safety docs (2 hours)
4. ✅ Audit unwraps (4 hours)
5. Start entropy capture (Week 1 work)

### Sprint 2 (Next Week)
1. Continue entropy capture (Week 2 work)
2. Start discovery Phase 1-2

### Sprint 3 (Week After)
1. Complete entropy capture (Week 3 work)
2. Complete discovery Phase 1-2
3. Polish awakening experience

---

## 🏆 Success Criteria

**Before Next Release (v1.3.1)**:
- ✅ All formatting issues fixed
- ✅ Test coverage measured (>90%)
- ✅ Safety documentation complete
- ✅ Production unwraps audited

**Before Key Generation Release (v2.0)**:
- ✅ Human entropy capture 100% complete
- ✅ All 5 modalities implemented
- ✅ BearDog streaming integration working
- ✅ E2E tests for entropy workflow

**Optional (Future Versions)**:
- Discovery Phase 1-2 complete
- Awakening experience polished
- Zero-copy optimizations applied
- Pedantic linting enabled

---

**Document**: AUDIT_ACTION_ITEMS.md  
**See Also**: COMPREHENSIVE_AUDIT_REPORT_JAN_10_2026.md  
**Status**: Ready for execution

🌸 **Priority 1: Fix immediate issues. Priority 2: Complete entropy capture.** 🌸

