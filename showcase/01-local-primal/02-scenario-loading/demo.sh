#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
# Demo: Scenario Loading
# Duration: ~60 seconds
# Prerequisites: petalTongue built, curl
# External deps: none

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

print_header "Scenario Loading"
print_version_info

step 1 "Prerequisites"
require_petaltongue
require_binary curl

step 2 "Available scenarios"
print_command "ls sandbox/scenarios/*.json"
SCENARIO_COUNT=$(ls -1 "${SCENARIOS_DIR}"/*.json 2>/dev/null | wc -l)
record_pass "Found ${SCENARIO_COUNT} scenario files"
for f in "${SCENARIOS_DIR}"/*.json; do
    NAME=$(python3 -c "import json; print(json.load(open('$f')).get('name','?'))" 2>/dev/null || basename "$f")
    print_info "  $(basename "$f") - ${NAME}"
done

WEB_PORT=13372

step 3 "Load simple.json scenario"
require_scenario "simple.json"
print_command "petaltongue web --bind 127.0.0.1:${WEB_PORT} --scenario sandbox/scenarios/simple.json"
PID=$(start_petaltongue_bg web --bind "127.0.0.1:${WEB_PORT}" --scenario "${SCENARIOS_DIR}/simple.json")
if wait_for_port $WEB_PORT 8; then
    SNAP=$(curl -sf "http://127.0.0.1:${WEB_PORT}/api/snapshot" 2>/dev/null || echo "{}")
    if check_json_valid "$SNAP"; then
        record_pass "simple.json: /api/snapshot returns valid JSON"
    else
        record_fail "simple.json: /api/snapshot invalid"
    fi
else
    record_fail "simple.json: server did not start"
fi
stop_pid "$PID"
sleep 1

step 4 "Load complex.json scenario"
require_scenario "complex.json"
PID=$(start_petaltongue_bg web --bind "127.0.0.1:${WEB_PORT}" --scenario "${SCENARIOS_DIR}/complex.json")
if wait_for_port $WEB_PORT 8; then
    SNAP=$(curl -sf "http://127.0.0.1:${WEB_PORT}/api/snapshot" 2>/dev/null || echo "{}")
    if check_json_valid "$SNAP"; then
        record_pass "complex.json: /api/snapshot returns valid JSON"
    else
        record_fail "complex.json: /api/snapshot invalid"
    fi
else
    record_fail "complex.json: server did not start"
fi
stop_pid "$PID"
sleep 1

step 5 "Load healthspring-diagnostic.json scenario"
require_scenario "healthspring-diagnostic.json"
PID=$(start_petaltongue_bg web --bind "127.0.0.1:${WEB_PORT}" --scenario "${SCENARIOS_DIR}/healthspring-diagnostic.json")
if wait_for_port $WEB_PORT 8; then
    SNAP=$(curl -sf "http://127.0.0.1:${WEB_PORT}/api/snapshot" 2>/dev/null || echo "{}")
    if check_json_valid "$SNAP"; then
        record_pass "healthspring-diagnostic.json: /api/snapshot returns valid JSON"
    else
        record_fail "healthspring-diagnostic.json: /api/snapshot invalid"
    fi
else
    record_fail "healthspring-diagnostic.json: server did not start"
fi
stop_pid "$PID"

echo
print_results || true
demo_complete "03-web-server"
