# 🎨 Modular UI Control - Fine-Grained Subsystem Selection

**Date**: January 15, 2026  
**Goal**: Demonstrate petalTongue's modular architecture - each subsystem is **opt-in**, not bundled  
**Status**: 🚧 **IN PROGRESS**

---

## 🎯 The Vision

**Current Problem**: All scenarios show the same UI with all subsystems enabled:
- Left sidebar (always visible)
- Right sidebar (always visible)
- Audio sonification (always playing)
- Proprioception dashboard (always calculating)
- System metrics (always polling)
- Graph statistics (always visible)

**This violates TRUE PRIMAL**: The UI is **hardcoded as a bundle**, not composable!

---

## ✨ The Solution: Modular Subsystem Control

Each scenario can **selectively enable** only the subsystems it needs:

### Example 1: `paint-simple.json` - Pure Canvas
```json
{
  "mode": "paint-canvas",
  "ui_config": {
    "layout": "canvas-only",
    "show_panels": {
      "left_sidebar": false,
      "right_sidebar": false,
      "top_menu": true,
      "system_dashboard": false,
      "audio_panel": false,
      "proprioception": false
    },
    "features": {
      "audio_sonification": false,
      "auto_refresh": false,
      "neural_api": false
    }
  }
}
```

**Result**: JUST the canvas with 3 painted nodes. Nothing else.

---

### Example 2: `live-ecosystem.json` - Full Dashboard
```json
{
  "mode": "live-ecosystem",
  "ui_config": {
    "layout": "full-dashboard",
    "show_panels": {
      "left_sidebar": true,
      "right_sidebar": true,
      "system_dashboard": true,
      "audio_panel": true,
      "proprioception": true
    },
    "features": {
      "audio_sonification": true,
      "auto_refresh": true,
      "neural_api": true
    }
  }
}
```

**Result**: Full benchTop experience with all subsystems active.

---

### Example 3: `neural-api-test.json` - API Focus
```json
{
  "mode": "neural-dashboard",
  "ui_config": {
    "layout": "dashboard-centered",
    "show_panels": {
      "left_sidebar": false,
      "right_sidebar": true,
      "proprioception": true,
      "neural_api_metrics": true
    },
    "features": {
      "audio_sonification": false,
      "neural_api": true
    }
  }
}
```

**Result**: Focus on Neural API metrics and proprioception, no audio clutter.

---

## 🧬 Architecture: Composable Subsystems

### Current (Monolithic):
```
PetalTongueApp {
    // ALL systems always created
    visual_renderer ✅
    audio_renderer ✅
    proprioception ✅
    trust_dashboard ✅
    neural_metrics ✅
    // ... etc
}

fn update() {
    // ALL panels always shown
    egui::SidePanel::left() { ... }
    egui::SidePanel::right() { ... }
    egui::CentralPanel::default() { ... }
}
```

### Evolved (Modular):
```
PetalTongueApp {
    // Core always present
    visual_renderer ✅
    
    // Optional subsystems (Option<T>)
    audio_renderer: Option<AudioRenderer>
    proprioception: Option<ProprioceptionMonitor>
    trust_dashboard: Option<TrustDashboard>
    neural_metrics: Option<NeuralMetricsDashboard>
}

fn update() {
    // Conditional panel rendering
    if scenario.show_panel("left_sidebar") {
        egui::SidePanel::left() { ... }
    }
    
    if scenario.show_panel("audio_panel") {
        audio_renderer.as_ref().unwrap().render();
    }
    
    // Central panel ALWAYS shown (canvas)
    egui::CentralPanel::default() { 
        visual_renderer.render();
    }
}
```

---

## 📋 Implementation Plan

### Phase 1: Scenario Schema Extension ✅
- [x] Add `ui_config.layout` field
- [x] Add `ui_config.show_panels` object
- [x] Add `ui_config.features` object
- [x] Update `paint-simple.json` with minimal config

### Phase 2: Conditional Panel Rendering
- [ ] Read `show_panels` config from scenario
- [ ] Make left sidebar conditional
- [ ] Make right sidebar conditional
- [ ] Make proprioception panel conditional
- [ ] Make audio panel conditional

### Phase 3: Conditional Subsystem Creation
- [ ] Make `audio_renderer` Optional
- [ ] Make `proprioception` Optional (or create stub)
- [ ] Make `trust_dashboard` Optional
- [ ] Make `neural_metrics` Optional

