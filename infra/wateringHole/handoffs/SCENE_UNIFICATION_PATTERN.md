# Scene Unification Pattern — 2D-as-3D-slice

**Wave**: 150h | **Date**: July 18, 2026 | **From**: petalTongue on eastGate

---

## Architecture

petalTongue's scene graph supports both 2D and 3D in a unified hierarchy:

- **2D scenes** use `Transform2D` and are rendered with an orthographic camera at z=0
- **3D scenes** use `Transform3D` with perspective or custom orthographic cameras
- **Mixed scenes** work: nodes without explicit `transform_3d` are auto-embedded at z=0

This enables a single rendering pipeline for narrative, scientific, geospatial,
and molecular visualizations.

## Types

```rust
// Camera + Projection (in petal-tongue-scene/src/transform.rs)
pub enum Projection {
    Orthographic { width, height, near, far },
    Perspective { fov_y, aspect, near, far },
}

pub struct Camera {
    pub transform: Transform3D,
    pub projection: Projection,
}

// SceneNode (in petal-tongue-scene/src/scene_graph/node.rs)
pub struct SceneNode {
    pub transform: Transform2D,
    pub transform_3d: Option<Transform3D>,  // overrides 2D for 3D renderers
    // ...
}
```

## Usage

```rust
// 2D scene (unchanged — backward compatible)
let graph = compiler.compile(&expr_2d, &data);
let flat_2d = graph.flatten();  // Vec<(Transform2D, &Primitive)>

// 3D-aware rendering
let flat_3d = graph.flatten_3d();  // Vec<(Transform3D, &Primitive, &NodeId)>
let camera = graph.effective_camera();  // Camera (ortho 2D if not set)

// Grammar with z-axis
let expr = GrammarExpr::new("data", GeometryType::Sphere)
    .with_x("x").with_y("y").with_z("depth");
expr.coordinate = CoordinateSystem::Perspective3D;
```

## Key Design Decisions

1. **Non-breaking**: `transform_3d` is `Option<T>` with `#[serde(skip_serializing_if)]`
2. **Auto-embed**: `effective_transform_3d()` creates 3D from 2D when no explicit set
3. **Camera defaults**: Missing camera → orthographic 800×600 at z=0
4. **Grammar integration**: `VariableRole::Z` + `CoordinateSystem::Perspective3D` → auto-camera
5. **SVG viewport**: Derived from camera projection — orthographic uses explicit dims, perspective uses aspect ratio
6. **3D geometry**: `Sphere`, `Cylinder`, `Mesh3D` compile to `Primitive::Mesh` with proper tessellation
7. **Ribbon evolved**: Produces `Polygon` from `ymin`/`ymax` data fields (no longer a placeholder)

## 3D Geometry Compilation

```rust
// Sphere — UV-sphere tessellation at data-driven position + radius
let expr = GrammarExpr::new("data", GeometryType::Sphere)
    .with_x("x").with_y("y").with_z("z");
// Data rows: {"x": 0, "y": 0, "z": 0, "radius": 1.5}
// → Primitive::Mesh with position=[x,y,z], proper normals

// Cylinder — ring tessellation with data-driven radius/height
let expr = GrammarExpr::new("data", GeometryType::Cylinder)
    .with_x("x").with_y("y");
// Data rows: {"x": 0, "y": 0, "radius": 0.5, "height": 3.0}

// Mesh3D — pre-built vertex/index passthrough
// Data rows: {"vertices": [[x,y,z], ...], "indices": [0, 1, 2, ...]}
```

---

*Pattern reference for primals needing unified 2D/3D scene rendering.
All 4 phases complete — no remaining work.*
