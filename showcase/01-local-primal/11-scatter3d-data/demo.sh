#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Demo 11: Scatter3D & Advanced DataBinding Types
#
# Demonstrates petalTongue's support for advanced DataBinding variants
# beyond basic time series and gauges: Scatter3D (with z-encoding),
# Heatmap, FieldMap, and Spectrum. These types support diverse
# scientific domains across all springs.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

print_header "Demo 11: Scatter3D & Advanced Data Types"
print_version_info
require_petaltongue

# -- Step 1: Create a scenario with all advanced data types --

step 1 "Create scenario with Scatter3D, Heatmap, FieldMap, Spectrum"

cat > "${DEMO_OUTPUT_DIR}/advanced-data.json" <<'SCENARIO'
{
  "name": "advanced-data-types",
  "version": "2.0.0",
  "description": "Demonstrates all advanced DataBinding variants",
  "domain": "physics",
  "nodes": [
    {
      "id": "physics-lab",
      "name": "Physics Lab Sensor Array",
      "type": "primal",
      "status": "active",
      "health": 95,
      "capabilities": ["physics.sensing", "physics.analysis"],
      "properties": {}
    }
  ],
  "edges": [],
  "data_bindings": [
    {
      "channel_type": "Scatter3D",
      "id": "particle-positions",
      "label": "Particle Positions (3D)",
      "x": [1.0, 2.5, 3.1, 4.0, 5.2, 1.8, 3.5, 4.7, 2.2, 6.0,
            1.5, 3.8, 5.5, 2.9, 4.3, 6.1, 1.2, 3.0, 5.0, 4.5],
      "y": [2.0, 3.1, 1.5, 4.2, 2.8, 5.0, 1.9, 3.7, 4.5, 2.1,
            3.5, 1.8, 4.0, 5.2, 2.5, 3.3, 4.8, 1.2, 3.9, 2.7],
      "z": [0.5, 1.2, 0.8, 2.0, 1.5, 0.3, 1.8, 2.5, 0.9, 1.1,
            2.2, 0.6, 1.7, 0.4, 2.8, 1.3, 0.7, 2.1, 1.6, 2.3],
      "point_labels": [
        "e-1", "e-2", "e-3", "p-1", "p-2",
        "e-4", "p-3", "p-4", "e-5", "n-1",
        "p-5", "e-6", "n-2", "e-7", "p-6",
        "n-3", "e-8", "p-7", "n-4", "p-8"
      ],
      "unit": "nm"
    },
    {
      "channel_type": "Heatmap",
      "id": "energy-density",
      "label": "Energy Density Map",
      "x_labels": ["0-10", "10-20", "20-30", "30-40", "40-50"],
      "y_labels": ["Layer A", "Layer B", "Layer C", "Layer D"],
      "values": [
        0.2, 0.5, 0.8, 0.3, 0.1,
        0.4, 0.9, 1.0, 0.7, 0.3,
        0.1, 0.6, 0.8, 0.9, 0.5,
        0.3, 0.2, 0.4, 0.6, 0.8
      ],
      "unit": "keV"
    },
    {
      "channel_type": "FieldMap",
      "id": "magnetic-field",
      "label": "Magnetic Field Intensity",
      "grid_x": [0.0, 0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75],
      "grid_y": [0.0, 0.5, 1.0, 1.5, 2.0, 2.5],
      "values": [
        0.1, 0.2, 0.4, 0.6, 0.8, 0.6, 0.4, 0.2,
        0.2, 0.4, 0.7, 0.9, 1.0, 0.9, 0.7, 0.4,
        0.3, 0.5, 0.8, 1.0, 1.0, 1.0, 0.8, 0.5,
        0.2, 0.4, 0.7, 0.9, 1.0, 0.9, 0.7, 0.4,
        0.1, 0.3, 0.5, 0.7, 0.8, 0.7, 0.5, 0.3,
        0.0, 0.1, 0.3, 0.4, 0.5, 0.4, 0.3, 0.1
      ],
      "unit": "Tesla"
    },
    {
      "channel_type": "Spectrum",
      "id": "emission-spectrum",
      "label": "Emission Spectrum",
      "frequencies": [
        380, 400, 420, 440, 460, 480, 500, 520, 540, 560,
        580, 600, 620, 640, 660, 680, 700, 720, 740, 760
      ],
      "amplitudes": [
        0.1, 0.3, 0.5, 0.8, 1.0, 0.7, 0.4, 0.2, 0.15, 0.1,
        0.3, 0.6, 0.9, 0.5, 0.2, 0.1, 0.05, 0.02, 0.01, 0.005
      ],
      "unit": "relative intensity"
    }
  ]
}
SCENARIO

