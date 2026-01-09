# Zero-Copy Optimization Plan
## Deep Debt Solution - Allocation Reduction

**Date**: January 8, 2026  
**Philosophy**: Smart optimization based on profiling, not blind micro-optimization  
**Target**: Reduce allocations in hot paths by 50%+

---

## Current State Analysis

### Allocation Hotspots (petal-tongue-core)
```
File                           .to_string()  .clone()  Priority
─────────────────────────────────────────────────────────────────
graph_engine.rs                26            8         HIGH ⭐⭐⭐
awakening.rs                   23            -         MEDIUM ⭐⭐
awakening_coordinator.rs       14            -         MEDIUM ⭐⭐
toadstool_compute.rs           14            2         MEDIUM ⭐⭐
primal_types.rs                9             -         LOW ⭐
instance.rs                    8             5         MEDIUM ⭐⭐
types.rs                       6             1         HIGH ⭐⭐⭐
types_tests.rs                 48            1         SKIP (tests)
error_tests.rs                 18            -         SKIP (tests)
```

### Total Allocations
- **Production**: ~205 `.to_string()`, ~39 `.clone()` (petal-tongue-core only)
- **Tests**: ~66 `.to_string()`, ~1 `.clone()` (acceptable)

---

## Optimization Strategy

### Phase 1: Hot Path Optimizations (THIS SESSION) ⭐
**Target Files**: graph_engine.rs, types.rs  
**Expected Reduction**: 30-40 allocations  
**Principle**: Eliminate unnecessary clones, use borrowing

#### 1.1 graph_engine.rs Optimizations
```rust
// BEFORE (clone on every add_node):
pub fn add_node(&mut self, info: PrimalInfo) {
    let node_id = info.id.clone();  // ← CLONE
    let node = Node::new(info);
    self.node_index.insert(node_id, self.nodes.len());
    self.nodes.push(node);
}

// AFTER (borrow from node):
pub fn add_node(&mut self, info: PrimalInfo) {
    let node = Node::new(info);
    let node_id = node.info.id.clone();  // Only one clone
    self.node_index.insert(node_id, self.nodes.len());
    self.nodes.push(node);
}
```

#### 1.2 HashMap Lookups (use &str, not String)
```rust
// BEFORE:
self.node_index.contains_key(&edge.from)  // No issue
self.node_index.contains_key(&edge.to)    // No issue

// ALREADY OPTIMAL: HashMap<String, _> accepts &str for lookups
```

#### 1.3 hierarchical_layout Optimizations
```rust
// BEFORE (many clones in BFS):
let roots: Vec<String> = nodes.iter()
    .filter(|node| !incoming_counts.contains_key(&node.info.id))
    .map(|node| node.info.id.clone())  // ← CLONE
    .collect();

for root in &roots {
    levels.insert(root.clone(), 0);  // ← CLONE
}

// AFTER (use indices, not IDs):
let root_indices: Vec<usize> = nodes.iter().enumerate()
    .filter(|(_, node)| !incoming_counts.contains_key(&node.info.id))
    .map(|(idx, _)| idx)
    .collect();

// Use usize indices in HashMap<usize, usize> instead of HashMap<String, usize>
```

#### 1.4 Test String Literals
```rust
// BEFORE:
graph.add_node(create_test_primal("1", "Node 1"));
// "1".to_string() inside test helper

// AFTER:
const TEST_ID_1: &str = "1";
const TEST_NAME_1: &str = "Node 1";
graph.add_node(create_test_primal(TEST_ID_1, TEST_NAME_1));
// Only converts to String at construction, not at call site
```

### Phase 2: Structural Improvements (FUTURE)
**Target**: Arc<str> for shared IDs  
**Expected Reduction**: 70-80% of remaining clones  
**Principle**: Share immutable data

```rust
// Future: Change PrimalInfo to use Arc<str> for IDs
pub struct PrimalInfo {
    pub id: Arc<str>,          // ← Zero-copy clone
    pub name: Arc<str>,        // ← Zero-copy clone
    pub primal_type: Arc<str>, // ← Zero-copy clone
    pub endpoint: String,      // Keep String (often constructed)
    pub capabilities: Vec<Arc<str>>, // ← Zero-copy clone
    // ...
}
```

### Phase 3: Advanced Optimizations (FUTURE)
**Target**: Zero-copy deserialization  
**Principle**: Borrow from input buffers

- Use `serde_json` with borrowing where possible
- Implement custom deserializers for high-frequency types
- Use `Cow<'a, str>` for conditional ownership

---

## Implementation Plan

### Step 1: graph_engine.rs (HIGH PRIORITY)
- [ ] Restructure `add_node` to avoid ID clone
- [ ] Change `hierarchical_layout` to use indices instead of ID clones
- [ ] Verify tests still pass

### Step 2: types.rs (HIGH PRIORITY)
- [ ] Review `PrimalInfo::new()` for unnecessary conversions
- [ ] Add `#[inline]` hints for hot constructors
- [ ] Document clone rationale where unavoidable

### Step 3: Measure Impact
- [ ] Count allocations before/after
- [ ] Run benchmarks if available
- [ ] Document wins in CHANGELOG

---

## Deep Debt Principles Applied

### ✅ Complete Solutions Over Patches
- Not just adding `#[inline]` everywhere
- Restructuring algorithms to eliminate clones
- Using indices instead of repeated ID lookups

### ✅ Modern Idiomatic Rust
- Leveraging borrowing (`&str`) where possible
- Using `Arc<str>` for shared immutable data (future)
- `Cow<str>` for conditional ownership (future)

### ✅ Fast AND Safe Rust
- Zero unsafe code
- All optimizations maintain correctness
- Borrowing > Cloning > Unsafe

### ✅ Measurable Impact
- Before/after allocation counts
- Benchmark comparisons
- Production profiling

---

## Expected Impact

### Phase 1 (This Session)
- **Allocations Reduced**: 30-40 (15-20%)
- **Files Changed**: 2-3
- **Risk**: LOW (local changes only)
- **Tests**: All passing

### Future Phases (Arc<str> refactor)
- **Allocations Reduced**: 150+ (70%+)
- **Files Changed**: 20+
- **Risk**: MEDIUM (API changes)
- **Timeline**: 1-2 sprints

---

## Success Metrics

### Immediate (This Session)
```
BEFORE: 205 .to_string() + 39 .clone() = 244 allocations
AFTER:  175 .to_string() + 30 .clone() = 205 allocations
REDUCTION: 39 allocations (16%)
```

### Ultimate Goal (Future)
```
TARGET: < 100 hot-path allocations
CURRENT: 244
GAP: 144 allocations to eliminate
```

---

**Status**: Phase 1 in progress  
**Next**: Implement graph_engine.rs optimizations

