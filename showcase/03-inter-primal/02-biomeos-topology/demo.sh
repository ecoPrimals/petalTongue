#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
# Demo: biomeOS Topology Visualization
# Duration: ~45 seconds
# Prerequisites: petalTongue built, curl
# External deps: biomeOS (optional - graceful skip with fallback data)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

print_header "biomeOS Topology Visualization"
print_version_info

step 1 "Prerequisites"
require_petaltongue
require_binary curl

step 2 "Check for biomeOS"
print_info "Probing for biomeOS ecosystem orchestrator..."
if check_biomeos_running; then
    BIOMEOS_AVAILABLE=true
    record_pass "biomeOS is running"
else
    BIOMEOS_AVAILABLE=false
    record_skip "biomeOS not running - using fallback scenario"
    print_info "biomeOS provides ecosystem topology data."
    print_info "Using live-ecosystem.json scenario as fallback."
fi

WEB_PORT=13380

step 3 "Load topology data"
if [[ "$BIOMEOS_AVAILABLE" == "true" ]]; then
    print_info "Fetching live topology from biomeOS Neural API..."
    LIVE_DATA=$(curl -sf "http://localhost:3000/api/v1/topology" 2>/dev/null || echo "")
    if check_json_valid "$LIVE_DATA"; then
        record_pass "Live topology data from biomeOS"
        print_output "$(echo "$LIVE_DATA" | python3 -c "import sys,json; d=json.load(sys.stdin); print(json.dumps(d, indent=2)[:300])" 2>/dev/null || echo "$LIVE_DATA")"
    else
        record_fail "Could not fetch biomeOS topology"
    fi
fi

require_scenario "live-ecosystem.json"
print_command "petaltongue web --bind 127.0.0.1:${WEB_PORT} --scenario sandbox/scenarios/live-ecosystem.json"
PID=$(start_petaltongue_bg web --bind "127.0.0.1:${WEB_PORT}" --scenario "${SCENARIOS_DIR}/live-ecosystem.json")
if ! wait_for_port $WEB_PORT 10; then
    record_fail "Web server did not start"
    print_results || true
    exit 1
fi
record_pass "Topology visualization server started"

step 4 "Fetch rendered topology"
SNAPSHOT=$(curl -sf "http://127.0.0.1:${WEB_PORT}/api/snapshot" 2>/dev/null || echo "{}")
if check_json_valid "$SNAPSHOT"; then
    record_pass "Topology snapshot returned"
    print_output "$(echo "$SNAPSHOT" | python3 -c "
import sys, json
d = json.load(sys.stdin)
primals = d.get('primals', [])
edges = d.get('edges', [])
print(f'Primals: {len(primals)}, Edges: {len(edges)}')
for p in primals[:5]:
    status = p.get('status', '?')
    print(f'  {p.get(\"name\",\"?\")} ({p.get(\"type\",\"?\")}) - {status}')
if len(primals) > 5:
    print(f'  ... and {len(primals)-5} more')
" 2>/dev/null || echo "$SNAPSHOT" | head -c 300)"
else
    record_fail "Could not fetch topology snapshot"
fi

step 5 "Shutdown"
stop_pid "$PID"
record_pass "Server shutdown"

step 6 "biomeOS integration reference"
print_info "biomeOS provides the Neural API for ecosystem orchestration."
print_info "petalTongue visualizes the topology graph that biomeOS manages."
print_info ""
print_info "Data flow:"
print_info "  biomeOS Neural API -> JSON -> petalTongue scenario -> render"
print_info ""
print_info "In production, petalTongue subscribes to biomeOS events"
print_info "for live topology updates (visualization.render.stream)."

echo
print_results || true
demo_complete "03-ecosystem-health"
