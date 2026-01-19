# 🌍 petalTongue FULL UniBin + ecoBin Evolution Plan

**Date**: January 18, 2026  
**Status**: 🚧 Planning  
**Goal**: Go BEYOND upstream guidance - TRUE UniBin + ecoBin for ALL binaries!  
**Philosophy**: "Hand them a better evolved system"

---

## 🎯 **Mission**

**Current**: 2/3 binaries ecoBin, 3 separate binaries (not UniBin)  
**Target**: 1/1 binary ecoBin, UniBin architecture with subcommands  
**Ambition**: Prove Pure Rust GUI is possible!

---

## 📊 **Gap Analysis**

### **Gap 1: UniBin Violation**

**Current Architecture**:
```
petalTongue/
├── petal-tongue-ui (GUI binary, 35M)
├── petal-tongue-headless (server binary, 1.9M)
└── petaltongue (CLI binary, 2.4M)
```

**UniBin Architecture** (Target):
```
petalTongue/
└── petaltongue (ONE binary, ~40M)
    ├── petaltongue ui          # GUI mode
    ├── petaltongue headless    # Server mode
    ├── petaltongue status      # Status query
    ├── petaltongue connect     # Connect to instance
    └── petaltongue --help      # Show all subcommands
```

**Benefits**:
- ✅ Single binary to distribute
- ✅ Consistent user experience
- ✅ Shared code compiled once
- ✅ TRUE UniBin compliance

**Challenges**:
- GUI dependencies increase CLI binary size
- Need smart feature flags to minimize
- Need subcommand routing

**Effort**: ~2-4 hours

---

### **Gap 2: GUI C Dependencies (ecoBin Violation)**

**Current GUI Stack**:
```
egui 0.29
├── eframe 0.29
│   ├── winit (windowing)
│   │   └── wayland-sys ❌ (C dependency!)
│   │   └── x11-dl ❌ (C dependency!)
│   └── glow (OpenGL)
│       └── glutin
│           └── wayland-sys ❌
```

**Problem**: egui/eframe inherently depends on platform windowing (Wayland/X11/Win32)

**Solutions**:

#### **Option A: Pure Rust Windowing** (~1-2 weeks)
Use Pure Rust windowing alternatives:
- `softbuffer`: Pure Rust software rendering to window
- `raw-window-handle`: Abstraction over platform windows
- Direct framebuffer access (we already have this!)

**Pros**:
- ✅ TRUE ecoBin (100% Pure Rust!)
- ✅ No C dependencies
- ✅ Future-proof architecture

**Cons**:
- ⚠️ More work upfront
- ⚠️ Need to rewrite windowing layer
- ⚠️ May lose some egui features

#### **Option B: Software Rendering Only** (~3-5 days)
Keep egui but render to memory, display via Pure Rust:
- Render egui to framebuffer in memory
- Use `softbuffer` or direct `/dev/fb0` for display
- No GPU, no platform windowing needed

**Pros**:
- ✅ Keep egui (familiar API)
- ✅ Pure Rust display chain
- ✅ Moderate effort

**Cons**:
- ⚠️ Slower rendering (CPU-based)
- ⚠️ Need to handle input ourselves
- ⚠️ Still need window management somehow

#### **Option C: Headless egui + Custom Display** (~1 week)
Use egui in headless mode, create Pure Rust display:
- `egui` has headless rendering support
- Render to texture/framebuffer
- Custom Pure Rust display backend

**Pros**:
- ✅ Keep egui rendering
- ✅ Pure Rust display
- ✅ Flexible architecture

**Cons**:
- ⚠️ Need custom display backend
- ⚠️ Need custom input handling
- ⚠️ Medium complexity

#### **Option D: Pure Rust GUI from Scratch** (~2-4 weeks)
Build Pure Rust GUI using our primitives:
- Use `petal-tongue-primitives` framebuffer
- Implement basic widgets ourselves
- Software rendering pipeline

**Pros**:
- ✅ 100% Pure Rust, 100% control
- ✅ Minimal dependencies
- ✅ TRUE PRIMAL architecture

**Cons**:
- ⚠️ Most work
- ⚠️ Lose egui ecosystem
- ⚠️ Need to reimplement everything

---

## 🔬 **Deep Dive: What We Already Have**

### **Pure Rust Rendering Capabilities**

From `crates/petal-tongue-primitives/src/framebuffer.rs`:
```rust
// We already have direct framebuffer access!
pub struct Framebuffer {
    width: u32,
    height: u32,
    buffer: Vec<u32>,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Self { ... }
    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) { ... }
    pub fn get_buffer(&self) -> &[u32] { ... }
}
```

**This means we CAN render Pure Rust!**

### **What's Missing**:

1. **Windowing** (creating OS window without Wayland/X11)
2. **Input** (keyboard/mouse without platform libs)
3. **Display** (showing framebuffer without OpenGL)

---

## 🚀 **Recommended Evolution Path**

### **Phase 1: UniBin Consolidation** (~2-4 hours)

**Goal**: Single binary with subcommands