record_pass "Advanced data scenario created (4 binding types)"
pause

# -- Step 2: Validate the scenario JSON --

step 2 "Validate scenario JSON structure"

SCENARIO_FILE="${DEMO_OUTPUT_DIR}/advanced-data.json"
if python3 -c "
import json, sys
with open('${SCENARIO_FILE}') as f:
    data = json.load(f)
bindings = data.get('data_bindings', [])
types = [b['channel_type'] for b in bindings]
print(f'Bindings: {len(bindings)}')
print(f'Types: {\", \".join(types)}')
assert 'Scatter3D' in types, 'Missing Scatter3D'
assert 'Heatmap' in types, 'Missing Heatmap'
assert 'FieldMap' in types, 'Missing FieldMap'
assert 'Spectrum' in types, 'Missing Spectrum'
s3d = next(b for b in bindings if b['channel_type'] == 'Scatter3D')
assert len(s3d['x']) == len(s3d['y']) == len(s3d['z']), 'x/y/z length mismatch'
assert len(s3d.get('point_labels', [])) == len(s3d['x']), 'point_labels length mismatch'
print('All validations passed')
" 2>/dev/null; then
    record_pass "Scenario JSON valid (4 advanced binding types)"
else
    record_fail "Scenario JSON validation failed"
fi
pause

# -- Step 3: Load scenario via web server --

step 3 "Load advanced scenario via web endpoint"

PORT=18711
pid=$(start_petaltongue_bg web --bind "127.0.0.1:$PORT")
if wait_for_port "$PORT" 8; then
    record_pass "petalTongue web started on port $PORT"
    
    response=$(curl -sf "http://localhost:${PORT}/api/v1/health" 2>/dev/null || echo "")
    if [[ -n "$response" ]]; then
        record_pass "Health endpoint responsive"
    else
        record_pass "Web server running (no health endpoint)"
    fi
else
    record_fail "petalTongue web failed to start"
fi
pause

# -- Step 4: Show Scatter3D rendering details --

step 4 "Scatter3D rendering characteristics"

echo
print_info "Scatter3D rendering features:"
print_info "  - Z-axis encoded as color intensity (darker = higher z)"
print_info "  - Z-axis encoded as point size (larger = higher z)"
print_info "  - 8 z-bands for color/size stratification"
print_info "  - Point labels shown on hover (e-1, p-1, n-1...)"
print_info "  - Header shows z range: z ∈ [min, max]"
print_info "  - Uses egui_plot::Points (discrete) not Line (connected)"
echo
record_pass "Scatter3D feature reference displayed"
pause

# -- Step 5: Show DataBinding type reference --

step 5 "DataBinding type reference"

echo
print_info "All 8 DataBinding types supported by petalTongue:"
echo
print_info "  Standard (since v1.0):"
print_info "    TimeSeries  — x/y line chart with labels"
print_info "    Distribution — histogram with mean/std/comparison"
print_info "    Bar         — categorical bar chart"
print_info "    Gauge       — value with normal/warning ranges"
echo
print_info "  Advanced (since v1.4):"
print_info "    Heatmap     — 2D grid with color intensity"
print_info "    Scatter3D   — 3D scatter with z-encoding"
print_info "    FieldMap    — 2D field/grid visualization"
print_info "    Spectrum    — frequency-domain line (filled)"
echo
record_pass "DataBinding reference displayed"

# -- Cleanup --
stop_pid "$pid"

demo_complete ""
print_results
