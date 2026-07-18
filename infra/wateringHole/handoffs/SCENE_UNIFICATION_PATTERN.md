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

---

*Pattern reference for primals needing unified 2D/3D scene rendering.*
