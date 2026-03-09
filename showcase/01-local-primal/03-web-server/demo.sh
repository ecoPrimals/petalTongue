#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
# Demo: Web Server
# Duration: ~60 seconds
# Prerequisites: petalTongue built, curl
# External deps: none

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

print_header "Web Server (Pure Rust)"
print_version_info

step 1 "Prerequisites"
require_petaltongue
require_binary curl

WEB_PORT=13373

step 2 "Start web server with scenario"
require_scenario "simple.json"
print_command "petaltongue web --bind 127.0.0.1:${WEB_PORT} --scenario sandbox/scenarios/simple.json"
PID=$(start_petaltongue_bg web --bind "127.0.0.1:${WEB_PORT}" --scenario "${SCENARIOS_DIR}/simple.json")
if ! wait_for_port $WEB_PORT 10; then
    record_fail "Web server did not start"
    print_results || true
    exit 1
fi
record_pass "Web server listening on port ${WEB_PORT}"

step 3 "GET / - HTML dashboard"
print_command "curl -s http://127.0.0.1:${WEB_PORT}/"
HTML=$(curl -sf "http://127.0.0.1:${WEB_PORT}/" 2>/dev/null || echo "")
if echo "$HTML" | grep -qi "html"; then
    record_pass "/ returns HTML content"
    TITLE=$(echo "$HTML" | grep -oP '(?<=<title>).*?(?=</title>)' || echo "unknown")
    print_info "  Page title: ${TITLE}"
else
    record_fail "/ did not return HTML"
fi

step 4 "GET /health - JSON health check"
print_command "curl -s http://127.0.0.1:${WEB_PORT}/health"
HEALTH=$(curl -sf "http://127.0.0.1:${WEB_PORT}/health" 2>/dev/null || echo "")
echo "  $HEALTH"
if check_json_valid "$HEALTH"; then
    record_pass "/health returns valid JSON"
    check_json_field "$HEALTH" "status" "ok" || true
else
    record_fail "/health did not return valid JSON"
fi

step 5 "GET /api/status - system status"
print_command "curl -s http://127.0.0.1:${WEB_PORT}/api/status"
API_STATUS=$(curl -sf "http://127.0.0.1:${WEB_PORT}/api/status" 2>/dev/null || echo "")
echo "  $API_STATUS"
if check_json_valid "$API_STATUS"; then
    record_pass "/api/status returns valid JSON"
    check_json_field "$API_STATUS" "pure_rust" "True" || true
else
    record_fail "/api/status did not return valid JSON"
fi

step 6 "GET /api/primals - primal data"
print_command "curl -s http://127.0.0.1:${WEB_PORT}/api/primals"
PRIMALS=$(curl -sf "http://127.0.0.1:${WEB_PORT}/api/primals" 2>/dev/null || echo "")
if check_json_valid "$PRIMALS"; then
    record_pass "/api/primals returns valid JSON"
    print_output "$(echo "$PRIMALS" | python3 -c "import sys,json; print(json.dumps(json.load(sys.stdin), indent=2)[:400])" 2>/dev/null || echo "$PRIMALS")"
else
    record_fail "/api/primals did not return valid JSON"
fi

step 7 "GET /api/snapshot - full data snapshot"
print_command "curl -s http://127.0.0.1:${WEB_PORT}/api/snapshot"
SNAPSHOT=$(curl -sf "http://127.0.0.1:${WEB_PORT}/api/snapshot" 2>/dev/null || echo "")
if check_json_valid "$SNAPSHOT"; then
    record_pass "/api/snapshot returns valid JSON"
    SIZE=$(echo "$SNAPSHOT" | wc -c)
    print_info "  Snapshot size: ${SIZE} bytes"
else
    record_fail "/api/snapshot did not return valid JSON"
fi

step 8 "Shutdown"
stop_pid "$PID"
record_pass "Server shutdown cleanly"

echo
print_results || true
demo_complete "04-headless-api"
