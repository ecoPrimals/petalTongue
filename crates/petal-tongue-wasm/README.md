# petal-tongue-wasm

Client-side WASM rendering module for petalTongue (WS-4). Compiles the
Grammar of Graphics pipeline to `wasm32-unknown-unknown` so springs can
embed offline-capable visualization rendering in browser UIs without
calling petalTongue's server-side RPC.

## Motivation

petalTongue normally renders visualizations server-side: a spring sends
a `visualization.render.grammar` JSON-RPC request, and petalTongue
returns SVG. This works well but requires the petalTongue server to be
running. For offline-capable web UIs (e.g. wetSpring's Explorer,
sporePrint living content, lithoSpore deployments), the same grammar
pipeline can run directly in the browser via WASM.

## API

All functions accept and return JSON strings for cross-language
compatibility. Errors are returned as strings prefixed with `"Error:"`.

### Grammar rendering

| Function | Input | Output |
|----------|-------|--------|
| `render_grammar(grammar, data)` | `GrammarExpr` JSON + data array JSON | SVG string |
| `render_grammar_to_modality(grammar, data, modality)` | Same + modality (`"svg"`, `"description"`, `"terminal"`) | Rendered string |

Faceted grammars (with `facets` field set) are automatically compiled
via `compile_faceted` for small-multiples output.

### DataBinding rendering

| Function | Input | Output |
|----------|-------|--------|
| `render_binding(binding, domain)` | `DataBinding` JSON + domain hint | SVG string |
| `render_binding_to_modality(binding, domain, modality)` | Same + modality | Rendered string |
| `render_binding_with_thresholds(binding, domain, thresholds)` | Same + `ThresholdRange[]` JSON | SVG with status coloring |

All 13 `DataBinding` channel types are supported: `timeseries`,
`distribution`, `bar`, `gauge`, `heatmap`, `scatter`, `scatter3d`,
`fieldmap`, `spectrum`, `game_scene`, `soundscape`, `genome_track`,
`circular_map`.

### Dashboard / batch rendering

| Function | Input | Output |
|----------|-------|--------|
| `render_dashboard(bindings, config)` | `DataBinding[]` JSON + config JSON | SVG dashboard |
| `render_bindings(bindings, domain)` | `DataBinding[]` JSON + domain hint | JSON array of `{id, label, svg}` |

Dashboard config accepts: `layout` (`"grid"`, `"vertical"`,
`"horizontal"`), `max_columns`, `panel_width`, `panel_height`,
`spacing`, `title`, `domain`.

### Scene graph

| Function | Input | Output |
|----------|-------|--------|
| `compile_scene(grammar, data)` | `GrammarExpr` JSON + data array JSON | `SceneGraph` JSON |
| `render_scene(scene)` | `SceneGraph` JSON | SVG string |
| `render_scene_to_modality(scene, modality)` | Same + modality | Rendered string |

### Tufte validation

| Function | Input | Output |
|----------|-------|--------|
| `validate_grammar(grammar, data)` | `GrammarExpr` JSON + data array JSON | Tufte report JSON |

Evaluates all 7 Tufte constraints (data-ink ratio, lie factor,
chartjunk, color accessibility, data density, smallest effective
difference, small multiples).

### Meta

| Function | Input | Output |
|----------|-------|--------|
| `version()` | — | Version string |

## Usage from JavaScript

```js
import init, { render_grammar, render_binding, render_dashboard, version } from './petal_tongue_wasm.js';

await init();
console.log(`petalTongue WASM v${version()}`);

// Render a grammar expression
const grammar = JSON.stringify({
  data_source: "measurements",
  variables: [
    { name: "x", field: "time", role: "X" },
    { name: "y", field: "value", role: "Y" }
  ],
  scales: [
    { variable: "x", scale_type: "Linear" },
    { variable: "y", scale_type: "Linear" }
  ],
  geometry: "Line",
  coordinate: "Cartesian",
  aesthetics: []
});

const data = JSON.stringify([
  { time: 0, value: 1.2 },
  { time: 1, value: 3.4 },
  { time: 2, value: 2.8 }
]);

const svg = render_grammar(grammar, data);
document.getElementById('chart').innerHTML = svg;

// Render a DataBinding directly (the type springs push via RPC)
const binding = JSON.stringify({
  channel_type: "timeseries",
  id: "glucose",
  label: "Blood Glucose",
  x_label: "Time (hr)",
  y_label: "mg/dL",
  unit: "mg/dL",
  x_values: [0, 1, 2, 3],
  y_values: [90, 120, 95, 88]
});

const svg2 = render_binding(binding, "health");
document.getElementById('binding-chart').innerHTML = svg2;

// Render a multi-panel dashboard
const bindings = JSON.stringify([
  { channel_type: "timeseries", id: "ts", label: "Trend", ... },
  { channel_type: "bar", id: "bar", label: "Summary", ... }
]);
const dashboardSvg = render_dashboard(bindings, '{"title":"Patient Overview","domain":"health"}');
document.getElementById('dashboard').innerHTML = dashboardSvg;
```

## Building

```bash
# Check compiles for WASM
cargo check --target wasm32-unknown-unknown -p petal-tongue-wasm

# Build WASM binary (requires wasm-pack)
wasm-pack build crates/petal-tongue-wasm --target web

# Run tests (native)
cargo test -p petal-tongue-wasm
```

## Architecture

```
petal-tongue-types    (DataBinding, ThresholdRange — serde only)
       ↓
petal-tongue-scene    (GrammarCompiler, SvgCompiler, SceneGraph — portable)
       ↓
petal-tongue-wasm     (wasm_bindgen exports — wasm32-unknown-unknown)
```

The types crate was extracted from `petal-tongue-core` to break the
dependency on non-portable crates (tokio, reqwest, rustix). The scene
crate's only dependencies are `bytes`, `serde`, `serde_json`, and
`tracing` — all wasm32 compatible.

### Server vs WASM parity

| Capability | Server RPC | WASM |
|------------|-----------|------|
| Grammar → SVG | `visualization.render.grammar` | `render_grammar` |
| Grammar → modality | `visualization.render.grammar` + modality | `render_grammar_to_modality` |
| DataBinding → SVG | `visualization.render` | `render_binding` |
| DataBinding → modality | `visualization.export` | `render_binding_to_modality` |
| Threshold coloring | `visualization.export` + thresholds | `render_binding_with_thresholds` |
| Dashboard | `visualization.render.dashboard` | `render_dashboard` |
| Batch render | `visualization.render` (session) | `render_bindings` |
| SceneGraph export | `visualization.render.grammar` | `compile_scene` |
| Pre-built scene render | `visualization.render.scene` | `render_scene` |
| Tufte validation | `visualization.validate` | `validate_grammar` |
| Faceted grammars | Automatic | Automatic |
| Streaming | `visualization.render.stream` | N/A (server-only) |
| Session state | Built-in | N/A (stateless by design) |
| Audio/haptic/braille | `visualization.export` | N/A (browser APIs) |

## Integration Pattern for Springs

Springs with web facades can embed this WASM module to render grammar
visualizations client-side. The pattern:

1. Spring web UI loads `petal_tongue_wasm.js` + `.wasm`
2. Data is fetched from the spring's API (JSON)
3. Grammar expressions are constructed client-side
4. `render_grammar()` or `render_binding()` produces SVG in-browser
5. No petalTongue server needed for rendering

When petalTongue _is_ available, springs should prefer the RPC path
(`visualization.render.grammar`) for richer features (streaming,
session state, audio/haptic modalities). The WASM module is the
offline fallback and sovereign rendering path.

## License

AGPL-3.0-or-later
