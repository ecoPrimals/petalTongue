# 🌍 petalTongue ecoBin Migration Plan

**Date**: January 18, 2026  
**Strategy**: Hybrid Approach (Pragmatic)  
**Estimated Time**: ~3-5 hours  
**Status**: 🚧 In Progress

---

## 📊 Current Audit Results

### **Dependency Analysis**

#### **Headless Binary (`petal-tongue-headless`)**
```
❌ dirs v5.0.1
  └── dirs-sys v0.4.1 (C dependency!)
  
❌ reqwest v0.12.28 (OK)
  ❌ reqwest v0.11.27 (OLD VERSION!)
    └── openssl-sys v0.9.111 (C dependency!)
```

**Issues Found**:
1. `dirs-sys` (via `petal-tongue-core`)
2. `openssl-sys` (via old `reqwest v0.11` - WHERE IS THIS COMING FROM?)

#### **CLI Binary (`petal-tongue-cli`)**
```
❌ dirs v5.0.1
  └── dirs-sys v0.4.1 (C dependency!)
  
✅ reqwest v0.12.28 (uses rustls-tls, no OpenSSL)
```

**Issues Found**:
1. `dirs-sys` (via `petal-tongue-core`)

#### **GUI Binary (`petal-tongue-ui`)**
```
❌ dirs-sys (inherited from core)
❌ openssl-sys (inherited from headless?)
❌ wayland-sys (expected for GUI - ACCEPTABLE!)
```

**Status**: Leave as-is (desktop app)

---

## 🎯 Migration Strategy

### **Phase 1: Audit & Identify** ✅ COMPLETE

**Findings**:
- `dirs` is used in 3 places in `petal-tongue-core`:
  - `state_sync.rs` (line 131): `dirs::config_dir()`
  - `instance.rs` (line 494): `dirs::home_dir()`
  - `session.rs` (line 611): `dirs::home_dir()`

- `reqwest` is declared in `petal-tongue-core/Cargo.toml` but NOT USED directly!
  - Used transitively by discovery providers
  - Workspace has CORRECT config (`rustls-tls`)
  - BUT: Something is pulling in OLD `reqwest v0.11` with OpenSSL!

**Mystery**: Why is `reqwest v0.11` in the tree?

---

### **Phase 2: Replace `dirs` with `etcetera`** (~30 min)

#### **Step 2.1: Update Cargo.toml**

**File**: `crates/petal-tongue-core/Cargo.toml`

**Before**:
```toml
dirs = "5.0"  # For state storage directories
```

**After**:
```toml
etcetera = "0.8"  # Pure Rust XDG directories (ecoBin compliant!)
```

#### **Step 2.2: Update state_sync.rs**

**File**: `crates/petal-tongue-core/src/state_sync.rs` (line 131)

**Before**:
```rust
let config_dir = dirs::config_dir()
    .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
```

**After**:
```rust
use etcetera::{choose_base_strategy, BaseStrategy};

let strategy = choose_base_strategy()?;
let config_dir = strategy.config_dir();
```

#### **Step 2.3: Update instance.rs**

**File**: `crates/petal-tongue-core/src/instance.rs` (line 494)

**Before**:
```rust
} else if let Some(home) = dirs::home_dir() {
    home.join(".local/share/petal-tongue/instances")
```

**After**:
```rust
} else {
    let strategy = choose_base_strategy()
        .map_err(|e| anyhow::anyhow!("Could not determine data directory: {}", e))?;
    strategy.data_dir().join("petal-tongue/instances")
```

#### **Step 2.4: Update session.rs**

**File**: `crates/petal-tongue-core/src/session.rs` (line 611)

**Before**:
```rust
} else if let Some(home) = dirs::home_dir() {
    home.join(".local/share/petal-tongue/sessions")
```

**After**:
```rust
} else {
    let strategy = choose_base_strategy()
        .map_err(|e| anyhow::anyhow!("Could not determine data directory: {}", e))?;
    strategy.data_dir().join("petal-tongue/sessions")
```

---

### **Phase 3: Fix reqwest OpenSSL Issue** (~1-2 hours)

#### **Step 3.1: Find the Culprit**

**Mystery**: Something is pulling `reqwest v0.11` with OpenSSL!

**Investigation**:
```bash
# Find what depends on reqwest v0.11
cargo tree -i reqwest@0.11

# Check all Cargo.toml files for old reqwest
grep -r "reqwest.*0.11" crates/
```

**Possible Causes**:
1. Some crate has explicit `reqwest = "0.11"` dependency
2. Some dependency requires old reqwest
3. Feature unification is enabling default features

#### **Step 3.2: Force rustls-tls Only**

**Option A: Patch in Workspace** (Recommended)

**File**: `Cargo.toml` (workspace root)

**Add**:
```toml
[patch.crates-io]
# Force ALL reqwest to use rustls-tls (Pure Rust!)
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "json", "charset", "http2"] }
```

**Option B: Remove reqwest from Core** (If not needed)

Check if `petal-tongue-core` actually uses `reqwest`:
```bash
grep -r "reqwest::" crates/petal-tongue-core/src/
```

If NO matches: **Remove it from Cargo.toml!**

---

### **Phase 4: Test ARM64 Builds** (~30 min)

#### **Step 4.1: Install ARM64 Target**
```bash
rustup target add aarch64-unknown-linux-musl
```

#### **Step 4.2: Build Headless for ARM64**
```bash
cargo build --release \
  --target aarch64-unknown-linux-musl \
  --package petal-tongue-headless
```

