# Real-Time Collaborative Pipeline

**Version**: 1.0.0
**Date**: March 11, 2026
**Status**: Implementation Phase
**Priority**: Critical (enables ludoSpring, Squirrel AI, and multi-primal collaboration)
**Depends On**: `INTERACTION_ENGINE_ARCHITECTURE.md`, `UNIVERSAL_VISUALIZATION_PIPELINE.md`,
`BIDIRECTIONAL_UUI_ARCHITECTURE.md`
**Informed By**: ludoSpring `GAME_ENGINE_NICHE_SPECIFICATION.md`, ludoSpring `LUDOLOGY_DOMAIN_SPECIFICATION.md`

---

## Purpose

Close the three gaps that prevent petalTongue from serving as the real-time,
multi-modal, collaborative representation engine the ecoPrimals ecosystem
requires. Today petalTongue can compile data to any modality and return exports
via IPC. What it cannot do is:

1. Run a continuous 60 Hz loop for game-style and animation-driven visualization
2. Surface IPC visualization sessions as live panels in the running UI
3. Stream sensor events to external primals for engagement and AI analysis

These three capabilities transform petalTongue from a request/response visualizer
into a real-time collaborative partner where AI primals, science primals, and
humans interact through a shared data space.

---

## Motivating Scenario

```
Atomic (node primal) emits telemetry
    → Squirrel (AI) analyzes, detects anomaly
    → Squirrel calls visualization.render to petalTongue with anomaly data
    → petalTongue renders the data in the user's active modality (egui/audio/braille)
    → User clicks the anomalous data point
    → petalTongue broadcasts visualization.interact to Squirrel
    → ludoSpring receives sensor events, evaluates engagement/flow
    → Squirrel adjusts visualization based on user's flow state
    → petalTongue updates live panels with new perspective
```

This is a continuous loop. No primal knows any other by name. Discovery is
capability-based. The data is the shared truth; perspectives are how each
participant (human or AI) perceives it.

---

## Gap 1: Game Loop Integration

### Problem

petalTongue has a fully implemented game loop (`petal-tongue-scene/src/game_loop.rs`)
with fixed timestep, accumulator, physics, and animation stepping. It is tested
but not wired into the main application. The UI runs event-driven (repaint on
demand), not continuously.

### Architecture

The existing `TickConfig` provides:
- Fixed dt: 16.67ms (60 Hz)
- Max accumulator: 250ms (prevents spiral of death)
- Physics and animation stepping via `tick_frame()`
- Interpolation factor via `TickClock::alpha()`

### Integration Design

```
┌─────────────────────────────────────────────────────────────┐
│                    PetalTongueApp                           │
│                                                             │
│  update_headless(ctx):                                      │
│    dt = ctx.input(|i| i.stable_dt)    ← unified delta time  │
│    tick_clock.begin_frame(dt)                                │
│                                                             │
│    while tick_clock.should_tick():                           │
│      tick_frame(config, scene, physics, animation)          │
│      tick_clock.consume_tick()                              │
│                                                             │
│    if tick_result.scene_dirty:                               │
│      recompile modalities                                   │
│      ctx.request_repaint()            ← continuous repaint   │
│                                                             │
│    render_panels(ctx, alpha)          ← interpolated render  │
└─────────────────────────────────────────────────────────────┘
```

### Delta Time Unification

All animation systems use one source: `ctx.input(|i| i.stable_dt)`.

| System | Current | After |
|--------|---------|-------|
| Awakening overlay | `ctx.input(stable_dt)` | Same (no change) |
| `AnimationEngine` (flow) | Internal `Instant` | Receives `dt` from app |
| Scene `AnimationPlayer` | Expects `dt` from caller | Receives `dt` via `tick_frame()` |
| `PhysicsWorld` | Expects `dt` from caller | Receives `dt` via `tick_frame()` |

### Continuous Repaint

`ctx.request_repaint()` is called when:
- `TickResult::scene_dirty` is true (physics or animation changed the scene)
- An active IPC visualization session has streaming updates
- `AnimationEngine` has active flow particles or pulses

This means idle petalTongue uses zero CPU (event-driven), but transitions
seamlessly to 60 Hz when real-time content is active.

