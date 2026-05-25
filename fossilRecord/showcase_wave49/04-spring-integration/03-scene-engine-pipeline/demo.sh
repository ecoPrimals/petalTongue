#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Demo: Full scene engine pipeline (grammar -> compile -> render -> multi-modality)
# Duration: ~45 seconds
# Prerequisites: petalTongue built
# External deps: none

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

print_header "Scene Engine: Full Pipeline"
print_version_info

step 1 "Run full pipeline (all scene engine subsystems)"
print_command "cargo run --example scene_engine_demo -- all"
OUTPUT=$(cd "${PROJECT_ROOT}" && DEMO_OUTPUT_DIR="${DEMO_OUTPUT_DIR}" cargo run --example scene_engine_demo -- all 2>&1)

PASS_ALL=true
for subsystem in "grammar compilation" "scene graph operations" "tufte validation" "math objects" "animation system" "SVG modality" "audio modality" "accessibility" "physics bridge"; do
    if echo "$OUTPUT" | grep -q "RESULT: $subsystem OK"; then
        record_pass "$subsystem"
    else
        record_fail "$subsystem"
        PASS_ALL=false
    fi
done

step 2 "Verify SVG artifact"
SVG_FILE="${DEMO_OUTPUT_DIR}/weekly_temperature.svg"
if [[ -f "$SVG_FILE" ]]; then
    SIZE=$(stat -c%s "$SVG_FILE" 2>/dev/null || stat -f%z "$SVG_FILE" 2>/dev/null || echo "?")
    record_pass "SVG artifact: ${SIZE} bytes"
else
    record_fail "No SVG artifact produced"
fi

step 3 "End-to-end pipeline (informational)"
print_info "The full pipeline:"
print_info "  Data -> GrammarExpr -> GrammarCompiler -> SceneGraph"
print_info "  SceneGraph -> TufteReport (quality validation)"
print_info "  SceneGraph -> Animation (transitions, easing)"
print_info "  SceneGraph -> SvgCompiler (SVG output)"
print_info "  SceneGraph -> AudioCompiler (sonification)"
print_info "  SceneGraph -> DescriptionCompiler (accessibility)"
print_info "  SceneGraph -> PhysicsWorld (barraCuda delegation)"
print_info ""
print_info "Any primal can send GrammarExpr JSON to petalTongue"
print_info "via visualization.render IPC. petalTongue compiles to"
print_info "the best available output modality."

echo
if [[ "$PASS_ALL" == "true" ]]; then
    echo "$OUTPUT" | grep "All demos passed"
fi
print_results || true
demo_complete ""
