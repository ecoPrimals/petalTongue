#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Demo: Scene Graph (declarative scene engine)
# Duration: ~30 seconds
# Prerequisites: petalTongue built
# External deps: none

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

print_header "Scene Graph (petal-tongue-scene)"
print_version_info

step 1 "Run scene graph operations"
print_command "cargo run --example scene_engine_demo -- scene-graph"
OUTPUT=$(cd "${PROJECT_ROOT}" && cargo run --example scene_engine_demo -- scene-graph 2>&1)
echo "$OUTPUT"

if echo "$OUTPUT" | grep -q "RESULT: scene graph operations OK"; then
    record_pass "Scene graph: create, add, flatten, find_by_data_id, remove"
else
    record_fail "Scene graph operations failed"
fi

step 2 "Verify node operations"
if echo "$OUTPUT" | grep -q "Flattened primitives:"; then
    record_pass "Flatten produces world-transformed primitives"
else
    record_fail "Flatten not working"
fi

if echo "$OUTPUT" | grep -q "find_by_data_id.*found"; then
    record_pass "Hit-testing via data_id works"
else
    record_fail "Hit-testing failed"
fi

print_results || true
demo_complete "01-local-primal/13-grammar-compilation"
