# 🎮 Doom as Evolution Test - Exposing petalTongue's Gaps

**Date**: January 15, 2026  
**Approach**: Test-Driven Evolution  
**Philosophy**: Real use cases expose real gaps

---

## 🎯 The Philosophy

### **Traditional Approach** ❌:
1. Speculate about features we might need
2. Build abstract frameworks
3. Hope they work for real use cases

### **TRUE PRIMAL Approach** ✅:
1. Pick a real, demanding use case (Doom)
2. Try to implement it
3. Let the gaps reveal themselves
4. Evolve to fill those gaps
5. Document the lessons learned

> **"Doom isn't just a demo. It's a stress test."**

---

## 🔍 What Doom Will Expose

### **Known Gaps We'll Hit**:

#### **1. Panel Lifecycle Management**
```rust
// Current: No clear lifecycle
struct DoomPanel { ... }

// What we need:
trait PanelLifecycle {
    fn init(&mut self);           // Setup
    fn update(&mut self, dt: f32); // Game tick
    fn render(&mut self, ui: &mut Ui); // Display
    fn cleanup(&mut self);         // Shutdown
    fn pause(&mut self);           // Backgrounded
    fn resume(&mut self);          // Foregrounded
}
```

**Why it matters**: Games need precise control over when they run.  
**Lesson**: All embedded panels will need this.

---

#### **2. Input Focus & Priority**
```rust
// Current: All panels get all input
// Problem: Doom needs exclusive input when focused

// What we need:
struct InputRouter {
    focused_panel: Option<PanelId>,
    modal_stack: Vec<PanelId>,
}

impl InputRouter {
    fn route_input(&self, event: InputEvent) -> Option<PanelId> {
        if let Some(modal) = self.modal_stack.last() {
            return Some(*modal);  // Modal gets priority
        }
        self.focused_panel
    }
}
```

**Why it matters**: Can't have Doom AND graph both responding to keyboard.  
**Lesson**: Input routing is critical for any multi-panel UI.

---

#### **3. Performance Budget & Frame Timing**
```rust
// Current: egui requests repaints, we respond
// Problem: Doom expects 35 FPS game tick, 60 FPS render

// What we need:
struct PerformanceBudget {
    target_fps: u32,
    max_frame_time: Duration,
    panel_budgets: HashMap<PanelId, Duration>,
}

impl PerformanceBudget {
    fn allocate(&self, panel: PanelId) -> Duration {
        // Each panel gets a time budget per frame
        // Doom gets 28ms (35 Hz), Graph gets 16ms (60 Hz)
    }
    
    fn measure_and_warn(&self, panel: PanelId, elapsed: Duration) {
        if elapsed > self.panel_budgets[&panel] {
            tracing::warn!("Panel {panel:?} exceeded budget: {elapsed:?}");
        }
    }
}
```

**Why it matters**: Mixed-framerate panels, performance visibility.  
**Lesson**: Need observable performance budgets.

---

#### **4. Shared Resources (GPU, Audio, Memory)**
```rust
// Current: Each system grabs what it needs
// Problem: Doom + Graph + Audio all want GPU

// What we need:
struct ResourceCoordinator {
    gpu_budget: Arc<Semaphore>,      // 100% available
    audio_channels: Arc<Semaphore>,  // 32 channels
    memory_budget: usize,            // 512 MB
}

impl ResourceCoordinator {
    async fn request_gpu(&self, panel: PanelId, percent: f32) -> Result<GpuSlice> {
        // Doom requests 70% GPU for rendering
        // Graph gets remaining 30%
    }
    
    async fn request_audio_channel(&self) -> Result<AudioChannel> {
        // First come, first served, up to limit
    }
}
```

**Why it matters**: Can't have every panel using 100% GPU.  
**Lesson**: Explicit resource coordination.

---

#### **5. Asset Loading & Caching**
```rust
// Current: No asset system
// Problem: Doom needs doom1.wad (2.5 MB), textures, sprites, sounds

// What we need:
struct AssetManager {
    cache: LruCache<AssetId, Arc<Asset>>,
    loader: AssetLoader,
}

impl AssetManager {
    async fn load<T: Asset>(&self, path: &str) -> Result<Arc<T>> {
        // Check cache first
        // Load asynchronously if not present
        // Store in cache
        // Return Arc for zero-copy sharing
    }
    
    fn preload(&self, paths: &[&str]) {
        // Load assets in background before panel starts
    }
}
```

**Why it matters**: Don't want to freeze UI while loading 2.5 MB WAD.  
**Lesson**: Async asset loading is essential.

---

