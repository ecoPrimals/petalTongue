#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
# Demo: healthSpring data push via visualization.render
# Duration: ~60 seconds
# Prerequisites: petalTongue built, curl
# External deps: none (uses mock-biomeos or standalone)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

print_header "healthSpring Clinical Data Push"
print_version_info

WEB_PORT=13390

step 1 "Prerequisites"
require_petaltongue
require_binary curl

step 2 "Start petalTongue with healthSpring scenario"
require_scenario "healthspring-diagnostic.json"
print_command "petaltongue web --bind 127.0.0.1:${WEB_PORT} --scenario healthspring-diagnostic.json"
PID=$(start_petaltongue_bg web --bind "127.0.0.1:${WEB_PORT}" --scenario "${SCENARIOS_DIR}/healthspring-diagnostic.json")
if ! wait_for_port $WEB_PORT 10; then
    record_fail "Server did not start"
    print_results || true
    exit 1
fi
record_pass "Web server started with healthSpring diagnostic data"

step 3 "Verify health endpoint"
HEALTH=$(curl -sf "http://127.0.0.1:${WEB_PORT}/health" 2>/dev/null || echo "")
if check_json_valid "$HEALTH"; then
    record_pass "Health endpoint returns valid JSON"
else
    record_fail "Health endpoint failed"
fi

step 4 "Fetch clinical data snapshot"
SNAPSHOT=$(curl -sf "http://127.0.0.1:${WEB_PORT}/api/snapshot" 2>/dev/null || echo "{}")
if check_json_valid "$SNAPSHOT"; then
    record_pass "Clinical snapshot available"
    print_output "$(echo "$SNAPSHOT" | python3 -c "
import sys, json
d = json.load(sys.stdin)
primals = d.get('primals', [])
print(f'  Primals: {len(primals)}')
for p in primals[:3]:
    print(f'    {p.get(\"name\",\"?\")} ({p.get(\"type\",\"?\")})')
" 2>/dev/null || echo "(parse failed)")"
else
    record_fail "Clinical snapshot not available"
fi

step 5 "Test additional clinical scenarios"
for scenario in clinical-trt-standard-pellet clinical-trt-young-athlete healthspring-endocrine; do
    if [[ -f "${SCENARIOS_DIR}/${scenario}.json" ]]; then
        record_pass "Scenario available: ${scenario}.json"
    else
        record_skip "Scenario not found: ${scenario}.json"
    fi
done

step 6 "Shutdown"
stop_pid "$PID"
record_pass "Clean shutdown"

step 7 "Spring integration (informational)"
print_info "healthSpring pushes clinical data to petalTongue via IPC:"
print_info "  - visualization.render: one-shot rendering request"
print_info "  - visualization.render.stream: incremental streaming"
print_info "  - UiConfig: panel visibility, mode, zoom, theme"
print_info "  - DataChannel types: TimeSeries, Distribution, Bar, Gauge"
print_info "  - Domain-aware palettes: health, physics, ecology, neural..."

echo
print_results || true
demo_complete "04-spring-integration/02-biomeos-atomic-viz"
