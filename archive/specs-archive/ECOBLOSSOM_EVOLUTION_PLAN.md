# 🌸 ecoBlossom Evolution Plan - 100% Pure Rust GUI

**Version**: 2.0  
**Date**: January 19, 2026  
**Status**: 🚀 **IN PROGRESS** (Phase 1 starting)  
**Goal**: Achieve 100% Pure Rust GUI via Toadstool collaboration

---

## 📋 Document Overview

This specification defines the evolution path from **ecoBud** (80% Pure Rust) to **ecoBlossom** (100% Pure Rust GUI) through collaboration with Toadstool compute primal.

### Related Documents

- `TOADSTOOL_DISPLAY_BACKEND_REQUEST.md` - Handoff to Toadstool team
- `ECOBLOSSOM_DEEP_ANALYSIS_JAN_19_2026.md` - Gap analysis and solution
- `ECOBLOSSOM_PHASE_2_PLAN.md` - Original long-term plan
- `PURE_RUST_DISPLAY_ARCHITECTURE.md` - Display architecture spec

---

## 🎯 Vision

**ecoBlossom is petalTongue with 100% Pure Rust GUI.**

Not a separate binary—the evolutionary path for petalTongue itself.

---

## 📊 Current State: ecoBud

### UniBin Structure ✅

```
petaltongue (single binary)
├── ui        (Desktop GUI)
├── tui       (Terminal UI) ✅ Pure Rust
├── web       (Web server) ✅ Pure Rust
├── headless  (Rendering) ✅ Pure Rust
└── status    (System info) ✅ Pure Rust
```

### Purity Status: 85% Pure Rust

**Pure Rust (4/5 modes)**: ✅
- TUI: ratatui, crossterm
- Web: axum, tower-http
- Headless: Pure Rust rendering
- Status: Pure Rust system info

**Has C Dependencies (1/5 modes)**: ⚠️
- UI: egui/eframe with wayland-sys, x11rb

### The C Dependencies

```
egui (Pure Rust!) ✅
└── eframe (Pure Rust!) ✅
    └── winit (Pure Rust!) ✅
        ├── wayland-client
        │   └── wayland-sys (C FFI) ❌
        └── x11rb
            └── x11rb-protocol (C FFI) ❌
```

**Key Finding**: Rendering is Pure Rust. C dependencies are in:
1. Window management (Wayland/X11 protocols)
2. Input handling (libinput, Win32, Cocoa)

---

## 💡 The Solution: Toadstool Display Backend

### Core Insight

**Toadstool provisions compute. Why not display and input too?**

### Architecture

```
Traditional petalTongue:
egui → winit → Wayland (C) → Compositor → DRM → GPU

ecoBlossom with Toadstool:
egui → ToadstoolBackend (Pure Rust!) → Toadstool → DRM (drm-rs) → GPU
                                         ↓
                                    Input (evdev-rs)
                                    (ALL Pure Rust!)
```

### How It Works

1. **Toadstool provisions windows**
   - Uses `drm-rs` for DRM/KMS (Pure Rust!)
   - Uses `gbm-rs` for buffer allocation (Pure Rust!)
   - No Wayland/X11 needed!

2. **Toadstool provides input**
   - Uses `evdev-rs` for input devices (Pure Rust!)
   - Routes events to correct window
   - No libinput C library needed!

3. **petalTongue renders**
   - egui renders to pixels (already Pure Rust!)
   - Sends pixels to Toadstool framebuffer
   - Toadstool presents to display

4. **Result**: 100% Pure Rust! 🎉

---

## 🏗️ Architecture

### Component Diagram

```
┌─────────────────────────────────────────────────────┐
│ petalTongue (100% Pure Rust!)                       │
│ ├── egui (UI rendering)                             │
│ ├── ToadstoolBackend (window + input)               │
│ └── Business logic                                  │
└─────────────────────────────────────────────────────┘
                    ↓ RPC
┌─────────────────────────────────────────────────────┐
│ Toadstool Display Service (100% Pure Rust!)         │
│ ├── Window management (drm-rs)                      │
│ ├── Framebuffer management (gbm-rs)                 │
│ ├── Input handling (evdev-rs)                       │
│ ├── GPU management (wgpu)                           │
│ └── Multi-client support                            │
└─────────────────────────────────────────────────────┘
                    ↓ syscalls
┌─────────────────────────────────────────────────────┐
│ Linux Kernel                                         │
│ ├── DRM/KMS (display)                               │
│ ├── evdev (input)                                   │
│ └── GPU drivers                                     │
└─────────────────────────────────────────────────────┘
```

