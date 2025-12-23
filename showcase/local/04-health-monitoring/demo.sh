#!/usr/bin/env bash
#
# 04-health-monitoring: Dynamic Health Status Demo
#
# Purpose: Observe health degradation and recovery
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
echo -e "${PINK}║                   🌸 Health Monitoring Demo                                  ║${NC}"
echo -e "${PINK}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}This demo shows how petalTongue visualizes health state changes.${NC}"
echo -e "${BLUE}We'll trigger warnings, critical failures, and recoveries!${NC}"
echo ""

# Check prerequisites
if ! curl -s http://localhost:3000/health >/dev/null 2>&1; then
    echo -e "${RED}❌ BiomeOS not running${NC}"
    echo -e "${YELLOW}   Run: cd ../00-setup && ./demo.sh${NC}"
    exit 1
fi

echo -e "${GREEN}✅ BiomeOS is running${NC}"
echo ""

# Phase 1: Healthy Baseline
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Phase 1: Healthy Baseline${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

echo -e "${YELLOW}🧹 Ensuring clean slate...${NC}"
"$SCRIPT_DIR/cleanup.sh"
sleep 2

echo -e "${CYAN}🚀 Launching healthy ecosystem...${NC}"
"$SCRIPT_DIR/launch-healthy.sh"

echo ""
echo -e "${YELLOW}⏳ Waiting for discovery (15 seconds)...${NC}"
for i in {15..1}; do
    echo -ne "${YELLOW}   $i...${NC}\r"
    sleep 1
done
echo ""

echo -e "${GREEN}✅ Baseline established!${NC}"
echo -e "${YELLOW}   Check petalTongue: All nodes should be GREEN (healthy)${NC}"
echo ""
echo -e "${YELLOW}Press Enter to trigger first warning...${NC}"
read -r

# Phase 2: Warning State
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Phase 2: Warning State (Degraded)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

echo -e "${YELLOW}⚠️  Triggering WARNING state on BearDog...${NC}"
"$SCRIPT_DIR/trigger-warning.sh" beardog

echo ""
echo -e "${YELLOW}⏳ Wait for state change to be detected (10 seconds)...${NC}"
for i in {10..1}; do
    echo -ne "${YELLOW}   $i...${NC}\r"
    sleep 1
done
echo ""

echo -e "${YELLOW}📊 Expected observation:${NC}"
echo -e "   • BearDog node turns YELLOW"
echo -e "   • Warning indicator appears"
echo -e "   • Audio: Slightly off-pitch (if enabled)"
echo -e "   • Stats: Health drops to ~70-80%"
echo ""
echo -e "${YELLOW}❓ Can you notice the change immediately?${NC}"
echo ""
echo -e "${YELLOW}Press Enter to escalate to critical...${NC}"
read -r

# Phase 3: Critical State
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Phase 3: Critical State (Failed)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

echo -e "${RED}🚨 Triggering CRITICAL state on BearDog...${NC}"
"$SCRIPT_DIR/trigger-critical.sh" beardog

echo ""
echo -e "${YELLOW}⏳ Wait for critical alert (10 seconds)...${NC}"
for i in {10..1}; do
    echo -ne "${YELLOW}   $i...${NC}\r"
    sleep 1
done
echo ""

echo -e "${YELLOW}📊 Expected observation:${NC}"
echo -e "   • BearDog node turns RED"
echo -e "   • Critical alert appears"
echo -e "   • Audio: Dissonant, urgent tone"
echo -e "   • Stats: Health drops to ~20-30%"
echo ""
echo -e "${YELLOW}❓ Does this feel urgent enough?${NC}"
echo ""
echo -e "${YELLOW}Press Enter to create cascading failure...${NC}"
read -r

# Phase 4: Multiple Failures
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Phase 4: Cascading Failure${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

echo -e "${RED}🚨 Creating system-wide degradation...${NC}"
echo ""

echo -e "${RED}   1/2: NestGate going CRITICAL...${NC}"
"$SCRIPT_DIR/trigger-critical.sh" nestgate
sleep 3

# Only trigger on primals that exist
if pgrep -f "songbird" > /dev/null 2>&1; then
    echo -e "${YELLOW}   2/2: Songbird going WARNING...${NC}"
    "$SCRIPT_DIR/trigger-warning.sh" songbird
else
    echo -e "${YELLOW}   (Songbird not running, skipping)${NC}"
fi

echo ""
echo -e "${YELLOW}⏳ Observing cascading failure (15 seconds)...${NC}"
for i in {15..1}; do
    echo -ne "${YELLOW}   $i...${NC}\r"
    sleep 1
done
echo ""

echo -e "${YELLOW}📊 Expected observation:${NC}"
echo -e "   • Multiple RED and YELLOW nodes"
echo -e "   • System-wide health % drops significantly"
echo -e "   • Complex audio dissonance"
echo -e "   • Visual chaos but still understandable"
echo ""
echo -e "${YELLOW}❓ Can you still identify which node to fix first?${NC}"
echo -e "${YELLOW}❓ (Hint: RED nodes are higher priority than YELLOW)${NC}"
echo ""
echo -e "${YELLOW}Press Enter to begin recovery...${NC}"
read -r

# Phase 5: Recovery
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Phase 5: System Recovery${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

echo -e "${GREEN}♻️  Restoring services to health...${NC}"
echo ""

echo -e "${GREEN}   1/3: Restoring BearDog...${NC}"
"$SCRIPT_DIR/restore-health.sh" beardog
sleep 8

echo -e "${GREEN}   2/3: Restoring NestGate...${NC}"
"$SCRIPT_DIR/restore-health.sh" nestgate
sleep 8

if pgrep -f "songbird" > /dev/null 2>&1; then
    echo -e "${GREEN}   3/3: Restoring Songbird...${NC}"
    "$SCRIPT_DIR/restore-health.sh" songbird
    sleep 8
fi

echo ""
echo -e "${GREEN}✅ All services restored!${NC}"
echo -e "${YELLOW}   Check petalTongue: All nodes should return to GREEN${NC}"
echo ""
echo -e "${YELLOW}Press Enter to finish demo...${NC}"
read -r

# Summary
echo ""
echo -e "${GREEN}╔══════════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                    ✅ Health Monitoring Complete!                            ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}✅ You've observed health degradation (green → yellow → red)${NC}"
echo -e "${GREEN}✅ You've seen cascading failures${NC}"
echo -e "${GREEN}✅ You've validated recovery detection${NC}"
echo -e "${GREEN}✅ You understand health as operational awareness${NC}"
echo ""
echo -e "${YELLOW}📖 Key Learnings:${NC}"
echo -e "  • Health states are visually distinct (green/yellow/red)"
echo -e "  • Detection latency is 5-10 seconds (acceptable for monitoring)"
echo -e "  • Audio complements visual (accessibility)"
echo -e "  • Multiple failures are understandable but urgent"
echo -e "  • Recovery is as visible as degradation"
echo ""
echo -e "${YELLOW}🔍 Did you notice any gaps?${NC}"
echo -e "  • Color distinction clear enough?"
echo -e "  • Audio feedback helpful?"
echo -e "  • Detection speed acceptable?"
echo -e "  • Multiple failures manageable?"
echo -e "  • Recovery transitions smooth?"
echo -e "  • Document in: ../GAPS.md"
echo ""
echo -e "${YELLOW}🧹 Cleanup:${NC}"
echo -e "  ./cleanup.sh  # Stop all primals"
echo ""
echo -e "${YELLOW}📚 Next Steps:${NC}"
echo -e "  cd ../05-accessibility-validation/"
echo -e "  cat README.md"
echo ""
echo -e "${PINK}🌱 Health monitoring validated! On to accessibility! 🌸${NC}"

