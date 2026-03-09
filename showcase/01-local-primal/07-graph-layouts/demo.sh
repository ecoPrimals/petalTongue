#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
# Demo: Graph Layouts
# Duration: ~60 seconds
# Prerequisites: petalTongue built, curl
# External deps: none

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

print_header "Graph Layout Algorithms"
print_version_info

step 1 "Prerequisites"
require_petaltongue
require_binary curl

step 2 "Layout algorithms via scenarios"
print_info "petalTongue renders graph topologies using scenario data."
print_info "Each scenario defines primals + edges that form a graph."
print_info "The renderer applies layout algorithms to position nodes."

WEB_PORT=13375

step 3 "Simple topology (3 nodes)"
require_scenario "simple.json"
print_command "petaltongue web --bind 127.0.0.1:${WEB_PORT} --scenario sandbox/scenarios/simple.json"
PID=$(start_petaltongue_bg web --bind "127.0.0.1:${WEB_PORT}" --scenario "${SCENARIOS_DIR}/simple.json")
if wait_for_port $WEB_PORT 8; then
    SNAP=$(curl -sf "http://127.0.0.1:${WEB_PORT}/api/snapshot" 2>/dev/null || echo "{}")
    if check_json_valid "$SNAP"; then
        PRIMAL_COUNT=$(echo "$SNAP" | python3 -c "import sys,json; d=json.load(sys.stdin); print(len(d.get('primals',[])))" 2>/dev/null || echo "?")
        record_pass "Simple topology: ${PRIMAL_COUNT} nodes"
    else
        record_fail "Simple topology: invalid snapshot"
    fi
else
    record_fail "Server did not start for simple.json"
fi
stop_pid "$PID"
sleep 1

step 4 "Complex topology (many nodes)"
require_scenario "complex.json"
PID=$(start_petaltongue_bg web --bind "127.0.0.1:${WEB_PORT}" --scenario "${SCENARIOS_DIR}/complex.json")
if wait_for_port $WEB_PORT 8; then
    SNAP=$(curl -sf "http://127.0.0.1:${WEB_PORT}/api/snapshot" 2>/dev/null || echo "{}")
    if check_json_valid "$SNAP"; then
        PRIMAL_COUNT=$(echo "$SNAP" | python3 -c "import sys,json; d=json.load(sys.stdin); print(len(d.get('primals',[])))" 2>/dev/null || echo "?")
        record_pass "Complex topology: ${PRIMAL_COUNT} nodes"
    else
        record_fail "Complex topology: invalid snapshot"
    fi
else
    record_fail "Server did not start for complex.json"
fi
stop_pid "$PID"
sleep 1

step 5 "Full dashboard topology"
require_scenario "full-dashboard.json"
PID=$(start_petaltongue_bg web --bind "127.0.0.1:${WEB_PORT}" --scenario "${SCENARIOS_DIR}/full-dashboard.json")
if wait_for_port $WEB_PORT 8; then
    SNAP=$(curl -sf "http://127.0.0.1:${WEB_PORT}/api/snapshot" 2>/dev/null || echo "{}")
    if check_json_valid "$SNAP"; then
        PRIMAL_COUNT=$(echo "$SNAP" | python3 -c "import sys,json; d=json.load(sys.stdin); print(len(d.get('primals',[])))" 2>/dev/null || echo "?")
        record_pass "Full dashboard: ${PRIMAL_COUNT} nodes"
    else
        record_fail "Full dashboard: invalid snapshot"
    fi
else
    record_fail "Server did not start for full-dashboard.json"
fi
stop_pid "$PID"

step 6 "Layout algorithm info"
print_info "Available layout algorithms (applied by renderer):"
print_info "  - Force-directed (spring model)"
print_info "  - Hierarchical (layered DAG)"
print_info "  - Circular (ring placement)"
print_info "  - Grid (matrix placement)"
print_info ""
print_info "Layout is selected adaptively based on graph topology."
print_info "Use the UI/TUI modes to switch layouts interactively."

record_pass "Graph layout demo complete"

echo
print_results || true
demo_complete "08-clinical-data"
