# 🔴 DEEP ARCHITECTURAL DEBT: Live Evolution

**Date**: January 15, 2026  
**Status**: 🔴 CRITICAL - Current implementation is **brittle and static**  
**Impact**: petalTongue cannot evolve across devices, schemas, or time

---

## 🐛 The Deep Problem

### User's Vision (TRUE PRIMAL):
> "petalTongue will often be a fully live and evolving UI. JSONs can change, inputs and outputs change. Maybe I start on a computer, move to a phone while driving, and then want a biosensor like a watch for daily life. petalTongue lives and evolves."

### Current Reality (BRITTLE):

**❌ Static JSON Schema** (scenario.rs):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalDefinition {
    pub id: String,
    pub name: String,
    pub primal_type: String,
    pub family: String,
    pub status: String,
    pub health: u8,
    pub confidence: u8,
    pub position: Position,
    pub capabilities: Vec<String>,
    pub metrics: PrimalMetrics,
    pub proprioception: Option<ScenarioProprioception>,
}
```

**Problems**:
1. **Recompilation Required**: New field → recompile → redeploy
2. **No Unknown Fields**: Can't capture future data
3. **Device Assumptions**: Desktop-centric (1400x900 window)
4. **No State Sync**: Can't move from desktop → phone → watch
5. **Hardcoded UI**: Components assume specific data shapes

---

## 🏗️ Architectural Violations

### TRUE PRIMAL Principles Violated:

| Principle | Current State | Required State |
|-----------|---------------|----------------|
| **Zero Hardcoding** | ❌ 10+ hardcoded structs | ✅ Dynamic schema discovery |
| **Capability-Based** | ❌ Assumes desktop UI | ✅ Discovers device capabilities |
| **Live Evolution** | ❌ Static at compile-time | ✅ Adapts at runtime |
| **Self-Knowledge Only** | ❌ Knows about PrimalDefinition | ✅ Only knows own capabilities |
| **Graceful Degradation** | ⚠️ Partial (fails on unknown fields) | ✅ Renders what it can |

---

## 📊 Brittle Points Identified

### 1. **Static Schema Definitions** (10+ structs)

**Files**:
- `crates/petal-tongue-ui/src/scenario.rs` (240 lines)
  - `Scenario`, `UiConfig`, `AnimationConfig`, `PerformanceConfig`
  - `Ecosystem`, `PrimalDefinition`, `Position`, `PrimalMetrics`
  - `ScenarioProprioception`, `SelfAwareness`, `Motor`, `Sensory`
  - `NeuralApiConfig`

**Problem**: Every struct field is **compiled in**. New fields require:
1. Edit Rust struct
2. Recompile (12s+ build)
3. Redeploy binary
4. Restart UI

**Impact**: 
- Can't evolve JSON schemas dynamically
- Can't handle versioned schemas
- Can't capture unknown/future fields

### 2. **Device-Specific Assumptions**

**Files**:
- `crates/petal-tongue-ui/src/main.rs:126-133`

```rust
let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default()
        .with_inner_size([1400.0, 900.0])  // ❌ HARDCODED desktop size
        .with_min_inner_size([800.0, 600.0])
        .with_title("🌸 petalTongue - Universal Representation System")
        .with_visible(true),
    ..Default::default()
};
```

**Problem**: Assumes desktop with large screen

**Missing**:
- Phone UI (320x568 to 428x926)
- Watch UI (184x224 to 368x448)
- Tablet UI (768x1024 to 1024x1366)
- TV UI (1920x1080 to 3840x2160)

### 3. **No State Synchronization**

**Missing**:
- Session persistence across devices
- State migration (desktop → phone → watch)
- Bookmark/resume functionality
- Cross-device coordination

**User Story FAILS**:
> "Start on computer, move to phone while driving, then watch for daily life"

**Why**: No state sync, no device discovery, no handoff protocol

### 4. **Static UI Rendering**

**Files**:
- `crates/petal-tongue-ui/src/app_panels.rs` (500+ lines of hardcoded panels)
- `crates/petal-tongue-ui/src/graph_canvas.rs` (assumes mouse/keyboard)
- `crates/petal-tongue-ui/src/node_palette.rs` (assumes drag-and-drop)

**Problem**: UI components assume:
- Large screen
- Mouse + keyboard input
- Full egui capabilities
- Specific data structures

**Fails On**:
- Phone (touch-only, small screen)
- Watch (tiny screen, glance interactions)
- Voice interface (no visual)
- CLI (no graphics)

### 5. **No Schema Versioning**

**Current**:
```json
{
  "name": "Live Ecosystem",
  "version": "1.0.0",  // ❌ Ignored! Just metadata
  "mode": "live-ecosystem"
}
```

**Problem**: `version` field is ignored, no migration logic

**Missing**:
- Schema version detection
- Migration functions (v1 → v2 → v3)
- Backward compatibility checks
- Forward compatibility (ignore unknown)

### 6. **No Hot-Reload**

**Current**: Static file load on startup

**Missing**:
- File watch (inotify/FSEvents)
- Dynamic reload without restart
- Partial updates (only changed data)
- Live schema evolution

---

## 🎯 Required Solutions

### Solution 1: **Dynamic Schema System** ✅ CRITICAL

**Replace**:
```rust
// BEFORE (Brittle)
#[derive(Deserialize)]
pub struct PrimalDefinition {
    pub id: String,
    pub name: String,
    // ... 10 more fields
}
```

**With**:
```rust
// AFTER (Adaptive)
pub struct DynamicPrimal {
    /// Known/required fields (minimal)
    pub id: String,
    
