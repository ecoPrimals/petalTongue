# 🧬 Doom Evolution - Deep Insights & Evolution Opportunities

**Date**: January 15, 2026  
**Context**: After successfully implementing panel system via test-driven evolution  
**Status**: Foundation complete, deep insights revealed

---

## 🎯 **What This "Successfully Fail" Taught Us**

The Doom integration wasn't just about rendering a test pattern. It was a **probe** into petalTongue's architecture, revealing fundamental gaps and evolution opportunities.

---

## 🔍 **GAP #5: Silent Deserialization Failures**

### **What We Discovered**

```rust
// This is DANGEROUS:
#[derive(Deserialize)]
pub struct Scenario {
    pub mode: String,  // ❌ Required, no default
    #[serde(default)]  // ⚠️ Silent failure zone!
    pub ui_config: UiConfig,
}
```

**The Problem**:
- Missing `mode` field → parent struct parse fails
- `#[serde(default)]` on child → uses empty defaults
- **No error message!**
- Looks successful, but data is missing

**The Symptom**:
```
✅ Scenario loaded successfully: Doom MVP
✅ Panel registry initialized
✅ Custom panels initialized: 0 panels  ← Silent failure!
```

### **Deep Debt Revealed**

**1. Validation Gap**

petalTongue has **no explicit validation layer** between deserialization and use:

```rust
// Current (UNSAFE):
let scenario = serde_json::from_str(&contents)?;  // ✅ Parses
// Uses scenario.ui_config.custom_panels  // ❌ Might be empty!

// Needed (SAFE):
let scenario = serde_json::from_str(&contents)?;
scenario.validate()?;  // ✅ Explicit check
// Now safe to use!
```

**Evolution Opportunity**: Add `Scenario::validate()` method

**2. Error Message Quality**

Current error handling is **too permissive**:
- Parse failures are caught, but **silently degraded**
- Missing required fields use defaults instead of failing
- No indication of what went wrong

**Evolution Opportunity**: Fail-fast with helpful messages

**3. Schema Flexibility vs Safety Trade-off**

```rust
// Too flexible (current):
#[serde(default)]  // Everything optional
pub ui_config: UiConfig,

// Too rigid:
pub ui_config: UiConfig,  // Everything required

// Sweet spot (evolution):
pub ui_config: Option<UiConfig>,  // Explicit optionality
// Then: if ui_config.is_none() { use sensible defaults }
```

**Evolution Opportunity**: Make optionality **explicit**, not implicit

---

## 🧬 **EVOLUTION OPPORTUNITY #1: Scenario Validation Layer**

### **The Vision**

Add explicit validation that catches problems **before** they cause mysterious bugs:

```rust
impl Scenario {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let scenario: Scenario = serde_json::from_str(&contents)?;
        
        // ✅ NEW: Explicit validation
        scenario.validate()?;
        
        Ok(scenario)
    }
    
    pub fn validate(&self) -> Result<()> {
        // Check required fields
        if self.name.is_empty() {
            bail!("Scenario name cannot be empty");
        }
        
        // Check sensory config
        self.sensory_config.validate()?;
        
        // Check UI config
        self.ui_config.validate()?;
        
        // Check for orphaned references
        self.check_panel_references()?;
        
        Ok(())
    }
}

impl UiConfig {
    pub fn validate(&self) -> Result<()> {
        // Check custom panels
        for (idx, panel) in self.custom_panels.iter().enumerate() {
            if panel.panel_type.is_empty() {
                bail!("Panel {} has empty type", idx);
            }
            if panel.title.is_empty() {
                bail!("Panel {} has empty title", idx);
            }
            // Validate dimensions
            if let Some(w) = panel.width {
                if w == 0 {
                    bail!("Panel '{}' has zero width", panel.title);
                }
            }
        }
        Ok(())
    }
}
```

### **Benefits**

✅ **Fail-fast**: Catch problems at load time, not runtime  
✅ **Clear errors**: "Panel 0 has empty type" vs silent empty vec  
✅ **Self-documenting**: Validation code shows what's required  
✅ **Test-friendly**: Can unit test validation logic  

### **Implementation Plan**

**Phase 1**: Add validation methods (no behavior change)
- Add `validate()` to Scenario, UiConfig, SensoryConfig
- Call after deserialization
- Tests for each validation rule

**Phase 2**: Improve error messages
- Use `thiserror` for structured errors
- Include context (line numbers, field paths)
- Suggest fixes ("Did you forget 'mode'?")

