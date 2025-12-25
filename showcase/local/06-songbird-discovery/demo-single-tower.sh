#!/usr/bin/env bash
#
# Songbird Discovery: Single Tower Demo
#
# Demonstrates single Songbird tower discovering primals

set -euo pipefail

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SONGBIRD_DIR="$(cd "$SCRIPT_DIR/../../../../songbird" && pwd)"

echo -e "${BLUE}════════════════════════════════════════════${NC}"
echo -e "${BLUE}  Single Tower Discovery Demonstration${NC}"
echo -e "${BLUE}════════════════════════════════════════════${NC}"
echo ""

mkdir -p "$SCRIPT_DIR/logs"

# Clean any existing processes
echo -e "${YELLOW}Cleaning up any existing processes...${NC}"
pkill -f "songbird" 2>/dev/null || true
sleep 2

# Launch Songbird tower
echo -e "${GREEN}→${NC} Launching Songbird tower..."
cd "$SONGBIRD_DIR"
RUST_LOG=info cargo run --release > "$SCRIPT_DIR/logs/songbird-tower.log" 2>&1 &
SONGBIRD_PID=$!
echo $SONGBIRD_PID > "$SCRIPT_DIR/logs/songbird.pid"

echo -e "${GREEN}✅${NC} Songbird tower started (PID: $SONGBIRD_PID)"
echo -e "   Logs: $SCRIPT_DIR/logs/songbird-tower.log"
echo ""

echo -e "${YELLOW}⏳ Waiting for Songbird to initialize (10 seconds)...${NC}"
sleep 10

echo -e "${GREEN}✅${NC} Songbird ready for discovery"
echo ""

# Instructions for user
echo -e "${BLUE}════════════════════════════════════════════${NC}"
echo -e "${BLUE}  petalTongue Visualization${NC}"
echo -e "${BLUE}════════════════════════════════════════════${NC}"
echo ""
echo -e "${YELLOW}👀 In petalTongue UI:${NC}"
echo -e "  • You should see a single node (Songbird tower)"
echo -e "  • Instrument: Light Chimes (discovery sound!)"
echo -e "  • Color: Green (healthy)"
echo -e "  • Position: Center of graph"
echo ""
echo -e "${YELLOW}🎧 Audio:${NC}"
echo -e "  • If audio is enabled, you'll hear light chimes"
echo -e "  • Harmonic tone = healthy discovery service"
echo ""
echo -e "${YELLOW}📊 Now let's add some primals for Songbird to discover...${NC}"
echo ""

# Launch some primals if available
BIOMEOS_DIR="$(cd "$SCRIPT_DIR/../../../biomeOS" 2>/dev/null && pwd)" || BIOMEOS_DIR=""

if [ -n "$BIOMEOS_DIR" ] && [ -d "$BIOMEOS_DIR/bin/primals" ]; then
    primals_found=0
    for primal in beardog nestgate; do
        PRIMAL_BIN="$BIOMEOS_DIR/bin/primals/$primal"
        if [ -f "$PRIMAL_BIN" ]; then
            echo -e "${GREEN}→${NC} Launching $primal for discovery..."
            "$PRIMAL_BIN" > "$SCRIPT_DIR/logs/${primal}.log" 2>&1 &
            echo $! > "$SCRIPT_DIR/logs/${primal}.pid"
            ((primals_found++))
            sleep 3
        fi
    done
    
    if [ $primals_found -gt 0 ]; then
        echo ""
        echo -e "${GREEN}✅${NC} Launched $primals_found primals"
        echo ""
        echo -e "${YELLOW}👀 Watch petalTongue:${NC}"
        echo -e "  • New nodes should appear as Songbird discovers them"
        echo -e "  • Edges connect Songbird to discovered primals"
        echo -e "  • Each primal has its own instrument (bass, strings, etc.)"
        echo -e "  • Layout adjusts to show discovery topology"
        echo ""
    fi
else
    echo -e "${YELLOW}⚠️  No primals found to launch${NC}"
    echo -e "${YELLOW}   (This is okay - you can see Songbird alone)${NC}"
    echo ""
fi

echo -e "${YELLOW}💡 Observation Points:${NC}"
echo -e "  1. Songbird is the hub (discovery service)"
echo -e "  2. Primals are discovered and connected"
echo -e "  3. Real-time updates show discovery events"
echo -e "  4. Both visual (graph) and audio (instruments) show same info"
echo ""
echo -e "${YELLOW}Press Enter when ready to stop this demo...${NC}"
read -r

# Cleanup
echo ""
echo -e "${YELLOW}Stopping services...${NC}"
pkill -f "songbird\|beardog\|nestgate" 2>/dev/null || true
echo -e "${GREEN}✅ Demo complete!${NC}"
echo ""

