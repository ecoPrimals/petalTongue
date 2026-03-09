# Universal Visualization Pipeline

**Version**: 1.0.0  
**Date**: March 8, 2026  
**Status**: Design Phase  
**Priority**: High  
**Depends On**: `GRAMMAR_OF_GRAPHICS_ARCHITECTURE.md`  
**Related**: `TUFTE_CONSTRAINT_SYSTEM.md`, wateringHole `UNIVERSAL_IPC_STANDARD_V3.md`

---

## Purpose

Define the end-to-end pipeline from raw data arriving at petalTongue through grammar
compilation, optional GPU compute offload via barraCuda, modality-specific rendering,
and interaction feedback. This spec governs how petalTongue becomes a universal
visualization engine -- not by adding more widgets, but by providing one composable
pipeline that any data domain flows through.

---

## Pipeline Overview

```
                         ┌──────────────┐
                         │  Data Source  │
                         │  (any primal │
                         │   or file)   │
                         └──────┬───────┘
                                │ JSON-RPC / tarpc / file
                                ▼
┌───────────────────────────────────────────────────────────────┐
│                    petalTongue Pipeline                        │
│                                                               │
│  ┌─────────────┐    ┌──────────────┐    ┌──────────────────┐ │
│  │ Data Ingest  │───▶│   Grammar    │───▶│ Grammar Compiler │ │
│  │ + Schema     │    │  Expression  │    │ (validate, scale │ │
│  │ Detection    │    │  (explicit   │    │  train, defaults │ │
│  └─────────────┘    │   or auto)   │    │  Tufte checks)   │ │
│                      └──────────────┘    └────────┬─────────┘ │
│                                                   │           │
│                           ┌───────────────────────┤           │
│                           │ Heavy computation?    │           │
│                           ▼ Yes                   ▼ No        │
│                  ┌─────────────────┐    ┌──────────────────┐  │
│                  │ barraCuda IPC   │    │ Local Compute    │  │
│                  │ (GPU stats,     │    │ (in-process      │  │
│                  │  tessellation,  │    │  statistics,      │  │
│                  │  3D projection) │    │  scale mapping)   │  │
│                  └────────┬────────┘    └────────┬─────────┘  │
│                           │                      │            │
│                           └──────────┬───────────┘            │
│                                      ▼                        │
│                           ┌──────────────────┐                │
│                           │   Render Plan    │                │
│                           │ (abstract prims  │                │
│                           │  + scale meta)   │                │
│                           └────────┬─────────┘                │
│                                    │                          │
│             ┌──────────┬───────────┼───────────┬─────────┐   │
│             ▼          ▼           ▼           ▼         ▼   │
│         ┌──────┐  ┌────────┐  ┌───────┐  ┌───────┐ ┌─────┐ │
│         │ egui │  │ratatui │  │ Audio │  │SVG/PNG│ │JSON │ │
│         │      │  │        │  │       │  │       │ │     │ │
│         └──┬───┘  └───┬────┘  └──┬────┘  └──┬────┘ └──┬──┘ │
│            │          │          │           │         │    │
│            └──────────┴──────────┴───────────┴─────────┘    │
│                                  │                          │
│                                  ▼                          │
│                        ┌──────────────────┐                 │
│                        │  Human / Client  │                 │
│                        └────────┬─────────┘                 │
│                                 │ Interaction event          │
│                                 ▼                           │
│                        ┌──────────────────┐                 │
│                        │ Inverse Scale    │                 │
│                        │ Pipeline         │                 │
│                        │ (screen → data)  │                 │
│                        └────────┬─────────┘                 │
│                                 │                           │
│                    ┌────────────┴────────────┐              │
│                    ▼                         ▼              │
│           ┌────────────────┐    ┌────────────────────┐      │
│           │ Local state    │    │ IPC event to other │      │
│           │ update (zoom,  │    │ primals (selection, │      │
│           │ selection)     │    │ filter, command)    │      │
│           └────────────────┘    └────────────────────┘      │
└───────────────────────────────────────────────────────────────┘
```

---

## Stage 1: Data Ingest

### Data Sources

petalTongue accepts data from any source that can produce rows and columns.
The ingest layer normalizes all sources to a common `DataFrame` abstraction.

| Source | Transport | Notes |
|--------|-----------|-------|
| Primal metrics | JSON-RPC subscription | Streaming, live updates |
| Primal topology | JSON-RPC / tarpc | Graph structure (nodes + edges) |
| File (CSV, JSON, Parquet) | Local filesystem | Static datasets |
| NestGate blobs | JSON-RPC `storage.get` | Content-addressed data |
| healthSpring diagnostics | JSON-RPC subscription | Clinical time series |
| Simulation state | tarpc stream | Game / physics / molecular dynamics |
| User-provided | Drag-and-drop or CLI | Ad hoc exploration |

