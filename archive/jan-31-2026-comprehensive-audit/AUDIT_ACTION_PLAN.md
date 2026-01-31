# 🎯 petalTongue Audit Action Plan

**Date**: January 31, 2026  
**Source**: COMPREHENSIVE_AUDIT_JAN_31_2026.md  
**Status**: Ready for Execution

---

## ✅ Quick Wins (1-2 days)

### 1. License Compliance (1 hour)

**Task**: Add `license = "AGPL-3.0"` to 12 Cargo.toml files

**Files to update**:
- [ ] crates/petal-tongue-core/Cargo.toml
- [ ] crates/petal-tongue-ui/Cargo.toml
- [ ] crates/petal-tongue-graph/Cargo.toml
- [ ] crates/petal-tongue-ipc/Cargo.toml
- [ ] crates/petal-tongue-api/Cargo.toml
- [ ] crates/petal-tongue-animation/Cargo.toml
- [ ] crates/petal-tongue-modalities/Cargo.toml
- [ ] crates/petal-tongue-headless/Cargo.toml
- [ ] crates/petal-tongue-ui-core/Cargo.toml
- [ ] crates/petal-tongue-cli/Cargo.toml
- [ ] crates/petal-tongue-telemetry/Cargo.toml
- [ ] sandbox/mock-biomeos/Cargo.toml

**Command**:
```bash
# Add after [package] section in each Cargo.toml
license = "AGPL-3.0"
```

**Validation**:
```bash
grep -r "^license = " crates/*/Cargo.toml | wc -l
# Should return 19 (all crates)
```

---

### 2. Code Formatting (5 minutes)

**Task**: Run rustfmt on all code

**Command**:
```bash
cargo fmt --all
```

**Validation**:
```bash
cargo fmt --check
# Should return: exit code 0 (success)
```

---

### 3. Fix Clippy Warnings (2-3 hours)

**Task**: Address 25+ clippy warnings

**Step 1**: Auto-fix what's possible
```bash
cargo clippy --fix --allow-dirty --allow-staged --all-targets --all-features
```

**Step 2**: Manual fixes for:
- [ ] Unused imports (6 instances)
- [ ] Deprecated field usage (2 instances)
- [ ] Manual range_contains (1 instance)
- [ ] Upper case acronyms (2 instances)
- [ ] Collapsible else-if (1 instance)

**Step 3**: Document acceptable warnings
- [ ] Dead code in test fixtures (8 instances) - ACCEPTABLE
- [ ] Missing documentation (9 instances) - FIX OR DOCUMENT

**Validation**:
```bash
cargo clippy --all-targets --all-features -- -D warnings
# Should return: exit code 0 (no warnings)
```

---

## 🔧 Critical Fixes (3-5 days)

### 4. Fix Test Compilation (1-2 days)

**Task**: Make all tests compile and pass

**Current Errors**:
1. Deprecated `trust_level` field usage
2. Dead code warnings in test fixtures
3. Missing documentation warnings

**Step 1**: Fix deprecated field warnings
```bash
# Find and replace all instances of:
primals[0].trust_level
# With:
primals[0].properties.get("trust_level")
```

**Step 2**: Allow dead code in test fixtures
```rust
#[allow(dead_code)]
struct TestFixture { ... }
```

**Step 3**: Add missing documentation
```rust
/// Test fixture for ...
#[allow(dead_code)]
struct TestFixture { ... }
```

**Validation**:
```bash
cargo test --workspace --all-features
# Should return: all tests pass
```

---

### 5. Measure Test Coverage (1 day)

**Task**: Run llvm-cov and generate coverage report

