#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Demo: TUI Dashboard
# Duration: ~15 seconds (non-interactive verification only)
# Prerequisites: petalTongue built
# External deps: none
# Note: TUI requires a terminal; this demo verifies the mode is runnable

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

print_header "TUI Dashboard (Pure Rust, ratatui)"
print_version_info

step 1 "Prerequisites"
require_petaltongue

step 2 "Verify TUI mode accepts scenarios"
print_command "petaltongue tui --help"
TUI_HELP=$("${PETALTONGUE_BIN}" tui --help 2>&1)
echo "$TUI_HELP"

if echo "$TUI_HELP" | grep -q "\-\-scenario"; then
    record_pass "TUI accepts --scenario flag"
else
    record_fail "TUI missing --scenario flag"
fi

if echo "$TUI_HELP" | grep -q "\-\-refresh-rate"; then
    record_pass "TUI accepts --refresh-rate flag"
else
    record_fail "TUI missing --refresh-rate flag"
fi

step 3 "Interactive usage (informational)"
print_info "To launch TUI interactively, run:"
print_info ""
print_command "petaltongue tui --scenario sandbox/scenarios/simple.json"
print_info ""
print_info "Controls:"
print_info "  q / Ctrl+C  - quit"
print_info "  Tab         - cycle panels"
print_info "  Arrow keys  - navigate"
print_info ""
print_info "TUI renders the same data as web mode but in your terminal."
print_info "Built with ratatui (Pure Rust)."

record_pass "TUI mode available and documented"

echo
print_results || true
demo_complete "06-audio-export"