### Motor Commands

New motor commands for continuous mode:

- `SetContinuousMode { enabled: bool }` — Enable/disable 60 Hz tick loop
- `SetPhysics { enabled: bool }` — Enable/disable physics simulation
- `SetAnimation { enabled: bool }` — Enable/disable scene animation

---

## Gap 2: IPC-to-UI Bridge

### Problem

When an external primal calls `visualization.render`, the data is stored in
`VisualizationState` on the IPC server side. The live UI does not see it.
Sessions can only be consumed via `visualization.export` (SVG, audio, etc.).

### Architecture

```
┌─────────────────────────┐     Arc<RwLock<>>     ┌──────────────────┐
│    IPC Server           │ ──────────────────── │   PetalTongueApp  │
│                         │                       │                   │
│  visualization.render   │                       │  poll_sessions()  │
│    → VisualizationState │                       │    → active list  │
│  visualization.stream   │                       │  render_session() │
│    → update bindings    │                       │    → DataBinding  │
│  visualization.dismiss  │                       │    → Compiler     │
│    → remove session     │                       │    → SceneGraph   │
└─────────────────────────┘                       │    → Panel        │
                                                  └──────────────────┘
```

### Shared State

`VisualizationState` is wrapped in `Arc<RwLock<VisualizationState>>` and shared
between the IPC server and the UI app. The IPC server writes (session CRUD),
the app reads (poll for active sessions each frame).

### Live Session Panel

A new `LiveSessionsPanel` in the UI:
- Polls `VisualizationState` for sessions with `updated_since(last_check)`
- For each session, extracts `DataBinding`s and compiles them using the existing
  `DataBindingCompiler` → `GrammarCompiler` → `SceneGraph` pipeline
- Renders the resulting scene graph using `EguiCompiler` (or any active modality)
- Domain theming applied from session metadata (e.g., `domain: "game"` for
  ludoSpring, `domain: "health"` for healthSpring)

### ludoSpring GameDataChannel Mapping

| ludoSpring Channel | DataBinding Variant | Notes |
|--------------------|---------------------|-------|
| EngagementCurve | TimeSeries | x=time, y=engagement |
| DifficultyProfile | TimeSeries | x=progress, y=difficulty |
| FlowTimeline | Bar | categories=flow states, values=durations |
| InteractionCostMap | Heatmap | x=screen region, y=action, z=Fitts cost |
| GenerationPreview | Scatter | Procedural content preview |
| AccessibilityReport | FieldMap | WCAG metrics per component |
| UiAnalysis | FieldMap | Tufte metrics per panel |

### Streaming Updates

When a primal calls `visualization.render.stream` with `StreamOperation::Append`,
the binding is updated in-place in `VisualizationState`. The UI detects the
change via `updated_since()` and re-renders the affected panel. This enables
real-time data feeds (e.g., healthSpring vital signs, Squirrel AI reasoning
trace, ludoSpring engagement curve).

---

## Gap 3: Sensor Event Streaming

### Problem

ludoSpring needs `SensorEvent[]` to evaluate Fitts's law, Hick's law, engagement,
and flow state. Squirrel needs interaction events to adapt its AI-driven
visualization. Currently petalTongue captures sensor events internally but does
not stream them over IPC.

### Architecture

```
┌──────────────┐    broadcast    ┌──────────────────┐    JSON-RPC
│ Sensor Layer │ ──────────────▶ │ SensorBroadcaster │ ──────────▶ subscribers
│ (keyboard,   │                 │ (mpsc channel)    │
│  mouse,      │                 │                   │
│  screen)     │                 │                   │
└──────────────┘                 └──────────────────┘
```

### IPC Methods

| Method | Purpose |
|--------|---------|
| `interaction.sensor_stream.subscribe` | Register for sensor events. Returns `subscription_id`. |
| `interaction.sensor_stream.unsubscribe` | Remove subscription by ID. |

Sensor events are delivered as JSON-RPC notifications to the subscriber's socket:

