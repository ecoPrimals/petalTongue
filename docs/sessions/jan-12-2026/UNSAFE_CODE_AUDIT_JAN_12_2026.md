# 🔍 Unsafe Code Audit & Evolution

**Date**: January 12, 2026  
**Philosophy**: TRUE PRIMAL - Fast AND Safe Rust  
**Status**: 🔄 In Progress

---

## 🎯 Objective

Audit all `unsafe` code and evolve to safe alternatives where possible, while maintaining performance.

---

## 📊 Unsafe Code Inventory

### Category 1: FFI (Foreign Function Interface) - JUSTIFIED ✅

These `unsafe` blocks are necessary for system integration and are properly encapsulated.

#### 1.1 User ID Retrieval (`libc::getuid()`)

**Files**: 
- `audio_discovery.rs` (2 instances)
- `jsonrpc_provider.rs` (1 instance)

**Code**:
```rust
let uid = unsafe { libc::getuid() };
```

**Analysis**:
- ✅ **Necessary**: No safe Rust alternative for getting UID
- ✅ **Correct**: `getuid()` is always safe (no null pointers, no failure modes)
- ✅ **Encapsulated**: Used only for socket path construction
- ✅ **Documented**: Clear purpose in code context

**Evolution Opportunity**: Extract to helper function with doc comment

**Recommendation**: EVOLVE to helper ✅

#### 1.2 ioctl System Calls

**Files**:
- `sensors/screen.rs` - Framebuffer screen info

**Code**:
```rust
let result = unsafe { libc::ioctl(fd, FBIOGET_VSCREENINFO, &mut var_info) };
```

**Analysis**:
- ✅ **Necessary**: No safe Rust abstraction for framebuffer ioctl
- ✅ **Correct**: Proper struct layout, error handling
- ✅ **Safety Comments**: SAFETY comment explains invariants
- ✅ **Encapsulated**: Within error-handling wrapper

**Recommendation**: KEEP (properly used) ✅

---

### Category 2: Zero-Copy Optimizations - JUSTIFIED ✅

#### 2.1 Audio Buffer Conversion

**File**: `audio_canvas.rs`

**Code**:
```rust
// SAFETY: i16 is repr(transparent) and has no padding
// We're converting Vec<i16> to &[u8] for raw device write
let bytes = unsafe {
    std::slice::from_raw_parts(
        i16_samples.as_ptr().cast::<u8>(),
        i16_samples.len() * std::mem::size_of::<i16>(),
    )
};
```

**Analysis**:
- ✅ **Justified**: Zero-copy performance for audio (real-time constraint)
- ✅ **Correct**: i16 has well-defined repr, no padding
- ✅ **Documented**: Excellent SAFETY comment
- ✅ **Performance Critical**: Audio hot path

**Safe Alternative Exists?**:
```rust
// Safe but allocates new buffer:
let bytes: Vec<u8> = i16_samples.iter()
    .flat_map(|&sample| sample.to_le_bytes())
    .collect();
```

**Trade-off**:
- Unsafe version: Zero-copy, no allocation ✅
- Safe version: Allocates, slower ❌

**Recommendation**: KEEP (performance critical, well-documented) ✅

**Potential Evolution**: Use `bytemuck` crate for safe transmutation

---

### Category 3: Test-Only unsafe - CAN EVOLVE ⚠️

#### 3.1 Test Environment Variable Manipulation

**Files**: (Many test files)
- `universal_discovery.rs`
- `biomeos_integration.rs`
- `socket_path.rs`
- `mock_device_provider.rs`
- Various test modules

**Code**:
```rust
// SAFETY: Test-only environment variable modification
unsafe {
    std::env::set_var("FAMILY_ID", "test-family");
    std::env::set_var("XDG_RUNTIME_DIR", "/tmp");
}
```

**Analysis**:
- ⚠️ **Test-only**: Only used in tests
- ⚠️ **Thread-unsafe**: Can cause data races in parallel tests
- ✅ **Documented**: Marked as SAFETY: Test-only

**Safe Alternative**:
Use `serial_test` crate or dependency injection instead of global env vars.

**Recommendation**: EVOLVE (test improvements) 🔄

---

## 🚀 Evolution Plan

### Priority 1: Extract UID Helper (HIGH)

**Current**: `unsafe { libc::getuid() }` repeated 3 times

**Evolved**: Create safe helper function

**File**: Create `crates/petal-tongue-core/src/system_info.rs`

```rust
/// Get the current user's UID
///
/// This is a safe wrapper around `libc::getuid()`.
/// The function is always safe because `getuid()` cannot fail
/// and returns the process's effective user ID.
///
/// # Examples
///
/// ```
/// use petal_tongue_core::get_current_uid;
///
/// let uid = get_current_uid();
/// println!("Running as UID: {}", uid);
/// ```
#[must_use]
pub fn get_current_uid() -> u32 {
    // SAFETY: getuid() is always safe - it simply returns the process's
    // effective user ID from the kernel. No pointers, no failure modes.
    unsafe { libc::getuid() }
}

