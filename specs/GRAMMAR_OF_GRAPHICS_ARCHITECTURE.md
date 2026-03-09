# Grammar of Graphics Architecture

**Version**: 1.0.0  
**Date**: March 8, 2026  
**Status**: Design Phase  
**Priority**: High (Core Evolution Path)  
**Depends On**: `UNIVERSAL_VISUALIZATION_PIPELINE.md`, `TUFTE_CONSTRAINT_SYSTEM.md`

---

## Vision

Replace petalTongue's ad-hoc per-widget data-to-pixel mapping with a **compositional,
type-safe Grammar of Graphics** layer that sits between data and rendering.

Every visualization petalTongue produces -- graph topology, timeline, traffic heat map,
trust dashboard, health gauge, molecular structure, game world -- should be expressible
as a grammar statement that the rendering backends compile to their native output.

**Core principle**: Data defines structure. Grammar defines mapping. Modality defines
rendering. The human defines interaction. The application is secondary.

---

## Background: Wilkinson's Grammar

Leland Wilkinson's *Grammar of Graphics* (1999, 2005) decomposes every statistical
graphic into orthogonal, composable layers:

```
DATA  →  VARIABLES  →  ALGEBRA  →  SCALES  →  STATISTICS  →  GEOMETRY  →  COORDINATES  →  AESTHETICS
```

A "bar chart" and a "pie chart" are the same statistical mapping in different coordinate
systems (Cartesian vs polar). A scatterplot and a line chart are the same geometry with
different aesthetic bindings. The grammar makes this explicit.

Implementations: ggplot2 (R), Vega-Lite (JS), Polaris/Tableau. None are type-safe. None
support multi-modal output. None provide inverse mappings for interaction. Rust's type
system lets petalTongue do all three.

---

## Architecture

### Layer Decomposition

```
┌──────────────────────────────────────────────────────────────┐
│                      Grammar Expression                       │
│  GrammarExpr { data, variables, scales, stat, geom, coord,   │
│                aesthetics, facets, perspective }               │
├──────────────────────────────────────────────────────────────┤
│                      Grammar Compiler                         │
│  Validates expression, resolves defaults, applies Tufte       │
│  constraints, produces a RenderPlan                            │
├──────────────────────────────────────────────────────────────┤
│                       Render Plan                             │
│  Abstract geometric primitives with resolved aesthetics       │
│  (positions, colors, sizes, shapes, text, audio params)       │
├──────────────┬───────────────┬────────────────┬──────────────┤
│  egui/GUI    │  ratatui/TUI  │  Audio/Sonic   │  SVG/PNG     │
│  Compiler    │  Compiler     │  Compiler      │  Compiler    │
└──────────────┴───────────────┴────────────────┴──────────────┘
```

### The Grammar Expression

A grammar expression is an immutable, serializable description of a visualization.
It does not contain rendering code. It does not reference a backend. It is pure data
about the relationship between data and human perception.

```rust
pub struct GrammarExpr {
    pub data: DataSource,
    pub variables: Vec<VariableBinding>,
    pub scales: Vec<ScaleSpec>,
    pub statistics: Vec<StatSpec>,
    pub geometry: Vec<GeomSpec>,
    pub coordinates: CoordSpec,
    pub aesthetics: Vec<AestheticBinding>,
    pub facets: Option<FacetSpec>,
    pub perspective: Option<Perspective>,
    pub constraints: TufteConstraints,
}
```

This struct is `Serialize + Deserialize`. Other primals can send grammar expressions
to petalTongue via JSON-RPC:

```json
{
  "jsonrpc": "2.0",
  "method": "visualization.render",
  "params": {
    "data": {"source": "primal.health_metrics", "filter": "last_hour"},
    "variables": [
      {"name": "time", "field": "timestamp", "role": "x"},
      {"name": "cpu",  "field": "cpu_percent", "role": "y"},
      {"name": "host", "field": "primal_id", "role": "facet"}
    ],
    "scales": [
      {"variable": "time", "type": "temporal"},
      {"variable": "cpu",  "type": "linear", "domain": [0, 100]}
    ],
    "geometry": [{"type": "line"}, {"type": "area", "alpha": 0.2}],
    "coordinates": {"type": "cartesian"},
    "facets": {"variable": "host", "layout": "wrap", "columns": 3}
  },
  "id": 1
}
```

### Trait Hierarchy

#### Data Layer

```rust
pub trait DataSource: Send + Sync {
    fn schema(&self) -> &Schema;
    fn row_count(&self) -> usize;
    fn column(&self, name: &str) -> Option<&ColumnRef>;
    fn stream(&self) -> Option<Pin<Box<dyn Stream<Item = Row> + Send>>>;
}
```