**Steps**:
1. Create new `src/main.rs` with `clap` subcommands
2. Move current binaries to subcommands:
   - `ui` → current petal-tongue-ui
   - `headless` → current petal-tongue-headless  
   - `status` → current petaltongue CLI
3. Use feature flags to minimize binary size
4. Test all modes work

**Result**: UniBin compliance! ✅

---

### **Phase 2: Pure Rust Display Backend** (~3-7 days)

**Goal**: Remove wayland-sys dependency from GUI

**Recommended Approach**: **Option C** (Headless egui + Custom Display)

**Why**:
- Keeps egui (proven, feature-rich)
- Pure Rust display chain
- Reasonable effort (1 week)
- Best balance of pragmatism + purity

**Steps**:

#### **2.1: Research Pure Rust Display Options** (~4 hours)
Evaluate:
- `softbuffer` crate (Pure Rust software rendering)
- `minifb` crate (Minimal framebuffer)
- Direct `/dev/fb0` access (Linux framebuffer device)
- `raw-window-handle` + custom backend

**Goal**: Find Pure Rust way to display pixels

#### **2.2: Create Pure Rust Display Backend** (~2-3 days)
- New crate: `petal-tongue-display` (Pure Rust!)
- Traits for display abstraction
- Implementations:
  - Linux: `/dev/fb0` or DRM/KMS (Pure Rust via `drm-rs`)
  - Cross-platform: `softbuffer`
  - Fallback: `minifb`

#### **2.3: Integrate with egui** (~1-2 days)
- Use egui's headless rendering
- Render to our framebuffer
- Display via Pure Rust backend
- Handle input ourselves (Pure Rust!)

#### **2.4: Test & Validate** (~1 day)
- Test on Linux (primary target)
- Verify zero C dependencies
- Test Doom still plays!
- Performance benchmarks

**Result**: ecoBin GUI! ✅

---

### **Phase 3: Optimization & Polish** (~2-3 days)

**Goals**:
- Minimize binary size
- Optimize software rendering
- Add GPU acceleration (if available, via Pure Rust)
- Documentation

---

## 📐 **Technical Architecture**

### **UniBin Structure**

```rust
// src/main.rs (ONE binary!)

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "petaltongue")]
#[command(about = "Universal UI & Visualization System")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch GUI mode (Pure Rust display!)
    Ui {
        #[arg(long)]
        scenario: Option<String>,
    },
    
    /// Run headless server
    Headless {
        #[arg(long, default_value = "0.0.0.0:8080")]
        bind: String,
    },
    
    /// Query status
    Status,
    
    /// Connect to running instance
    Connect {
        instance_id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Ui { scenario } => {
            // Use Pure Rust display backend!
            ui_mode::run(scenario).await
        }
        Commands::Headless { bind } => {
            headless_mode::run(bind).await
        }
        Commands::Status => {
            cli_mode::status().await
        }
        Commands::Connect { instance_id } => {
            cli_mode::connect(instance_id).await
        }
    }
}
```

### **Pure Rust Display Stack**

```rust
// crates/petal-tongue-display/src/lib.rs (NEW!)

/// Pure Rust display backend trait
pub trait DisplayBackend {
    fn create_window(&mut self, width: u32, height: u32) -> Result<()>;
    fn present(&mut self, framebuffer: &[u32]) -> Result<()>;
    fn poll_events(&mut self) -> Vec<InputEvent>;
}

/// Linux framebuffer backend (Pure Rust via /dev/fb0)
#[cfg(target_os = "linux")]
pub struct LinuxFbBackend {
    fb_device: File,
    width: u32,
    height: u32,
}

/// Software rendering backend (cross-platform)
pub struct SoftbufferBackend {
    context: softbuffer::Context,
    surface: softbuffer::Surface,
}

/// Choose best backend for platform
pub fn create_backend() -> Box<dyn DisplayBackend> {
    #[cfg(target_os = "linux")]
    if let Ok(backend) = LinuxFbBackend::new() {
        return Box::new(backend);
    }
    
    // Fallback to softbuffer (Pure Rust!)
    Box::new(SoftbufferBackend::new())
}
```

### **egui Integration**

```rust
// crates/petal-tongue-ui/src/pure_rust_app.rs (NEW!)

use egui::{Context, RawInput};
use petal_tongue_display::DisplayBackend;

pub struct PureRustApp {
    egui_ctx: Context,
    display: Box<dyn DisplayBackend>,
    framebuffer: Vec<u32>,
}

impl PureRustApp {
    pub fn new() -> Result<Self> {
        let display = petal_tongue_display::create_backend();
        display.create_window(1400, 900)?;
        
        Ok(Self {
            egui_ctx: Context::default(),
            display,
            framebuffer: vec![0; 1400 * 900],
        })
    }
    
    pub fn run(&mut self) -> Result<()> {
        loop {
            // Poll input (Pure Rust!)
            let events = self.display.poll_events();
            let raw_input = self.convert_events(events);
            
            // Run egui (headless)
            let output = self.egui_ctx.run(raw_input, |ctx| {
                // Your UI code here!
                self.render_ui(ctx);
            });
            
            // Render to framebuffer (Pure Rust!)
            self.paint_to_framebuffer(&output.shapes);
            
            // Display (Pure Rust!)
            self.display.present(&self.framebuffer)?;
        }
    }
    
    fn paint_to_framebuffer(&mut self, shapes: &[egui::ClippedShape]) {
        // Software rasterization of egui shapes
        // Could use egui_glow's software renderer or custom
    }
}
```

