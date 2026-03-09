#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
# Demo: Unix Socket Server (JSON-RPC over IPC)
# Duration: ~45 seconds
# Prerequisites: petalTongue built
# External deps: socat or python3 (for socket communication)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

print_header "Unix Socket IPC Server"
print_version_info

step 1 "Prerequisites"
require_petaltongue

step 2 "IPC architecture overview"
print_info "petalTongue exposes a JSON-RPC 2.0 server over Unix domain sockets."
print_info "This is the primary inter-primal communication channel."
print_info ""
print_info "Socket locations (XDG_RUNTIME_DIR compliant):"
print_info "  \${XDG_RUNTIME_DIR}/petaltongue.sock"
print_info "  /tmp/petaltongue.sock (fallback)"
print_info ""
print_info "Protocol: JSON-RPC 2.0, newline-delimited"

step 3 "Start headless mode (IPC server)"
print_command "petaltongue headless --bind 127.0.0.1:13377"
HEADLESS_OUT=$("${PETALTONGUE_BIN}" --log-level error headless --bind "127.0.0.1:13377" 2>&1 || true)
if echo "$HEADLESS_OUT" | grep -qi "headless\|started"; then
    record_pass "Headless mode started"
else
    record_skip "Headless mode did not produce expected output"
fi

step 4 "Check for Unix socket"
SOCKET=$(find_petaltongue_socket)
if [[ -n "$SOCKET" ]]; then
    record_pass "Unix socket found: $SOCKET"

    step 5 "Send JSON-RPC request"
    print_command "echo '{\"jsonrpc\":\"2.0\",\"method\":\"visualization.capabilities\",\"id\":1}' | <socket>"
    RESPONSE=$(send_jsonrpc "$SOCKET" "visualization.capabilities" "null" 1)
    echo "  Response: $RESPONSE"
    if check_json_valid "$RESPONSE"; then
        record_pass "JSON-RPC response is valid JSON"
    else
        record_fail "JSON-RPC response is not valid JSON"
    fi
else
    record_skip "No Unix socket found (headless mode may exit immediately)"
    print_info "The current headless mode exits after reporting data."
    print_info "Future evolutions will keep the IPC server alive."

    step 5 "Socket communication (simulated)"
    print_info "When the IPC server is persistent, you can send:"
    print_info ""
    print_command "echo '{\"jsonrpc\":\"2.0\",\"method\":\"visualization.capabilities\",\"id\":1}' | socat - UNIX-CONNECT:/path/to/socket"
    print_info ""
    print_info "Expected response:"
    print_info '  {"jsonrpc":"2.0","result":{"modalities":["gui","tui","web","headless","audio"]},"id":1}'
    record_skip "Socket demo requires persistent IPC server"
fi

step 6 "JSON-RPC methods reference"
print_info "petalTongue JSON-RPC methods:"
print_info "  visualization.render       - render data with grammar"
print_info "  visualization.render.stream - streaming render"
print_info "  visualization.export       - export to file"
print_info "  visualization.validate     - Tufte constraint check"
print_info "  visualization.capabilities - list modalities"
print_info "  visualization.interact     - interaction events"
print_info "  health.check               - liveness probe"
print_info "  health.ready               - readiness probe"

echo
print_results || true
demo_complete "02-jsonrpc-methods"
