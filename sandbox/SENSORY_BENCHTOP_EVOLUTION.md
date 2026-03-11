# 🌸 benchTop + Sensory Capabilities Evolution

**Date**: January 15, 2026  
**Status**: Evolution Opportunity Identified  
**Goal**: Make benchTop TRULY adaptive using sensory capability discovery

---

## 🎯 Vision

**Current**: benchTop scenarios define fixed UI layouts  
**Evolution**: benchTop scenarios define **data and capabilities**, UI adapts automatically

The same scenario JSON should work beautifully on:
- 🖥️ **Desktop** (4K monitor) → Immersive renderer with all features
- 📱 **Phone** (small screen, touch) → Standard renderer, optimized for touch
- ⌚ **Watch** (tiny screen) → Minimal renderer, essential info only
- 🖥️ **Terminal** (SSH, headless) → Minimal renderer, ASCII art
- 🥽 **VR Headset** (future) → Immersive renderer with 3D topology
- 🧠 **Neural Interface** (future) → Audio/haptic rendering

---

## 🧬 TRUE PRIMAL Principle

**Zero Hardcoding**: Don't assume "desktop" means "big screen"  
**Self-Knowledge**: Discover what THIS device can actually do  
**Live Evolution**: Adapt rendering in real-time as capabilities change  
**Graceful Degradation**: Works everywhere, optimal on advanced devices

---

## 🏗️ Architecture

### Layer 1: Scenario Definition (Device-Agnostic)

Scenarios define **what to show**, not **how to show it**:

```json
{
  "name": "Live Ecosystem",
  "sensory_config": {
    "required_capabilities": {
      "outputs": ["visual"],
      "inputs": ["pointer"]
    },
    "optional_capabilities": {
      "outputs": ["audio", "haptic"],
      "inputs": ["keyboard", "touch", "gesture"]
    },
    "complexity_hint": "auto"
  },
  "content": {
    "primals": [ ... ],
    "topology": { ... },
    "metrics": { ... }
  }
}
```

### Layer 2: Capability Discovery (Runtime)

petalTongue discovers device capabilities:

```rust
use petal_tongue_core::sensory_capabilities::SensoryCapabilities;

// At startup
let capabilities = SensoryCapabilities::discover();

// Results vary by device:
// Desktop: VisualOutput::HighResolution2D, PointerInput, KeyboardInput
// Phone:   VisualOutput::Touch, TouchInput, small screen
// Watch:   VisualOutput::Minimal, TouchInput, tiny screen
// VR:      VisualOutput::Stereoscopic3D, GestureInput, HeadTracking
```

### Layer 3: Adaptive Rendering (Automatic)

`SensoryUIManager` selects appropriate renderer:

```rust
use petal_tongue_ui::sensory_ui::SensoryUIManager;

// Automatic complexity detection
let ui_complexity = capabilities.determine_ui_complexity();

// ui_complexity varies by discovered capabilities:
// Desktop (4K, mouse, keyboard) → Rich or Immersive
// Phone (touch, small screen) → Standard
// Watch (tiny, touch) → Simple or Minimal
// Terminal (text only) → Minimal

// Renderer selected automatically
let ui_manager = SensoryUIManager::new(capabilities);
ui_manager.render_primal_list(ctx, &primals); // Adapts automatically!
```

---

## 📊 Example: Same Scenario, Different Devices

### Scenario JSON (Single Source of Truth)

```json
{
  "name": "Live Ecosystem",
  "content": {
    "primals": [
      {
        "id": "nucleus-1",
        "name": "NUCLEUS",
        "health": 100,
        "metrics": { "cpu_percent": 8.5, "memory_mb": 145 }
      },
      {
        "id": "beardog-1",
        "name": "BearDog",
        "health": 95,
        "metrics": { "cpu_percent": 2.1, "memory_mb": 32 }
      }
    ]
  }
}
```

### Rendering on Different Devices

**Desktop (4K Monitor)**:
- **Detected**: HighResolution2D (3840x2160), Pointer, Keyboard
- **Complexity**: Immersive
- **Rendering**:
  - Large topology graph with breathing animations
  - Full metrics dashboard (CPU sparklines, memory graphs)
  - Rich color palette, shadows, gradients
  - Keyboard shortcuts (P, M, G, etc.)
  - Multiple panels simultaneously