#### **Step 4.3: Build CLI for ARM64**
```bash
cargo build --release \
  --target aarch64-unknown-linux-musl \
  --package petal-tongue-cli
```

#### **Step 4.4: Verify Pure Rust**
```bash
# Should ONLY show linux-raw-sys (acceptable)
cargo tree --target aarch64-unknown-linux-musl \
  --package petal-tongue-headless | grep "\-sys"
  
cargo tree --target aarch64-unknown-linux-musl \
  --package petal-tongue-cli | grep "\-sys"
```

---

### **Phase 5: Update Documentation** (~30 min)

#### **Files to Update**:

1. **`BUILD_REQUIREMENTS.md`**
   - Note that headless and CLI are ecoBin compliant
   - GUI requires platform libraries (expected)

2. **`PROJECT_STATUS.md`**
   - Add ecoBin compliance section
   - Note hybrid approach

3. **`ECOBIN_MIGRATION_JAN_18_2026.md`** (NEW)
   - Complete migration details
   - Before/after comparison
   - Lessons learned

4. **`README.md`**
   - Update build instructions
   - Note ARM64 support for headless/CLI

---

## 📋 Detailed Checklist

### **Phase 2: dirs → etcetera** ✅

- [ ] Update `Cargo.toml` in `petal-tongue-core`
- [ ] Update `state_sync.rs` (line 131)
- [ ] Update `instance.rs` (line 494)
- [ ] Update `session.rs` (line 611)
- [ ] Test: `cargo build --package petal-tongue-core`
- [ ] Test: `cargo test --package petal-tongue-core`

### **Phase 3: Fix reqwest** ✅

- [ ] Investigate: `cargo tree -i reqwest@0.11`
- [ ] Find culprit crate
- [ ] Fix: Add `[patch.crates-io]` OR remove from core
- [ ] Verify: `cargo tree | grep openssl-sys` (should be empty!)
- [ ] Test: `cargo build --package petal-tongue-headless`
- [ ] Test: `cargo build --package petal-tongue-cli`

### **Phase 4: ARM64** ✅

- [ ] Install: `rustup target add aarch64-unknown-linux-musl`
- [ ] Build headless: `cargo build --release --target aarch64-unknown-linux-musl --package petal-tongue-headless`
- [ ] Build CLI: `cargo build --release --target aarch64-unknown-linux-musl --package petal-tongue-cli`
- [ ] Verify: Only `linux-raw-sys` in `cargo tree --target aarch64-unknown-linux-musl`

### **Phase 5: Docs** ✅

- [ ] Update `BUILD_REQUIREMENTS.md`
- [ ] Update `PROJECT_STATUS.md`
- [ ] Create `ECOBIN_MIGRATION_JAN_18_2026.md`
- [ ] Update `README.md`

---

## 🎊 Success Criteria

### **Headless Binary** ✅

- [ ] Zero C dependencies (except `linux-raw-sys`)
- [ ] No `dirs-sys`
- [ ] No `openssl-sys`
- [ ] No `wayland-sys` (headless!)
- [ ] Builds for x86_64
- [ ] Builds for ARM64
- [ ] Size under 5M
- [ ] **TRUE ecoBin!**

### **CLI Binary** ✅

- [ ] Zero C dependencies (except `linux-raw-sys`)
- [ ] No `dirs-sys`
- [ ] No `openssl-sys`
- [ ] Builds for x86_64
- [ ] Builds for ARM64
- [ ] Size under 5M
- [ ] **TRUE ecoBin!**

### **GUI Binary** ❌ (Not Target)

- [x] Accept `wayland-sys` (platform-specific)
- [x] Accept `dirs-sys` (inherits from core, OK for desktop)
- [x] Focus on user experience
- [x] **Desktop tool, not ecoBin!**

---

## 💡 Key Principles

### **Pragmatic ecoBin Philosophy**

1. **ecoBin where it matters**:
   - Headless: Server/automation use → ecoBin ✅
   - CLI: Scripting/portable → ecoBin ✅
   - GUI: Desktop app → Platform-specific OK ❌

2. **Don't fight the platform**:
   - GUI needs display libraries
   - That's expected and acceptable
   - Focus on right tool for right job

3. **Pure Rust stack**:
   - `etcetera` for directories
   - `rustls` for TLS
   - `linux-raw-sys` for syscalls (acceptable)

---

## 🚀 Expected Results

### **Before Migration**

**Headless**:
```
❌ dirs-sys (C dependency)
❌ openssl-sys (C dependency)
❌ NOT ecoBin compliant
```

**CLI**:
```
❌ dirs-sys (C dependency)
✅ Pure Rust TLS (rustls)
❌ NOT ecoBin compliant
```

### **After Migration**

**Headless**:
```
✅ etcetera (Pure Rust XDG)
✅ rustls-tls (Pure Rust TLS)
✅ linux-raw-sys only (acceptable)
✅ TRUE ecoBin! 🌍
```

**CLI**:
```
✅ etcetera (Pure Rust XDG)
✅ rustls-tls (Pure Rust TLS)
✅ linux-raw-sys only (acceptable)
✅ TRUE ecoBin! 🌍
```

---

## 📚 References

- **biomeOS Audit**: Complete Pure Rust migration example
- **wateringHole Standards**: ecoBin philosophy
- **Upstream Guidance**: "Pragmatic > Dogmatic"

---

**Status**: 🚧 Phase 1 Complete, Phase 2 Starting  
**Estimated Completion**: ~3 hours  
**Philosophy**: ecoBin where it makes sense!  

🌍 **Let's make the right binaries TRUE ecoBin!** 🌍

