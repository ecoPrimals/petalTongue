#!/usr/bin/env bash
# 🌍 Full Ecosystem Demo
# Demonstrates petalTongue visualizing the complete ecoPrimals ecosystem

set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
BIOMEOS_URL="${BIOMEOS_URL:-http://localhost:3000}"
PETALTONGUE_BIN="${PETALTONGUE_BIN:-../../../target/release/petal-tongue}"
DEMO_DURATION="${DEMO_DURATION:-600}" # 10 minutes default for full ecosystem

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🌸 petalTongue Showcase: Full Ecosystem"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo -e "${CYAN}🌍 Complete ecoPrimals ecosystem visualization${NC}"
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
    echo -e "${RED}❌ BiomeOS not detected at ${BIOMEOS_URL}${NC}"
    echo ""
    echo "BiomeOS is required to aggregate ecosystem discovery."
    exit 1
fi

echo -e "${GREEN}✅ BiomeOS running at ${BIOMEOS_URL}${NC}"
echo ""

# Get ecosystem state
echo -e "${BLUE}[00:02]${NC} Discovering ecosystem..."
HEALTH=$(curl -s "${BIOMEOS_URL}/api/v1/health" 2>/dev/null || echo '{}')
echo "$HEALTH" | jq '.' 2>/dev/null || echo "  (Could not parse JSON)"
echo ""

# Get all primals
echo -e "${BLUE}[00:03]${NC} Found primals in ecosystem:"
PRIMALS=$(curl -s "${BIOMEOS_URL}/api/v1/primals" 2>/dev/null || echo '{"primals":[], "count":0}')
PRIMAL_COUNT=$(echo "$PRIMALS" | jq '.count // 0' 2>/dev/null || echo "0")

if [ "$PRIMAL_COUNT" -eq 0 ]; then
    echo -e "${RED}❌ No primals discovered!${NC}"
    echo ""
    echo "The ecosystem is empty. Start some primals:"
    echo "  cd /home/eastgate/Development/ecoPrimals/primalBins"
    echo "  ./songbird-orchestrator &"
    echo "  ./beardog &"
    echo ""
    exit 1
elif [ "$PRIMAL_COUNT" -eq 1 ]; then
    echo -e "${YELLOW}⚠️  Only 1 primal found${NC}"
    echo ""
    echo "For better ecosystem visualization, start more primals."
    echo "Demo will continue with limited ecosystem."
    echo ""
fi

# List primals by type
echo "$PRIMALS" | jq -r '.primals[] | "  ✅ \(.id) (\(.primal_type)) - \(.health) health"' 2>/dev/null
echo -e "  ${GREEN}Total: $PRIMAL_COUNT primals${NC}"
echo ""

# Analyze by type
echo -e "${BLUE}[00:04]${NC} Analyzing topology..."
TOPOLOGY=$(curl -s "${BIOMEOS_URL}/api/v1/topology" 2>/dev/null || echo '{"nodes":[], "edges":[]}')
NODE_COUNT=$(echo "$TOPOLOGY" | jq '.nodes | length' 2>/dev/null || echo "0")
EDGE_COUNT=$(echo "$TOPOLOGY" | jq '.edges | length' 2>/dev/null || echo "0")

echo "  Nodes: $NODE_COUNT"
echo "  Edges: $EDGE_COUNT"

if [ "$EDGE_COUNT" -gt 0 ]; then
    echo "  Relationships:"
    echo "$TOPOLOGY" | jq -r '.edges[] | "    • \(.from) → \(.to) (\(.edge_type))"' 2>/dev/null | head -10
    if [ "$EDGE_COUNT" -gt 10 ]; then
        echo "    ... and $((EDGE_COUNT - 10)) more"
    fi
fi
echo ""

# Ecosystem summary
echo -e "${BLUE}[00:05]${NC} 🌟 ${GREEN}Ecosystem Summary:${NC}"

# Count by type
ORCHESTRATION=$(echo "$PRIMALS" | jq '[.primals[] | select(.primal_type == "orchestration")] | length' 2>/dev/null || echo "0")
SECURITY=$(echo "$PRIMALS" | jq '[.primals[] | select(.primal_type == "security")] | length' 2>/dev/null || echo "0")
COMPUTE=$(echo "$PRIMALS" | jq '[.primals[] | select(.primal_type == "compute")] | length' 2>/dev/null || echo "0")
STORAGE=$(echo "$PRIMALS" | jq '[.primals[] | select(.primal_type == "storage")] | length' 2>/dev/null || echo "0")

echo "  Orchestration: $ORCHESTRATION primal(s)"
echo "  Security: $SECURITY primal(s)"
echo "  Compute: $COMPUTE primal(s)"
echo "  Storage: $STORAGE primal(s)"
echo ""

