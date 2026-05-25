#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Demo: JSON-RPC Methods
# Duration: ~30 seconds
# Prerequisites: petalTongue built, curl
# External deps: none

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

print_header "JSON-RPC Method Catalog"
print_version_info

step 1 "Prerequisites"
require_petaltongue
require_binary curl

WEB_PORT=13378

step 2 "Start web server as HTTP JSON-RPC proxy"
print_info "Web mode exposes REST endpoints that map to JSON-RPC methods."
print_info "Each endpoint demonstrates the data flow that the IPC protocol uses."
require_scenario "complex.json"
PID=$(start_petaltongue_bg web --bind "127.0.0.1:${WEB_PORT}" --scenario "${SCENARIOS_DIR}/complex.json")
if ! wait_for_port $WEB_PORT 10; then
    record_fail "Web server did not start"
    print_results || true
    exit 1
fi
record_pass "Web server as JSON-RPC proxy"

step 3 "health.check -> GET /health"
print_command "curl -s http://127.0.0.1:${WEB_PORT}/health"
HEALTH=$(curl -sf "http://127.0.0.1:${WEB_PORT}/health" 2>/dev/null || echo "")
if check_json_valid "$HEALTH"; then
    record_pass "health.check: valid JSON response"
    echo "  $HEALTH"
else
    record_fail "health.check: invalid response"
fi

step 4 "visualization.capabilities -> GET /api/status"
print_command "curl -s http://127.0.0.1:${WEB_PORT}/api/status"
STATUS=$(curl -sf "http://127.0.0.1:${WEB_PORT}/api/status" 2>/dev/null || echo "")
if check_json_valid "$STATUS"; then
    record_pass "visualization.capabilities: valid JSON"
    echo "  $STATUS"
else
    record_fail "visualization.capabilities: invalid response"
fi

step 5 "visualization.render -> GET /api/snapshot"
print_command "curl -s http://127.0.0.1:${WEB_PORT}/api/snapshot"
SNAPSHOT=$(curl -sf "http://127.0.0.1:${WEB_PORT}/api/snapshot" 2>/dev/null || echo "")
if check_json_valid "$SNAPSHOT"; then
    record_pass "visualization.render: valid JSON snapshot"
    SIZE=$(echo "$SNAPSHOT" | wc -c)
    print_info "  Response size: ${SIZE} bytes"
else
    record_fail "visualization.render: invalid response"
fi

step 6 "visualization.render (primals) -> GET /api/primals"
print_command "curl -s http://127.0.0.1:${WEB_PORT}/api/primals"
PRIMALS=$(curl -sf "http://127.0.0.1:${WEB_PORT}/api/primals" 2>/dev/null || echo "")
if check_json_valid "$PRIMALS"; then
    record_pass "primal data endpoint: valid JSON"
    COUNT=$(echo "$PRIMALS" | python3 -c "import sys,json; d=json.load(sys.stdin); print(len(d.get('primals',[])))" 2>/dev/null || echo "?")
    print_info "  Primals returned: ${COUNT}"
else
    record_fail "primal data endpoint: invalid response"
fi

step 7 "Shutdown"
stop_pid "$PID"
record_pass "Server shutdown"

step 8 "Method mapping reference"
print_info "JSON-RPC (Unix socket)  <->  HTTP (web mode)"
print_info "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
print_info "health.check            <->  GET /health"
print_info "visualization.capabilities <-> GET /api/status"
print_info "visualization.render    <->  GET /api/snapshot"
print_info "visualization.interact  <->  (WebSocket, future)"

echo
print_results || true
demo_complete "03-health-monitoring"