`DataSource` can be backed by in-memory vectors, streaming JSON-RPC subscriptions,
or zero-copy `bytes::Bytes` from tarpc. The grammar does not care.

#### Scale Layer

```rust
pub trait Scale: Send + Sync {
    type Domain;
    type Range;

    fn transform(&self, input: &Self::Domain) -> Self::Range;
    fn inverse(&self, output: &Self::Range) -> Option<Self::Domain>;
    fn domain(&self) -> (Self::Domain, Self::Domain);
    fn range(&self) -> (Self::Range, Self::Range);
    fn breaks(&self) -> Vec<Self::Domain>;
    fn labels(&self) -> Vec<String>;
}
```

The `inverse` method is what makes the grammar interactive. When a user clicks a
pixel, the inverse scale maps screen coordinates back to data values. This is the
bridge between motor output and sensory input (SAME DAVE).

Built-in scale types:

| Scale | Domain | Range | Notes |
|-------|--------|-------|-------|
| `LinearScale` | `f64` | `f64` | Continuous numeric |
| `LogScale` | `f64` | `f64` | Logarithmic, base configurable |
| `TemporalScale` | `chrono::DateTime` | `f64` | Time axis with calendar-aware breaks |
| `CategoricalScale` | `String` | `f64` | Discrete categories to positions |
| `ColorScale` | `f64` | `[u8; 4]` | Sequential, diverging, or qualitative palettes |
| `SizeScale` | `f64` | `f64` | Area-proportional (Tufte: never radius) |
| `OrdinalScale` | `usize` | `f64` | Rank-order mapping |

#### Statistic Layer

Statistics transform data before geometry. They are pure functions from a data frame
to a data frame.

```rust
pub trait Statistic: Send + Sync {
    fn compute(&self, data: &DataFrame) -> Result<DataFrame>;
    fn required_aesthetics(&self) -> &[AestheticRole];
}
```

Built-in statistics:

| Stat | Purpose | Output Variables |
|------|---------|-----------------|
| `StatIdentity` | Pass-through | Same as input |
| `StatBin` | Histogram binning | `x`, `count`, `density` |
| `StatSmooth` | LOESS / moving average | `x`, `y`, `ymin`, `ymax` |
| `StatDensity` | Kernel density estimate | `x`, `density` |
| `StatBoxplot` | Five-number summary | `lower`, `middle`, `upper`, `ymin`, `ymax` |
| `StatCount` | Category counts | `x`, `count` |
| `StatSummary` | Aggregation (mean, median, etc.) | Configurable |

Heavy statistics (KDE, smoothing, large aggregations) can be offloaded to
barraCuda via `math.stat.*` JSON-RPC methods. See `UNIVERSAL_VISUALIZATION_PIPELINE.md`.

#### Geometry Layer

Geometries define the visual marks that represent data.

```rust
pub trait Geometry: Send + Sync {
    fn render(
        &self,
        data: &DataFrame,
        scales: &ScaleSet,
        coord: &dyn CoordinateSystem,
    ) -> Vec<Primitive>;

    fn required_aesthetics(&self) -> &[AestheticRole];
    fn optional_aesthetics(&self) -> &[AestheticRole];
}
```

`Primitive` is the abstract output:

```rust
pub enum Primitive {
    Point { x: f64, y: f64, size: f64, color: Color, shape: Shape },
    Line { points: Vec<(f64, f64)>, color: Color, width: f64 },
    Rect { x: f64, y: f64, w: f64, h: f64, fill: Color, stroke: Option<Color> },
    Text { x: f64, y: f64, content: String, size: f64, anchor: Anchor },
    Polygon { points: Vec<(f64, f64)>, fill: Color, stroke: Option<Color> },
    Arc { cx: f64, cy: f64, r: f64, start: f64, end: f64, fill: Color },
    Mesh { vertices: Vec<[f64; 3]>, indices: Vec<u32>, normals: Vec<[f64; 3]> },
}
```

`Mesh` is the escape hatch for 3D content (molecules, game worlds, universe
simulations). It gets compiled by the GPU-capable backends or forwarded to
barraCuda for tessellation.

Built-in geometries:

| Geom | Required Aesthetics | Primitives Produced |
|------|--------------------|--------------------|
| `GeomPoint` | x, y | `Point` |
| `GeomLine` | x, y | `Line` |
| `GeomBar` | x, y | `Rect` |
| `GeomArea` | x, y | `Polygon` |
| `GeomRibbon` | x, ymin, ymax | `Polygon` |
| `GeomText` | x, y, label | `Text` |
| `GeomTile` | x, y, fill | `Rect` (heatmap) |
| `GeomArc` | angle, fill | `Arc` (pie/donut) |
| `GeomErrorbar` | x, ymin, ymax | `Line` pairs |
| `GeomMesh3D` | x, y, z | `Mesh` |
| `GeomSphere` | x, y, z, radius | `Mesh` (icosphere) |
| `GeomCylinder` | x1, y1, z1, x2, y2, z2, radius | `Mesh` (bonds) |

#### Coordinate System Layer

```rust
pub trait CoordinateSystem: Send + Sync {
    fn project(&self, point: &[f64]) -> [f64; 2];
    fn inverse(&self, screen: &[f64; 2]) -> Option<Vec<f64>>;
    fn aspect_ratio(&self) -> Option<f64>;
    fn clip(&self) -> bool;
}
```

| Coord | Dimensions | Notes |
|-------|-----------|-------|
| `CartesianCoord` | 2D | Standard x/y, optional fixed aspect |
| `PolarCoord` | 2D | Angle + radius; pie charts, radar |
| `Perspective3DCoord` | 3D | Camera position, FOV; molecules, worlds |
| `OrthographicCoord` | 3D | No perspective distortion; technical/scientific |
| `GeoCoord` | 2D | Lat/lon projections (Mercator, etc.) |
| `HierarchicalCoord` | N-D | Nested zoom (galaxy → planet → surface) |

`HierarchicalCoord` is the key to universe-scale visualization. It composes
coordinate systems at different levels of detail, with smooth transitions
between them. The grammar expression specifies the hierarchy; barraCuda
computes the LOD transitions.

#### Aesthetic Bindings

Aesthetics map data variables to perceptual channels:

```rust
pub enum AestheticRole {
    X, Y, Z,
    Color, Fill, Stroke,
    Size, Shape, Alpha,
    Label, Tooltip,
    // Multi-modal extensions
    Pitch, Volume, Pan,         // Audio sonification
    Vibration, Pressure,         // Haptic
    Row, Column,                 // Faceting
}
```

The audio aesthetics (`Pitch`, `Volume`, `Pan`) mean the same grammar expression
can render as a visual chart OR an audio sonification. The grammar compiler
selects which aesthetics are active based on the available modality.

#### Faceting

```rust
pub struct FacetSpec {
    pub variable: String,
    pub layout: FacetLayout,
    pub scales: FacetScales,
}

pub enum FacetLayout {
    Wrap { columns: usize },
    Grid { rows: String, columns: String },
}

pub enum FacetScales {
    Fixed,
    FreeX,
    FreeY,
    Free,
}
```

Faceting produces Tufte's "small multiples" -- the single most effective
technique for showing change across a categorical variable. Fixed scales
enable comparison; free scales enable detail.

#### Interaction

> **Full specification**: See `INTERACTION_ENGINE_ARCHITECTURE.md`
>
> Interaction is a full subsystem -- not a field on the grammar expression.
> The Interaction Engine defines semantic intents (Select, Inspect, Navigate,
> Manipulate, Annotate, Command) that are modality-agnostic. Any input device
> produces the same `InteractionIntent`; the generalized inverse pipeline
> resolves it to `DataTarget` via per-modality `InversePipeline` implementations.

The grammar expression no longer carries an `InteractionSpec` enum. Instead:

1. Each `ModalityCompiler` produces a corresponding `InversePipeline` that
   maps modality-native events back to data space.
2. The `Perspective` on the grammar expression defines viewport, filters,
   orientation, and synchronization mode for multi-user collaboration.
3. `Scale::inverse` remains the core mechanism for mapping rendered
   coordinates back to data values within any modality.

```rust
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
```

Perspective enables the "6 vs 9" solution: the same `DataObject` rendered
through different perspectives produces different but valid representations.
Selection and focus operate on `DataObjectId` (perspective-invariant), not on
rendered primitives. See `INTERACTION_ENGINE_ARCHITECTURE.md` §5 for details.

---

## Grammar Compiler

The grammar compiler takes a `GrammarExpr` and produces a `RenderPlan`.

Compilation phases:

1. **Validation**: Check that all required aesthetics are bound, scales match
   variable types, coordinate system is compatible with geometry.

2. **Default Resolution**: Fill missing scales with sensible defaults (linear for
   numeric, categorical for string, temporal for datetime). Apply Tufte
   constraints to prune unnecessary chrome (see `TUFTE_CONSTRAINT_SYSTEM.md`).