### API Design

```rust
// petalTongue side
pub struct ToadstoolBackend {
    client: ToadstoolDisplayClient,
    window: WindowId,
    event_stream: mpsc::Receiver<InputEvent>,
}

impl UIBackend for ToadstoolBackend {
    fn create_window(&mut self, width: u32, height: u32) -> Result<WindowHandle> {
        self.window = self.client.create_window(width, height).await?;
        self.event_stream = self.client.subscribe_input(self.window).await?;
        Ok(WindowHandle(self.window))
    }
    
    fn poll_events(&mut self) -> Vec<Event> {
        self.event_stream.try_recv()
            .into_iter()
            .map(|e| self.convert_event(e))
            .collect()
    }
    
    fn render(&mut self, pixels: &[u8]) -> Result<()> {
        self.client.present(self.window, pixels).await
    }
}

// Toadstool side
#[tarpc::service]
pub trait ToadstoolDisplay {
    async fn create_window(width: u32, height: u32) -> WindowId;
    async fn present(window: WindowId, pixels: Vec<u8>) -> ();
    async fn poll_events() -> Vec<InputEvent>;
    async fn destroy_window(window: WindowId) -> ();
}
```

### Feature Flags

```toml
[features]
default = ["ui-auto"]

# Auto-detect best available backend
ui-auto = []

# Toadstool backend (100% Pure Rust on Linux!)
ui-toadstool = ["toadstool-display"]

# eframe backend (fallback, has C deps)
ui-eframe = ["eframe", "winit"]

# Build commands:
# Pure Rust: cargo build --features ui-toadstool
# Fallback:  cargo build --features ui-eframe
# Auto:      cargo build --features ui-auto
```

---

## 📅 Implementation Roadmap

### Phase 1: Toadstool Display Backend (4-6 weeks)

**Owner**: Toadstool Team  
**Status**: 🚧 Starting (Handoff sent Jan 19, 2026)

#### Milestones

**Week 1-2: Core Display**
- [ ] DRM/KMS integration (drm-rs)
- [ ] Buffer allocation (gbm-rs)
- [ ] Window creation API
- [ ] Framebuffer management
- [ ] Basic pixel rendering

**Week 3-4: Input Handling**
- [ ] libinput integration (evdev-rs)
- [ ] Keyboard event routing
- [ ] Mouse event routing
- [ ] Touch event routing
- [ ] Multi-window event routing

**Week 5-6: Production Features**
- [ ] Zero-copy framebuffer sharing
- [ ] VSync support
- [ ] Multi-window support
- [ ] Window focus management
- [ ] Performance optimization
- [ ] Error handling
- [ ] Documentation

**Deliverable**: Toadstool provides display and input services (Pure Rust!)

---

### Phase 2: petalTongue Integration (2-3 weeks)

**Owner**: petalTongue Team  
**Status**: 📋 Planned

#### Milestones

**Week 1: Backend Implementation**
- [ ] Create `ToadstoolBackend` struct
- [ ] Implement `UIBackend` trait
- [ ] Connect to Toadstool display service
- [ ] Handle connection errors and fallback

**Week 2: egui Integration**
- [ ] Adapt egui rendering to Toadstool framebuffers
- [ ] Map Toadstool input events to egui events
- [ ] Handle window lifecycle (create, resize, close)
- [ ] Test full rendering pipeline

**Week 3: Feature Flags & Testing**
- [ ] Implement auto-detection (prefer Toadstool, fall back to eframe)
- [ ] Add feature flag support
- [ ] Integration tests
- [ ] Performance benchmarks
- [ ] Update documentation

**Deliverable**: petalTongue can use Toadstool for display (Pure Rust on Linux!)

---

### Phase 3: Production Hardening (2-3 weeks)

**Owner**: Both Teams  
**Status**: 📋 Planned

#### Milestones

**Week 1: Performance**
- [ ] Profile rendering pipeline
- [ ] Optimize framebuffer updates
- [ ] Reduce IPC overhead
- [ ] GPU-accelerated compositing
- [ ] 60+ FPS target

**Week 2: Stability**
- [ ] Error recovery
- [ ] Graceful degradation
- [ ] Connection retry logic
- [ ] Resource cleanup
- [ ] Memory leak prevention

**Week 3: Polish**
- [ ] Multi-monitor support
- [ ] Window decorations
- [ ] Full-screen mode
- [ ] Custom cursors
- [ ] User documentation

**Deliverable**: Production-ready Pure Rust GUI on Linux!

---

