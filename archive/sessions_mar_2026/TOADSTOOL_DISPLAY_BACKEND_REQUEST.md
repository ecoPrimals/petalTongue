# 🍄 Toadstool Display Backend - Request for Collaboration

**From**: petalTongue Team  
**To**: Toadstool Team  
**Date**: January 19, 2026  
**Priority**: HIGH  
**Goal**: Enable 100% Pure Rust GUI via Toadstool

---

## 🎯 Executive Summary

**Request**: Extend Toadstool to provide display and input services, enabling petalTongue to achieve 100% Pure Rust GUI on Linux.

**Why**: Current GUI depends on C libraries (wayland-sys, x11rb) for window management and input. By having Toadstool provision these via Pure Rust (drm-rs, evdev-rs), we eliminate all C dependencies.

**Timeline**: 4-6 weeks for Phase 1  
**Impact**: Enables 100% Pure Rust GUI for entire ecoPrimals ecosystem  
**Benefit**: TRUE PRIMAL architecture - hardware provisioning by compute primal

---

## 🔍 The Problem

### Current petalTongue GUI Stack

```
egui (Pure Rust!) ✅
└── eframe (Pure Rust!) ✅
    └── winit (Pure Rust!) ✅
        ├── wayland-client
        │   └── wayland-sys (C FFI) ❌
        └── x11rb
            └── x11rb-protocol (C FFI) ❌
```

**The Issue**: egui rendering is Pure Rust, but window creation and input handling require C libraries for display server protocols (Wayland/X11).

### Why This Matters

1. **ecoBin Compliance**: We want 100% Pure Rust for server/automation deployments
2. **Cross-Platform**: Pure Rust compiles everywhere without C toolchains
3. **TRUE PRIMAL**: Compute primal should provision hardware, not UI primal
4. **Security**: Reduce attack surface by eliminating C dependencies
5. **Maintainability**: Pure Rust is easier to maintain and evolve

---

## 💡 The Solution: Toadstool Display Backend

### Core Insight

**Toadstool already provisions GPU compute. Why not display and input too?**

If Toadstool can:
- Create framebuffers (already does via GPU)
- Handle DRM/KMS (kernel display interface)
- Route input events (keyboard, mouse, touch)

Then petalTongue can:
- Render to Toadstool framebuffers (Pure Rust egui)
- Get input from Toadstool (Pure Rust events)
- Eliminate all display server dependencies!

### Architecture

```
Traditional:
petalTongue → winit → Wayland (C) → Compositor → DRM → GPU

With Toadstool:
petalTongue → Toadstool (Pure Rust!) → DRM (drm-rs) → GPU
              ↓
         Input (evdev-rs)
         (ALL Pure Rust!)
```

---

## 📋 What We Need from Toadstool

### Phase 1: Display Backend API (2-3 weeks)

**Goal**: Toadstool can create windows, handle input, provide framebuffers

#### 1. Window Management

```rust
/// Create a window/surface
pub async fn create_window(
    &self,
    width: u32,
    height: u32,
    title: Option<String>,
) -> Result<WindowId>;

/// Resize window
pub async fn resize_window(
    &self,
    window: WindowId,
    width: u32,
    height: u32,
) -> Result<()>;

/// Destroy window
pub async fn destroy_window(&self, window: WindowId) -> Result<()>;

/// Get window properties
pub async fn window_info(&self, window: WindowId) -> Result<WindowInfo>;

pub struct WindowInfo {
    pub width: u32,
    pub height: u32,
    pub scale_factor: f32,
    pub focused: bool,
}
```

**Implementation Notes**:
- Use `drm-rs` for DRM/KMS access
- Use `gbm-rs` for buffer allocation
- Each window = one framebuffer
- Support multiple windows (multi-client)

#### 2. Framebuffer Management

```rust
/// Get framebuffer for window
pub async fn get_framebuffer(
    &self,
    window: WindowId,
) -> Result<Framebuffer>;

/// Present pixels to window
pub async fn present(
    &self,
    window: WindowId,
    pixels: &[u8],  // RGBA8888 format
) -> Result<()>;

pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub format: PixelFormat,
}

pub enum PixelFormat {
    RGBA8888,
    BGRA8888,
    RGB888,
}
```

**Implementation Notes**:
- Zero-copy if possible (shared memory)
- Double-buffering for smooth rendering
- VSync support

