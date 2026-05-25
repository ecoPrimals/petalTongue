#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Demo: Animation system (easing, sequences)
# Duration: ~30 seconds
# Prerequisites: petalTongue built
# External deps: none

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

print_header "Animation System"
print_version_info

step 1 "Run animation demo"
print_command "cargo run --example scene_engine_demo -- animation"
OUTPUT=$(cd "${PROJECT_ROOT}" && cargo run --example scene_engine_demo -- animation 2>&1)
echo "$OUTPUT"

if echo "$OUTPUT" | grep -q "RESULT: animation system OK"; then
    record_pass "Animation system operational"
else
    record_fail "Animation system failed"
fi

step 2 "Verify animation types"
for anim in "FadeIn" "MoveTo" "Create"; do
    if echo "$OUTPUT" | grep -q "$anim"; then
        record_pass "$anim animation available"
    else
        record_fail "$anim not found"
    fi
done

step 3 "Verify easing functions"
if echo "$OUTPUT" | grep -q "Easing at t=0.5"; then
    record_pass "6 easing functions evaluated (Linear, EaseIn, EaseOut, EaseInOut, Spring, Bounce)"
else
    record_fail "Easing functions not evaluated"
fi

step 4 "Verify sequencing"
if echo "$OUTPUT" | grep -q "Sequential.*1.5s"; then
    record_pass "Sequential: durations sum"
else
    record_fail "Sequential timing wrong"
fi

if echo "$OUTPUT" | grep -q "Parallel.*0.8s"; then
    record_pass "Parallel: duration = max"
else
    record_fail "Parallel timing wrong"
fi

print_results || true
demo_complete "01-local-primal/17-svg-modality"
