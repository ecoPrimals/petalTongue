# Central Nervous System Evolution
## Completing the Bidirectional Sensory Architecture
### January 9, 2026

**Status**: 🔧 IN PROGRESS  
**Priority**: CRITICAL - Core sovereignty capability  
**Context**: Post-GUI fix, completing the sensory/motor coordination layer

---

## 🎯 Mission

Complete the **central nervous system** that coordinates:
1. **Motor Output** (rendering to displays)
2. **Sensory Input** (verification frames reach user)
3. **Bidirectional Verification** (complete feedback loop)
4. **Runtime Discovery** (zero hardcoded peripheral knowledge)

---

## 📊 Current State Assessment

### ✅ Infrastructure EXISTS (90% complete!)

**Core Components**:
- ✅ `RenderingAwareness` - Motor + sensory coordination (crates/petal-tongue-core/src/rendering_awareness.rs)
- ✅ `Sensor` trait - Universal abstraction (crates/petal-tongue-core/src/sensor.rs)
- ✅ `SensorRegistry` - Discovery coordinator
- ✅ `ValidationPipeline` - Frame confirmation tracking

**Sensor Implementations** (744 lines):
- ✅ `ScreenSensor` - Display output with heartbeat verification
- ✅ `KeyboardSensor` - Runtime keyboard discovery
- ✅ `MouseSensor` - Spatial input with crossterm
- ✅ `AudioSensor` - Audio I/O capabilities

### ❌ Integration MISSING (Critical Gaps)

1. **RenderingAwareness NOT Connected to UI**
   - Defined in core, never instantiated in app
   - Motor commands not recorded
   - Sensory feedback not processed
   - **Impact**: Can't verify if frames reach user

2. **SensorRegistry NOT Populated**
   - Sensors exist but not discovered at runtime
   - No central coordinator running
   - **Impact**: Sensors aren't used

3. **Display Substrate Verification INCOMPLETE**
   - Window creation checked, but not *visibility*
   - No confirmation frames are seen
   - **Impact**: False positives (window exists ≠ user sees it)

4. **Hardcoded Display Dimensions**
   - `ScreenSensor::new()` uses fixed 1400x900, 1920x1080
   - Should discover from actual display
   - **Impact**: Layout calculations may be wrong

5. **No Sensory Event Loop**
   - `poll_events()` defined but never called
   - Events don't flow to `RenderingAwareness`
   - **Impact**: One-way communication only

---

## 🚀 Evolution Plan

### Phase 1: Connect RenderingAwareness to UI ✅ Priority 1

**Goal**: Integrate the central nervous system into the application loop

**Tasks**:
1. Add `RenderingAwareness` field to `PetalTongueApp`
2. Record motor commands when frames are sent
3. Process sensory feedback from egui events
4. Update `SelfAssessment` in status dashboard

**Files to Modify**:
- `crates/petal-tongue-ui/src/app.rs`
- `crates/petal-tongue-ui/src/system_dashboard.rs`

### Phase 2: Runtime Sensor Discovery ✅ Priority 1

**Goal**: Populate `SensorRegistry` at startup with discovered peripherals

**Tasks**:
1. Create sensor discovery coordinator
2. Call `screen::discover()`, `keyboard::discover()`, `mouse::discover()`
3. Register discovered sensors in registry
4. Log what was found (observability)

**Files to Modify**:
- `crates/petal-tongue-ui/src/app.rs` (in `new()`)
- Create `crates/petal-tongue-ui/src/sensor_discovery.rs`

### Phase 3: Sensory Event Loop ✅ Priority 1

**Goal**: Poll sensors and feed events to `RenderingAwareness`

**Tasks**:
1. Add background task to poll `SensorRegistry`
2. Convert egui input to `SensorEvent`
3. Feed events to `RenderingAwareness::sensory_feedback()`
4. Update confirmation metrics

**Files to Modify**:
- `crates/petal-tongue-ui/src/app.rs` (in `update()`)
- Create `crates/petal-tongue-ui/src/event_loop.rs`

### Phase 4: Display Substrate Verification ✅ Priority 2

**Goal**: Actually verify display is visible, not just present

