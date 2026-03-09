# Interaction Engine Architecture

**Version**: 1.0.0
**Date**: March 9, 2026
**Status**: Design Phase
**Priority**: High (Completes Bidirectional Pipeline)
**Depends On**: `GRAMMAR_OF_GRAPHICS_ARCHITECTURE.md`, `UNIVERSAL_VISUALIZATION_PIPELINE.md`,
`BIDIRECTIONAL_UUI_ARCHITECTURE.md`

---

## 1. Vision: The "6 vs 9" Problem

Two people stand on opposite sides of a symbol drawn on the ground. One says
"it's a 6." The other says "it's a 9." Both are correct from their perspective.
The problem is not ambiguity in the symbol -- it is the absence of a shared
understanding of the underlying object independent of any single viewpoint.

petalTongue solves this. The underlying object is the data. Perspectives are
how different humans -- with different sensory capabilities, different positions
in space, different cognitive models -- perceive and interact with that data.

A deaf person, a blind person, and a sighted person working together on the
same dataset should each:

1. Interact through their own channels (keyboard, audio, visual, haptic, voice)
2. Perceive each other's interactions in their own modality
3. All understand the same underlying data objects

When the sighted user clicks a node, the blind user hears a tone change and
the deaf user sees a highlight. They are all pointing at the same data row.
No one's perspective is more "real" than another's. The system maintains the
identity of the data object across all perspectives.

This is not an accessibility feature bolted on after the fact. It is the
architecture. If the interaction engine works for three people with different
sensory systems, it works for everyone.

---

## 2. Semantic Intent Model

### The Abstraction Layers

Device events are NOT interactions. They are physical signals from hardware.
The interaction engine translates them through three layers:

```
Layer 0: Device Event
    Mouse click at (423, 187), Key 'Enter', Audio amplitude spike at t=3.2s

Layer 1: Semantic Intent
    Select, Inspect, Navigate, Manipulate, Annotate, Command

Layer 2: Data Operation
    Filter rows where primal_id = "songbird-alpha"
    Zoom to time range [14:20, 14:25]
    Set health_threshold = 80

Layer 3: Shared State Change
    Broadcast to all perspectives and subscribers
```

Layer 1 is where modalities converge. A mouse click and a keyboard Enter and
a voice command "select that" and a Braille display button press all produce
the same `InteractionIntent::Select`. The device is irrelevant. The intent is
universal.

### InteractionIntent

```rust
pub enum InteractionIntent {
    Select {
        target: InteractionTarget,
        mode: SelectionMode,
    },
    Inspect {
        target: InteractionTarget,
        depth: InspectionDepth,
    },
    Navigate {
        direction: NavigationDirection,
        magnitude: f64,
    },
    Manipulate {
        target: InteractionTarget,
        operation: ManipulationOp,
    },
    Annotate {
        target: InteractionTarget,
        content: AnnotationContent,
    },
    Command {
        verb: String,
        arguments: serde_json::Value,
    },
    Focus {
        target: InteractionTarget,
    },
    Dismiss,
}

pub enum SelectionMode {
    Replace,
    Add,
    Remove,
    Toggle,
}

pub enum InspectionDepth {
    Summary,
    Detail,
    Raw,
}

pub enum NavigationDirection {
    Forward,
    Backward,
    Up,
    Down,
    Left,
    Right,
    In,
    Out,
    ToData { target: DataTarget },
}

pub enum ManipulationOp {
    Move { delta: [f64; 3] },
    Resize { factor: f64 },
    Reorder { before: DataTarget },
    SetValue { field: String, value: serde_json::Value },
}
```

### InteractionTarget

An `InteractionTarget` is a reference that can be resolved to data through
any modality's inverse pipeline.

```rust
pub enum InteractionTarget {
    DataRow { data_id: DataObjectId },
    DataRange { variable: String, range: (Value, Value) },
    DataSet { predicate: FilterExpr },
    Region { bounds: BoundingBox },
    Primitive { primitive_id: PrimitiveId },
    Nothing,
}

pub struct DataObjectId {
    pub source: DataSourceId,
    pub row_key: serde_json::Value,
}
```

