#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
# Demo: Tufte constraint validation
# Duration: ~30 seconds
# Prerequisites: petalTongue built
# External deps: none

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

print_header "Tufte Constraint Validation"
print_version_info

step 1 "Validate primitives against Tufte principles"
print_command "cargo run --example scene_engine_demo -- tufte"
OUTPUT=$(cd "${PROJECT_ROOT}" && cargo run --example scene_engine_demo -- tufte 2>&1)
echo "$OUTPUT"

if echo "$OUTPUT" | grep -q "RESULT: tufte validation OK"; then
    record_pass "Tufte constraint evaluation"
else
    record_fail "Tufte validation failed"
fi

step 2 "Verify constraints"
if echo "$OUTPUT" | grep -q "DataInkRatio"; then
    record_pass "Data-ink ratio constraint checked"
else
    record_fail "Data-ink ratio missing"
fi

if echo "$OUTPUT" | grep -q "ChartjunkDetection"; then
    record_pass "Chartjunk detection constraint checked"
else
    record_fail "Chartjunk detection missing"
fi

if echo "$OUTPUT" | grep -q "Overall score:"; then
    record_pass "Overall quality score computed"
else
    record_fail "No overall score"
fi

step 3 "Machine-checked quality (informational)"
print_info "Tufte constraints enforce visualization quality principles:"
print_info "  - Data-ink ratio: maximize data-carrying ink"
print_info "  - Lie factor: visual effect proportional to data"
print_info "  - Chartjunk: minimize non-data decoration"
print_info "  - Color accessibility: ensure color-blind readability"
print_info "  - Data density: information per unit area"

print_results || true
demo_complete "01-local-primal/15-math-objects"
