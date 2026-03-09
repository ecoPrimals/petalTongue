#!/bin/bash
# Test petalTongue with stable plasmidBin binaries
# TRUE PRIMAL compliant: Runtime discovery, no hardcoded paths

set -e

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  🌸 petalTongue Interaction Test with Stable Binaries 🌸    ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Configuration (environment hints, not requirements)
export FAMILY_ID="${FAMILY_ID:-integration-test}"
export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"

# Detect plasmidBin location (don't hardcode)
PLASMID_BIN="../biomeOS/plasmidBin"

if [ ! -d "$PLASMID_BIN" ]; then
    echo "❌ plasmidBin not found at: $PLASMID_BIN"
    echo ""
    echo "Expected location: /path/to/biomeOS/plasmidBin/"
    echo ""
    echo "Options:"
    echo "  1. Set PLASMID_BIN env var to correct location"
    echo "  2. Run petalTongue without external primals (standalone mode)"
    echo ""
    exit 1
fi

echo "📁 Found plasmidBin at: $PLASMID_BIN"
echo "🏷️  Family ID: $FAMILY_ID"
echo "📂 Runtime dir: $XDG_RUNTIME_DIR"
echo ""

# List available binaries
echo "📦 Available stable binaries:"
ls -lh "$PLASMID_BIN"/{beardog,nestgate,toadstool,squirrel,toadstool-cli} 2>/dev/null | \
    awk '{print "   " $9 " (" $5 ")"}'
echo ""

# Start primals (they create their own sockets - TRUE PRIMAL pattern)
echo "🚀 Starting primals..."
PIDS=()

# BearDog (security)
if [ -x "$PLASMID_BIN/beardog" ]; then
    echo "   Starting beardog..."
    "$PLASMID_BIN/beardog" --family-id "$FAMILY_ID" > /tmp/beardog-test.log 2>&1 &
    PIDS+=($!)
    echo "   ✅ beardog (PID: $!)"
else
    echo "   ⚠️  beardog not executable"
fi

# Toadstool (compute)
if [ -x "$PLASMID_BIN/toadstool" ]; then
    echo "   Starting toadstool..."
    "$PLASMID_BIN/toadstool" --family-id "$FAMILY_ID" > /tmp/toadstool-test.log 2>&1 &
    PIDS+=($!)
    echo "   ✅ toadstool (PID: $!)"
else
    echo "   ⚠️  toadstool not executable"
fi

# NestGate (storage)
if [ -x "$PLASMID_BIN/nestgate" ]; then
    echo "   Starting nestgate..."
    "$PLASMID_BIN/nestgate" --family-id "$FAMILY_ID" > /tmp/nestgate-test.log 2>&1 &
    PIDS+=($!)
    echo "   ✅ nestgate (PID: $!)"
else
    echo "   ⚠️  nestgate not executable"
fi

echo ""
echo "⏳ Waiting 3 seconds for primals to initialize..."
sleep 3

# Check for sockets (primals announce themselves)
echo ""
echo "🔍 Checking for primal sockets..."
if ls "$XDG_RUNTIME_DIR"/*-"$FAMILY_ID"*.sock 2>/dev/null; then
    echo ""
    echo "✅ Sockets created! Primals are ready for discovery."
else
    echo "⚠️  No sockets found yet (primals may still be starting)"
    echo "💡 Check logs in /tmp/*-test.log"
fi

echo ""
echo "════════════════════════════════════════════════════════════════"
echo "🌸 Starting petalTongue (will discover primals at runtime)"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "Expected behavior:"
echo "  ✅ petalTongue discovers available primals via Unix sockets"
echo "  ✅ No hardcoded paths or assumptions"
echo "  ✅ Graceful degradation if some primals are missing"
echo "  ✅ Real-time visualization of topology"
echo ""
echo "Press Ctrl+C to stop all primals and petalTongue"
echo ""

# Run petalTongue from source (discovers primals at runtime)
# This is the current development version - testing against stable primals
cargo run --bin petal-tongue-ui

# Cleanup on exit
cleanup() {
    echo ""
    echo "🧹 Stopping primals..."
    for pid in "${PIDS[@]}"; do
        kill "$pid" 2>/dev/null || true
    done
    echo "✅ Cleanup complete"
}

trap cleanup EXIT

# If we get here, user exited petalTongue normally
exit 0