The critical property: `DataObjectId` is perspective-invariant. It refers to
the same data row regardless of which modality resolved it. When shared over
IPC, any petalTongue instance can highlight the corresponding object in its
own rendering.

### InteractionResult

```rust
pub struct InteractionResult {
    pub intent: InteractionIntent,
    pub resolved_target: InteractionTarget,
    pub state_changes: Vec<StateChange>,
    pub perspective: PerspectiveId,
    pub timestamp: Instant,
}

pub enum StateChange {
    SelectionChanged { selected: Vec<DataObjectId> },
    FilterChanged { variable: String, range: Option<(Value, Value)> },
    ViewChanged { viewport: Viewport },
    DataMutated { source: DataSourceId, mutation: DataMutation },
    AnnotationAdded { target: DataObjectId, content: AnnotationContent },
}
```

---

## 3. Input Adapters

Each input modality implements the `InputAdapter` trait to convert raw device
events into semantic intents.

```rust
pub trait InputAdapter: Send + Sync {
    fn name(&self) -> &str;
    fn modality(&self) -> InputModality;
    fn capabilities(&self) -> Vec<InteractionCapability>;

    fn translate(
        &self,
        event: DeviceEvent,
        context: &InteractionContext,
    ) -> Option<InteractionIntent>;

    fn active_target(&self, context: &InteractionContext) -> Option<InteractionTarget>;

    fn feedback(&self, result: &InteractionResult);
}

pub enum InputModality {
    PointerMouse,
    PointerTouch,
    PointerStylus,
    Keyboard,
    Gamepad,
    VoiceCommand,
    BrailleDisplay,
    SwitchAccess,
    EyeGaze,
    MotionCapture,
    Custom(String),
}

pub enum InteractionCapability {
    PointSelect,
    RangeSelect,
    FreeformSelect,
    Navigate2D,
    Navigate3D,
    TextInput,
    ContinuousValue,
    DiscreteChoice,
}
```

### Adapter Implementations

**PointerAdapter** (mouse, touch, stylus):
- Click -> `Select { target: resolve_at_position(pos) }`
- Double-click -> `Inspect { target: resolve_at_position(pos) }`
- Drag -> `Navigate { direction: delta }` or `Select { mode: brush }`
- Scroll -> `Navigate { direction: In/Out }`
- Hover -> `Focus { target: resolve_at_position(pos) }`

**KeyboardAdapter**:
- Enter -> `Select { target: currently_focused }`
- Tab -> `Focus { target: next_in_order }`
- Arrow keys -> `Navigate { direction }`
- Delete -> `Manipulate { operation: Remove }`
- / -> `Command { verb: "search" }`

**AudioInputAdapter** (voice commands):
- "select songbird" -> `Select { target: resolve_by_name("songbird") }`
- "zoom in" -> `Navigate { direction: In }`
- "what is this" -> `Inspect { target: currently_focused }`
- "go to 14:23" -> `Navigate { direction: ToData { time: 14:23 } }`

**BrailleAdapter** (Braille display with routing keys):
- Routing key press -> `Select { target: resolve_at_cell(cell) }`
- Panning keys -> `Navigate { direction }`
- Status key -> `Inspect { target: currently_focused }`

**GamepadAdapter** (controller):
- Left stick -> `Navigate { direction, magnitude }`
- A button -> `Select { target: currently_focused }`
- Right trigger -> `Navigate { direction: In, magnitude: pressure }`
- Bumpers -> `Focus { target: next/previous }`

The adapter list is extensible. Any struct implementing `InputAdapter` can be
registered at runtime via `SensorRegistry`.

---

## 4. Generalized Inverse Pipeline

The Grammar of Graphics pipeline maps data to visual/auditory/haptic output.
The inverse pipeline maps interactions BACK to data. Each modality has its own
inverse path, but all converge to the same `DataTarget`.

### Visual Inverse (egui, SVG, web)