### Phase 4: Cross-Platform (3-6 months)

**Owner**: Community  
**Status**: 🔬 Research

#### Windows

**Challenge**: Win32 API is C/C++

**Options**:
1. Use `windows-rs` (Safe Rust bindings, still calls C++ APIs)
2. Wait for Pure Rust Win32 implementation
3. Keep eframe fallback for Windows (pragmatic)

**Recommendation**: Option 3 (pragmatic) for now, contribute to Option 2 long-term

#### macOS

**Challenge**: Cocoa/AppKit is Objective-C

**Options**:
1. Use `objc2` (Safe Rust bindings, still calls Obj-C APIs)
2. Wait for Pure Rust Cocoa implementation
3. Keep eframe fallback for macOS (pragmatic)

**Recommendation**: Option 3 (pragmatic) for now, contribute to Option 2 long-term

#### Mobile

**Challenge**: Android/iOS have platform-specific APIs

**Status**: Future work, not in scope for ecoBlossom Phase 1

---

## 🎯 Success Criteria

### Phase 1 Success (Toadstool)

- [ ] Can create a window with DRM/KMS
- [ ] Can write pixels to window
- [ ] Can read keyboard events via evdev
- [ ] Can read mouse events via evdev
- [ ] 100% Pure Rust (zero C dependencies)
- [ ] API is async and non-blocking
- [ ] Performance: 60+ FPS
- [ ] Multi-window support
- [ ] Documentation complete

### Phase 2 Success (petalTongue)

- [ ] petalTongue UI mode uses Toadstool backend
- [ ] All UI features work (graphs, panels, interactions)
- [ ] Auto-detection works (prefer Toadstool, fall back to eframe)
- [ ] Feature flags work correctly
- [ ] Performance meets 60 FPS target
- [ ] Integration tests pass
- [ ] User documentation updated

### Phase 3 Success (Production)

- [ ] Deployed in production environment
- [ ] Zero crashes or major bugs
- [ ] Performance metrics met
- [ ] User feedback positive
- [ ] Multi-monitor tested
- [ ] Resource usage acceptable
- [ ] Error recovery tested

### ecoBlossom Complete

- [x] Linux: 100% Pure Rust GUI ✅
- [ ] Windows: Pragmatic (eframe fallback)
- [ ] macOS: Pragmatic (eframe fallback)
- [ ] Mobile: Future work

**Linux ecoBlossom achieved = Mission accomplished!** 🎉

---

## 📊 Benefits

### Technical

1. **100% Pure Rust on Linux**
   - No C dependencies for GUI
   - Easier to maintain
   - Smaller attack surface
   - Better security

2. **TRUE PRIMAL Architecture**
   - Compute primal provisions hardware
   - UI primal focuses on representation
   - Clear separation of concerns
   - Primal collaboration demonstrated

3. **Performance**
   - Direct DRM/KMS access
   - Zero-copy rendering possible
   - GPU-accelerated compositing
   - Lower latency

4. **Cross-Platform**
   - Pure Rust compiles everywhere
   - No platform-specific C toolchains
   - Easier cross-compilation
   - Better ARM64/RISC-V support

### Ecosystem

1. **Sets Precedent**
   - Shows Pure Rust GUI is possible
   - Demonstrates primal collaboration
   - Validates TRUE PRIMAL architecture
   - Inspires other teams

2. **Reduces Complexity**
   - One less set of dependencies
   - Simpler build process
   - Fewer failure modes
   - Easier debugging

3. **Future-Proof**
   - Not tied to Wayland/X11 evolution
   - Can adapt to new display tech
   - Pure Rust evolves faster
   - Community-driven improvements

---

## 🔬 Technical Considerations

### Permissions

**Issue**: DRM/KMS and evdev require elevated permissions

**Solutions**:
1. **logind/elogind**: Session management (preferred)
2. **libseat**: Seat management library
3. **User in `video` group**: Simple but less secure
4. **Wayland/X11 fallback**: For non-privileged contexts

**Recommendation**: Use libseat for session management

### Multi-User

**Issue**: Multiple users running GUIs simultaneously

**Solution**: Toadstool manages per-user sessions, routes display and input correctly

### GPU Sharing

**Issue**: Multiple apps want GPU access

**Solution**: Toadstool is already compute primal, manages GPU resources

### Error Handling

**Issue**: What if Toadstool crashes or is unavailable?

**Solution**: 
- Graceful fallback to eframe
- Auto-restart of Toadstool service
- Error reporting to user
- Connection retry logic

---

## 🧪 Testing Strategy

### Unit Tests

