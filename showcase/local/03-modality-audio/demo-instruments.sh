#!/usr/bin/env bash
#
# Audio Modality: Instrument Mapping Demo
#
# Demonstrates the 5 instrument types mapped to primal roles

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
echo -e "${BLUE}  Instrument Mapping Demonstration${NC}"
echo -e "${BLUE}════════════════════════════════════════════${NC}"
echo ""

# Stop any existing primals
pkill -f "beardog\|nestgate\|songbird\|toadstool\|squirrel" 2>/dev/null || true
sleep 2

mkdir -p "$SCRIPT_DIR/logs"

# Demonstrate each instrument type
instruments=(
    "beardog:Security:Deep Bass:Foundation, low-level, solid"
    "nestgate:Storage:Sustained Strings:Persistence, holding, continuous"
)

for instrument in "${instruments[@]}"; do
    IFS=':' read -r primal role instrument_name why <<< "$instrument"
    
    echo -e "${GREEN}●${NC} ${YELLOW}$role Primal: $primal${NC}"
    echo -e "  Instrument: ${BLUE}$instrument_name${NC}"
    echo -e "  Why: $why"
    echo ""
    
    PRIMAL_BIN="$BIOMEOS_DIR/bin/primals/$primal"
    if [ -f "$PRIMAL_BIN" ]; then
        echo -e "  ${BLUE}→${NC} Launching $primal..."
        "$PRIMAL_BIN" > "$SCRIPT_DIR/logs/${primal}.log" 2>&1 &
        PRIMAL_PID=$!
        
        echo -e "  ${GREEN}✅${NC} Running (PID: $PRIMAL_PID)"
        echo ""
        echo -e "  ${YELLOW}🎧 In petalTongue:${NC}"
        echo -e "     • Enable audio (check audio toggle)"
        echo -e "     • You should hear: ${BLUE}$instrument_name${NC}"
        echo -e "     • Check the audio panel for description"
        echo ""
        echo -e "${YELLOW}Press Enter to hear the next instrument...${NC}"
        read -r
        
        # Stop this primal
        kill $PRIMAL_PID 2>/dev/null || true
        echo -e "  ${GREEN}→${NC} Stopped $primal"
        echo ""
        sleep 2
    else
        echo -e "  ${YELLOW}⚠️${NC} Binary not found: $PRIMAL_BIN"
        echo -e "  ${YELLOW}   You would hear: ${BLUE}$instrument_name${NC}"
        echo ""
        echo -e "${YELLOW}Press Enter to continue...${NC}"
        read -r
        echo ""
    fi
done

# Summary
echo -e "${BLUE}════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Instrument Summary${NC}"
echo -e "${BLUE}════════════════════════════════════════════${NC}"
echo ""
echo -e "${GREEN}🐻 BearDog (Security)${NC}      → Deep Bass"
echo -e "${GREEN}🍄 ToadStool (Compute)${NC}     → Rhythmic Drums"
echo -e "${GREEN}🐦 Songbird (Discovery)${NC}   → Light Chimes"
echo -e "${GREEN}🏠 NestGate (Storage)${NC}      → Sustained Strings"
echo -e "${GREEN}🐿️  Squirrel (AI)${NC}          → High Synth"
echo ""
echo -e "${YELLOW}🎯 Key Insight:${NC}"
echo -e "  Each primal TYPE has a unique audio signature!"
echo -e "  You can identify what type of service is running just by listening."
echo ""
echo -e "${GREEN}✅ Instrument mapping complete!${NC}"
echo ""

