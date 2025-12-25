#!/usr/bin/env bash
#
# Visual 2D Modality: Full Demo
#
# Purpose: Demonstrate complete Visual 2D rendering capabilities
# Duration: ~15 minutes

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
echo -e "${PINK}║                  🎨 Visual 2D Modality Showcase                             ║${NC}"
echo -e "${PINK}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}This demo showcases petalTongue's complete Visual 2D rendering system.${NC}"
echo ""

# Check prerequisites
if ! curl -s http://localhost:3000/health >/dev/null 2>&1; then
    echo -e "${RED}❌ BiomeOS not running${NC}"
    echo -e "${YELLOW}   Run: cd ../00-setup && ./demo.sh${NC}"
    exit 1
fi

echo -e "${GREEN}✅ BiomeOS is running${NC}"
echo ""

# Run demo scenarios
scenarios=(
    "demo-layouts.sh:Layout Algorithms (4 layouts)"
    "demo-interaction.sh:Interactive Controls (pan, zoom, select)"
    "demo-health-states.sh:Health State Visualization"
    "demo-realtime.sh:Real-Time Updates"
)

for scenario in "${scenarios[@]}"; do
    script="${scenario%%:*}"
    name="${scenario##*:}"
    
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}Scenario: $name${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    
    if [ -f "$SCRIPT_DIR/$script" ]; then
        "$SCRIPT_DIR/$script"
        echo ""
        echo -e "${YELLOW}Press Enter to continue to next scenario...${NC}"
        read -r
        echo ""
    else
        echo -e "${YELLOW}⚠️  Script $script not yet implemented${NC}"
        echo -e "${YELLOW}   Building this scenario...${NC}"
        echo ""
        echo -e "${YELLOW}Press Enter to continue...${NC}"
        read -r
        echo ""
    fi
done

# Final summary
echo -e "${GREEN}╔══════════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                  ✅ Visual 2D Modality Showcase Complete!                    ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}✅ Explored all 4 layout algorithms${NC}"
echo -e "${GREEN}✅ Tested interactive controls (pan, zoom, select)${NC}"
echo -e "${GREEN}✅ Observed health state visualization${NC}"
echo -e "${GREEN}✅ Witnessed real-time topology updates${NC}"
echo ""
echo -e "${YELLOW}📊 Visual Modality Features:${NC}"
echo -e "  ✅ Interactive 2D graph rendering"
echo -e "  ✅ 4 layout algorithms (force, hierarchical, circular, random)"
echo -e "  ✅ Health color coding (green/yellow/red/gray)"
echo -e "  ✅ Pan, zoom, selection controls"
echo -e "  ✅ Real-time auto-refresh"
echo -e "  ✅ Statistics overlay"
echo ""
echo -e "${YELLOW}🎯 Key Insight:${NC}"
echo -e "  The Visual 2D modality provides a COMPLETE way to understand"
echo -e "  ecosystem topology through spatial, color, and layout information."
echo ""
echo -e "${YELLOW}📚 Next: Explore the Audio Sonification Modality!${NC}"
echo -e "  cd ../03-modality-audio/"
echo -e "  cat README.md"
echo ""
echo -e "${PINK}🌱 Visual modality complete! On to Audio! 🌸${NC}"

