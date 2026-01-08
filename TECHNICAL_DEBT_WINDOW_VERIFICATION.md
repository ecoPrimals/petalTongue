# Technical Debt: Window Verification Issue

**Date**: January 8, 2026  
**Severity**: Medium  
**Component**: `crates/petal-tongue-ui/src/main.rs`  
**Status**: Identified, needs fix

---

## Issue Description

The main `petal-tongue` binary reports "Display server detected" and appears to run successfully, but there's no confirmation that the eframe window actually becomes visible to the user.

### Symptoms

1. Process starts and runs (shows PID)
2. Logs show "✅ Display server detected"
3. `eframe::run_native()` is called
4. No error occurs
5. **BUT**: No visible window appears for the user
6. No post-initialization logging to confirm window creation
7. Process blocks forever in eframe event loop with no user feedback

### Root Cause

**"Trust but don't verify" pattern**:

```rust
// main.rs line 87-91
} else {
    tracing::info!("✅ Display server detected");
}

// Try to run with eframe
let result = run_with_eframe();  // This blocks forever if successful!
```

The code:
1. Detects `DISPLAY` environment variable exists ✅
2. Assumes that means eframe will work ✅
3. Calls `eframe::run_native()` which blocks ✅
4. **Never verifies the window is actually visible** ❌

### The Problem

`eframe::run_native()` is synchronous and blocks:
- If successful: blocks forever in event loop, no logs after this point
- If failed: returns error, but too late to do anything about it
- No way to confirm window appeared without external tools
- User has no feedback about whether GUI is actually working

### Why This Matters

This violates primal principles:
- **No self-knowledge**: The process doesn't know if it's actually rendering
- **No graceful degradation**: Can't fall back if window creation fails silently
- **No verification**: Trusts external substrate without confirmation
- **Poor UX**: User doesn't know if they should see something

---

## Test Case

### Environment
- System: Linux (X11)
- Display: `DISPLAY=:1`
- X Server: Running (Xorg PID 5019)

### Behavior
```bash
$ AWAKENING_ENABLED=true cargo run --release --bin petal-tongue

2026-01-08T16:04:28.710192Z  INFO petal_tongue: ✅ Display server detected
[process blocks here forever]
[no further logs]
[no visible window appears for user]
[process shows as running in ps aux]
```

### Expected Behavior
```bash
$ AWAKENING_ENABLED=true cargo run --release --bin petal-tongue

2026-01-08T16:04:28.710192Z  INFO petal_tongue: ✅ Display server detected
2026-01-08T16:04:28.850000Z  INFO petal_tongue: 🪟 Creating window...
2026-01-08T16:04:29.100000Z  INFO petal_tongue: ✅ Window created and visible
2026-01-08T16:04:29.150000Z  INFO petal_tongue: 🌸 Awakening experience starting
[window appears]
[user can interact]
```

---

## Impact

### Severity: Medium

**User Impact**:
- Confusion: Process runs but nothing visible
- No feedback: Can't tell if it's working
- Poor experience: Looks broken even when it's not

**Development Impact**:
- Hard to debug: No logs after eframe call
- Can't verify: No way to test window creation
- False positives: Process running doesn't mean it's working

**Deployment Impact**:
- SSH/Remote: Can't tell if GUI would work
- Containers: May think it works when it doesn't
- CI/CD: Can't validate GUI functionality

---

## Proposed Solutions

### Solution 1: Pre-flight Window Test (Best)

**Add verification before main app**:

```rust
fn verify_window_creation() -> Result<bool> {
    // Create a minimal test window
    let test_result = std::thread::spawn(|| {
        // Try to create a small test window
        // Return success/failure within 2 seconds
    }).join_timeout(Duration::from_secs(2));
    
    match test_result {
        Ok(Ok(true)) => {
            tracing::info!("✅ Window creation verified");
            Ok(true)
        }
        _ => {
            tracing::warn!("⚠️  Could not create window");
            Ok(false)
        }
    }
}
```

**Pros**: 
- Verifies before committing
- Can gracefully fall back
- User gets immediate feedback

**Cons**:
- Adds startup latency (~2 seconds)
- Extra complexity

### Solution 2: Parallel Logging Thread

**Start a thread that logs status**:

```rust
fn run_with_eframe() -> Result<(), eframe::Error> {
    // Start status thread
    let (tx, rx) = std::sync::mpsc::channel();
    
    std::thread::spawn(move || {
        loop {
            if let Ok(msg) = rx.recv_timeout(Duration::from_secs(5)) {
                tracing::info!("📊 Status: {}", msg);
            } else {
                tracing::info!("💚 GUI running (heartbeat)");
            }
        }
    });
    
    tx.send("Starting window creation").ok();
    
    let result = eframe::run_native(
        "petalTongue",
        options,
        Box::new(move |cc| {
            tx.send("Window created").ok();
            Ok(Box::new(PetalTongueApp::new(cc)))
        }),
    );
    
    result
}
```

**Pros**:
- No extra latency
- Provides ongoing feedback
- Easy to implement

**Cons**:
- Still doesn't verify visibility
- Thread overhead

### Solution 3: Use Our Pure Rust Backend (Immediate)

**Fall back to software rendering**:

```rust
if !has_display || !verify_window_creation()? {
    tracing::info!("🎨 Using Pure Rust software rendering");
    run_with_software_backend()?;
} else {
    tracing::info!("🪟 Using native window");
    run_with_eframe()?;
}
```

**Pros**:
- Always works
- We already built this!
- True sovereignty

**Cons**:
- Different code paths
- Need to maintain both

### Solution 4: Document and Accept (Not Recommended)

**Add clear documentation**:

```rust
// NOTE: eframe::run_native() blocks here. If successful,
// the window should be visible but we have no way to verify.
// If you don't see a window, check your display server.
let result = run_with_eframe();
```

**Pros**:
- No code changes
- Simple

**Cons**:
- Doesn't solve the problem
- Poor user experience
- Not primal

---

## Recommendation

**Implement Solution 2 (Parallel Logging) + Solution 3 (Software Fallback)**:

1. Add heartbeat logging so user knows it's running
2. Provide clear instructions if window doesn't appear
3. Offer Pure Rust backend as fallback option
4. Eventually migrate to Pure Rust as primary

This gives:
- ✅ Immediate feedback
- ✅ Graceful degradation
- ✅ Path to full sovereignty
- ✅ Better user experience

---

## Action Items

- [ ] Add heartbeat logging thread
- [ ] Add window creation timeout
- [ ] Provide software rendering fallback
- [ ] Update user documentation
- [ ] Add verification tests
- [ ] Consider migrating to Pure Rust as primary

---

## Related Work

- **Pure Rust Display System**: Already complete (v0.3.0-dev)
- **Software Backend**: Working and tested
- **Examples**: `awakening_pure_rust.rs` demonstrates this works
- **EguiPixelRenderer**: Complete implementation

We have all the pieces to solve this - just need to wire them together!

---

**Priority**: Medium (doesn't break functionality, but hurts UX)  
**Effort**: Low (2-3 hours)  
**Impact**: High (much better user experience)

This is addressable debt that we should fix in v0.3.1. 🌸