**Phase 3**: Make optionality explicit
- Change `#[serde(default)]` to `Option<T>`
- Handle `None` explicitly with logged defaults
- Document why each field is optional

---

## 🧬 **EVOLUTION OPPORTUNITY #2: Better Error Reporting**

### **The Problem**

Current error handling loses context:

```rust
// What we get:
❌ Failed to load scenario: invalid type: null, expected a string

// What we need:
❌ Failed to load scenario 'doom-mvp.json':
   Missing required field 'mode' at line 1
   
   Expected:
   {
     "name": "...",
     "mode": "doom-showcase",  ← Add this!
     ...
   }
```

### **Evolution: Context-Rich Errors**

```rust
#[derive(Debug, Error)]
pub enum ScenarioError {
    #[error("Missing required field '{field}' in {file}:{line}")]
    MissingField {
        field: String,
        file: String,
        line: usize,
    },
    
    #[error("Invalid panel type '{panel_type}' (available: {available:?})")]
    UnknownPanelType {
        panel_type: String,
        available: Vec<String>,
    },
    
    #[error("Panel '{title}' references unknown capability '{capability}'")]
    UnknownCapability {
        title: String,
        capability: String,
    },
}
```

### **Benefits**

✅ Users know **exactly** what's wrong  
✅ Errors suggest **how to fix**  
✅ Debugging time: minutes → seconds  
✅ Better documentation through errors  

---

## 🧬 **EVOLUTION OPPORTUNITY #3: Schema Migration System**

### **The Problem**

We're on scenario schema `v2.0.0`, but what happens when we need `v3.0.0`?

Currently: **Breaking changes break everything**

### **Evolution: Schema Migrations**

```rust
pub struct ScenarioMigration {
    from_version: String,
    to_version: String,
    migrate: fn(Value) -> Result<Value>,
}

impl Scenario {
    pub fn load_with_migration<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut value: Value = serde_json::from_str(&contents)?;
        
        // Get version
        let version = value["version"].as_str()
            .ok_or_else(|| anyhow!("Missing version field"))?;
        
        // Migrate if needed
        let migrations = get_migrations();
        let migrated = migrations.apply(value, version, CURRENT_VERSION)?;
        
        // Now deserialize
        let scenario: Scenario = serde_json::from_value(migrated)?;
        Ok(scenario)
    }
}

// Example migration: v1.0.0 → v2.0.0
fn migrate_v1_to_v2(mut value: Value) -> Result<Value> {
    // v1.0.0 didn't have 'mode' field
    if value["mode"].is_null() {
        // Infer from 'name' field
        let mode = infer_mode_from_name(value["name"].as_str());
        value["mode"] = json!(mode);
    }
    
    // v1.0.0 used 'device_type' instead of 'sensory_config'
    if let Some(device) = value["device_type"].take() {
        value["sensory_config"] = convert_device_to_sensory(device);
    }
    
    Ok(value)
}
```

### **Benefits**

✅ **Backward compatible**: Old scenarios still work  
✅ **Forward evolution**: Can evolve schema freely  
✅ **Automated**: Migrations run transparently  
✅ **Testable**: Each migration is a pure function  

---

## 🧬 **EVOLUTION OPPORTUNITY #4: Runtime Diagnostics**

### **The Problem**

When things go wrong, we only see:
```
✅ Custom panels initialized: 0 panels
```

**We need to know WHY it's 0!**

### **Evolution: Diagnostic Mode**

```rust
// Enable with environment variable
PETALTONGUE_DIAGNOSTICS=true cargo run --scenario doom-mvp.json

// Output:
🔍 Diagnostic Mode Enabled
📋 Loading scenario: sandbox/scenarios/doom-mvp.json
   ✅ Parsed JSON (61 lines)
   ✅ Found version: 2.0.0
   ⚠️  Missing field 'mode' at line 1
   ✅ Applied default for ui_config
   ⚠️  ui_config.custom_panels is empty (no panels defined)
   
🎨 Panel Registry:
   ✅ Registered: doom_game
   ❌ No panels to create (custom_panels = [])
   
💡 Suggestion: Check if 'mode' field is present in JSON
```

### **Implementation**

```rust
pub struct DiagnosticCollector {
    warnings: Vec<String>,
    suggestions: Vec<String>,
}

impl Scenario {
    pub fn load_with_diagnostics<P: AsRef<Path>>(
        path: P,
        diagnostics: &mut DiagnosticCollector,
    ) -> Result<Self> {
        // Collect warnings during parse
        // Suggest fixes
        // Log what's happening
    }
}
```

