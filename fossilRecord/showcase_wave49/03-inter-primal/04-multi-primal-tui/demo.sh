#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Demo: Multi-Primal TUI
# Duration: ~20 seconds
# Prerequisites: petalTongue built
# External deps: other primals (optional)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

print_header "Multi-Primal TUI"
print_version_info

step 1 "Prerequisites"
require_petaltongue

step 2 "TUI with ecosystem data"
print_info "The TUI mode renders the same data as web mode, but in your terminal."
print_info "With live primals, it shows real-time health + topology."
print_info "Without them, it uses scenario data."
print_info ""
print_info "To launch the multi-primal TUI:"
print_info ""
print_command "petaltongue tui --scenario sandbox/scenarios/live-ecosystem.json"
print_info ""
print_info "This loads the live-ecosystem scenario which includes:"

require_scenario "live-ecosystem.json"
python3 -c "
import json
with open('${SCENARIOS_DIR}/live-ecosystem.json') as f:
    d = json.load(f)
primals = d.get('ecosystem', {}).get('primals', [])
for p in primals:
    print(f'  - {p.get(\"name\",\"?\")} ({p.get(\"type\",\"?\")})')
" 2>/dev/null || print_info "  (could not parse scenario)"
record_pass "TUI scenario documented"

step 3 "TUI controls reference"
print_info ""
print_info "TUI controls:"
print_info "  q / Ctrl+C  - quit"
print_info "  Tab         - cycle panels (topology / health / details)"
print_info "  Arrow keys  - navigate nodes"
print_info "  Enter       - select/expand node"
print_info "  l           - cycle layout algorithm"
print_info "  r           - refresh data"
print_info ""
print_info "The TUI is built with ratatui (Pure Rust)."
print_info "It runs on any terminal, no display server required."

record_pass "Multi-primal TUI documented"

echo
print_results || true
demo_complete "05-full-ecosystem"
