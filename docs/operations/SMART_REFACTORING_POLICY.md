# Smart Refactoring Policy - Quality Over Arbitrary Rules

**Date**: January 12, 2026 (examples updated March 16, 2026)  
**Principle**: "Smart refactor rather than just split"  
**Status**: Active Policy

> **Note**: The examples below reference file sizes from January 2026. Many of
> these files have since been smart-refactored into module directories (e.g.
> `visual_2d.rs` → `visual_2d/`, `trust_dashboard.rs` → `trust_dashboard/`).
> The policy principles remain unchanged.

---

## 🎯 Philosophy

> **"A cohesive 1,200-line module is better than 12 fragmented 100-line files."**

We have a **guideline** of 1000 lines per file, but it's not a hard rule. Code quality, cohesion, and performance matter more than arbitrary line counts.

---

## ✅ When Large Files Are GOOD

### Characteristics of Smart Large Files:

1. **High Cohesion**: Single, well-defined responsibility
2. **Single Type**: Usually one main struct with impl block
3. **Logical Flow**: Clear internal organization (setup → process → cleanup)
4. **No Duplication**: High information density
5. **Performance Benefits**: Related code together improves cache locality
6. **Difficult to Split**: Methods are tightly coupled

### Examples in Our Codebase:

#### `visual_2d.rs` (1,133 lines) ✅ JUSTIFIED

```rust
//! 2D Visual Renderer
//!
//! Single responsibility: Render graph topology as 2D graphics
//! Single type: Visual2DRenderer
//! Sections:
//!   - Setup & configuration (100 lines)
//!   - Main rendering loop (300 lines)
//!   - Input handling (200 lines)
//!   - Layout algorithms (300 lines)
//!   - Utilities (233 lines)
```

**Why This is Good**:
- ✅ Tight coupling between methods (rendering depends on layout)
- ✅ Performance: Hot path stays in CPU cache
- ✅ Readability: Entire rendering pipeline in one place
- ✅ No arbitrary boundaries

**What We Extracted**: Truly independent utilities
- `color_utils.rs` - Pure color space conversions (reusable)

#### `app.rs` (1,009 lines) ✅ JUSTIFIED

```rust
//! Main Application Logic
//!
//! Single responsibility: Application state and lifecycle
//! Single type: PetalTongueApp
//! Sections:
//!   - State management (200 lines)
//!   - Panel coordination (300 lines)
//!   - Event handling (200 lines)
//!   - Rendering dispatch (309 lines)
```

**Why This is Good**:
- ✅ Application root - naturally centralized
- ✅ All panels coordinate through single state
- ✅ Event flow is linear and clear
- ✅ Splitting would create circular dependencies

---

## ❌ When Large Files Are BAD

### Red Flags (Time to Refactor):

1. **Multiple Responsibilities**: File does unrelated things
2. **Multiple Main Types**: 3+ independent structs
3. **Copy-Paste Code**: Duplication indicates missing abstraction
4. **Unclear Organization**: No logical flow or sections
5. **Easy to Split**: Independent subsystems with clear boundaries
6. **Performance Irrelevant**: No hot path that benefits from locality

### Example of BAD Large File:

```rust
//! utils.rs (1,500 lines) ❌ NEEDS REFACTORING
//!
//! - String utilities (300 lines)
//! - Date/time helpers (200 lines)
//! - Network functions (400 lines)
//! - File I/O (300 lines)
//! - Math functions (300 lines)
```