#### **6. Audio Mixing**
```rust
// Current: petal-tongue-audio exists but not integrated
// Problem: Doom produces raw PCM samples, needs mixing with other panels

// What we need:
struct AudioMixer {
    channels: Vec<AudioChannel>,
    master_volume: f32,
}

impl AudioMixer {
    fn mix(&mut self, output: &mut [f32]) {
        // Mix all active channels
        // Apply volume
        // Clipping protection
        // Output to device
    }
    
    fn add_channel(&mut self, source: AudioSource) -> ChannelId {
        // Doom creates channel for game audio
        // Graph creates channel for sonification
        // Both mixed together
    }
}
```

**Why it matters**: Multiple panels producing audio simultaneously.  
**Lesson**: Audio is a first-class coordinated resource.

---

#### **7. Error Recovery & Graceful Degradation**
```rust
// Current: Panels crash, app crashes
// Problem: If Doom crashes, should petalTongue survive?

// What we need:
struct PanelSandbox {
    panel: Box<dyn Panel>,
    error_state: Option<Error>,
}

impl PanelSandbox {
    fn update(&mut self) {
        match std::panic::catch_unwind(AssertUnwindSafe(|| {
            self.panel.update();
        })) {
            Ok(_) => { /* success */ }
            Err(e) => {
                self.error_state = Some(format!("{:?}", e));
                tracing::error!("Panel panicked: {:?}", e);
                // Show error UI instead of crashing
            }
        }
    }
}
```

**Why it matters**: Doom bug shouldn't kill entire UI.  
**Lesson**: Panel isolation and error boundaries.

---

#### **8. State Persistence**
```rust
// Current: LocalStatePersistence exists but not used
// Problem: User plays Doom, closes petalTongue, wants to resume

// What we need:
trait PanelState: Serialize + Deserialize {
    fn save(&self) -> Result<Vec<u8>>;
    fn load(data: &[u8]) -> Result<Self>;
}

impl DoomPanel {
    fn save_state(&self) -> DoomState {
        DoomState {
            level: self.current_level,
            health: self.player_health,
            ammo: self.ammo,
            position: self.player_pos,
            // Full save state
        }
    }
}
```

**Why it matters**: User expectations (save/load).  
**Lesson**: Panels need serializable state.

---

#### **9. Multi-Window & Display Scaling**
```rust
// Current: Single window, desktop resolution
// Problem: User wants Doom fullscreen on monitor 2, graph on monitor 1

// What we need:
struct DisplayManager {
    windows: HashMap<WindowId, Window>,
    panels_to_windows: HashMap<PanelId, WindowId>,
}

impl DisplayManager {
    fn move_panel_to_window(&mut self, panel: PanelId, window: WindowId) {
        // Doom goes fullscreen on gaming monitor
        // Graph stays on primary monitor
    }
    
    fn handle_scaling(&self, window: WindowId) -> f32 {
        // 4K display at 200% scaling
        // 1080p display at 100% scaling
    }
}
```

**Why it matters**: Professional workflows use multi-monitor.  
**Lesson**: Window management is complex.

---

#### **10. Input Abstraction**
```rust
// Current: egui events directly
// Problem: Doom expects game controller, keyboard, mouse - different APIs

// What we need:
enum UnifiedInput {
    Keyboard(KeyCode, bool),     // Key, pressed
    Mouse(MouseButton, bool),    // Button, pressed
    MouseMove(f32, f32),         // Delta x, y
    GameController(ControllerButton, f32), // Button, value
    Touch(TouchPhase, Vec2),     // Phase, position
}

trait InputMapper {
    fn map(&self, input: UnifiedInput) -> Option<GameInput>;
}

// User can remap: WASD → arrows, Space → jump, etc.
```

**Why it matters**: Different input paradigms (keyboard vs controller vs touch).  
**Lesson**: Input abstraction layer needed.

---

## 📊 Gap Matrix

| Gap | Doom Exposes? | Priority | Complexity |
|-----|---------------|----------|------------|
| Panel Lifecycle | ✅ Yes | High | Low |
| Input Focus | ✅ Yes | High | Medium |
| Performance Budget | ✅ Yes | Medium | Medium |
| Resource Coordination | ✅ Yes | High | High |
| Asset Loading | ✅ Yes | High | Medium |
| Audio Mixing | ✅ Yes | Medium | Medium |
| Error Recovery | ⚠️ Maybe | Medium | Low |
| State Persistence | ⚠️ Maybe | Low | Low |
| Multi-Window | ❌ No | Low | High |
| Input Abstraction | ✅ Yes | Medium | Medium |

---

## 🎓 Lessons from Game HUDs