### Schema Detection

When a grammar expression specifies `"data": {"source": "primal.health_metrics"}`,
petalTongue resolves the source via capability-based discovery (never hardcoded
primal names) and infers the schema from the first response:

```rust
pub struct Schema {
    pub columns: Vec<ColumnDef>,
}

pub struct ColumnDef {
    pub name: String,
    pub dtype: DataType,
    pub nullable: bool,
}

pub enum DataType {
    Float64,
    Int64,
    String,
    Boolean,
    DateTime,
    Duration,
    Binary,
    Array(Box<DataType>),
    Struct(Vec<ColumnDef>),
}
```

### Zero-Copy Data Path

For high-throughput data (simulation frames, large datasets), the pipeline
uses `bytes::Bytes` end-to-end:

```
tarpc stream → Bytes → columnar decode (zero-copy slice) → DataFrame view
```

The `DataFrame` holds `Bytes` buffers and provides typed column accessors
that reference into those buffers without copying. This is critical for
game-loop and simulation visualization where data arrives every frame.

---

## Stage 2: Grammar Expression

Grammar expressions can arrive from three sources:

1. **Explicit**: Another primal sends a `visualization.render` JSON-RPC call
   with a complete grammar expression.

2. **Auto-generated**: petalTongue detects the data schema and generates a
   default grammar expression using heuristics:
   - 1 numeric column → histogram (GeomBar + StatBin)
   - 2 numeric columns → scatterplot (GeomPoint)
   - 1 temporal + 1 numeric → time series (GeomLine)
   - 1 categorical + 1 numeric → bar chart (GeomBar)
   - Node + edge columns → graph layout (GeomPoint + GeomLine, force-directed)
   - 3D coordinates → 3D scatter/mesh (GeomPoint/GeomMesh3D, Perspective3DCoord)

3. **User-composed**: The graph builder UI (already partially implemented)
   becomes a grammar expression editor. Users drag aesthetics onto variables,
   select geometries, configure scales.

### Grammar Builder UI

The existing graph builder (`graph_canvas.rs`, `graph_editor/`) evolves into a
grammar editor:

```
┌──────────────────────────────────────────────────────────┐
│  Grammar Builder                                          │
│                                                           │
│  Data: [primal.health_metrics ▾]     Schema: 5 columns   │
│                                                           │
│  ┌──────────────────────┐  ┌──────────────────────────┐  │
│  │  Variables           │  │  Preview                 │  │
│  │                      │  │                          │  │
│  │  timestamp  → X      │  │  ┌─────────────────┐    │  │
│  │  cpu_pct    → Y      │  │  │  (live render)  │    │  │
│  │  primal_id  → Color  │  │  │                 │    │  │
│  │  memory_mb  → Size   │  │  └─────────────────┘    │  │
│  │                      │  │                          │  │
│  │  Geometry: [Line ▾]  │  │  Tufte Score: 0.87      │  │
│  │  Coord: [Cartesian]  │  │  Warnings: 1            │  │
│  │  Facet: [primal_id]  │  │                          │  │
│  └──────────────────────┘  └──────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

---

## Stage 3: Grammar Compilation

See `GRAMMAR_OF_GRAPHICS_ARCHITECTURE.md` for the compiler internals.

The compiler's output is a `RenderPlan`:

```rust
pub struct RenderPlan {
    pub panels: Vec<Panel>,
    pub legends: Vec<Legend>,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub constraints_report: ConstraintsReport,
    pub interaction_map: InteractionMap,
}

