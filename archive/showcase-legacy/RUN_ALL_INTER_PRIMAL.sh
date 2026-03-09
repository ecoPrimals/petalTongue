#!/usr/bin/env bash
# 🌸 Run All Inter-Primal Showcases
# Demonstrates petalTongue with all ecosystem primals

set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🌸 petalTongue Showcase: Inter-Primal Integration (Phase 3)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${BLUE}Phase 3 showcases petalTongue visualizing other primals:${NC}"
echo "  01. 🎵 Songbird Discovery (15 min)"
echo "  02. 🐻 BearDog Security (10 min)"
echo "  03. 🏡 NestGate Storage (10 min)"
echo "  04. 🍄 Toadstool Compute (15 min)"
echo "  05. 🌾 LoamSpine Permanence (10 min) - Future"
echo "  06. 🍄‍🟫 RhizoCrypt DAG (10 min) - Future"
echo "  07. 🌍 Full Ecosystem (20 min)"
echo ""
echo "Total time: ~90 minutes (current: ~50 min ready)"
echo ""

# Check which demos are ready
READY_DEMOS=()
PLANNED_DEMOS=()

if [ -f "01-songbird-discovery/demo.sh" ]; then
    READY_DEMOS+=("01-songbird-discovery")
else
    PLANNED_DEMOS+=("01-songbird-discovery")
fi

if [ -f "02-beardog-security/demo.sh" ]; then
    READY_DEMOS+=("02-beardog-security")
else
    PLANNED_DEMOS+=("02-beardog-security")
fi

if [ -f "03-nestgate-storage/demo.sh" ]; then
    READY_DEMOS+=("03-nestgate-storage")
else
    PLANNED_DEMOS+=("03-nestgate-storage")
fi

if [ -f "04-toadstool-compute/demo.sh" ]; then
    READY_DEMOS+=("04-toadstool-compute")
else
    PLANNED_DEMOS+=("04-toadstool-compute")
fi

if [ -f "07-full-ecosystem/demo.sh" ]; then
    READY_DEMOS+=("07-full-ecosystem")
else
    PLANNED_DEMOS+=("07-full-ecosystem")
fi

echo -e "${GREEN}✅ Ready demos (${#READY_DEMOS[@]}):${NC}"
for demo in "${READY_DEMOS[@]}"; do
    echo "  - $demo"
done
echo ""

if [ ${#PLANNED_DEMOS[@]} -gt 0 ]; then
    echo -e "${YELLOW}📋 Planned demos (${#PLANNED_DEMOS[@]}):${NC}"
    for demo in "${PLANNED_DEMOS[@]}"; do
        echo "  - $demo"
    done
    echo ""
fi

# Check if user wants to run all or select
echo -e "${BLUE}Run options:${NC}"
echo "  [a] Run ALL ready demos (${#READY_DEMOS[@]} demos)"
echo "  [1] Run individual demo"
echo "  [q] Quit"
echo ""
read -p "Choose [a/1/q]: " choice

case $choice in
    a|A)
        echo ""
        echo -e "${GREEN}Running all ready demos...${NC}"
        echo ""
        
        for demo in "${READY_DEMOS[@]}"; do
            echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
            echo -e "${BLUE}Starting: $demo${NC}"
            echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
            echo ""
            
            cd "$demo"
            ./demo.sh || {
                echo -e "${RED}❌ Demo failed: $demo${NC}"
                echo ""
                read -p "Continue to next demo? [y/n]: " continue_choice
                if [ "$continue_choice" != "y" ]; then
                    exit 1
                fi
            }
            cd ..
            
            echo ""
            echo -e "${GREEN}✅ Completed: $demo${NC}"
            echo ""
            
            if [ "$demo" != "${READY_DEMOS[-1]}" ]; then
                read -p "Press Enter to continue to next demo..."
            fi
        done
        
        echo ""
        echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${GREEN}✅ All demos complete!${NC}"
        echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        ;;
    
    1)
        echo ""
        echo -e "${BLUE}Available demos:${NC}"
        for i in "${!READY_DEMOS[@]}"; do
            echo "  [$i] ${READY_DEMOS[$i]}"
        done
        echo ""
        read -p "Choose demo number: " demo_num
        
        if [ "$demo_num" -ge 0 ] && [ "$demo_num" -lt "${#READY_DEMOS[@]}" ]; then
            demo="${READY_DEMOS[$demo_num]}"
            echo ""
            echo -e "${GREEN}Running: $demo${NC}"
            echo ""
            cd "$demo"
            ./demo.sh
        else
            echo -e "${RED}Invalid demo number${NC}"
            exit 1
        fi
        ;;
    
    q|Q)
        echo "Exiting."
        exit 0
        ;;
    
    *)
        echo -e "${RED}Invalid choice${NC}"
        exit 1
        ;;
esac

echo ""
echo -e "${BLUE}🎓 What you learned in Phase 3:${NC}"
echo "  ✅ petalTongue visualizes any primal automatically"
echo "  ✅ Federation, security, storage, compute all visible"
echo "  ✅ Multi-modal throughout (visual + audio)"
echo "  ✅ Real-time monitoring works"
echo "  ✅ Ecosystem coordination is transparent"
echo ""

echo -e "${BLUE}🚀 What's next:${NC}"
echo "  Try: ../04-accessibility/ (universal design validation)"
echo "  Try: ../05-production-scenarios/ (real-world patterns)"
echo "  Read: ../../README.md (complete documentation)"
echo ""

echo "Done! 🌸🚀"

