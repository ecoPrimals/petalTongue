#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
# Demo: Physics bridge (barraCuda IPC)
# Duration: ~30 seconds
# Prerequisites: petalTongue built
# External deps: none

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

print_header "Physics Bridge (barraCuda IPC)"
print_version_info

step 1 "Run physics simulation"
print_command "cargo run --example scene_engine_demo -- physics"
OUTPUT=$(cd "${PROJECT_ROOT}" && cargo run --example scene_engine_demo -- physics 2>&1)
echo "$OUTPUT"

if echo "$OUTPUT" | grep -q "RESULT: physics bridge OK"; then
    record_pass "Physics bridge operational"
else
    record_fail "Physics bridge failed"
fi

step 2 "Verify CPU fallback"
if echo "$OUTPUT" | grep -q "After CPU Euler step"; then
    record_pass "CPU Euler integration (fallback when barraCuda unavailable)"
else
    record_fail "CPU fallback not working"
fi

step 3 "Verify IPC serialization"
if echo "$OUTPUT" | grep -q "IPC request (math.physics.nbody)"; then
    record_pass "IPC request serialized for math.physics.nbody"
else
    record_fail "IPC serialization failed"
fi

step 4 "Verify IPC response handling"
if echo "$OUTPUT" | grep -q "After IPC response"; then
    record_pass "IPC response applied (body positions updated)"
else
    record_fail "IPC response not applied"
fi

step 5 "Ecosystem delegation (informational)"
print_info "Physics compute delegation:"
print_info "  - petalTongue: scene description, animation, visualization"
print_info "  - barraCuda: math engine, WGSL shaders, physics simulations"
print_info "  - Toadstool: GPU hardware orchestration, dispatch"
print_info "  - coralReef: WGSL/SPIR-V to native GPU binary compilation"
print_info ""
print_info "IPC: math.physics.nbody JSON-RPC method"
print_info "CPU fallback: Euler integration when barraCuda unavailable"

print_results || true
demo_complete "02-ipc-protocol/01-unix-socket-server"
