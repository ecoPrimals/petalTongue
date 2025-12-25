#!/usr/bin/env bash
#
# Audio Sonification Modality: Full Demo
#
# Purpose: Demonstrate complete Audio Sonification capabilities
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
echo -e "${PINK}║                  🎵 Audio Sonification Modality Showcase                     ║${NC}"
echo -e "${PINK}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}This demo showcases petalTongue's revolutionary Audio Sonification system.${NC}"
echo ""
echo -e "${YELLOW}🌟 REVOLUTIONARY CAPABILITY:${NC}"
echo -e "   A blind user can understand the SAME ecosystem topology through audio"
echo -e "   that a sighted user understands visually!"
echo ""

# Check prerequisites
if ! curl -s http://localhost:3000/health >/dev/null 2>&1; then
    echo -e "${RED}❌ BiomeOS not running${NC}"
    echo -e "${YELLOW}   Run: cd ../00-setup && ./demo.sh${NC}"
    exit 1
fi

echo -e "${GREEN}✅ BiomeOS is running${NC}"
echo ""

echo -e "${YELLOW}📢 IMPORTANT: Turn on your speakers or headphones!${NC}"
echo -e "${YELLOW}   Audio is essential for this demo.${NC}"
echo ""
echo -e "${YELLOW}Press Enter when audio is ready...${NC}"
read -r
echo ""

# Run demo scenarios
scenarios=(
    "demo-instruments.sh:Instrument Mapping (5 instrument types)"
    "demo-health-audio.sh:Health State Mapping (harmonic/off-key/dissonant)"
    "demo-spatial-audio.sh:Spatial Audio (stereo panning)"
    "demo-narration.sh:AI Narration (natural language descriptions)"
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
        echo -e "${YELLOW}   This scenario is planned for future development.${NC}"
        echo ""
        echo -e "${YELLOW}Press Enter to continue...${NC}"
        read -r
        echo ""
    fi
done

# Final summary
echo -e "${GREEN}╔══════════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║              ✅ Audio Sonification Modality Showcase Complete!                ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}✅ Explored 5 instrument types (primal role mapping)${NC}"
echo -e "${GREEN}✅ Heard health state pitch mapping (harmonic/off-key/dissonant)${NC}"
echo -e "${GREEN}✅ Experienced spatial audio (stereo panning)${NC}"
echo -e "${GREEN}✅ Listened to AI narration${NC}"
echo ""
echo -e "${YELLOW}🎵 Audio Modality Features:${NC}"
echo -e "  ✅ 5 unique instruments (BearDog=Bass, ToadStool=Drums, Songbird=Chimes, etc.)"
echo -e "  ✅ Health pitch mapping (3 states + degraded)"
echo -e "  ✅ Spatial audio positioning (stereo panning)"
echo -e "  ✅ Activity volume modulation"
echo -e "  ✅ Master volume control"
echo -e "  ✅ AI soundscape narration"
echo ""
echo -e "${YELLOW}🌟 Revolutionary Impact:${NC}"
echo -e "  ${GREEN}→${NC} Blind users can operate systems through audio alone"
echo -e "  ${GREEN}→${NC} Sighted users gain redundant information channels"
echo -e "  ${GREEN}→${NC} Same information, different sensory channel"
echo -e "  ${GREEN}→${NC} Universal design in action!"
echo ""
echo -e "${YELLOW}🎯 Key Insight:${NC}"
echo -e "  Audio is NOT just a \"nice to have\" accessibility feature."
echo -e "  It's a COMPLETE, EQUIVALENT representation of the ecosystem."
echo ""
echo -e "${YELLOW}📚 Next: Experience Both Modalities Simultaneously!${NC}"
echo -e "  cd ../04-dual-modality/"
echo -e "  cat README.md"
echo ""
echo -e "${PINK}🌱 Audio modality complete! On to Dual-Modality! 🌸${NC}"