---

## 🧪 **Proof of Concept: Minimal Steps**

### **PoC 1: UniBin** (~1 hour)

```bash
# Create new main.rs with subcommands
# Test: petaltongue ui (should launch GUI)
# Test: petaltongue headless (should start server)
# Test: petaltongue status (should show status)
```

### **PoC 2: softbuffer Display** (~2 hours)

```bash
# Add softbuffer dependency
# Create window
# Display pixels
# Verify: cargo tree | grep "\-sys" | grep -v "linux-raw-sys"
# Should be EMPTY!
```

### **PoC 3: egui Headless Rendering** (~2 hours)

```bash
# Render egui to texture
# Display texture via softbuffer
# Verify it works!
```

---

## 📊 **Effort Estimation**

### **Conservative** (Full quality):
- Phase 1 (UniBin): 4 hours
- Phase 2 (Pure Rust Display): 7 days
- Phase 3 (Optimization): 3 days
- **Total: ~10 days**

### **Aggressive** (MVP quality):
- Phase 1 (UniBin): 2 hours
- Phase 2 (Pure Rust Display, basic): 3 days
- Phase 3 (Skip for now): 0 days
- **Total: ~3 days**

---

## 🎯 **Success Criteria**

### **UniBin** ✅
- [ ] ONE binary: `petaltongue`
- [ ] Subcommands: `ui`, `headless`, `status`, `connect`
- [ ] Feature flags to minimize size
- [ ] All modes functional

### **ecoBin (ALL binaries)** ✅
- [ ] Zero C dependencies (except linux-raw-sys)
- [ ] `cargo tree | grep "\-sys" | grep -v "linux-raw-sys"` = EMPTY
- [ ] GUI works with Pure Rust display
- [ ] Builds for ARM64
- [ ] Size reasonable (<50M for UniBin)

---

## 💡 **Key Insights**

### **We Already Have the Pieces!**

1. **Framebuffer**: `petal-tongue-primitives/framebuffer.rs` ✅
2. **Software Rendering**: We render Doom this way! ✅
3. **Input Handling**: We handle keyboard/mouse for Doom! ✅
4. **Display Abstraction**: Just need Pure Rust backend! 🔨

### **The Missing Link**:

Just the **Pure Rust windowing/display layer**!

Options:
- `softbuffer`: Most mature Pure Rust option
- `minifb`: Lightweight, minimal
- Direct `/dev/fb0`: Linux-specific but pure
- `drm-rs`: Linux DRM/KMS (Pure Rust!)

---

## 🚀 **Recommended Immediate Action**

### **Step 1: Validate Approach** (~1-2 hours)

Create proof-of-concept:
1. Can we use `softbuffer` to display pixels? (Pure Rust test)
2. Can we render egui headless?
3. Can we connect them?

If YES to all 3: **Full steam ahead!** 🚀

If NO to any: **Pivot to alternative approach**

---

## 📚 **Dependencies to Research**

### **Pure Rust Display Options**:

1. **softbuffer** (v0.4+)
   - Pure Rust software rendering
   - Cross-platform
   - Active development
   - **CHECK**: Dependency tree for C deps

2. **minifb** (v0.25+)
   - Minimal framebuffer
   - Simple API
   - **CHECK**: May have platform C deps

3. **drm-rs** (Linux only)
   - Direct Rendering Manager
   - Pure Rust
   - Linux-specific but powerful
   - **CHECK**: Zero C deps?

4. **Direct `/dev/fb0`**
   - Framebuffer device access
   - `std::fs::File` + `mmap`
   - **PROS**: Definitely Pure Rust!
   - **CONS**: Linux-only, needs root or permissions

---

## 🎊 **Vision**

**Imagine**:
```bash
$ cargo tree --package petaltongue | grep "\-sys" | grep -v "linux-raw-sys"
(empty)

$ file target/release/petaltongue
petaltongue: ELF 64-bit LSB executable, statically linked

$ petaltongue --help
petaltongue - Universal UI & Visualization System

USAGE:
    petaltongue <SUBCOMMAND>

SUBCOMMANDS:
    ui         Launch GUI mode (Pure Rust!)
    headless   Run headless server
    status     Query status
    connect    Connect to instance

$ petaltongue ui
🌸 petalTongue GUI - 100% Pure Rust! 🦀
```

**THAT is TRUE UniBin + ecoBin!** 🌍

---

**Status**: 🚧 Ready to begin!  
**Philosophy**: Go beyond guidance, hand them better!  
**Ambition**: Prove Pure Rust GUI is possible!  

🚀 **Let's build the future!** 🚀

