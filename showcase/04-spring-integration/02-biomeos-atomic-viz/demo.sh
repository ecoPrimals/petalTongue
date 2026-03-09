#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
# Demo: biomeOS atomic deployment visualization
# Duration: ~60 seconds
# Prerequisites: petalTongue built, curl, mock-biomeos or real biomeOS
# External deps: biomeOS (graceful skip if absent)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

print_header "biomeOS Atomic Deployment Visualization"
print_version_info

step 1 "Prerequisites"
require_petaltongue
require_binary curl

step 2 "Check for biomeOS"
if check_biomeos_running; then
    BIOMEOS_AVAILABLE=true
    print_info "biomeOS detected -- will use live topology"
else
    BIOMEOS_AVAILABLE=false
    print_info "biomeOS not running -- using mock data (graceful degradation)"
fi

WEB_PORT=13391

step 3 "Start petalTongue with ecosystem scenario"
require_scenario "live-ecosystem.json"
print_command "petaltongue web --bind 127.0.0.1:${WEB_PORT} --scenario live-ecosystem.json"
PID=$(start_petaltongue_bg web --bind "127.0.0.1:${WEB_PORT}" --scenario "${SCENARIOS_DIR}/live-ecosystem.json")
if ! wait_for_port $WEB_PORT 10; then
    record_fail "Server did not start"
    print_results || true
    exit 1
fi
record_pass "Web server started with ecosystem scenario"

step 4 "Verify topology visualization"
SNAPSHOT=$(curl -sf "http://127.0.0.1:${WEB_PORT}/api/snapshot" 2>/dev/null || echo "{}")
if check_json_valid "$SNAPSHOT"; then
    record_pass "Ecosystem topology snapshot available"
    python3 -c "
import sys, json
d = json.load(sys.stdin)
primals = d.get('primals', [])
print(f'  Primals in topology: {len(primals)}')
for p in primals[:5]:
    health = p.get('health', 'unknown')
    print(f'    {p.get(\"name\",\"?\")} [{p.get(\"type\",\"?\")}] health={health}')
if len(primals) > 5:
    print(f'    ... and {len(primals)-5} more')
" <<< "$SNAPSHOT" 2>/dev/null || print_info "(topology parse)"
else
    record_fail "Topology snapshot not available"
fi

step 5 "Shutdown"
stop_pid "$PID"
record_pass "Clean shutdown"

step 6 "Atomic deployment model (informational)"
print_info "biomeOS atomic deployments are visualized as ecosystem topologies:"
print_info "  - Primals: nodes in the topology graph"
print_info "  - Health: color-coded (green/yellow/red)"
print_info "  - Connections: edges show IPC relationships"
print_info "  - Families: grouped by FAMILY_ID"
print_info "  - Trust levels: indicated by node rings"
print_info ""
print_info "petalTongue discovers topology at runtime via:"
print_info "  - Unix socket scanning (XDG_RUNTIME_DIR)"
print_info "  - Songbird mesh registration"
print_info "  - biomeOS /api/v1/topology endpoint"

echo
print_results || true
demo_complete "04-spring-integration/03-scene-engine-pipeline"