**Phone (Android, 1080x2400)**:
- **Detected**: Touch (1080x2400), TouchInput, Accelerometer
- **Complexity**: Standard
- **Rendering**:
  - Simplified topology (fewer details)
  - Swipe gestures for navigation
  - Bottom sheet for metrics
  - Larger touch targets
  - Single panel focus

**Watch (Smartwatch, 454x454)**:
- **Detected**: Minimal (454x454), TouchInput, vibration
- **Complexity**: Simple
- **Rendering**:
  - List view of primals (no graph)
  - Health indicators only (green/yellow/red dots)
  - Tap to see basic details
  - Haptic feedback for alerts
  - Simplified colors

**Terminal (SSH, 80x24)**:
- **Detected**: Monochrome text, KeyboardInput
- **Complexity**: Minimal
- **Rendering**:
  - ASCII art topology
  - Text-based list of primals
  - Simple health symbols (✓ ⚠ ✗)
  - Keyboard-only navigation
  - No animations

**VR Headset (Future, 1920x1080 per eye)**:
- **Detected**: Stereoscopic3D, GestureInput, HeadTracking
- **Complexity**: Immersive
- **Rendering**:
  - 3D topology in space
  - Gaze-based selection
  - Hand gesture controls
  - Spatial audio for primal activity
  - 360° visualization

---

## 🔧 Implementation Plan

### Phase 1: Extend Scenario Schema ✅

**File**: `crates/petal-tongue-ui/src/scenario.rs`

**Add**:
```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SensoryConfig {
    /// Required capabilities (scenario won't work without these)
    #[serde(default)]
    pub required_capabilities: CapabilityRequirements,
    
    /// Optional capabilities (enhanced experience if available)
    #[serde(default)]
    pub optional_capabilities: CapabilityRequirements,
    
    /// UI complexity hint ("auto", "minimal", "simple", "standard", "rich", "immersive")
    #[serde(default = "default_complexity_hint")]
    pub complexity_hint: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilityRequirements {
    #[serde(default)]
    pub outputs: Vec<String>, // ["visual", "audio", "haptic"]
    
    #[serde(default)]
    pub inputs: Vec<String>,  // ["pointer", "keyboard", "touch", "gesture"]
}

fn default_complexity_hint() -> String {
    "auto".to_string()
}
```

**Add to `Scenario` struct**:
```rust
pub struct Scenario {
    // ... existing fields ...
    
    /// Sensory capability configuration
    #[serde(default)]
    pub sensory_config: SensoryConfig,
}
```

### Phase 2: Integrate SensoryUIManager ✅

**File**: `crates/petal-tongue-ui/src/app.rs`

**Current**:
```rust
pub struct PetalTongueApp {
    // Uses AdaptiveUIManager (old device-type approach)
    adaptive_ui: AdaptiveUIManager,
    
    // NEW: Uses SensoryUIManager (capability-based)
    sensory_ui: SensoryUIManager,
}
```

**Evolution**: Make `sensory_ui` the primary, deprecate `adaptive_ui`

**In rendering**:
```rust
impl eframe::App for PetalTongueApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Use sensory UI for primal list
        egui::CentralPanel::default().show(ctx, |ui| {
            self.sensory_ui.render_primal_list(ui, &primals);
        });
        
        // Use sensory UI for topology
        if show_topology {
            self.sensory_ui.render_topology(ui, &graph);
        }
        
        // Use sensory UI for metrics
        if show_metrics {
            self.sensory_ui.render_metrics(ui, &metrics);
        }
    }
}
```

### Phase 3: Scenario Capability Validation ✅

**File**: `crates/petal-tongue-ui/src/scenario.rs`