pub struct Panel {
    pub primitives: Vec<Primitive>,
    pub x_scale: Box<dyn Scale<Domain = f64, Range = f64>>,
    pub y_scale: Box<dyn Scale<Domain = f64, Range = f64>>,
    pub x_breaks: Vec<Break>,
    pub y_breaks: Vec<Break>,
    pub bounds: Rect,
    pub facet_label: Option<String>,
}
```

---

## Stage 4: Compute Offload (barraCuda Integration)

### When to Offload

The grammar compiler decides whether to compute locally or offload to barraCuda
based on:

| Criterion | Local | barraCuda |
|-----------|-------|-----------|
| Data size | < 10,000 rows | >= 10,000 rows |
| Statistic | Identity, Count, simple agg | KDE, smoothing, binning on large data |
| Geometry | 2D primitives | 3D mesh tessellation, isosurface extraction |
| Coordinate | Cartesian, Polar | 3D projection with lighting |
| Physics | N/A | N-body, molecular dynamics, collision |

### IPC Contract

petalTongue discovers barraCuda via capability, not name. The capability
queries:

```json
{"method": "discovery.query", "params": {"capability": "math.stat"}}
{"method": "discovery.query", "params": {"capability": "math.tessellate"}}
{"method": "discovery.query", "params": {"capability": "math.physics"}}
```

Once discovered, the grammar compiler invokes barraCuda methods using
semantic naming per `SEMANTIC_METHOD_NAMING_STANDARD.md`:

| Method | Purpose | Input | Output |
|--------|---------|-------|--------|
| `math.stat.kde` | Kernel density estimate | Numeric column as `Bytes` | Density curve as `Bytes` |
| `math.stat.smooth` | LOESS / moving average | x,y columns as `Bytes` | Smoothed y + bands as `Bytes` |
| `math.stat.bin` | Histogram binning | Numeric column as `Bytes` | Bin edges + counts as `Bytes` |
| `math.stat.summary` | Grouped aggregation | Data frame as `Bytes` | Summary frame as `Bytes` |
| `math.tessellate.sphere` | Icosphere generation | Center, radius, detail level | Vertex + index buffer as `Bytes` |
| `math.tessellate.cylinder` | Bond geometry | Endpoints, radius, segments | Vertex + index buffer as `Bytes` |
| `math.tessellate.isosurface` | Marching cubes | 3D scalar field as `Bytes` | Mesh as `Bytes` |
| `math.project.perspective` | 3D → 2D projection | Vertices + camera as `Bytes` | Screen coords as `Bytes` |
| `math.project.lighting` | Phong/PBR shading | Vertices + normals + lights | Colors as `Bytes` |
| `math.physics.nbody` | Gravitational N-body step | Positions + masses as `Bytes` | New positions as `Bytes` |
| `math.physics.md_forces` | Molecular dynamics | Atom positions + params | Forces as `Bytes` |
| `math.physics.collision` | Collision detection | AABB/sphere set as `Bytes` | Collision pairs as `Bytes` |

All payloads use `bytes::Bytes` for zero-copy transfer over tarpc.
JSON-RPC fallback serializes as base64-encoded binary in the `data` field.

### Streaming Pipeline

For simulation and game visualization, the pipeline runs continuously:

```
barraCuda stream → tarpc Bytes channel → DataFrame update → Grammar recompile (incremental) → RenderPlan diff → Modality update
```

The grammar compiler supports incremental recompilation: when only the data
changes (not the grammar expression), it reuses trained scales and regenerates
only the geometry. This keeps frame times under 16ms for 60fps visualization.

---

## Stage 5: Modality Compilation

### Modality Selection

petalTongue detects available modalities at startup and selects the best match:

```rust
pub enum Modality {
    DesktopGui,       // DISPLAY or WAYLAND_DISPLAY set, GPU available
    TerminalUi,       // TERM set, no display server
    HeadlessRender,   // No display, no terminal (server/CI)
    AudioOnly,        // No display, audio device available
    WebServer,        // Serving to remote browser
    ApiOnly,          // Machine-to-machine (JSON output)
}
```

Multiple modalities can be active simultaneously. A sighted user with speakers
gets DesktopGui + Audio. A blind user gets Audio + ApiOnly (for screen reader).

### Compiler Contracts

Each modality compiler translates the abstract `RenderPlan` into native output.
The compiler must declare what it supports so the grammar compiler can adapt:

```rust
pub trait ModalityCompiler: Send + Sync {
    fn compile(&self, plan: &RenderPlan, viewport: &Viewport) -> ModalityOutput;
    fn supported_primitives(&self) -> &[PrimitiveKind];
    fn supported_aesthetics(&self) -> &[AestheticRole];
    fn supported_interactions(&self) -> &[InteractionKind];
    fn max_primitives(&self) -> usize;
    fn supports_streaming(&self) -> bool;
}
```

If a modality doesn't support a primitive (e.g., TUI can't render Mesh), the
grammar compiler falls back:
- `Mesh` → 2D projection (wireframe lines) for TUI
- `Polygon` → ASCII area fill for TUI
- `Arc` → text percentage for TUI
- `Point` → braille dot for TUI

### Modality-Specific Compilation Details

#### egui (Desktop GUI)

The egui compiler maps primitives to egui paint commands:

| Primitive | egui Output |
|-----------|-------------|
| `Point` | `painter.circle_filled()` |
| `Line` | `painter.line()` with `PathStroke` |
| `Rect` | `painter.rect_filled()` |
| `Text` | `painter.text()` with `FontId` |
| `Polygon` | `painter.add(Shape::convex_polygon())` |
| `Arc` | `painter.add(Shape::Path)` with arc segments |
| `Mesh` | Forwarded to eframe's glow backend or barraCuda |

Interaction: egui `Response` objects → inverse scale pipeline → data events.

#### ratatui (Terminal UI)

| Primitive | ratatui Output |
|-----------|----------------|
| `Point` | Braille character in `canvas::Canvas` |
| `Line` | Braille line drawing or `Sparkline` widget |
| `Rect` | `Block` with filled background |
| `Text` | `Paragraph` or `Span` |
| `Polygon` | Filled braille region |
| `Arc` | Text-based gauge: `[████░░░░] 62%` |
| `Mesh` | Wireframe edges projected to 2D braille |

The TUI compiler automatically switches between block characters (low-res
terminals) and braille patterns (Unicode-capable terminals).

#### Audio (Sonification)

| Aesthetic | Audio Parameter | Mapping |
|-----------|----------------|---------|
| Y (pitch) | Frequency (Hz) | Linear or log scale, 100Hz-4000Hz |
| X (time) | Playback position | Left-to-right temporal sweep |
| Color (timbre) | Waveform shape | Sine, square, saw, triangle |
| Size (volume) | Amplitude | 0.0-1.0 |
| X position (pan) | Stereo pan | -1.0 (left) to 1.0 (right) |
| Facet | Sequential playback | One sound per facet, with pause |

A scatterplot becomes a cloud of tones. A time series becomes a melody.
A heatmap becomes a texture sweep. The same grammar expression, different
sensory channel.

#### JSON (Machine-Readable)

The JSON compiler outputs the RenderPlan as structured JSON for consumption by
other primals, web clients, or AI systems:

```json
{
  "panels": [{
    "primitives": [
      {"type": "line", "points": [[0,0.2],[1,0.5],[2,0.8]], "color": "#2196F3"},
      {"type": "point", "x": 1.5, "y": 0.65, "size": 8, "color": "#FF5722"}
    ],
    "x_axis": {"label": "Time (s)", "breaks": [0, 1, 2, 3]},
    "y_axis": {"label": "CPU %", "breaks": [0, 25, 50, 75, 100]}
  }],
  "title": "Primal Health"
}
```

---

## Stage 6: Interaction and Inverse Mapping

> **Full specification**: See `INTERACTION_ENGINE_ARCHITECTURE.md`
>
> The interaction engine is a full subsystem with semantic intents, per-modality
> inverse pipelines, a perspective system for multi-user collaboration, and an
> IPC protocol for cross-primal interaction.

### The Generalized Inverse Pipeline

The forward pipeline maps data to modality output. The inverse pipeline maps
human interaction BACK to data. Each active modality has its own inverse path,
but all converge to the same `DataTarget` -- a reference to actual data
rows/objects that is perspective-invariant.

#### Visual Inverse (egui, SVG, web)

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

#### Audio Inverse (sonification)

```
Time offset in soundscape (3.2s into the render)
    -> Which sonic element is playing (tone_id=7)
    -> Inverse sonification mapping
        pitch -> data value (health=85)
        pan -> data position (x=0.3 -> primal "songbird")
        timbre -> data category (type="discovery")
    -> DataObjectId (source=topology, row="songbird-alpha")