### **Benefits**

✅ **Self-documenting failures**: System explains itself  
✅ **Faster debugging**: Know what to fix immediately  
✅ **Better user experience**: Helpful, not cryptic  
✅ **Teaching tool**: New users learn by doing  

---

## 🧬 **EVOLUTION OPPORTUNITY #5: Panel Lifecycle Management**

### **What We Discovered**

Panels have **implicit** lifecycle:
1. Created (constructor)
2. Rendered (update loop)
3. ??? (no explicit cleanup, pause, resume)

### **The Gap**

```rust
// Current:
pub trait PanelInstance {
    fn render(&mut self, ui: &mut egui::Ui);
    fn title(&self) -> &str;
    fn update(&mut self) {}  // Optional
}

// What happens when:
// - User closes panel window?
// - App goes to background?
// - System runs out of memory?
// - Panel crashes?
```

### **Evolution: Explicit Lifecycle**

```rust
pub trait PanelInstance {
    // Existing
    fn render(&mut self, ui: &mut egui::Ui);
    fn title(&self) -> &str;
    
    // NEW: Lifecycle hooks
    fn on_open(&mut self) -> Result<()> { Ok(()) }
    fn on_close(&mut self) -> Result<()> { Ok(()) }
    fn on_pause(&mut self) { }
    fn on_resume(&mut self) { }
    fn on_error(&mut self, error: &dyn Error) -> PanelAction {
        PanelAction::ShowError(error.to_string())
    }
    
    // State queries
    fn is_closable(&self) -> bool { true }
    fn is_pausable(&self) -> bool { true }
    fn can_save_state(&self) -> bool { false }
    fn save_state(&self) -> Option<Value> { None }
    fn restore_state(&mut self, state: Value) -> Result<()> { Ok(()) }
}

pub enum PanelAction {
    Continue,
    Close,
    ShowError(String),
    RequestRestart,
}
```

### **Benefits**

✅ **Resource management**: Panels can clean up properly  
✅ **Error isolation**: Panel crash ≠ app crash  
✅ **State persistence**: Save/restore panel state  
✅ **Performance**: Pause inactive panels  

---

## 🧬 **EVOLUTION OPPORTUNITY #6: Performance Budgets**

### **The Problem**

Doom wants **35 Hz** tick rate (game logic), but egui runs at **60 Hz** (rendering).

Currently: **Uncoordinated**
- Doom ticks whenever it wants
- No budget enforcement
- No coordination between panels

### **Evolution: Performance Budget System**

```rust
pub struct PerformanceBudget {
    /// Target frame time (ms)
    target_frame_time: f32,
    
    /// Panel-specific budgets
    panel_budgets: HashMap<String, PanelBudget>,
    
    /// Global budget
    total_budget: f32,
}

pub struct PanelBudget {
    /// How much CPU time this panel gets per frame (ms)
    cpu_budget: f32,
    
    /// Update rate (Hz)
    update_rate: f32,
    
    /// Last update time
    last_update: Instant,
    
    /// Priority (higher = more budget when constrained)
    priority: u8,
}

impl PanelInstance {
    fn performance_profile(&self) -> PerformanceProfile {
        PerformanceProfile {
            update_rate: 35.0,  // Hz (Doom-specific)
            cpu_budget: 5.0,    // ms per frame
            priority: 5,        // Medium priority
        }
    }
}
```

### **Benefits**

✅ **Smooth rendering**: No frame drops  
✅ **Fair resource allocation**: All panels get time  
✅ **Adaptive quality**: Degrade gracefully under load  
✅ **Battery friendly**: Don't waste CPU  

---

## 🧬 **EVOLUTION OPPORTUNITY #7: Input Focus & Routing**

### **The Problem** (Gap #4)

Multiple panels want input:
- Doom wants WASD, mouse clicks
- Graph canvas wants drag, scroll
- Panels want keyboard shortcuts

**Who gets input?**

Currently: **Whoever renders first** (implicit)

### **Evolution: Explicit Focus Management**

