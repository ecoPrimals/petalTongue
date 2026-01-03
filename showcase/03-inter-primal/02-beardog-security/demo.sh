#!/usr/bin/env bash
# 🐻 BearDog Security Demo
# Demonstrates petalTongue visualizing BearDog security primal

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
echo "🌸 petalTongue Showcase: BearDog Security"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check prerequisites
echo -e "${BLUE}[00:00]${NC} Checking prerequisites..."

# Check if petalTongue is built
if [ ! -f "$PETALTONGUE_BIN" ]; then
    echo -e "${RED}❌ petalTongue not built${NC}"
    echo ""
    echo "Build it first:"
    echo "  cd /home/eastgate/Development/ecoPrimals/phase2/petalTongue"
    echo "  cargo build --release"
    exit 1
fi

# Check if BiomeOS is running
echo -e "${BLUE}[00:01]${NC} Checking if BiomeOS is running..."
if ! curl -s -f "${BIOMEOS_URL}/api/v1/health" > /dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  BiomeOS not detected at ${BIOMEOS_URL}${NC}"
    echo ""
    echo "BiomeOS is required to aggregate primal discovery."
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

# Look for security primals
echo -e "${BLUE}[00:03]${NC} Looking for security primals..."
PRIMALS=$(curl -s "${BIOMEOS_URL}/api/v1/primals" 2>/dev/null || echo '{"primals":[]}')
BEARDOG_FOUND=$(echo "$PRIMALS" | jq '.primals[] | select(.primal_type == "security" or .name == "BearDog" or .name == "beardog" or (.capabilities | contains(["btsp"])))' 2>/dev/null || echo "")

if [ -z "$BEARDOG_FOUND" ]; then
    echo -e "${YELLOW}⚠️  No security primals found${NC}"
    echo ""
    echo "BearDog should be discovered by BiomeOS."
    echo "Make sure BearDog is running:"
    echo "  cd /home/eastgate/Development/ecoPrimals/primalBins"
    echo "  ./beardog &"
    echo ""
    echo "Demo will continue, but visualization will be empty."
    echo ""
    read -p "Press Enter to continue anyway, or Ctrl+C to exit..."
else
    echo -e "${GREEN}✅ Found security primal(s)${NC}"
    echo "$BEARDOG_FOUND" | jq '.' 2>/dev/null | head -20
    echo ""
    
    # Check trust level
    TRUST_LEVEL=$(echo "$BEARDOG_FOUND" | jq -r '.trust_level // "null"' 2>/dev/null || echo "null")
    FAMILY_ID=$(echo "$BEARDOG_FOUND" | jq -r '.family_id // "null"' 2>/dev/null || echo "null")
    
    echo -e "${BLUE}Security Primal Details:${NC}"
    echo "  Trust Level: $TRUST_LEVEL (0=none, 1=limited, 2=elevated, 3=full)"
    echo "  Family ID: $FAMILY_ID"
    echo ""
fi

# Explain what we're about to do
echo -e "${BLUE}[00:04]${NC} ${GREEN}What you'll see:${NC}"
echo "  📊 Visual: BearDog node with trust-level colors"
echo "  🎵 Audio: Chime for full trust, lower notes for lower trust"
echo "  🎨 Colors: Green=full, Orange=elevated, Yellow=limited, Gray=none"
echo "  ⚡ Badge: Trust level indicator on node"
echo "  💍 Ring: Family ID color (if multiple primals in same family)"
echo ""

echo -e "${BLUE}[00:05]${NC} ${GREEN}What you can do:${NC}"
echo "  🖱️  Click node to see capabilities"
echo "  🔍 Scroll to zoom in/out"
echo "  ⌨️  Press L to cycle layouts"
echo "  ⌨️  Press C to show capabilities panel"
echo "  ⌨️  Press T to highlight trust relationships"
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
echo "  Modalities: Visual + Audio"
echo ""

echo -e "${BLUE}🎓 What you learned:${NC}"
echo "  ✅ petalTongue visualizes security primals via BiomeOS"
echo "  ✅ Trust levels are color-coded intuitively"
echo "  ✅ Capabilities are discoverable at runtime"
echo "  ✅ Family grouping helps identify relationships"
echo "  ✅ Audio provides trust-level feedback"
echo ""

echo -e "${BLUE}🚀 What's next:${NC}"
echo "  Try: ../04-toadstool-compute/demo.sh (compute mesh)"
echo "  Try: ../07-full-ecosystem/demo.sh (all primals together)"
echo "  Read: /phase1/beardog/docs/TRUST_FRAMEWORK.md"
echo ""

echo -e "${BLUE}💡 Pro Tips:${NC}"
echo "  • Higher trust = brighter color + higher audio pitch"
echo "  • Family ring helps identify related primals"
echo "  • Capabilities panel shows what BearDog can do"
echo "  • Trust elevation will be interactive in Phase 4"
echo ""

echo "Done! 🌸🐻"