### **What Game Developers Solved** (that we can learn from):

#### **1. Immediate Mode UI** (Dear ImGui pattern)
```rust
// Games discovered: Retained mode is too slow
// Solution: Rebuild UI every frame (egui does this!)

// We're already doing this right! ✅
```

#### **2. Hot Reloading**
```rust
// Games discovered: Recompiling for UI tweaks is painful
// Solution: Load UI from scripts/JSON, hot-reload on change

// We do this with scenarios! ✅
```

#### **3. Performance Overlays**
```rust
// Games discovered: Need to see FPS, frame time, GPU usage
// Solution: Built-in performance HUD

// What we need:
struct PerformanceOverlay {
    fps: f32,
    frame_time: Duration,
    gpu_usage: f32,
    memory_usage: usize,
    panel_timings: HashMap<PanelId, Duration>,
}

// Press F3 → Show performance overlay
```

#### **4. Console / Debug UI**
```rust
// Games discovered: Need runtime introspection
// Solution: Developer console (~ key)

// What we need:
struct DebugConsole {
    commands: HashMap<String, Box<dyn Fn(&[String])>>,
    history: Vec<String>,
    output: Vec<String>,
}

// Type: "panel.doom.health 100" → Set player health
// Type: "panel.list" → Show all panels
// Type: "resource.gpu" → Show GPU allocation
```

#### **5. Layout Templates**
```rust
// Games discovered: Different layouts for different situations
// Solution: Layout presets

// HUD layouts we can learn from:
- "fullscreen"    → One panel, no chrome
- "split-screen"  → Two panels, 50/50
- "pip"           → Main + small overlay
- "dashboard"     → Multiple small panels
- "theater"       → Video centered, controls bottom

// JSON scenario defines layout ✅
```

#### **6. Input Contexts**
```rust
// Games discovered: Different screens need different inputs
// Solution: Input context stack

enum InputContext {
    Menu,          // Arrow keys, Enter, Escape
    Gameplay,      // WASD, Mouse, Space
    Inventory,     // Click, Drag, Drop
    Console,       // Text input
}

// When console opens, push Console context
// Gameplay input ignored until Console pops
```

#### **7. Audio Ducking**
```rust
// Games discovered: Multiple audio sources clash
// Solution: Priority-based mixing

struct AudioPriority {
    dialog: 1.0,      // Always full volume
    sfx: 0.8,         // Slightly reduced
    music: 0.5,       // Background
    ambient: 0.3,     // Very quiet
}

// When dialog plays, music ducks to 20%
// When dialog ends, music fades back to 50%
```

#### **8. Frame Pacing**
```rust
// Games discovered: Variable frame time causes judder
// Solution: Frame pacing and v-sync

struct FramePacer {
    target_frame_time: Duration,
    last_frame: Instant,
}

impl FramePacer {
    fn wait_for_next_frame(&mut self) {
        let elapsed = self.last_frame.elapsed();
        if elapsed < self.target_frame_time {
            std::thread::sleep(self.target_frame_time - elapsed);
        }
        self.last_frame = Instant::now();
    }
}
```

---

## 🚀 Evolution Roadmap (Driven by Doom)

### **Phase 1: Minimal Viable Doom** (Week 1)
**Goal**: Get Doom running, even if janky

- [ ] Embed doomgeneric
- [ ] Basic framebuffer rendering
- [ ] Keyboard input routing
- [ ] Mouse input routing
- [ ] Basic panel lifecycle

**Gaps Exposed**: Input focus, lifecycle management

---

### **Phase 2: Polish & Performance** (Week 2)
**Goal**: Make it smooth and professional

- [ ] Frame timing (35 Hz game, 60 Hz render)
- [ ] Input focus system
- [ ] Performance overlay (F3 key)
- [ ] Audio output
- [ ] Asset loading (doom1.wad)

**Gaps Exposed**: Performance budgets, audio mixing, asset management

---

### **Phase 3: Multi-Panel Composition** (Week 3)
**Goal**: Doom + other panels simultaneously

- [ ] Split-screen scenario (Doom + Graph)
- [ ] Resource coordination (GPU budget)
- [ ] Audio mixing (Doom + sonification)
- [ ] Input routing (which panel is focused?)
- [ ] Panel priorities

**Gaps Exposed**: Resource coordination, audio mixing

---

### **Phase 4: Advanced Features** (Week 4)
**Goal**: Professional-grade features

- [ ] Save/load game state
- [ ] Developer console
- [ ] Error boundaries (Doom crash ≠ app crash)
- [ ] Layout presets
- [ ] Input remapping

**Gaps Exposed**: State persistence, error recovery, configurability

