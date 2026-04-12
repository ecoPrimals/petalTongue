#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Demo: UniBin Modes
# Duration: ~60 seconds
# Prerequisites: petalTongue built
# External deps: curl

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

print_header "UniBin: 1 Binary, 6 Subcommands"
print_version_info

step 1 "Prerequisites"
require_petaltongue

step 2 "Enumerate all 6 subcommands"
print_command "petaltongue --help"
HELP=$("${PETALTONGUE_BIN}" --help 2>&1)
for mode in ui tui web headless server status; do
    if echo "$HELP" | grep -qw "$mode"; then
        record_pass "Mode found: $mode"
    else
        record_fail "Mode missing: $mode"
    fi
done

step 3 "Status mode (Pure Rust)"
print_command "petaltongue status --format json"
STATUS=$("${PETALTONGUE_BIN}" --log-level error status --format json 2>&1)
if check_json_valid "$STATUS"; then
    record_pass "status --format json returns valid JSON"
else
    record_fail "status --format json returned invalid JSON"
fi

step 4 "Web mode (Pure Rust) - start, probe, stop"
WEB_PORT=13370
print_command "petaltongue web --bind 127.0.0.1:${WEB_PORT}"
if check_port_free $WEB_PORT; then
    PID=$(start_petaltongue_bg web --bind "127.0.0.1:${WEB_PORT}")
    if wait_for_port $WEB_PORT 8; then
        HEALTH=$(curl -sf "http://127.0.0.1:${WEB_PORT}/health" 2>/dev/null || echo "")
        if check_json_valid "$HEALTH"; then
            record_pass "web /health returns valid JSON"
        else
            record_fail "web /health did not return valid JSON"
        fi
    else
        record_fail "web mode did not bind within 8s"
    fi
    stop_pid "$PID"
else
    record_skip "Port $WEB_PORT in use, skipping web mode test"
fi

step 5 "Headless mode (Pure Rust) - start, verify exit"
print_command "petaltongue headless --bind 127.0.0.1:13371"
HEADLESS_OUT=$("${PETALTONGUE_BIN}" --log-level error headless --bind "127.0.0.1:13371" 2>&1 || true)
if echo "$HEADLESS_OUT" | grep -qi "headless"; then
    record_pass "headless mode produces output"
else
    record_fail "headless mode produced no recognizable output"
fi

step 6 "TUI mode help"
print_command "petaltongue tui --help"
TUI_HELP=$("${PETALTONGUE_BIN}" tui --help 2>&1)
if echo "$TUI_HELP" | grep -q "scenario"; then
    record_pass "tui mode accepts --scenario flag"
else
    record_fail "tui mode missing --scenario flag"
fi

step 7 "UI mode help (requires display, not launched)"
print_command "petaltongue ui --help"
UI_HELP=$("${PETALTONGUE_BIN}" ui --help 2>&1)
if echo "$UI_HELP" | grep -q "scenario"; then
    record_pass "ui mode accepts --scenario flag"
else
    record_fail "ui mode missing --scenario flag"
fi
print_info "UI mode requires display server; use 'petaltongue ui' interactively"

echo
print_results || true
demo_complete "02-scenario-loading"