    /// All data (including unknown fields)
    pub data: HashMap<String, DynamicValue>,
    
    /// Schema version
    pub schema_version: SchemaVersion,
}

pub enum DynamicValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<DynamicValue>),
    Object(HashMap<String, DynamicValue>),
    Null,
}
```

**Benefits**:
- ✅ Captures **all** fields (known and unknown)
- ✅ No recompilation for new fields
- ✅ Forward/backward compatible
- ✅ Can migrate between versions

### Solution 2: **Capability-Based Rendering** ✅ CRITICAL

**Add**:
```rust
pub struct RenderingCapabilities {
    /// What modalities are available?
    pub modalities: Vec<Modality>,
    
    /// Screen size (if visual)
    pub screen_size: Option<(f32, f32)>,
    
    /// Input methods (touch, mouse, keyboard, voice)
    pub input_methods: Vec<InputMethod>,
    
    /// Performance tier (low, medium, high)
    pub performance_tier: PerformanceTier,
}

pub enum Modality {
    Visual2D { resolution: (f32, f32) },
    Visual3D { gpu: bool },
    Audio { sample_rate: u32 },
    Haptic { precision: HapticPrecision },
    CLI { color: bool },
}

pub trait AdaptiveRenderer {
    /// Can this renderer handle these capabilities?
    fn supports(&self, caps: &RenderingCapabilities) -> bool;
    
    /// Render with available capabilities
    fn render_adaptive(&self, data: &DynamicPrimal, caps: &RenderingCapabilities);
}
```

**Example**:
```rust
// Desktop: Full UI
if caps.screen_size >= (800.0, 600.0) && caps.has_mouse() {
    render_full_ui();
}
// Phone: Touch-optimized
else if caps.screen_size >= (320.0, 568.0) && caps.has_touch() {
    render_mobile_ui();
}
// Watch: Glance UI
else if caps.screen_size >= (184.0, 224.0) {
    render_glance_ui();
}
// Fallback: CLI
else {
    render_cli();
}
```

### Solution 3: **State Synchronization** ✅ HIGH

**Add**:
```rust
pub struct DeviceState {
    /// Device ID (persistent across sessions)
    pub device_id: String,
    
    /// Device type (desktop, phone, watch, etc.)
    pub device_type: DeviceType,
    
    /// Current UI state
    pub ui_state: HashMap<String, DynamicValue>,
    
    /// Timestamp
    pub last_updated: DateTime<Utc>,
}

pub trait StateSync {
    /// Save state to persistent storage
    fn save_state(&self, state: &DeviceState) -> Result<()>;
    
    /// Load state from persistent storage
    fn load_state(&self, device_id: &str) -> Result<Option<DeviceState>>;
    
    /// Sync state across devices (via Neural API or local)
    async fn sync_state(&self) -> Result<DeviceState>;
}
```

**Storage Options**:
1. **Local**: `~/.config/petalTongue/state.json`
2. **Neural API**: Centralized state via biomeOS
3. **Distributed**: CRDT-based sync across devices

### Solution 4: **Schema Versioning & Migration** ✅ HIGH

**Add**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SchemaVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

pub trait SchemaMigration {
    /// Can this migration handle version X → Y?
    fn can_migrate(&self, from: SchemaVersion, to: SchemaVersion) -> bool;
    
    /// Migrate data from old schema to new schema
    fn migrate(&self, data: DynamicValue, from: SchemaVersion) -> Result<DynamicValue>;
}

// Example migrations
pub struct PrimalV1ToV2;
impl SchemaMigration for PrimalV1ToV2 {
    fn can_migrate(&self, from: SchemaVersion, to: SchemaVersion) -> bool {
        from.major == 1 && to.major == 2
    }
    
    fn migrate(&self, data: DynamicValue) -> Result<DynamicValue> {
        // v1: { "status": "healthy" }
        // v2: { "health": { "status": "healthy", "percentage": 100 } }
        // ... migration logic ...
    }
}
```

