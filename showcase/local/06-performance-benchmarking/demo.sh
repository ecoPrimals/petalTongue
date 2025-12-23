#!/usr/bin/env bash
#
# 06-performance-benchmarking: Progressive Stress Testing
#
# Purpose: Test performance at scale, measure limits
# Duration: ~30-45 minutes

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PINK='\033[1;35m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${PINK}╔══════════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${PINK}║                  🌸 Performance Benchmarking                                 ║${NC}"
echo -e "${PINK}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}This demo stress tests petalTongue to find performance limits.${NC}"
echo -e "${BLUE}We'll progressively increase node count and measure FPS, CPU, memory!${NC}"
echo ""

echo -e "${YELLOW}📋 Prerequisites:${NC}"
echo -e "   • System monitor running (htop / Activity Monitor / Task Manager)"
echo -e "   • petalTongue UI open"
echo -e "   • Mock server running with performance scenarios"
echo ""
echo -e "${YELLOW}Press Enter to begin benchmarking...${NC}"
read -r

# Phase 1: Baseline
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Phase 1: Baseline (10 Nodes)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

echo -e "${CYAN}📊 Switch to 'simple.json' scenario (10 nodes)${NC}"
echo -e "${YELLOW}   In petalTongue: Use sandbox/scenarios/simple.json${NC}"
echo ""
echo -e "${YELLOW}Measure:${NC}"
echo -e "  • FPS: Should be 60 (check UI or dev tools)"
echo -e "  • CPU: Should be <20% (check system monitor)"
echo -e "  • Memory: Note baseline usage"
echo -e "  • Feel: Silky smooth?"
echo ""
echo -e "${YELLOW}Press Enter after observing 10-node performance...${NC}"
read -r

# Phase 2: Medium
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Phase 2: Medium Scale (50 Nodes)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

echo -e "${CYAN}📊 Switch to 'performance.json' scenario (50 nodes)${NC}"
echo -e "${YELLOW}   In petalTongue: Use sandbox/scenarios/performance.json${NC}"
echo ""
echo -e "${YELLOW}Measure:${NC}"
echo -e "  • FPS: Still 60? Or dropped?"
echo -e "  • CPU: Increased to 20-40%?"
echo -e "  • Memory: How much growth?"
echo -e "  • Feel: Still smooth?"
echo ""
echo -e "${YELLOW}Press Enter after observing 50-node performance...${NC}"
read -r

# Phase 3: Large
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Phase 3: Large Scale (100+ Nodes)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

echo -e "${CYAN}📊 For 100+ nodes, you'll need to create a custom scenario${NC}"
echo -e "${YELLOW}   Or modify performance.json to have more primals${NC}"
echo ""
echo -e "${YELLOW}Measure:${NC}"
echo -e "  • FPS: Dropped below 30?"
echo -e "  • CPU: High (40-60%)?"
echo -e "  • Memory: Significant increase?"
echo -e "  • Feel: Noticeable lag?"
echo ""
echo -e "${YELLOW}Press Enter after observing large-scale performance...${NC}"
read -r

# Phase 4: Layout Comparison
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Phase 4: Layout Algorithm Benchmarking${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

echo -e "${CYAN}🔀 Switch between layouts and time each:${NC}"
echo ""

for layout in "Force-Directed" "Hierarchical" "Circular" "Random"; do
    echo -e "${YELLOW}Test: $layout${NC}"
    echo -e "  1. Switch to $layout in UI"
    echo -e "  2. Note how long it takes to stabilize"
    echo -e "  3. Observe CPU spike"
    echo ""
    echo -e "${YELLOW}Press Enter for next layout...${NC}"
    read -r
done

echo -e "${YELLOW}Which layout was fastest? Which was prettiest?${NC}"
echo ""
echo -e "${YELLOW}Press Enter to proceed to summary...${NC}"
read -r

# Summary
echo ""
echo -e "${GREEN}╔══════════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                  ✅ Performance Benchmarking Complete!                       ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}✅ You've tested baseline performance (10 nodes)${NC}"
echo -e "${GREEN}✅ You've tested medium scale (50 nodes)${NC}"
echo -e "${GREEN}✅ You've tested large scale (100+ nodes)${NC}"
echo -e "${GREEN}✅ You've compared layout algorithms${NC}"
echo ""
echo -e "${YELLOW}📖 Key Learnings:${NC}"
echo -e "  • Performance degrades with node count (expected)"
echo -e "  • Hierarchical layout typically fastest"
echo -e "  • Force-directed slowest but prettiest"
echo -e "  • Memory grows linearly with nodes"
echo-e "  • CPU spikes during layout computation"
echo ""
echo -e "${YELLOW}🔍 Document your findings:${NC}"
echo -e "  • What's the practical node limit for 60 FPS?"
echo -e "  • What's acceptable for 30 FPS?"
echo -e "  • At what point is it unusable?"
echo -e "  • Any memory leaks noticed?"
echo -e "  • Which layout is best for production?"
echo -e "  • Document in: ../GAPS.md"
echo ""
echo -e "${YELLOW}📚 Next Steps:${NC}"
echo -e "  cd ../07-real-world-scenarios/"
echo -e "  cat README.md"
echo ""
echo -e "${PINK}🌱 Performance limits known! On to real-world scenarios! 🌸${NC}"

