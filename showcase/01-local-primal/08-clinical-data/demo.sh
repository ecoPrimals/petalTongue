#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Demo: Clinical Data (healthSpring DataChannel)
# Duration: ~45 seconds
# Prerequisites: petalTongue built, curl
# External deps: none

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

print_header "Clinical Data Rendering (healthSpring)"
print_version_info

step 1 "Prerequisites"
require_petaltongue
require_binary curl

step 2 "Inspect healthSpring diagnostic scenario"
require_scenario "healthspring-diagnostic.json"
print_command "cat sandbox/scenarios/healthspring-diagnostic.json | python3 -m json.tool | head -30"
python3 -c "
import json
with open('${SCENARIOS_DIR}/healthspring-diagnostic.json') as f:
    d = json.load(f)
print(f\"  Name: {d.get('name', '?')}\")
print(f\"  Description: {d.get('description', '?')}\")
print(f\"  Mode: {d.get('mode', '?')}\")
primals = d.get('ecosystem', {}).get('primals', [])
print(f\"  Primals: {len(primals)}\")
for p in primals[:5]:
    print(f\"    - {p.get('name','?')} ({p.get('type','?')}) health={p.get('health','?')}\")
" 2>/dev/null || print_info "Could not parse scenario"

WEB_PORT=13376

step 3 "Load healthSpring scenario via web mode"
print_command "petaltongue web --bind 127.0.0.1:${WEB_PORT} --scenario sandbox/scenarios/healthspring-diagnostic.json"
PID=$(start_petaltongue_bg web --bind "127.0.0.1:${WEB_PORT}" --scenario "${SCENARIOS_DIR}/healthspring-diagnostic.json")
if ! wait_for_port $WEB_PORT 10; then
    record_fail "Server did not start"
    print_results || true
    exit 1
fi
record_pass "Web server started with healthSpring scenario"

step 4 "Verify health endpoint"
HEALTH=$(curl -sf "http://127.0.0.1:${WEB_PORT}/health" 2>/dev/null || echo "")
if check_json_valid "$HEALTH"; then
    record_pass "/health returns valid JSON"
else
    record_fail "/health did not return valid JSON"
fi

step 5 "Fetch clinical data snapshot"
print_command "curl -s http://127.0.0.1:${WEB_PORT}/api/snapshot"
SNAPSHOT=$(curl -sf "http://127.0.0.1:${WEB_PORT}/api/snapshot" 2>/dev/null || echo "{}")
if check_json_valid "$SNAPSHOT"; then
    record_pass "/api/snapshot returns valid JSON"
    print_output "$(echo "$SNAPSHOT" | python3 -c "
import sys, json
d = json.load(sys.stdin)
primals = d.get('primals', [])
print(f'Primals in snapshot: {len(primals)}')
for p in primals[:3]:
    print(f'  {p.get(\"name\",\"?\")} - status: {p.get(\"status\",\"?\")}')
if len(primals) > 3:
    print(f'  ... and {len(primals)-3} more')
" 2>/dev/null || echo "$SNAPSHOT" | head -c 300)"
else
    record_fail "/api/snapshot invalid JSON"
fi

step 6 "Shutdown"
stop_pid "$PID"
record_pass "Server shutdown cleanly"

step 7 "Clinical data types (informational)"
print_info "healthSpring DataChannel renders these clinical types:"
print_info "  - TimeSeries: vital signs over time"
print_info "  - Distribution: lab value distributions"
print_info "  - Bar: categorical comparisons"
print_info "  - Gauge: real-time health metrics"
print_info ""
print_info "petalTongue maps these to appropriate visual/audio/TUI representations"
print_info "based on the active modality and Tufte constraint validation."

echo
print_results || true
demo_complete "09-domain-themes"
