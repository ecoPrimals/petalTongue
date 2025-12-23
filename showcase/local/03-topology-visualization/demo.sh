#!/usr/bin/env bash
#
# 03-topology-visualization: Full Ecosystem Topology Demo
#
# Purpose: Visualize complete multi-primal topology with layout comparison
# Duration: ~15-20 minutes
# Dependencies: 00-setup complete, BiomeOS running, petalTongue open

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

echo -e "${PINK}╔══════════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${PINK}║               🌸 Full Ecosystem Topology Visualization                       ║${NC}"
echo -e "${PINK}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}This demo shows how petalTongue visualizes complex multi-primal topologies.${NC}"
echo -e "${BLUE}We'll launch a full ecosystem and explore different layout algorithms!${NC}"
echo ""

# Check prerequisites
if ! curl -s http://localhost:3000/health >/dev/null 2>&1; then
    echo -e "${RED}❌ BiomeOS not running${NC}"
    echo -e "${YELLOW}   Run: cd ../00-setup && ./demo.sh${NC}"
    exit 1
fi

echo -e "${GREEN}✅ BiomeOS is running${NC}"
echo ""

# Clean slate
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Phase 1: Full Ecosystem Launch${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

echo -e "${YELLOW}🧹 Ensuring clean slate...${NC}"
"$SCRIPT_DIR/cleanup.sh"
sleep 2

echo -e "${GREEN}✅ Starting with empty graph${NC}"
echo -e "${YELLOW}   Check petalTongue: Should show 0 nodes, 0 edges${NC}"
echo ""
echo -e "${YELLOW}Press Enter to launch full ecosystem...${NC}"
read -r

echo ""
echo -e "${CYAN}🚀 Launching Full Ecosystem...${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

"$SCRIPT_DIR/launch-ecosystem.sh"

echo ""
echo -e "${YELLOW}⏳ Waiting for full discovery (20 seconds)...${NC}"
echo -e "${YELLOW}   Watch petalTongue - nodes should appear one by one!${NC}"
echo ""

# Countdown with dynamic status
for i in {20..1}; do
    # Check current primal count
    PRIMAL_COUNT=$(curl -s http://localhost:3000/api/v1/primals 2>/dev/null | jq '. | length' 2>/dev/null || echo "?")
    echo -ne "${YELLOW}   $i seconds remaining... (Discovered: $PRIMAL_COUNT primals)${NC}\r"
    sleep 1
done
echo ""

# Final count
FINAL_COUNT=$(curl -s http://localhost:3000/api/v1/primals 2>/dev/null | jq '. | length' 2>/dev/null || echo "unknown")
EDGE_COUNT=$(curl -s http://localhost:3000/api/v1/topology 2>/dev/null | jq '. | length' 2>/dev/null || echo "unknown")

echo ""
echo -e "${GREEN}✅ Full ecosystem launched!${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "   ${GREEN}Nodes:${NC} $FINAL_COUNT"
echo -e "   ${GREEN}Edges:${NC} $EDGE_COUNT"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

echo -e "${YELLOW}Check petalTongue now!${NC}"
echo -e "${YELLOW}You should see the full ecosystem topology.${NC}"
echo ""
echo -e "${YELLOW}Press Enter to proceed to layout comparison...${NC}"
read -r

# Layout comparison
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Phase 2: Layout Algorithm Comparison${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

layouts=("Force-Directed" "Hierarchical" "Circular" "Random")
descriptions=(
    "Natural clustering, nodes repel like magnets"
    "Layered architecture, shows dependencies"
    "Symmetrical circle, minimal edge crossings"
    "Random positions, useful baseline"
)
best_for=(
    "General-purpose, organic topology"
    "Service dependencies, data pipelines"
    "Small ecosystems, demos, symmetry"
    "Debugging, stress testing"
)

for i in "${!layouts[@]}"; do
    layout="${layouts[$i]}"
    desc="${descriptions[$i]}"
    best="${best_for[$i]}"
    
    echo -e "${CYAN}──────────────────────────────────────────────────────────────────────${NC}"
    echo -e "${CYAN}Layout $((i + 1))/4: ${layout}${NC}"
    echo -e "${CYAN}──────────────────────────────────────────────────────────────────────${NC}"
    echo ""
    echo -e "${YELLOW}Description:${NC} $desc"
    echo -e "${YELLOW}Best for:${NC} $best"
    echo ""
    echo -e "${YELLOW}📋 Instructions:${NC}"
    echo -e "   1. Switch to '${layout}' in petalTongue layout dropdown"
    echo -e "   2. Observe how nodes are positioned"
    echo -e "   3. Notice edge patterns"
    echo -e "   4. Try pan/zoom to explore"
    echo ""
    
    if [ $i -lt $((${#layouts[@]} - 1)) ]; then
        echo -e "${YELLOW}Press Enter when ready for next layout...${NC}"
        read -r
        echo ""
    fi
done

echo ""
echo -e "${YELLOW}Which layout made the topology clearest?${NC}"
echo -e "${YELLOW}This is subjective and depends on your ecosystem!${NC}"
echo ""
echo -e "${YELLOW}Press Enter to proceed to navigation test...${NC}"
read -r

# Navigation
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Phase 3: Graph Navigation${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

echo -e "${YELLOW}Let's test graph navigation controls:${NC}"
echo ""
echo -e "${CYAN}1. Pan (Move Camera)${NC}"
echo -e "   Action: Click and drag on empty space"
echo -e "   Result: Entire graph moves"
echo ""
echo -e "${CYAN}2. Zoom${NC}"
echo -e "   Action: Mouse wheel up/down"
echo -e "   Result: Graph scales in/out"
echo ""
echo -e "${CYAN}3. Select Node${NC}"
echo -e "   Action: Click on a node"
echo -e "   Result: Right panel shows node details"
echo ""
echo -e "${CYAN}4. Reset View${NC}"
echo -e "   Action: Click 'Reset Camera' button"
echo -e "   Result: Camera returns to origin"
echo ""

echo -e "${YELLOW}Try each of these now!${NC}"
echo ""
echo -e "${YELLOW}Press Enter when done exploring...${NC}"
read -r

# Topology patterns
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Phase 4: Topology Pattern Identification${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

echo -e "${YELLOW}Common topology patterns to look for:${NC}"
echo ""

echo -e "${CYAN}1. Hub-and-Spoke${NC}"
echo -e "   Pattern: One central node, many peripherals"
echo -e "   Example: Songbird (discovery hub) connecting all services"
echo -e "   Look for: High-degree node in center"
echo ""

echo -e "${CYAN}2. Mesh${NC}"
echo -e "   Pattern: Many nodes, many connections"
echo -e "   Example: Peer-to-peer coordination"
echo -e "   Look for: Dense connectivity"
echo ""

echo -e "${CYAN}3. Layered${NC}"
echo -e "   Pattern: Clear separation into tiers"
echo -e "   Example: Security → Core → Extensions"
echo -e "   Look for: Horizontal levels (best in Hierarchical layout)"
echo ""

echo -e "${YELLOW}What patterns do you see in your topology?${NC}"
echo -e "${YELLOW}Try the Hierarchical layout to make patterns clearer!${NC}"
echo ""
echo -e "${YELLOW}Press Enter to finish demo...${NC}"
read -r

# Summary
echo ""
echo -e "${GREEN}╔══════════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                  ✅ Topology Visualization Complete!                         ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}✅ You've visualized a full multi-primal ecosystem${NC}"
echo -e "${GREEN}✅ You understand different layout algorithms${NC}"
echo -e "${GREEN}✅ You've tested graph navigation${NC}"
echo -e "${GREEN}✅ You can identify topology patterns${NC}"
echo ""
echo -e "${YELLOW}📖 Key Learnings:${NC}"
echo -e "  • Layout choice matters - different layouts reveal different insights"
echo -e "  • Force-Directed: General purpose, natural clustering"
echo -e "  • Hierarchical: Shows dependencies and data flow"
echo -e "  • Circular: Symmetry, good for small ecosystems"
echo -e "  • Topology patterns (hub, mesh, layered) have implications"
echo ""
echo -e "${YELLOW}🔍 Did you notice any gaps?${NC}"
echo -e "  • Layout quality with this topology?"
echo -e "  • Performance with $FINAL_COUNT nodes?"
echo -e "  • Navigation smoothness?"
echo -e "  • Visual clarity at different zoom levels?"
echo -e "  • Document in: ../GAPS.md"
echo ""
echo -e "${YELLOW}🧹 Cleanup:${NC}"
echo -e "  ./cleanup.sh  # Stop all primals"
echo ""
echo -e "${YELLOW}📚 Next Steps:${NC}"
echo -e "  cd ../04-health-monitoring/"
echo -e "  cat README.md"
echo ""
echo -e "${PINK}🌱 Topology visualization mastered! On to health monitoring! 🌸${NC}"

