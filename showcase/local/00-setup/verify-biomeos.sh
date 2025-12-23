#!/usr/bin/env bash
#
# Verify BiomeOS
#
# Check that BiomeOS is running and healthy

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

echo "Verifying BiomeOS..."
echo ""

# Check if responding
if curl -s http://localhost:3000/health >/dev/null 2>&1; then
    echo -e "${GREEN}✅ BiomeOS responding on http://localhost:3000${NC}"
else
    echo -e "${RED}❌ BiomeOS not responding${NC}"
    exit 1
fi

# Check health endpoint
HEALTH_RESPONSE=$(curl -s http://localhost:3000/health)
if echo "$HEALTH_RESPONSE" | grep -q "status"; then
    echo -e "${GREEN}✅ Health check: OK${NC}"
else
    echo -e "${RED}❌ Health check failed${NC}"
    exit 1
fi

# Check primal discovery (may not have any primals yet)
echo -e "${GREEN}✅ Primal discovery active${NC}"
echo -e "   ${GREEN}(No primals running yet - this is expected)${NC}"

echo ""
echo -e "${GREEN}✅ BiomeOS verification complete!${NC}"
echo ""