```rust
pub trait PanelInstance {
    // NEW: Input handling
    fn wants_keyboard_input(&self) -> bool { false }
    fn wants_mouse_input(&self) -> bool { false }
    fn wants_exclusive_input(&self) -> bool { false }  // Like games
    
    fn on_keyboard_event(&mut self, event: &KeyEvent) -> InputAction {
        InputAction::Ignored
    }
    
    fn on_mouse_event(&mut self, event: &MouseEvent) -> InputAction {
        InputAction::Ignored
    }
}

pub enum InputAction {
    Consumed,   // Panel handled it, don't pass to others
    Ignored,    // Panel didn't care, try next panel
    Global,     // Global shortcut, everyone should see
}

pub struct FocusManager {
    focused_panel: Option<String>,
    input_stack: Vec<String>,  // Stack of panels that want input
}

impl FocusManager {
    pub fn route_keyboard(&mut self, event: &KeyEvent, panels: &mut [Box<dyn PanelInstance>]) {
        // Try focused panel first
        if let Some(id) = &self.focused_panel {
            if let Some(panel) = panels.get_mut(id) {
                if panel.on_keyboard_event(event) == InputAction::Consumed {
                    return;
                }
            }
        }
        
        // Try panels in stack order
        for id in &self.input_stack {
            if let Some(panel) = panels.get_mut(id) {
                match panel.on_keyboard_event(event) {
                    InputAction::Consumed => return,
                    InputAction::Global => continue,  // Let others see too
                    InputAction::Ignored => continue,
                }
            }
        }
    }
}
```

### **Benefits**

✅ **Predictable**: Users know which panel gets input  
✅ **Flexible**: Panels can share or exclude  
✅ **Game-friendly**: Exclusive mode for games  
✅ **Accessible**: Focus can be managed programmatically  

---

## 🧬 **EVOLUTION OPPORTUNITY #8: Panel Composition**

### **The Vision**

Right now: **1 panel type = 1 window**

Future: **Compose panels into layouts**

```rust
pub enum PanelLayout {
    Single(PanelId),
    Split {
        direction: SplitDirection,
        ratio: f32,
        left: Box<PanelLayout>,
        right: Box<PanelLayout>,
    },
    Tabs(Vec<PanelId>),
    Grid {
        rows: usize,
        cols: usize,
        panels: Vec<PanelId>,
    },
}

// Example: Doom + Graph side-by-side
let layout = PanelLayout::Split {
    direction: SplitDirection::Vertical,
    ratio: 0.6,  // 60% left, 40% right
    left: Box::new(PanelLayout::Single("doom".into())),
    right: Box::new(PanelLayout::Single("graph".into())),
};
```

### **Benefits**

✅ **Flexible workspaces**: User-configurable layouts  
✅ **Multi-panel scenarios**: Rich demonstrations  
✅ **Save/restore layouts**: Persistent workspaces  
✅ **Adaptive**: Layout adapts to screen size  

---

## 🧬 **EVOLUTION OPPORTUNITY #9: Hot Reloading**

### **The Vision**

Change `doom-mvp.json` → panel updates **instantly** (no restart!)

```rust
pub struct ScenarioWatcher {
    path: PathBuf,
    last_modified: SystemTime,
    watcher: Notify,
}

impl ScenarioWatcher {
    pub fn check_reload(&mut self, app: &mut PetalTongueApp) -> Result<()> {
        if self.has_changed() {
            tracing::info!("🔄 Scenario changed, reloading...");
            
            let scenario = Scenario::load(&self.path)?;
            app.reload_scenario(scenario)?;
            
            tracing::info!("✅ Scenario reloaded");
        }
        Ok(())
    }
}

impl PetalTongueApp {
    pub fn reload_scenario(&mut self, scenario: Scenario) -> Result<()> {
        // Close old panels
        for panel in &mut self.custom_panels {
            panel.on_close()?;
        }
        
        // Create new panels
        self.custom_panels.clear();
        for config in &scenario.ui_config.custom_panels {
            let panel = self.panel_registry.create(config)?;
            self.custom_panels.push(panel);
        }
        
        Ok(())
    }
}
```

### **Benefits**

✅ **Rapid iteration**: See changes instantly  
✅ **Live demos**: Change scenarios during presentation  
✅ **Debugging**: Tweak configs without restart  
✅ **TRUE PRIMAL**: Live evolution in action!  

---

## 🧬 **EVOLUTION OPPORTUNITY #10: Panel Discovery**

### **The Vision**

Panels shouldn't be compiled in! Discover them at runtime:

```rust
// Panels as plugins (dynamic libraries)
pub trait PanelPlugin {
    fn panel_type(&self) -> &str;
    fn create_factory(&self) -> Arc<dyn PanelFactory>;
}

// Load from directory
let plugin_dir = "/usr/local/lib/petaltongue/panels/";
for entry in fs::read_dir(plugin_dir)? {
    let lib = unsafe { Library::new(entry.path())? };
    let plugin: Symbol<fn() -> Box<dyn PanelPlugin>> = 
        unsafe { lib.get(b"create_plugin")? };
    
    let panel_plugin = plugin();
    registry.register(panel_plugin.create_factory());
}
```

