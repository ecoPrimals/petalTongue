#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Demo: Math objects (Manim-style)
# Duration: ~30 seconds
# Prerequisites: petalTongue built
# External deps: none

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

print_header "Math Objects (Manim-style Rendering)"
print_version_info

step 1 "Create math objects"
print_command "cargo run --example scene_engine_demo -- math-objects"
OUTPUT=$(cd "${PROJECT_ROOT}" && cargo run --example scene_engine_demo -- math-objects 2>&1)
echo "$OUTPUT"

if echo "$OUTPUT" | grep -q "RESULT: math objects OK"; then
    record_pass "Math objects compile to primitives"
else
    record_fail "Math objects failed"
fi

step 2 "Verify object types"
for obj in "NumberLine" "Axes" "FunctionPlot" "ParametricCurve" "VectorField"; do
    if echo "$OUTPUT" | grep -q "$obj"; then
        record_pass "$obj rendered"
    else
        record_fail "$obj not found"
    fi
done

step 3 "Verify coordinate mapping"
if echo "$OUTPUT" | grep -q "round-trip"; then
    record_pass "data_to_screen / screen_to_data round-trip"
else
    record_fail "Coordinate mapping failed"
fi

step 4 "Math objects (informational)"
print_info "Manim-style math objects compile to rendering primitives:"
print_info "  - NumberLine: axis with ticks and labels"
print_info "  - Axes: 2D Cartesian with arrows and grid"
print_info "  - FunctionPlot: y=f(x) sampled as polyline"
print_info "  - ParametricCurve: (x(t), y(t)) sampled curve"
print_info "  - VectorField: arrows at grid points"

print_results || true
demo_complete "01-local-primal/16-animation-system"
