#!/usr/bin/env bash
#
# 07-real-world-scenarios: Production-Like Operational Testing
#
# Purpose: Test petalTongue with realistic operational scenarios
# Duration: ~45-60 minutes

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
echo -e "${PINK}║                   🌸 Real-World Operational Scenarios                        ║${NC}"
echo -e "${PINK}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BLUE}This demo simulates production operational scenarios.${NC}"
echo -e "${BLUE}We'll test deployments, incidents, maintenance, and more!${NC}"
echo ""

echo -e "${YELLOW}📋 Prerequisites:${NC}"
echo -e "   • Actual primal binaries available"
echo -e "   • BiomeOS running"
echo -e "   • petalTongue UI open"
echo -e "   • ~1 hour of uninterrupted time"
echo ""
echo -e "${YELLOW}Press Enter to begin real-world testing...${NC}"
read -r

# Scenario 1: New Service Deployment
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Scenario 1: New Service Deployment${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

echo -e "${CYAN}🚀 Situation: Deploy a new primal to production${NC}"
echo ""
echo -e "${YELLOW}Workflow:${NC}"
echo -e "  1. Start with healthy ecosystem"
echo -e "  2. Deploy new primal"
echo -e "  3. Watch discovery process"
echo -e "  4. Validate connections form"
echo -e "  5. Confirm ecosystem stabilizes"
echo ""
echo -e "${YELLOW}This scenario tests: Discovery, connection formation, layout adaptation${NC}"
echo ""
echo -e "${YELLOW}Press Enter to execute deployment...${NC}"
read -r

echo ""
echo -e "${CYAN}📝 Scenario 1 is a guided manual test.${NC}"
echo -e "${YELLOW}   Follow the instructions in README.md${NC}"
echo -e "${YELLOW}   Document observations in ../GAPS.md${NC}"
echo ""
echo -e "${YELLOW}Press Enter when scenario 1 complete...${NC}"
read -r

# Scenario 2: Incident Response
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Scenario 2: Incident Response${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

echo -e "${CYAN}🚨 Situation: A primal crashes and must be detected${NC}"
echo ""
echo -e "${YELLOW}Workflow:${NC}"
echo -e "  1. Healthy ecosystem baseline"
echo -e "  2. Kill a primal process (simulate crash)"
echo -e "  3. Observe failure detection time"
echo -e "  4. Check for cascade effects"
echo -e "  5. Restart primal"
echo -e "  6. Validate recovery"
echo ""
echo -e "${YELLOW}This scenario tests: Failure detection, cascading effects, recovery${NC}"
echo ""
echo -e "${YELLOW}Press Enter to simulate incident...${NC}"
read -r

echo ""
echo -e "${CYAN}📝 Scenario 2 is a guided manual test.${NC}"
echo -e "${YELLOW}   Kill a primal: kill -9 \$(pgrep beardog)${NC}"
echo -e "${YELLOW}   Watch petalTongue for detection${NC}"
echo -e "${YELLOW}   Restart: cd biomeOS/bin/primals && ./beardog &${NC}"
echo -e "${YELLOW}   Document detection time in ../GAPS.md${NC}"
echo ""
echo -e "${YELLOW}Press Enter when scenario 2 complete...${NC}"
read -r

# Scenario 3: Planned Maintenance
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Scenario 3: Planned Maintenance (Rolling Restart)${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

echo -e "${CYAN}🔧 Situation: Perform zero-downtime upgrades${NC}"
echo ""
echo -e "${YELLOW}Workflow:${NC}"
echo -e "  1. Healthy ecosystem"
echo -e "  2. For each primal:"
echo -e "     a. Drain traffic (if supported)"
echo -e "     b. Stop primal"
echo -e "     c. Upgrade"
echo -e "     d. Start primal"
echo -e "     e. Wait for health"
echo -e "  3. Validate zero downtime"
echo ""
echo -e "${YELLOW}This scenario tests: Maintenance workflows, brief disruptions${NC}"
echo ""
echo -e "${YELLOW}Press Enter to begin maintenance...${NC}"
read -r

echo ""
echo -e "${CYAN}📝 Scenario 3 is a guided manual test.${NC}"
echo -e "${YELLOW}   Restart each primal one by one${NC}"
echo -e "${YELLOW}   Watch for brief disappearance + return${NC}"
echo -e "${YELLOW}   Validate other primals stay healthy${NC}"
echo ""
echo -e "${YELLOW}Press Enter when scenario 3 complete...${NC}"
read -r

# Summary
echo ""
echo -e "${GREEN}╔══════════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                 ✅ Real-World Scenarios Complete!                            ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}✅ You've tested new service deployment${NC}"
echo -e "${GREEN}✅ You've simulated incident response${NC}"
echo -e "${GREEN}✅ You've performed planned maintenance${NC}"
echo ""
echo -e "${YELLOW}📖 Key Learnings:${NC}"
echo -e "  • Real primals behave differently than mocks"
echo -e "  • Timing matters (detection latency, recovery time)"
echo -e "  • Operational workflows need smooth UX"
echo -e "  • Cascading effects are important to visualize"
echo ""
echo -e "${YELLOW}🔍 Critical Questions:${NC}"
echo -e "  • Would you trust petalTongue in production?"
echo -e "  • What gaps did real scenarios reveal?"
echo -e "  • Which workflows were smooth? Which were rough?"
echo -e "  • Document ALL in: ../GAPS.md"
echo ""
echo -e "${YELLOW}📚 Next Steps:${NC}"
echo -e "  cd ../08-integration-testing/"
echo -e "  cat README.md"
echo ""
echo -e "${PINK}🌱 Real-world validation complete! Final integration next! 🌸${NC}"