**OR** (safer, pure Rust):

```rust
// Panels as separate crates
// Discovered via Cargo workspace + feature flags

[dependencies]
doom-panel = { path = "../panels/doom-panel", optional = true }
web-panel = { path = "../panels/web-panel", optional = true }
video-panel = { path = "../panels/video-panel", optional = true }

[features]
default = ["doom-panel"]
all-panels = ["doom-panel", "web-panel", "video-panel"]
```

### **Benefits**

✅ **Extensible**: Users can add panels  
✅ **Modular**: Only compile what you need  
✅ **Sovereign**: Each panel is independent  
✅ **Ecosystem**: Community can contribute panels  

---

## 📊 **PRIORITY MATRIX**

Which evolutions should we do **before** completing Doom challenge?

| Priority | Evolution | Why Before Doom | Effort |
|----------|-----------|-----------------|---------|
| 🔴 **Critical** | Validation Layer | Will catch more gaps early | 2 days |
| 🔴 **Critical** | Error Reporting | Essential for debugging | 1 day |
| 🟡 **High** | Lifecycle Hooks | Doom needs cleanup | 3 days |
| 🟡 **High** | Input Focus | Doom needs keyboard | 2 days |
| 🟢 **Medium** | Performance Budgets | Nice to have, not blocking | 4 days |
| 🟢 **Medium** | Schema Migration | Future-proofing | 3 days |
| ⚪ **Low** | Panel Composition | After basic Doom works | 5 days |
| ⚪ **Low** | Hot Reloading | Quality of life | 2 days |
| ⚪ **Low** | Runtime Diagnostics | Helps debugging | 2 days |
| ⚪ **Deferred** | Panel Discovery | Much later | 10 days |

---

## 🎯 **RECOMMENDED EVOLUTION SEQUENCE**

### **Before Real Doom (Week 1)**

1. **Validation Layer** (2 days)
   - Add `Scenario::validate()`
   - Add `UiConfig::validate()`
   - Add `CustomPanelConfig::validate()`
   - Tests for all validation rules

2. **Better Error Messages** (1 day)
   - Context-rich errors
   - Suggestions for fixes
   - File/line numbers

3. **Input Focus System** (2 days)
   - `FocusManager`
   - Input routing
   - Keyboard event handling
   - Tests for focus scenarios

### **During Doom Integration (Week 2)**

4. **Lifecycle Hooks** (3 days)
   - Add to `PanelInstance` trait
   - Implement in `DoomPanel`
   - Error isolation
   - Tests for lifecycle

5. **Performance Budgets** (2 days)
   - Basic budget system
   - Per-panel timing
   - Adaptive quality
   - Benchmarks

### **After Doom Works (Week 3+)**

6. **Schema Migration** (3 days)
7. **Panel Composition** (5 days)
8. **Hot Reloading** (2 days)
9. **Runtime Diagnostics** (2 days)

---

## 💡 **KEY INSIGHTS**

### **1. Test-Driven Evolution Works!**

We predicted 10 gaps. We found **5 different gaps**, including one (Silent Deserialization) that wasn't on our list!

**Lesson**: You can't predict all problems. Build → Discover → Solve.

### **2. "Successfully Fail" Reveals Architecture**

The Doom integration **probe** exposed fundamental gaps:
- Validation missing
- Error messages poor
- Lifecycle implicit
- Input routing ad-hoc
- Performance unmanaged

**Without building Doom, we wouldn't have found these!**

### **3. Each Gap is an Evolution Opportunity**

Every "failure" revealed:
- What's missing (validation, error messages)
- What's implicit (lifecycle, focus)
- What's uncoordinated (performance, input)

**Each makes petalTongue stronger!**

### **4. The Panel System is Universal**

Doom proves the pattern works. Now we can embed:
- Web browsers
- Video players
- Terminals
- IDEs
- Debuggers
- Monitoring tools
- **Anything!**

---

## 🚀 **NEXT STEPS**

1. **Implement Critical Evolutions** (Validation, Errors, Focus)
2. **Complete Doom Integration** (Real doomgeneric)
3. **Document Lessons** (Update specs with discoveries)
4. **Build More Panels** (Web, Video, Terminal)

---

**Version**: Post-MVP Insights  
**Date**: January 15, 2026  
**Status**: Foundation Complete, Evolution Path Clear  

🌸 **Every gap discovered is a gift!** 🌸

