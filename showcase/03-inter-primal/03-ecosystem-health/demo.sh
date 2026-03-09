#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
# Demo: Ecosystem Health Dashboard
# Duration: ~45 seconds
# Prerequisites: petalTongue built, curl
# External deps: other primals (optional - uses scenarios as fallback)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

print_header "Ecosystem Health Dashboard"
print_version_info

step 1 "Prerequisites"
require_petaltongue
require_binary curl

step 2 "Probe ecosystem primals"
print_info "Checking for running primals..."
PRIMALS_FOUND=0

if check_songbird_running; then
    PRIMALS_FOUND=$((PRIMALS_FOUND + 1))
else
    print_skip "Songbird not running"
fi

if check_biomeos_running; then
    PRIMALS_FOUND=$((PRIMALS_FOUND + 1))
else
    print_skip "biomeOS not running"
fi

print_info "Live primals found: ${PRIMALS_FOUND}"
if [[ $PRIMALS_FOUND -eq 0 ]]; then
    print_info "No live primals detected. Using unhealthy.json scenario"
    print_info "to demonstrate health dashboard with mixed-state data."
fi

WEB_PORT=13381

step 3 "Load health scenario"
require_scenario "unhealthy.json"
print_command "petaltongue web --bind 127.0.0.1:${WEB_PORT} --scenario sandbox/scenarios/unhealthy.json"
PID=$(start_petaltongue_bg web --bind "127.0.0.1:${WEB_PORT}" --scenario "${SCENARIOS_DIR}/unhealthy.json")
if ! wait_for_port $WEB_PORT 10; then
    record_fail "Web server did not start"
    print_results || true
    exit 1
fi
record_pass "Health dashboard server started"

step 4 "Examine health states"
SNAPSHOT=$(curl -sf "http://127.0.0.1:${WEB_PORT}/api/snapshot" 2>/dev/null || echo "{}")
if check_json_valid "$SNAPSHOT"; then
    record_pass "Health snapshot returned"
    print_output "$(echo "$SNAPSHOT" | python3 -c "
import sys, json
d = json.load(sys.stdin)
primals = d.get('primals', [])
healthy = sum(1 for p in primals if p.get('status') == 'healthy')
degraded = sum(1 for p in primals if p.get('status') == 'degraded')
unhealthy = sum(1 for p in primals if p.get('status') not in ('healthy', 'degraded'))
print(f'Total: {len(primals)} | Healthy: {healthy} | Degraded: {degraded} | Unhealthy: {unhealthy}')
for p in primals:
    h = p.get('health', '?')
    s = p.get('status', '?')
    print(f'  {p.get(\"name\",\"?\")} - {s} (health: {h})')
" 2>/dev/null || echo "$SNAPSHOT" | head -c 400)"
else
    record_fail "Could not parse health snapshot"
fi

step 5 "Shutdown"
stop_pid "$PID"
record_pass "Server shutdown"

step 6 "Health dashboard integration"
print_info "In a live ecosystem, petalTongue aggregates health from:"
print_info "  - Songbird (discovery + health aggregation)"
print_info "  - biomeOS (Neural API orchestration state)"
print_info "  - Direct primal health probes (health.check)"
print_info ""
print_info "Visualization adapts to health state:"
print_info "  Healthy:   green nodes, harmonic audio tones"
print_info "  Degraded:  yellow nodes, dissonant audio"
print_info "  Unhealthy: red nodes, alarm tones"

echo
print_results || true
demo_complete "04-multi-primal-tui"
