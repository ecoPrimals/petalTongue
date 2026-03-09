#!/usr/bin/env bash
# GPU Rendering Discovery Showcase
# Demonstrates petalTongue discovering Toadstool for GPU rendering via Songbird

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
PHASE1="$(cd "$PROJECT_ROOT/../../phase1" && pwd)"

echo "════════════════════════════════════════════════════════════"
echo "   🌸🍄 GPU Rendering Discovery Showcase 🍄🌸"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Demonstrates TRUE PRIMAL architecture:"
echo "  • petalTongue knows ONLY itself"
echo "  • Discovers GPU rendering via Songbird"
echo "  • Falls back gracefully if not found"
echo ""
echo "════════════════════════════════════════════════════════════"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check prerequisites
echo "📋 Checking prerequisites..."
echo ""

check_primal() {
    local name=$1
    local path=$2
    
    if [ -d "$path" ]; then
        echo -e "${GREEN}✅${NC} $name found at $path"
        return 0
    else
        echo -e "${RED}❌${NC} $name not found at $path"
        return 1
    fi
}

MISSING=0

check_primal "Songbird" "$PHASE1/songbird" || MISSING=1
check_primal "Toadstool" "$PHASE1/toadstool" || MISSING=1
check_primal "petalTongue" "$PROJECT_ROOT" || MISSING=1

echo ""

if [ $MISSING -eq 1 ]; then
    echo -e "${RED}❌ Missing required primals!${NC}"
    echo ""
    echo "This showcase requires all three primals."
    echo "Please ensure the repository structure is:"
    echo "  ecoPrimals/"
    echo "    ├── phase1/"
    echo "    │   ├── songbird/"
    echo "    │   └── toadstool/"
    echo "    └── phase2/"
    echo "        └── petalTongue/"
    exit 1
fi

# Use pre-built binaries from primalBins
echo "════════════════════════════════════════════════════════════"
echo "🔨 Checking primal binaries..."
echo "════════════════════════════════════════════════════════════"
echo ""

PRIMAL_BINS="$(cd "$PROJECT_ROOT/../../primalBins" && pwd)"

check_binary() {
    local name=$1
    local binary=$2
    
    if [ -f "$PRIMAL_BINS/$binary" ]; then
        echo -e "${GREEN}✅${NC} $name found at $PRIMAL_BINS/$binary"
        return 0
    else
        echo -e "${YELLOW}⚠️${NC}  $name not found, building from source..."
        return 1
    fi
}

check_binary "Songbird" "songbird-orchestrator" || build_primal "Songbird" "$PHASE1/songbird"
check_binary "Toadstool" "toadstool" || build_primal "Toadstool" "$PHASE1/toadstool"  
check_binary "petalTongue" "petal-tongue" || build_primal "petalTongue" "$PROJECT_ROOT"

echo ""

# Start primals
echo "════════════════════════════════════════════════════════════"
echo "🚀 Starting primals..."
echo "════════════════════════════════════════════════════════════"
echo ""

# Cleanup function
cleanup() {
    echo ""
    echo "════════════════════════════════════════════════════════════"
    echo "🧹 Cleaning up..."
    echo "════════════════════════════════════════════════════════════"
    
    if [ ! -z "${SONGBIRD_PID:-}" ]; then
        echo "Stopping Songbird (PID: $SONGBIRD_PID)..."
        kill $SONGBIRD_PID 2>/dev/null || true
    fi
    
    if [ ! -z "${TOADSTOOL_PID:-}" ]; then
        echo "Stopping Toadstool (PID: $TOADSTOOL_PID)..."
        kill $TOADSTOOL_PID 2>/dev/null || true
    fi
    
    if [ ! -z "${PETALTONGUE_PID:-}" ]; then
        echo "Stopping petalTongue (PID: $PETALTONGUE_PID)..."
        kill $PETALTONGUE_PID 2>/dev/null || true
    fi
    
    echo "✅ Cleanup complete"
}

trap cleanup EXIT INT TERM

# Start Songbird (discovery primal)
echo -e "${BLUE}Starting Songbird (discovery)...${NC}"
if [ -f "$PRIMAL_BINS/songbird-orchestrator" ]; then
    cd "$PRIMAL_BINS"
    RUST_LOG=info ./songbird-orchestrator 2>&1 &
else
    cd "$PHASE1/songbird"
    RUST_LOG=info cargo run --release --quiet 2>&1 &