```json
{
  "jsonrpc": "2.0",
  "method": "interaction.sensor_events",
  "params": {
    "subscription_id": "sub-001",
    "batch": [
      {"type": "pointer_move", "x": 423.0, "y": 187.0, "timestamp_ms": 1710000000000},
      {"type": "click", "x": 423.0, "y": 187.0, "button": "left", "timestamp_ms": 1710000000050},
      {"type": "key_press", "key": "A", "modifiers": {"ctrl": true}, "timestamp_ms": 1710000000100}
    ]
  }
}
```

### SensorEventBatch

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorEventBatch {
    pub subscription_id: String,
    pub events: Vec<SensorEventIpc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SensorEventIpc {
    #[serde(rename = "pointer_move")]
    PointerMove { x: f32, y: f32, timestamp_ms: u64 },
    #[serde(rename = "click")]
    Click { x: f32, y: f32, button: String, timestamp_ms: u64 },
    #[serde(rename = "key_press")]
    KeyPress { key: String, modifiers: KeyModifiersIpc, timestamp_ms: u64 },
    #[serde(rename = "scroll")]
    Scroll { delta_x: f32, delta_y: f32, timestamp_ms: u64 },
    #[serde(rename = "key_release")]
    KeyRelease { key: String, timestamp_ms: u64 },
}
```

### Batching

Sensor events are batched per tick (16.67ms at 60 Hz) to avoid flooding the
IPC channel. One notification per tick containing all events since the last tick.

### Privacy

Sensor events contain only interaction data (positions, keys, timing). No screen
content, no text input content, no accessibility information. Events are
local-only (Unix socket). No network transmission.

---

## Collaborative Loop (Complete)

When all three gaps are closed:

```
┌─────────────────────────────────────────────────────────────────────┐
│                    CONTINUOUS COLLABORATIVE LOOP                     │
│                                                                     │
│  1. POLL    — Sensor events captured, batched for IPC subscribers   │
│  2. TRANSLATE — Raw events → InteractionIntent                      │
│  3. RESOLVE — Inverse pipeline → DataObjectId                       │
│  4. APPLY   — Update selection, focus, navigation state             │
│  5. RECOMPILE — Re-evaluate grammar if data/view changed            │
│              — Poll VisualizationState for IPC session updates       │
│  6. RENDER  — Compile scene to all active modalities                │
│  7. BROADCAST — visualization.interact to IPC subscribers           │
│              — interaction.sensor_events to sensor subscribers       │
│  8. CONFIRM — SAME DAVE loop closure (proprioception)               │
│                                                                     │
│  External primals:                                                  │
│  • Squirrel: subscribes to interact + sensors, drives visualization │
│  • ludoSpring: subscribes to sensors, evaluates engagement/flow     │
│  • healthSpring: pushes live data via visualization.render.stream   │
│  • barraCuda: receives compute offload, returns results             │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Implementation Phases

### Phase 1: Game Loop Wiring
- Wire `TickClock` into `PetalTongueApp`
- Unify delta time source
- Enable continuous repaint when scene is dirty
- Motor commands for continuous mode
- Tests: headless harness verifies tick progression

### Phase 2: IPC-to-UI Bridge
- Share `VisualizationState` via `Arc<RwLock<>>`
- Poll sessions in app update loop
- Render sessions as live panels
- Streaming update detection
- Tests: IPC server creates session, headless verifies panel appears

### Phase 3: Sensor Streaming
- `SensorBroadcaster` channel in sensor layer
- IPC subscribe/unsubscribe handlers
- Batched notification delivery
- Tests: subscribe, simulate input, verify batch received

---

## References

- ludoSpring `GAME_ENGINE_NICHE_SPECIFICATION.md` — 60 Hz niche topology
- ludoSpring `LUDOLOGY_DOMAIN_SPECIFICATION.md` — Fitts/Hick/Flow models
- `INTERACTION_ENGINE_ARCHITECTURE.md` — 8-step cycle, inverse pipeline
- `UNIVERSAL_VISUALIZATION_PIPELINE.md` — Data → grammar → modality pipeline
- `BIDIRECTIONAL_UUI_ARCHITECTURE.md` — SAME DAVE loop, proprioception

---

**Status**: Implementation Phase
**Blocking**: None (all prerequisites exist in codebase)
**First Milestone**: Game loop wired, headless test verifies 60 Hz tick