### Phase 4: Feature Flags
- [ ] Respect `features.audio_sonification`
- [ ] Respect `features.auto_refresh`
- [ ] Respect `features.neural_api`

### Phase 5: Layout Modes
- [ ] Implement `canvas-only` layout
- [ ] Implement `dashboard-centered` layout
- [ ] Implement `full-dashboard` layout

---

## 🎨 Visual Examples

### Paint Mode (canvas-only):
```
┌─────────────────────────────────────────┐
│ File  View  Layout  Tools        ⚙️     │ ← Minimal top menu
├─────────────────────────────────────────┤
│                                         │
│                                         │
│             🔴  Red Circle              │
│                                         │
│      🟢  Green Square                   │
│                                         │
│                  🔵  Blue Triangle      │
│                                         │
│                                         │
│                                         │
└─────────────────────────────────────────┘
```

### Full Dashboard Mode:
```
┌─────────────────────────────────────────────────────────────┐
│ File  View  Layout  Tools  Dashboard  Audio  Help     ⚙️    │
├──────┬───────────────────────────────────────┬──────────────┤
│ 📊   │                                       │ 🎵 Audio     │
│Stats │         Graph Canvas                  │ Master: 0.7  │
│      │                                       │              │
│Nodes │      🟢 ───→ 🔵 ───→ 🟣             │ 🧠 Proprio   │
│  8   │      ↓        ↓        ↓             │ Health: 85%  │
│      │      🟡 ───→ 🔴 ───→ 🟠             │ Confidence   │
│Edges │                                       │ 92%          │
│  12  │                                       │              │
│      │                                       │ 📡 Network   │
│Trust │                                       │ 5 primals    │
│ 95%  │                                       │ 3 families   │
└──────┴───────────────────────────────────────┴──────────────┘
```

### Neural API Focus:
```
┌─────────────────────────────────────────────────────────────┐
│ File  View  Neural API  Metrics                      ⚙️     │
├───────────────────────────────────────┬─────────────────────┤
│                                       │ 🧠 NUCLEUS Proprio  │
│         Neural Graph                  │                     │
│                                       │ Health: 100%        │
│      [NUCLEUS]                        │ Confidence: 95%     │
│          ↓                            │                     │
│    ┌─────┴─────┐                     │ Self-Awareness:     │
│    ↓           ↓                     │ • Knows: 5 primals  │
│ [BearDog]  [Songbird]                │ • Coordinate: Yes   │
│                                       │ • Security: Yes     │
│                                       │                     │
│                                       │ 📊 Metrics:         │
│                                       │ CPU: 12.5%          │
│                                       │ Mem: 256 MB         │
│                                       │ Uptime: 2h          │
│                                       │ RPS: 150            │
└───────────────────────────────────────┴─────────────────────┘
```

---

## 🧪 Test Scenarios

### Minimal Paint Test
```bash
cargo run --release --bin petal-tongue -- \
  --scenario sandbox/scenarios/paint-simple.json
```

**Expected**:
- ✅ Full-screen canvas
- ✅ 3 nodes rendered at exact positions
- ✅ Minimal top menu bar
- ❌ NO left sidebar
- ❌ NO right sidebar
- ❌ NO audio sonification
- ❌ NO proprioception panel
- ❌ NO graph statistics

### Full Ecosystem
```bash
cargo run --release --bin petal-tongue -- \
  --scenario sandbox/scenarios/live-ecosystem.json
```

**Expected**:
- ✅ Left sidebar with statistics
- ✅ Right sidebar with audio + proprioception
- ✅ Audio sonification playing
- ✅ Auto-refresh polling
- ✅ All 8 nodes with animations

### Neural API Focus
```bash
cargo run --release --bin petal-tongue -- \
  --scenario sandbox/scenarios/neural-api-test.json
```

**Expected**:
- ✅ Right sidebar with Neural API metrics
- ✅ Proprioception dashboard
- ❌ NO audio
- ❌ NO left statistics panel

---

## 🎯 Benefits

### 1. **TRUE PRIMAL Compliance**
- ✅ **Zero Hardcoding**: UI layout defined in scenario JSON
- ✅ **Capability-Based**: Features activated based on configuration
- ✅ **Live Evolution**: Can hot-swap between modes

### 2. **Performance**
- Skip unused subsystems (no audio processing if disabled)
- Reduce CPU/memory usage for minimal scenarios
- Faster startup when features disabled

