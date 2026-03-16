#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Demo: Full Ecosystem Integration
# Duration: ~60 seconds
# Prerequisites: petalTongue built, curl
# External deps: multiple primals (optional - graceful degradation)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

print_header "Full Ecosystem Integration"
print_version_info

step 1 "Prerequisites"
require_petaltongue
require_binary curl

step 2 "Ecosystem probe"
print_info "Checking for all ecosystem primals..."
echo

PRIMALS_RUNNING=()
PRIMALS_MISSING=()

if check_songbird_running; then
    PRIMALS_RUNNING+=("Songbird (discovery)")
else
    PRIMALS_MISSING+=("Songbird (discovery)")
    print_skip "Songbird not running"
fi

if check_biomeos_running; then
    PRIMALS_RUNNING+=("biomeOS (orchestrator)")
else
    PRIMALS_MISSING+=("biomeOS (orchestrator)")
    print_skip "biomeOS not running"
fi

# Probe for other primals by socket
for primal in beardog toadstool nestgate squirrel; do
    RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
    if ls "${RUNTIME_DIR}/${primal}"*.sock /tmp/"${primal}"*.sock 2>/dev/null | head -1 > /dev/null 2>&1; then
        PRIMALS_RUNNING+=("$primal")
        print_success "$primal socket found"
    else
        PRIMALS_MISSING+=("$primal")
    fi
done

echo
print_info "Running: ${#PRIMALS_RUNNING[@]} primals"
for p in "${PRIMALS_RUNNING[@]:-}"; do
    [[ -n "$p" ]] && print_info "  + $p"
done
print_info "Missing: ${#PRIMALS_MISSING[@]} primals"
for p in "${PRIMALS_MISSING[@]:-}"; do
    [[ -n "$p" ]] && print_info "  - $p"
done

if [[ ${#PRIMALS_RUNNING[@]} -gt 0 ]]; then
    record_pass "Ecosystem partially available (${#PRIMALS_RUNNING[@]} primals)"
else
    record_skip "No primals running - full ecosystem demo requires other primals"
fi

WEB_PORT=13382

step 3 "Visualize full ecosystem (scenario fallback)"
require_scenario "full-dashboard.json"
print_command "petaltongue web --bind 127.0.0.1:${WEB_PORT} --scenario sandbox/scenarios/full-dashboard.json"
PID=$(start_petaltongue_bg web --bind "127.0.0.1:${WEB_PORT}" --scenario "${SCENARIOS_DIR}/full-dashboard.json")
if ! wait_for_port $WEB_PORT 10; then
    record_fail "Web server did not start"
    print_results || true
    exit 1
fi
record_pass "Full ecosystem visualization started"

step 4 "Ecosystem snapshot"
SNAPSHOT=$(curl -sf "http://127.0.0.1:${WEB_PORT}/api/snapshot" 2>/dev/null || echo "{}")
if check_json_valid "$SNAPSHOT"; then
    record_pass "Full ecosystem snapshot"
    print_output "$(echo "$SNAPSHOT" | python3 -c "
import sys, json
d = json.load(sys.stdin)
primals = d.get('primals', [])
edges = d.get('edges', [])
print(f'Primals: {len(primals)}, Edges: {len(edges)}')
types = {}
for p in primals:
    t = p.get('type', 'unknown')
    types[t] = types.get(t, 0) + 1
for t, c in sorted(types.items()):
    print(f'  {t}: {c}')
" 2>/dev/null || echo "$SNAPSHOT" | head -c 400)"
else
    record_fail "Could not parse ecosystem snapshot"
fi

step 5 "Shutdown"
stop_pid "$PID"
record_pass "Server shutdown"

step 6 "Full ecosystem architecture"
print_info "In production, petalTongue is the visualization layer for the ecosystem:"
print_info ""
print_info "  biomeOS (orchestrator) -> topology events -> petalTongue"
print_info "  Songbird (discovery)   -> primal registry -> petalTongue"
print_info "  BearDog (security)     -> trust scores    -> petalTongue"
print_info "  ToadStool (compute)    -> resource usage  -> petalTongue"
print_info "  NestGate (data)        -> storage metrics -> petalTongue"
print_info "  Squirrel (AI)          -> model state     -> petalTongue"
print_info "  healthSpring           -> clinical data   -> petalTongue"
print_info "  barraCuda (math)       -> GPU compute     -> petalTongue"
print_info ""
print_info "All communication via JSON-RPC 2.0 over Unix sockets."
print_info "All discovery via capability probing, no hardcoded names."

echo
print_results || true
demo_complete "(showcase complete)"
