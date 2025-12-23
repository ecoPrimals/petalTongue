#!/usr/bin/env bash
#
# 01-single-primal: Single Primal Visualization Demo
#
# Purpose: Learn to visualize individual primals
# Duration: ~5-10 minutes
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
echo -e "${PINK}║                  🌸 Single Primal Visualization Demo                        ║${NC}"
echo -e "${PINK}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}This demo shows each primal type individually.${NC}"
echo -e "${BLUE}You'll see one node at a time in petalTongue.${NC}"
echo ""

# Check prerequisites
if ! curl -s http://localhost:3000/health >/dev/null 2>&1; then
    echo -e "${RED}❌ BiomeOS not running${NC}"
    echo -e "${YELLOW}   Run: cd ../00-setup && ./demo.sh${NC}"
    exit 1
fi

echo -e "${GREEN}✅ BiomeOS is running${NC}"
echo ""

# Demo each primal type
primals=("beardog" "nestgate")

for primal in "${primals[@]}"; do
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}Demonstrating: $primal${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    
    if [ -f "$SCRIPT_DIR/${primal}-only.sh" ]; then
        echo -e "${YELLOW}⏩ Launching $primal...${NC}"
        "$SCRIPT_DIR/${primal}-only.sh"
        
        echo ""
        echo -e "${YELLOW}📊 Check petalTongue UI:${NC}"
        echo -e "   • You should see: ${GREEN}1 node, 0 edges${NC}"
        echo -e "   • Node color: ${GREEN}Green (healthy)${NC}"
        echo -e "   • Right panel: Audio description for $primal"
        echo ""
        echo -e "${YELLOW}Press Enter to stop $primal and continue...${NC}"
        read -r
        
        # Stop this primal
        "$SCRIPT_DIR/stop-all-primals.sh" >/dev/null 2>&1
        
        echo -e "${GREEN}✅ $primal stopped${NC}"
        echo ""
        sleep 2
    else
        echo -e "${YELLOW}⚠️  Script ${primal}-only.sh not found, skipping${NC}"
        echo ""
    fi
done

# Final summary
echo -e "${GREEN}╔══════════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                     ✅ Single Primal Demo Complete!                          ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}✅ You've seen individual primals visualized${NC}"
echo -e "${GREEN}✅ You understand single node rendering${NC}"
echo -e "${GREEN}✅ You've observed audio descriptions${NC}"
echo ""
echo -e "${YELLOW}📖 Key Learnings:${NC}"
echo -e "  • Single nodes are centered by default"
echo -e "  • Each primal type has unique audio mapping"
echo -e "  • Health states are color-coded (green = healthy)"
echo -e "  • Auto-refresh happens every 5 seconds"
echo ""
echo -e "${YELLOW}📚 Next Steps:${NC}"
echo -e "  cd ../02-primal-discovery/"
echo -e "  cat README.md"
echo ""
echo -e "${PINK}🌱 Fermentation continues! On to real-time discovery! 🌸${NC}"

