#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
# Demo: Health Monitoring Protocol
# Duration: ~30 seconds
# Prerequisites: petalTongue built, curl
# External deps: none

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

print_header "Health Monitoring Protocol"
print_version_info

step 1 "Prerequisites"
require_petaltongue
require_binary curl

WEB_PORT=13379

step 2 "Start web server"
require_scenario "simple.json"
PID=$(start_petaltongue_bg web --bind "127.0.0.1:${WEB_PORT}" --scenario "${SCENARIOS_DIR}/simple.json")
if ! wait_for_port $WEB_PORT 10; then
    record_fail "Web server did not start"
    print_results || true
    exit 1
fi
record_pass "Web server started"

step 3 "Liveness probe"
print_command "curl -sf http://127.0.0.1:${WEB_PORT}/health"
print_info "Liveness: Is the process alive and responsive?"
HEALTH=$(curl -sf "http://127.0.0.1:${WEB_PORT}/health" 2>/dev/null || echo "")
if check_json_valid "$HEALTH"; then
    record_pass "Liveness probe: OK"
    echo "  $HEALTH"
else
    record_fail "Liveness probe: failed"
fi

step 4 "Readiness probe (via /api/status)"
print_command "curl -sf http://127.0.0.1:${WEB_PORT}/api/status"
print_info "Readiness: Is the service ready to serve data?"
STATUS=$(curl -sf "http://127.0.0.1:${WEB_PORT}/api/status" 2>/dev/null || echo "")
if check_json_valid "$STATUS"; then
    record_pass "Readiness probe: OK"
    echo "  $STATUS"
else
    record_fail "Readiness probe: failed"
fi

step 5 "Repeated health checks (simulate monitoring)"
print_info "Polling /health 5 times (1s interval) to demonstrate stability..."
CHECKS_OK=0
CHECKS_FAIL=0
for i in $(seq 1 5); do
    H=$(curl -sf -o /dev/null -w "%{http_code}" "http://127.0.0.1:${WEB_PORT}/health" 2>/dev/null || echo "000")
    if [[ "$H" == "200" ]]; then
        CHECKS_OK=$((CHECKS_OK + 1))
        print_output "Check $i: HTTP $H (ok)"
    else
        CHECKS_FAIL=$((CHECKS_FAIL + 1))
        print_output "Check $i: HTTP $H (fail)"
    fi
    sleep 1
done

if [[ $CHECKS_OK -ge 4 ]]; then
    record_pass "Health stability: ${CHECKS_OK}/5 checks passed"
else
    record_fail "Health stability: only ${CHECKS_OK}/5 checks passed"
fi

step 6 "Shutdown"
stop_pid "$PID"
record_pass "Server shutdown"

step 7 "Health protocol reference"
print_info "ecoPrimals health protocol (from wateringHole):"
print_info "  health.check  - liveness (is process running?)"
print_info "  health.ready  - readiness (can serve requests?)"
print_info "  health.status - detailed status with metrics"
print_info ""
print_info "All primals implement this protocol for ecosystem monitoring."
print_info "Songbird aggregates health from all registered primals."

echo
print_results || true
demo_complete "03-inter-primal/01-songbird-discovery"