- [ ] Window creation/destruction
- [ ] Framebuffer operations
- [ ] Input event routing
- [ ] Multi-window management
- [ ] Error conditions

### Integration Tests

- [ ] petalTongue → Toadstool communication
- [ ] Full rendering pipeline
- [ ] Input handling end-to-end
- [ ] Feature flag switching
- [ ] Fallback behavior

### Performance Tests

- [ ] Rendering FPS (target: 60+)
- [ ] Input latency (target: <16ms)
- [ ] Memory usage
- [ ] CPU usage
- [ ] GPU utilization

### Platform Tests

- [ ] Different Linux distros
- [ ] Different GPU vendors (Intel, AMD, NVIDIA)
- [ ] Different input devices
- [ ] Multi-monitor setups
- [ ] High DPI displays

---

## 📚 Reference Implementation

### Minimal Example

```rust
// Toadstool side
pub struct ToadstoolDisplay {
    drm: drm::Device,
    gbm: gbm::Device<drm::Device>,
    input: input::Libinput,
    windows: HashMap<WindowId, Window>,
}

// petalTongue side
pub struct ToadstoolBackend {
    client: ToadstoolDisplayClient,
    window: WindowId,
}

// Usage
let mut backend = ToadstoolBackend::new().await?;
let window = backend.create_window(1920, 1080)?;

loop {
    // Render
    let pixels = egui_ctx.render();
    backend.present(window, &pixels)?;
    
    // Input
    for event in backend.poll_events()? {
        egui_ctx.handle_event(event);
    }
}
```

---

## 🤝 Team Responsibilities

### Toadstool Team

- [ ] Implement display backend (Phase 1)
- [ ] Provide API documentation
- [ ] Performance optimization
- [ ] Bug fixes and support
- [ ] Answer integration questions

### petalTongue Team

- [ ] Integrate Toadstool backend (Phase 2)
- [ ] Testing and validation
- [ ] User documentation
- [ ] Feature flag management
- [ ] Production deployment

### Shared

- [ ] Architecture decisions
- [ ] API design
- [ ] Performance targets
- [ ] Error handling strategy
- [ ] Documentation standards

---

## 📞 Communication

### Channels

- **GitHub Issues**: Track implementation tasks
- **Discord #toadstool-display**: Real-time questions
- **Bi-weekly Sync**: 30min video call
- **Code Reviews**: Cross-team PR reviews
- **Documentation**: Shared specs and guides

### Status Updates

- **Weekly**: Progress summary in Discord
- **Bi-weekly**: Sync call with both teams
- **Monthly**: Written status report
- **Milestones**: Announcement and demo

---

## 🎉 Conclusion

ecoBlossom represents the evolution of petalTongue to 100% Pure Rust GUI through collaboration with Toadstool compute primal.

**Phase 1** (4-6 weeks): Toadstool implements display backend  
**Phase 2** (2-3 weeks): petalTongue integrates Toadstool backend  
**Phase 3** (2-3 weeks): Production hardening  
**Result**: 100% Pure Rust GUI on Linux! 🎉

**Timeline**: 8-12 weeks to ecoBlossom Phase 1 complete!

---

## 📋 Appendix

### A. Glossary

- **ecoBud**: Current petalTongue (85% Pure Rust)
- **ecoBlossom**: Future petalTongue (100% Pure Rust GUI)
- **DRM/KMS**: Direct Rendering Manager / Kernel Mode Setting
- **evdev**: Event device (kernel input subsystem)
- **GBM**: Generic Buffer Manager
- **TRUE PRIMAL**: Architecture principle (self-knowledge, zero hardcoding)

### B. Related Documents

- `TOADSTOOL_DISPLAY_BACKEND_REQUEST.md`
- `ECOBLOSSOM_DEEP_ANALYSIS_JAN_19_2026.md`
- `PURE_RUST_DISPLAY_ARCHITECTURE.md`
- `ECOBLOSSOM_PHASE_2_PLAN.md`

### C. External Resources

- [drm-rs documentation](https://docs.rs/drm/)
- [gbm-rs documentation](https://docs.rs/gbm/)
- [input-rs documentation](https://docs.rs/input/)
- [Smithay compositor](https://github.com/Smithay/smithay)
- [Cosmic desktop](https://github.com/pop-os/cosmic)

---

**Version**: 2.0  
**Date**: January 19, 2026  
**Status**: Ready for Implementation  
**Next Review**: After Phase 1 completion

🌸🍄 **ecoBlossom: petalTongue + Toadstool = 100% Pure Rust GUI!** 🍄🌸

