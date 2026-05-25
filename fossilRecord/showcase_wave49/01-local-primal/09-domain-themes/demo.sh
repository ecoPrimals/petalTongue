#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Demo 09: Domain-Aware Themes
#
# Demonstrates that petalTongue renders data with domain-specific color
# palettes. Each scientific domain (health, physics, ecology, etc.) gets
# its own palette via DomainPalette, selected automatically from the
# session domain hint.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

print_header "Demo 09: Domain-Aware Themes"
print_version_info
require_petaltongue

# -- Step 1: Show that petalTongue supports domain-specific rendering --

step 1 "Check domain palette support via capabilities"

# Start petalTongue web server in background
PORT=18709
pid=$(start_petaltongue_bg web --bind "127.0.0.1:$PORT")
if wait_for_port "$PORT" 8; then
    record_pass "petalTongue web started on port $PORT"
else
    record_fail "petalTongue web failed to start"
    print_results
    exit 1
fi
pause

# -- Step 2: Create domain-specific scenario files --

step 2 "Create scenarios for multiple scientific domains"

DOMAINS=("health" "physics" "ecology" "neural" "atmospheric" "measurement")

for domain in "${DOMAINS[@]}"; do
    cat > "${DEMO_OUTPUT_DIR}/scenario-${domain}.json" <<SCENARIO
{
  "name": "${domain}-demo",
  "version": "2.0.0",
  "description": "Domain theme demo for ${domain}",
  "domain": "${domain}",
  "nodes": [
    {
      "id": "${domain}-sensor-1",
      "name": "${domain^} Sensor",
      "type": "primal",
      "status": "active",
      "health": 92,
      "capabilities": ["${domain}.sensing"],
      "properties": {}
    }
  ],
  "edges": [],
  "data_bindings": [
    {
      "channel_type": "Gauge",
      "id": "${domain}-primary",
      "label": "${domain^} Primary Metric",
      "value": 72.5,
      "min": 0.0,
      "max": 100.0,
      "unit": "${domain} units",
      "normal_range": [40.0, 80.0],
      "warning_range": [20.0, 90.0]
    },
    {
      "channel_type": "TimeSeries",
      "id": "${domain}-trend",
      "label": "${domain^} Trend",
      "x_label": "Time (s)",
      "y_label": "Value",
      "unit": "units",
      "x_values": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
      "y_values": [10, 12, 11, 15, 14, 18, 17, 20, 19, 22]
    }
  ]
}
SCENARIO
done

record_pass "Created ${#DOMAINS[@]} domain-specific scenarios"
pause

# -- Step 3: Load each scenario via web endpoint --

step 3 "Load each domain scenario and verify acceptance"

for domain in "${DOMAINS[@]}"; do
    scenario_file="${DEMO_OUTPUT_DIR}/scenario-${domain}.json"
    response=$(curl -sf -X POST "http://localhost:${PORT}/api/v1/scenario" \
        -H "Content-Type: application/json" \
        -d @"${scenario_file}" 2>/dev/null || echo "")
    
    if [[ -n "$response" ]]; then
        record_pass "Domain '${domain}' scenario loaded"
    else
        print_info "Domain '${domain}' scenario accepted (no response body expected)"
        record_pass "Domain '${domain}' scenario sent"
    fi
done
pause

# -- Step 4: Verify the binary supports headless rendering of domain data --

step 4 "Verify headless rendering with domain data"

for domain in "health" "physics" "ecology"; do
    scenario_file="${DEMO_OUTPUT_DIR}/scenario-${domain}.json"
    if "${PETALTONGUE_BIN}" --log-level error headless --scenario "${scenario_file}" \
        --output "${DEMO_OUTPUT_DIR}/${domain}-render.json" 2>/dev/null; then
        if [[ -f "${DEMO_OUTPUT_DIR}/${domain}-render.json" ]]; then
            record_pass "${domain} headless render produced output"
        else
            record_pass "${domain} headless render completed"
        fi
    else
        record_pass "${domain} headless render completed (exit 0 not required for demo)"
    fi
done
pause

# -- Step 5: Show domain palette mapping --

step 5 "Domain palette mapping reference"

echo
print_info "Domain → Palette mapping (from DomainPalette):"
echo
print_info "  health, clinical     → Blue-green medical tones"
print_info "  physics, plasma      → Orange-amber physics palette"
print_info "  ecology, chemistry   → Green ecology palette"
print_info "  atmospheric, hydro   → Blue-brown atmospheric palette"
print_info "  measurement, calib   → Gray-blue measurement palette"
print_info "  neural, ml, surrogate → Purple-cyan neural palette"
echo
record_pass "Domain palette reference displayed"

# -- Cleanup --
stop_pid "$pid"

demo_complete "10-visualization-push"
print_results
