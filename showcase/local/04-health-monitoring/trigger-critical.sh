#!/usr/bin/env bash
#
# Trigger Critical State
#
# Simulate a CRITICAL health state on a primal

set -euo pipefail

# Colors
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

if [ $# -eq 0 ]; then
    echo -e "${RED}Error: Primal name required${NC}"
    echo "Usage: ./trigger-critical.sh <primal-name>"
    echo "Example: ./trigger-critical.sh beardog"
    exit 1
fi

PRIMAL="$1"

echo -e "${RED}🚨 Simulating CRITICAL state for $PRIMAL${NC}"
echo ""
echo -e "${YELLOW}📝 NOTE:${NC}"
echo -e "   In production, critical states indicate:"
echo -e "   • Service is non-functional"
echo -e "   • Severe resource exhaustion"
echo -e "   • Unrecoverable errors"
echo ""
echo -e "   For this demo, we would typically:"
echo -e "   1. Send SIGUSR2 to primal (if it supports health simulation)"
echo -e "   2. Call primal's admin API: POST /admin/health/critical"
echo -e "   3. Kill dependent services to trigger cascade"
echo ""
echo -e "${YELLOW}   Since primals may not have health simulation yet, this is a placeholder.${NC}"
echo -e "${YELLOW}   In a real deployment, critical states occur from actual failures.${NC}"
echo ""
echo -e "${YELLOW}   For fermentation purposes, manually verify petalTongue handles CRITICAL states.${NC}"