### Solution 5: **Hot-Reload System** ✅ MEDIUM

**Add**:
```rust
pub struct HotReloader {
    /// File paths being watched
    watched_files: Vec<PathBuf>,
    
    /// File watcher (inotify/FSEvents)
    watcher: notify::RecommendedWatcher,
    
    /// Reload callback
    on_reload: Box<dyn Fn(PathBuf, DynamicValue)>,
}

impl HotReloader {
    /// Watch a file for changes
    pub fn watch(&mut self, path: PathBuf) -> Result<()>;
    
    /// Handle file change event
    fn on_file_changed(&self, path: PathBuf) -> Result<()> {
        let data = load_dynamic_json(&path)?;
        (self.on_reload)(path, data);
        Ok(())
    }
}
```

**Usage**:
```rust
// Watch scenario file for changes
hot_reloader.watch("scenarios/live-ecosystem.json")?;

// When file changes:
// 1. Parse new JSON
// 2. Migrate schema (if needed)
// 3. Update UI (without restart)
```

### Solution 6: **Device Discovery & Handoff** ✅ MEDIUM

**Add**:
```rust
pub struct DeviceCoordinator {
    /// All known devices for this user/family
    devices: HashMap<String, DeviceInfo>,
    
    /// Active device (currently in use)
    active_device: Option<String>,
}

pub struct DeviceInfo {
    pub id: String,
    pub device_type: DeviceType,
    pub capabilities: RenderingCapabilities,
    pub last_seen: DateTime<Utc>,
    pub state: Option<DeviceState>,
}

pub trait DeviceHandoff {
    /// Transfer state from this device to another
    async fn handoff_to(&self, target_device: &str) -> Result<()>;
    
    /// Accept state from another device
    async fn receive_handoff(&self, from_device: &str) -> Result<DeviceState>;
}
```

**User Story**:
```rust
// Desktop: User working on graph
desktop.handoff_to("my-phone").await?;

// Phone: Receives state, continues work
phone.receive_handoff("my-desktop").await?;

// Watch: Minimal monitoring UI
watch.receive_handoff("my-phone").await?;
```

---

## 📈 Implementation Priority

### Phase 1: **Foundation** (CRITICAL)
1. ✅ Dynamic schema system (`DynamicValue`, `DynamicPrimal`)
2. ✅ Schema versioning (`SchemaVersion`, `SchemaMigration`)
3. ✅ Capability detection (`RenderingCapabilities`)

### Phase 2: **Adaptive Rendering** (HIGH)
4. ✅ Device-specific renderers (Desktop, Phone, Watch, CLI)
5. ✅ Adaptive component system (`AdaptiveRenderer`)
6. ✅ Layout engine (responsive, device-aware)

### Phase 3: **State Persistence** (HIGH)
7. ✅ Local state storage (`~/.config/petalTongue/state.json`)
8. ✅ State sync via Neural API
9. ✅ Cross-device coordination (`DeviceCoordinator`)

### Phase 4: **Live Evolution** (MEDIUM)
10. ✅ Hot-reload system (`HotReloader`)
11. ✅ File watching (inotify/FSEvents)
12. ✅ Partial updates (delta sync)

---

## 🎨 Example: Desktop → Phone → Watch

### Scenario: User monitors ecosystem health

**Desktop** (Full UI):
```rust
// Large screen, mouse, keyboard
RenderingCapabilities {
    modalities: [Visual2D { resolution: (1400, 900) }],
    screen_size: Some((1400.0, 900.0)),
    input_methods: [Mouse, Keyboard],
    performance_tier: High,
}

// Renders:
// - Full topology graph (8 primals)
// - Detailed property panels
// - Graph builder canvas
// - Multiple side panels
```

**Phone** (Driving):
```rust
// Small screen, touch-only, voice commands
RenderingCapabilities {
    modalities: [Visual2D { resolution: (428, 926) }, Audio { sample_rate: 48000 }],
    screen_size: Some((428.0, 926.0)),
    input_methods: [Touch, Voice],
    performance_tier: Medium,
}

// Renders:
// - Simplified list view (8 primals)
// - Health status badges (red/yellow/green)
// - Voice: "All systems healthy"
// - Tap for details
```

