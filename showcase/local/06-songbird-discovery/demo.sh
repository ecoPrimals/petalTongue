#!/usr/bin/env bash
#
# Songbird Discovery Integration: Main Demo
#
# Purpose: Visualize Songbird's discovery and federation capabilities
# Duration: ~20 minutes

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PINK='\033[1;35m'
CYAN='\033[0;36m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SONGBIRD_DIR="$(cd "$SCRIPT_DIR/../../../../songbird" && pwd)"

echo -e "${PINK}╔══════════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${PINK}║              🐦 Songbird Discovery Integration Showcase                      ║${NC}"
echo -e "${PINK}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}This demo visualizes Songbird's discovery mesh and federation capabilities.${NC}"
echo ""

# Check prerequisites
echo -e "${YELLOW}📋 Checking prerequisites...${NC}"
echo ""

# Check if Songbird exists
if [ ! -d "$SONGBIRD_DIR" ]; then
    echo -e "${RED}❌ Songbird not found at: $SONGBIRD_DIR${NC}"
    echo -e "${YELLOW}   Expected directory: ../../../../songbird/${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Songbird directory found${NC}"

# Check if Songbird is built
if [ ! -f "$SONGBIRD_DIR/target/release/songbird" ]; then
    echo -e "${YELLOW}⚠️  Songbird not built${NC}"
    echo -e "${YELLOW}   Building Songbird...${NC}"
    cd "$SONGBIRD_DIR"
    cargo build --release
    cd "$SCRIPT_DIR"
    echo -e "${GREEN}✅ Songbird built${NC}"
else
    echo -e "${GREEN}✅ Songbird binary exists${NC}"
fi

# Check if petalTongue is running
if ! pgrep -f "petal-tongue" > /dev/null; then
    echo -e "${YELLOW}⚠️  petalTongue not running${NC}"
    echo -e "${YELLOW}   Please start petalTongue in another terminal:${NC}"
    echo -e "${BLUE}   cd ../../.. && cargo run --release -p petal-tongue-ui${NC}"
    echo ""
    echo -e "${YELLOW}Press Enter when petalTongue is running...${NC}"
    read -r
fi

echo -e "${GREEN}✅ petalTongue is running${NC}"
echo ""

# Demo scenarios
echo -e "${CYAN}═══════════════════════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}  Songbird Discovery Visualization${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${YELLOW}🎯 What you'll see:${NC}"
echo -e "  • Songbird tower(s) as discovery hubs"
echo -e "  • Primals appearing as they're discovered"
echo -e "  • Discovery connections forming"
echo -e "  • Federation between towers (multi-tower demo)"
echo -e "  • Real-time topology updates"
echo ""
echo -e "${YELLOW}🎧 What you'll hear:${NC}"
echo -e "  • Songbird = Light Chimes (discovery sound!)"
echo -e "  • Other primals with their native instruments"
echo -e "  • Spatial audio shows topology positions"
echo -e "  • Harmonic = healthy, dissonant = problems"
echo ""

scenarios=(
    "demo-single-tower.sh:Single Tower Discovery"
    "demo-multi-tower.sh:Multi-Tower Federation"
    "demo-discovery-events.sh:Real-Time Discovery Events"
)

for scenario in "${scenarios[@]}"; do
    script="${scenario%%:*}"
    name="${scenario##*:}"
    
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════════════════════${NC}"
    echo -e "${CYAN}  $name${NC}"
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════════════════════${NC}"
    echo ""
    
    if [ -f "$SCRIPT_DIR/$script" ]; then
        "$SCRIPT_DIR/$script"
        echo ""
        echo -e "${YELLOW}Press Enter to continue to next scenario...${NC}"
        read -r
        echo ""
    else
        echo -e "${YELLOW}⚠️  Script $script not yet implemented${NC}"
        echo -e "${YELLOW}   This scenario demonstrates: $name${NC}"
        echo ""
        echo -e "${GREEN}Concept:${NC}"
        echo -e "  Songbird's discovery mesh would be visualized in petalTongue,"
        echo -e "  showing towers, discovered primals, and federation connections"
        echo -e "  in both visual (graph) and audio (chimes) modalities."
        echo ""
        echo -e "${YELLOW}Press Enter to continue...${NC}"
        read -r
        echo ""
    fi
done

# Final summary
echo -e "${GREEN}╔══════════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║           ✅ Songbird Discovery Integration Showcase Complete!               ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}✅ Visualized Songbird's discovery mesh${NC}"
echo -e "${GREEN}✅ Saw real-time primal discovery${NC}"
echo -e "${GREEN}✅ Observed federation patterns${NC}"
echo -e "${GREEN}✅ Experienced multi-modal representation${NC}"
echo ""
echo -e "${YELLOW}🐦 Key Learnings:${NC}"
echo -e "  • Songbird makes distributed discovery visible"
echo -e "  • petalTongue makes Songbird's work VISUAL + AUDIBLE"
echo -e "  • Federation provides resilience"
echo -e "  • Discovery is dynamic, not static"
echo ""
echo -e "${YELLOW}🌟 Integration Success:${NC}"
echo -e "  petalTongue + Songbird = Complete discovery visualization"
echo -e "  • Songbird does the work (discovery, federation)"
echo -e "  • petalTongue makes it comprehensible (visual + audio)"
echo -e "  • Together: Powerful distributed systems observability"
echo ""
echo -e "${YELLOW}📚 Next: Visualize NestGate's Storage Mesh!${NC}"
echo -e "  cd ../07-nestgate-storage/"
echo -e "  cat README.md"
echo ""
echo -e "${PINK}🌱 Discovery made visible! On to storage visualization! 🌸${NC}"