```
Pixel coordinate (423, 187)
    -> Viewport normalize to [0, 1]
    -> Panel hit test (which facet?)
    -> Inverse CoordinateSystem (Cartesian, Polar, Perspective)
    -> Inverse Scale per axis (linear, log, temporal, categorical)
    -> Data-space values (time=14:23, cpu=67.2)
    -> Nearest primitive (by distance in data space)
    -> DataObjectId (source=health_metrics, row=42)
```

### Audio Inverse (sonification)

```
Time offset in soundscape (3.2 seconds into the render)
    -> Which sonic element is playing (tone_id=7)
    -> Inverse sonification mapping
        pitch -> data value (health=85)
        pan -> data position (x=0.3 -> primal "songbird")
        timbre -> data category (type="discovery")
    -> DataObjectId (source=topology, row="songbird-alpha")
```

### TUI Inverse (ratatui, terminal)

```
Cursor position (row=12, col=34)
    -> Character cell content lookup
    -> Cell -> RenderPlan primitive mapping
    -> Inverse character-space to data-space
        Braille dot position -> approximate continuous value
        Block character -> binned range
        Text label -> categorical value
    -> DataObjectId
```

### Haptic Inverse (Braille display, force feedback)

```
Braille cell or force position
    -> Tactile element mapping
    -> Inverse haptic encoding
        raised dot pattern -> value approximation
        force magnitude -> distance to data object
    -> DataObjectId
```

### Voice/Command Inverse

```
Parsed command: "select the unhealthy primal"
    -> Entity resolution against current DataSource
        "unhealthy" -> filter: status != "healthy"
        "primal" -> entity type constraint
    -> Matching DataObjectIds (may be multiple)
```

### The Inverse Pipeline Trait

```rust
pub trait InversePipeline: Send + Sync {
    fn modality(&self) -> OutputModality;

    fn resolve_at(
        &self,
        event: &DeviceEvent,
        plan: &RenderPlan,
        context: &InteractionContext,
    ) -> Option<InteractionTarget>;

    fn nearest_primitive(
        &self,
        target: &InteractionTarget,
        plan: &RenderPlan,
    ) -> Option<PrimitiveId>;

    fn data_values_at(
        &self,
        target: &InteractionTarget,
        plan: &RenderPlan,
    ) -> Option<DataRow>;
}
```

Each `ModalityCompiler` from the Grammar of Graphics pipeline produces a
corresponding `InversePipeline`. The compiler knows the forward mapping; the
inverse is its mirror.

---

## 5. Perspective System

### What Is a Perspective?

A `Perspective` is the complete context through which a human perceives and
interacts with data. Two people using the same petalTongue instance with
different settings have different perspectives. One person using GUI and
another using TUI have different perspectives. They see different
representations of the same truth.

```rust
pub struct Perspective {
    pub id: PerspectiveId,
    pub modalities: Vec<ActiveModality>,
    pub viewport: Viewport,
    pub filters: Vec<FilterExpr>,
    pub coordinate_orientation: Orientation,
    pub scale_ranges: HashMap<String, (Value, Value)>,
    pub selection: Vec<DataObjectId>,
    pub focus: Option<DataObjectId>,
    pub user: Option<UserId>,
}

pub struct ActiveModality {
    pub output: OutputModality,
    pub input: Vec<InputModality>,
    pub inverse: Box<dyn InversePipeline>,
}

pub enum Orientation {
    Default,
    Rotated(f64),
    Mirrored(Axis),
    Custom(Transform),
}
```

### Perspective-Invariant Identity

The "6 vs 9" problem is solved by the distinction between:

- **DataObject**: The underlying truth. `primal_id = "songbird-alpha"` is
  the same DataObject regardless of how it is rendered.
- **Primitive**: A rendered representation within a specific perspective.
  The same DataObject produces a green circle in egui, a `[S]` character in
  TUI, a C4 tone in audio, and a raised dot pattern on a Braille display.
  These are four different Primitives of one DataObject.

When User A selects a DataObject, the system resolves the DataObjectId, not
the Primitive. User B receives `SelectionChanged { selected: [songbird-alpha] }`
and highlights the corresponding Primitive in their own perspective.