**Prerequisites**: Tests must compile (Task #4)

**Commands**:
```bash
# Install llvm-cov if needed
cargo install cargo-llvm-cov

# Run coverage analysis
cargo llvm-cov --workspace --all-features --html

# View report
open target/llvm-cov/html/index.html
```

**Analysis**:
1. Identify uncovered code paths
2. Document coverage percentage per crate
3. Create test plan for uncovered code
4. Document acceptable gaps

**Target**: 90% coverage

**Deliverable**: Coverage report with action plan

---

### 6. Add SAFETY Comments (2-3 hours)

**Task**: Document all 66 unsafe blocks

**Format**:
```rust
// SAFETY: <explanation of why this is safe>
unsafe {
    // ... code ...
}
```

**Categories**:
1. Environment variable manipulation (tests) - 42 blocks
2. Unix socket operations - 10 blocks
3. System calls (ioctl) - 11 blocks
4. Other - 3 blocks

**Template Comments**:
```rust
// SAFETY: Test-only code manipulating environment variables.
// No concurrent access to env vars in test execution.
unsafe { ... }

// SAFETY: Unix socket operations require raw file descriptors.
// Descriptor is valid and properly owned by this process.
unsafe { ... }

// SAFETY: ioctl syscall for framebuffer queries.
// File descriptor is valid, buffer is properly sized.
unsafe { ... }
```

**Validation**:
```bash
# Check all unsafe blocks have SAFETY comments
rg 'unsafe \{' -A 1 | grep -v 'SAFETY' | wc -l
# Should return: 0
```

---

## 🔨 High Priority (1-2 weeks)

### 7. Refactor scenario.rs (2-3 days)

**Task**: Split scenario.rs (1,081 lines) into multiple modules

**Current Structure**:
```
crates/petal-tongue-ui/src/scenario.rs (1,081 lines)
```

**Proposed Structure**:
```
crates/petal-tongue-ui/src/scenario/
  mod.rs (exports)
  loader.rs (scenario loading logic, ~300 lines)
  builder.rs (scenario construction, ~250 lines)
  provider.rs (scenario data provider, ~250 lines)
  types.rs (data structures, ~200 lines)
  tests.rs (unit tests, ~150 lines)
```

**Steps**:
1. Create `scenario/` directory
2. Extract data structures to `types.rs`
3. Extract loading logic to `loader.rs`
4. Extract builder logic to `builder.rs`
5. Extract provider logic to `provider.rs`
6. Move tests to `tests.rs`
7. Create `mod.rs` with public exports
8. Update imports in `lib.rs`

**Validation**:
```bash
cargo build --release
cargo test -p petal-tongue-ui
```

---

### 8. Improve Error Handling (3-5 days)

**Task**: Replace unwrap/expect with proper error handling

**Targets**:
- [ ] petal-tongue-ipc: 39 .unwrap() calls
- [ ] petal-tongue-core: 17 .expect() calls
- [ ] Graph lock operations: 15+ instances

**Strategy**:
1. **Graph lock poisoning** - Handle gracefully:
   ```rust
   // Before:
   let graph = graph.write().expect("graph lock poisoned");
   
   // After:
   let graph = graph.write().map_err(|e| {
       tracing::error!("Graph lock poisoned: {}", e);
       GraphError::LockPoisoned
   })?;
   ```

2. **Property access** - Use ? operator:
   ```rust
   // Before:
   let value = properties.get("trust_level").unwrap();
   
   // After:
   let value = properties.get("trust_level")
       .ok_or(PropertyError::MissingKey("trust_level"))?;
   ```

3. **IPC operations** - Proper error propagation:
   ```rust
   // Before:
   let response = response.result.unwrap();
   
   // After:
   let response = response.result
       .ok_or(IpcError::NoResult)?;
   ```

**Validation**:
```bash
# Count remaining unwrap/expect in production code (excluding tests)
rg '\.unwrap\(\)|\.expect\(' --type rust crates/*/src/ | wc -l
# Target: <10 instances (all documented)
```

---

### 9. Semantic Naming Audit (1 day)

**Task**: Ensure all method names follow `domain.operation` format

**Scope**: All IPC/RPC methods

**Files to Audit**:
- crates/petal-tongue-ipc/src/json_rpc.rs
- crates/petal-tongue-ipc/src/tarpc_types.rs
- crates/petal-tongue-ipc/src/unix_socket_server.rs
- crates/petal-tongue-api/src/biomeos_jsonrpc_client.rs

**Check Format**:
```rust
// ✅ Good:
"discovery.query"
"neural_api.get_proprioception"
"ipc.register"
"display.commit_frame"

// ❌ Bad:
"queryDiscovery"
"getProprioception"
"register_with_songbird"
```

**Create Document**: `SEMANTIC_NAMING_AUDIT.md`

---

## 📋 Medium Priority (2-4 weeks)

### 10. Zero-Copy Optimizations (1 week)

**Task**: Reduce cloning in hot paths

**Targets**:
1. Graph validation operations (50+ clone calls)
2. Session state management
3. Node/edge operations

**Techniques**:
- Use `&str` instead of `String.clone()`
- Implement `Copy` for small types
- Use `Arc<str>` for shared strings
- Use slices instead of vector clones

**Measurement**:
```bash
# Before optimization
hyperfine 'cargo run --release -- status'

# After optimization  
hyperfine 'cargo run --release -- status'

# Compare results
```

---

### 11. Complete TODO Items (Ongoing)

**Task**: Address 171 TODO comments

**Priority TODOs** (12 critical):
1. ToadStool integration (8 items) - BLOCKED (waiting on handoff)
2. Rendering implementations (3 items)
3. mDNS discovery (1 item)

**Medium Priority** (85 items):
- Audio features (15)
- Display backends (10)
- Graph rendering (8)
- UI components (20)
- Neural API integration (12)
- Discovery mechanisms (10)
- Panel system features (10)

**Low Priority** (74 items):
- Documentation notes
- Architecture notes
- Implementation notes

**Strategy**:
1. Convert TODOs to GitHub issues
2. Prioritize by user impact
3. Schedule in sprints
4. Remove obsolete TODOs

---

### 12. Remove Commented Code (1 day)

**Task**: Clean up 41+ instances of commented code

**Strategy**:
1. Review each commented block
2. Remove if obsolete
3. Convert to TODO if planned
4. Uncomment if needed

**Files with Commented Code**:
- crates/petal-tongue-ui/src/lib.rs (4 blocks)
- crates/petal-tongue-primitives/src/lib.rs (6 blocks)
- crates/petal-tongue-entropy/src/lib.rs (4 blocks)
- crates/petal-tongue-ui/src/backend/toadstool.rs (1 block)
- Multiple test files (examples, acceptable)

**Validation**:
```bash
# Find remaining commented code (excluding examples/notes)
rg '^\s*//\s*(pub\s+)?mod\s+\w+;' crates/*/src/ | wc -l
# Target: 0
```

---

## 📈 Success Metrics

### P0 Success (Critical)
- [ ] All 19 crates have AGPL-3.0 license
- [ ] All tests compile and pass
- [ ] cargo fmt --check passes
- [ ] cargo clippy passes (0 warnings)

### P1 Success (High Priority)
- [ ] Test coverage ≥ 90%
- [ ] scenario.rs refactored (<1000 lines per file)
- [ ] All unsafe blocks have SAFETY comments
- [ ] Semantic naming audit complete

### P2 Success (Medium Priority)
- [ ] <10 unwrap/expect in production code
- [ ] Commented code removed
- [ ] Critical TODOs addressed
- [ ] Zero-copy optimizations implemented

---

## 📅 Timeline

### Week 1: Critical Fixes
- Day 1: License compliance + formatting + clippy auto-fixes
- Day 2-3: Fix test compilation
- Day 4: Measure test coverage
- Day 5: Add SAFETY comments

### Week 2: High Priority
- Day 1-2: Refactor scenario.rs
- Day 3-4: Improve error handling (phase 1)
- Day 5: Semantic naming audit

### Week 3-4: Medium Priority
- Week 3: Complete error handling improvements
- Week 4: Zero-copy optimizations + TODO cleanup

---

## 🎯 Immediate Next Steps

**RIGHT NOW**:
1. Run `cargo fmt --all`
2. Add license to all Cargo.toml files
3. Run `cargo clippy --fix --allow-dirty`
4. Commit changes: "chore: fix formatting, license, clippy warnings"

**THIS WEEK**:
5. Fix test compilation
6. Measure test coverage
7. Add SAFETY comments

**NEXT WEEK**:
8. Refactor scenario.rs
9. Semantic naming audit

---

*Action plan ready for execution. Start with Quick Wins!*
