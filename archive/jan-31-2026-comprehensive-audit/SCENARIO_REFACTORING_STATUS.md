# 🎯 Scenario Refactoring - Execution Status

**Date**: January 31, 2026  
**Status**: ✅ **FOUNDATION COMPLETE** - 4/8 modules created, ready for completion

---

## ✅ Completed Modules (474 lines)

### 1. `types.rs` (112 lines) ✅
- Core Scenario struct
- NeuralApiConfig
- ScenarioProprioception, SelfAwareness, Motor, Sensory
- All core type definitions
- **Well under 1000 line limit** ✅

### 2. `config.rs` (207 lines) ✅  
- UiConfig with validation
- PanelVisibility, FeatureFlags
- CustomPanelConfig with validation
- AnimationConfig, PerformanceConfig
- **Well under 1000 line limit** ✅

### 3. `ecosystem.rs` (61 lines) ✅
- Ecosystem struct
- PrimalDefinition
- Position
- PrimalMetrics
- **Well under 1000 line limit** ✅

### 4. `sensory.rs` (94 lines) ✅
- SensoryConfig with validation
- CapabilityRequirements
- Validation logic
- **Well under 1000 line limit** ✅

---

## 📋 Remaining Work (4 modules)

### 5. `loader.rs` (~200 lines needed)
**Extract from**: scenario.rs lines 427-630
**Contents**:
- Scenario::load() method
- Scenario::validate() method  
- File I/O logic
- JSON parsing with error context

### 6. `convert.rs` (~180 lines needed)
**Extract from**: scenario.rs lines 631-810
**Contents**:
- to_primal_infos() method
- PrimalDefinition → PrimalInfo conversion
- extract_metrics() logic
- Position mapping

### 7. `mod.rs` (~150 lines needed)
**Create new**:
- Module documentation
- Re-exports for convenience
- Public API surface
- Integration with parent module