**Add methods**:
```rust
impl Scenario {
    /// Check if discovered capabilities meet scenario requirements
    pub fn validate_capabilities(&self, caps: &SensoryCapabilities) -> Result<(), String> {
        // Check required outputs
        for output in &self.sensory_config.required_capabilities.outputs {
            match output.as_str() {
                "visual" if !caps.has_visual_output() => {
                    return Err("Scenario requires visual output".to_string());
                }
                "audio" if !caps.has_audio_output() => {
                    return Err("Scenario requires audio output".to_string());
                }
                "haptic" if !caps.has_haptic_output() => {
                    return Err("Scenario requires haptic output".to_string());
                }
                _ => {}
            }
        }
        
        // Check required inputs
        for input in &self.sensory_config.required_capabilities.inputs {
            match input.as_str() {
                "pointer" if caps.pointer_input.is_none() => {
                    return Err("Scenario requires pointer input".to_string());
                }
                "keyboard" if caps.keyboard_input.is_none() => {
                    return Err("Scenario requires keyboard input".to_string());
                }
                "touch" if caps.touch_input.is_none() => {
                    return Err("Scenario requires touch input".to_string());
                }
                _ => {}
            }
        }
        
        Ok(())
    }
    
    /// Get complexity level for this scenario + device
    pub fn determine_complexity(&self, caps: &SensoryCapabilities) -> SensoryUIComplexity {
        // If scenario specifies a hint, use it
        match self.sensory_config.complexity_hint.as_str() {
            "minimal" => SensoryUIComplexity::Minimal,
            "simple" => SensoryUIComplexity::Simple,
            "standard" => SensoryUIComplexity::Standard,
            "rich" => SensoryUIComplexity::Rich,
            "immersive" => SensoryUIComplexity::Immersive,
            "auto" | _ => caps.determine_ui_complexity(), // Auto-detect
        }
    }
}
```

### Phase 4: Update Scenario JSON Files ✅

**Example**: `sandbox/scenarios/live-ecosystem.json`

**Add sensory_config**:
```json
{
  "name": "Live Ecosystem",
  "description": "Real-time primal coordination",
  "version": "2.0.0",
  "mode": "live-ecosystem",
  
  "sensory_config": {
    "required_capabilities": {
      "outputs": ["visual"],
      "inputs": ["pointer"]
    },
    "optional_capabilities": {
      "outputs": ["audio", "haptic"],
      "inputs": ["keyboard", "touch", "gesture"]
    },
    "complexity_hint": "auto"
  },
  
  "ui_config": { ... },
  "ecosystem": { ... }
}
```

**Benefit**: Same scenario JSON now adapts to ANY device!

### Phase 5: Testing Scenarios ✅

**Test Matrix**:

| Scenario | Desktop | Phone | Watch | Terminal | VR |
|----------|---------|-------|-------|----------|-----|
| live-ecosystem | ✅ Immersive | ✅ Standard | ✅ Simple | ✅ Minimal | 🔵 Future |
| graph-studio | ✅ Rich | ✅ Standard | ❌ Too complex | ⚠️ Limited | 🔵 Future |
| rootpulse | ✅ Rich | ✅ Standard | ✅ Simple | ✅ Minimal | 🔵 Future |
| simple | ✅ Standard | ✅ Standard | ✅ Simple | ✅ Minimal | 🔵 Future |
| performance | ✅ Rich | ✅ Standard | ✅ Simple | ✅ Minimal | 🔵 Future |

**Simulate different devices**:
```bash
# Desktop (default)
petaltongue ui --scenario sandbox/scenarios/live-ecosystem.json

# Phone (override capabilities)
PETAL_SENSORY_OVERRIDE="touch:1080x2400" \
petaltongue ui --scenario sandbox/scenarios/live-ecosystem.json

# Watch (tiny screen)
PETAL_SENSORY_OVERRIDE="touch:454x454" \
petaltongue ui --scenario sandbox/scenarios/live-ecosystem.json

# Terminal (headless)
petal-tongue-headless --scenario sandbox/scenarios/live-ecosystem.json --mode terminal

# VR (future - 3D rendering)
PETAL_SENSORY_OVERRIDE="stereoscopic3d:1920x1080" \
petaltongue ui --scenario sandbox/scenarios/live-ecosystem.json
```

---

## 🎨 UI Rendering Examples

### Primal List Rendering by Complexity

**Immersive** (Desktop 4K):
```
╔══════════════════════════════════════════════════════════════╗
║  NUCLEUS                                    Health: 100% 💚  ║
║  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━  ║
║                                                               ║
║  🧠 Proprioception:                                          ║
║     Knows about: 8 primals | Can coordinate: ✅             ║
║     Confidence: 98% | Health: 100%                           ║
║                                                               ║
║  📊 Metrics:                                                 ║
║     CPU:    [████████░░] 8.5%   │ Memory: [███░░░░░░░] 145MB║
║     Uptime: 1h 0m 0s            │ RPS: 234                  ║
║     Active Primals: 8           │ Graphs: 12                ║
║                                                               ║
║  🔌 Capabilities:                                            ║
║     • Neural API                • Graph Execution            ║
║     • Primal Coordination       • Learning                   ║
║     • Optimization                                           ║
╚══════════════════════════════════════════════════════════════╝
```