#### 3. Input Handling

```rust
/// Poll input events
pub async fn poll_events(&self) -> Result<Vec<InputEvent>>;

/// Subscribe to input stream
pub fn subscribe_input(&self) -> mpsc::Receiver<InputEvent>;

#[derive(Debug, Clone)]
pub enum InputEvent {
    /// Keyboard events
    KeyPress {
        key: KeyCode,
        modifiers: Modifiers,
        window: WindowId,
    },
    KeyRelease {
        key: KeyCode,
        modifiers: Modifiers,
        window: WindowId,
    },
    
    /// Mouse events
    MouseMove {
        x: i32,
        y: i32,
        window: WindowId,
    },
    MouseButton {
        button: MouseButton,
        pressed: bool,
        x: i32,
        y: i32,
        window: WindowId,
    },
    MouseWheel {
        delta_x: f32,
        delta_y: f32,
        window: WindowId,
    },
    
    /// Touch events
    Touch {
        id: u32,
        phase: TouchPhase,
        x: i32,
        y: i32,
        window: WindowId,
    },
    
    /// Window events
    WindowFocused { window: WindowId },
    WindowUnfocused { window: WindowId },
    WindowResized { window: WindowId, width: u32, height: u32 },
    WindowClosed { window: WindowId },
}

pub enum KeyCode {
    // Standard keys (same as winit::event::VirtualKeyCode)
    A, B, C, // ... etc
    Escape, Return, Space,
    Left, Right, Up, Down,
    // ... etc
}

pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub logo: bool,
}

pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u8),
}

pub enum TouchPhase {
    Started,
    Moved,
    Ended,
    Cancelled,
}
```

**Implementation Notes**:
- Use `evdev-rs` for input device access
- Route events to correct window
- Support multiple input devices
- Handle hotplugging

#### 4. Capability Discovery

```rust
/// Check if display backend is available
pub async fn is_display_available(&self) -> bool;

/// Get display capabilities
pub async fn display_capabilities(&self) -> Result<DisplayCapabilities>;

pub struct DisplayCapabilities {
    pub max_windows: usize,
    pub supported_formats: Vec<PixelFormat>,
    pub has_vsync: bool,
    pub has_hardware_cursor: bool,
    pub input_devices: Vec<InputDevice>,
}

pub struct InputDevice {
    pub name: String,
    pub device_type: InputDeviceType,
    pub capabilities: Vec<InputCapability>,
}

pub enum InputDeviceType {
    Keyboard,
    Mouse,
    Touchscreen,
    Touchpad,
    Other,
}

pub enum InputCapability {
    Keys,
    RelativePointer,
    AbsolutePointer,
    MultiTouch,
    Scroll,
}
```

---

### Phase 2: Production Features (2-3 weeks)

#### 1. Performance Optimizations

- **Zero-copy rendering**: Shared memory for framebuffers
- **GPU acceleration**: Use existing Toadstool GPU for compositing
- **VSync**: Frame-perfect timing
- **Event batching**: Reduce IPC overhead

#### 2. Multi-Window Support

- **Window focus management**: Track active window
- **Z-ordering**: Window stacking
- **Window decorations**: Optional title bars
- **Full-screen mode**: Exclusive display access

#### 3. Advanced Input

- **Input focus**: Route input to focused window
- **Input grab**: Exclusive input for games
- **Custom cursors**: Application-defined cursors
- **Clipboard**: Copy/paste support

---

## 🛠️ Implementation Guide

### Recommended Crates

```toml
[dependencies]
# Display management (Pure Rust!)
drm = "0.12"           # DRM/KMS bindings
gbm = "0.14"           # Buffer allocation
libseat = "0.2"        # Session management

# Input handling (Pure Rust!)
input = "0.9"          # evdev wrapper
calloop = "0.13"       # Event loop

# GPU (already in Toadstool)
wgpu = "0.19"          # GPU abstraction

# IPC (for petalTongue communication)
tarpc = { version = "0.34", features = ["tokio1"] }
tokio = { version = "1", features = ["full"] }
```

### Minimal Example