**Watch** (Daily Life):
```rust
// Tiny screen, glance interaction
RenderingCapabilities {
    modalities: [Visual2D { resolution: (368, 448) }, Haptic { precision: High }],
    screen_size: Some((368.0, 448.0)),
    input_methods: [Touch, Crown, Haptic],
    performance_tier: Low,
}

// Renders:
// - Single metric: "8/8 Healthy ✅"
// - Tap: Cycles through primals
// - Haptic: Vibrate on critical status
```

---

## 🔮 Future Vision

### Live Evolution Scenarios:

1. **Schema Evolution**:
   ```
   v1: { "status": "healthy" }
   v2: { "health": { "status": "healthy", "percentage": 100 } }
   v3: { "health": { "status": "healthy", "percentage": 100, "trend": "improving" } }
   ```
   - UI adapts to new fields **without recompilation**
   - Renders `trend` if available, gracefully degrades if not

2. **Device Continuity**:
   ```
   Desktop → Editing graph builder
   Phone   → Receives notification, opens simplified view
   Watch   → Glances at execution status
   Desktop → Resumes editing (state preserved)
   ```

3. **Modality Switching**:
   ```
   Visual (Desktop)  → "8 primals, all healthy"
   Audio (Car)       → "All systems healthy. NUCLEUS at 8% CPU."
   Haptic (Watch)    → Gentle pulse (healthy), strong vibrate (critical)
   CLI (SSH)         → "✅ 8/8 primals healthy"
   ```

4. **JSON Hot-Reload**:
   ```
   # Edit scenario while UI is running
   vim scenarios/live-ecosystem.json
   # Add new primal: { "id": "newbird", "name": "NewBird" }
   
   # UI automatically:
   # 1. Detects file change (inotify)
   # 2. Reloads JSON
   # 3. Migrates schema (if needed)
   # 4. Adds new primal to topology
   # 5. No restart required!
   ```

---

## 🏆 Success Criteria

### Must Have:
- ✅ Dynamic schema (no hardcoded structs)
- ✅ Schema versioning & migration
- ✅ Capability-based rendering
- ✅ Device detection (desktop, phone, watch)
- ✅ State persistence (local)

### Should Have:
- ✅ Hot-reload (file watching)
- ✅ State sync (Neural API)
- ✅ Adaptive UI (responsive layouts)
- ✅ Multi-modal rendering (visual, audio, haptic)

### Nice to Have:
- ✅ Device handoff (desktop → phone)
- ✅ Distributed state (CRDT)
- ✅ Voice interface
- ✅ AR/VR support

---

## 📚 Related Work

### Existing Components (Reusable):

1. **`universal_discovery.rs`** ✅
   - Already implements zero-hardcoding discovery
   - Can be adapted for device discovery

2. **`Properties` + `PropertyValue`** ✅
   - Already dynamic key-value system
   - Close to `DynamicValue` concept

3. **`AdapterRegistry`** ✅
   - Already renders unknown property types
   - Foundation for adaptive rendering

4. **`Modality` enum** ✅
   - Already defines Visual2D, Audio, etc.
   - Foundation for multi-modal support

### New Components (Required):

1. **`DynamicScenario`** - Schema-agnostic data model
2. **`SchemaVersion`** - Versioning and migration
3. **`DeviceCoordinator`** - Multi-device state sync
4. **`HotReloader`** - File watching and live reload
5. **`AdaptiveLayout`** - Responsive UI engine

---

## 🚀 Next Steps

### Immediate Actions:

1. **Create `DynamicScenario` system** (replace hardcoded structs)
2. **Add schema versioning** (detect and migrate)
3. **Implement device detection** (desktop, phone, watch)
4. **Create adaptive renderers** (device-specific UI)
5. **Add state persistence** (local file storage)

### Documentation:

- ✅ This analysis document
- 🔄 Architecture evolution plan
- 🔄 Migration guide (old → new)
- 🔄 Device support matrix

---

**Conclusion**: The current implementation is **fundamentally brittle**. It violates TRUE PRIMAL principles by hardcoding schemas, assuming desktop UI, and lacking live evolution. We need a **complete architectural refactor** to enable true adaptivity and cross-device continuity.

**Priority**: 🔴 CRITICAL - This is not a "nice to have", it's **core to petalTongue's mission** as a universal, adaptive interface for ecoPrimals.

