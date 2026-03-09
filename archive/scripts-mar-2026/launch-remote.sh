#!/usr/bin/env bash
#
# Launch script for petalTongue with remote desktop visibility fix
#
# ISSUE: egui/eframe windows are created unmapped in headless+remote setups
# WORKAROUND: Manually map the window after launch
#
# This script:
# 1. Kills any existing petalTongue instances
# 2. Launches petalTongue
# 3. Waits for window creation
# 4. Maps the window (makes it visible)
# 5. Activates the window (brings to front)

set -e

echo "🌸 Launching petalTongue for remote viewing..."

# Kill existing instances
echo "🧹 Cleaning up existing instances..."
pkill petal-tongue 2>/dev/null || true
sleep 1

# Launch petalTongue in background
echo "🚀 Starting petalTongue..."
cd "$(dirname "$0")"
DISPLAY=${DISPLAY:-:1} ./target/release/petal-tongue > /tmp/petaltongue.log 2>&1 &
PETAL_PID=$!

echo "   PID: $PETAL_PID"
echo "   Display: ${DISPLAY:-:1}"

# Wait for window to be created
echo "⏳ Waiting for window creation..."
for i in {1..10}; do
    sleep 1
    if DISPLAY=${DISPLAY:-:1} xdotool search --name "petalTongue" > /dev/null 2>&1; then
        echo "✅ Window detected!"
        break
    fi
    if [ $i -eq 10 ]; then
        echo "❌ Window not created after 10 seconds"
        exit 1
    fi
done

# Map and activate the window
echo "🗺️  Mapping window (making visible)..."
WINDOW_ID=$(DISPLAY=${DISPLAY:-:1} xdotool search --name "petalTongue" | head -1)
echo "   Window ID: $WINDOW_ID"

DISPLAY=${DISPLAY:-:1} xdotool windowmap "$WINDOW_ID"
sleep 0.5
DISPLAY=${DISPLAY:-:1} xdotool windowactivate "$WINDOW_ID"

# Verify
MAP_STATE=$(DISPLAY=${DISPLAY:-:1} xwininfo -id "$WINDOW_ID" 2>/dev/null | grep "Map State" | awk '{print $3}')
echo "   Map State: $MAP_STATE"

if [ "$MAP_STATE" = "IsViewable" ]; then
    echo ""
    echo "✅ SUCCESS! petalTongue is now visible"
    echo ""
    echo "📊 You should now see:"
    echo "   - Graph visualization with nodes and edges"
    echo "   - Left sidebar with system metrics"
    echo "   - 🧠 SAME DAVE Proprioception section"
    echo ""
    echo "🧪 TEST: Click anywhere in the window"
    echo "   Watch the proprioception metrics update:"
    echo "   - Health should jump to ~95%"
    echo "   - Confidence should increase"
    echo "   - Status should turn green (✅)"
    echo ""
    echo "📝 Logs: /tmp/petaltongue.log"
    echo "🛑 Stop: pkill petal-tongue"
else
    echo "⚠️  Warning: Window may not be fully visible"
    echo "   Map State: $MAP_STATE"
    exit 1
fi