```
User A (sighted, egui):
    clicks green circle at (423, 187)
    -> Visual inverse -> DataObjectId("songbird-alpha")
    -> Broadcast: SelectionChanged

User B (blind, audio):
    hears selection tone on the C4 note (mapped to songbird)
    -> Understands: "songbird-alpha" is now selected

User C (deaf, TUI + Braille):
    sees [S] cell highlighted, Braille display shows "songbird-alpha healthy"
    -> Understands: "songbird-alpha" is now selected
```

All three users are now pointing at the same data row. They can discuss it,
annotate it, manipulate it. The perspective is different. The truth is shared.

### Perspective Synchronization

Perspectives can be linked or independent:

```rust
pub enum PerspectiveSync {
    Independent,
    SharedSelection,
    SharedViewport,
    FullSync,
}
```

- `Independent`: Each perspective is isolated. Useful for comparison.
- `SharedSelection`: Selection in one perspective highlights in all others.
  This is the default for multi-user collaboration.
- `SharedViewport`: Zoom, pan, and filter changes propagate across perspectives.
- `FullSync`: All state changes propagate. Useful for instructor/student or
  pair programming on data.

---

## 6. Multi-User Collaboration Protocol

### Local Multi-Modality

A single user can have multiple active modalities simultaneously:
- GUI window + audio sonification + haptic feedback
- TUI + screen reader (audio)
- Web browser + audio description

All modalities share one `Perspective` and receive the same `InteractionResult`
events. The input adapters for all active input modalities feed into the same
intent resolution pipeline.

### Remote Collaboration (IPC)

Multiple petalTongue instances (or other primals) can share interaction state
over JSON-RPC.

**Subscribe to interaction events:**

```json
{
  "jsonrpc": "2.0",
  "method": "visualization.interact.subscribe",
  "params": {
    "grammar_id": "health_overview",
    "events": ["select", "focus", "filter", "annotate"],
    "callback_method": "my_primal.on_interaction"
  },
  "id": 1
}
```

**Receive interaction events:**

```json
{
  "jsonrpc": "2.0",
  "method": "visualization.interact",
  "params": {
    "event": "select",
    "targets": [
      {"source": "health_metrics", "row_key": {"primal_id": "songbird-alpha"}}
    ],
    "perspective_id": "user_a_egui",
    "grammar_id": "health_overview",
    "timestamp": "2026-03-09T14:23:00Z"
  }
}
```

**Emit interaction events (from another primal to petalTongue):**

Other primals can drive petalTongue's selection state:

```json
{
  "jsonrpc": "2.0",
  "method": "visualization.interact.apply",
  "params": {
    "intent": "select",
    "targets": [
      {"source": "health_metrics", "row_key": {"primal_id": "songbird-alpha"}}
    ]
  },
  "id": 2
}
```

This enables patterns like: Squirrel's AI detects an anomaly and tells
petalTongue to highlight it. The human sees the highlight and can investigate.

### IPC Methods (Complete)

| Method | Direction | Purpose |
|--------|-----------|---------|
| `visualization.interact` | Outbound | Report user interaction events |
| `visualization.interact.subscribe` | Inbound | Subscribe to interaction events |
| `visualization.interact.apply` | Inbound | Programmatically trigger an interaction |
| `visualization.interact.perspectives` | Inbound | List active perspectives |
| `visualization.interact.sync` | Inbound | Set perspective synchronization mode |

---

## 7. The Interaction Loop

petalTongue runs a game-engine-style tick loop. Every frame:

```
1. POLL: Collect DeviceEvents from all active InputAdapters
         Collect IPC InteractionEvents from subscribers

2. TRANSLATE: Each DeviceEvent -> InputAdapter.translate() -> InteractionIntent
              Each IPC event -> InteractionIntent

3. RESOLVE: Each InteractionIntent -> InversePipeline.resolve_at() -> DataTarget
            Deduplicate (same target from multiple modalities = one intent)

4. APPLY: Each resolved intent -> StateChange
          Selection updates, filter changes, data mutations
          Authorization check for mutations

5. RECOMPILE: If data or filters changed:
              Grammar.incremental_recompile(state_changes)
              Produce updated RenderPlan

6. RENDER: For each active modality:
           ModalityCompiler.compile(render_plan, viewport) -> output
           All modalities render simultaneously from the same RenderPlan

7. BROADCAST: Emit InteractionResult to:
              - All local modalities (for feedback: haptic click, audio confirmation)
              - All IPC subscribers
              - Proprioception system (SAME DAVE confirmation)

8. CONFIRM: Proprioception verifies that output reached the user:
            - Display visible? (sensory afferent)
            - Audio audible? (if applicable)
            - Selection state matches intent?
            Loop to step 1.
```

This loop runs at the display refresh rate (typically 60Hz). Steps 2-4 are
the input half. Steps 5-6 are the output half. Steps 7-8 close the
bidirectional loop.

The same loop structure works for:
- A static health dashboard (mostly step 6, occasional steps 2-4)
- An interactive data explorer (continuous steps 2-6)
- A molecular dynamics simulation (continuous steps 4-6 with external data)
- A walkable soundscape (continuous steps 2-3 with navigation intents)
- A multiplayer data annotation session (continuous steps 2-7 with IPC)

### Walkable Soundscape Example

A developer exploring a dataset as a spatial audio environment:

```
Input: WASD keys or gamepad stick
    -> KeyboardAdapter or GamepadAdapter
    -> Navigate { direction: Forward, magnitude: 0.5 }
    -> Resolve: move listener position in data space
    -> Recompile: update sonification for new position
        closer data points are louder
        data density maps to reverb
        anomalies are dissonant tones
    -> Render: audio output with updated spatial positioning
    -> Feedback: footstep sound (proprioceptive confirmation of movement)

Input: 'Enter' key at a position
    -> Select { target: nearest_data_object }
    -> Audio: selected tone plays a distinctive pattern
    -> IPC: SelectionChanged broadcast

Simultaneously, a sighted colleague viewing the same data in egui:
    sees a cursor icon moving through the scatter plot
    sees the selected point highlighted
    can click their own points, which the audio user hears
```

---

## 8. SAME DAVE Integration

The Interaction Engine is the concrete implementation of the SAME DAVE model
defined in `BIDIRECTIONAL_UUI_ARCHITECTURE.md`.

```
SAME DAVE                    Interaction Engine
---------                    ------------------
Sensory Afferent     <->     InputAdapter.translate()
Motor Efferent       <->     ModalityCompiler.compile()
Central Awareness    <->     InteractionContext (state knowledge)
Feedback Loop        <->     Proprioception confirmation (step 8)
```

### Proprioception

The existing `Proprioception` system in `petal-tongue-ui` confirms that motor
output reached the sensory pathway. The Interaction Engine extends this:

- Motor command: "highlight songbird-alpha in green"
- Sensory confirmation: egui reports the frame was drawn; screen is visible
- Cross-modal confirmation: audio modality played the selection tone
- IPC confirmation: remote subscribers acknowledged the event

If any confirmation fails, the system can:
- Retry via a different modality
- Escalate (e.g., switch from visual highlight to audio alert if display lost)
- Report degraded state to the user via an available channel

### SensorEvent Mapping

The existing `SensorEvent` enum in `petal-tongue-core/src/sensor.rs` becomes
the `DeviceEvent` input to `InputAdapter`. The mapping:

| SensorEvent | InputAdapter | InteractionIntent |
|-------------|-------------|-------------------|
| Click | PointerAdapter | Select |
| Position (hover) | PointerAdapter | Focus |
| Scroll | PointerAdapter | Navigate (zoom) |
| KeyPress(Enter) | KeyboardAdapter | Select (focused) |
| KeyPress(Tab) | KeyboardAdapter | Focus (next) |
| KeyPress(Arrow) | KeyboardAdapter | Navigate |
| Heartbeat | (system) | (no intent) |
| FrameAcknowledged | (proprioception) | (confirmation) |

---

## 9. Rust Type Sketches

### Core Types (petal-tongue-grammar crate)