```rust
use drm::control::{Device, ResourceHandle};
use gbm::{BufferObjectFlags, Format};
use input::{Libinput, LibinputInterface};

pub struct ToadstoolDisplay {
    drm: drm::Device,
    gbm: gbm::Device<drm::Device>,
    input: Libinput,
    windows: HashMap<WindowId, Window>,
}

impl ToadstoolDisplay {
    pub fn new() -> Result<Self> {
        // Open DRM device
        let drm = drm::Device::open("/dev/dri/card0")?;
        
        // Create GBM device
        let gbm = gbm::Device::new(drm.clone())?;
        
        // Initialize libinput
        let mut input = Libinput::new_with_udev(Interface)?;
        input.udev_assign_seat("seat0")?;
        
        Ok(Self {
            drm,
            gbm,
            input,
            windows: HashMap::new(),
        })
    }
    
    pub fn create_window(&mut self, width: u32, height: u32) -> Result<WindowId> {
        // Allocate framebuffer
        let bo = self.gbm.create_buffer_object::<()>(
            width,
            height,
            Format::ARGB8888,
            BufferObjectFlags::SCANOUT | BufferObjectFlags::RENDERING,
        )?;
        
        // Create DRM framebuffer
        let fb = self.drm.add_framebuffer(&bo, 32, 32)?;
        
        // Store window
        let id = WindowId::new();
        self.windows.insert(id, Window { bo, fb, width, height });
        
        Ok(id)
    }
    
    pub fn present(&mut self, window: WindowId, pixels: &[u8]) -> Result<()> {
        let window = self.windows.get_mut(&window)?;
        
        // Write pixels to buffer object
        let mut map = window.bo.map(&self.gbm)?;
        map.as_mut().copy_from_slice(pixels);
        drop(map);
        
        // Present to display
        self.drm.page_flip(window.fb)?;
        
        Ok(())
    }
    
    pub fn poll_events(&mut self) -> Result<Vec<InputEvent>> {
        let mut events = Vec::new();
        
        // Process libinput events
        self.input.dispatch()?;
        for event in &mut self.input {
            match event {
                input::Event::Keyboard(e) => {
                    events.push(InputEvent::KeyPress {
                        key: e.key(),
                        // ... etc
                    });
                }
                input::Event::Pointer(e) => {
                    events.push(InputEvent::MouseMove {
                        x: e.x() as i32,
                        y: e.y() as i32,
                        // ... etc
                    });
                }
                // ... etc
            }
        }
        
        Ok(events)
    }
}
```

---

## 🔌 Integration with petalTongue

### petalTongue Side

```rust
// petalTongue will implement this backend:
pub struct ToadstoolBackend {
    client: ToadstoolDisplayClient,
    window: WindowId,
}

impl UIBackend for ToadstoolBackend {
    fn create_window(&mut self, width: u32, height: u32) -> Result<WindowHandle> {
        self.window = self.client.create_window(width, height).await?;
        Ok(WindowHandle(self.window))
    }
    
    fn poll_events(&mut self) -> Vec<Event> {
        self.client.poll_events().await
            .unwrap_or_default()
            .into_iter()
            .map(|e| self.convert_event(e))
            .collect()
    }
    
    fn render(&mut self, pixels: &[u8]) -> Result<()> {
        self.client.present(self.window, pixels).await
    }
}
```

### Feature Flags

```toml
# petalTongue Cargo.toml
[features]
default = ["ui-toadstool"]  # Prefer Toadstool!

ui-toadstool = ["toadstool-display"]  # Pure Rust via Toadstool
ui-eframe = ["eframe", "winit"]       # Fallback to eframe

# Build for Pure Rust:
# cargo build --features ui-toadstool

# Build with fallback:
# cargo build --features ui-eframe
```

---

## 📊 Benefits

### For ecoPrimals Ecosystem

1. **100% Pure Rust GUI** on Linux
2. **TRUE PRIMAL Architecture** - compute primal provisions hardware
3. **Reduced Dependencies** - eliminate wayland-sys, x11rb
4. **Better Security** - smaller attack surface
5. **Easier Maintenance** - all Rust, no C FFI
6. **Faster Evolution** - no waiting for upstream C libraries

### For Toadstool

1. **Expanded Scope** - from compute-only to compute + display
2. **Broader Use Cases** - any GUI app can use Toadstool backend
3. **Performance Wins** - direct GPU access, zero-copy rendering
4. **Showcase Architecture** - demonstrates primal collaboration
5. **Community Impact** - enables Pure Rust ecosystem

### For Users

