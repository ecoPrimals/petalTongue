# Critical Bug Fix - January 8, 2026

## Bug: SIGTERM Sent Instead of Signal 0

**Severity**: 🔴 CRITICAL  
**Impact**: Program killed itself during startup  
**Fixed**: ✅ Yes  
**Date**: January 8, 2026

## Description

The `process_exists()` function in `crates/petal-tongue-core/src/instance.rs` was sending **SIGTERM** (terminate signal) instead of signal 0 (null signal) when checking if a process exists.

### What Happened

```rust
// BEFORE (BUG):
match kill(Pid::from_raw(pid as i32), Signal::SIGTERM) {
    Ok(()) | Err(nix::errno::Errno::EPERM) => true,
    Err(nix::errno::Errno::ESRCH | _) => false,
}
```

When `registry.gc()` ran during startup, it would:
1. Check if old instances were still alive
2. Call `process_exists(pid)` for each old PID
3. **Send SIGTERM to those PIDs** to "check" if they exist
4. One of those PIDs would be the current process
5. **Kill the process it was trying to check**

This caused the mysterious exit code 143 (128 + 15 = SIGTERM).

### Root Cause

Misuse of `nix::sys::signal::kill()`:
- Signal 0 (`None`) = check if process exists (no signal sent)
- SIGTERM = terminate the process
- The code was using SIGTERM when it should have used None

### The Fix

```rust
// AFTER (CORRECT):
match kill(Pid::from_raw(pid as i32), None) {  // None = signal 0
    Ok(()) | Err(nix::errno::Errno::EPERM) => true,
    Err(nix::errno::Errno::ESRCH) => false,
    Err(_) => false,
}
```

Signal 0 is a special "null signal" that:
- Doesn't send any actual signal to the process
- Returns success if the process exists
- Returns ESRCH if the process doesn't exist
- Returns EPERM if process exists but we lack permissions

### Impact

**Before Fix:**
- Program would crash immediately after startup
- Exit code: 143 (SIGTERM)
- Logs showed it stopped right after "Running garbage collection..."
- **22 dead instances had accumulated** because gc() kept killing itself

**After Fix:**
- Program starts successfully
- Cleans up all 22 dead instances properly
- Continues to display check and tutorial mode
- Runs normally

## Lessons Learned

1. **Signal 0 is for existence checks**: Always use `None` (signal 0) when checking if a process exists
2. **Test garbage collection**: This bug only manifested when old instances existed
3. **Be careful with system calls**: `kill()` is a dangerous function - wrong signal = terminated process
4. **Deep debt solutions**: This was found during the Pure Rust display system integration
5. **Follow the logs**: The exact crash point (after gc()) led us to the bug

## Related Issues

- Exit code 143 during startup
- GUI termination before display prompt
- Registry accumulating dead instances

## Testing

```bash
# Verify fix
cargo build --release --bin petal-tongue
RUST_LOG=info ./target/release/petal-tongue

# Should see:
# ✅ Instance registered
# 🧹 Cleaned up 22 dead instances
# ✅ Display server detected
# (No SIGTERM crash)
```

## Code Quality Notes

This fix exemplifies:
- ✅ **Fast AND Safe Rust**: Using correct system APIs
- ✅ **Modern idiomatic Rust**: Proper error handling
- ✅ **Deep debt solutions**: Found and fixed during feature work

---

**Status**: ✅ RESOLVED  
**Files Changed**: `crates/petal-tongue-core/src/instance.rs`  
**Lines Changed**: 1 (SIGTERM → None)  
**Impact**: Critical bug eliminated

