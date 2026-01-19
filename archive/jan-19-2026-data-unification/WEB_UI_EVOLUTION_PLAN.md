# 🌐 Web UI Evolution Plan - Unified Rendering

**Date**: January 19, 2026  
**Goal**: Web UI renders the same as Desktop GUI  
**Approach**: egui WASM compilation

---

## 🎯 Vision

**Current**:
```
Desktop GUI:  egui → eframe → native window
Web UI:       Static HTML → Simple API calls
```

**Goal**:
```
Desktop GUI:  egui → eframe → native window
Web UI:       egui → WASM → browser canvas
             (same rendering, different backend!)
```

---

## ✨ Why This Works

### egui Already Supports WASM!
```toml
# From petal-tongue-ui/Cargo.toml
[features]
toadstool-wasm = ["wasm-bindgen", "wasm-bindgen-futures", "web-sys"]
```

### Architecture
```
PetalTongueApp (same code!)
     ↓
   egui (Pure Rust UI)
     ↓
  ┌──────────┴──────────┐
  ↓                     ↓
eframe (native)    eframe_web (WASM)
  ↓                     ↓
Window              Canvas element
```

**One UI system, two rendering targets!** 🎉

---

## 🚀 Implementation Plan

### Phase 1: WASM Build Setup
1. Add `wasm32-unknown-unknown` target
2. Create WASM entry point
3. Configure `trunk` or `wasm-pack` for building

### Phase 2: Web Entry Point
1. Create `src/web_ui_mode.rs`
2. Compile `PetalTongueApp` to WASM
3. Serve WASM + HTML

### Phase 3: Integration
1. Update web mode to serve WASM UI
2. Fallback to static HTML if WASM not available
3. Progressive enhancement

---

## 📝 Code Structure

### New: `src/web_ui_mode.rs`
```rust
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn start_web_ui() -> Result<(), JsValue> {
    use petal_tongue_ui::PetalTongueApp;
    
    // Set up logging for WASM
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    
    // Start egui app in browser
    let web_options = eframe::WebOptions::default();
    
    eframe::WebRunner::new()
        .start(
            "petaltongue_canvas", // Canvas element ID
            web_options,
            Box::new(|cc| Ok(Box::new(PetalTongueApp::new(cc, None, None)))),
        )
        .await
}
```

### Updated: `web/index.html`
```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>🌸 petalTongue Web</title>
    <style>
        html, body, #petaltongue_canvas {
            margin: 0;
            padding: 0;
            width: 100%;
            height: 100%;
        }
    </style>
</head>
<body>
    <canvas id="petaltongue_canvas"></canvas>
    <script type="module">
        import init from './petaltongue_web.js';
        init().then(() => {
            console.log('🌸 petalTongue WASM loaded!');
        });
    </script>
</body>
</html>
```

### Updated: `src/web_mode.rs`
```rust
// Serve both static HTML (fallback) and WASM UI
async fn serve_web_ui() {
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/petaltongue_web.js", get(wasm_js_handler))
        .route("/petaltongue_web_bg.wasm", get(wasm_handler))
        .route("/api/*", /* existing API routes */);
}
```

---

## 🎯 Benefits

### Unified Codebase
- ✅ **One UI implementation** (PetalTongueApp)
- ✅ **Same features** everywhere
- ✅ **Single source of truth**

### TRUE PRIMAL
- ✅ **Zero duplication**: Desktop and web share code
- ✅ **Capability-based**: Browser renders what it can
- ✅ **Graceful degradation**: Falls back to static HTML

### User Experience
- ✅ **Consistent**: Desktop and web look identical
- ✅ **Interactive**: Full GUI in browser
- ✅ **Modern**: Canvas-based rendering

---

## 🧪 Testing Strategy

### Development
```bash
# Build WASM
wasm-pack build --target web --out-dir web/pkg

# Serve locally
petaltongue web --bind 0.0.0.0:8080

# Test in browser
open http://localhost:8080
```

### Production
```bash
# Build optimized WASM
wasm-pack build --release --target web

# Serve via web mode
petaltongue web --bind 0.0.0.0:80
```

---

## 📊 Size Comparison

### Current (Static HTML)
- `index.html`: ~3KB
- Total: **~3KB**

### With WASM
- `index.html`: ~2KB (simpler!)
- `petaltongue_web_bg.wasm`: ~1-2MB (compressed)
- `petaltongue_web.js`: ~50KB
- Total: **~1-2MB** (but full GUI!)

**Trade-off**: Larger initial download for complete GUI functionality

---

## 🔮 Future Enhancements

### Progressive Loading
```rust
// Load UI incrementally
1. Show splash screen
2. Load core WASM
3. Stream panels on demand
4. Lazy-load Doom/video panels
```

### Hybrid Mode
```rust
// Mix WASM UI with server-side rendering
1. Critical UI: WASM (instant)
2. Heavy compute: Server-side (WebSocket updates)
3. Best of both worlds!
```

### Mobile Optimization
```rust
// Detect mobile, adapt UI
if is_mobile() {
    // Use touch-friendly layout
    // Smaller WASM bundle
    // Progressive enhancement
}
```

---

## ⚡ Quick Start (Immediate)

For now, let's enhance the current HTML to fetch and display real data:

### Enhanced `web/index.html`
- Fetch from Neural API
- Display proprioception
- Show graph visualization
- Real-time updates via WebSocket

This gives immediate value while we prepare WASM!

---

## 🎯 Recommendation

**Immediate** (Tonight):
1. Enhance current HTML with real data
2. Add graph visualization (SVG)
3. WebSocket for real-time updates

**Short-term** (This week):
1. Set up WASM build
2. Compile PetalTongueApp to WASM
3. Serve WASM UI

**Long-term** (Q1 2026):
1. Hybrid WASM + SSR
2. Progressive loading
3. Mobile optimization

---

**Status**: 🚀 Ready to evolve!  
**Approach**: Progressive enhancement (HTML → WASM)

🌸 **One UI, multiple targets!** 🌸