```rust
// --- Interaction types ---

pub type PerspectiveId = u64;
pub type DataSourceId = String;
pub type PrimitiveId = u64;
pub type GrammarId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataObjectId {
    pub source: DataSourceId,
    pub row_key: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionTarget {
    DataRow { data_id: DataObjectId },
    DataRange { variable: String, range: (Value, Value) },
    DataSet { predicate: FilterExpr },
    Region { bounds: BoundingBox },
    Primitive { primitive_id: PrimitiveId },
    Nothing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionEvent {
    pub intent: InteractionIntent,
    pub resolved_targets: Vec<DataObjectId>,
    pub perspective_id: PerspectiveId,
    pub grammar_id: GrammarId,
    pub timestamp_ms: u64,
}
```

### Traits (petal-tongue-grammar crate)

```rust
pub trait InputAdapter: Send + Sync {
    fn name(&self) -> &str;
    fn modality(&self) -> InputModality;
    fn capabilities(&self) -> Vec<InteractionCapability>;
    fn translate(
        &self,
        event: DeviceEvent,
        context: &InteractionContext,
    ) -> Option<InteractionIntent>;
    fn active_target(&self, context: &InteractionContext) -> Option<InteractionTarget>;
    fn feedback(&self, result: &InteractionResult);
}

pub trait InversePipeline: Send + Sync {
    fn modality(&self) -> OutputModality;
    fn resolve_at(
        &self,
        event: &DeviceEvent,
        plan: &RenderPlan,
        context: &InteractionContext,
    ) -> Option<InteractionTarget>;
    fn nearest_primitive(
        &self,
        target: &InteractionTarget,
        plan: &RenderPlan,
    ) -> Option<PrimitiveId>;
    fn data_values_at(
        &self,
        target: &InteractionTarget,
        plan: &RenderPlan,
    ) -> Option<DataRow>;
}
```

### Perspective (petal-tongue-grammar crate)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Perspective {
    pub id: PerspectiveId,
    pub active_modalities: Vec<ModalityConfig>,
    pub viewport: Viewport,
    pub filters: Vec<FilterExpr>,
    pub orientation: Orientation,
    pub selection: Vec<DataObjectId>,
    pub focus: Option<DataObjectId>,
    pub sync_mode: PerspectiveSync,
    pub user: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerspectiveSync {
    Independent,
    SharedSelection,
    SharedViewport,
    FullSync,
}
```

### Interaction Loop (petal-tongue-core crate)

```rust
pub struct InteractionEngine {
    adapters: Vec<Box<dyn InputAdapter>>,
    inverse_pipelines: HashMap<OutputModality, Box<dyn InversePipeline>>,
    perspectives: HashMap<PerspectiveId, Perspective>,
    event_bus: EventBus<InteractionEvent>,
    ipc_subscribers: Vec<IpcSubscription>,
}

impl InteractionEngine {
    pub fn tick(
        &mut self,
        device_events: &[DeviceEvent],
        ipc_events: &[InteractionEvent],
        render_plan: &RenderPlan,
        context: &mut InteractionContext,
    ) -> Vec<InteractionResult> {
        // Steps 2-4 of the interaction loop
        todo!()
    }

    pub fn register_adapter(&mut self, adapter: Box<dyn InputAdapter>);
    pub fn register_inverse(&mut self, pipeline: Box<dyn InversePipeline>);
    pub fn add_perspective(&mut self, perspective: Perspective) -> PerspectiveId;
    pub fn subscribe_ipc(&mut self, subscription: IpcSubscription);
}
```

All types are `#![forbid(unsafe_code)]`, `Serialize + Deserialize`,
`Send + Sync`. No rendering backend dependencies. Pure data and traits.

---

## 10. Evolution Path

### Phase 1: Semantic Intents (with Grammar Phase 3)

- Define `InteractionIntent`, `InteractionTarget`, `DataObjectId` types
- Implement `PointerAdapter` and `KeyboardAdapter`
- Implement visual `InversePipeline` for `EguiCompiler`
- Wire into existing `graph_canvas` and `visual_2d` interaction handlers
- Replace direct `screen_to_world` calls with `InversePipeline` trait