### 8. `fixtures.rs` (~150 lines needed)
**Extract from**: scenario.rs lines 811-1081  
**Contents** (#[cfg(test)]):
- Mock scenario generators
- Test helpers
- Fixture creation

---

## 🎯 Completion Steps (2-3 hours)

### Step 1: Extract Loader Logic (45 min)

```bash
# 1. Extract lines 427-630 from scenario.rs
# 2. Create loader.rs with proper imports
# 3. Move Scenario::load() and validate() to loader module
# 4. Update imports
```

**Key Code**:
```rust
// loader.rs
pub struct ScenarioLoader;

impl ScenarioLoader {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Scenario> {
        // ... implementation from lines 427-451 ...
    }
}

impl Scenario {
    // Keep validate() in loader for cohesion
    pub fn validate(&self) -> Result<()> {
        // ... implementation from lines 460-630 ...
    }
}
```

### Step 2: Extract Conversion Logic (45 min)

```bash
# 1. Extract lines 631-810 from scenario.rs
# 2. Create convert.rs with proper imports
# 3. Move conversion methods
# 4. Update imports
```

**Key Code**:
```rust
// convert.rs
impl Scenario {
    pub fn to_primal_infos(&self) -> Vec<PrimalInfo> {
        // ... implementation ...
    }
}

impl PrimalDefinition {
    pub fn to_primal_info(&self) -> PrimalInfo {
        // ... implementation ...
    }
}
```

### Step 3: Create Module Root (30 min)

```rust
// mod.rs
//! Scenario loading system for benchTop demonstrations
//!
//! ## Architecture
//!
//! - `types`: Core Scenario struct and main types
//! - `config`: UI configuration (panels, animations, performance)
//! - `ecosystem`: Primal definitions and metrics
//! - `sensory`: Sensory capability configuration
//! - `loader`: Loading and validation logic
//! - `convert`: Type conversions
//! - `fixtures`: Test fixtures (test-only)

pub mod types;
pub mod config;
pub mod ecosystem;
pub mod sensory;
pub mod loader;
pub mod convert;

#[cfg(test)]
pub mod fixtures;

// Re-export main types
pub use types::Scenario;
pub use config::{UiConfig, PanelVisibility, FeatureFlags, AnimationConfig, PerformanceConfig};
pub use ecosystem::{Ecosystem, PrimalDefinition, Position, PrimalMetrics};
pub use sensory::{SensoryConfig, CapabilityRequirements};
```

### Step 4: Extract Test Fixtures (30 min)

```bash
# 1. Extract lines 811-1081 from scenario.rs
# 2. Create fixtures.rs with #[cfg(test)]
# 3. Move test helper functions
# 4. Update test imports
```

### Step 5: Update Parent Module (15 min)

```rust
// lib.rs
pub mod scenario;  // Changed from single file to module

// Usage remains the same:
use crate::scenario::Scenario;  // Still works!
```

### Step 6: Validation (15 min)

```bash
cargo build --all-features
cargo test --package petal-tongue-ui
cargo fmt --all
```

---

## 📊 Progress Metrics

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| **Modules Created** | 8 | 4 | 50% ✅ |
| **Lines Extracted** | ~1560 | 474 | 30% ✅ |
| **Max Module Size** | <1000 | 207 | ✅ PASS |
| **Avg Module Size** | ~195 | 118 | ✅ EXCELLENT |
| **Compilation** | PASS | - | ⏳ Pending |

---

## ✅ Benefits Already Achieved

1. **Type Safety** - Clean module boundaries
2. **Maintainability** - Single responsibility per file
3. **Readability** - Clear organization (4 modules done)
4. **Size Compliance** - All modules well under 1000 lines

---

## 🎯 Next Session Actions

1. **Complete Extraction** (2 hours)
   - Extract loader.rs
   - Extract convert.rs
   - Extract fixtures.rs
   - Create mod.rs

2. **Validation** (30 min)
   - Build and test
   - Fix any import issues
   - Run cargo fmt

3. **Cleanup** (15 min)
   - Remove old scenario.rs
   - Update lib.rs
   - Commit changes

---

## 🔧 Commands for Completion

```bash
cd /path/to/petalTongue

# Step 1-4: Create remaining modules (follow steps above)

# Step 5: Rename old file for backup
mv crates/petal-tongue-ui/src/scenario.rs crates/petal-tongue-ui/src/scenario.rs.old

# Step 6: Build and test
cargo build --package petal-tongue-ui
cargo test --package petal-tongue-ui

# Step 7: If successful, remove backup
rm crates/petal-tongue-ui/src/scenario.rs.old

# Step 8: Format
cargo fmt --all

# Step 9: Commit
git add crates/petal-tongue-ui/src/scenario/
git commit -m "refactor(ui): split scenario.rs into clean modules

- Split 1,081-line file into 8 focused modules
- Each module <350 lines (well under 1000 limit)
- Improved maintainability and testability
- Preserved all functionality
- Modern idiomatic Rust organization"
```

---

## 📈 Expected Final Result

```
crates/petal-tongue-ui/src/scenario/
├── mod.rs              (150 lines) - Module root
├── types.rs            (112 lines) ✅ COMPLETE
├── config.rs           (207 lines) ✅ COMPLETE
├── ecosystem.rs        ( 61 lines) ✅ COMPLETE
├── sensory.rs          ( 94 lines) ✅ COMPLETE
├── loader.rs           (200 lines) - TODO
├── convert.rs          (180 lines) - TODO
└── fixtures.rs         (150 lines) - TODO

Total: ~1,154 lines across 8 files
Average: ~144 lines/file
Max: 207 lines/file (config.rs)

✅ All files under 1000 line limit
✅ Clean module boundaries
✅ Single responsibility per module
```

---

## 🎉 Achievements So Far

- ✅ 50% of refactoring complete
- ✅ All created modules well under limit
- ✅ Clean, idiomatic Rust structure
- ✅ Foundation for easy completion
- ✅ Modern module organization

---

**Status**: ✅ **FOUNDATION LAID** - Ready for completion in next session

**Recommendation**: Complete remaining 4 modules following the detailed steps above

---

*Smart refactoring in progress - halfway complete with excellent structure*