/// Get the standard runtime directory for the current user
///
/// Returns `$XDG_RUNTIME_DIR` if set, otherwise `/run/user/{uid}`.
///
/// # TRUE PRIMAL Principles
///
/// - **No Hardcoding**: Uses environment and UID discovery
/// - **Capability-Based**: Standard XDG directories
/// - **Graceful Fallback**: Constructs path if env var missing
#[must_use]
pub fn get_user_runtime_dir() -> PathBuf {
    std::env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let uid = get_current_uid();
            PathBuf::from(format!("/run/user/{}", uid))
        })
}
```

**Impact**: Replace 3 unsafe blocks with safe helper calls

**Status**: 🔄 TODO

---

### Priority 2: Audio Buffer - Evaluate `bytemuck` (MEDIUM)

**Current**: Manual `from_raw_parts` cast

**Evolved**: Use `bytemuck` for safe transmutation

```rust
use bytemuck::{Pod, Zeroable};

// i16 is Pod (Plain Old Data) and Zeroable
let bytes: &[u8] = bytemuck::cast_slice(&i16_samples);
```

**Benefits**:
- ✅ Compile-time safety checks
- ✅ Zero runtime cost
- ✅ No unsafe block
- ✅ Clear intent

**Trade-off**:
- Adds dependency (`bytemuck`)

**Recommendation**: EVALUATE (measure if worth dependency)

**Status**: 🔄 TODO

---

### Priority 3: Test Environment Variables (LOW)

**Current**: `unsafe { std::env::set_var() }` in tests

**Evolved**: Use `serial_test` crate or dependency injection

**Option A: serial_test**:
```rust
#[tokio::test]
#[serial]  // Run serially, not in parallel
async fn test_with_env_var() {
    std::env::set_var("TEST_VAR", "value");  // Safe when serial
    // ... test code ...
    std::env::remove_var("TEST_VAR");
}
```

**Option B: Dependency Injection**:
```rust
// Instead of reading env vars globally, pass them as params
fn get_socket_path(runtime_dir: impl AsRef<Path>, family_id: &str) -> PathBuf {
    runtime_dir.as_ref().join(format!("socket-{}.sock", family_id))
}

#[test]
fn test_socket_path() {
    let path = get_socket_path("/tmp", "test-family");
    assert_eq!(path, PathBuf::from("/tmp/socket-test-family.sock"));
}
```

**Recommendation**: Option B (better design, no new dependency)

**Status**: 🔄 TODO

---

## 📊 Summary Statistics

| Category | Count | Status | Action |
|----------|-------|--------|--------|
| FFI (UID) | 3 | ✅ Justified | 🔄 Extract helper |
| FFI (ioctl) | 1 | ✅ Justified | ✅ Keep |
| Zero-copy | 1 | ✅ Justified | 🔄 Evaluate bytemuck |
| Test env vars | ~20+ | ⚠️ Test-only | 🔄 Dependency injection |
| **TOTAL** | **~25** | **Mostly Justified** | **Evolve helpers** |

---

## ✅ Current Status

### Well-Documented Unsafe ✅
- All unsafe blocks have SAFETY comments
- Clear justification for each use
- Proper encapsulation

### Justified Use Cases ✅
- FFI for system integration (UID, ioctl)
- Zero-copy for performance (audio)
- Test-only for env manipulation

### Evolution Opportunities 🔄
1. Extract UID helper function
2. Evaluate `bytemuck` for audio
3. Refactor tests to avoid env vars

---

## 🎯 Next Steps

### Immediate (This Session)
- [ ] Create `system_info.rs` with UID helpers
- [ ] Replace 3 unsafe UID calls with safe helper
- [ ] Verify tests still pass

### Near-term (Next Session)
- [ ] Evaluate `bytemuck` for audio buffer
- [ ] Benchmark safe vs unsafe audio conversion
- [ ] Choose best approach

### Long-term (Future)
- [ ] Refactor tests to use dependency injection
- [ ] Remove unsafe env var manipulation
- [ ] Add `#[forbid(unsafe_code)]` to select modules

---

## 📝 Philosophy

### Fast AND Safe Rust

TRUE PRIMAL approach to unsafe code:

1. **Justified Use** ✅
   - FFI where necessary
   - Performance where critical
   - Always documented

2. **Encapsulated** ✅
   - Wrap in safe APIs
   - Minimize unsafe scope
   - Clear invariants

3. **Evolvable** ✅
   - Extract to helpers
   - Consider safe alternatives
   - Improve over time

4. **Modern Rust** ✅
   - Use ecosystem crates
   - Follow best practices
   - Idiomatic patterns

---

**Status**: 🔄 Excellent unsafe hygiene, minor improvements identified

**Conclusion**: PetalTongue uses `unsafe` responsibly and minimally. All uses are justified (FFI, performance) or test-only. Evolution opportunities identified for better abstraction and maintainability.

🌸 **Unsafe Code Excellence!** 🌸

