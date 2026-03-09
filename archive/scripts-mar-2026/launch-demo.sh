#!/bin/bash
# Launch Script for petalTongue Multi-Primal Demo
# Starts primals and petalTongue with proper configuration

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINS_DIR="$SCRIPT_DIR/../phase1bins"

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║  🌈 ecoPrimals Multi-Primal + Tool Ecosystem Demo 🌈     ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

# Check if binaries exist
if [ ! -d "$BINS_DIR" ]; then
    echo "❌ Phase1 bins directory not found at: $BINS_DIR"
    exit 1
fi

echo "📍 Binary directory: $BINS_DIR"
echo ""

# Configuration
export PETALTONGUE_MOCK_MODE="${PETALTONGUE_MOCK_MODE:-true}"
export BIOMEOS_URL="${BIOMEOS_URL:-http://localhost:3000}"
export SONGBIRD_URL="${SONGBIRD_URL:-http://localhost:8080}"
export TOADSTOOL_URL="${TOADSTOOL_URL:-http://localhost:4000}"

echo "🔧 Configuration:"
echo "  PETALTONGUE_MOCK_MODE=$PETALTONGUE_MOCK_MODE"
echo "  BIOMEOS_URL=$BIOMEOS_URL"
echo "  SONGBIRD_URL=$SONGBIRD_URL"
echo "  TOADSTOOL_URL=$TOADSTOOL_URL"
echo ""

# Function to launch a primal
launch_primal() {
    local name=$1
    local bin=$2
    local port=$3
    
    if [ ! -x "$BINS_DIR/$bin" ]; then
        echo "⚠️  $name binary not found or not executable: $bin"
        return 1
    fi
    
    echo "🚀 Launching $name on port $port..."
    cd "$BINS_DIR"
    ./$bin --port $port > "/tmp/$name.log" 2>&1 &
    local pid=$!
    echo "  ✅ $name started (PID: $pid)"
    sleep 1
    return 0
}

# Launch primals (optional - can run separately)
if [ "$LAUNCH_PRIMALS" = "true" ]; then
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🌟 Launching Primals..."
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Launch in order: discovery → compute → others
    launch_primal "Songbird" "songbird-bin" 8080 || true
    launch_primal "ToadStool" "toadstool-bin" 4000 || true
    launch_primal "NestGate" "nestgate-bin" 3002 || true
    launch_primal "BearDog" "beardog-bin" 3001 || true
    launch_primal "LoamSpine" "loamspine-bin" 3003 || true
    
    echo ""
    echo "⏳ Waiting 3 seconds for primals to initialize..."
    sleep 3
    echo ""
else
    echo "ℹ️  Skipping primal launch (set LAUNCH_PRIMALS=true to enable)"
    echo "   You can start primals manually from $BINS_DIR"
    echo ""
fi

# Build petalTongue if needed
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔨 Building petalTongue..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
cd "$SCRIPT_DIR"
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎨 Launching petalTongue UI..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📊 Available Tools:"
echo "  🎲 BingoCube         - Cryptographic verification"
echo "  📡 System Monitor    - CPU, memory, sparklines"
echo "  📋 Process Viewer    - Process list with filtering"
echo "  📈 Graph Metrics     - Real-time topology"
if [ "$PETALTONGUE_MOCK_MODE" = "false" ]; then
    echo "  🌟 [Real primals discovered via Songbird]"
fi
echo ""
echo "Press Ctrl+C to stop"
echo ""

# Launch petalTongue
cargo run --release --bin petal-tongue

# Cleanup on exit
trap 'echo ""; echo "🛑 Shutting down..."; pkill -f songbird-bin; pkill -f toadstool-bin; pkill -f nestgate-bin; pkill -f beardog-bin; pkill -f loamspine-bin; echo "✅ Cleanup complete"' EXIT