# Trust levels
TRUST_FULL=$(echo "$PRIMALS" | jq '[.primals[] | select(.trust_level == 3)] | length' 2>/dev/null || echo "0")
TRUST_ELEVATED=$(echo "$PRIMALS" | jq '[.primals[] | select(.trust_level == 2)] | length' 2>/dev/null || echo "0")
TRUST_LIMITED=$(echo "$PRIMALS" | jq '[.primals[] | select(.trust_level == 1)] | length' 2>/dev/null || echo "0")
TRUST_NONE=$(echo "$PRIMALS" | jq '[.primals[] | select(.trust_level == 0)] | length' 2>/dev/null || echo "0")

echo "  Trust Levels:"
if [ "$TRUST_FULL" -gt 0 ]; then echo "    Full (3): $TRUST_FULL primal(s)"; fi
if [ "$TRUST_ELEVATED" -gt 0 ]; then echo "    Elevated (2): $TRUST_ELEVATED primal(s)"; fi
if [ "$TRUST_LIMITED" -gt 0 ]; then echo "    Limited (1): $TRUST_LIMITED primal(s)"; fi
if [ "$TRUST_NONE" -gt 0 ]; then echo "    None (0): $TRUST_NONE primal(s)"; fi
echo ""

# Explain what we're about to do
echo -e "${BLUE}[00:06]${NC} ${GREEN}What you'll see:${NC}"
echo "  📊 Visual: Complete ecosystem topology graph"
echo "  🎵 Audio: Orchestra mode (multiple instruments)"
echo "  🎨 Colors: Health-based (green/yellow/red/gray)"
echo "  ⚡ Edges: All inter-primal relationships"
echo "  💍 Rings: Trust families grouped"
echo "  🔍 Zoom: Explore $PRIMAL_COUNT primals, $EDGE_COUNT connections"
echo ""

echo -e "${BLUE}[00:07]${NC} ${GREEN}What you can do:${NC}"
echo "  🖱️  Drag any node to reposition"
echo "  🔍 Scroll to zoom in/out"
echo "  ⌨️  Press L to cycle layouts (find best view)"
echo "  ⌨️  Press F to filter by primal type"
echo "  ⌨️  Press T to highlight trust relationships"
echo "  ⌨️  Press C to show capabilities panel"
echo "  ⌨️  Press D to show dashboard"
echo "  ⌨️  Press A to toggle audio (orchestra mode)"
echo "  ⌨️  Press R to refresh (see new primals)"
echo "  ⌨️  Press ESC to exit"
echo ""

echo -e "${CYAN}💡 Pro Tip: Try adding/removing primals while demo runs!${NC}"
echo ""

echo -e "${BLUE}[00:08]${NC} Starting petalTongue (will run for ${DEMO_DURATION}s)..."
echo ""

# Set up environment for petalTongue
export RUST_LOG="${RUST_LOG:-info}"
export BIOMEOS_URL="$BIOMEOS_URL"
export AUTO_DISCOVER=1
export AUDIO_ENABLED=1
export SHOW_DASHBOARD=1

# Run petalTongue
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}🌸 Launching petalTongue - Full Ecosystem View${NC}"
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
echo "  Primals visualized: $PRIMAL_COUNT"
echo "  Connections: $EDGE_COUNT"
echo "  Modalities: Visual + Audio (Orchestra)"
echo ""

echo -e "${BLUE}🎓 What you learned:${NC}"
echo "  ✅ Complete ecosystem in single view"
echo "  ✅ Zero hardcoding - all discovered"
echo "  ✅ Real-time topology updates"
echo "  ✅ Multi-modal awareness (visual + audio)"
echo "  ✅ TRUE PRIMAL architecture proven"
echo ""

echo -e "${BLUE}🌟 Key Insights:${NC}"
echo "  • Hub nodes = orchestrators (high connectivity)"
echo "  • Leaf nodes = specialized services"
echo "  • Trust families = related primals (same ring)"
echo "  • Capability clusters = functional grouping"
echo ""

echo -e "${BLUE}🚀 What's next:${NC}"
echo "  Try: Phase 4 accessibility demos (universal design)"
echo "  Try: Phase 5 production scenarios (ops patterns)"
echo "  Deploy: Use petalTongue for your production ecosystem"
echo ""

echo -e "${BLUE}💡 Advanced Experiments:${NC}"
echo "  • Add a primal while running (watch it appear)"
echo "  • Kill a primal (watch health change)"
echo "  • Compare layouts (L key - which reveals most?)"
echo "  • Filter by type (F key - focus on one layer)"
echo ""

echo "🌸🌍 Ecosystem visualized! ${PRIMAL_COUNT} primals, $EDGE_COUNT connections, TRUE PRIMAL proven! 🚀"

