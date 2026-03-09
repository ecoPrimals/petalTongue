#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
# Demo: Grammar of Graphics compilation
# Duration: ~30 seconds
# Prerequisites: petalTongue built
# External deps: none

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

print_header "Grammar of Graphics Compilation"
print_version_info

step 1 "Compile grammar expression to scene graph"
print_command "cargo run --example scene_engine_demo -- grammar"
OUTPUT=$(cd "${PROJECT_ROOT}" && cargo run --example scene_engine_demo -- grammar 2>&1)
echo "$OUTPUT"

if echo "$OUTPUT" | grep -q "RESULT: grammar compilation OK"; then
    record_pass "GrammarExpr -> SceneGraph compilation"
else
    record_fail "Grammar compilation failed"
fi

step 2 "Verify grammar properties"
if echo "$OUTPUT" | grep -q "Variables: 2"; then
    record_pass "Variable bindings (x, y) resolved"
else
    record_fail "Variable bindings not found"
fi

if echo "$OUTPUT" | grep -q 'Domain: Some("health")'; then
    record_pass "Domain-aware grammar (health)"
else
    record_fail "Domain not set"
fi

step 3 "Verify scene output"
if echo "$OUTPUT" | grep -q "Scene nodes:"; then
    record_pass "Scene graph produced from grammar"
else
    record_fail "No scene graph output"
fi

print_results || true
demo_complete "01-local-primal/14-tufte-constraints"