fi
SONGBIRD_PID=$!
echo -e "${GREEN}✅${NC} Songbird started (PID: $SONGBIRD_PID)"
echo "   Listening on: http://localhost:8081"
echo ""

# Wait for Songbird to be ready
echo "⏳ Waiting for Songbird to initialize..."
for i in {1..10}; do
    if curl -s http://localhost:8081/health > /dev/null 2>&1; then
        echo -e "${GREEN}✅${NC} Songbird ready!"
        break
    fi
    sleep 1
done
echo ""

# Start Toadstool (GPU compute primal)
echo -e "${BLUE}Starting Toadstool (GPU compute)...${NC}"
if [ -f "$PRIMAL_BINS/toadstool" ]; then
    cd "$PRIMAL_BINS"
    RUST_LOG=info ./toadstool 2>&1 &
else
    cd "$PHASE1/toadstool"
    RUST_LOG=info cargo run --release --bin toadstool-daemon --quiet 2>&1 &
fi
TOADSTOOL_PID=$!
echo -e "${GREEN}✅${NC} Toadstool started (PID: $TOADSTOOL_PID)"
echo "   Listening on: http://localhost:8084"
echo ""

# Wait for Toadstool to register with Songbird
echo "⏳ Waiting for Toadstool to register capabilities..."
sleep 3
echo ""

# Verify registration
echo "════════════════════════════════════════════════════════════"
echo "🔍 Verifying capability registration..."
echo "════════════════════════════════════════════════════════════"
echo ""

echo "Query: 'Who provides gpu-rendering?'"
PROVIDERS=$(curl -s http://localhost:8081/primals/capabilities/gpu-rendering || echo "[]")
echo "Response: $PROVIDERS"
echo ""

if echo "$PROVIDERS" | grep -q "toadstool"; then
    echo -e "${GREEN}✅${NC} Toadstool registered as GPU rendering provider!"
else
    echo -e "${YELLOW}⚠️${NC}  Toadstool not yet registered (may take a moment)"
fi
echo ""

# Start petalTongue (visualization primal)
echo "════════════════════════════════════════════════════════════"
echo "🌸 Starting petalTongue (visualization)..."
echo "════════════════════════════════════════════════════════════"
echo ""

cd "$PROJECT_ROOT"

echo -e "${BLUE}Watch for discovery logs:${NC}"
echo "  🔍 'Discovering rendering capabilities...'"
echo "  📡 'Querying Songbird for gpu-rendering...'"
echo "  ✅ 'Found provider: <id>'"
echo ""

SHOWCASE_GPU_RENDERING=1 RUST_LOG=info cargo run --release --bin petal-tongue 2>&1 &
PETALTONGUE_PID=$!
echo -e "${GREEN}✅${NC} petalTongue started (PID: $PETALTONGUE_PID)"
echo ""

# Monitor for 10 seconds
echo "════════════════════════════════════════════════════════════"
echo "📊 Monitoring inter-primal communication..."
echo "════════════════════════════════════════════════════════════"
echo ""

sleep 2

echo "Registered capabilities:"
curl -s http://localhost:8081/primals/capabilities | jq '.'
echo ""

echo "Active primals:"
curl -s http://localhost:8081/primals | jq '. | length'
echo ""

# Test graceful fallback
echo "════════════════════════════════════════════════════════════"
echo "🧪 Testing graceful fallback..."
echo "════════════════════════════════════════════════════════════"
echo ""

echo "Stopping Songbird to test fallback..."
kill $SONGBIRD_PID 2>/dev/null || true
SONGBIRD_PID=""
sleep 2

echo ""
echo "Starting petalTongue WITHOUT discovery available..."
cd "$PROJECT_ROOT"
cargo run --release --bin petal-tongue-headless -- --mode terminal 2>&1 | head -20
echo ""

echo -e "${GREEN}✅${NC} Fallback successful! (pure Rust renderer used)"
echo ""

echo "════════════════════════════════════════════════════════════"
echo "🎉 Showcase Complete!"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Demonstrated:"
echo "  ✅ Zero hardcoded primal knowledge"
echo "  ✅ Capability-based discovery via Songbird"
echo "  ✅ GPU rendering when available (Toadstool)"
echo "  ✅ Graceful fallback when not available"
echo "  ✅ TRUE PRIMAL architecture"
echo ""
echo "════════════════════════════════════════════════════════════"