**Tasks**:
1. Implement window visibility check (X11/Wayland/Win32)
2. Add frame acknowledgment from egui
3. Measure confirmation rate
4. Fall back if confirmation drops below threshold

**Files to Modify**:
- `crates/petal-tongue-ui/src/display/manager.rs`
- `crates/petal-tongue-ui/src/sensors/screen.rs`

### Phase 5: Remove Hardcoded Display Dimensions ✅ Priority 2

**Goal**: Discover actual screen size at runtime

**Tasks**:
1. Query X11/Wayland for screen dimensions
2. Use `winit` or `eframe` to get actual window size
3. Update `ScreenSensor` with real dimensions
4. Recalculate layouts based on actual size

**Files to Modify**:
- `crates/petal-tongue-ui/src/sensors/screen.rs`
- `crates/petal-tongue-ui/src/main.rs`

### Phase 6: Complete Display Backend TODOs ✅ Priority 3

**Goal**: Finish incomplete display backend implementations

**Tasks**:
1. Complete `SoftwareBackend::Window` presentation
2. Optimize pixel conversion in renderer
3. Implement clipping for software rendering

**Files to Modify**:
- `crates/petal-tongue-ui/src/display/backends/software.rs`
- `crates/petal-tongue-ui/src/display/renderer.rs`

### Phase 7: Testing & Validation ✅ Priority 3

**Goal**: Verify bidirectional loop is working

**Tasks**:
1. Add tests for `RenderingAwareness` integration
2. Verify `SelfAssessment` reports correctly
3. Test sensor discovery on different platforms
4. Chaos testing: kill display, measure recovery

**Files to Create**:
- `crates/petal-tongue-ui/tests/sensory_integration_tests.rs`
- `crates/petal-tongue-core/tests/rendering_awareness_tests.rs`

---

## 🎯 Success Criteria

### Minimum Viable (v0.6.0)
- [ ] `RenderingAwareness` instantiated and used
- [ ] Sensors discovered at runtime
- [ ] Events flow: Display → egui → SensorEvent → RenderingAwareness
- [ ] `SelfAssessment` shows:
  - `can_render: true`
  - `can_sense: true`
  - `is_complete_loop: true`
  - `confirmation_rate > 0%`

### Production Ready (v1.0.0)
- [ ] Display dimensions discovered (not hardcoded)
- [ ] Window visibility verified (not just existence)
- [ ] Confirmation rate > 90%
- [ ] Graceful degradation if display fails
- [ ] All display backend TODOs resolved
- [ ] E2E tests for sensor discovery
- [ ] Chaos tests for display failure

### Excellence (v1.1.0)
- [ ] Multi-display support
- [ ] Hot-plug peripheral detection
- [ ] Predictive frame confirmation (ML)
- [ ] Sub-frame latency measurement
- [ ] Accessibility sensor discovery (braille, TTS)

---

## 📝 Implementation Notes

### Key Architectural Principles

1. **TRUE PRIMAL**: No hardcoded assumptions
   - Discover peripherals at runtime
   - Test capabilities, don't assume
   - Graceful degradation if sensors unavailable

2. **Bidirectional UUI**: Complete feedback loop
   - Motor output (rendering)
   - Sensory input (verification)
   - Confirmation (user actually sees it)

3. **Sovereignty**: Works independently
   - Doesn't require other primals for sensory functions
   - Self-assessment of capabilities
   - Honest about what it can/can't do

### Technical Decisions

**Why not integrate with egui's input system?**
- We do! But we wrap it in our `SensorEvent` abstraction
- This allows us to add non-egui sensors later (camera, biometric, etc.)
- Decouples rendering framework from sensory system

**Why separate `RenderingAwareness` from `SensorRegistry`?**
- `SensorRegistry`: Discovery + polling (what sensors exist?)
- `RenderingAwareness`: Validation + metrics (are they working?)
- Separation of concerns: detection vs. verification

**Why track confirmation rate?**
- Display can exist but be invisible (minimized, occluded, etc.)
- Low confirmation = maybe user can't see us
- Enables adaptive behavior (switch to audio, alert, etc.)

---

## 🔍 Deep Debt Solutions

### Problem: Window Creation != User Visibility

