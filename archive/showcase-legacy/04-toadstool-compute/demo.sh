#!/usr/bin/env bash
# 🍄 Toadstool Compute Demo
# Demonstrates petalTongue visualizing toadstool workloads

set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Configuration
BIOMEOS_URL="${BIOMEOS_URL:-http://localhost:3000}"
PETALTONGUE_BIN="${PETALTONGUE_BIN:-../../../target/release/petal-tongue}"
DEMO_DURATION="${DEMO_DURATION:-300}" # 5 minutes default

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🌸 petalTongue Showcase: Toadstool Compute"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check prerequisites
echo -e "${BLUE}[00:00]${NC} Checking prerequisites..."

# Check if petalTongue is built
if [ ! -f "$PETALTONGUE_BIN" ]; then
    echo -e "${RED}❌ petalTongue not built${NC}"
    echo ""
    echo "Build it first:"
    echo "  cd /path/to/petalTongue"
    echo "  cargo build --release"
    exit 1
fi

# Check if BiomeOS is running
echo -e "${BLUE}[00:01]${NC} Checking if BiomeOS is running..."
if ! curl -s -f "${BIOMEOS_URL}/api/v1/health" > /dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  BiomeOS not detected at ${BIOMEOS_URL}${NC}"
    echo ""
    echo "BiomeOS is required to aggregate primal discovery."
    echo "Start BiomeOS first, which will discover toadstool automatically."
    echo ""
    read -p "Press Enter once BiomeOS is running, or Ctrl+C to exit..."
    
    # Check again
    if ! curl -s -f "${BIOMEOS_URL}/api/v1/health" > /dev/null 2>&1; then
        echo -e "${RED}❌ Still can't reach BiomeOS${NC}"
        exit 1
    fi
fi

echo -e "${GREEN}✅ BiomeOS running at ${BIOMEOS_URL}${NC}"
echo ""

# Get BiomeOS info
echo -e "${BLUE}[00:02]${NC} Discovering compute primals..."
BIOMEOS_INFO=$(curl -s "${BIOMEOS_URL}/api/v1/health" 2>/dev/null || echo '{}')
echo "$BIOMEOS_INFO" | jq '.' 2>/dev/null || echo "  (Could not parse JSON)"
echo ""

# Check for active workloads
echo -e "${BLUE}[00:03]${NC} Checking for compute primals..."
PRIMALS=$(curl -s "${BIOMEOS_URL}/api/v1/primals" 2>/dev/null || echo '{"primals":[]}')
TOADSTOOL_FOUND=$(echo "$PRIMALS" | jq '.primals[] | select(.primal_type == "compute" or .name == "Toadstool" or .name == "toadstool" or (.capabilities | contains(["compute"])))' 2>/dev/null || echo "")

if [ -z "$TOADSTOOL_FOUND" ]; then
    echo -e "${YELLOW}⚠️  No compute primals found${NC}"
    echo ""
    echo "For better visualization, start toadstool:"
    echo "  cd /path/to/ecoPrimals/phase1/toadstool"
    echo "  cargo run --release &"
    echo ""
else
    echo -e "${GREEN}✅ Found compute primal(s)${NC}"
    echo "$TOADSTOOL_FOUND" | jq '.' 2>/dev/null || echo "  (Could not parse JSON)"
    echo ""
fi

# Explain what we're about to do
echo -e "${BLUE}[00:04]${NC} ${GREEN}What you'll see:${NC}"
echo "  📊 Visual: Compute mesh graph"
echo "  🎵 Audio: Drums=start, Strings=running, Chime=complete"
echo "  🎨 Colors: Green=idle, Yellow=busy, Blue=queued, Red=failed"
echo "  ⚡ Animation: Pulsing during execution"
echo ""

echo -e "${BLUE}[00:05]${NC} ${GREEN}What you can do:${NC}"
echo "  🖱️  Drag nodes to reposition"
echo "  🔍 Scroll to zoom in/out"
echo "  ⌨️  Press L to cycle layouts"
echo "  ⌨️  Press W to show workload details"
echo "  ⌨️  Press R to refresh (update workload status)"
echo "  ⌨️  Press A to toggle audio"
echo "  ⌨️  Press ESC to exit"
echo ""

echo -e "${BLUE}[00:06]${NC} ${GREEN}Pro Tip:${NC} Submit workloads in another terminal while demo runs!"
echo "  cd /path/to/ecoPrimals/phase1/toadstool/showcase/local-capabilities"
echo "  ./01-native-hello.sh"
echo ""

echo -e "${BLUE}[00:07]${NC} Starting petalTongue (will run for ${DEMO_DURATION}s)..."
echo ""

# Set up environment for petalTongue
export RUST_LOG="${RUST_LOG:-info}"
export BIOMEOS_URL="$BIOMEOS_URL"  # Point to BiomeOS aggregator
export AUTO_DISCOVER=1
export AUDIO_ENABLED=1
export CONTINUOUS_REFRESH="${CONTINUOUS_REFRESH:-0}"

# Run petalTongue
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}🌸 Launching petalTongue...${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Run with timeout
timeout "${DEMO_DURATION}s" "$PETALTONGUE_BIN" || true

echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ Demo complete!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Summary
echo -e "${BLUE}📊 Demo Summary:${NC}"
echo "  Duration: ${DEMO_DURATION}s"
echo "  BiomeOS URL: ${BIOMEOS_URL}"
echo "  Discovery: Automatic via BiomeOS aggregator"
echo "  Modalities: Visual + Audio"
echo ""

echo -e "${BLUE}🎓 What you learned:${NC}"
echo "  ✅ petalTongue discovers compute primals via BiomeOS"
echo "  ✅ Active workloads are visualized in real-time"
echo "  ✅ Execution state is sonified (drums/strings/chime)"
echo "  ✅ Resource usage is shown via colors/size"
echo "  ✅ Distributed compute mesh can be monitored"
echo ""

echo -e "${BLUE}🚀 What's next:${NC}"
echo "  Try: ../01-songbird-discovery/demo.sh (orchestration layer)"
echo "  Try: ../02-beardog-security/demo.sh (secure compute)"
echo "  Try: ../07-full-ecosystem/demo.sh (full stack)"
echo ""

echo -e "${BLUE}💡 Pro Tips:${NC}"
echo "  • Submit workloads during demo for live updates"
echo "  • Try GPU workloads for different visualization"
echo "  • Listen to audio spatial panning (CPU=left, GPU=right)"
echo "  • Check toadstool's showcase for more workload types"
echo ""

echo "Done! 🌸🍄"

