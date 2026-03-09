#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
# Demo: SVG modality compiler (grammar -> SVG)
# Duration: ~30 seconds
# Prerequisites: petalTongue built
# External deps: none

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

print_header "SVG Modality Compiler"
print_version_info

step 1 "Compile grammar to SVG"
print_command "cargo run --example scene_engine_demo -- svg"
OUTPUT=$(cd "${PROJECT_ROOT}" && DEMO_OUTPUT_DIR="${DEMO_OUTPUT_DIR}" cargo run --example scene_engine_demo -- svg 2>&1)
echo "$OUTPUT"

if echo "$OUTPUT" | grep -q "RESULT: SVG modality OK"; then
    record_pass "SVG compilation from grammar expression"
else
    record_fail "SVG compilation failed"
fi

step 2 "Verify SVG output"
SVG_FILE="${DEMO_OUTPUT_DIR}/weekly_temperature.svg"
if [[ -f "$SVG_FILE" ]]; then
    record_pass "SVG file written: $SVG_FILE"
    SIZE=$(stat -c%s "$SVG_FILE" 2>/dev/null || stat -f%z "$SVG_FILE" 2>/dev/null || echo "?")
    print_info "File size: ${SIZE} bytes"

    if head -1 "$SVG_FILE" | grep -q "<svg"; then
        record_pass "Valid SVG document (starts with <svg)"
    else
        record_fail "Not a valid SVG document"
    fi
else
    record_fail "SVG file not written"
fi

step 3 "Audio sonification"
print_command "cargo run --example scene_engine_demo -- audio"
AUDIO_OUTPUT=$(cd "${PROJECT_ROOT}" && cargo run --example scene_engine_demo -- audio 2>&1)
echo "$AUDIO_OUTPUT"

if echo "$AUDIO_OUTPUT" | grep -q "RESULT: audio modality OK"; then
    record_pass "Audio sonification parameters generated"
else
    record_fail "Audio modality failed"
fi

step 4 "Accessibility description"
print_command "cargo run --example scene_engine_demo -- accessibility"
ACC_OUTPUT=$(cd "${PROJECT_ROOT}" && cargo run --example scene_engine_demo -- accessibility 2>&1)
echo "$ACC_OUTPUT"

if echo "$ACC_OUTPUT" | grep -q "RESULT: accessibility OK"; then
    record_pass "Accessibility text description generated"
else
    record_fail "Accessibility description failed"
fi

step 5 "Multi-modality (informational)"
print_info "petalTongue compiles one scene graph to multiple modalities:"
print_info "  - SVG: scalable vector graphics for web/export"
print_info "  - Audio: spatial sonification (frequency, pan, amplitude)"
print_info "  - Description: text for screen readers and accessibility"
print_info "  - Terminal: character grid for TUI (future)"
print_info "  - GPU: commands for Toadstool dispatch (future)"

print_results || true
demo_complete "01-local-primal/18-physics-bridge"
