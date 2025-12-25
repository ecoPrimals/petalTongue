#!/usr/bin/env bash
#
# Visual Modality: Layout Algorithms Demo
#
# Demonstrates all 4 layout algorithms on the same topology

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOMEOS_DIR="$(cd "$SCRIPT_DIR/../../../biomeOS" && pwd)"

echo -e "${BLUE}════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Layout Algorithms Demonstration${NC}"
echo -e "${BLUE}════════════════════════════════════════════${NC}"
echo ""

# Create a 5-node topology
echo -e "${YELLOW}📊 Setting up 5-node topology...${NC}"
echo ""

# Stop any existing primals
pkill -f "beardog\|nestgate\|songbird\|toadstool\|squirrel" 2>/dev/null || true
sleep 2

# Launch 5 primals (if available)
mkdir -p "$SCRIPT_DIR/logs"

primals=("beardog" "nestgate")
available_count=0

for primal in "${primals[@]}"; do
    PRIMAL_BIN="$BIOMEOS_DIR/bin/primals/$primal"
    if [ -f "$PRIMAL_BIN" ]; then
        echo -e "  ${GREEN}→${NC} Launching $primal..."
        "$PRIMAL_BIN" > "$SCRIPT_DIR/logs/${primal}.log" 2>&1 &
        ((available_count++))
        sleep 1
    fi
done

if [ $available_count -eq 0 ]; then
    echo -e "${YELLOW}⚠️  No primal binaries found${NC}"
    echo -e "${YELLOW}   This demo will show empty graphs (layouts still work!)${NC}"
fi

echo ""
echo -e "${GREEN}✅ Topology ready: $available_count primals${NC}"
echo ""
echo -e "${YELLOW}⏳ Waiting for BiomeOS discovery (5 seconds)...${NC}"
sleep 5

# Now demonstrate each layout
layouts=(
    "Force-Directed:Physics-based, organic positioning"
    "Hierarchical:Top-down tree structure"
    "Circular:Symmetrical circle arrangement"
    "Random:Baseline comparison"
)

echo ""
echo -e "${BLUE}════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Switching Layouts in petalTongue${NC}"
echo -e "${BLUE}════════════════════════════════════════════${NC}"
echo ""

for layout in "${layouts[@]}"; do
    name="${layout%%:*}"
    desc="${layout##*:}"
    
    echo -e "${GREEN}●${NC} ${YELLOW}$name Layout${NC}"
    echo -e "  $desc"
    echo ""
    echo -e "  ${BLUE}→ In petalTongue UI: Select '$name' from layout dropdown${NC}"
    echo -e "  ${BLUE}→ Observe how nodes reposition${NC}"
    echo -e "  ${BLUE}→ Notice the different spatial relationships${NC}"
    echo ""
    echo -e "${YELLOW}Press Enter when ready for next layout...${NC}"
    read -r
    echo ""
done

# Summary
echo -e "${BLUE}════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Layout Comparison${NC}"
echo -e "${BLUE}════════════════════════════════════════════${NC}"
echo ""
echo -e "${GREEN}✅ Force-Directed:${NC} Best for organic, relationship-based views"
echo -e "${GREEN}✅ Hierarchical:${NC} Best for clear parent-child structures"
echo -e "${GREEN}✅ Circular:${NC} Best for equal relationships, symmetry"
echo -e "${GREEN}✅ Random:${NC} Useful for testing, comparison baseline"
echo ""
echo -e "${YELLOW}🎯 Key Insight:${NC}"
echo -e "  The SAME data can be represented 4 different ways!"
echo -e "  Choose the layout that best shows the relationships YOU need to see."
echo ""

# Cleanup
echo -e "${YELLOW}Stopping primals...${NC}"
pkill -f "beardog\|nestgate\|songbird\|toadstool\|squirrel" 2>/dev/null || true
echo -e "${GREEN}✅ Demo complete!${NC}"
echo ""