### 3. **Developer Experience**
- Easy to create focused demos
- Test individual subsystems in isolation
- Clear separation of concerns

### 4. **User Experience**
- Paint mode: Simple, uncluttered canvas
- Dashboard mode: Full power-user experience
- Custom modes: Tailor UI to specific use cases

---

## 📊 Subsystem Inventory

### Always Active (Core):
- `visual_renderer` - Graph canvas rendering
- `graph` - Graph engine (data model)
- `scenario` - Scenario loader (if provided)

### Conditionally Active:
- `audio_renderer` - Audio sonification
- `proprioception` - SAME DAVE monitoring
- `trust_dashboard` - Network trust visualization
- `neural_metrics` - Neural API metrics
- `accessibility_panel` - A11y features
- `keyboard_shortcuts` - Keyboard help overlay
- `animation_engine` - Breathing/pulse animations
- `tutorial_mode` - Interactive tutorial

### UI Panels (Conditional):
- Left sidebar (statistics)
- Right sidebar (audio + proprioception)
- Top menu bar (always shown, but simplified in minimal mode)
- Bottom status bar (optional)
- Graph Builder window (optional)

---

## 🔧 Technical Implementation

### Scenario Config Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    pub theme: String,
    pub layout: LayoutMode,
    pub show_panels: PanelVisibility,
    pub animations: AnimationConfig,
    pub performance: PerformanceConfig,
    pub features: FeatureFlags,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutMode {
    #[serde(rename = "canvas-only")]
    CanvasOnly,
    #[serde(rename = "dashboard-centered")]
    DashboardCentered,
    #[serde(rename = "full-dashboard")]
    FullDashboard,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelVisibility {
    pub left_sidebar: bool,
    pub right_sidebar: bool,
    pub top_menu: bool,
    pub system_dashboard: bool,
    pub audio_panel: bool,
    pub trust_dashboard: bool,
    pub proprioception: bool,
    pub graph_stats: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub audio_sonification: bool,
    pub auto_refresh: bool,
    pub neural_api: bool,
    pub tutorial_mode: bool,
}
```

### Conditional Rendering

```rust
impl eframe::App for PetalTongueApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let palette = ColorPalette::default();
        
        // Top menu (always shown, but simplified in minimal mode)
        egui::TopBottomPanel::top("top_menu").show(ctx, |ui| {
            self.render_top_menu(ui, &palette);
        });
        
        // LEFT SIDEBAR - Conditional
        if self.should_show_panel("left_sidebar") {
            egui::SidePanel::left("left_panel")
                .default_width(300.0)
                .show(ctx, |ui| {
                    self.render_left_sidebar(ui, &palette);
                });
        }
        
        // RIGHT SIDEBAR - Conditional
        if self.should_show_panel("right_sidebar") {
            egui::SidePanel::right("right_panel")
                .default_width(350.0)
                .show(ctx, |ui| {
                    self.render_right_sidebar(ui, &palette);
                });
        }
        
        // CENTRAL PANEL - Always shown (canvas)
        egui::CentralPanel::default().show(ctx, |ui| {
            self.visual_renderer.render(ui);
        });
    }
    
    fn should_show_panel(&self, panel_name: &str) -> bool {
        self.scenario
            .as_ref()
            .and_then(|s| s.ui_config.show_panels.get(panel_name))
            .copied()
            .unwrap_or(true) // Default to showing if not specified
    }
}
```

---

## 🚀 Next Steps

1. ✅ Update `paint-simple.json` with minimal config
2. Add `UIConfig`, `PanelVisibility`, `FeatureFlags` to `scenario.rs`
3. Read panel config in `app.rs` initialization
4. Make sidebar rendering conditional
5. Make subsystem creation conditional
6. Test paint mode (canvas-only)
7. Update `live-ecosystem.json` with full config
8. Test full dashboard mode
9. Create `neural-api-test.json` focused mode
10. Document all layout modes

---

## ✅ Success Criteria

- [ ] Paint mode shows ONLY canvas (no sidebars)
- [ ] Full dashboard mode shows all panels
- [ ] Neural API mode shows focused metrics
- [ ] Can hot-swap between modes without restart
- [ ] Each subsystem is independently toggleable
- [ ] Performance improves when features disabled

---

**Status**: 🚧 Implementing conditional panel rendering  
**Next**: Update scenario schema and app.rs panel logic

---

🌸 **TRUE PRIMAL**: Composable, not bundled! Each subsystem is a choice, not a requirement.

