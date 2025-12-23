#!/usr/bin/env bash
#
# Trigger Warning State
#
# Simulate a WARNING health state on a primal
# NOTE: This is a simulation script. In production, health states are
# determined by actual primal metrics.

set -euo pipefail

# Colors
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

if [ $# -eq 0 ]; then
    echo -e "${RED}Error: Primal name required${NC}"
    echo "Usage: ./trigger-warning.sh <primal-name>"
    echo "Example: ./trigger-warning.sh beardog"
    exit 1
fi

PRIMAL="$1"

echo -e "${YELLOW}⚠️  Simulating WARNING state for $PRIMAL${NC}"
echo ""
echo -e "${YELLOW}📝 NOTE:${NC}"
echo -e "   In production, health states are determined by actual primal metrics:"
echo -e "   • CPU usage, memory, disk space"
echo -e "   • Request latency, error rates"
echo -e "   • Dependency health"
echo ""
echo -e "   For this demo, we would typically:"
echo -e "   1. Send SIGUSR1 to primal (if it supports health simulation)"
echo -e "   2. Call primal's admin API: POST /admin/health/warning"
echo -e "   3. Inject load to trigger real degradation"
echo ""
echo -e "${YELLOW}   Since primals may not have health simulation yet, this is a placeholder.${NC}"
echo -e "${YELLOW}   In a real deployment, health would degrade naturally under load.${NC}"
echo ""
echo -e "${YELLOW}   For fermentation purposes, manually verify petalTongue handles WARNING states.${NC}"

