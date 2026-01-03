#!/usr/bin/env bash
# 🎵 Songbird Discovery Demo
# Demonstrates petalTongue visualizing songbird federation

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
echo "🌸 petalTongue Showcase: Songbird Discovery"
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
    echo "Start BiomeOS first, which will discover songbird automatically."
    echo ""
    echo "Expected BiomeOS at: ${BIOMEOS_URL}"
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
echo -e "${BLUE}[00:02]${NC} Checking BiomeOS health..."
BIOMEOS_INFO=$(curl -s "${BIOMEOS_URL}/api/v1/health" 2>/dev/null || echo '{}')
echo "$BIOMEOS_INFO" | jq '.' 2>/dev/null || echo "  (Could not parse JSON)"
echo ""

# Check for discovered primals
echo -e "${BLUE}[00:03]${NC} Checking for discovered primals..."
PRIMALS=$(curl -s "${BIOMEOS_URL}/api/v1/primals" 2>/dev/null || echo '{"primals":[]}')
PRIMAL_COUNT=$(echo "$PRIMALS" | jq '.primals | length' 2>/dev/null || echo "0")

if [ "$PRIMAL_COUNT" -eq 0 ]; then
    echo -e "${YELLOW}⚠️  No primals discovered yet${NC}"
    echo ""
    echo "BiomeOS hasn't discovered any primals yet."
    echo "Make sure songbird is running and BiomeOS can discover it."
    echo ""
    echo "Expected: songbird running on port 8080"
    echo ""
else
    echo -e "${GREEN}✅ Found $PRIMAL_COUNT primal(s) discovered by BiomeOS${NC}"
    
    # Check if songbird is among them
    SONGBIRD_FOUND=$(echo "$PRIMALS" | jq '.primals[] | select(.primal_type == "orchestration" or .name == "Songbird" or .name == "songbird")' 2>/dev/null || echo "")
    if [ -n "$SONGBIRD_FOUND" ]; then
        echo -e "${GREEN}✅ Songbird discovered!${NC}"
        echo "$SONGBIRD_FOUND" | jq '.' 2>/dev/null | head -15
    else
        echo -e "${YELLOW}⚠️  Songbird not yet discovered${NC}"
        echo "BiomeOS found primals, but songbird isn't among them yet."
        echo "This demo works with any discovered primals, continuing..."
    fi
    echo ""
fi

# Explain what we're about to do
echo -e "${BLUE}[00:04]${NC} ${GREEN}What you'll see:${NC}"
echo "  📊 Visual: Graph of ecosystem topology (via BiomeOS)"
echo "  🎵 Audio: Piano for discovery, strings for connections"
echo "  🎨 Colors: Green=healthy, Yellow=warning, Gray=unknown"
echo "  ⚡ Animation: Flow particles showing data flow"
echo ""

echo -e "${BLUE}[00:05]${NC} ${GREEN}What you can do:${NC}"
echo "  🖱️  Drag nodes to reposition"
echo "  🔍 Scroll to zoom in/out"
echo "  ⌨️  Press L to cycle layouts (Force/Circular/Hierarchical/Grid)"
echo "  ⌨️  Press C to show capabilities panel"
echo "  ⌨️  Press A to toggle audio"
echo "  ⌨️  Press R to refresh discovery"
echo "  ⌨️  Press ESC to exit"
echo ""

echo -e "${BLUE}[00:06]${NC} Starting petalTongue (will run for ${DEMO_DURATION}s)..."
echo ""

# Set up environment for petalTongue
export RUST_LOG="${RUST_LOG:-info}"
export BIOMEOS_URL="$BIOMEOS_URL"  # Point to BiomeOS aggregator
export AUTO_DISCOVER=1
export AUDIO_ENABLED=1

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
echo "  Primals shown: $PRIMAL_COUNT"
echo "  Modalities: Visual + Audio"
echo ""

echo -e "${BLUE}🎓 What you learned:${NC}"
echo "  ✅ petalTongue discovers primals via BiomeOS aggregator"
echo "  ✅ BiomeOS aggregates discovery from all primals"
echo "  ✅ Songbird/beardog/etc are shown in single topology"
echo "  ✅ Protocol escalation is visualized"
echo "  ✅ Real-time updates work"
echo ""

echo -e "${BLUE}🚀 What's next:${NC}"
echo "  Try: ../02-beardog-security/demo.sh (key lineage visualization)"
echo "  Try: ../04-toadstool-compute/demo.sh (compute mesh)"
echo "  Try: ../07-full-ecosystem/demo.sh (all primals together)"
echo ""

echo -e "${BLUE}💡 Pro Tips:${NC}"
echo "  • Add more towers for richer visualization"
echo "  • Try different layouts (press L)"
echo "  • Listen to audio patterns (protocol escalation)"
echo "  • Check songbird's showcase for federation setup"
echo ""

echo "Done! 🌸🎵"

