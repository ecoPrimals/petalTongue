#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
# Demo: Songbird Discovery
# Duration: ~30 seconds
# Prerequisites: petalTongue built
# External deps: Songbird (optional - graceful skip)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

print_header "Songbird Discovery Integration"
print_version_info

step 1 "Prerequisites"
require_petaltongue

step 2 "Check for Songbird"
print_info "Probing for Songbird discovery service..."
if check_songbird_running; then
    SONGBIRD_AVAILABLE=true
    record_pass "Songbird is running"
else
    SONGBIRD_AVAILABLE=false
    record_skip "Songbird not running"
    print_info "Songbird provides service discovery for the ecosystem."
    print_info "To run this demo fully, start Songbird first:"
    print_info ""
    print_command "cd ../songbird && cargo run --release"
    print_info ""
fi

step 3 "petalTongue registration"
if [[ "$SONGBIRD_AVAILABLE" == "true" ]]; then
    print_info "petalTongue auto-registers on startup via ipc.register"
    print_command "petaltongue status --format json"
    STATUS=$("${PETALTONGUE_BIN}" --log-level error status --format json 2>&1)
    if check_json_valid "$STATUS"; then
        record_pass "Status shows registration info"
    else
        record_fail "Could not get status"
    fi
else
    print_info "When Songbird runs, petalTongue automatically:"
    print_info "  1. Discovers the registration service (capability-based)"
    print_info "  2. Sends ipc.register with its capabilities"
    print_info "  3. Spawns a heartbeat task"
    print_info "  4. Appears in Songbird's service mesh"
    print_info ""
    print_info "petalTongue capabilities registered:"
    print_info "  - visualization.render"
    print_info "  - visualization.render.stream"
    print_info "  - visualization.export"
    print_info "  - visualization.validate"
    print_info "  - visualization.capabilities"
    print_info "  - visualization.interact"
    print_info "  - health.check"
    record_skip "Registration demo requires Songbird"
fi

step 4 "Discovery protocol reference"
print_info "Discovery flow (from wateringHole/PRIMAL_IPC_PROTOCOL.md):"
print_info "  1. Primal starts -> probes for discovery service"
print_info "  2. Finds socket via XDG_RUNTIME_DIR convention"
print_info "  3. Sends JSON-RPC: ipc.register {name, version, capabilities}"
print_info "  4. Discovery service responds with ecosystem state"
print_info "  5. Heartbeat maintains presence (configurable interval)"
print_info ""
print_info "No hardcoded primal names or ports - pure capability discovery."

echo
print_results || true
demo_complete "02-biomeos-topology"
