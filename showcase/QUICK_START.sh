#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# petalTongue Showcase - Automated Tour
# Runs all local demos (01-local-primal, 02-ipc-protocol, 04-spring-integration) end-to-end.
# No external dependencies required.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/scripts/common.sh"

print_header "petalTongue Showcase - Automated Tour"
print_version_info

echo -e "${BOLD}This tour runs all local demos (~15 minutes).${NC}"
echo -e "${DIM}Set PAUSE_DURATION=0 for faster runs (CI mode).${NC}"
echo

require_petaltongue
echo

TOTAL_PASS=0
TOTAL_FAIL=0
TOTAL_SKIP=0

run_demo() {
    local dir=$1
    local name=$2
    local demo="${SCRIPT_DIR}/${dir}/demo.sh"

    if [[ ! -x "$demo" ]]; then
        print_skip "Not found: $demo"
        TOTAL_SKIP=$((TOTAL_SKIP + 1))
        return
    fi

    echo
    echo -e "${CYAN}--- ${name} ---${NC}"
    if PAUSE_DURATION=0 bash "$demo"; then
        TOTAL_PASS=$((TOTAL_PASS + 1))
        print_success "$name completed"
    else
        TOTAL_FAIL=$((TOTAL_FAIL + 1))
        print_fail "$name had failures (non-fatal)"
    fi
}

# 01-local-primal
run_demo "01-local-primal/00-hello-petaltongue" "Hello petalTongue"
run_demo "01-local-primal/01-unibin-modes" "UniBin Modes"
run_demo "01-local-primal/02-scenario-loading" "Scenario Loading"
run_demo "01-local-primal/03-web-server" "Web Server"
run_demo "01-local-primal/04-headless-api" "Headless API"
run_demo "01-local-primal/05-tui-dashboard" "TUI Dashboard"
run_demo "01-local-primal/06-audio-export" "Audio Export"
run_demo "01-local-primal/07-graph-layouts" "Graph Layouts"
run_demo "01-local-primal/08-clinical-data" "Clinical Data"
run_demo "01-local-primal/12-scene-graph" "Scene Graph"
run_demo "01-local-primal/13-grammar-compilation" "Grammar Compilation"
run_demo "01-local-primal/14-tufte-constraints" "Tufte Constraints"
run_demo "01-local-primal/15-math-objects" "Math Objects"
run_demo "01-local-primal/16-animation-system" "Animation System"
run_demo "01-local-primal/17-svg-modality" "SVG Modality"
run_demo "01-local-primal/18-physics-bridge" "Physics Bridge"

# 02-ipc-protocol
run_demo "02-ipc-protocol/01-unix-socket-server" "Unix Socket Server"
run_demo "02-ipc-protocol/02-jsonrpc-methods" "JSON-RPC Methods"
run_demo "02-ipc-protocol/03-health-monitoring" "Health Monitoring"

# 04-spring-integration
run_demo "04-spring-integration/03-scene-engine-pipeline" "Scene Engine Pipeline"

# Summary
echo
echo -e "${PURPLE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BOLD}${CYAN}  Showcase Tour Complete${NC}"
echo -e "${PURPLE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo
echo -e "  ${GREEN}Passed: ${TOTAL_PASS}${NC}  ${RED}Failed: ${TOTAL_FAIL}${NC}  ${YELLOW}Skipped: ${TOTAL_SKIP}${NC}"
echo
if [[ $TOTAL_FAIL -eq 0 ]]; then
    echo -e "  ${GREEN}All demos passed.${NC}"
else
    echo -e "  ${YELLOW}Some demos had failures. Review output above.${NC}"
fi
echo
echo -e "  ${DIM}Next: Run inter-primal demos with other primals running:${NC}"
echo -e "  ${DIM}  cd 03-inter-primal/01-songbird-discovery/ && ./demo.sh${NC}"
echo