3. **Statistics**: Evaluate `StatSpec` transforms on the data. Offload heavy
   computation to barraCuda if available.

4. **Scale Training**: Compute scale domains from data (or use explicit domains).
   Generate breaks and labels.

5. **Geometry Rendering**: Invoke `Geometry::render` to produce abstract
   `Primitive` values in normalized coordinates.

6. **Facet Layout**: Duplicate geometry across facet panels. Arrange panels
   according to `FacetLayout`.

7. **Constraint Checking**: Evaluate Tufte constraints (data-ink ratio, lie
   factor, redundancy). Emit warnings or auto-correct.

8. **Interaction Wiring**: Produce `InversePipeline` metadata for each
   modality compiler. See `INTERACTION_ENGINE_ARCHITECTURE.md` §4.

Output: a `RenderPlan` containing `Vec<Primitive>` plus scale metadata, labels,
legends, interaction handlers, and constraint diagnostics.

---

## Modality Compilers

Each rendering backend implements a modality compiler that translates a
`RenderPlan` into native output.

```rust
pub trait ModalityCompiler: Send + Sync {
    type Output;

    fn compile(&self, plan: &RenderPlan, viewport: &Viewport) -> Self::Output;
    fn supported_primitives(&self) -> &[PrimitiveKind];
    fn supported_aesthetics(&self) -> &[AestheticRole];
}
```

| Compiler | Output | Notes |
|----------|--------|-------|
| `EguiCompiler` | egui draw commands | Full interactivity, GPU-accelerated |
| `RatatuiCompiler` | ratatui widgets | Sparklines, braille dots, block chars |
| `SvgCompiler` | SVG string | Static export, web embedding |
| `PngCompiler` | PNG bytes | Headless rendering via tiny-skia |
| `AudioCompiler` | Audio sample buffer | Pitch=y, pan=x, volume=size |
| `ToadstoolCompiler` | WASM + framebuffer | Pure Rust display via Toadstool |
| `JsonCompiler` | JSON description | Machine-readable for other primals |

The `JsonCompiler` is important: it lets petalTongue serve grammar-rendered
visualizations to any client that can parse JSON, including web browsers,
other primals, or AI systems (Squirrel).

---

## Domain Applications

### Primal Ecosystem (Current)

Grammar expression for the topology view currently implemented ad-hoc in
`graph_canvas.rs`:

- DATA: primal list from biomeOS
- VARIABLES: primal_id (node), connection (edge), health (color), latency (size)
- GEOMETRY: GeomPoint (nodes) + GeomLine (edges)
- COORDINATES: force-directed layout (custom CoordSystem)
- AESTHETICS: health → color (green/yellow/red), latency → edge width

### Molecular Visualization

- DATA: PDB/mmCIF atom coordinates + bond table
- VARIABLES: x, y, z (position), element (color), bond_type (geometry)
- GEOMETRY: GeomSphere (atoms) + GeomCylinder (bonds)
- COORDINATES: Perspective3DCoord with orbit camera
- STATISTICS: optional electron density as StatDensity → GeomTile (isosurface)
- barraCuda: tessellation, lighting, molecular dynamics force computation

### Video Games

A game is a visualization of a simulation state with bidirectional interaction:

- DATA: entity component system (positions, health, inventory, terrain)
- VARIABLES: entity fields mapped to visual channels
- GEOMETRY: sprites (GeomTile), meshes (GeomMesh3D), particles (GeomPoint)
- COORDINATES: camera system (Perspective3DCoord or orthographic)
- INTERACTION: full SAME DAVE loop -- motor input modifies simulation state
- barraCuda: physics simulation, collision detection, particle systems

The grammar handles the rendering. The game loop is:
`simulate(dt) → update DataSource → recompile grammar → render`.

### Universe Simulation

- DATA: hierarchical (galaxy catalog → star systems → planetary bodies → surfaces)
- COORDINATES: HierarchicalCoord with LOD transitions
- GEOMETRY: GeomPoint at galactic scale, GeomSphere at stellar scale, GeomMesh3D
  at planetary scale
- SCALES: logarithmic spatial scales spanning 10^26 meters
- barraCuda: N-body gravitational simulation, LOD mesh generation, atmospheric
  scattering shaders

### Clinical / healthSpring Data

Grammar replaces the current ad-hoc `DataChannel` rendering:

- `TimeSeries` → GeomLine with TemporalScale
- `Distribution` → GeomBar with StatBin, or GeomArea with StatDensity
- `Bar` → GeomBar with CategoricalScale
- `Gauge` → GeomArc (polar) or GeomRect with annotation layers