### Phase 2: Multi-Modal Inverse

- Implement `InversePipeline` for `RatatuiCompiler` (TUI cursor -> data)
- Implement `InversePipeline` for `AudioCompiler` (time/pitch -> data)
- Implement `VoiceCommandAdapter` (parsed text -> intent)
- First multi-modal test: select a node via keyboard in TUI, see it
  highlighted in egui

### Phase 3: Perspective System

- Implement `Perspective` struct and `PerspectiveSync`
- Multi-perspective rendering from shared `RenderPlan`
- Local multi-modality (one user, GUI + audio)
- Selection synchronization across perspectives

### Phase 4: IPC Interaction Protocol

- Implement `visualization.interact` outbound events
- Implement `visualization.interact.subscribe` and `.apply` inbound
- Remote collaboration: two petalTongue instances sharing selection
- Cross-primal interaction: Squirrel highlights anomalies in petalTongue

### Phase 5: Spatial and Immersive

- `GamepadAdapter` for spatial navigation
- Walkable soundscape (data as spatial audio environment)
- `MotionCaptureAdapter` for AR/VR input
- 3D perspective with orbit/fly navigation via barraCuda

### Phase 6: Assistive and Adaptive

- `BrailleAdapter` for Braille display routing keys
- `SwitchAccessAdapter` for single-switch scanning input
- `EyeGazeAdapter` for eye tracking
- Automatic modality fallback when primary modality is unavailable

---

## 11. Accessibility by Construction

Traditional accessibility is an afterthought: build the visual UI, then add
ARIA labels, then add keyboard navigation, then add screen reader support.
Each layer is a retrofit that fights the original architecture.

The Interaction Engine inverts this. Accessibility is not a feature. It is a
consequence of the architecture:

**Why it works:**

1. **Semantic intents are modality-agnostic.** There is no "click handler."
   There is a "select handler" that any input modality can trigger.

2. **DataObjectId is perspective-invariant.** The same data row has the same
   identity whether rendered as a pixel, a tone, a Braille cell, or a JSON
   field. Selection and focus operate on data, not on visual primitives.

3. **The inverse pipeline is per-modality.** Each modality knows how to map
   its own space back to data space. Adding a new modality means implementing
   one `InversePipeline` and one `InputAdapter`. The rest of the system works
   unchanged.

4. **Perspectives are first-class.** The system does not assume one "correct"
   view. Multiple simultaneous perspectives are the normal case, not an edge
   case. This means multi-user and multi-modality support is structural, not
   bolted on.

5. **The interaction loop is closed.** Proprioception confirms that output
   reached the user. If the visual channel fails (screen not visible), the
   system detects this and can escalate to another modality.

**The test:** If three people -- one deaf, one blind, one sighted -- can
collaboratively explore a dataset, annotate findings, and reach shared
conclusions without any of them being disadvantaged, the architecture is
correct.

---

## References

- Norman, D.A. (2013). *The Design of Everyday Things*. Basic Books.
  (Affordances and feedback loops)
- Wilkinson, L. (2005). *The Grammar of Graphics*. Springer.
  (Interaction as inverse of the grammar pipeline)
- Tufte, E.R. (2006). *Beautiful Evidence*. Graphics Press.
  (Sparklines as minimal-latency data representation)
- Hermann, T. et al. (2011). *The Sonification Handbook*. Logos Verlag.
  (Auditory data exploration and interaction)
- Shinohara, K. & Wobbrock, J.O. (2011). In the Shadow of Misperception.
  *CHI 2011*. (Ability-based design vs. disability-focused design)

---

**Status**: Ready for implementation (Phase 1 can begin with Grammar Phase 3)
**Blocking**: `GRAMMAR_OF_GRAPHICS_ARCHITECTURE.md` Phase 1 (RenderPlan must
exist before InversePipeline can invert it)
**First Milestone**: PointerAdapter + KeyboardAdapter + Visual InversePipeline
replacing direct `screen_to_world` in `graph_canvas`