1. **Better Performance** - direct hardware access
2. **More Reliability** - fewer dependencies = fewer bugs
3. **Wider Platform Support** - Pure Rust compiles everywhere
4. **Future-Proof** - not tied to specific display servers

---

## 🎯 Success Criteria

### Phase 1 (Minimum Viable)

- [ ] Can create a window
- [ ] Can write pixels to window
- [ ] Can read keyboard events
- [ ] Can read mouse events
- [ ] Can destroy window
- [ ] Works on Linux with DRM/KMS
- [ ] API is async and non-blocking
- [ ] 100% Pure Rust (no C dependencies)

### Phase 2 (Production Ready)

- [ ] Zero-copy framebuffer sharing
- [ ] Multi-window support
- [ ] Window focus management
- [ ] Touch input support
- [ ] VSync support
- [ ] Performance benchmarks (60+ FPS)
- [ ] Error handling and recovery
- [ ] Documentation and examples

---

## 📅 Proposed Timeline

### Week 1-2: Core Display

- DRM/KMS integration (drm-rs)
- Buffer allocation (gbm-rs)
- Window creation
- Framebuffer management
- Basic rendering (write pixels)

### Week 3-4: Input Handling

- libinput integration (evdev-rs)
- Keyboard events
- Mouse events
- Touch events
- Event routing to windows

### Week 5-6: Integration & Testing

- petalTongue integration
- End-to-end testing
- Performance optimization
- Documentation
- Example applications

**Total: 6 weeks to production-ready display backend!**

---

## 🤝 Collaboration

### What petalTongue Team Provides

1. **Requirements** - this document!
2. **Testing** - integration testing with petalTongue
3. **Feedback** - API design feedback
4. **Documentation** - user-facing docs for petalTongue side
5. **Example Apps** - showcase applications

### What We Need from Toadstool Team

1. **API Implementation** - display backend as specified
2. **Performance** - optimize for low latency
3. **Stability** - production-grade error handling
4. **Documentation** - API docs and integration guide
5. **Support** - answer questions during integration

### Communication

- **Slack/Discord**: Real-time questions
- **GitHub Issues**: Track implementation progress
- **Bi-weekly Sync**: 30min video call to align
- **Code Reviews**: Cross-team review of PRs

---

## 📚 Reference Materials

### Existing Specs

- `specs/PURE_RUST_DISPLAY_ARCHITECTURE.md` - petalTongue display strategy
- `specs/ECOBLOSSOM_EVOLUTION_PLAN.md` - Long-term Pure Rust GUI plan
- `ECOBLOSSOM_DEEP_ANALYSIS_JAN_19_2026.md` - Gap analysis

### Similar Projects

- **Smithay** - Wayland compositor in Rust (uses similar architecture)
- **Cosmic** - System76's Pure Rust desktop (drm-rs + smithay)
- **Alacritty** - Terminal with Pure Rust GPU rendering

### Technical Resources

- [drm-rs documentation](https://docs.rs/drm/)
- [gbm-rs documentation](https://docs.rs/gbm/)
- [input-rs documentation](https://docs.rs/input/)
- [DRM/KMS kernel docs](https://www.kernel.org/doc/html/latest/gpu/drm-kms.html)

---

## 🚀 Next Steps

1. **Review this document** with Toadstool team
2. **Align on timeline** and milestones
3. **Create tracking issues** in Toadstool repo
4. **Start with proof of concept** - simple window + input
5. **Iterate and integrate** with petalTongue

---

## 📞 Contact

**petalTongue Team**:
- Lead: [Your Name]
- Email: [Your Email]
- GitHub: [petalTongue repo]
- Discord: [ecoPrimals Discord]

**Questions?**
- Open GitHub issue in Toadstool repo tagged `display-backend`
- Ping in #toadstool-display Discord channel
- Email team directly for urgent items

---

## 🎉 Conclusion

This is an exciting evolution for both teams!

**For Toadstool**: Expanding from pure compute to compute + display + input = universal hardware provisioning primal

**For petalTongue**: Achieving 100% Pure Rust GUI = TRUE PRIMAL architecture realized

**For ecoPrimals**: Demonstrating primal collaboration = living the vision

Let's make this happen! 🍄🌸

---

**Document Version**: 1.0  
**Date**: January 19, 2026  
**Status**: Ready for Toadstool Team Review  
**Priority**: HIGH

🌸🍄 **petalTongue + Toadstool = 100% Pure Rust GUI!** 🍄🌸