---

## 📝 Documentation as We Go

### **For Each Gap Discovered**:

1. **Document the Problem**:
   ```markdown
   ## Gap: Input Focus System
   
   **Discovered**: Day 2 of Doom implementation
   **Problem**: Both Doom and Graph respond to keyboard
   **Impact**: User can't play game, conflicting inputs
   ```

2. **Design the Solution**:
   ```markdown
   ## Solution: Input Router with Focus
   
   **Design**: Panel-based focus system
   **Implementation**: InputRouter struct
   **API**: `panel.request_focus()`, `panel.has_focus()`
   ```

3. **Implement & Test**:
   ```rust
   // Implementation in petal-tongue-core/input_router.rs
   #[test]
   fn test_input_focus() { ... }
   ```

4. **Document the Lesson**:
   ```markdown
   ## Lesson Learned
   
   Input focus is critical for multi-panel UIs.
   Games taught us: Use stack-based context system.
   We evolved: InputRouter with priority queue.
   ```

---

## 🎯 Success Criteria

### **Technical**:
- [ ] Doom runs at 35 FPS consistently
- [ ] Input only goes to focused panel
- [ ] Audio plays without glitches
- [ ] Multiple panels work simultaneously
- [ ] No crashes or memory leaks
- [ ] Performance overlay shows budgets

### **Architectural**:
- [ ] Panel lifecycle system designed
- [ ] Input routing system designed
- [ ] Resource coordination system designed
- [ ] Audio mixing system designed
- [ ] Asset loading system designed
- [ ] All gaps documented

### **Philosophical** (TRUE PRIMAL):
- [ ] Real use case drove design (not speculation)
- [ ] Gaps discovered organically
- [ ] Solutions evolved from needs
- [ ] Lessons documented for future
- [ ] Code remains flexible and composable

---

## 🌟 The Bigger Picture

### **Doom is Just the Start**:

Once we solve these gaps for Doom, we've solved them for:

- **Web Browsers** (same input focus, performance budgets)
- **Video Players** (same audio mixing, resource coordination)
- **Terminal Emulators** (same lifecycle, input routing)
- **Data Visualizations** (same performance budgets, asset loading)
- **ANY embedded app**

### **The Vision**:

```
Doom taught us → Panel lifecycle, input focus, resource coordination
Web browser teaches us → Network coordination, security boundaries
Video player teaches us → Codec integration, streaming
Terminal teaches us → Text rendering, process management

Each use case evolves petalTongue.
Each evolution makes the next use case easier.
```

---

## 🧬 TRUE PRIMAL Evolution

This approach embodies TRUE PRIMAL principles:

1. **Real Use Cases** - Doom is real, not hypothetical
2. **Organic Discovery** - Gaps emerge naturally
3. **Test-Driven Evolution** - Implementation drives design
4. **Documentation** - Lessons recorded as we learn
5. **Composability** - Solutions work for future panels
6. **No Speculation** - Build what we need, when we need it

---

## 📊 Tracking Progress

### **Gap Discovery Log**:
```markdown
## Week 1
- Day 1: Discovered panel lifecycle gap (init/cleanup)
- Day 2: Discovered input focus gap (conflicting inputs)
- Day 3: Discovered frame timing gap (35 Hz vs 60 Hz)

## Week 2
- Day 1: Discovered asset loading gap (blocking on WAD)
- Day 2: Discovered audio mixing gap (no coordination)
...
```

### **Evolution Log**:
```markdown
## Week 1
- Created PanelLifecycle trait
- Implemented InputRouter with focus
- Added FramePacer for timing

## Week 2
- Created AssetManager with async loading
- Implemented AudioMixer for channels
...
```

---

## 🎮 Why This Matters

**Traditional approach**: "Let's build a panel system that MIGHT support games"  
**Our approach**: "Let's embed Doom and LET IT TELL US what we need"

**Result**: We build exactly what's needed, nothing more, nothing less.

---

## 🚀 Let's Begin!

**Next Steps**:
1. Set up doom-core crate
2. Integrate doomgeneric
3. Create minimal DoomPanel
4. **Watch the gaps reveal themselves**
5. **Evolve to fill them**
6. **Document everything we learn**

---

**Status**: Ready to start test-driven evolution  
**Philosophy**: Build through discovery, not speculation  
**Goal**: petalTongue that's PROVEN by real use cases

🌸 **Let Doom teach us what petalTongue needs to become!** 🎮

---

**Date**: January 15, 2026  
**Approach**: Test-Driven Evolution  
**First Test Case**: Doom  
**Expected Outcome**: 10+ architectural improvements discovered organically