---

## Crate Structure

```
crates/
  petal-tongue-grammar/          ← NEW
    src/
      lib.rs                     # Public API
      expr.rs                    # GrammarExpr and builder
      data.rs                    # DataSource trait + impls
      scale.rs                   # Scale trait + built-in scales
      stat.rs                    # Statistic trait + built-in stats
      geom.rs                    # Geometry trait + built-in geoms
      coord.rs                   # CoordinateSystem trait + built-ins
      aesthetic.rs               # AestheticRole, AestheticBinding
      facet.rs                   # FacetSpec, small multiples
      interaction.rs             # Perspective, InversePipeline trait (see INTERACTION_ENGINE_ARCHITECTURE.md)
      primitive.rs               # Primitive enum (abstract output)
      compiler.rs                # Grammar → RenderPlan
      render_plan.rs             # RenderPlan struct
      constraints.rs             # Tufte constraint evaluation
```

All types: `#![forbid(unsafe_code)]`, `Serialize + Deserialize`,
`Send + Sync`. No rendering backend dependencies. Pure data and traits.

---

## IPC Integration

### JSON-RPC Methods (Semantic Naming)

| Method | Direction | Purpose |
|--------|-----------|---------|
| `visualization.render` | Inbound | Render a grammar expression |
| `visualization.render.stream` | Inbound | Streaming grammar (live data) |
| `visualization.export` | Inbound | Export grammar to SVG/PNG/JSON |
| `visualization.interact` | Outbound | Report interaction events (selection, hover) |
| `visualization.capabilities` | Inbound | Query supported geoms, scales, coords |
| `visualization.validate` | Inbound | Validate a grammar expression without rendering |

### tarpc Service Definition

```rust
#[tarpc::service]
pub trait VisualizationService {
    async fn render(expr: GrammarExpr, viewport: Viewport) -> RenderResult;
    async fn render_stream(expr: GrammarExpr, viewport: Viewport) -> StreamHandle;
    async fn export(expr: GrammarExpr, format: ExportFormat) -> Bytes;
    async fn validate(expr: GrammarExpr) -> ValidationResult;
    async fn capabilities() -> CapabilitySet;
}
```

---

## Evolution Path

### Phase 1: Foundation (petal-tongue-grammar crate)

- Core traits: Scale, Geometry, CoordinateSystem, Statistic
- Built-in 2D scales and geometries (point, line, bar, area, text)
- CartesianCoord and PolarCoord
- Grammar compiler producing RenderPlan
- EguiCompiler as first modality compiler
- Port existing graph_canvas.rs topology view to grammar

### Phase 2: Multi-Modal

- RatatuiCompiler (sparklines, braille, block chars)
- AudioCompiler (sonification)
- SvgCompiler, PngCompiler
- Faceting (small multiples)
- Port remaining ad-hoc views to grammar

### Phase 3: Interaction Engine (see `INTERACTION_ENGINE_ARCHITECTURE.md`)

- Semantic intent model (modality-agnostic Select, Inspect, Navigate, etc.)
- InputAdapter trait + PointerAdapter, KeyboardAdapter implementations
- Generalized InversePipeline per ModalityCompiler
- Perspective system for multi-user, multi-modality collaboration
- IPC interaction protocol: `visualization.interact`, `.subscribe`, `.apply`
- Cross-panel linked selection via shared `DataObjectId`

### Phase 4: 3D + barraCuda

- Perspective3DCoord, OrthographicCoord
- GeomMesh3D, GeomSphere, GeomCylinder
- barraCuda IPC for tessellation, physics, lighting
- Molecule viewer, basic 3D scene

### Phase 5: Hierarchical + Universe Scale

- HierarchicalCoord with LOD transitions
- Streaming data sources
- N-body simulation via barraCuda
- Game loop integration

---

## References

- Wilkinson, L. (2005). *The Grammar of Graphics*. Springer. 2nd edition.
- Wickham, H. (2010). A Layered Grammar of Graphics. *Journal of Computational
  and Graphical Statistics*, 19(1), 3-28.
- Tufte, E.R. (2001). *The Visual Display of Quantitative Information*. 2nd edition.
- Satyanarayan, A. et al. (2017). Vega-Lite: A Grammar of Interactive Graphics.
  *IEEE Trans. Visualization and Computer Graphics*, 23(1).

---

**Status**: Ready for implementation  
**Blocking**: None (can start immediately; existing views continue working)  
**First Milestone**: Port topology view (graph_canvas.rs) to grammar expression