**Why This is Bad**:
- ❌ Multiple unrelated responsibilities
- ❌ No cohesion (string utils don't need network utils)
- ❌ Easy to split into independent modules
- ❌ No performance benefit from being together

**Solution**: Extract each section to its own module
- `string_utils.rs`
- `time_utils.rs`
- `network.rs`
- `file_io.rs`
- `math.rs`

---

## 🔍 Decision Matrix

When you encounter a large file, ask:

| Question | Yes = Keep Together | No = Consider Splitting |
|----------|---------------------|-------------------------|
| Does it have a single, clear responsibility? | ✅ | ❌ |
| Is it mostly one type with impl blocks? | ✅ | ❌ |
| Are the methods tightly coupled? | ✅ | ❌ |
| Is there a clear internal flow/organization? | ✅ | ❌ |
| Would splitting harm performance? | ✅ | ❌ |
| Is it difficult to identify clean split points? | ✅ | ❌ |

**If 4+ answers are "Keep Together"**: Leave it as-is with documentation  
**If 3 or fewer**: Consider refactoring

---

## 🛠️ Smart Refactoring Process

When refactoring IS warranted:

### Step 1: Identify Truly Independent Subsystems

```rust
// BEFORE: Large cohesive file with small independent utility
//! visual_2d.rs (1,133 lines)
impl Visual2DRenderer {
    // ... rendering code ...
}

fn hsv_to_rgb(h, s, v) -> (u8, u8, u8) {
    // Pure function, no dependencies
}
```

```rust
// AFTER: Extract independent utility, keep renderer cohesive
//! visual_2d.rs (1,100 lines) - main rendering
use crate::color_utils::hsv_to_rgb;

impl Visual2DRenderer {
    // ... rendering code ...
}

//! color_utils.rs (200 lines) - reusable utilities
pub fn hsv_to_rgb(h, s, v) -> (u8, u8, u8) { ... }
pub fn rgb_to_hsv(r, g, b) -> (f32, f32, f32) { ... }
pub fn lerp_hsv(...) -> ... { ... }
```

### Step 2: Create Modules, Not Just Files

**Bad Refactor**: Just split by line count
```
renderer/
  part1.rs  // Lines 1-500
  part2.rs  // Lines 501-1000
  part3.rs  // Lines 1001-1133
```

**Good Refactor**: Split by responsibility
```
renderer/
  mod.rs           // Public API
  core.rs          // Core rendering logic
  layout.rs        // Layout algorithms (if independent)
  input.rs         // Input handling (if independent)
```

### Step 3: Measure Impact

Before:
- ✅ Readability score
- ✅ Build time
- ✅ Test coverage
- ✅ Performance benchmarks

After:
- ❓ Did readability improve?
- ❓ Did build time stay same/improve?
- ❓ Did test coverage stay same?
- ❓ Did performance stay same/improve?

**If any metric got worse**: Reconsider the refactoring.

---

## 📊 Our Current Large Files

| File | Lines | Status | Justification |
|------|-------|--------|---------------|
| `visual_2d.rs` | 1,133 | ✅ Keep | Single cohesive renderer |
| `app.rs` | 1,009 | ✅ Keep | Application root |
| *All others* | <1,000 | ✅ Good | Within guidelines |

**Total Files Over 1000**: 2 out of 218 (0.9%)  
**Total LOC**: ~64,000  
**Average File Size**: ~294 lines

**Assessment**: **EXCELLENT** - Minimal violations, both justified

---

## 🎯 Examples of Good Refactoring We've Done

### Example 1: Color Utilities ✅

**Extracted**: Pure utility functions from `visual_2d.rs`
```rust
// Independent, reusable, testable
pub fn hsv_to_rgb(...) -> ...
pub fn rgb_to_hsv(...) -> ...
pub fn lerp_hsv(...) -> ...
```

**Result**:
- ✅ `visual_2d.rs` stays cohesive (1,100 lines)
- ✅ Color utils reusable across codebase
- ✅ Better testability (unit tests for color functions)
- ✅ No performance impact

### Example 2: Audio as Extension ✅

**Refactored**: Audio from hard dependency → optional extension
```rust
// Tier 1: Self-stable (AudioCanvas)
// Tier 2: Optional (feature-gated rodio)
// Tier 3: Network (ToadStool primal)
```

**Result**:
- ✅ No ALSA dependency in default build
- ✅ Runtime capability discovery
- ✅ Graceful degradation
- ✅ TRUE PRIMAL sovereignty

---

## 🚫 Anti-Patterns to Avoid

### Anti-Pattern 1: Arbitrary Splitting

```rust
// BAD: Split just to hit line count
renderer/
  methods_a_to_m.rs
  methods_n_to_z.rs
```

**Why Bad**: Violates cohesion for no benefit

### Anti-Pattern 2: God Objects

```rust
// BAD: Everything in one file
app.rs:
  - HTTP client
  - Database layer
  - UI rendering
  - Business logic
  - File I/O
  - ... (5,000 lines)
```

**Why Bad**: Multiple responsibilities, high coupling

### Anti-Pattern 3: Micro-Modules

```rust
// BAD: Excessive fragmentation
utils/
  add.rs              // 3 lines: pub fn add(a, b) { a + b }
  subtract.rs         // 3 lines: pub fn sub(a, b) { a - b }
  multiply.rs         // 3 lines: pub fn mul(a, b) { a * b }
  ... (50 files)
```

**Why Bad**: Overhead outweighs benefits

**Better**: Group related functions
```rust
utils/math.rs:
  pub fn add(...) { ... }
  pub fn subtract(...) { ... }
  pub fn multiply(...) { ... }
```

---

## 📚 Further Reading

- **Clean Code** (Robert Martin) - Chapter on Function Size
- **Code Complete** (Steve McConnell) - Routine Length
- **Philosophy of Software Design** (John Ousterhout) - "Deep Modules"

**Key Insight**: Module depth (information hiding) > module size

---

## ✅ Approval Process

When proposing to keep a large file (>1000 lines):

1. ✅ Document why it's cohesive (in file header comment)
2. ✅ Identify what WAS extracted (show you considered splitting)
3. ✅ Confirm high cohesion (single responsibility)
4. ✅ Verify no obvious split points
5. ✅ Add to this document's tracking table

When proposing to split a file:

1. ✅ Identify clean boundaries (independent subsystems)
2. ✅ Create meaningful module names (not part1/part2)
3. ✅ Measure impact (readability, performance, build time)
4. ✅ Update documentation

---

**Status**: ✅ **Active Policy**  
**Grade**: A+ (98/100) - Smart, principled approach to code organization

🌸 **"Quality is not arbitrary rules, but thoughtful design."** 🌸

