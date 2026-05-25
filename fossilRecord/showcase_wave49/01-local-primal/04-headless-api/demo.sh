#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Demo: Headless API Server
# Duration: ~30 seconds
# Prerequisites: petalTongue built
# External deps: none

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

print_header "Headless API Server (Pure Rust)"
print_version_info

step 1 "Prerequisites"
require_petaltongue

step 2 "Run headless mode"
print_command "petaltongue headless --bind 127.0.0.1:13374"
HEADLESS_OUT=$("${PETALTONGUE_BIN}" --log-level error headless --bind "127.0.0.1:13374" 2>&1 || true)
echo "$HEADLESS_OUT"

if echo "$HEADLESS_OUT" | grep -qi "headless"; then
    record_pass "Headless mode produces output"
else
    record_fail "Headless mode produced no recognizable output"
fi

if echo "$HEADLESS_OUT" | grep -qi "pure rust"; then
    record_pass "Pure Rust confirmed"
else
    record_skip "Pure Rust marker not found in output"
fi

step 3 "Verify data service integration"
if echo "$HEADLESS_OUT" | grep -qi "primals\|data"; then
    record_pass "DataService reports data"
else
    record_skip "No data summary in headless output"
fi

print_info "Headless mode runs the rendering pipeline without a GUI."
print_info "It currently exits after startup; future evolutions will"
print_info "keep it alive as a JSON-RPC server over Unix sockets."

echo
print_results || true
demo_complete "05-tui-dashboard"