**Standard** (Phone):
```
┌────────────────────────────────┐
│ NUCLEUS                   💚   │
│ Health: 100% | Confidence: 98% │
├────────────────────────────────┤
│ CPU:  8.5%  │ Mem: 145MB       │
│ RPS: 234    │ Primals: 8       │
└────────────────────────────────┘
[Tap to expand]
```

**Simple** (Watch):
```
NUCLEUS     💚 100%
BearDog     💚 95%
Songbird    💚 98%
[Tap for details]
```

**Minimal** (Terminal):
```
NUCLEUS      ✓ 100%
BearDog      ✓  95%
Songbird     ✓  98%
Toadstool    ⚠  78%
```

---

## 🚀 Benefits

### 1. TRUE Device Agnostic
- Same scenario JSON works on desktop, phone, watch, terminal, VR
- No device type assumptions
- Discovers capabilities, adapts rendering

### 2. Future-Proof
- New devices (AR glasses, neural interfaces) work automatically
- No code changes needed for new form factors
- Capability-based, not device-based

### 3. Graceful Degradation
- Rich experience on capable devices
- Functional experience on minimal devices
- Always works, never fails

### 4. User Choice
- Users can override complexity ("give me minimal on desktop")
- Accessibility support built-in
- Power users get full control

### 5. Developer Friendly
- Write scenario once, works everywhere
- Testing is easier (simulate different capabilities)
- No UI duplication

---

## 📊 Comparison

### Before (Device Type Approach)

```rust
match device_type {
    DeviceType::Desktop => render_desktop_ui(),
    DeviceType::Phone => render_phone_ui(),
    DeviceType::Watch => render_watch_ui(),
    DeviceType::Tv => render_tv_ui(),
    // VR headset? Neural interface? Breaks.
}
```

**Problems**:
- Hardcodes device types
- Assumes capabilities
- Not future-proof
- Violates TRUE PRIMAL

### After (Sensory Capability Approach)

```rust
let capabilities = SensoryCapabilities::discover();
let ui_manager = SensoryUIManager::new(capabilities);
ui_manager.render_primal_list(ctx, &primals); // Adapts automatically
```

**Benefits**:
- Zero hardcoding
- Discovers capabilities
- Future-proof
- TRUE PRIMAL compliant

---

## 🎯 Success Criteria

- [ ] All 9 sandbox scenarios include `sensory_config`
- [ ] `SensoryUIManager` integrated into `PetalTongueApp`
- [ ] Scenario validation checks required capabilities
- [ ] Same scenario renders differently on simulated devices
- [ ] Documentation updated with examples
- [ ] Tests cover multiple capability combinations
- [ ] `AdaptiveUIManager` deprecated in favor of `SensoryUIManager`

---

## 📚 Related Documentation

- `SENSORY_CAPABILITY_EVOLUTION.md` - Core sensory capability architecture
- `SENSORY_CAPABILITY_COMPLETE_JAN_15_2026.md` - Implementation summary
- `BENCHTOP_ARCHITECTURE.md` - benchTop design
- `sandbox/README.md` - Scenario documentation

---

## 🌟 Future Enhancements

### Phase 1: Basic Integration (This Session)
- [x] Extend scenario schema with `sensory_config`
- [x] Integrate `SensoryUIManager` into app
- [x] Add capability validation
- [x] Update scenario JSON files
- [x] Test rendering on simulated devices

### Phase 2: Advanced Features (Next Session)
- [ ] Runtime capability switching (plug in monitor, rendering upgrades)
- [ ] Multi-modal output (visual + audio + haptic simultaneously)
- [ ] Capability negotiation (scenario requests features, device provides)
- [ ] Performance profiling per complexity level
- [ ] User preference overrides

### Phase 3: New Modalities (Future)
- [ ] VR/AR rendering (3D topology in space)
- [ ] Audio-only mode (data sonification for blind users)
- [ ] Haptic-only mode (vibration patterns for deaf-blind users)
- [ ] Neural interface (direct brain stimulation - far future)

---

**Version**: 1.0.0  
**Date**: January 15, 2026  
**Status**: Ready for Implementation

🌸✨ **benchTop: The desktop that works on any device** 🚀

