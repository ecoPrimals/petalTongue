# Remote Display Diagnostic - petalTongue Window Visibility Issue

**Date**: January 9, 2026  
**Status**: ✅ RESOLVED - Mutex deadlock fixed  
**Environment**: RustDesk remote desktop, HDMI dummy plug, physical monitor now connected

## Resolution

**Root Cause**: Mutex deadlock in `StatusReporter::update_modality()`  
**Fix**: Scope mutex lock to avoid re-entrant lock attempt  
**Result**: petalTongue now works perfectly on remote displays

See `CRITICAL_BUG_FIX_DEADLOCK.md` for complete details.

---

# Original Investigation (Archived)

## Summary of Issue

User reports that when petalTongue launches on `DISPLAY=:1` (via RustDesk), the window appears but shows Cursor IDE's content instead of petalTongue's UI.

## Symptoms

1. ✅ Window creates successfully (`xwininfo` shows window with correct title)
2. ✅ Window can be mapped with `xdotool windowmap`  
3. ✅ Window state is `IsViewable` and `Normal`
4. ✅ Window responds to move/activate commands
5. ❌ Window content shows Cursor IDE instead of petalTongue UI
6. ❌ Application logs stop after initialization (no update loop messages)
7. ❌ No proprioception system updates in logs (should update every 5s)

## Critical Discovery: Cursor File Descriptor Leakage

**MAJOR FINDING**: petalTongue process has Cursor log files open as file descriptors:

```
lsof -p <PID> output:
petal-ton 156816 user   44w REG  ~/.config/Cursor/logs/.../exthost/exthost.log
petal-ton 156816 user   46w REG  ~/.config/Cursor/logs/.../extHostTelemetry.log
petal-ton 156816 user   47w REG  ~/.config/Cursor/logs/.../Cursor Agent Review.log
... (multiple Cursor log files)
```

**This persists even when:**
- Launched with `nohup` and `disown`
- Completely detached from terminal
- No pipes or redirects to Cursor

### Possible Causes

1. **File Descriptor Inheritance**: Cursor terminal is passing open FDs to child processes
2. **Shared Library Issue**: Some dependency is opening these files
3. **Environment Pollution**: Environment variables or config causing this
4. **egui/winit Bug**: Window content rendering failure showing "through" to background

## Window Mapping Timeline

### Initial Issue (No Monitor)
- Window created but `Map State: IsUnMapped`
- `.with_visible(true)` in eframe config didn't help
- Manual `xdotool windowmap` made it "viewable"

### Current Issue (With Monitor)
- Window `Map State: IsViewable`
- Window geometry correct (1280x655)
- **But content is wrong** - shows Cursor UI not petalTongue UI

## Logging Analysis

### Expected Behavior
```
2026-01-09T17:54:49.385469Z  INFO status_reporter: Status reporter starting
... (5 seconds later)
2026-01-09T17:54:54.XXX  INFO proprioception: Health: 85%, Confidence: 72%
2026-01-09T17:54:54.XXX  INFO proprioception: Visual output: Forwarded(ssh)
... (every 5 seconds)
```

### Actual Behavior
```
2026-01-09T17:54:49.385469Z  INFO status_reporter: Status reporter starting
... NOTHING ELSE ...
```

**Conclusion**: The egui event loop is **not running** or is **hung**.

## Potential Root Causes

### 1. egui/winit Window Creation Hang
- `eframe::run_native` may be blocking/hanging
- Event loop never starts
- Window shows "garbage" or background content

### 2. X11/RustDesk Compositing Issue  
- Window created but not receiving paint events
- Content defaults to "whatever is in that memory/framebuffer region"
- Happens to be Cursor's window content

### 3. Graphics Context Failure
- OpenGL/EGL context fails silently
- egui can't render
- Window exists but has no drawable surface

### 4. File Descriptor Pollution
- Cursor FDs interfering with egui's event loop
- epoll/select getting confused
- Event loop waiting on wrong descriptors

## Attempted Fixes

1. ✅ Added `.with_visible(true)` to `egui::ViewportBuilder` - **No effect**
2. ✅ Manual `xdotool windowmap` - **Window becomes viewable but still wrong content**
3. ✅ Detached process from Cursor terminal - **Cursor FDs still present**
4. ❌ Screenshot tools not available to see actual content

## Next Steps

### Immediate Diagnostic
1. **User Confirmation**: What exactly do you see in the petalTongue window?
   - Cursor IDE windows/tabs?
   - Black/blank window?
   - Static image?
   - Cursor but can interact with petalTongue?

2. **Test with Direct Local Display**: 
   ```bash
   DISPLAY=:0 ./target/release/petal-tongue
   ```
   (If you have a :0 display available)

3. **Minimal Reproduction**:
   ```bash
   # Test if ANY egui app has this issue
   cargo run --example pure_rust_gui_demo
   ```

### Code Investigation

1. **Check `crates/petal-tongue-ui/src/main.rs`**:
   - Add extensive logging around `eframe::run_native`
   - Log before/after window creation
   - Log in `PetalTongueApp::new()`
   - Log first few `update()` calls

2. **Check for Cursor-specific code**:
   ```bash
   grep -r "Cursor\|\.config/Cursor" crates/
   ```

3. **Check environment cleanup**:
   - Unset all Cursor-related env vars before launch
   - Clean FD inheritance

### Potential Fixes

#### Option A: Force Event Loop Start
```rust
// In main.rs
fn run_with_eframe() -> Result<(), eframe::Error> {
    env_logger::init(); // Verbose logging
    log::info!("ABOUT TO START EFRAME");
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_visible(true)
            .with_active(true) // NEW
            .with_decorations(true) // NEW
            .with_resizable(true),
        ..Default::default()
    };
    
    log::info!("CALLING run_native");
    let result = eframe::run_native(
        "petalTongue",
        options,
        Box::new(|cc| {
            log::info!("INSIDE APP CONSTRUCTOR");
            Ok(Box::new(PetalTongueApp::new(cc)))
        }),
    );
    log::info!("run_native RETURNED: {:?}", result);
    result
}
```

#### Option B: Close Inherited FDs
```rust
// At very start of main()
fn main() {
    // Close all non-standard file descriptors
    for fd in 3..256 {
        unsafe {
            libc::close(fd as i32);
        }
    }
    // ... rest of main
}
```

#### Option C: Test with winit directly
Create minimal window without eframe to isolate issue:
```rust
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Test")
        .build(&event_loop)
        .unwrap();
    
    event_loop.run(move |event, _, control_flow| {
        println!("Event: {:?}", event);
        // ...
    });
}
```

## Questions for User

1. **What exactly do you see** in the petalTongue window content?
2. **Can you interact** with what you see (clicks, typing)?
3. **Does it update** or is it static?
4. **Any other apps** work correctly via RustDesk?
5. **Can you test** on DISPLAY=:0 if available?

## Impact

**SEVERITY**: CRITICAL - Blocks all remote usage  
**SCOPE**: All headless/remote display scenarios  
**WORKAROUND**: None currently

This is a fundamental issue with how petalTongue interacts with remote X11 sessions and needs deep investigation into the eframe/winit/X11 interaction.

