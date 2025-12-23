#!/usr/bin/env bash
#
# 02-primal-discovery: Real-Time Primal Discovery Demo
#
# Purpose: Watch primals being discovered as they start/stop
# Duration: ~10-15 minutes
# Dependencies: 00-setup complete, BiomeOS running, petalTongue open

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PINK='\033[1;35m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo -e "${PINK}╔══════════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${PINK}║                  🌸 Real-Time Primal Discovery Demo                         ║${NC}"
echo -e "${PINK}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}This demo shows how petalTongue discovers primals in real-time.${NC}"
echo -e "${BLUE}Watch the graph as primals appear and disappear!${NC}"
echo ""

# Check prerequisites
if ! curl -s http://localhost:3000/health >/dev/null 2>&1; then
    echo -e "${RED}❌ BiomeOS not running${NC}"
    echo -e "${YELLOW}   Run: cd ../00-setup && ./demo.sh${NC}"
    exit 1
fi

echo -e "${GREEN}✅ BiomeOS is running${NC}"
echo ""

# Start with clean slate
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Phase 1: Sequential Discovery (Adding Primals)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Clean slate
echo -e "${YELLOW}🧹 Ensuring clean slate (no primals running)...${NC}"
"$SCRIPT_DIR/remove-all-primals.sh"
sleep 2

echo -e "${GREEN}✅ Starting with empty graph${NC}"
echo -e "${YELLOW}   Check petalTongue: Should show 0 nodes, 0 edges${NC}"
echo ""
echo -e "${YELLOW}Press Enter to add first primal...${NC}"
read -r

# Add primals one by one
primals=("beardog" "nestgate")

for i in "${!primals[@]}"; do
    primal="${primals[$i]}"
    node_count=$((i + 1))
    
    echo -e "${BLUE}──────────────────────────────────────────────────────────────────────${NC}"
    echo -e "${BLUE}Step $((i + 1)): Adding $primal${NC}"
    echo -e "${BLUE}──────────────────────────────────────────────────────────────────────${NC}"
    echo ""
    
    "$SCRIPT_DIR/add-primal.sh" "$primal"
    
    echo ""
    echo -e "${YELLOW}⏳ Waiting for discovery (5-10 seconds)...${NC}"
    echo -e "${YELLOW}   Watch petalTongue - new node should appear!${NC}"
    
    # Countdown
    for j in {10..1}; do
        echo -ne "${YELLOW}   $j...${NC}\r"
        sleep 1
    done
    echo ""
    
    echo -e "${GREEN}✅ Discovery window complete${NC}"
    echo -e "${YELLOW}   Expected: $node_count node(s) visible${NC}"
    echo ""
    
    if [ $i -lt $((${#primals[@]} - 1)) ]; then
        echo -e "${YELLOW}Press Enter to add next primal...${NC}"
        read -r
    fi
done

echo ""
echo -e "${GREEN}✅ All primals added!${NC}"
echo -e "${YELLOW}   Check petalTongue: Should show ${#primals[@]} nodes${NC}"
echo ""
echo -e "${YELLOW}Press Enter to proceed to removal phase...${NC}"
read -r

# Remove primals
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Phase 2: Primal Disappearance (Removing Primals)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

for i in "${!primals[@]}"; do
    primal="${primals[$i]}"
    remaining=$((${#primals[@]} - i - 1))
    
    echo -e "${BLUE}──────────────────────────────────────────────────────────────────────${NC}"
    echo -e "${BLUE}Removing $primal${NC}"
    echo -e "${BLUE}──────────────────────────────────────────────────────────────────────${NC}"
    echo ""
    
    "$SCRIPT_DIR/remove-primal.sh" "$primal"
    
    echo ""
    echo -e "${YELLOW}⏳ Waiting for removal to be detected...${NC}"
    echo -e "${YELLOW}   Watch petalTongue - node should disappear!${NC}"
    
    # Countdown
    for j in {8..1}; do
        echo -ne "${YELLOW}   $j...${NC}\r"
        sleep 1
    done
    echo ""
    
    echo -e "${GREEN}✅ Removal window complete${NC}"
    echo -e "${YELLOW}   Expected: $remaining node(s) remaining${NC}"
    echo ""
    
    if [ $remaining -gt 0 ]; then
        echo -e "${YELLOW}Press Enter to remove next primal...${NC}"
        read -r
    fi
done

# Final summary
echo ""
echo -e "${GREEN}╔══════════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                     ✅ Discovery Demo Complete!                              ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}✅ You've seen real-time primal discovery${NC}"
echo -e "${GREEN}✅ You understand discovery timing (5-10s latency)${NC}"
echo -e "${GREEN}✅ You've validated add/remove both work${NC}"
echo ""
echo -e "${YELLOW}📖 Key Learnings:${NC}"
echo -e "  • Discovery is not instant (pipeline: primal → BiomeOS → petalTongue)"
echo -e "  • Auto-refresh interval is 5 seconds"
echo -e "  • Manual 'Refresh Now' button provides immediate update"
echo -e "  • Graph handles dynamic changes gracefully"
echo ""
echo -e "${YELLOW}🔍 Did you notice any gaps?${NC}"
echo -e "  • Was discovery timing acceptable?"
echo -e "  • Did layout update smoothly?"
echo -e "  • Any visual glitches?"
echo -e "  • Document in: ../GAPS.md"
echo ""
echo -e "${YELLOW}📚 Next Steps:${NC}"
echo -e "  cd ../03-topology-visualization/"
echo -e "  cat README.md"
echo ""
echo -e "${PINK}🌱 Fermentation continues! On to full topology! 🌸${NC}"