```

#### TUI Inverse (ratatui, terminal)

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

#### Voice / Command Inverse

```
Parsed command: "select the unhealthy primal"
    -> Entity resolution against current DataSource
        "unhealthy" -> filter: status != "healthy"
        "primal" -> entity type constraint
    -> Matching DataObjectIds
```

All inverse paths produce `DataObjectId` values that are the same regardless
of which modality resolved them. This is how the "6 vs 9" problem is solved:
three users with different sensory systems all point at the same data row.

### Semantic Intents (Replacing Device Events)

Device events (mouse click, key press, voice command) are NOT interactions.
The Interaction Engine translates them to semantic `InteractionIntent` values:

```
Device event (mouse click at 423, 187)
    -> InputAdapter.translate()
    -> InteractionIntent::Select { target, mode: Replace }
    -> InversePipeline.resolve_at() -> DataObjectId
    -> StateChange::SelectionChanged
    -> Broadcast to all perspectives and IPC subscribers
```

A keyboard Enter on a focused node, a voice command "select that", and a
Braille display routing key all produce the same `InteractionIntent::Select`.
See `INTERACTION_ENGINE_ARCHITECTURE.md` §2 for the full intent taxonomy.

### Interaction Events (IPC)

Interaction events are sent over JSON-RPC to other primals. The event carries
`DataObjectId` (data-space, not screen-space) so any modality can highlight
the referenced objects:

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

Other primals can subscribe to interaction events:

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

And programmatically drive petalTongue's selection:

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

This enables patterns like: Squirrel detects an anomaly and tells petalTongue
to highlight it. The human sees the highlight in their own modality and
investigates.

### Linked Views

Multiple grammar expressions can share interaction state. A brush selection in
one view filters the data in another:

```rust
pub struct LinkedSelection {
    pub source_grammar: GrammarId,
    pub variable: String,
    pub range: (Value, Value),
}
```

Linked selections are local to petalTongue (no IPC needed). The grammar
compiler listens for selection changes and incrementally recompiles affected
views. With perspectives, linked views extend naturally to multi-modal:
a selection in the egui view highlights in the TUI and triggers a tone in
the audio sonification simultaneously.

### The Interaction Loop

petalTongue runs a game-engine-style tick loop integrating forward and
inverse pipelines:

```
1. POLL    - Collect device events from all InputAdapters + IPC events
2. TRANSLATE - DeviceEvent -> InputAdapter -> InteractionIntent
3. RESOLVE - InteractionIntent -> InversePipeline -> DataTarget
4. APPLY   - DataTarget -> StateChange (selection, filter, mutation)
5. RECOMPILE - Grammar.incremental_recompile(state_changes)
6. RENDER  - All active ModalityCompilers render simultaneously
7. BROADCAST - Emit InteractionResult to local modalities + IPC subscribers
8. CONFIRM - Proprioception verifies output reached the user
```

See `INTERACTION_ENGINE_ARCHITECTURE.md` §7 for the full loop specification.

---

## Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Grammar compile (2D, <1K rows) | < 1ms | In-process, no barraCuda |
| Grammar compile (2D, <100K rows) | < 10ms | In-process statistics |
| Grammar compile (3D, any size) | < 50ms | Includes barraCuda IPC |
| Incremental recompile (data only) | < 1ms | Reuse scales, regenerate geoms |
| Modality compile (egui, <10K prims) | < 2ms | Direct paint commands |
| Modality compile (TUI, <1K prims) | < 1ms | Braille character buffer |
| Inverse scale lookup | < 0.01ms | O(1) for linear/log scales |
| Full frame (data → pixels) at 60fps | < 16ms | Entire pipeline |
| barraCuda round-trip (tarpc) | < 5ms | Zero-copy Bytes, local socket |

---

## Data Flow Contracts

### petalTongue → barraCuda

```
GrammarExpr.statistics    → math.stat.{operation}
GrammarExpr.geometry(3D)  → math.tessellate.{shape}
GrammarExpr.coordinates(3D) → math.project.{type}
Simulation tick request    → math.physics.{solver}
```

### barraCuda → petalTongue

```
Computed statistics  → DataFrame (Bytes, columnar)
Tessellated meshes   → Vertex + Index buffers (Bytes)
Projected coordinates → Screen-space positions (Bytes)
Physics state        → Updated positions (Bytes, streaming)
```

### Other Primals → petalTongue

```
visualization.render              → Full grammar expression
visualization.render.stream       → Grammar + streaming data subscription
visualization.export              → Grammar → static SVG/PNG/JSON
visualization.capabilities        → What can petalTongue render?
visualization.interact.subscribe  → Subscribe to interaction events
visualization.interact.apply      → Programmatically trigger an interaction
visualization.interact.perspectives → List active perspectives
visualization.interact.sync       → Set perspective synchronization mode
```

### petalTongue → Other Primals

```
visualization.interact     → Interaction events (semantic intents with DataObjectId)
discovery.query            → Capability-based data source resolution
{source}.subscribe         → Streaming data subscription
{callback_method}          → Interaction event delivery to subscribers
```

---

## Sovereignty Considerations

1. **No hardcoded primal names**: Data sources resolved via `discovery.query`
   with capability strings, never primal names.

2. **No hardcoded ports**: All IPC via Unix socket discovery or env var override.

3. **No vendor lock-in**: Grammar expressions are backend-agnostic. Swap egui for
   any other renderer without changing the grammar.

4. **Data stays local**: Visualization happens on the user's machine. No telemetry,
   no cloud rendering, no data exfiltration.

5. **User controls modality**: The user (or their config) decides GUI vs TUI vs
   Audio vs API. petalTongue adapts, never dictates.

---

**Status**: Ready for implementation  
**Blocking**: `GRAMMAR_OF_GRAPHICS_ARCHITECTURE.md` Phase 1  
**First Milestone**: Data ingest + schema detection for biomeOS primal metrics
