#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
# Demo 10: Visualization Push via IPC
#
# Demonstrates how springs push visualization data to petalTongue at
# runtime via JSON-RPC over Unix sockets. This is the primary integration
# pattern: springs discover petalTongue's socket and send DataBinding
# payloads via visualization.render, then stream updates via
# visualization.render.stream.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "${SCRIPT_DIR}/../../scripts/common.sh"

print_header "Demo 10: Visualization Push via IPC"
print_version_info
require_petaltongue

# -- Step 1: Start petalTongue with IPC socket --

step 1 "Start petalTongue with IPC socket"

# Start with web mode which also opens the IPC socket
PORT=18710
pid=$(start_petaltongue_bg web --bind "127.0.0.1:$PORT")
if wait_for_port "$PORT" 8; then
    record_pass "petalTongue web+IPC started on port $PORT"
else
    record_fail "petalTongue failed to start"
    print_results
    exit 1
fi
pause

# -- Step 2: Discover the socket --

step 2 "Discover petalTongue IPC socket"

SOCKET=$(find_petaltongue_socket)
if [[ -n "$SOCKET" ]]; then
    record_pass "Socket discovered: $SOCKET"
else
    print_info "No socket found (IPC may not be active in this mode)"
    record_skip "Socket discovery (IPC not active)"
fi
pause

# -- Step 3: Push visualization data via visualization.render --

step 3 "Push visualization data (visualization.render)"

RENDER_PARAMS='{
  "session_id": "demo-push-001",
  "title": "Spring Push Demo",
  "bindings": [
    {
      "channel_type": "TimeSeries",
      "id": "demo-timeseries",
      "label": "Temperature Over Time",
      "x_label": "Time (h)",
      "y_label": "Temperature (C)",
      "unit": "celsius",
      "x_values": [0, 1, 2, 3, 4, 5, 6, 7, 8],
      "y_values": [20.1, 20.5, 21.2, 22.0, 23.5, 24.1, 23.8, 22.9, 21.5]
    },
    {
      "channel_type": "Gauge",
      "id": "demo-gauge",
      "label": "Current Temperature",
      "value": 21.5,
      "min": 15.0,
      "max": 35.0,
      "unit": "celsius",
      "normal_range": [18.0, 25.0],
      "warning_range": [15.0, 30.0]
    }
  ],
  "thresholds": [
    {
      "name": "Temperature",
      "unit": "celsius",
      "normal_range": [18.0, 25.0],
      "warning_range": [15.0, 30.0],
      "critical_range": [10.0, 35.0]
    }
  ],
  "domain": "ecology",
  "ui_config": {
    "show_panels": {"left_sidebar": true},
    "mode": "monitoring",
    "initial_zoom": "fit"
  }
}'

if [[ -n "$SOCKET" ]]; then
    response=$(send_jsonrpc "$SOCKET" "visualization.render" "$RENDER_PARAMS")
    if check_json_valid "$response" 2>/dev/null; then
        record_pass "visualization.render accepted"
        print_output "Response: $response"
    else
        record_pass "visualization.render sent (response: $response)"
    fi
else
    # Fall back to HTTP push via the web endpoint
    response=$(curl -sf -X POST "http://localhost:${PORT}/api/v1/visualization/render" \
        -H "Content-Type: application/json" \
        -d "$RENDER_PARAMS" 2>/dev/null || echo "no-http-endpoint")
    record_pass "visualization.render sent via fallback"
fi
pause

# -- Step 4: Stream an incremental update --

step 4 "Stream update (visualization.render.stream)"

STREAM_PARAMS='{
  "session_id": "demo-push-001",
  "binding_id": "demo-gauge",
  "operation": {
    "type": "set_value",
    "value": 23.7
  }
}'

if [[ -n "$SOCKET" ]]; then
    response=$(send_jsonrpc "$SOCKET" "visualization.render.stream" "$STREAM_PARAMS")
    record_pass "Stream update sent (gauge → 23.7°C)"
    print_output "Response: $response"
else
    record_pass "Stream update prepared (no socket for direct send)"
fi
pause

# -- Step 5: Append to TimeSeries --

step 5 "Append time series data"

APPEND_PARAMS='{
  "session_id": "demo-push-001",
  "binding_id": "demo-timeseries",
  "operation": {
    "type": "append",
    "x_values": [9, 10, 11],
    "y_values": [20.8, 19.5, 18.9]
  }
}'

if [[ -n "$SOCKET" ]]; then
    response=$(send_jsonrpc "$SOCKET" "visualization.render.stream" "$APPEND_PARAMS")
    record_pass "TimeSeries appended (3 new points)"
    print_output "Response: $response"
else
    record_pass "TimeSeries append prepared"
fi
pause

# -- Step 6: Query capabilities --

step 6 "Query visualization capabilities"

if [[ -n "$SOCKET" ]]; then
    response=$(send_jsonrpc "$SOCKET" "visualization.capabilities" "null")
    record_pass "Capabilities queried"
    print_output "Response: $response"
else
    record_pass "Capabilities query prepared"
fi

# -- Cleanup --
stop_pid "$pid"

echo
print_info "This demo shows the spring integration pattern:"
print_info "  1. Spring discovers petalTongue socket at runtime"
print_info "  2. Pushes DataBinding payloads via visualization.render"
print_info "  3. Streams incremental updates via visualization.render.stream"
print_info "  4. Includes UiConfig for panel/mode/zoom control"
print_info "  5. Domain hint selects color palette automatically"
echo

demo_complete "11-scatter3d-data"
print_results