**Current Code** (`crates/petal-tongue-ui/src/main.rs:67-88`):
```rust
let has_display = std::env::var("DISPLAY").is_ok()
    || std::env::var("WAYLAND_DISPLAY").is_ok()
    || cfg!(target_os = "windows")
    || cfg!(target_os = "macos");

if !has_display {
    tracing::info!("🪟 No display server detected");
    // ... prompt user ...
}
```

**Problem**: This checks if a display *server* exists, not if our *window* is visible!

**Solution**: Add actual visibility verification
```rust
// Phase 1: Server detection (current)
let has_display_server = detect_display_server();

// Phase 2: Window creation
let window = create_window()?;

// Phase 3: Visibility verification (NEW!)
let rendering_awareness = RenderingAwareness::new();
rendering_awareness.motor_command(MotorCommand::RenderFrame { frame_id: 0 });

// Phase 4: Wait for confirmation
let timeout = Duration::from_secs(2);
if !rendering_awareness.wait_for_confirmation(timeout).await {
    tracing::warn!("Window created but visibility unconfirmed!");
    // Fall back to alternate modality
}
```

### Problem: Hardcoded Display Dimensions

**Current Code** (`crates/petal-tongue-ui/src/sensors/screen.rs:135-145`):
```rust
// Method 2: Framebuffer
if std::path::Path::new("/dev/fb0").exists() {
    tracing::debug!("Discovered framebuffer screen");
    return Some(ScreenSensor::new(DisplayType::Framebuffer, 1920, 1080)); // HARDCODED!
}

// Method 3: Window
if std::env::var("DISPLAY").is_ok() ... {
    tracing::debug!("Discovered window screen");
    return Some(ScreenSensor::new(DisplayType::Window, 1400, 900)); // HARDCODED!
}
```

**Problem**: Fixed dimensions may not match actual display!

**Solution**: Query actual dimensions
```rust
// Method 2: Framebuffer (EVOLVED)
if std::path::Path::new("/dev/fb0").exists() {
    if let Ok((width, height)) = query_framebuffer_size("/dev/fb0") {
        tracing::debug!("Discovered framebuffer screen: {}x{}", width, height);
        return Some(ScreenSensor::new(DisplayType::Framebuffer, width, height));
    }
}

// Method 3: Window (EVOLVED)
if let Some(display_size) = query_display_size() {
    tracing::debug!("Discovered window screen: {}x{}", display_size.0, display_size.1);
    return Some(ScreenSensor::new(DisplayType::Window, display_size.0, display_size.1));
}
```

### Problem: Sensors Defined But Never Used

**Current State**:
- ✅ `Sensor` trait defined
- ✅ 4 sensor implementations exist
- ❌ Never instantiated in application
- ❌ `SensorRegistry` never populated
- ❌ `poll_events()` never called

**Solution**: Complete the integration (Phases 1-3)

---

## 📈 Expected Impact

### Performance
- **Minimal overhead**: Sensor polling is async, non-blocking
- **Confirmation tracking**: ~100 bytes per frame
- **Event processing**: <1ms per frame

### User Experience
- **Transparency**: Users see `SelfAssessment` in dashboard
- **Reliability**: System knows if it's working
- **Adaptive**: Can switch modalities if display fails

### Architecture
- **Modularity**: Sensors are plugins
- **Testability**: Can mock sensor events
- **Extensibility**: Easy to add new sensor types

---

## 🚦 Execution Status

### Completed
- ✅ Bidirectional sensory fix (GUI now appears)
- ✅ Infrastructure audit (found all components)
- ✅ Evolution plan created

### In Progress
- 🔄 Phase 1: Connect RenderingAwareness to UI

### Pending
- ⏳ Phase 2: Runtime Sensor Discovery
- ⏳ Phase 3: Sensory Event Loop
- ⏳ Phase 4: Display Substrate Verification
- ⏳ Phase 5: Remove Hardcoded Dimensions
- ⏳ Phase 6: Complete Display Backend TODOs
- ⏳ Phase 7: Testing & Validation

---

**Next Action**: Begin Phase 1 - Integrate `RenderingAwareness` into `PetalTongueApp`

🌸 **Building a nervous system that knows itself!**

